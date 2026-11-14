//! Application state and message handling for the egui UI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use eframe::egui;

use rexpaper_core::platform::tray::{TrayAction, set_tray_paused};
use rexpaper_core::{AppState, LiveWallpaperItem, Settings, SharedState, WallpaperItem};

use crate::state::{LibraryKind, Msg, SettingsState, Worker};
use crate::{state, theme, ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Static,
    Live,
    Settings,
}

pub enum ApplyTarget {
    Static,
    Live,
    Next,
}

pub enum ThumbState {
    Pending,
    Failed,
    Ready(egui::TextureHandle),
}

pub struct RexApp {
    pub ctx: egui::Context,
    pub settings: Settings,
    pub settings_state: SettingsState,
    pub state: SharedState,
    pub worker: Worker,
    rx: Receiver<Msg>,

    pub route: Route,
    pub static_items: Vec<WallpaperItem>,
    pub live_items: Vec<LiveWallpaperItem>,
    pub static_categories: Vec<String>,
    pub live_categories: Vec<String>,
    pub static_category: String,
    pub live_category: String,
    pub static_search: String,
    pub live_search: String,
    pub scan_counts: Option<(usize, usize)>,

    pub scanning: bool,
    generation: u64,
    scanned_dirs: Option<(Option<PathBuf>, Option<PathBuf>)>,
    pub scan_error: Option<String>,
    pub gallery_error: Option<String>,
    pub settings_error: Option<String>,

    pub applying: Option<ApplyTarget>,
    pub stopping: bool,
    pub pausing: bool,
    pub live_paused: bool,

    pub thumbs: HashMap<PathBuf, ThumbState>,
    thumb_job: Option<LibraryKind>,

    pub show_about: bool,
    initial_fit: bool,
}

impl RexApp {
    pub fn new(cc: &eframe::CreationContext<'_>, _start_hidden: bool) -> Self {
        theme::apply(&cc.egui_ctx);

        let settings_state: SettingsState = Arc::new(Mutex::new(Settings::load()));
        let settings = settings_state.lock().map(|s| s.clone()).unwrap_or_default();
        let state = AppState::new();

        let (tx, rx) = mpsc::channel();
        let worker = Worker::new(tx, cc.egui_ctx.clone());

        let mut app = Self {
            ctx: cc.egui_ctx.clone(),
            settings,
            settings_state,
            state,
            worker,
            rx,
            route: Route::Static,
            static_items: Vec::new(),
            live_items: Vec::new(),
            static_categories: vec!["All".to_string()],
            live_categories: vec!["All".to_string()],
            static_category: "All".to_string(),
            live_category: "All".to_string(),
            static_search: String::new(),
            live_search: String::new(),
            scan_counts: None,
            scanning: false,
            generation: 0,
            scanned_dirs: None,
            scan_error: None,
            gallery_error: None,
            settings_error: None,
            applying: None,
            stopping: false,
            pausing: false,
            live_paused: false,
            thumbs: HashMap::new(),
            thumb_job: None,
            show_about: false,
            initial_fit: true,
        };

        app.start_background();
        app
    }

    fn start_background(&mut self) {
        let worker = self.worker.clone();
        if let Err(e) = rexpaper_core::platform::tray::setup_tray(move |action| {
            worker.send(Msg::Tray(action));
        }) {
            eprintln!("[RexPaper] tray setup failed: {e}");
        }

        let pause_worker = self.worker.clone();
        rexpaper_core::platform::windows::start_fullscreen_monitor(
            self.settings_state.clone(),
            move |paused| pause_worker.send(Msg::LivePaused(paused)),
        );

        state::spawn_restore(self.settings_state.clone());
    }

    // --- Commands ---------------------------------------------------------

    pub fn start_scan(&mut self) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.scan_error = None;
        self.generation += 1;
        self.thumbs.clear();
        self.thumb_job = None;

