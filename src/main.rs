#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod platform;
mod ui;

use eframe::egui;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(true)
            .with_inner_size([640.0, 952.0])
            .with_min_inner_size([560.0, 760.0])
            .with_title(platform::window::WINDOW_TITLE)
            .with_icon(Arc::new(platform::icon::calendar_icon_data())),
        ..Default::default()
    };
    eframe::run_native(
        "deskTodo",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
