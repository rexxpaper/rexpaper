//! Small reusable egui widgets.

use eframe::egui;

use crate::theme;

pub fn ghost_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).size(13.0).color(theme::TEXT))
        .fill(theme::CARD)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(6))
}

pub fn primary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).size(13.0).color(theme::BG))
        .fill(theme::PRIMARY)
        .stroke(egui::Stroke::new(1.0_f32, theme::PRIMARY))
        .corner_radius(egui::CornerRadius::same(6))
}

/// Section card with a small muted heading.
pub fn section(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(theme::CARD)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(title)
                    .size(13.0)
                    .color(theme::SECONDARY),
            );
            ui.add_space(4.0);
            add(ui);
        });
    ui.add_space(16.0);
}

/// Hairline-divided property row: label (+ hint) on the left, controls right.
pub fn setting_row(
    ui: &mut egui::Ui,
    label: &str,
    hint: Option<&str>,
    add: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).size(14.0).color(theme::TEXT));
            if let Some(hint) = hint {
                ui.add_space(1.0);
                ui.label(egui::RichText::new(hint).size(11.0).color(theme::MUTED));
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add(ui)
        });
    });
    ui.separator();
}

/// Animated 36x20 pill toggle.
pub fn toggle(ui: &mut egui::Ui, value: &mut bool) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(egui::vec2(36.0, 20.0), egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let t = ui.ctx().animate_bool(response.id, *value);
        let fill = if *value { theme::TEXT } else { theme::CARD };
        let stroke = if *value { theme::TEXT } else { theme::BORDER };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(10), fill);
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(10),
            egui::Stroke::new(1.0_f32, stroke),
            egui::StrokeKind::Inside,
        );

        let x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), t);
        let knob = if *value { theme::BG } else { theme::SECONDARY };
        ui.painter()
            .circle_filled(egui::pos2(x, rect.center().y), 7.0, knob);
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn error_banner(ui: &mut egui::Ui, message: &str) {
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 24,
            right: 24,
            top: 16,
            bottom: 0,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            banner(ui, message);
        });
}

/// Inline error callout without outer page margins.
pub fn banner(ui: &mut egui::Ui, message: &str) {
    egui::Frame::new()
        .fill(theme::CARD)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(message).size(13.0).color(theme::TEXT));
        });
}

/// Centered empty state with exactly one primary action. Returns true when the
/// action was clicked.
pub fn empty_state(ui: &mut egui::Ui, title: &str, hint: &str, action: &str) -> bool {
    let mut clicked = false;
    ui.add_space(96.0);
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(title).size(14.0).color(theme::MUTED));
        ui.add_space(2.0);
        ui.label(egui::RichText::new(hint).size(13.0).color(theme::MUTED));
        ui.add_space(10.0);
        if ui.add(primary_button(action)).clicked() {
            clicked = true;
        }
    });
    clicked
}
