#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use std::sync::{Arc, Mutex};

use rexpaper_core::{AppState, Settings};
use tauri::Emitter;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;

const TRAY_ID: &str = "main-tray";

fn main() {
    // Settings are shared with the installed v1 app via the same
    // ProjectDirs config file; don't run both apps at once.
    let settings = Arc::new(Mutex::new(Settings::load()));
    let app_state = AppState::new();

    let startup_dirs: Vec<std::path::PathBuf> = {
        let s = settings.lock().expect("settings lock at startup");
        [s.wallpaper_dir.clone(), s.live_wallpaper_dir.clone()]
            .into_iter()
            .flatten()
            .collect()
    };

    // Autostart launches stay in the tray (stories 15/25).
    let start_hidden = std::env::args()
        .skip(1)
        .any(|arg| arg == "--autostart" || arg == "--minimized");

    tauri::Builder::default()
        .manage(settings.clone())
        .manage(app_state)
        .setup(move |app| {
            // Thumbnails and video previews load over the asset protocol;
            // grant the cache and the configured libraries up front.
            let scope = app.asset_protocol_scope();
            let _ = scope.allow_directory(rexpaper_core::thumbnail::get_cache_dir(), true);
            for dir in &startup_dirs {
                let _ = scope.allow_directory(dir, true);
            }

            if start_hidden {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            build_tray(app)?;

            // Restore the previous wallpaper state (story 21) without
            // blocking window creation: static first, live on top.
            {
                let settings = settings.clone();
                std::thread::Builder::new()
                    .name("rexpaper-restore".to_string())
                    .spawn(move || {
                        let (static_path, live_path) = {
                            let s = settings.lock().expect("settings lock at restore");
                            let live = if s.wallpaper_mode == "live" {
                                s.active_live_wallpaper.clone()
                            } else {
                                None
                            };
                            (s.active_static_wallpaper.clone(), live)
                        };
                        if let Some(path) = static_path.filter(|p| p.exists()) {
                            let _ = rexpaper_core::static_wallpaper::apply_static_wallpaper(&path);
                        }
                        if let Some(path) = live_path.filter(|p| p.exists()) {
                            let _ = rexpaper_core::platform::windows::apply_live_wallpaper(&path);
                        }
                    })
                    .ok();
            }

            // Fullscreen auto-pause (story 22), driving the UI through
            // `live-paused` events.
            let emit_handle = app.handle().clone();
            rexpaper_core::platform::windows::start_fullscreen_monitor(
                settings.clone(),
                move |paused| {
                    let _ = emit_handle.emit("live-paused", paused);
                },
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::set_wallpaper_dir,
            commands::set_live_wallpaper_dir,
            commands::set_run_on_startup,
            commands::set_pause_on_fullscreen,
            commands::set_mute_live_wallpapers,
            commands::pick_wallpaper_dir,
            commands::pick_live_wallpaper_dir,
            commands::scan_static,
            commands::scan_live,
            commands::apply_static,
            commands::apply_live,
            commands::stop_live,
            commands::next_wallpaper,
            commands::open_in_explorer,
            commands::toggle_live_pause,
            commands::resolve_thumbnails,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// --- Tray ---------------------------------------------------------------------

/// Shows the main window, optionally navigating it to `route` first so tray
/// actions land on the right page (story 15).
fn focus_main_window(app: &tauri::AppHandle, route: Option<&str>) {
    if let Some(route) = route {
        let _ = app.emit("route", route);
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn tray_next_wallpaper(app: &tauri::AppHandle) {
    // Apply runs on a worker: BMP conversion would otherwise stall the
    // menu thread. The UI follows via `settings-changed`.
    let app = app.clone();
    std::thread::Builder::new()
        .name("rexpaper-tray-next".to_string())
        .spawn(move || match commands::next_wallpaper_impl(&app) {
            Ok(updated) => {
                let _ = app.emit("settings-changed", updated);
            }
            Err(e) => eprintln!("[RexPaper] tray next wallpaper failed: {e}"),
        })
        .ok();
}

fn tray_open_folder(app: &tauri::AppHandle) {
    let dir = app
        .state::<commands::SettingsState>()
        .lock()
        .map(|s| s.wallpaper_dir.clone())
        .ok()
        .flatten();
    if let Some(dir) = dir {
        let _ = commands::open_in_explorer(dir.to_string_lossy().into_owned());
    }
}

/// The 8-item tray menu (story 13): Open, the three pages, Pause Live
/// Wallpaper, Next Wallpaper, Open Wallpaper Folder, and Quit.
fn build_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let static_page = MenuItem::with_id(app, "static", "Static Page", true, None::<&str>)?;
    let live_page = MenuItem::with_id(app, "live", "Live Page", true, None::<&str>)?;
    let settings_page = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let pause_item = MenuItem::with_id(app, "pause", "Pause Live Wallpaper", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "Next Wallpaper", true, None::<&str>)?;
    let open_folder = MenuItem::with_id(app, "folder", "Open Wallpaper Folder", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &open,
            &static_page,
            &live_page,
            &settings_page,
            &separator,
            &pause_item,
            &next,
            &open_folder,
            &separator,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(
            app.default_window_icon()
                .expect("bundled window icon")
                .clone(),
        )
        .tooltip("RexPaper")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "open" => focus_main_window(app, None),
            "static" => focus_main_window(app, Some("static")),
            "live" => focus_main_window(app, Some("live")),
            "settings" => focus_main_window(app, Some("settings")),
            "pause" => match commands::toggle_live_pause_impl(app) {
                Ok(paused) => {
                    let _ = pause_item.set_text(if paused {
                        "Resume Live Wallpaper"
                    } else {
                        "Pause Live Wallpaper"
                    });
                }
                Err(e) => eprintln!("[RexPaper] tray pause failed: {e}"),
            },
            "next" => tray_next_wallpaper(app),
            "folder" => tray_open_folder(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
