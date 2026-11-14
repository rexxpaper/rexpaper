//! Flat mono dark theme.

use std::sync::Arc;

use eframe::egui;

pub const BG: egui::Color32 = egui::Color32::from_rgb(0x0a, 0x0a, 0x0a);
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(0x11, 0x11, 0x11);
pub const CARD: egui::Color32 = egui::Color32::from_rgb(0x17, 0x17, 0x17);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(0x26, 0x26, 0x26);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(0xfa, 0xfa, 0xfa);
pub const PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xfa, 0xfa, 0xfa);
pub const SECONDARY: egui::Color32 = egui::Color32::from_rgb(0xa3, 0xa3, 0xa3);
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(0x73, 0x73, 0x73);
pub const DANGER: egui::Color32 = egui::Color32::from_rgb(0xe8, 0x11, 0x23);

pub fn apply(ctx: &egui::Context) {
    load_fonts(ctx);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = CARD;
    visuals.extreme_bg_color = PANEL;
    visuals.faint_bg_color = PANEL;
    visuals.override_text_color = Some(TEXT);

    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, SECONDARY);

    visuals.widgets.inactive.bg_fill = CARD;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, TEXT);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = PANEL;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, SECONDARY);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, TEXT);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

    visuals.widgets.active.bg_fill = CARD;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, TEXT);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, TEXT);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

    visuals.selection.bg_fill = egui::Color32::from_rgb(0x3a, 0x3a, 0x3a);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, TEXT);

    visuals.window_corner_radius = egui::CornerRadius::same(10);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.window_stroke = egui::Stroke::new(1.0_f32, BORDER);
    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = true;
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    ctx.set_style(style);
}

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Prefer the Windows UI font when available; egui's bundled font is the
    // fallback on systems without it.
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts.font_data.insert(
            "segoe-ui".to_owned(),
            Arc::new(egui::FontData::from_owned(bytes)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe-ui".to_owned());
    }

    ctx.set_fonts(fonts);
}
