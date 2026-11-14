//! Gallery toolbar, responsive card grid, and loading state.

use std::path::PathBuf;

use eframe::egui;

use crate::app::{RexApp, ThumbState};
use crate::state::LibraryKind;
use crate::theme;
use crate::ui::widgets;

const CARD_W: f32 = 176.0;
const GAP: f32 = 16.0;

pub fn toolbar(app: &mut RexApp, ui: &mut egui::Ui, kind: LibraryKind) {
    let (categories, mut category) = match kind {
        LibraryKind::Static => (app.static_categories.clone(), app.static_category.clone()),
        LibraryKind::Live => (app.live_categories.clone(), app.live_category.clone()),
    };
    let mut search = match kind {
        LibraryKind::Static => app.static_search.clone(),
        LibraryKind::Live => app.live_search.clone(),
    };

    let live_active =
        app.settings.wallpaper_mode == "live" && app.settings.active_live_wallpaper.is_some();

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);

        if categories.len() > 1 {
            egui::ComboBox::from_id_salt(("category", kind))
                .selected_text(if category == "All" {
                    "All subfolders"
                } else {
                    category.as_str()
                })
                .width(180.0)
                .show_ui(ui, |ui| {
                    for cat in &categories {
                        let label = if cat == "All" { "All subfolders" } else { cat };
                        ui.selectable_value(&mut category, cat.clone(), label);
                    }
                });
        }

        let label = if app.scanning {
            "Scanning…"
        } else {
            "Rescan"
        };
        if ui
            .add_enabled(!app.scanning, widgets::ghost_button(label))
            .clicked()
        {
            app.start_scan();
        }

        let (count, noun) = match kind {
            LibraryKind::Static => (app.static_items.len(), "wallpapers"),
            LibraryKind::Live => (app.live_items.len(), "videos"),
        };
        ui.label(
            egui::RichText::new(format!("{count} {noun}"))
                .size(12.0)
                .color(theme::MUTED),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let placeholder = match kind {
                LibraryKind::Static => "Search wallpapers…",
                LibraryKind::Live => "Search videos…",
            };
            ui.add_sized(
                [224.0, 30.0],
                egui::TextEdit::singleline(&mut search)
                    .hint_text(placeholder)
                    .font(egui::TextStyle::Body),
            );
            if !search.is_empty()
                && ui
                    .add(
                        egui::Button::new(egui::RichText::new("✕").size(11.0).color(theme::MUTED))
                            .frame(false),
                    )
                    .clicked()
            {
                search.clear();
            }

            if kind == LibraryKind::Live && live_active {
                let label = if app.stopping {
                    "Stopping…"
                } else {
                    "Stop live"
                };
                if ui
                    .add_enabled(!app.stopping, widgets::ghost_button(label))
                    .clicked()
                {
                    app.stop_live();
                }
            }

            if app.applying.is_some() {
                ui.label(
                    egui::RichText::new("Applying…")
                        .size(11.0)
                        .color(theme::MUTED),
                );
            }
        });

        ui.add_space(12.0);
    });
    ui.add_space(12.0);

    match kind {
        LibraryKind::Static => {
            app.static_search = search;
            app.static_category = category;
        }
        LibraryKind::Live => {
            app.live_search = search;
            app.live_category = category;
        }
    }

    ui.separator();
}

pub fn grid(app: &mut RexApp, ui: &mut egui::Ui, kind: LibraryKind, cards: Vec<(PathBuf, bool)>) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(24.0);
            let content = (ui.available_width() - 48.0).max(CARD_W);
            let columns = (((content + GAP) / (CARD_W + GAP)).floor() as usize).max(1);

            for chunk in cards.chunks(columns) {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    for (path, active) in chunk {
                        card(ui, app, kind, path, *active);
                        ui.add_space(GAP);
                    }
                });
                ui.add_space(GAP);
            }
            ui.add_space(24.0);
        });
}

pub fn skeleton(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(24.0);
            let content = (ui.available_width() - 48.0).max(CARD_W);
            let columns = (((content + GAP) / (CARD_W + GAP)).floor() as usize).max(1);
            let height = CARD_W * 9.0 / 16.0;

            for _ in 0..3 {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    for _ in 0..columns {
                        let (rect, _) = ui
                            .allocate_exact_size(egui::vec2(CARD_W, height), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(rect, egui::CornerRadius::same(8), theme::CARD);
                        ui.painter().rect_stroke(
                            rect,
                            egui::CornerRadius::same(8),
                            egui::Stroke::new(1.0_f32, theme::BORDER),
                            egui::StrokeKind::Inside,
                        );
                        ui.add_space(GAP);
                    }
                });
                ui.add_space(GAP);
            }
            ui.add_space(24.0);
        });
}

fn card(ui: &mut egui::Ui, app: &mut RexApp, kind: LibraryKind, path: &PathBuf, active: bool) {
    let height = CARD_W * 9.0 / 16.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(CARD_W, height), egui::Sense::click());
    let hovered = response.hovered();

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, egui::CornerRadius::same(8), theme::CARD);

        match app.thumbs.get(path) {
            Some(ThumbState::Ready(texture)) => {
                let image_rect = rect.shrink(1.0);
                painter.with_clip_rect(image_rect).image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            Some(ThumbState::Failed) => {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No preview",
                    egui::FontId::proportional(12.0),
                    theme::MUTED,
                );
            }
            Some(ThumbState::Pending) | None => {
                painter.rect_filled(rect.shrink(1.0), egui::CornerRadius::same(7), theme::PANEL);
            }
        }

        let stroke = if active {
            egui::Stroke::new(1.5_f32, theme::TEXT)
        } else if hovered {
            egui::Stroke::new(1.0_f32, theme::SECONDARY)
        } else {
            egui::Stroke::new(1.0_f32, theme::BORDER)
        };
        painter.rect_stroke(
            rect,
            egui::CornerRadius::same(8),
            stroke,
            egui::StrokeKind::Inside,
        );
    }

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let response = response.on_hover_text(name);

    if response.clicked() {
        match kind {
            LibraryKind::Static => app.apply_static(path.clone()),
            LibraryKind::Live => app.apply_live(path.clone()),
        }
    }
}
