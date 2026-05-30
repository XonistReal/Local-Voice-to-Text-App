use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{thread, time::Duration};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InjectError {
    #[error("Failed to initialize keyboard: {0}")]
    Init(String),
    #[error("Failed to inject text: {0}")]
    TypeText(String),
}

pub fn inject_text(text: &str, paste_mode: bool, delay_ms: u32) -> Result<(), InjectError> {
    if text.is_empty() {
        return Ok(());
    }

    if delay_ms > 0 {
        thread::sleep(Duration::from_millis(delay_ms as u64));
    }

    if paste_mode {
        inject_via_clipboard(text)
    } else {
        inject_via_typing(text)
    }
}

fn inject_via_typing(text: &str) -> Result<(), InjectError> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| InjectError::Init(e.to_string()))?;
    enigo
        .text(text)
        .map_err(|e| InjectError::TypeText(e.to_string()))?;
    Ok(())
}

fn inject_via_clipboard(text: &str) -> Result<(), InjectError> {
    use arboard::Clipboard;

    let mut clipboard = Clipboard::new().map_err(|e| InjectError::Init(e.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|e| InjectError::TypeText(e.to_string()))?;

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| InjectError::Init(e.to_string()))?;

    #[cfg(target_os = "macos")]
    {
        enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| InjectError::TypeText(e.to_string()))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InjectError::TypeText(e.to_string()))?;
        enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| InjectError::TypeText(e.to_string()))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InjectError::TypeText(e.to_string()))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InjectError::TypeText(e.to_string()))?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InjectError::TypeText(e.to_string()))?;
    }

    Ok(())
}
