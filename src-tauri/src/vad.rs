const MIN_SPEECH_MS: u32 = 300;
const FRAME_MS: u32 = 30;
const ENERGY_THRESHOLD: f32 = 0.008;

fn frame_size(sample_rate: u32) -> usize {
    (sample_rate as f32 * FRAME_MS as f32 / 1000.0) as usize
}

fn frame_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

pub fn has_speech(samples: &[f32], sample_rate: u32) -> bool {
    let frame = frame_size(sample_rate).max(1);
    let min_frames = (MIN_SPEECH_MS / FRAME_MS).max(1) as usize;
    let mut speech_frames = 0usize;

    for chunk in samples.chunks(frame) {
        if frame_rms(chunk) >= ENERGY_THRESHOLD {
            speech_frames += 1;
            if speech_frames >= min_frames {
                return true;
            }
        }
    }
    false
}

pub fn trim_silence(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let frame = frame_size(sample_rate).max(1);
    let mut start = 0usize;
    let mut end = samples.len();

    while start + frame <= samples.len() {
        if frame_rms(&samples[start..start + frame]) >= ENERGY_THRESHOLD {
            break;
        }
        start += frame;
    }

    while end > start + frame {
        if frame_rms(&samples[end - frame..end]) >= ENERGY_THRESHOLD {
            break;
        }
        end -= frame;
    }

    if end <= start {
        return Vec::new();
    }

    let pad = (sample_rate as f32 * 0.05) as usize;
    let start = start.saturating_sub(pad);
    let end = (end + pad).min(samples.len());
    samples[start..end].to_vec()
}
