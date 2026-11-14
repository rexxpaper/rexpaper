//! Static, Live, and Settings pages.

use std::path::PathBuf;

use eframe::egui;

use crate::app::RexApp;
use crate::state::LibraryKind;
use crate::theme;
use crate::ui::{gallery, widgets};

pub fn static_page(app: &mut RexApp, ui: &mut egui::Ui) {
    if app.settings.wallpaper_dir.is_none() {
        if widgets::empty_state(
            ui,
            "No wallpaper folder selected",
            "Pick a folder to build your library. Subfolders become categories you can filter by.",
            "Choose folder",
        ) {
            app.choose_folder(LibraryKind::Static);
        }
        return;
    }

    if app.scanning && app.static_items.is_empty() {
        gallery::skeleton(ui);
        return;
    }

    let error = app.gallery_error.clone().or_else(|| app.scan_error.clone());

    if app.static_items.is_empty() {
        if let Some(error) = error {
            widgets::error_banner(ui, &error);
        }
        let hint = format!(
            "RexPaper found no images in {}.",
            app.settings
                .wallpaper_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        );
        if widgets::empty_state(ui, "No wallpapers found", &hint, "Choose another folder") {
            app.choose_folder(LibraryKind::Static);
        }
        return;
    }

    gallery::toolbar(app, ui, LibraryKind::Static);
    if let Some(error) = error {
        widgets::error_banner(ui, &error);
    }

    let filtered = app.filtered_static();
    if filtered.is_empty() {
        let title = format!(
            "No matches for “{}”",
            if app.static_search.trim().is_empty() {
                app.static_category.clone()
            } else {
                app.static_search.trim().to_string()
            }
        );
        if widgets::empty_state(
            ui,
            &title,
            "No wallpaper in this view matches the current search and category.",
            "Clear filters",
        ) {
            app.static_search.clear();
            app.static_category = "All".to_string();
        }
        return;
    }

    let cards = filtered
        .iter()
        .map(|item| {
            let active = app.settings.active_static_wallpaper.as_ref() == Some(&item.path);
            (item.path.clone(), active)
        })
        .collect();
    gallery::grid(app, ui, LibraryKind::Static, cards);
}

pub fn live_page(app: &mut RexApp, ui: &mut egui::Ui) {
    if app.settings.live_wallpaper_dir.is_none() {
        if widgets::empty_state(
            ui,
            "No video folder selected",
            "Pick a folder of video files (mp4, webm, mkv, …). Subfolders become categories you can filter by.",
            "Choose folder",
        ) {
            app.choose_folder(LibraryKind::Live);
        }
        return;
    }

    if app.scanning && app.live_items.is_empty() {
        gallery::skeleton(ui);
        return;
    }

    let error = app.gallery_error.clone().or_else(|| app.scan_error.clone());

    if app.live_items.is_empty() {
        if let Some(error) = error {
            widgets::error_banner(ui, &error);
        }
        let hint = format!(
            "RexPaper found no video files in {}.",
            app.settings
                .live_wallpaper_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        );
        if widgets::empty_state(ui, "No videos found", &hint, "Choose another folder") {
            app.choose_folder(LibraryKind::Live);
        }
        return;
    }

    gallery::toolbar(app, ui, LibraryKind::Live);
    if let Some(error) = error {
        widgets::error_banner(ui, &error);
    }

    let filtered = app.filtered_live();
    if filtered.is_empty() {
        let title = format!(
            "No matches for “{}”",
            if app.live_search.trim().is_empty() {
                app.live_category.clone()
            } else {
                app.live_search.trim().to_string()
            }
        );
        if widgets::empty_state(
            ui,
            &title,
            "No video in this view matches the current search and category.",
            "Clear filters",
        ) {
            app.live_search.clear();
            app.live_category = "All".to_string();
        }
        return;
    }

    let cards = filtered
        .iter()
        .map(|item| {
            let active = app.settings.wallpaper_mode == "live"
                && app.settings.active_live_wallpaper.as_ref() == Some(&item.path);
            (item.path.clone(), active)
        })
        .collect();
    gallery::grid(app, ui, LibraryKind::Live, cards);
}

pub fn settings_page(app: &mut RexApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(24.0);
            let pad = ((ui.available_width() - 768.0) / 2.0).max(24.0);
            ui.horizontal(|ui| {
                ui.add_space(pad);
                ui.vertical(|ui| {
                    ui.set_max_width(768.0);

                    if let Some(error) = app.settings_error.clone() {
                        widgets::banner(ui, &error);
                        ui.add_space(16.0);
                    }

                    libraries_section(app, ui);
                    behavior_section(app, ui);
                    wallpaper_section(app, ui);
                    about_section(app, ui);
                    ui.add_space(24.0);
                });
            });
        });
}

