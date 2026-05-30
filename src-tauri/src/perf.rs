use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PerfProfile {
    Potato,
    Balanced,
    Quality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub total_memory_gb: f64,
    pub cpu_cores: usize,
    pub recommended_profile: PerfProfile,
}

pub fn detect_profile() -> PerfProfile {
    let mut sys = System::new();
    sys.refresh_memory();
    let total_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);

    if total_gb <= 4.5 || cores <= 2 {
        PerfProfile::Potato
    } else if total_gb >= 16.0 && cores >= 8 {
        PerfProfile::Quality
    } else {
        PerfProfile::Balanced
    }
}

pub fn detect_hardware() -> HardwareInfo {
    let mut sys = System::new();
    sys.refresh_memory();
    let total_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    let recommended = if total_gb <= 4.5 || cores <= 2 {
        PerfProfile::Potato
    } else if total_gb >= 16.0 && cores >= 8 {
        PerfProfile::Quality
    } else {
        PerfProfile::Balanced
    };
    HardwareInfo {
        total_memory_gb: (total_gb * 10.0).round() / 10.0,
        cpu_cores: cores,
        recommended_profile: recommended,
    }
}

pub fn thread_count(profile: PerfProfile) -> i32 {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2) as i32;
    match profile {
        PerfProfile::Potato => cores.min(2).max(1),
        PerfProfile::Balanced => cores.min(4).max(1),
        PerfProfile::Quality => cores.min(6).max(1),
    }
}

pub fn default_model_id(profile: PerfProfile) -> &'static str {
    match profile {
        PerfProfile::Potato => "tiny.en",
        PerfProfile::Balanced => "base",
        PerfProfile::Quality => "small",
    }
}

pub fn max_recording_secs(profile: PerfProfile) -> u32 {
    match profile {
        PerfProfile::Potato => 30,
        PerfProfile::Balanced => 60,
        PerfProfile::Quality => 120,
    }
}
