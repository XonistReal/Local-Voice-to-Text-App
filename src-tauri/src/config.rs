use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::perf::PerfProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub perf_profile: PerfProfile,
    pub auto_detect_profile: bool,
    pub model_id: String,
    pub language: String,
    pub hotkey: String,
    pub preload_model: bool,
    pub max_recording_secs: u32,
    pub silence_skip: bool,
    pub show_latency: bool,
    pub inject_delay_ms: u32,
    pub paste_mode: bool,
    pub unload_when_idle: bool,
    pub onboarding_complete: bool,
    #[serde(default = "default_use_gpu")]
    pub use_gpu: bool,
    #[serde(default = "default_polish_transcripts")]
    pub polish_transcripts: bool,
}

fn default_polish_transcripts() -> bool {
    true
}

fn default_use_gpu() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        let profile = crate::perf::detect_profile();
        Self {
            perf_profile: profile,
            auto_detect_profile: true,
            model_id: crate::perf::default_model_id(profile).to_string(),
            language: "auto".to_string(),
            hotkey: "Ctrl+Shift+Space".to_string(),
            preload_model: profile != PerfProfile::Potato,
            max_recording_secs: crate::perf::max_recording_secs(profile),
            silence_skip: true,
            show_latency: true,
            inject_delay_ms: 50,
            paste_mode: false,
            unload_when_idle: profile == PerfProfile::Potato,
            onboarding_complete: false,
            use_gpu: true,
            polish_transcripts: true,
        }
    }
}

pub fn config_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "joerh", "vtt")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn history_path() -> PathBuf {
    config_dir().join("history.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), data).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptEntry {
    pub id: String,
    pub text: String,
    pub timestamp: i64,
    pub latency_ms: u64,
}

pub fn load_history() -> Vec<TranscriptEntry> {
    let path = history_path();
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(entries) = serde_json::from_str(&data) {
                return entries;
            }
        }
    }
    Vec::new()
}

pub fn save_history(entries: &[TranscriptEntry]) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let capped: Vec<_> = entries.iter().take(50).cloned().collect();
    let data = serde_json::to_string_pretty(&capped).map_err(|e| e.to_string())?;
    std::fs::write(history_path(), data).map_err(|e| e.to_string())
}

pub fn append_history(text: &str, latency_ms: u64) -> Result<TranscriptEntry, String> {
    let mut entries = load_history();
    let entry = TranscriptEntry {
        id: uuid::Uuid::new_v4().to_string(),
        text: text.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        latency_ms,
    };
    entries.insert(0, entry.clone());
    save_history(&entries)?;
    Ok(entry)
}
