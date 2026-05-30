# VTT — Local Voice to Text

**VTT** is a privacy-first, fully offline voice-to-text dictation app for Windows, macOS, and Linux. Hold a global hotkey, speak, release — transcribed text is typed at your cursor in any application.

No cloud. No subscription. No telemetry.

![VTT](https://img.shields.io/badge/offline-local-green) ![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **Push-to-talk dictation** — global hotkey works from any app (default: `Ctrl+Shift+Space`)
- **100% local processing** — powered by [whisper.cpp](https://github.com/ggerganov/whisper.cpp) via `whisper-rs`
- **GPU acceleration** — Vulkan on Windows/Linux, Metal on macOS
- **Potato mode** — auto-detects low-end hardware and tunes threads, model size, and RAM usage
- **Transcript polish** — rule-based cleanup (filler removal, casing, punctuation)
- **Neumorphic UI** — custom title bar, onboarding, settings, transcript history, recording overlay
- **Silence trimming** — skips empty recordings to save CPU cycles
- **System tray** — runs in the background with idle / recording / transcribing states

## Performance profiles

| Profile | Model | Threads | Best for |
|---------|-------|---------|----------|
| **Potato** | `tiny.en` (~75 MB) | 2 | ≤ 4 GB RAM, dual-core CPUs |
| **Balanced** | `base` (~142 MB) | 4 | 8 GB RAM, 4+ cores |
| **Quality** | `small` (~466 MB) | 6 | 16 GB+ RAM |

## Privacy

- Audio is captured **in memory only** — never written to disk
- No network calls after the initial model download
- Config and history stored locally at `%APPDATA%\com.joerh.vtt\` (Windows) or platform equivalent
- Fully open source — inspect the code yourself

## Prerequisites

### All platforms

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.77+
- [CMake](https://cmake.org/)
- C/C++ compiler (MSVC on Windows, Xcode CLT on macOS, build-essential on Linux)

### Windows

- [LLVM](https://releases.llvm.org/) — set `LIBCLANG_PATH` to the LLVM `bin` folder, e.g. `C:\Program Files\LLVM\bin`
- **Vulkan-capable GPU drivers** (NVIDIA, AMD, or Intel)
- WebView2 (included in Windows 10/11)

On Windows, if the default build path hits `MAX_PATH` limits with Vulkan shaders, use a short target dir:

```powershell
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:VULKAN_SDK = "C:\VulkanSDK\<version>"
$env:CARGO_TARGET_DIR = "C:\b"
npm run tauri build
```

### Linux

```bash
sudo apt install libasound2-dev libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libclang-dev cmake build-essential
```

### macOS

```bash
xcode-select --install
```

## Development

```bash
git clone https://github.com/XonistReal/Local-Voice-to-Text-App.git
cd Local-Voice-to-Text-App
npm install
npm run tauri dev
```

## Building a release

```bash
npm run tauri build
```

Installers appear in `src-tauri/target/release/bundle/` (or your `CARGO_TARGET_DIR` if set).

## First run

1. Complete the onboarding wizard (hardware detection → model download → hotkey setup)
2. Hold your hotkey anywhere and speak
3. Release to transcribe and inject text at the cursor

Models download from [HuggingFace whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp) on first use.

## Project structure

```
Local-Voice-to-Text-App/
├── assets/                 # Source icon (synced to Tauri + public on build)
├── public/                 # Static assets (app-icon.png generated at build)
├── scripts/
│   └── sync-icon.mjs       # Regenerates Tauri icons before frontend build
├── src/                    # React + TypeScript frontend
│   ├── components/         # UI (TitleBar, Layout, NeoCard, …)
│   ├── hooks/              # Recording state, model management
│   ├── pages/              # Home, Settings, History, Onboarding, Overlay
│   └── styles/             # Neumorphic theme + scrollbar styling
├── src-tauri/              # Rust backend (Tauri 2)
│   ├── icons/              # Generated app icons (Windows/macOS/Linux/mobile)
│   └── src/
│       ├── audio.rs        # Microphone capture (cpal)
│       ├── transcribe.rs   # Whisper inference
│       ├── gpu.rs          # Vulkan / Metal / CUDA backend selection
│       ├── vad.rs          # Silence detection / trim
│       ├── polish.rs       # Transcript cleanup rules
│       ├── perf.rs         # Hardware profiles
│       ├── models.rs       # Model download & management
│       ├── inject.rs       # Text injection (enigo)
│       └── config.rs       # Local settings & history
├── index.html              # Main window entry
├── overlay.html            # Recording overlay entry
└── .github/workflows/      # CI (lint, cargo check, cross-platform build)
```

## Tech stack

| Layer | Technology |
|-------|------------|
| Shell | Tauri 2 |
| STT | whisper.cpp / whisper-rs |
| GPU | Vulkan (Windows/Linux), Metal (macOS) |
| Audio | cpal |
| UI | React 19, TypeScript, Tailwind CSS 4 |
| Hotkeys | tauri-plugin-global-shortcut |

## Known limitations

- Text injection may not work in elevated/admin apps — use paste mode as fallback
- Linux Wayland global shortcuts have limited support; X11 recommended
- First transcription after launch loads the model (one-time delay on Potato tier)

## License

MIT — see [LICENSE](LICENSE).

## Contributing

PRs welcome. File issues for platform-specific bugs or feature requests.
