use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use thiserror::Error;

pub const TARGET_SAMPLE_RATE: u32 = 16000;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No input device available")]
    NoDevice,
    #[error("Failed to build audio stream: {0}")]
    Stream(String),
    #[error("Already recording")]
    AlreadyRecording,
    #[error("Not recording")]
    NotRecording,
}

pub struct AudioCapture {
    recording: Arc<AtomicBool>,
    raw_samples: Arc<Mutex<Vec<f32>>>,
    input_sample_rate: Arc<Mutex<u32>>,
    max_samples: usize,
    stream: Mutex<Option<cpal::Stream>>,
}

impl AudioCapture {
    pub fn new(max_recording_secs: u32) -> Self {
        let max_samples = (TARGET_SAMPLE_RATE as usize) * max_recording_secs as usize;
        Self {
            recording: Arc::new(AtomicBool::new(false)),
            raw_samples: Arc::new(Mutex::new(Vec::with_capacity(max_samples))),
            input_sample_rate: Arc::new(Mutex::new(TARGET_SAMPLE_RATE)),
            max_samples,
            stream: Mutex::new(None),
        }
    }

    pub fn set_max_duration(&mut self, secs: u32) {
        self.max_samples = (TARGET_SAMPLE_RATE as usize) * secs as usize;
        let mut buf = self.raw_samples.lock();
        if buf.capacity() < self.max_samples {
            let needed = self.max_samples - buf.capacity();
            buf.reserve(needed);
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::SeqCst)
    }

    pub fn start(&self) -> Result<(), AudioError> {
        if self.recording.load(Ordering::SeqCst) {
            return Err(AudioError::AlreadyRecording);
        }

        {
            let mut buf = self.raw_samples.lock();
            buf.clear();
            if buf.capacity() < self.max_samples {
                let needed = self.max_samples - buf.capacity();
                buf.reserve(needed);
            }
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::NoDevice)?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::Stream(e.to_string()))?;

        let sample_rate = config.sample_rate();
        *self.input_sample_rate.lock() = sample_rate;

        let recording = Arc::clone(&self.recording);
        let raw_samples = Arc::clone(&self.raw_samples);
        let max_samples = self.max_samples;

        recording.store(true, Ordering::SeqCst);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => build_stream::<f32>(
                &device,
                &config.into(),
                recording,
                raw_samples,
                max_samples,
            ),
            cpal::SampleFormat::I16 => build_stream::<i16>(
                &device,
                &config.into(),
                recording,
                raw_samples,
                max_samples,
            ),
            cpal::SampleFormat::U16 => build_stream::<u16>(
                &device,
                &config.into(),
                recording,
                raw_samples,
                max_samples,
            ),
            _ => return Err(AudioError::Stream("Unsupported sample format".into())),
        }
        .map_err(|e| AudioError::Stream(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioError::Stream(e.to_string()))?;

        *self.stream.lock() = Some(stream);
        Ok(())
    }

    pub fn stop(&self) -> Result<Vec<f32>, AudioError> {
        if !self.recording.load(Ordering::SeqCst) {
            return Err(AudioError::NotRecording);
        }

        self.recording.store(false, Ordering::SeqCst);
        *self.stream.lock() = None;

        let raw = self.raw_samples.lock().clone();
        let input_rate = *self.input_sample_rate.lock();
        Ok(resample_linear(&raw, input_rate, TARGET_SAMPLE_RATE))
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    recording: Arc<AtomicBool>,
    raw_samples: Arc<Mutex<Vec<f32>>>,
    max_samples: usize,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    device.build_input_stream(
        config,
        move |data: &[T], _| {
            if !recording.load(Ordering::SeqCst) {
                return;
            }
            let mut buf = raw_samples.lock();
            if buf.len() >= max_samples {
                return;
            }
            for frame in data.chunks(channels) {
                if buf.len() >= max_samples {
                    break;
                }
                let sample: f32 = frame[0].to_sample();
                buf.push(sample);
            }
        },
        |err| eprintln!("Audio stream error: {err}"),
        None,
    )
}

fn resample_linear(input: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }
    if input_rate == output_rate {
        return input.to_vec();
    }

    let ratio = input_rate as f64 / output_rate as f64;
    let out_len = (input.len() as f64 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src = i as f64 * ratio;
        let idx = src.floor() as usize;
        let frac = (src - idx as f64) as f32;
        let s0 = input.get(idx).copied().unwrap_or(0.0);
        let s1 = input.get(idx + 1).copied().unwrap_or(s0);
        output.push(s0 + (s1 - s0) * frac);
    }

    output
}
