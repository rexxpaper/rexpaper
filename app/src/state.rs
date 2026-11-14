//! Background work for the egui UI.
//!
//! Every operation that touches the filesystem, spawns mpv, or walks a
//! wallpaper library runs on a worker thread and reports back with a
//! [`Msg`]. The UI thread never blocks on scanning, applying, folder
//! pickers, or thumbnail generation.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use eframe::egui;

use rexpaper_core::settings::Settings;
use rexpaper_core::{
    LiveWallpaperItem, SharedState, WallpaperItem, platform::windows, scanner, static_wallpaper,
    thumbnail,
};

/// Shared settings handle used by the UI and background workers.
pub type SettingsState = Arc<Mutex<Settings>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LibraryKind {
    Static,
    Live,
}

impl LibraryKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::Static => "Choose wallpaper folder",
            Self::Live => "Choose video folder",
        }
    }
}

/// Messages from worker threads (and the tray) to the UI thread.
pub enum Msg {
    Scanned {
        generation: u64,
        dirs: (Option<PathBuf>, Option<PathBuf>),
        static_items: Vec<WallpaperItem>,
        live_items: Vec<LiveWallpaperItem>,
        static_categories: Vec<String>,
        live_categories: Vec<String>,
        error: Option<String>,
    },
    ScanFailed {
        generation: u64,
        message: String,
    },
    Thumbs {
        generation: u64,
        items: Vec<(PathBuf, Option<egui::ColorImage>)>,
        remaining: usize,
    },
    LivePaused(bool),
    ApplyDone(Result<Settings, String>),
    StopDone(Result<Settings, String>),
    PauseDone(Result<bool, String>),
    PickDone {
        kind: LibraryKind,
        result: Result<Option<Settings>, String>,
    },
    Tray(rexpaper_core::platform::tray::TrayAction),
}

/// Channel + repaint handle handed to every worker.
#[derive(Clone)]
pub struct Worker {
    tx: Sender<Msg>,
    ctx: egui::Context,
}

impl Worker {
    pub fn new(tx: Sender<Msg>, ctx: egui::Context) -> Self {
        Self { tx, ctx }
    }

    pub fn send(&self, msg: Msg) {
        let _ = self.tx.send(msg);
        self.ctx.request_repaint();
    }
}

pub fn spawn_scan(
    worker: Worker,
    state: SharedState,
    generation: u64,
    dirs: (Option<PathBuf>, Option<PathBuf>),
) {
    std::thread::Builder::new()
        .name("rexpaper-scan".to_string())
        .spawn(move || {
            let mut error = None;

            if let Some(dir) = &dirs.0
                && let Err(e) = scanner::scan_static(dir, state.clone())
            {
                error = Some(e.to_string());
            }
            if let Some(dir) = &dirs.1
                && let Err(e) = scanner::scan_live(dir, state.clone())
            {
                error = Some(e.to_string());
            }

            let (static_items, live_items, static_categories, live_categories) = match state.lock()
            {
                Ok(s) => (
                    s.static_wallpapers.clone(),
                    s.live_wallpapers.clone(),
                    s.get_categories(),
                    s.get_live_categories(),
                ),
                Err(e) => {
                    worker.send(Msg::ScanFailed {
                        generation,
                        message: e.to_string(),
                    });
                    return;
                }
            };

            worker.send(Msg::Scanned {
                generation,
                dirs,
                static_items,
                live_items,
                static_categories,
                live_categories,
                error,
            });
        })
        .ok();
}

pub fn spawn_thumbnails(worker: Worker, paths: Vec<PathBuf>, kind: LibraryKind, generation: u64) {
    std::thread::Builder::new()
        .name("rexpaper-thumbs".to_string())
        .spawn(move || {
            let batch_size = match kind {
                LibraryKind::Static => 24,
                LibraryKind::Live => 8,
            };
            let total = paths.len();
            let mut processed = 0usize;

            for chunk in paths.chunks(batch_size) {
                match kind {
                    LibraryKind::Static => thumbnail::precompute_static_thumbnails(chunk),
                    LibraryKind::Live => thumbnail::precompute_video_thumbnails(chunk),
                }

                let items = chunk
                    .iter()
                    .map(|path| {
                        let thumb = match kind {
                            LibraryKind::Static => thumbnail::static_thumbnail_path(path),
                            LibraryKind::Live => thumbnail::video_thumbnail_path(path),
                        };
                        let image = thumb
                            .and_then(|thumb| image::open(thumb).ok())
                            .map(|image| {
                                let rgba = image.to_rgba8();
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [rgba.width() as usize, rgba.height() as usize],
                                    rgba.as_raw(),
                                )
                            });
                        (path.clone(), image)
                    })
                    .collect();

                processed += chunk.len();
                let remaining = total - processed;
                worker.send(Msg::Thumbs {
                    generation,
                    items,
                    remaining,
                });
                if remaining == 0 {
                    break;
                }
            }
        })
        .ok();
}

pub fn spawn_apply_static(worker: Worker, settings: SettingsState, path: PathBuf) {
    std::thread::Builder::new()
        .name("rexpaper-apply-static".to_string())
        .spawn(move || {
            worker.send(Msg::ApplyDone(apply_static(&settings, &path)));
        })
        .ok();
}

pub fn spawn_apply_live(worker: Worker, settings: SettingsState, path: PathBuf) {
    std::thread::Builder::new()
        .name("rexpaper-apply-live".to_string())
        .spawn(move || {
            worker.send(Msg::ApplyDone(apply_live(&settings, &path)));
        })
        .ok();
}

