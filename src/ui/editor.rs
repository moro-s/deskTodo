use super::time_picker::draw_time_picker;
use super::widgets::{el_button, el_checkbox, ButtonKind};
use super::WEEKDAY_LABELS;
use crate::app::App;
use chrono::{Datelike, Local, Timelike};
use eframe::egui::{self, Key, RichText};

impl App {
    pub(crate) fn draw_editor(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let weekday = WEEKDAY_LABELS[self.selected.weekday().num_days_from_monday() as usize];
        let title = format!(
            "{}月{}日 周{}",
            self.selected.month(),
            self.selected.day(),
            weekday
        );
        ui.horizontal(|ui| {
            let editor_theme = self.theme();
            ui.label(
                RichText::new(title)
                    .size(16.0)
                    .color(editor_theme.text_title)
                    .strong(),
            );
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if self.remind_enabled {
                        let theme = self.theme();
                        let mut hour = self.remind_hour;
                        let mut minute = self.remind_minute;
                        let mut second = self.remind_second;
                        let now_clicked =
                            draw_time_picker(ui, &mut hour, &mut minute, &mut second, theme);
                        self.remind_hour = hour;
                        self.remind_minute = minute;
                        self.remind_second = second;
                        if now_clicked {
                            let now = Local::now();
                            self.remind_hour = now.hour() as i32;
                            self.remind_minute = now.minute() as i32;
                            self.remind_second = now.second() as i32;
                        }
                    }
                    let checkbox_theme = self.theme();
                    el_checkbox(
                        ui,
                        &mut self.remind_enabled,
                        "⏰ 到点提醒",
                        checkbox_theme,
                    );
                },
            );
        });
        ui.add_space(8.0);

        if self.editor_expanded {
            let mut submit = false;
            let mut collapse = false;
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.input)
                    .id(egui::Id::new("todo_input"))
                    .hint_text("添加待办，Enter 提交，Shift+Enter 换行…")
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .return_key(egui::KeyboardShortcut::new(
                        egui::Modifiers::SHIFT,
                        Key::Enter,
                    )),
            );
            if self.focus_expanded_input {
                response.request_focus();
                self.focus_expanded_input = false;
            }
            if response.has_focus()
                && ui.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.shift)
            {
                submit = true;
            }
            if response.has_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
                collapse = true;
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        let theme = self.theme();
                        if el_button(ui, "添加", ButtonKind::Primary, theme).clicked() {
                            submit = true;
                        }
                        if el_button(ui, "收起", ButtonKind::Text, theme).clicked() {
                            collapse = true;
                        }
                        ui.label(
                            RichText::new("Enter 提交 · Shift+Enter 换行 · Esc 收起")
                                .small()
                                .color(self.theme().text_muted),
                        );
                    },
                );
            });
            if submit {
                self.add_todo();
                self.editor_expanded = false;
            } else if collapse {
                self.editor_expanded = false;
            }
        } else {
            let mut add_clicked = false;
            ui.horizontal(|ui| {
                let response = ui.add_sized(
                    [ui.available_width() - 78.0, 34.0],
                    egui::TextEdit::singleline(&mut self.input)
                        .id(egui::Id::new("todo_input"))
                        .hint_text("添加待办，点击展开编辑…"),
                );
                if response.gained_focus() {
                    self.editor_expanded = true;
                    self.focus_expanded_input = true;
                }
                if el_button(ui, "添加", ButtonKind::Primary, self.theme()).clicked() {
                    add_clicked = true;
                }
            });
            if add_clicked {
                self.add_todo();
            }
        }

    }
}
