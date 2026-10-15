//! IPC command layer between the Preact frontend and rexpaper-core.
//!
//! Contract: keep these signatures in sync with `frontend/src/api.ts`.
//! Invoke argument names are deliberately single-word (`path`, `enabled`)
//! so JS/Rust naming-conversion rules can never diverge. Mutating commands
//! return the full updated `Settings` snapshot so the frontend can
//! re-assign its store in one step.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager, State};

use rexpaper_core::settings::Settings;
use rexpaper_core::{LiveWallpaperItem, SharedState, WallpaperItem};

/// Managed by the Tauri app; the same settings state v1 keeps behind its
/// window callbacks.
pub type SettingsState = Arc<Mutex<Settings>>;

#[tauri::command]
pub fn get_settings(settings: State<'_, SettingsState>) -> Result<Settings, String> {
    let s = settings.lock().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

#[tauri::command]
pub fn set_wallpaper_dir(
    path: String,
    settings: State<'_, SettingsState>,
) -> Result<Settings, String> {
    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.set_wallpaper_dir(PathBuf::from(path));
    Ok(s.clone())
}

#[tauri::command]
pub fn set_live_wallpaper_dir(
    path: String,
    settings: State<'_, SettingsState>,
) -> Result<Settings, String> {
    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.set_live_wallpaper_dir(PathBuf::from(path));
    Ok(s.clone())
}

#[tauri::command]
pub fn set_run_on_startup(
    enabled: bool,
    settings: State<'_, SettingsState>,
) -> Result<Settings, String> {
    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.set_run_on_startup(enabled);
    Ok(s.clone())
}

#[tauri::command]
pub fn set_pause_on_fullscreen(
    enabled: bool,
    settings: State<'_, SettingsState>,
) -> Result<Settings, String> {
    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.set_pause_on_fullscreen(enabled);
    Ok(s.clone())
}

#[tauri::command]
pub fn set_mute_live_wallpapers(
    enabled: bool,
    settings: State<'_, SettingsState>,
) -> Result<Settings, String> {
    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.set_mute_live_wallpapers(enabled);
    Ok(s.clone())
}

/// Opens a native folder picker for the static wallpaper library. Returns the
/// updated `Settings`, or `null` when the dialog is cancelled. The chosen
/// directory is granted to the asset protocol so thumbnails resolve.
#[tauri::command(async)]
pub async fn pick_wallpaper_dir(
    app: tauri::AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<Option<Settings>, String> {
    let Some(handle) = rfd::AsyncFileDialog::new()
        .set_title("Choose wallpaper folder")
        .pick_folder()
        .await
    else {
        return Ok(None);
    };

    let path = handle.path().to_path_buf();
    app.asset_protocol_scope()
        .allow_directory(&path, true)
        .map_err(|e| e.to_string())?;

    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.set_wallpaper_dir(path);
    Ok(Some(s.clone()))
}

/// Same contract as [`pick_wallpaper_dir`] for the live video library.
#[tauri::command(async)]
pub async fn pick_live_wallpaper_dir(
    app: tauri::AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<Option<Settings>, String> {
    let Some(handle) = rfd::AsyncFileDialog::new()
        .set_title("Choose video folder")
        .pick_folder()
        .await
    else {
        return Ok(None);
    };

    let path = handle.path().to_path_buf();
    app.asset_protocol_scope()
        .allow_directory(&path, true)
        .map_err(|e| e.to_string())?;

    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.set_live_wallpaper_dir(path);
    Ok(Some(s.clone()))
}

/// Scans `settings.wallpaper_dir` and returns the flat item list; returns an
/// empty list when no directory has been configured yet. Heavy: directory
/// walk, so it runs off the main thread. The directory is granted to the
/// asset protocol so its thumbnails can load.
#[tauri::command(async)]
pub fn scan_static(
    app: tauri::AppHandle,
    settings: State<'_, SettingsState>,
    state: State<'_, SharedState>,
) -> Result<Vec<WallpaperItem>, String> {
    let dir = settings
        .lock()
        .map_err(|e| e.to_string())?
        .wallpaper_dir
        .clone();
    let Some(dir) = dir else {
        return Ok(Vec::new());
    };

    app.asset_protocol_scope()
        .allow_directory(&dir, true)
        .map_err(|e| e.to_string())?;

    rexpaper_core::scanner::scan_static(&dir, state.inner().clone()).map_err(|e| e.to_string())?;

    let items = state
        .lock()
        .map_err(|e| e.to_string())?
        .static_wallpapers
        .clone();
    Ok(items)
}

/// Same contract as [`scan_static`] for `settings.live_wallpaper_dir`. Also
/// grants the directory so hover previews stream over the asset protocol.
#[tauri::command(async)]
pub fn scan_live(
    app: tauri::AppHandle,
    settings: State<'_, SettingsState>,
    state: State<'_, SharedState>,
) -> Result<Vec<LiveWallpaperItem>, String> {
    let dir = settings
        .lock()
        .map_err(|e| e.to_string())?
        .live_wallpaper_dir
        .clone();
    let Some(dir) = dir else {
        return Ok(Vec::new());
    };

    app.asset_protocol_scope()
        .allow_directory(&dir, true)
        .map_err(|e| e.to_string())?;

    rexpaper_core::scanner::scan_live(&dir, state.inner().clone()).map_err(|e| e.to_string())?;

    let items = state
        .lock()
        .map_err(|e| e.to_string())?
        .live_wallpapers
        .clone();
    Ok(items)
}

/// Applies `path` as the static wallpaper and records it in settings,
/// mirroring v1's `on_apply_static_wallpaper`. Blocking (live-wallpaper
/// teardown + BMP conversion), so it runs off the main thread.
#[tauri::command(async)]
pub fn apply_static(
    path: String,
    settings: State<'_, SettingsState>,
) -> Result<Settings, String> {
    let wallpaper = PathBuf::from(path);
    rexpaper_core::static_wallpaper::apply_static_wallpaper(&wallpaper)
        .map_err(|e| e.to_string())?;

    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.active_live_wallpaper = None;
    s.active_static_wallpaper = Some(wallpaper);
    s.wallpaper_mode = "static".to_string();
    s.save().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

/// Applies `path` as the live desktop wallpaper (mpv behind the desktop
/// icons) and records it in settings. Blocking (host window resolution +
/// mpv spawn), so it runs off the main thread. `active_static_wallpaper` is
/// deliberately kept: the Windows static background stays underneath, and
/// [`stop_live`] returns to it.
#[tauri::command(async)]
pub fn apply_live(path: String, settings: State<'_, SettingsState>) -> Result<Settings, String> {
    let wallpaper = PathBuf::from(path);
    // The core spawns mpv muted/unmuted per `mute_live_wallpapers` read from
    // disk; that setting is saved on toggle, so nothing to flush here.
    rexpaper_core::platform::windows::apply_live_wallpaper(&wallpaper)
        .map_err(|e| e.to_string())?;

    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.active_live_wallpaper = Some(wallpaper);
    s.wallpaper_mode = "live".to_string();
    s.save().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

/// Stops the live desktop wallpaper; the static Windows background beneath
/// it is revealed (or `active_static_wallpaper` if one was applied before).
#[tauri::command(async)]
pub fn stop_live(settings: State<'_, SettingsState>) -> Result<Settings, String> {
    rexpaper_core::platform::windows::stop_live_wallpaper().map_err(|e| e.to_string())?;

    let mut s = settings.lock().map_err(|e| e.to_string())?;
    s.active_live_wallpaper = None;
    if s.wallpaper_mode == "live" {
        s.wallpaper_mode = "static".to_string();
    }
    s.save().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

// --- Thumbnail resolution -------------------------------------------------

/// One wallpaper path and its resolved cache thumbnail (`null` = generation
/// failed or not possible for this file).
#[derive(Clone, serde::Serialize)]
pub struct ThumbnailEntry {
    pub path: String,
    pub thumb: Option<String>,
}

/// Response of [`resolve_thumbnails`]: thumbnails that already exist, plus
/// the paths queued for background generation.
#[derive(Clone, serde::Serialize)]
pub struct ThumbnailResolution {
    pub ready: Vec<ThumbnailEntry>,
    pub pending: Vec<String>,
}

/// Payload of the `thumbnail-progress` event, emitted once per generated batch.
#[derive(Clone, serde::Serialize)]
pub struct ThumbnailBatch {
    pub items: Vec<ThumbnailEntry>,
    pub remaining: usize,
}

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThumbnailKind {
    Static,
    Live,
}

/// Serializes thumbnail generation to one worker at a time: a second resolve
/// waits here and then reports the already-valid cache files instead of two
/// threads racing on the same output file.
static THUMBNAIL_LOCK: Mutex<()> = Mutex::new(());

fn thumbnail_path(kind: ThumbnailKind, wallpaper: &Path) -> Option<PathBuf> {
    match kind {
        ThumbnailKind::Static => rexpaper_core::thumbnail::static_thumbnail_path(wallpaper),
        ThumbnailKind::Live => rexpaper_core::thumbnail::video_thumbnail_path(wallpaper),
    }
}

fn precompute(kind: ThumbnailKind, paths: &[PathBuf]) {
    match kind {
        ThumbnailKind::Static => rexpaper_core::thumbnail::precompute_static_thumbnails(paths),
        ThumbnailKind::Live => rexpaper_core::thumbnail::precompute_video_thumbnails(paths),
    }
}

/// Resolves cached thumbnails for `paths` and starts background generation
/// for the misses. Existing thumbnails return immediately; generated ones
/// arrive later via `thumbnail-progress` events.
#[tauri::command(async)]
pub fn resolve_thumbnails(
    paths: Vec<String>,
    kind: ThumbnailKind,
    app: tauri::AppHandle,
) -> Result<ThumbnailResolution, String> {
    let mut ready = Vec::new();
    let mut work: Vec<(String, PathBuf)> = Vec::new();

    for path in paths {
        let wallpaper = PathBuf::from(&path);
        match thumbnail_path(kind, &wallpaper) {
            Some(thumb) => ready.push(ThumbnailEntry {
                path,
                thumb: Some(thumb.to_string_lossy().into_owned()),
            }),
            None => work.push((path, wallpaper)),
        }
    }

    if !work.is_empty() {
        let pending: Vec<String> = work.iter().map(|(original, _)| original.clone()).collect();
        std::thread::Builder::new()
            .name("rexpaper-thumbs".to_string())
            .spawn(move || generate_thumbnails(work, kind, app))
            .map_err(|e| e.to_string())?;
        return Ok(ThumbnailResolution { ready, pending });
    }

    Ok(ThumbnailResolution {
        ready,
        pending: Vec::new(),
    })
}

/// Generates thumbnails in batches on a worker thread, emitting one
/// `thumbnail-progress` event per batch so the UI can drain its pending set.
fn generate_thumbnails(paths: Vec<(String, PathBuf)>, kind: ThumbnailKind, app: tauri::AppHandle) {
    let _guard = THUMBNAIL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let batch_size = match kind {
        ThumbnailKind::Static => 24,
        ThumbnailKind::Live => 8,
    };
    let total = paths.len();
    let mut processed = 0usize;

    for chunk in paths.chunks(batch_size) {
        let originals: Vec<PathBuf> = chunk.iter().map(|(_, wallpaper)| wallpaper.clone()).collect();
        precompute(kind, &originals);

        let items = chunk
            .iter()
            .map(|(original, wallpaper)| ThumbnailEntry {
                path: original.clone(),
                thumb: thumbnail_path(kind, wallpaper).map(|t| t.to_string_lossy().into_owned()),
            })
            .collect();

        processed += chunk.len();
        let _ = app.emit(
            "thumbnail-progress",
            ThumbnailBatch {
                items,
                remaining: total - processed,
            },
        );
    }
}
