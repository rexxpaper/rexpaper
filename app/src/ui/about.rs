//! About modal.

use eframe::egui;

use crate::app::RexApp;
use crate::theme;
use crate::ui::widgets;

pub fn show(app: &mut RexApp, ctx: &egui::Context) {
    if !app.show_about {
        return;
    }

    let mut close = false;
    let modal = egui::Modal::new(egui::Id::new("about-modal")).show(ctx, |ui| {
        ui.set_width(320.0);
        ui.label(
            egui::RichText::new("RexPaper")
                .size(14.0)
                .strong()
                .color(theme::TEXT),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                .size(13.0)
                .color(theme::SECONDARY),
        );
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(
                "A fast, native wallpaper manager for Windows with hardware-accelerated live video wallpapers.",
            )
            .size(13.0)
            .color(theme::MUTED),
        );
        ui.add_space(16.0);
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if ui.add(widgets::ghost_button("Close")).clicked() {
                    close = true;
                }
            },
        );
    });

    if close || modal.should_close() {
        app.show_about = false;
    }
}
