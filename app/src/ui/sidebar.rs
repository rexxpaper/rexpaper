//! Left navigation sidebar.

use eframe::egui;

use crate::app::{RexApp, Route};
use crate::theme;

pub fn show(app: &mut RexApp, ctx: &egui::Context) {
    egui::SidePanel::left("sidebar")
        .resizable(false)
        .exact_width(192.0)
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL)
                .inner_margin(egui::Margin::same(12)),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;

            if nav_item(
                ui,
                "Static Wallpapers",
                Route::Static,
                app.route,
                icon_static,
            ) {
                app.route = Route::Static;
            }
            if nav_item(ui, "Live Wallpapers", Route::Live, app.route, icon_live) {
                app.route = Route::Live;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                if nav_item(ui, "Settings", Route::Settings, app.route, icon_settings) {
                    app.route = Route::Settings;
                }
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .size(11.0)
                        .color(theme::MUTED),
                );
            });
        });
}

fn nav_item(
    ui: &mut egui::Ui,
    label: &str,
    route: Route,
    current: Route,
    icon: fn(&egui::Painter, egui::Rect, egui::Color32),
) -> bool {
    let active = route == current;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 36.0), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let fill = if active {
            theme::CARD
        } else if response.hovered() {
            theme::CARD.gamma_multiply(0.55)
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(6), fill);

        let color = if active {
            theme::TEXT
        } else {
            theme::SECONDARY
        };
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            egui::vec2(16.0, 16.0),
        );
        icon(ui.painter(), icon_rect, color);

        ui.painter().text(
            egui::pos2(rect.left() + 36.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.0),
            color,
        );
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

fn icon_static(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.3_f32, color);
    painter.rect_stroke(
        rect.shrink(2.0),
        egui::CornerRadius::same(2),
        stroke,
        egui::StrokeKind::Inside,
    );
    painter.circle_filled(egui::pos2(rect.left() + 5.0, rect.top() + 5.5), 1.1, color);
    painter.line_segment(
        [
            egui::pos2(rect.left() + 2.6, rect.bottom() - 4.0),
            egui::pos2(rect.left() + 6.5, rect.bottom() - 8.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(rect.left() + 6.5, rect.bottom() - 8.0),
            egui::pos2(rect.left() + 10.0, rect.bottom() - 4.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(rect.left() + 10.0, rect.bottom() - 4.5),
            egui::pos2(rect.right() - 2.5, rect.bottom() - 8.5),
        ],
        stroke,
    );
}

fn icon_live(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let center = rect.center();
    painter.circle_stroke(center, 6.0, egui::Stroke::new(1.3_f32, color));
    painter.add(egui::Shape::convex_polygon(
        vec![
            center + egui::vec2(-1.6, -2.2),
            center + egui::vec2(2.6, 0.0),
            center + egui::vec2(-1.6, 2.2),
        ],
        color,
        egui::Stroke::NONE,
    ));
}

fn icon_settings(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.3_f32, color);
    let y1 = rect.top() + 5.5;
    let y2 = rect.top() + 10.5;
    painter.line_segment(
        [
            egui::pos2(rect.left() + 2.5, y1),
            egui::pos2(rect.right() - 2.5, y1),
        ],
        stroke,
    );
    painter.circle_filled(egui::pos2(rect.left() + 10.5, y1), 1.6, color);
    painter.line_segment(
        [
            egui::pos2(rect.left() + 2.5, y2),
            egui::pos2(rect.right() - 2.5, y2),
        ],
        stroke,
    );
    painter.circle_filled(egui::pos2(rect.left() + 5.5, y2), 1.6, color);
}
