use crate::font;
use crate::hotkey;
use crate::models::{date_key, Config, TodoItem, TodoStore};
use crate::reminder::{due_reminders, next_delay};
use crate::storage::{load_config, load_todos, save_config, save_todos};
use crate::theme::{apply_style, Theme, THEMES};
use crate::tray::{build_tray, take_tray_actions, TrayAction};
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui::{self, ViewportCommand, WindowLevel};
use global_hotkey::GlobalHotKeyManager;
use std::collections::HashSet;
use std::time::Duration;
use tray_icon::TrayIcon;

pub(crate) struct App {
    pub(crate) todos: TodoStore,
    pub(crate) input: String,
    pub(crate) view_year: i32,
    pub(crate) view_month: u32,
    pub(crate) selected: NaiveDate,
    pub(crate) pinned: bool,
    pub(crate) hidden: bool,
    pub(crate) theme_index: usize,
    pub(crate) remind_enabled: bool,
    pub(crate) remind_hour: i32,
    pub(crate) remind_minute: i32,
    triggered_reminders: HashSet<String>,
    pub(crate) active_reminder: Option<String>,
    pub(crate) drag_index: Option<usize>,
    pub(crate) drop_target: Option<usize>,
    tray: Option<TrayIcon>,
    #[allow(dead_code)]
    hotkey_manager: GlobalHotKeyManager,
}

impl App {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        font::install_cjk_font(&cc.egui_ctx);
        let today = Local::now().date_naive();
        let hotkey_manager = hotkey::register_pin_toggle(&cc.egui_ctx);

        let config = load_config();
        let theme_index = config.theme.min(THEMES.len() - 1);
        let visuals = if THEMES[theme_index].dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        cc.egui_ctx.set_visuals(visuals);
        apply_style(&cc.egui_ctx);
        Self {
            todos: load_todos(),
            input: String::new(),
            view_year: today.year(),
            view_month: today.month(),
            selected: today,
            pinned: false,
            hidden: false,
            theme_index,
            remind_enabled: false,
            remind_hour: 9,
            remind_minute: 0,
            triggered_reminders: HashSet::new(),
            active_reminder: None,
            drag_index: None,
            drop_target: None,
            tray: None,
            hotkey_manager,
        }
    }

    pub(crate) fn theme(&self) -> &'static Theme {
        &THEMES[self.theme_index]
    }

    pub(crate) fn cycle_theme(&mut self, ctx: &egui::Context) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
        let visuals = if self.theme().dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_visuals(visuals);
        apply_style(ctx);
        save_config(&Config {
            theme: self.theme_index,
        });
    }

    pub(crate) fn set_pinned(&mut self, ctx: &egui::Context, pinned: bool) {
        self.pinned = pinned;
        let level = if pinned {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        };
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
    }

    pub(crate) fn hide_to_tray(&mut self, ctx: &egui::Context) {
        self.hidden = true;
        ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
    }

    fn show_from_tray(&mut self, ctx: &egui::Context) {
        self.hidden = false;
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }

    fn toggle_visibility(&mut self, ctx: &egui::Context) {
        if self.hidden {
            self.show_from_tray(ctx);
        } else {
            self.hide_to_tray(ctx);
        }
    }

    fn handle_tray_actions(&mut self, ctx: &egui::Context) {
        for action in take_tray_actions() {
            match action {
                TrayAction::ToggleVisibility => self.toggle_visibility(ctx),
                TrayAction::TogglePin => self.set_pinned(ctx, !self.pinned),
                TrayAction::Exit => {
                    save_todos(&self.todos);
                    std::process::exit(0);
                }
            }
        }
    }

    fn check_reminders(&mut self, ctx: &egui::Context) {
        for due in due_reminders(&self.todos, &self.triggered_reminders, Local::now()) {
            self.triggered_reminders.insert(due.trigger_id);
            self.active_reminder = Some(due.message);
            self.show_from_tray(ctx);
            ctx.send_viewport_cmd(ViewportCommand::RequestUserAttention(
                egui::UserAttentionType::Critical,
            ));
        }
    }

    fn next_repaint_delay(&self) -> Duration {
        next_delay(&self.todos, &self.triggered_reminders, Local::now())
    }

    pub(crate) fn add_todo(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        let remind_at = if self.remind_enabled {
            Some(format!(
                "{:02}:{:02}",
                self.remind_hour, self.remind_minute
            ))
        } else {
            None
        };
        let key = date_key(self.selected);
        self.todos.entry(key).or_default().push(TodoItem {
            text,
            done: false,
            remind_at,
        });
        self.input.clear();
        save_todos(&self.todos);
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.tray.is_none() {
            self.tray = build_tray(ctx);
        }

        self.handle_tray_actions(ctx);
        self.check_reminders(ctx);

        if ctx.input(|i| i.viewport().close_requested()) {
            self.hide_to_tray(ctx);
        }

        let delay = self.next_repaint_delay();
        ctx.request_repaint_after(delay);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.draw_titlebar(&ctx, ui);

        if let Some(reminder) = self.active_reminder.clone() {
            egui::Frame::NONE
                .fill(self.theme().reminder_banner)
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("⏰ 待办提醒：{reminder}"))
                                .size(14.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
                        if ui
                            .button(
                                egui::RichText::new("知道了")
                                    .size(13.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .clicked()
                        {
                            self.active_reminder = None;
                        }
                    });
                });
            ui.add_space(4.0);
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.theme().bg)
                    .inner_margin(10.0),
            )
            .show(ui, |ui| {
                self.draw_calendar(ui);
                ui.add_space(4.0);
                egui::Frame::NONE
                    .fill(self.theme().card)
                    .corner_radius(egui::CornerRadius::same(12))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.draw_editor(ui);
                    });
            });
    }
}
