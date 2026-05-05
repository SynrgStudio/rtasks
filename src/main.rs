#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use eframe::egui;
use rtasks::{
    app::{AppMode, RTasksApp},
    ipc::{IpcCommand, send_command},
};

fn main() -> eframe::Result<()> {
    let initial_mode = match std::env::args().nth(1).as_deref() {
        Some("daemon") => AppMode::Hidden,
        Some("quick-add") => return send_or_start(IpcCommand::QuickAdd, AppMode::QuickAdd),
        Some("panel") => return send_or_start(IpcCommand::Panel, AppMode::Panel),
        Some("shutdown" | "stop" | "quit") => {
            let _ = send_command(IpcCommand::Shutdown, Duration::from_millis(1_000));
            return Ok(());
        }
        _ => AppMode::QuickAdd,
    };

    run_app(initial_mode)
}

fn send_or_start(command: IpcCommand, fallback_mode: AppMode) -> eframe::Result<()> {
    if send_command(command, Duration::from_millis(500)).is_ok() {
        return Ok(());
    }

    run_app(fallback_mode)
}

fn run_app(initial_mode: AppMode) -> eframe::Result<()> {
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
        Box::new(move |_creation_context| Box::new(RTasksApp::new_with_mode(initial_mode))),
    )
}
