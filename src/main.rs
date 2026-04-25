#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use rtasks::app::RTasksApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RTasks")
            .with_inner_size([760.0, 72.0])
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "RTasks",
        options,
        Box::new(|_creation_context| Box::new(RTasksApp::new())),
    )
}