        let dirs = (
            self.settings.wallpaper_dir.clone(),
            self.settings.live_wallpaper_dir.clone(),
        );
        state::spawn_scan(
            self.worker.clone(),
            self.state.clone(),
            self.generation,
            dirs,
        );
    }

    pub fn choose_folder(&mut self, kind: LibraryKind) {
        state::spawn_pick_folder(self.worker.clone(), self.settings_state.clone(), kind);
    }

    pub fn apply_static(&mut self, path: PathBuf) {
        if self.applying.is_some() {
            return;
        }
        self.gallery_error = None;
        self.applying = Some(ApplyTarget::Static);
        state::spawn_apply_static(self.worker.clone(), self.settings_state.clone(), path);
    }

    pub fn apply_live(&mut self, path: PathBuf) {
        if self.applying.is_some() || self.stopping {
            return;
        }
        self.gallery_error = None;
        self.applying = Some(ApplyTarget::Live);
        state::spawn_apply_live(self.worker.clone(), self.settings_state.clone(), path);
    }

    pub fn stop_live(&mut self) {
        if self.stopping {
            return;
        }
        self.stopping = true;
        self.gallery_error = None;
        state::spawn_stop_live(self.worker.clone(), self.settings_state.clone());
    }

    pub fn apply_random(&mut self) {
        if self.applying.is_some() {
            return;
        }
        self.settings_error = None;
        self.applying = Some(ApplyTarget::Next);
        state::spawn_next_wallpaper(
            self.worker.clone(),
            self.settings_state.clone(),
            self.state.clone(),
        );
    }

    pub fn toggle_pause(&mut self) {
        if self.pausing {
            return;
        }
        self.pausing = true;
        self.settings_error = None;
        state::spawn_toggle_pause(self.worker.clone(), self.settings_state.clone());
    }

    pub fn set_run_on_startup(&mut self, enabled: bool) {
        if let Ok(mut s) = self.settings_state.lock() {
            s.set_run_on_startup(enabled);
            self.settings = s.clone();
        }
    }

    pub fn set_pause_on_fullscreen(&mut self, enabled: bool) {
        if let Ok(mut s) = self.settings_state.lock() {
            s.set_pause_on_fullscreen(enabled);
            self.settings = s.clone();
        }
    }

    pub fn set_mute_live_wallpapers(&mut self, enabled: bool) {
        if let Ok(mut s) = self.settings_state.lock() {
            s.set_mute_live_wallpapers(enabled);
            self.settings = s.clone();
        }
    }

    pub fn open_folder(&mut self, path: &Path) {
        self.settings_error = None;
        if let Err(e) = state::open_in_explorer(path) {
            self.settings_error = Some(e);
        }
    }

    // --- Filtering --------------------------------------------------------

    pub fn filtered_static(&self) -> Vec<WallpaperItem> {
        self.static_items
            .iter()
            .filter(|item| {
                matches_filter(&item.path, &item.category, &self.static_search)
                    && (self.static_category == "All" || item.category == self.static_category)
            })
            .cloned()
            .collect()
    }

    pub fn filtered_live(&self) -> Vec<LiveWallpaperItem> {
        self.live_items
            .iter()
            .filter(|item| {
                matches_filter(&item.path, &item.category, &self.live_search)
                    && (self.live_category == "All" || item.category == self.live_category)
            })
            .cloned()
            .collect()
    }

    fn maybe_auto_scan(&mut self) {
        if self.scanning {
            return;
        }
        let dirs = (
            self.settings.wallpaper_dir.clone(),
            self.settings.live_wallpaper_dir.clone(),
        );
        if dirs.0.is_none() && dirs.1.is_none() {
            return;
        }
        if self.scanned_dirs.as_ref() != Some(&dirs) {
            self.start_scan();
        }
    }

    fn ensure_page_thumbnails(&mut self) {
        if self.thumb_job.is_some() {
            return;
        }
        let (kind, paths): (LibraryKind, Vec<PathBuf>) = match self.route {
            Route::Static => (
                LibraryKind::Static,
                self.filtered_static().into_iter().map(|w| w.path).collect(),
            ),
            Route::Live => (
                LibraryKind::Live,
                self.filtered_live().into_iter().map(|w| w.path).collect(),
            ),
            Route::Settings => return,
        };

        let missing: Vec<PathBuf> = paths
            .into_iter()
            .filter(|path| !self.thumbs.contains_key(path))
            .collect();
        if missing.is_empty() {
            return;
        }

        for path in &missing {
            self.thumbs.insert(path.clone(), ThumbState::Pending);
        }
        self.thumb_job = Some(kind);
        state::spawn_thumbnails(self.worker.clone(), missing, kind, self.generation);
    }

    // --- Message handling -------------------------------------------------

    fn handle(&mut self, msg: Msg) {
        match msg {
            Msg::Scanned {
                generation,
                dirs,
                static_items,
                live_items,
                static_categories,
                live_categories,
                error,
            } => {
                if generation != self.generation {
                    return;
                }
                self.scanning = false;
                self.scan_error = error;
                self.scan_counts = Some((static_items.len(), live_items.len()));
                self.static_items = static_items;
                self.live_items = live_items;
                self.static_categories = static_categories;
                self.live_categories = live_categories;
                self.scanned_dirs = Some(dirs);
                self.thumbs.clear();
                self.thumb_job = None;
                if !self.static_categories.contains(&self.static_category) {
                    self.static_category = "All".to_string();
                }
                if !self.live_categories.contains(&self.live_category) {
                    self.live_category = "All".to_string();
                }
            }
            Msg::ScanFailed {
                generation,
                message,
            } => {
                if generation != self.generation {
                    return;
                }
                self.scanning = false;
                self.scan_error = Some(message);
            }
            Msg::Thumbs {
                generation,
                items,
                remaining,
            } => {
                if generation != self.generation {
                    return;
                }
                for (path, image) in items {
                    match image {
                        Some(image) => {
                            let texture = self.ctx.load_texture(
                                format!("thumb:{}", path.display()),
                                image,
                                egui::TextureOptions::LINEAR,
                            );
                            self.thumbs.insert(path, ThumbState::Ready(texture));
                        }
                        None => {
                            self.thumbs.insert(path, ThumbState::Failed);
                        }
                    }
                }
                if remaining == 0 {
                    self.thumb_job = None;
                }
            }
            Msg::LivePaused(paused) => {
                self.live_paused = paused;
                set_tray_paused(paused);
            }
            Msg::ApplyDone(result) => {
                self.applying = None;
                match result {
                    Ok(settings) => {
                        self.settings = settings;
                        self.gallery_error = None;
                        self.settings_error = None;
                    }
                    Err(e) => self.report_error(e),
                }
            }
            Msg::StopDone(result) => {
                self.stopping = false;
                match result {
                    Ok(settings) => {
                        self.settings = settings;
                        self.gallery_error = None;
                        self.settings_error = None;
                    }
                    Err(e) => self.report_error(e),
                }
            }
            Msg::PauseDone(result) => {
                self.pausing = false;
                match result {
                    Ok(paused) => {
                        self.live_paused = paused;
                        set_tray_paused(paused);
                        self.settings_error = None;
                    }
                    Err(e) => self.report_error(e),
                }
            }
            Msg::PickDone { kind, result } => match result {
                Ok(Some(settings)) => {
                    self.settings = settings;
                    self.generation += 1;
                    self.thumbs.clear();
                    self.thumb_job = None;
                    self.scanned_dirs = None;
                    match kind {
                        LibraryKind::Static => {
                            self.static_items.clear();
                            self.static_categories = vec!["All".to_string()];
                            self.static_category = "All".to_string();
                            self.static_search.clear();
                        }
                        LibraryKind::Live => {
                            self.live_items.clear();
                            self.live_categories = vec!["All".to_string()];
                            self.live_category = "All".to_string();
                            self.live_search.clear();
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => self.settings_error = Some(e),
            },
            Msg::Tray(action) => self.handle_tray(action),
        }
    }

    fn handle_tray(&mut self, action: TrayAction) {
        match action {
            TrayAction::Open => self.show_window(),
            TrayAction::StaticPage => {
                self.route = Route::Static;
                self.show_window();
            }
            TrayAction::LivePage => {
                self.route = Route::Live;
                self.show_window();
            }
            TrayAction::SettingsPage => {
                self.route = Route::Settings;
                self.show_window();
            }
            TrayAction::TogglePause => self.toggle_pause(),
            TrayAction::NextWallpaper => self.apply_random(),
            TrayAction::OpenFolder => {
                if let Some(dir) = self.settings.wallpaper_dir.clone() {
                    let _ = state::open_in_explorer(&dir);
                }
            }
            TrayAction::Quit => self.ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    fn show_window(&self) {
        self.ctx
            .send_viewport_cmd(egui::ViewportCommand::Visible(true));
        self.ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    fn report_error(&mut self, message: String) {
        if self.route == Route::Settings {
            self.settings_error = Some(message);
        } else {
            self.gallery_error = Some(message);
        }
    }

    fn clear_search(&mut self) {
        match self.route {
            Route::Static => self.static_search.clear(),
            Route::Live => self.live_search.clear(),
            Route::Settings => {}
        }
    }
}

impl eframe::App for RexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.initial_fit
            && let Some(monitor) = ctx.input(|i| i.viewport().monitor_size)
            && monitor.x > 1.0
            && monitor.y > 1.0
        {
            let usable_height = monitor.y - 56.0;
            let size = egui::vec2(
                1200.0_f32.min(monitor.x - 80.0),
                800.0_f32.min(usable_height - 40.0),
            );
            let position = egui::pos2(
                ((monitor.x - size.x) / 2.0).max(0.0),
                ((usable_height - size.y) / 2.0).max(0.0) + 8.0,
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(position));
            self.initial_fit = false;
        }

        while let Ok(msg) = self.rx.try_recv() {
            self.handle(msg);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.show_about {
                self.show_about = false;
            } else {
                self.clear_search();
            }
        }

        self.maybe_auto_scan();
        self.ensure_page_thumbnails();

        ui::draw(self, ctx);
        ui::resize_handles(ctx);

        if self.scanning
            || self.applying.is_some()
            || self.stopping
            || self.pausing
            || self.thumb_job.is_some()
        {
            ctx.request_repaint_after(Duration::from_millis(120));
        }
    }
}

fn matches_filter(path: &Path, category: &str, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_lowercase();
    name.contains(&query) || category.to_lowercase().contains(&query)
}
