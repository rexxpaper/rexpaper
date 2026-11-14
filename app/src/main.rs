#![windows_subsystem = "windows"]

mod app;
mod state;
mod theme;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    let start_hidden = std::env::args()
        .skip(1)
        .any(|arg| arg == "--autostart" || arg == "--minimized");

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("RexPaper")
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([900.0, 640.0])
        .with_decorations(false)
        .with_resizable(true)
        .with_visible(!start_hidden);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "RexPaper",
        options,
        Box::new(move |cc| Ok(Box::new(app::RexApp::new(cc, start_hidden)))),
    )
}

fn load_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../../assets/icon.ico");
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Ico)
        .ok()?
        .to_rgba8();
    Some(egui::IconData {
        rgba: image.as_raw().clone(),
        width: image.width(),
        height: image.height(),
    })
}
