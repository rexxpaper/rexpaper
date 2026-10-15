<p align="center">
  <img src="assets/logo.png" alt="RexPaper Logo" width="128" height="128" />
</p>

<h1 align="center">RexPaper</h1>

<p align="center">
  <strong>A high-performance wallpaper manager for Windows with hardware-accelerated live video wallpapers.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-1.0.6-blue?style=flat-square" alt="Version" />
  <img src="https://img.shields.io/badge/rust-edition%202024-orange?style=flat-square&logo=rust" alt="Rust Edition 2024" />
  <img src="https://img.shields.io/badge/tauri-v2-24C8DB?style=flat-square&logo=tauri" alt="Tauri v2" />
  <img src="https://img.shields.io/badge/license-GPL--3.0-green?style=flat-square" alt="License GPL-3.0" />
  <img src="https://img.shields.io/badge/platform-windows%2010%20%7C%2011%20(x64)-0078D6?style=flat-square&logo=windows" alt="Platform Windows" />
  <img src="https://img.shields.io/badge/mpv-Direct3D11%20%2F%20GPU-darkblue?style=flat-square" alt="Direct3D11" />
</p>

<p align="center">
  <a href="#overview">Overview</a> &bull;
  <a href="#download">Download</a> &bull;
  <a href="#features">Features</a> &bull;
  <a href="#supported-formats">Supported Formats</a> &bull;
  <a href="#tech-stack">Tech Stack</a> &bull;
  <a href="#getting-started">Getting Started</a> &bull;
  <a href="#project-structure">Project Structure</a> &bull;
  <a href="#roadmap">Roadmap</a> &bull;
  <a href="#license">License</a>
</p>

---

## Overview

**RexPaper** is a fast, lightweight wallpaper manager engineered specifically for Windows 10 and 11. The backend is pure **Rust (Edition 2024)** — Win32 desktop window manipulation, thumbnail pipeline, tray, and settings — while the UI is a minimal **Preact + Tailwind** app running in a **Tauri v2** WebView.

It manages thousands of ultra-high-resolution 4K/8K static wallpapers or renders seamless 60+ FPS live video wallpapers behind your desktop icons, with minimal resource usage and zero UI stutter.

> **Status:** the Slint app has been fully replaced by the Tauri v2 + Preact rewrite. Static and live galleries, settings, tray, autostart, and fullscreen auto-pause all ship, and tagged builds produce a standalone Windows installer. See [`docs/specs/tauri-v2-migration.md`](docs/specs/tauri-v2-migration.md).

---

## Current Status

_Last updated 2026-10-15._

RexPaper is feature-complete for its first Tauri release (`v1.0.6`), verified with `npm run tauri dev` and a full `npm run tauri build`:

- **App shell** — frameless window with a custom titlebar (minimize / maximize / close, drag-to-move, double-click to maximize) and a sidebar routing between the Static, Live, and Settings pages.
- **Static gallery** — `rfd` folder picker, thumbnails delivered over the Tauri asset protocol (batch resolver plus background precompute with `thumbnail-progress` events), category filter, live search, and click-to-apply with an active highlight.
- **Live gallery** — mpv frame thumbnails, hover previews streamed over range-request asset URLs, and apply / stop controls.
- **Settings** — directory pickers and toggles for run-on-startup, pause-on-fullscreen, and mute-live-wallpapers, persisted to the config file shared with v1.
- **Tray + window behaviour** — an 8-item tray menu, hidden start on `--autostart` / `--minimized`, wallpaper restore on launch, and a fullscreen auto-pause monitor driving the UI through `live-paused` events.
- **Backend** — 17 typed Tauri commands over the UI-free `crates/rexpaper-core` (scanner, thumbnail cache, Win32/WorkerW injection, mpv, settings).
- **Packaging** — a standalone, offline NSIS installer plus a tag-driven release workflow.

### Remaining

- **Multi-monitor wallpaper assignment** — independent wallpaper per display.
- **Live audio volume slider** — in-app control over live wallpaper volume.
- **README screenshot** — capture the finished UI.

Known dev-workaround: the registry write for "run on startup" stays guarded out of debug builds so dev runs cannot hijack the installed v1 autostart entry.

---

## Features

### High-Performance Static Wallpaper Gallery
- **Recursive Directory Scanning**: Scans custom wallpaper directories recursively and indexes entire collections in milliseconds.
- **Dynamic Category Organization**: Automatically groups wallpapers into clean categories based on subfolder structure (e.g., `Wallpapers/Anime/pic.jpg` &rarr; `Anime`).
- **Real-Time Instant Search**: Live text-filtering across titles and categories as you type.
- **Native Win32 Wallpaper Application**: Applies static wallpapers directly via `SystemParametersInfoW(SPI_SETDESKWALLPAPER)` with instant desktop refresh.