pub fn spawn_stop_live(worker: Worker, settings: SettingsState) {
    std::thread::Builder::new()
        .name("rexpaper-stop-live".to_string())
        .spawn(move || {
            worker.send(Msg::StopDone(stop_live(&settings)));
        })
        .ok();
}

pub fn spawn_next_wallpaper(worker: Worker, settings: SettingsState, state: SharedState) {
    std::thread::Builder::new()
        .name("rexpaper-next".to_string())
        .spawn(move || {
            worker.send(Msg::ApplyDone(next_wallpaper(&settings, &state)));
        })
        .ok();
}

pub fn spawn_toggle_pause(worker: Worker, settings: SettingsState) {
    std::thread::Builder::new()
        .name("rexpaper-pause".to_string())
        .spawn(move || {
            worker.send(Msg::PauseDone(toggle_pause(&settings)));
        })
        .ok();
}

pub fn spawn_pick_folder(worker: Worker, settings: SettingsState, kind: LibraryKind) {
    std::thread::Builder::new()
        .name("rexpaper-pick-folder".to_string())
        .spawn(move || {
            let dir = rfd::FileDialog::new().set_title(kind.title()).pick_folder();
            let result = match dir {
                None => Ok(None),
                Some(dir) => {
                    let mut s = match settings.lock() {
                        Ok(s) => s,
                        Err(e) => {
                            worker.send(Msg::PickDone {
                                kind,
                                result: Err(e.to_string()),
                            });
                            return;
                        }
                    };
                    match kind {
                        LibraryKind::Static => s.set_wallpaper_dir(dir),
                        LibraryKind::Live => s.set_live_wallpaper_dir(dir),
                    }
                    Ok(Some(s.clone()))
                }
            };
            worker.send(Msg::PickDone { kind, result });
        })
        .ok();
}

/// Restores the previously applied wallpapers without delaying the window.
pub fn spawn_restore(settings: SettingsState) {
    std::thread::Builder::new()
        .name("rexpaper-restore".to_string())
        .spawn(move || {
            let (static_path, live_path) = match settings.lock() {
                Ok(s) => {
                    let live = if s.wallpaper_mode == "live" {
                        s.active_live_wallpaper.clone()
                    } else {
                        None
                    };
                    (s.active_static_wallpaper.clone(), live)
                }
                Err(_) => return,
            };

            if let Some(path) = static_path.filter(|p| p.exists()) {
                let _ = static_wallpaper::apply_static_wallpaper(&path);
            }
            if let Some(path) = live_path.filter(|p| p.exists()) {
                let _ = windows::apply_live_wallpaper(&path);
            }
        })
        .ok();
}

fn apply_static(settings: &SettingsState, path: &Path) -> Result<Settings, String> {
    static_wallpaper::apply_static_wallpaper(path).map_err(|e| e.to_string())?;

    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.active_live_wallpaper = None;
    s.active_static_wallpaper = Some(path.to_path_buf());
    s.wallpaper_mode = "static".to_string();
    s.save().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

fn apply_live(settings: &SettingsState, path: &Path) -> Result<Settings, String> {
    windows::apply_live_wallpaper(path).map_err(|e| e.to_string())?;

    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.active_live_wallpaper = Some(path.to_path_buf());
    s.wallpaper_mode = "live".to_string();
    s.save().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

fn stop_live(settings: &SettingsState) -> Result<Settings, String> {
    windows::stop_live_wallpaper().map_err(|e| e.to_string())?;

    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.active_live_wallpaper = None;
    if s.wallpaper_mode == "live" {
        s.wallpaper_mode = "static".to_string();
    }
    s.save().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

fn next_wallpaper(settings: &SettingsState, state: &SharedState) -> Result<Settings, String> {
    let dir = settings
        .lock()
        .map_err(|e| e.to_string())?
        .wallpaper_dir
        .clone();
    let Some(dir) = dir else {
        return Err("No wallpaper folder is configured yet.".to_string());
    };

    let items = {
        let scanned = state
            .lock()
            .map_err(|e| e.to_string())?
            .static_wallpapers
            .clone();
        if scanned.is_empty() {
            scanner::scan_static(&dir, state.clone()).map_err(|e| e.to_string())?;
            state
                .lock()
                .map_err(|e| e.to_string())?
                .static_wallpapers
                .clone()
        } else {
            scanned
        }
    };
    if items.is_empty() {
        return Err("No wallpapers found to pick from.".to_string());
    }

    let current = settings
        .lock()
        .map_err(|e| e.to_string())?
        .active_static_wallpaper
        .clone();
    let mut candidates: Vec<&WallpaperItem> = items
        .iter()
        .filter(|item| Some(&item.path) != current.as_ref())
        .collect();
    if candidates.is_empty() {
        candidates = items.iter().collect();
    }

    let pick = candidates[rand::random_range(0..candidates.len())]
        .path
        .clone();
    apply_static(settings, &pick)
}

fn toggle_pause(settings: &SettingsState) -> Result<bool, String> {
    let path = settings
        .lock()
        .map_err(|e| e.to_string())?
        .active_live_wallpaper
        .clone();
    let Some(path) = path else {
        return Ok(false);
    };

    if windows::is_live_wallpaper_active() {
        windows::pause_for_fullscreen().map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        windows::resume_after_fullscreen(&path).map_err(|e| e.to_string())?;
        Ok(false)
    }
}

/// Reveals `path` in Windows Explorer with the file pre-selected.
pub fn open_in_explorer(path: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut cmd = std::process::Command::new("explorer");
    cmd.creation_flags(CREATE_NO_WINDOW);
    if path.is_dir() {
        cmd.arg(path);
    } else {
        cmd.args(["/select,", &path.to_string_lossy()]);
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}
