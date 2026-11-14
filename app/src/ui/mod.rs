mod about;
mod gallery;
mod pages;
mod sidebar;
mod titlebar;
mod widgets;

use eframe::egui;

use crate::app::{RexApp, Route};
use crate::theme;

pub fn draw(app: &mut RexApp, ctx: &egui::Context) {
    titlebar::show(ctx);
    sidebar::show(app, ctx);

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::BG))
        .show(ctx, |ui| match app.route {
            Route::Static => pages::static_page(app, ui),
            Route::Live => pages::live_page(app, ui),
            Route::Settings => pages::settings_page(app, ui),
        });

    about::show(app, ctx);
}

/// Invisible resize handles for the frameless window.
pub fn resize_handles(ctx: &egui::Context) {
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    let Some(pointer) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };

    let rect = ctx.screen_rect();
    let edge = 6.0;
    let left = pointer.x <= rect.left() + edge;
    let right = pointer.x >= rect.right() - edge;
    let top = pointer.y <= rect.top() + edge;
    let bottom = pointer.y >= rect.bottom() - edge;

    let direction = match (left, right, top, bottom) {
        (true, _, true, _) => Some(egui::ResizeDirection::NorthWest),
        (_, true, true, _) => Some(egui::ResizeDirection::NorthEast),
        (true, _, _, true) => Some(egui::ResizeDirection::SouthWest),
        (_, true, _, true) => Some(egui::ResizeDirection::SouthEast),
        (true, _, _, _) => Some(egui::ResizeDirection::West),
        (_, true, _, _) => Some(egui::ResizeDirection::East),
        (_, _, true, _) => Some(egui::ResizeDirection::North),
        (_, _, _, true) => Some(egui::ResizeDirection::South),
        _ => None,
    };

    let Some(direction) = direction else {
        return;
    };

    ctx.set_cursor_icon(match direction {
        egui::ResizeDirection::NorthWest | egui::ResizeDirection::SouthEast => {
            egui::CursorIcon::ResizeNwSe
        }
        egui::ResizeDirection::NorthEast | egui::ResizeDirection::SouthWest => {
            egui::CursorIcon::ResizeNeSw
        }
        egui::ResizeDirection::West | egui::ResizeDirection::East => {
            egui::CursorIcon::ResizeHorizontal
        }
        egui::ResizeDirection::North | egui::ResizeDirection::South => {
            egui::CursorIcon::ResizeVertical
        }
    });

    if ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) {
        ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
    }
}
