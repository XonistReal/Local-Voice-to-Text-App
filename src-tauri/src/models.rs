use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::perf::PerfProfile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelKind {
    Speech,
    Polish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogEntry {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub url: String,
    pub size_bytes: u64,
    pub profile: PerfProfile,
    pub kind: ModelKind,
}

const HF_BASE: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

pub fn speech_catalog() -> Vec<ModelCatalogEntry> {
    vec![
        ModelCatalogEntry {
            id: "tiny.en".into(),
            name: "Tiny (English)".into(),
            filename: "ggml-tiny.en.bin".into(),
            url: format!("{HF_BASE}/ggml-tiny.en.bin"),
            size_bytes: 77_000_000,
            profile: PerfProfile::Potato,
            kind: ModelKind::Speech,
        },
        ModelCatalogEntry {
            id: "tiny".into(),
            name: "Tiny (Multilingual)".into(),
            filename: "ggml-tiny.bin".into(),
            url: format!("{HF_BASE}/ggml-tiny.bin"),
            size_bytes: 77_000_000,
            profile: PerfProfile::Potato,
            kind: ModelKind::Speech,
        },
        ModelCatalogEntry {
            id: "base".into(),
            name: "Base".into(),
            filename: "ggml-base.bin".into(),
            url: format!("{HF_BASE}/ggml-base.bin"),
            size_bytes: 148_000_000,
            profile: PerfProfile::Balanced,
            kind: ModelKind::Speech,
        },
        ModelCatalogEntry {
            id: "small".into(),
            name: "Small".into(),
            filename: "ggml-small.bin".into(),
            url: format!("{HF_BASE}/ggml-small.bin"),
            size_bytes: 488_000_000,
            profile: PerfProfile::Quality,
            kind: ModelKind::Speech,
        },
    ]
}

pub fn catalog() -> Vec<ModelCatalogEntry> {
    speech_catalog()
}

pub fn models_dir() -> PathBuf {
    crate::config::config_dir().join("models")
}

pub fn model_path(filename: &str) -> PathBuf {
    models_dir().join(filename)
}

pub fn find_catalog_entry(id: &str) -> Option<ModelCatalogEntry> {
    catalog().into_iter().find(|m| m.id == id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledModel {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub size_bytes: u64,
    pub path: String,
}

pub fn list_installed() -> Vec<InstalledModel> {
    let dir = models_dir();
    if !dir.exists() {
        return Vec::new();
    }
    catalog()
        .into_iter()
        .filter_map(|entry| {
            let path = model_path(&entry.filename);
            if path.exists() {
                let size = std::fs::metadata(&path).ok()?.len();
                Some(InstalledModel {
                    id: entry.id,
                    name: entry.name,
                    filename: entry.filename,
                    size_bytes: size,
                    path: path.to_string_lossy().to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn is_installed(model_id: &str) -> bool {
    find_catalog_entry(model_id)
        .map(|e| model_path(&e.filename).exists())
        .unwrap_or(false)
}

pub fn active_model_path(model_id: &str) -> Option<PathBuf> {
    find_catalog_entry(model_id).map(|e| model_path(&e.filename))
}

pub async fn download_model(
    model_id: &str,
    progress: impl Fn(u64, u64) + Send + Sync + 'static,
) -> Result<PathBuf, String> {
    let entry = find_catalog_entry(model_id).ok_or_else(|| "Unknown model".to_string())?;
    let dir = models_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = model_path(&entry.filename);
    let tmp = dest.with_extension("bin.part");

    let client = reqwest::Client::builder()
        .user_agent("VTT/0.1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&entry.url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let total = response.content_length().unwrap_or(entry.size_bytes);
    let mut downloaded: u64 = 0;
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| e.to_string())?;
    use tokio::io::AsyncWriteExt;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        progress(downloaded, total);
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    if dest.exists() {
        std::fs::remove_file(&dest).ok();
    }
    std::fs::rename(&tmp, &dest).map_err(|e| e.to_string())?;
    progress(total, total);
    Ok(dest)
}

pub fn delete_model(model_id: &str) -> Result<(), String> {
    let entry = find_catalog_entry(model_id).ok_or_else(|| "Unknown model".to_string())?;
    let path = model_path(&entry.filename);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn ensure_model_on_disk(model_id: &str) -> Result<PathBuf, String> {
    let path = active_model_path(model_id).ok_or_else(|| "Unknown model".to_string())?;
    if path.exists() {
        Ok(path)
    } else {
        Err(format!(
            "Model '{model_id}' is not installed. Download it from Settings."
        ))
    }
}
