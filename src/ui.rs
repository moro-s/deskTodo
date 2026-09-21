use crate::app::App;
use crate::models::{date_key, TodoItem};
use crate::storage::save_todos;
use crate::theme::lighten;
use chrono::{Datelike, Local, NaiveDate, Timelike};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Key, Pos2, RichText, Sense, Vec2,
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

fn insert_position(row_rects: &[egui::Rect], pointer_y: f32) -> usize {
    for (index, rect) in row_rects.iter().enumerate() {
        if pointer_y < rect.bottom() {
            return if pointer_y > rect.center().y {
                index + 1
            } else {
                index
            };
        }
    }
    row_rects.len()
}

fn move_todo(todos: &mut Vec<TodoItem>, from: usize, insert_at: usize) -> bool {
    let target = if insert_at > from {
        insert_at - 1
    } else {
        insert_at
    };
    if target == from {
        return false;
    }
    let item = todos.remove(from);
    todos.insert(target, item);
    true
}

fn anim_towards(ui: &egui::Ui, id: egui::Id, target: f32, speed: f32) -> f32 {
    let dt = ui.input(|i| i.unstable_dt).min(0.1);
    let current = ui.memory(|m| m.data.get_temp::<f32>(id).unwrap_or(0.0));
    let new = current + (target - current) * (1.0 - (1.0 - speed).powf(dt * 60.0));
    ui.memory_mut(|m| m.data.insert_temp(id, new));
    if (new - target).abs() > 0.01 {
        ui.ctx().request_repaint();
    }
    new
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

impl App {
    pub(crate) fn draw_titlebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Panel::top("titlebar")
            .frame(egui::Frame::NONE.fill(self.theme().titlebar))
            .show(ui, |ui| {
                ui.set_min_height(46.0);
                let titlebar_rect = ui.max_rect();
                let titlebar_fill = self.theme().titlebar;

                egui::Panel::right("titlebar_right")
                    .frame(egui::Frame::NONE.fill(titlebar_fill))
                    .resizable(false)
                    .exact_size(216.0)
                    .show_separator_line(false)
                    .show(ui, |ui| {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.add_space(10.0);
                                if ui
                                    .button(
                                        RichText::new("×").size(16.0).color(self.theme().danger),
                                    )
                                    .on_hover_text("关闭到托盘")
                                    .clicked()
                                {
                                    self.hide_to_tray(ctx);
                                }
                                let maximized =
                                    ctx.input(|i| i.viewport().maximized).unwrap_or(false);
                                let max_tip = if maximized {
                                    "还原窗口"
                                } else {
                                    "最大化"
                                };
                                if ui
                                    .button(RichText::new("□").size(14.0))
                                    .on_hover_text(max_tip)
                                    .clicked()
                                {
                                    ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                                }
                                let pin_label = if self.pinned { "置顶√" } else { "置顶" };
                                if ui
                                    .button(RichText::new(pin_label).size(14.5))
                                    .on_hover_text("快捷键 Ctrl+Alt+T")
                                    .clicked()
                                {
                                    self.set_pinned(ctx, !self.pinned);
                                }
                                let theme = self.theme();
                                if ui
                                    .button(RichText::new(theme.icon).size(16.0))
                                    .on_hover_text(format!(
                                        "主题：{}（点击切换）",
                                        theme.name
                                    ))
                                    .clicked()
                                {
                                    self.cycle_theme(ctx);
                                }
                            },
                        );
                    });

                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);
                    if ui
                        .button(RichText::new("‹").size(19.0))
                        .on_hover_text("上个月")
                        .clicked()
                    {
                        self.shift_month(-1);
                    }
                    if ui
                        .button(RichText::new("›").size(19.0))
                        .on_hover_text("下个月")
                        .clicked()
                    {
                        self.shift_month(1);
                    }
                    if ui
                        .button(RichText::new("今天").size(15.5))
                        .on_hover_text("回到今天")
                        .clicked()
                    {
                        let today = Local::now().date_naive();
                        self.view_year = today.year();
                        self.view_month = today.month();
                        self.selected = today;
                    }

                    let drag_size = ui.available_size();
                    let (rect, response) = ui.allocate_exact_size(drag_size, Sense::drag());
                    if response.dragged() {
                        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                    }
                    if response.double_clicked() {
                        let maximized =
                            ctx.input(|i| i.viewport().maximized).unwrap_or(false);
                        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                    }
                    let _ = rect;
                });

                let month_text = format!("{}年{}月", self.view_year, self.view_month);
                ui.painter().text(
                    titlebar_rect.center(),
                    Align2::CENTER_CENTER,
                    month_text,
                    FontId::proportional(17.0),
                    self.theme().text_title,
                );
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
        let default_spacing = ui.spacing().item_spacing.x;
        ui.spacing_mut().item_spacing.x = 5.0;
        let spacing = ui.spacing().item_spacing.x;
        let cell_w = (ui.available_width() - spacing * 6.0) / 7.0;
        let cell_h = ((ui.available_height() - 386.0) / 6.0).clamp(82.0, 150.0);
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
                    let hover_id = response.id.with("cell_hover");
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
                    let hover_t =
                        anim_towards(ui, hover_id, if is_hovered && !is_selected { 1.0 } else { 0.0 }, 0.18);
                    let bg = lerp_color(bg, lighten(bg), hover_t);
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
                        let pending: Vec<_> =
                            items.iter().filter(|item| !item.done).collect();
                        let mut y = rect.left_top().y + 26.0;
                        for item in pending.iter().take(3) {
                            if y + 14.0 > rect.bottom() - 3.0 {
                                break;
                            }
                            let text = format!("• {}", truncate_chars(&item.text, 10));
                            painter.text(
                                Pos2::new(rect.left() + 8.0, y),
                                Align2::LEFT_TOP,
                                text,
                                FontId::proportional(11.0),
                                theme.text_secondary,
                            );
                            y += 15.0;
                        }
                        if pending.len() > 3 {
                            let badge_text = format!("+{}", pending.len() - 3);
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
        ui.spacing_mut().item_spacing.x = default_spacing;
    }

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
            ui.label(RichText::new(title).size(16.0).color(self.theme().accent));
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if self.remind_enabled {
                        if ui
                            .button(RichText::new("现在").size(13.5))
                            .on_hover_text("设为当前时间")
                            .clicked()
                        {
                            let now = Local::now();
                            self.remind_hour = now.hour() as i32;
                            self.remind_minute = now.minute() as i32;
                        }
                        ui.add_sized(
                            [110.0, 20.0],
                            egui::Slider::new(&mut self.remind_minute, 0..=59).suffix("分"),
                        );
                        ui.add_sized(
                            [110.0, 20.0],
                            egui::Slider::new(&mut self.remind_hour, 0..=23).suffix("时"),
                        );
                    }
                    ui.checkbox(&mut self.remind_enabled, "⏰ 到点提醒");
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
                        if ui
                            .button(
                                RichText::new("添加").size(15.0).color(self.theme().accent),
                            )
                            .clicked()
                        {
                            submit = true;
                        }
                        if ui
                            .button(
                                RichText::new("收起")
                                    .size(14.0)
                                    .color(self.theme().text_muted),
                            )
                            .clicked()
                        {
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
                if ui
                    .button(RichText::new("添加").size(15.0).color(self.theme().accent))
                    .clicked()
                {
                    add_clicked = true;
                }
            });
            if add_clicked {
                self.add_todo();
            }
        }

        ui.add_space(8.0);
        let separator_color = self.theme().separator;
        let separator_width = ui.available_width();
        let (separator_rect, _) = ui.allocate_exact_size(Vec2::new(separator_width, 1.0), Sense::hover());
        ui.painter().hline(
            separator_rect.left()..=separator_rect.right(),
            separator_rect.center().y,
            egui::Stroke::new(1.0, separator_color),
        );
        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .max_height(if self.editor_expanded {
                140.0
            } else {
                200.0
            })
            .auto_shrink([false, true])
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
                    let mut row_rects: Vec<egui::Rect> = Vec::with_capacity(todos.len());
                    let mut dragging_from: Option<usize> = None;
                    let mut drop_requested = false;
                    for index in 0..todos.len() {
                        let remind_label = todos[index]
                            .remind_at
                            .as_ref()
                            .map(|remind_at| format!("⏰ {remind_at}"));
                        let dragging_row = self.drag_index == Some(index);
                        let row = ui.horizontal(|ui| {
                            let handle_color = if dragging_row {
                                theme.accent
                            } else {
                                theme.text_muted
                            };
                            let handle = ui
                                .add(
                                    egui::Label::new(
                                        RichText::new("≡").size(15.0).color(handle_color),
                                    )
                                    .sense(Sense::drag()),
                                )
                                .on_hover_text("拖动排序");
                            if handle.dragged_by(egui::PointerButton::Primary) {
                                dragging_from = Some(index);
                            }
                            if handle.drag_stopped() {
                                drop_requested = true;
                            }
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
                            let text_width = (ui.available_width() - 110.0).max(60.0);
                            ui.allocate_ui_with_layout(
                                Vec2::new(text_width, 20.0),
                                egui::Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.add(egui::Label::new(text).truncate());
                                },
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(RichText::new("×").size(15.0).color(theme.danger))
                                        .on_hover_text("删除")
                                        .clicked()
                                    {
                                        todos.remove(index);
                                        changed = true;
                                    }
                                    if let Some(remind_label) = &remind_label {
                                        ui.label(
                                            RichText::new(remind_label.clone())
                                                .small()
                                                .color(theme.accent),
                                        );
                                    }
                                },
                            );
                        });
                        row_rects.push(row.response.rect);
                        if dragging_row {
                            ui.painter().rect_stroke(
                                row.response.rect,
                                CornerRadius::same(6),
                                egui::Stroke::new(1.5, theme.accent),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if index + 1 < todos.len() {
                            ui.add_space(4.0);
                        }
                    }
                    if let Some(from) = dragging_from {
                        self.drag_index = Some(from);
                        if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) {
                            self.drop_target = Some(insert_position(&row_rects, pointer.y));
                        }
                    } else if !drop_requested {
                        self.drag_index = None;
                        self.drop_target = None;
                    }
                    if let (Some(from), Some(insert_at)) = (dragging_from, self.drop_target)
                        && insert_at != from
                        && insert_at != from + 1
                        && let (Some(first), Some(last)) = (row_rects.first(), row_rects.last())
                    {
                        let indicator_y = if insert_at < row_rects.len() {
                            row_rects[insert_at].top() - 3.0
                        } else {
                            last.bottom() + 3.0
                        };
                        ui.painter().hline(
                            first.left()..=last.right(),
                            indicator_y,
                            egui::Stroke::new(2.0, theme.accent),
                        );
                    }
                    if drop_requested
                        && let Some(from) = self.drag_index.take()
                    {
                        let insert_at = self.drop_target.take().unwrap_or(from);
                        changed |= move_todo(todos, from, insert_at);
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new("清除已完成")
                                    .size(13.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn row(y: f32) -> egui::Rect {
        egui::Rect::from_min_size(Pos2::new(20.0, y), Vec2::new(600.0, 32.0))
    }

    fn sample_rows() -> Vec<egui::Rect> {
        vec![row(100.0), row(140.0), row(180.0)]
    }

    #[test]
    fn insert_position_above_first_row_inserts_at_start() {
        assert_eq!(insert_position(&sample_rows(), 90.0), 0);
    }

    #[test]
    fn insert_position_upper_half_inserts_before_row() {
        assert_eq!(insert_position(&sample_rows(), 105.0), 0);
        assert_eq!(insert_position(&sample_rows(), 145.0), 1);
    }

    #[test]
    fn insert_position_lower_half_inserts_after_row() {
        assert_eq!(insert_position(&sample_rows(), 125.0), 1);
        assert_eq!(insert_position(&sample_rows(), 165.0), 2);
    }

    #[test]
    fn insert_position_below_all_rows_inserts_at_end() {
        assert_eq!(insert_position(&sample_rows(), 500.0), 3);
    }

    #[test]
    fn insert_position_empty_list_inserts_at_end() {
        assert_eq!(insert_position(&[], 500.0), 0);
    }

    fn todos(names: &[&str]) -> Vec<TodoItem> {
        names
            .iter()
            .map(|text| TodoItem {
                text: (*text).to_string(),
                done: false,
                remind_at: None,
            })
            .collect()
    }

    #[test]
    fn move_todo_forward_moves_item_down() {
        let mut items = todos(&["A", "B", "C"]);
        assert!(move_todo(&mut items, 0, 2));
        let texts: Vec<_> = items.iter().map(|item| item.text.as_str()).collect();
        assert_eq!(texts, ["B", "A", "C"]);
    }

    #[test]
    fn move_todo_to_end_appends_item() {
        let mut items = todos(&["A", "B", "C"]);
        assert!(move_todo(&mut items, 0, 3));
        let texts: Vec<_> = items.iter().map(|item| item.text.as_str()).collect();
        assert_eq!(texts, ["B", "C", "A"]);
    }

    #[test]
    fn move_todo_backward_moves_item_up() {
        let mut items = todos(&["A", "B", "C"]);
        assert!(move_todo(&mut items, 2, 0));
        let texts: Vec<_> = items.iter().map(|item| item.text.as_str()).collect();
        assert_eq!(texts, ["C", "A", "B"]);
    }

    #[test]
    fn move_todo_same_position_is_noop() {
        let mut items = todos(&["A", "B", "C"]);
        assert!(!move_todo(&mut items, 1, 1));
        assert!(!move_todo(&mut items, 1, 2));
        let texts: Vec<_> = items.iter().map(|item| item.text.as_str()).collect();
        assert_eq!(texts, ["A", "B", "C"]);
    }
}
