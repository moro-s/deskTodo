use crate::app::App;
use crate::models::date_key;
use crate::storage::save_todos;
use crate::theme::lighten;
use chrono::{Datelike, Local, NaiveDate, Timelike};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, DragValue, FontId, Key, Pos2, RichText, Sense, Vec2,
    ViewportCommand,
};

const WEEKDAY_LABELS: [&str; 7] = ["一", "二", "三", "四", "五", "六", "日"];

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max_chars).collect();
        format!("{}…", cut)
    }
}

impl App {
    pub(crate) fn draw_titlebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Panel::top("titlebar")
            .frame(egui::Frame::NONE.fill(self.theme().titlebar))
            .show(ui, |ui| {
                ui.set_min_height(46.0);
                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);
                    if ui
                        .button(RichText::new("‹").size(17.0))
                        .on_hover_text("上个月")
                        .clicked()
                    {
                        self.shift_month(-1);
                    }
                    if ui
                        .button(RichText::new("›").size(17.0))
                        .on_hover_text("下个月")
                        .clicked()
                    {
                        self.shift_month(1);
                    }
                    if ui
                        .button(RichText::new("今天").size(14.0))
                        .on_hover_text("回到今天")
                        .clicked()
                    {
                        let today = Local::now().date_naive();
                        self.view_year = today.year();
                        self.view_month = today.month();
                        self.selected = today;
                    }

                    let month_text = format!("{}年{}月", self.view_year, self.view_month);
                    let title_rect = ui.available_rect_before_wrap();
                    let (rect, response) = ui.allocate_exact_size(title_rect.size(), Sense::drag());
                    if response.dragged() {
                        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                    }
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        month_text,
                        FontId::proportional(17.0),
                        self.theme().text_title,
                    );