### Hardware-Accelerated Live Video Wallpapers
- **GPU-Accelerated Video Engine**: Direct3D11 / GPU-powered rendering (`--vo=gpu`, `--hwdec=auto-safe`) delivering butter-smooth 60+ FPS playback with near-zero CPU consumption.
- **WorkerW Desktop Canvas Injection**: Injects video playback seamlessly behind Windows desktop icons (`SHELLDLL_DefView`) using `Progman` shell message `0x052C` and `WorkerW` reparenting.
- **Seamless Two-Way Transitions**: Live &rarr; static terminates `mpv` and hides `WorkerW`; static &rarr; live reveals `WorkerW` and attaches playback to `--wid=<hwnd>`.
- **Live Wallpaper Controls**: Start, pause, resume, and stop desktop live wallpapers at any time.

### Multi-Core Background Precomputation & Disk Caching
- **Parallel Multi-Core Resizing**: Uses Rayon (`rayon::par_iter()`) across all CPU threads to precompute thumbnails on background threads.
- **GPU-Accelerated Frame Extraction**: Captures video frames at 0.5s via `mpv` (`--vo=image`, `--hwdec=auto-safe`) for instantaneous video browsing.
- **Persistent Disk Caching**: Caches 384&times;216 px thumbnails in `%LOCALAPPDATA%/rexpaper/cache` using 64-bit cryptographic hashing for sub-millisecond lookup.
- **Crash-Proof Fault Tolerance**: All decoding and thumbnail generation is wrapped in `std::panic::catch_unwind` with dark placeholder fallbacks for corrupt files.

### Deep Windows System Integration
- **System Tray Integration**: Background tray with Open, page shortcuts, Pause / Resume Live Wallpaper, Next Wallpaper, Open Wallpaper Folder, and Quit.
- **Per-Monitor V2 HiDPI Awareness**: Razor-sharp rendering at every display scaling factor.
- **Silent Windows Autostart**: Registry integration (`HKCU\...\CurrentVersion\Run`) with `--autostart` / `--minimized` flags.

### Configurable Settings & Persistence
- **Persistent JSON Configuration**: Settings saved via Serde (shared with the v1 install).
- **Native Folder Pickers**: Windows folder pickers via `rfd` for static and live wallpaper library locations.
- **Configurable Toggles**: Run on startup, pause on fullscreen, mute live wallpapers.

---

## Supported Formats

| Media Type | Supported Formats / Extensions |
|---|---|
| **Static Images** | `.png`, `.jpg`, `.jpeg`, `.webp`, `.bmp`, `.gif`, `.avif`, `.tiff`, `.tif` |
| **Live Videos** | `.mp4`, `.webm`, `.mkv`, `.mov`, `.avi`, `.flv`, `.m4v`, `.gif`, `.mpg`, `.mpeg`, `.wmv` |

---

## Tech Stack

| Component | Technology | Description |
|---|---|---|
| **Language** | <img src="https://img.shields.io/badge/Rust-2024-orange?style=flat-square&logo=rust" alt="Rust" /> Rust (Edition 2024) | High-performance, memory-safe systems programming |
| **App Shell** | <img src="https://img.shields.io/badge/Tauri-v2-24C8DB?style=flat-square&logo=tauri" alt="Tauri" /> Tauri v2 | WebView shell, typed commands, events, NSIS bundling |
| **UI** | <img src="https://img.shields.io/badge/Preact-+Vite-673AB7?style=flat-square" alt="Preact" /> Preact + Vite + TypeScript | Minimal SPA with Preact Signals state |
| **Styling** | <img src="https://img.shields.io/badge/Tailwind_CSS-v4-38B2AC?style=flat-square" alt="Tailwind" /> Tailwind CSS v4 | Flat, hairline-bordered design language |
| **Core Library** | <img src="https://img.shields.io/badge/rexpaper--core-workspace-orange?style=flat-square" alt="rexpaper-core" /> `crates/rexpaper-core` | UI-free backend: scanner, settings, wallpaper, tray, thumbnails |
| **Video Engine** | <img src="https://img.shields.io/badge/mpv-darkblue?style=flat-square" alt="mpv" /> mpv (D3D11) | Hardware-accelerated playback and frame extraction |
| **Win32 OS APIs** | <img src="https://img.shields.io/badge/windows--rs-0.61-blue?style=flat-square&logo=windows" alt="windows-rs" /> windows-rs (0.61) | `WorkerW` injection, `Shell_NotifyIconW`, HiDPI |
| **Dialogs** | <img src="https://img.shields.io/badge/rfd-0.15-green?style=flat-square" alt="rfd" /> rfd (0.15) | Native Windows folder pickers |
| **Packaging** | <img src="https://img.shields.io/badge/NSIS-via%20Tauri-orange?style=flat-square" alt="NSIS" /> NSIS (Tauri bundler) | Windows installer |

---

## Download

