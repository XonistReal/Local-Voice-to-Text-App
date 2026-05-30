mod audio;
mod config;
mod inject;
mod models;
mod perf;
mod transcribe;
mod vad;

use audio::AudioCapture;
use config::{append_history, load_config, save_config, AppConfig, TranscriptEntry};
use models::{catalog, delete_model, download_model, ensure_model_on_disk, list_installed, InstalledModel, ModelCatalogEntry};
use parking_lot::Mutex;
use perf::{detect_hardware, HardwareInfo};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use transcribe::WhisperEngine;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppStatus {
    Idle,
    Recording,
    Transcribing,
    LoadingModel,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub status: AppStatus,
    pub message: Option<String>,
    pub last_latency_ms: Option<u64>,
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub audio: Mutex<AudioCapture>,
    pub whisper: Mutex<Option<WhisperEngine>>,
    pub status: Mutex<StatusPayload>,
    pub shortcut_id: Mutex<Option<Shortcut>>,
}

impl AppState {
    fn new() -> Self {
        let config = load_config();
        let audio = AudioCapture::new(config.max_recording_secs);
        Self {
            config: Mutex::new(config),
            audio: Mutex::new(audio),
            whisper: Mutex::new(None),
            status: Mutex::new(StatusPayload {
                status: AppStatus::Idle,
                message: None,
                last_latency_ms: None,
            }),
            shortcut_id: Mutex::new(None),
        }
    }
}

fn set_status(state: &AppState, status: AppStatus, message: Option<String>) {
    let mut s = state.status.lock();
    s.status = status;
    s.message = message;
}

fn emit_status(app: &AppHandle, state: &AppState) {
    let payload = state.status.lock().clone();
    let _ = app.emit("status-changed", payload.clone());
    update_tray_tooltip(app, &payload);
    sync_overlay(app, &payload);
}

fn update_tray_tooltip(app: &AppHandle, payload: &StatusPayload) {
    if let Some(tray) = app.tray_by_id("main") {
        let label = match payload.status {
            AppStatus::Idle => "VTT — Ready",
            AppStatus::Recording => "VTT — Recording…",
            AppStatus::Transcribing => "VTT — Transcribing…",
            AppStatus::LoadingModel => "VTT — Loading model…",
            AppStatus::Error => "VTT — Error",
        };
        let _ = tray.set_tooltip(Some(label));
    }
}

fn sync_overlay(app: &AppHandle, payload: &StatusPayload) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        match payload.status {
            AppStatus::Recording | AppStatus::Transcribing => {
                let _ = overlay.show();
                let _ = overlay.emit("overlay-state", payload);
            }
            _ => {
                let _ = overlay.hide();
            }
        }
    }
}

fn parse_hotkey(hotkey: &str) -> Result<Shortcut, String> {
    hotkey
        .parse::<Shortcut>()
        .map_err(|e| format!("Invalid hotkey: {e}"))
}

fn register_hotkey(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let hotkey = state.config.lock().hotkey.clone();
    let shortcut = parse_hotkey(&hotkey)?;

    if let Some(id) = state.shortcut_id.lock().take() {
        app.global_shortcut().unregister(id).ok();
    }

    let app_handle = app.clone();
    let handler_app = app_handle.clone();

    app_handle
        .global_shortcut()
        .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
            let state = handler_app.state::<AppState>();
            match event.state {
                ShortcutState::Pressed => {
                    if state.status.lock().status == AppStatus::Recording {
                        return;
                    }
                    if let Err(e) = start_recording_internal(&handler_app, state.inner()) {
                        set_status(state.inner(), AppStatus::Error, Some(e));
                        emit_status(&handler_app, state.inner());
                    }
                }
                ShortcutState::Released => {
                    if state.status.lock().status != AppStatus::Recording {
                        return;
                    }
                    let app_clone = handler_app.clone();
                    std::thread::spawn(move || {
                        let state = app_clone.state::<AppState>();
                        if let Err(e) = stop_and_transcribe_internal(&app_clone, state.inner()) {
                            set_status(state.inner(), AppStatus::Error, Some(e));
                            emit_status(&app_clone, state.inner());
                        }
                    });
                }
            }
        })
        .map_err(|e| e.to_string())?;

    *state.shortcut_id.lock() = Some(shortcut);
    Ok(())
}

fn ensure_whisper_loaded(app: &AppHandle, state: &AppState) -> Result<(), String> {
    if state.whisper.lock().is_some() {
        return Ok(());
    }

    set_status(state, AppStatus::LoadingModel, Some("Loading speech model…".into()));
    emit_status(app, state);

    let config = state.config.lock().clone();
    let path = ensure_model_on_disk(&config.model_id)?;
    let engine = WhisperEngine::load(&path, config.perf_profile)
        .map_err(|e| format!("{e}"))?;

    *state.whisper.lock() = Some(engine);
    set_status(state, AppStatus::Idle, None);
    emit_status(app, state);
    Ok(())
}

fn start_recording_internal(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let config = state.config.lock().clone();
    state
        .audio
        .lock()
        .set_max_duration(config.max_recording_secs);
    state.audio.lock().start().map_err(|e| e.to_string())?;
    set_status(state, AppStatus::Recording, None);
    emit_status(app, state);
    Ok(())
}

