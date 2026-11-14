//! Frameless titlebar: drag region plus minimize / maximize / close.

use eframe::egui;

use crate::theme;

const HEIGHT: f32 = 40.0;
const BUTTON_W: f32 = 38.0;
const BUTTON_H: f32 = 30.0;
const PAD: f32 = 8.0;

#[derive(Clone, Copy)]
enum Glyph {
    Minimize,
    Maximize,
    Restore,
    Close,
}

pub fn show(ctx: &egui::Context) {
    egui::TopBottomPanel::top("titlebar")
        .resizable(false)
        .exact_height(HEIGHT)
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

            let buttons_w = BUTTON_W * 3.0 + PAD;
            let drag_rect =
                egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x - buttons_w, rect.max.y));
            let drag = ui.interact(
                drag_rect,
                ui.id().with("titlebar-drag"),
                egui::Sense::click_and_drag(),
            );
            if drag.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if drag.double_clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }

            ui.painter().text(
                egui::pos2(rect.left() + 16.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                "RexPaper",
                egui::FontId::proportional(14.0),
                theme::TEXT,
            );

            let mut x = rect.right() - PAD;
            x -= BUTTON_W;
            if caption_button(ui, rect, x, Glyph::Close, theme::DANGER).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            x -= BUTTON_W;
            let maximize_glyph = if maximized {
                Glyph::Restore
            } else {
                Glyph::Maximize
            };
            if caption_button(ui, rect, x, maximize_glyph, theme::CARD).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }

            x -= BUTTON_W;
            if caption_button(ui, rect, x, Glyph::Minimize, theme::CARD).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        });
}

fn caption_button(
    ui: &mut egui::Ui,
    bar: egui::Rect,
    left: f32,
    glyph: Glyph,
    hover_fill: egui::Color32,
) -> egui::Response {
    let rect = egui::Rect::from_center_size(
        egui::pos2(left + BUTTON_W / 2.0, bar.center().y),
        egui::vec2(BUTTON_W - 4.0, BUTTON_H),
    );
    let response = ui.interact(
        rect,
        ui.id().with(("caption", left as i32)),
        egui::Sense::click(),
    );

    if ui.is_rect_visible(rect) {
        if response.hovered() {
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::same(6), hover_fill);
        }
        let color = if response.hovered() {
            theme::TEXT
        } else {
            theme::SECONDARY
        };
        let stroke = egui::Stroke::new(1.0_f32, color);
        let center = rect.center();

        match glyph {
            Glyph::Minimize => {
                ui.painter().line_segment(
                    [
                        center + egui::vec2(-5.0, 0.0),
                        center + egui::vec2(5.0, 0.0),
                    ],
                    stroke,
                );
            }
            Glyph::Maximize => {
                let glyph_rect = egui::Rect::from_center_size(center, egui::vec2(9.0, 9.0));
                ui.painter().rect_stroke(
                    glyph_rect,
                    egui::CornerRadius::same(1),
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
            Glyph::Restore => {
                let back = egui::Rect::from_min_size(
                    center + egui::vec2(-2.0, -5.0),
                    egui::vec2(7.0, 7.0),
                );
                ui.painter().rect_stroke(
                    back,
                    egui::CornerRadius::same(1),
                    stroke,
                    egui::StrokeKind::Inside,
                );
                let front = egui::Rect::from_min_size(
                    center + egui::vec2(-5.0, -2.0),
                    egui::vec2(7.0, 7.0),
                );
                ui.painter()
                    .rect_filled(front, egui::CornerRadius::same(1), theme::PANEL);
                ui.painter().rect_stroke(
                    front,
                    egui::CornerRadius::same(1),
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
            Glyph::Close => {
                ui.painter().line_segment(
                    [
                        center + egui::vec2(-5.0, -5.0),
                        center + egui::vec2(5.0, 5.0),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        center + egui::vec2(5.0, -5.0),
                        center + egui::vec2(-5.0, 5.0),
                    ],
                    stroke,
                );
            }
        }
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}
