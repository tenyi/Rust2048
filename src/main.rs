mod game;
mod gui;
mod persistence;

use eframe::egui;

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Rust 2048",
        native_options,
        Box::new(|_cc| Ok(Box::new(gui::window::GameWindow::new(_cc)))),
    )
    .expect("Failed to start egui application");
}
