use crate::font;
use crate::hotkey;
use crate::models::{date_key, Config, HotkeySpec, TodoItem, TodoStore};
use crate::reminder::{due_reminders, next_delay};
use crate::storage::{
    current_data_dir, load_config, load_todos, save_config, save_todos,
};
use crate::theme::{apply_style, Theme, THEMES};
use crate::tray::{build_tray, take_tray_actions, TrayAction};
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui::{self, ViewportCommand, WindowLevel};
use global_hotkey::hotkey::HotKey;
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
    pub(crate) show_settings: bool,
    pub(crate) font_scale: f32,
    pub(crate) hotkey_spec: HotkeySpec,
    pub(crate) recording_hotkey: bool,
    pub(crate) hotkey_status: Option<String>,
    pub(crate) pending_data_dir: String,
    pub(crate) storage_status: Option<String>,
    data_dir_override: Option<String>,
    triggered_reminders: HashSet<String>,
    pub(crate) active_reminder: Option<String>,
    pub(crate) drag_index: Option<usize>,
    pub(crate) drop_target: Option<usize>,
    pub(crate) editor_expanded: bool,
    pub(crate) focus_expanded_input: bool,
    tray: Option<TrayIcon>,
    hotkey_manager: GlobalHotKeyManager,
    current_hotkey: HotKey,
}

impl App {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        font::install_cjk_font(&cc.egui_ctx);
        let today = Local::now().date_naive();

        let config = load_config();
        let theme_index = config.theme.min(THEMES.len() - 1);
        let visuals = if THEMES[theme_index].dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        cc.egui_ctx.set_visuals(visuals);
        apply_style(&cc.egui_ctx, &THEMES[theme_index]);
        cc.egui_ctx.set_zoom_factor(config.font_scale);
        let (hotkey_manager, current_hotkey) =
            hotkey::register_hotkey(&cc.egui_ctx, &config.hotkey);
        let pending_data_dir = current_data_dir()
            .map(|dir| dir.display().to_string())
            .unwrap_or_default();
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
            show_settings: false,
            font_scale: config.font_scale,
            hotkey_spec: config.hotkey.clone(),
            recording_hotkey: false,
            hotkey_status: None,
            pending_data_dir,
            storage_status: None,
            data_dir_override: config.data_dir.clone(),
            triggered_reminders: HashSet::new(),
            active_reminder: None,
            drag_index: None,
            drop_target: None,
            editor_expanded: false,
            focus_expanded_input: false,
            tray: None,
            hotkey_manager,
            current_hotkey,
        }
    }

    fn persist_config(&self) {
        save_config(&Config {
            theme: self.theme_index,
            font_scale: self.font_scale,
            hotkey: self.hotkey_spec.clone(),
            data_dir: self.data_dir_override.clone(),
        });
    }

    pub(crate) fn theme(&self) -> &'static Theme {
        &THEMES[self.theme_index]
    }

    pub(crate) fn set_theme(&mut self, ctx: &egui::Context, index: usize) {
        self.theme_index = index.min(THEMES.len() - 1);
        let visuals = if self.theme().dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_visuals(visuals);
        apply_style(ctx, self.theme());
        self.persist_config();
    }

    pub(crate) fn cycle_theme(&mut self, ctx: &egui::Context) {
        let next = (self.theme_index + 1) % THEMES.len();
        self.set_theme(ctx, next);
    }

    pub(crate) fn set_font_scale(&mut self, ctx: &egui::Context, scale: f32) {
        self.font_scale = scale;
        ctx.set_zoom_factor(scale);
        self.persist_config();
    }

    pub(crate) fn apply_hotkey(&mut self, spec: HotkeySpec) {
        let old = self.current_hotkey;
        match hotkey::reregister(&self.hotkey_manager, old, &spec) {
            Ok(new_hotkey) => {
                self.current_hotkey = new_hotkey;
                self.hotkey_spec = spec;
                self.hotkey_status =
                    Some(format!("快捷键已更新为 {}", self.hotkey_spec.display()));
                self.persist_config();
            }
            Err(err) => {
                let _ = self.hotkey_manager.register(old);
                self.hotkey_status = Some(err);
            }
        }
    }

    pub(crate) fn apply_data_dir(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            self.data_dir_override = None;
            self.persist_config();
            save_todos(&self.todos);
            self.storage_status = Some("已恢复默认存储位置，数据已写回默认目录".to_string());
        } else {
            let path = std::path::PathBuf::from(trimmed);
            if std::fs::create_dir_all(&path).is_err() {
                self.storage_status =
                    Some("无法创建该目录，请检查路径是否正确".to_string());
                return;
            }
            self.data_dir_override = Some(trimmed.to_string());
            self.persist_config();
            save_todos(&self.todos);
            self.storage_status = Some("存储位置已更新，待办数据已迁移".to_string());
        }
        self.pending_data_dir = current_data_dir()
            .map(|dir| dir.display().to_string())
            .unwrap_or_default();
    }

    pub(crate) fn open_data_folder(&self) {
        if let Some(dir) = current_data_dir() {
            let _ = std::process::Command::new("explorer").arg(dir).spawn();
        }
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
                TrayAction::OpenSettings => {
                    self.show_settings = true;
                    if self.hidden {
                        self.show_from_tray(ctx);
                    }
                }
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
        let text = self
            .input
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
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

    fn draw_reminder_banner(&mut self, ui: &mut egui::Ui) {
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
        self.draw_reminder_banner(ui);

        if self.show_settings {
            self.draw_settings(ui);
            return;
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
                        ui.set_min_height(ui.available_height());
                        self.draw_editor(ui);
                    });
            });
    }
}
