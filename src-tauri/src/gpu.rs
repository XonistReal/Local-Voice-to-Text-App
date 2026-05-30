/// GPU backend compiled into whisper.cpp via whisper-rs feature flags.
pub fn compiled_backend() -> Option<&'static str> {
    if cfg!(feature = "gpu-cuda") {
        Some("cuda")
    } else if cfg!(feature = "gpu-vulkan") {
        Some("vulkan")
    } else if cfg!(feature = "gpu-metal") {
        Some("metal")
    } else {
        None
    }
}

pub fn gpu_compiled() -> bool {
    compiled_backend().is_some()
}

pub fn backend_display_name(backend: &str) -> &str {
    match backend {
        "cuda" => "CUDA (NVIDIA)",
        "vulkan" => "Vulkan (GPU)",
        "metal" => "Metal (Apple GPU)",
        _ => backend,
    }
}