fn stop_and_transcribe_internal(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let config = state.config.lock().clone();
    let samples = state.audio.lock().stop().map_err(|e| e.to_string())?;

    set_status(state, AppStatus::Transcribing, None);
    emit_status(app, state);

    if config.silence_skip && !vad::has_speech(&samples, audio::TARGET_SAMPLE_RATE) {
        set_status(state, AppStatus::Idle, Some("Silence detected — skipped".into()));
        emit_status(app, state);
        return Ok(());
    }

    let trimmed = if config.silence_skip {
        vad::trim_silence(&samples, audio::TARGET_SAMPLE_RATE)
    } else {
        samples
    };

    if trimmed.is_empty() {
        set_status(state, AppStatus::Idle, Some("No speech detected".into()));
        emit_status(app, state);
        return Ok(());
    }

    ensure_whisper_loaded(app, state)?;

    let language = if config.language == "auto" {
        None
    } else {
        Some(config.language.as_str())
    };

    let start = Instant::now();
    let text = {
        let whisper = state.whisper.lock();
        let engine = whisper.as_ref().ok_or("Model not loaded")?;
        engine
            .transcribe(&trimmed, language, config.perf_profile)
            .map_err(|e| e.to_string())?
    };
    let latency_ms = start.elapsed().as_millis() as u64;

    if !text.is_empty() {
        inject::inject_text(&text, config.paste_mode, config.inject_delay_ms)
            .map_err(|e| e.to_string())?;
        let entry = append_history(&text, latency_ms)?;
        let _ = app.emit("transcript-added", entry);
    }

    if config.unload_when_idle {
        *state.whisper.lock() = None;
    }

    {
        let mut s = state.status.lock();
        s.status = AppStatus::Idle;
        s.message = None;
        s.last_latency_ms = Some(latency_ms);
    }
    emit_status(app, state);

    Ok(())
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> AppConfig {
    state.config.lock().clone()
}

#[tauri::command]
fn save_app_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let hotkey_changed = state.config.lock().hotkey != config.hotkey;
    state.audio.lock().set_max_duration(config.max_recording_secs);
    save_config(&config)?;
    *state.config.lock() = config.clone();

    if hotkey_changed {
        register_hotkey(&app, &state)?;
    }

    if config.unload_when_idle {
        *state.whisper.lock() = None;
    } else if config.preload_model && models::is_installed(&config.model_id) {
        let _ = ensure_whisper_loaded(&app, &state);
    }

    Ok(())
}

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> StatusPayload {
    state.status.lock().clone()
}

#[tauri::command]
fn detect_hardware_info() -> HardwareInfo {
    detect_hardware()
}

#[tauri::command]
fn get_model_catalog() -> Vec<ModelCatalogEntry> {
    catalog()
}

#[tauri::command]
fn get_installed_models() -> Vec<InstalledModel> {
    list_installed()
}

#[tauri::command]
async fn download_model_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    model_id: String,
) -> Result<String, String> {
    let app_emit = app.clone();
    let model_id_for_progress = model_id.clone();
    let path = download_model(&model_id, move |downloaded, total| {
        let _ = app_emit.emit(
            "model-download-progress",
            serde_json::json!({
                "modelId": model_id_for_progress,
                "downloaded": downloaded,
                "total": total,
                "percent": if total > 0 { (downloaded as f64 / total as f64 * 100.0) as u32 } else { 0 }
            }),
        );
    })
    .await?;

    *state.whisper.lock() = None;
    let mut cfg = state.config.lock();
    cfg.model_id = model_id;
    save_config(&cfg)?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn delete_model_cmd(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    delete_model(&model_id)?;
    if state.config.lock().model_id == model_id {
        *state.whisper.lock() = None;
    }
    Ok(())
}

#[tauri::command]
fn get_history() -> Vec<TranscriptEntry> {
    config::load_history()
}

#[tauri::command]
fn clear_history() -> Result<(), String> {
    config::save_history(&[])
}

#[tauri::command]
fn complete_onboarding(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock();
    cfg.onboarding_complete = true;
    save_config(&cfg)?;
    register_hotkey(&app, &state)?;
    if cfg.preload_model && models::is_installed(&cfg.model_id) {
        let _ = ensure_whisper_loaded(&app, &state);
    }
    Ok(())
}

#[tauri::command]
fn start_recording(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    start_recording_internal(&app, &state)
}

#[tauri::command]
fn stop_and_transcribe(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    stop_and_transcribe_internal(&app, &state)?;
    Ok(state
        .status
        .lock()
        .last_latency_ms
        .map(|ms| format!("Done in {ms} ms"))
        .unwrap_or_else(|| "Done".into()))
}

#[tauri::command]
fn preload_model(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    ensure_whisper_loaded(&app, &state)
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn build_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Open VTT", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &settings, &quit])?;

    let app_handle = app.clone();
    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("VTT — Ready")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                let _ = show_main_window(app.clone());
            }
            "settings" => {
                let _ = show_main_window(app.clone());
                let _ = app.emit("navigate", "/settings");
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = show_main_window(app.clone());
            }
        })
        .build(app)?;

    let _ = app_handle;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::new())
        .setup(|app| {
            build_tray(app.handle())?;

            let state = app.state::<AppState>();
            let cfg = state.config.lock().clone();
            if cfg.onboarding_complete {
                register_hotkey(app.handle(), &state)?;
                if cfg.preload_model && models::is_installed(&cfg.model_id) {
                    let app_handle = app.handle().clone();
                    std::thread::spawn(move || {
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            let _ = ensure_whisper_loaded(&app_handle, state.inner());
                        }
                    });
                }
            }

            if let Some(overlay) = app.get_webview_window("overlay") {
                let _ = overlay.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_app_config,
            get_status,
            detect_hardware_info,
            get_model_catalog,
            get_installed_models,
            download_model_cmd,
            delete_model_cmd,
            get_history,
            clear_history,
            complete_onboarding,
            start_recording,
            stop_and_transcribe,
            preload_model,
            show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