                    let theme = self.theme();
                    if ui
                        .button(RichText::new(theme.icon).size(15.0))
                        .on_hover_text(format!("主题：{}（点击切换）", theme.name))
                        .clicked()
                    {
                        self.cycle_theme(ctx);
                    }
                    let pin_label = if self.pinned { "置顶✓" } else { "置顶" };
                    if ui
                        .button(RichText::new(pin_label).size(13.0))
                        .on_hover_text("快捷键 Ctrl+Alt+T")
                        .clicked()
                    {
                        self.set_pinned(ctx, !self.pinned);
                    }
                    if ui
                        .button(RichText::new("—").size(15.0))
                        .on_hover_text("最小化到托盘")
                        .clicked()
                    {
                        self.hide_to_tray(ctx);
                    }
                    if ui
                        .button(RichText::new("✕").size(15.0).color(self.theme().danger))
                        .on_hover_text("关闭到托盘")
                        .clicked()
                    {
                        self.hide_to_tray(ctx);
                    }
                    ui.add_space(10.0);
                });
            });
    }

    pub(crate) fn shift_month(&mut self, delta: i32) {
        let mut month = self.view_month as i32 + delta;
        let mut year = self.view_year;
        if month < 1 {
            month = 12;
            year -= 1;
        } else if month > 12 {
            month = 1;
            year += 1;
        }
        self.view_month = month as u32;
        self.view_year = year;
    }

    pub(crate) fn draw_calendar(&mut self, ui: &mut egui::Ui) {
        let spacing = ui.spacing().item_spacing.x;
        let cell_w = (ui.available_width() - spacing * 6.0) / 7.0;
        let cell_h = 82.0;
        let theme = self.theme();

        egui::Grid::new("weekday_header")
            .num_columns(7)
            .spacing([spacing, 4.0])
            .show(ui, |ui| {
                for (idx, label) in WEEKDAY_LABELS.iter().enumerate() {
                    let color = if idx >= 5 {
                        theme.weekend
                    } else {
                        theme.text_muted
                    };
                    ui.add_sized(
                        [cell_w, 18.0],
                        egui::Label::new(
                            RichText::new(*label)
                                .color(color)
                                .font(FontId::proportional(13.0)),
                        )
                        .selectable(false)
                        .halign(egui::Align::Center),
                    );
                }
            });

        ui.add_space(4.0);

        let first =
            NaiveDate::from_ymd_opt(self.view_year, self.view_month, 1).expect("非法日期");
        let lead = first.weekday().num_days_from_monday() as usize;
        let today = Local::now().date_naive();
        let mut clicked: Option<NaiveDate> = None;

        for row in 0..6 {
            ui.horizontal(|ui| {
                for col in 0..7 {
                    let date = first
                        - chrono::Duration::days(lead as i64 - (row as i64 * 7 + col as i64));
                    let in_month = date.month() == self.view_month;
                    let is_today = date == today;
                    let is_selected = date == self.selected;
                    let key = date_key(date);
                    let todos = self.todos.get(&key);

                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(cell_w, cell_h), Sense::click());
                    let was_clicked = response.clicked();
                    let is_hovered = response.hovered();
                    response.on_hover_cursor(egui::CursorIcon::PointingHand);
                    if was_clicked {
                        clicked = Some(date);
                    }

                    let bg = if is_selected {
                        theme.cell_selected
                    } else if is_today {
                        theme.cell_today
                    } else if in_month {
                        theme.cell_in_month
                    } else {
                        theme.cell_out_month
                    };
                    let bg = if is_hovered && !is_selected {
                        lighten(bg)
                    } else {
                        bg
                    };
                    let radius = CornerRadius::same(8);
                    ui.painter().rect_filled(rect, radius, bg);
                    if is_selected || is_today {
                        ui.painter().rect_stroke(
                            rect,
                            radius,
                            egui::Stroke::new(if is_selected { 1.8 } else { 1.0 }, theme.accent),
                            egui::StrokeKind::Inside,
                        );
                    }

                    let day_color = if is_today {
                        theme.accent
                    } else if in_month {
                        theme.text_primary
                    } else {
                        theme.text_dim
                    };
                    let painter = ui.painter().with_clip_rect(rect.shrink(1.0));
                    painter.text(
                        rect.left_top() + Vec2::new(8.0, 5.0),
                        Align2::LEFT_TOP,
                        date.day().to_string(),
                        FontId::proportional(13.5),
                        day_color,
                    );

                    if let Some(items) = todos {
                        let mut y = rect.left_top().y + 26.0;
                        for item in items.iter().take(3) {
                            if y + 14.0 > rect.bottom() - 3.0 {
                                break;
                            }
                            let prefix = if item.done {
                                "✓ "
                            } else if item.remind_at.is_some() {
                                "⏰ "
                            } else {
                                "• "
                            };
                            let text = format!("{}{}", prefix, truncate_chars(&item.text, 10));
                            let color = if item.done {
                                theme.text_done
                            } else {
                                theme.text_secondary
                            };
                            painter.text(
                                Pos2::new(rect.left() + 8.0, y),
                                Align2::LEFT_TOP,
                                text,
                                FontId::proportional(11.0),
                                color,
                            );
                            y += 15.0;
                        }
                        if items.len() > 3 {
                            let badge_text = format!("+{}", items.len() - 3);
                            let badge_pos = Pos2::new(rect.right() - 6.0, rect.bottom() - 5.0);
                            let badge_rect =
                                egui::Rect::from_center_size(badge_pos, Vec2::new(24.0, 14.0));
                            ui.painter().rect_filled(
                                badge_rect,
                                CornerRadius::same(6),
                                theme.accent,
                            );
                            painter.text(
                                badge_pos,
                                Align2::RIGHT_BOTTOM,
                                badge_text,
                                FontId::proportional(10.0),
                                Color32::BLACK,
                            );
                        }
                    }
                }
            });
        }

        if let Some(date) = clicked {
            if date.month() != self.view_month {
                self.view_year = date.year();
                self.view_month = date.month();
            }
            self.selected = date;
        }
    }

    pub(crate) fn draw_editor(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let weekday = WEEKDAY_LABELS[self.selected.weekday().num_days_from_monday() as usize];
        ui.label(
            RichText::new(format!(
                "{}月{}日 周{}",
                self.selected.month(),
                self.selected.day(),
                weekday
            ))
            .size(16.0)
            .color(self.theme().accent),
        );
        ui.add_space(8.0);

        let mut add_clicked = false;
        ui.horizontal(|ui| {
            let response = ui.add_sized(
                [ui.available_width() - 72.0, 32.0],
                egui::TextEdit::singleline(&mut self.input)
                    .hint_text("添加待办，回车确认…")
                    .desired_width(f32::INFINITY),
            );
            if ui
                .button(RichText::new("添加").size(14.0).color(self.theme().accent))
                .clicked()
            {
                add_clicked = true;
            }
            let enter = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if enter || add_clicked {
                self.add_todo();
                response.request_focus();
            }
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.remind_enabled, "⏰ 到点提醒");
            if self.remind_enabled {
                ui.add_space(6.0);
                ui.add(
                    DragValue::new(&mut self.remind_hour)
                        .range(0..=23)
                        .suffix("时"),
                );
                ui.label(":");
                ui.add(
                    DragValue::new(&mut self.remind_minute)
                        .range(0..=59)
                        .suffix("分"),
                );
                if ui
                    .button(RichText::new("现在").small())
                    .on_hover_text("设为当前时间")
                    .clicked()
                {
                    let now = Local::now();
                    self.remind_hour = now.hour() as i32;
                    self.remind_minute = now.minute() as i32;
                }
            }
        });

        ui.add_space(6.0);
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                let key = date_key(self.selected);
                let has_items = self.todos.get(&key).is_some_and(|items| !items.is_empty());
                if !has_items {
                    ui.set_min_height(40.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("这一天还没有待办")
                                .small()
                                .color(self.theme().text_muted),
                        );
                    });
                } else {
                    let theme = self.theme();
                    let todos = self.todos.get_mut(&key).expect("存在待办");
                    let mut changed = false;
                    let mut clear_done = false;
                    for index in 0..todos.len() {
                        ui.horizontal(|ui| {
                            let mut done = todos[index].done;
                            if ui.checkbox(&mut done, "").changed() {
                                todos[index].done = done;
                                changed = true;
                            }
                            let mut text = RichText::new(todos[index].text.clone())
                                .size(14.0)
                                .color(if todos[index].done {
                                    theme.text_done
                                } else {
                                    theme.text_secondary
                                });
                            if todos[index].done {
                                text = text.strikethrough();
                            }
                            ui.label(text);
                            if let Some(remind_at) = &todos[index].remind_at {
                                ui.label(
                                    RichText::new(format!("⏰ {remind_at}"))
                                        .small()
                                        .color(theme.accent),
                                );
                            }
                            if ui
                                .button(RichText::new("✕").size(13.0).color(theme.danger))
                                .on_hover_text("删除")
                                .clicked()
                            {
                                todos.remove(index);
                                changed = true;
                            }
                        });
                        if index + 1 < todos.len() {
                            ui.add_space(4.0);
                        }
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new("清除已完成")
                                    .small()
                                    .color(theme.text_muted),
                            )
                            .clicked()
                        {
                            clear_done = true;
                        }
                    });
                    if clear_done {
                        todos.retain(|item| !item.done);
                        changed = true;
                    }
                    if changed {
                        save_todos(&self.todos);
                    }
                }
            });
    }
}
