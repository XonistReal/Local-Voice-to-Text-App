use std::path::Path;
use thiserror::Error;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
};

use crate::gpu;
use crate::perf::{thread_count, PerfProfile};

#[derive(Debug, Error)]
pub enum TranscribeError {
    #[error("Failed to load model: {0}")]
    Load(String),
    #[error("Transcription failed: {0}")]
    Inference(String),
}

pub struct WhisperEngine {
    context: WhisperContext,
    gpu_active: bool,
}

impl WhisperEngine {
    pub fn gpu_active(&self) -> bool {
        self.gpu_active
    }

    pub fn load(model_path: &Path, profile: PerfProfile, prefer_gpu: bool) -> Result<Self, TranscribeError> {
        let path = model_path
            .to_str()
            .ok_or_else(|| TranscribeError::Load("Invalid path".into()))?;

        if prefer_gpu && gpu::gpu_compiled() {
            let mut params = WhisperContextParameters::default();
            params.use_gpu(true);
            params.gpu_device(0);

            if let Ok(context) = WhisperContext::new_with_params(path, params) {
                let _ = thread_count(profile, true);
                return Ok(Self {
                    context,
                    gpu_active: true,
                });
            }
        }

        let mut params = WhisperContextParameters::default();
        params.use_gpu(false);
        let context = WhisperContext::new_with_params(path, params)
            .map_err(|e| TranscribeError::Load(e.to_string()))?;

        Ok(Self {
            context,
            gpu_active: false,
        })
    }

    pub fn transcribe(
        &self,
        samples: &[f32],
        language: Option<&str>,
        profile: PerfProfile,
    ) -> Result<String, TranscribeError> {
        if samples.is_empty() {
            return Ok(String::new());
        }

        let mut state = self
            .context
            .create_state()
            .map_err(|e| TranscribeError::Inference(e.to_string()))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        // GPU handles the heavy work — keep CPU thread count minimal to avoid OS lag
        params.set_n_threads(thread_count(profile, self.gpu_active));
        params.set_translate(false);
        params.set_no_context(true);
        params.set_single_segment(true);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_token_timestamps(false);

        match language {
            Some("auto") | None => {}
            Some(lang) => params.set_language(Some(lang)),
        }

        state
            .full(params, samples)
            .map_err(|e| TranscribeError::Inference(e.to_string()))?;

        let n = state.full_n_segments();
        let mut text = String::new();
        for i in 0..n {
            if let Some(segment) = state.get_segment(i) {
                let segment_text = segment
                    .to_str_lossy()
                    .map_err(|e| TranscribeError::Inference(e.to_string()))?;
                text.push_str(segment_text.trim());
                if i + 1 < n {
                    text.push(' ');
                }
            }
        }

        Ok(text.trim().to_string())
    }
}
