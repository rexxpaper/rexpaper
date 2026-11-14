<p align="center">
  <img src="assets/logo.png" alt="RexPaper Logo" width="128" height="128" />
</p>

<h1 align="center">RexPaper</h1>

<p align="center">
  <strong>A fast, native wallpaper manager for Windows with hardware-accelerated live video wallpapers.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-1.0.7-blue?style=flat-square" alt="Version" />
  <img src="https://img.shields.io/badge/rust-edition%202024-orange?style=flat-square&logo=rust" alt="Rust Edition 2024" />
  <img src="https://img.shields.io/badge/UI-eframe%20%2F%20egui-673AB7?style=flat-square&logo=rust" alt="eframe / egui" />
  <img src="https://img.shields.io/badge/license-GPL--3.0-green?style=flat-square" alt="License GPL-3.0" />
  <img src="https://img.shields.io/badge/platform-windows%2010%20%7C%2011%20(x64)-0078D6?style=flat-square&logo=windows" alt="Platform Windows" />
  <img src="https://img.shields.io/badge/mpv-Direct3D11%20%2F%20GPU-darkblue?style=flat-square" alt="Direct3D11" />
</p>

<p align="center">
  <a href="#features">Features</a> &bull;
  <a href="#supported-formats">Formats</a> &bull;
  <a href="#tech-stack">Tech Stack</a> &bull;
  <a href="#build">Build</a> &bull;
  <a href="#download">Download</a> &bull;
  <a href="#license">License</a>
</p>

---

## Overview

RexPaper manages static and mpv-powered live wallpapers on Windows 10 and 11. It is a fully native Rust application: scanning, thumbnails, Win32 desktop integration, tray, and settings run in-process, with the UI rendered by eframe/egui through OpenGL.

## Features

- **Static gallery** — recursive scanning, automatic categories, live search, cached thumbnails, and instant Win32 wallpaper application.
- **Live wallpapers** — GPU-accelerated mpv playback injected behind desktop icons via WorkerW, with apply, pause, resume, and stop controls.
- **Background thumbnails** — multi-core Rayon precomputation with persistent disk caching.
- **Windows integration** — native tray menu, silent autostart, fullscreen auto-pause, and wallpaper restore on launch.
- **Settings** — native folder pickers and persistent JSON configuration.

## Supported Formats

| Media Type | Extensions |
|---|---|
| **Static Images** | `.png`, `.jpg`, `.jpeg`, `.webp`, `.bmp`, `.gif`, `.avif`, `.tiff`, `.tif` |
| **Live Videos** | `.mp4`, `.webm`, `.mkv`, `.mov`, `.avi`, `.flv`, `.m4v`, `.gif`, `.mpg`, `.mpeg`, `.wmv` |

## Tech Stack

| Component | Technology |
|---|---|
| **Language** | Rust (Edition 2024) |
| **UI** | eframe 0.31 / egui 0.31 (glow renderer) |
| **Core** | `crates/rexpaper-core` |
| **Video** | mpv with Direct3D11 GPU rendering |
| **Windows APIs** | windows-rs 0.61 |
| **Images** | image 0.25 + Rayon 1.10 |
| **Dialogs** | rfd 0.15 |
| **Packaging** | cargo-packager + NSIS |

## Build

### Prerequisites

- Rust 1.85+ (Edition 2024)
- Windows 10 SDK
- [7-Zip](https://www.7-zip.org/) for extracting the mpv archive

### Run

```powershell
git clone https://github.com/rexxpaper/rexpaper.git
cd rexpaper

# Stage the mpv runtime (first time, ~114 MB)
pwsh ./scripts/fetch-mpv.ps1

cargo run -p rexpaper-app
```

### Installer

```powershell
cargo install cargo-packager --locked

cargo build --release -p rexpaper-app
cargo packager --release --packages rexpaper-app --formats nsis
```

The standalone NSIS installer is written to:

```
dist/nsis/RexPaper_<version>_x64-setup.exe
```

## Download

Grab the latest installer from the [Releases](https://github.com/rexxpaper/rexpaper/releases/latest) page.

The installer bundles the mpv runtime and installs per-user without an admin prompt. Builds are unsigned, so Windows SmartScreen may warn on first run.

Releases are built by pushing a version tag:

```powershell
git tag v1.0.7
git push origin v1.0.7
```

## Project Layout

```
rexpaper/
├── app/                  # Native eframe/egui desktop app
├── crates/rexpaper-core/ # Scanner, thumbnails, wallpaper, tray, settings
├── scripts/fetch-mpv.ps1 # Stages the mpv runtime
├── assets/               # Icon and logo
└── .github/workflows/    # Release CI
```

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE).