fn libraries_section(app: &mut RexApp, ui: &mut egui::Ui) {
    widgets::section(ui, "Libraries", |ui| {
        folder_row(
            app,
            ui,
            "Wallpaper folder",
            app.settings.wallpaper_dir.clone(),
            LibraryKind::Static,
        );
        folder_row(
            app,
            ui,
            "Live wallpaper folder",
            app.settings.live_wallpaper_dir.clone(),
            LibraryKind::Live,
        );

        let counts = app.scan_counts;
        let hint = counts
            .map(|(static_count, live_count)| format!("{static_count} static · {live_count} live"));
        let scanning = app.scanning;
        let mut scan = false;
        widgets::setting_row(ui, "Scan libraries", hint.as_deref(), |ui| {
            let label = if scanning {
                "Scanning…"
            } else {
                "Scan libraries"
            };
            if ui
                .add_enabled(!scanning, widgets::primary_button(label))
                .clicked()
            {
                scan = true;
            }
        });
        if scan {
            app.start_scan();
        }

        if let Some(error) = app.scan_error.clone() {
            widgets::banner(ui, &error);
        }
    });
}

fn behavior_section(app: &mut RexApp, ui: &mut egui::Ui) {
    widgets::section(ui, "Behavior", |ui| {
        let mut run_on_startup = app.settings.run_on_startup;
        let mut run_changed = false;
        widgets::setting_row(
            ui,
            "Run on startup",
            Some("Starts RexPaper with Windows and restores your wallpaper."),
            |ui| {
                if widgets::toggle(ui, &mut run_on_startup).changed() {
                    run_changed = true;
                }
            },
        );
        if run_changed {
            app.set_run_on_startup(run_on_startup);
        }

        let mut pause_on_fullscreen = app.settings.pause_on_fullscreen;
        let mut pause_changed = false;
        widgets::setting_row(
            ui,
            "Pause on fullscreen",
            Some("Live wallpapers pause while a game or video runs fullscreen."),
            |ui| {
                if widgets::toggle(ui, &mut pause_on_fullscreen).changed() {
                    pause_changed = true;
                }
            },
        );
        if pause_changed {
            app.set_pause_on_fullscreen(pause_on_fullscreen);
        }

        let mut mute = app.settings.mute_live_wallpapers;
        let mut mute_changed = false;
        widgets::setting_row(
            ui,
            "Mute live wallpapers",
            Some(
                "Previews are always muted; this affects the applied wallpaper. Applies next time a live wallpaper is set.",
            ),
            |ui| {
                if widgets::toggle(ui, &mut mute).changed() {
                    mute_changed = true;
                }
            },
        );
        if mute_changed {
            app.set_mute_live_wallpapers(mute);
        }
    });
}

fn wallpaper_section(app: &mut RexApp, ui: &mut egui::Ui) {
    widgets::section(ui, "Wallpaper", |ui| {
        let busy = app.applying.is_some();
        let mut apply = false;
        widgets::setting_row(
            ui,
            "Next wallpaper",
            Some("Applies a random wallpaper from your library."),
            |ui| {
                let label = if busy { "Applying…" } else { "Apply random" };
                if ui
                    .add_enabled(!busy, widgets::ghost_button(label))
                    .clicked()
                {
                    apply = true;
                }
            },
        );
        if apply {
            app.apply_random();
        }
    });
}

fn about_section(app: &mut RexApp, ui: &mut egui::Ui) {
    widgets::section(ui, "About", |ui| {
        let mut open = false;
        widgets::setting_row(
            ui,
            "RexPaper",
            Some("A fast, native wallpaper manager for Windows."),
            |ui| {
                if ui.add(widgets::ghost_button("About")).clicked() {
                    open = true;
                }
            },
        );
        if open {
            app.show_about = true;
        }
    });
}

fn folder_row(
    app: &mut RexApp,
    ui: &mut egui::Ui,
    label: &str,
    path: Option<PathBuf>,
    kind: LibraryKind,
) {
    let mut open = false;
    let mut change = false;
    let text = path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "Not set".to_string());

    widgets::setting_row(ui, label, None, |ui| {
        if path.is_some() && ui.add(widgets::ghost_button("Open")).clicked() {
            open = true;
        }
        if ui.add(widgets::ghost_button("Change…")).clicked() {
            change = true;
        }
        ui.add(
            egui::Label::new(egui::RichText::new(text).size(12.0).color(theme::MUTED)).truncate(),
        );
    });

    if change {
        app.choose_folder(kind);
    }
    if open && let Some(path) = &path {
        app.open_folder(path);
    }
}