Grab the latest installer from the [Releases](https://github.com/rexxpaper/rexpaper/releases/latest) page:

**`RexPaper_<version>_x64-setup.exe`** — Windows 10 / 11 (x64)

The installer is fully self-contained: it bundles the mpv runtime and uses an embedded WebView2 bootstrapper, so no internet connection is required. It installs per-user (no admin prompt). Builds are currently **unsigned**, so Windows SmartScreen may warn on first run — choose *More info &rarr; Run anyway*.

---

## Getting Started

### Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/tools/install) (Edition 2024 / 1.85+)
- [Node.js](https://nodejs.org/) 20+
- Windows 10 SDK or later
- mpv runtime — staged automatically by `scripts/fetch-mpv.ps1` into `src-tauri/mpv/mpv.exe` (downloads a pinned x86_64 build; required for live wallpapers)
- [7-Zip](https://www.7-zip.org/) — used by the fetch script to extract the mpv archive

### Build and Run

```powershell
# Clone the repository
git clone https://github.com/rexxpaper/rexpaper.git
cd rexpaper

# Install frontend dependencies (first time)
npm install

# Stage the mpv runtime (first time, ~114 MB, needs 7-Zip)
pwsh ./scripts/fetch-mpv.ps1

# Development mode (Vite dev server + Tauri window)
npm run tauri dev

# Production build (standalone NSIS installer)
npm run tauri build
```

`cargo build` on its own builds the workspace (core + app), but the app needs the Vite dev server or a built `frontend/dist` to render — use `npm run tauri dev`.

### Installer

`npm run tauri build` produces a **standalone, offline** installer at:

```
src-tauri/target/release/bundle/nsis/RexPaper_<version>_x64-setup.exe
```

It bundles the mpv runtime and uses an embedded WebView2 bootstrapper, so it installs and runs without downloading anything. It installs per-user (no admin prompt) and is currently **unsigned** — Windows SmartScreen may show a warning on first run until the download builds reputation.

### Releases

Releases are built by the `Release` GitHub Actions workflow. Push a version tag and it builds the installer and attaches it to a GitHub release:

```powershell
git tag v1.0.6
git push origin v1.0.6
```

---

## Project Structure

```
rexpaper/
├── assets/
│   ├── icon.ico             # Application icon
│   └── logo.png             # RexPaper brand logo
├── crates/
│   └── rexpaper-core/       # UI-free Rust backend library
│       ├── build.rs         # Stages mpv.exe next to the built binary
│       └── src/
│           ├── scanner.rs          # Recursive filesystem wallpaper scanner
│           ├── thumbnail.rs        # Multi-core thumbnail precompute & disk cache
│           ├── static_wallpaper.rs # Win32 wallpaper application
│           ├── settings.rs         # JSON settings + autostart registry
│           ├── models.rs           # Shared item & app-state types
│           └── platform/           # WorkerW injection, live playback, fullscreen monitor
├── src-tauri/               # Tauri v2 app (commands, events, config)
│   ├── src/main.rs          # Entry point, tray, restore, command registration
│   ├── src/commands.rs      # 17 typed commands over rexpaper-core
│   ├── windows/hooks.nsh    # NSIS install/uninstall hooks
│   ├── mpv/                 # mpv runtime binary (fetched, not committed)
│   └── tauri.conf.json      # Window, dev server, NSIS bundle config
├── frontend/                # Preact + Vite + TypeScript + Tailwind
│   └── src/
│       ├── pages/           # Static, live, and settings pages
│       ├── components/      # Titlebar, sidebar, gallery grid
│       └── lib/             # Typed API mirror, signals store, gallery helpers
├── scripts/fetch-mpv.ps1    # Downloads the pinned mpv runtime
├── .github/workflows/       # Tag-driven release CI
├── docs/specs/              # Design specs
└── LICENSE                  # GNU General Public License v3.0
```

---

## Roadmap

- [x] **Core extraction** &mdash; UI-free `rexpaper-core` workspace crate.
- [x] **Tauri v2 + Preact scaffold** &mdash; typed commands, minimal settings/scan UI.
- [x] **First launch + app shell** &mdash; frameless window, custom titlebar with window controls, sidebar routing.
- [x] **Static gallery page** &mdash; grid, categories, search, lazy thumbnails, apply.
- [x] **Live gallery page** &mdash; video thumbnails, hover previews, apply/stop controls.
- [x] **Settings page** &mdash; directory pickers, toggles, tray + autostart wiring.
- [x] **NSIS packaging + release CI** &mdash; standalone offline installer (bundled mpv) and a tag-driven GitHub Actions release workflow.
- [ ] **Multi-Monitor Wallpaper Assignment** &mdash; Assign independent wallpapers per display monitor.
- [ ] **Granular Live Audio Volume Slider** &mdash; In-app slider to adjust live wallpaper volume levels.

---

## License

This project is licensed under the **GNU General Public License v3.0** &mdash; see the [LICENSE](LICENSE) file for details.

<p align="center">
  Made by <a href="https://github.com/rexxpaper">rexxpaper</a>
</p>
