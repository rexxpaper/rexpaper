#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use std::sync::{Arc, Mutex};

use rexpaper_core::{AppState, Settings};
use tauri::Manager;

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

    tauri::Builder::default()
        .manage(settings)
        .manage(app_state)
        .setup(move |app| {
            // Thumbnails and video previews load over the asset protocol;
            // grant the cache and the configured libraries up front.
            let scope = app.asset_protocol_scope();
            let _ = scope.allow_directory(rexpaper_core::thumbnail::get_cache_dir(), true);
            for dir in &startup_dirs {
                let _ = scope.allow_directory(dir, true);
            }
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
            commands::resolve_thumbnails,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
