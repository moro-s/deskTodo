#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod font;
mod hotkey;
mod icon;
mod models;
mod reminder;
mod storage;
mod theme;
mod tray;
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
            .with_title("桌面日历待办")
            .with_icon(Arc::new(icon::calendar_icon_data())),
        ..Default::default()
    };
    eframe::run_native(
        "deskTodo",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
