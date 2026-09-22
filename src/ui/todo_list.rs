use super::widgets::{
    el_button_centered_floating, el_button_ex, el_checkbox, track_button, track_checkbox,
    ButtonKind,
};
use crate::app::App;
use crate::core::models::{date_key, TodoItem};
use crate::core::storage::save_todos;
use eframe::egui::{self, Align, CornerRadius, RichText, Sense, Vec2};

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

impl App {
    pub(crate) fn draw_todo_list(&mut self, ui: &mut egui::Ui) {
        let key = date_key(self.selected);
        let has_items = self.todos.get(&key).is_some_and(|items| !items.is_empty());
        let theme = self.theme();
        let list_area = ui.max_rect();
        // 滚动视口预留悬浮按钮区域，保证最后一行永远不会与按钮重叠
        let viewport_height = (list_area.height() - 48.0).max(70.0);
        egui::ScrollArea::vertical()
            .max_height(viewport_height)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
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
                    let todos = self.todos.get_mut(&key).expect("存在待办");
                    let mut changed = false;
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
                            let checkbox_response = el_checkbox(ui, &mut done, "", theme);
                            track_checkbox(
                                &format!("todo.item.{index}.done"),
                                &checkbox_response,
                                "完成",
                                done,
                            );
                            if done != todos[index].done {
                                todos[index].done = done;
                                changed = true;
                            }
                            let mut text = RichText::new(todos[index].text.as_str())
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
                                    let delete_response = el_button_ex(
                                        ui,
                                        "×",
                                        ButtonKind::Danger,
                                        theme,
                                        15.0,
                                        Vec2::new(6.0, 4.0),
                                    )
                                    .on_hover_text("删除");
                                    track_button(
                                        &format!("todo.item.{index}.delete"),
                                        &delete_response,
                                        "删除",
                                    );
                                    if delete_response.clicked() {
                                        todos.remove(index);
                                        changed = true;
                                    }
                                    if let Some(remind_label) = &remind_label {
                                        ui.label(
                                            RichText::new(remind_label.as_str())
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
                    if changed {
                        save_todos(&self.todos);
                    }
                }
                // 滚动到底时给最后一行与悬浮按钮之间留出间距
                ui.add_space(12.0);
            });
        let clear_response =
            el_button_centered_floating(ui, "清除已完成", self.theme(), list_area);
        track_button("todo.clear_done", &clear_response, "清除已完成");
        if clear_response.clicked()
            && let Some(todos) = self.todos.get_mut(&key)
            && todos.iter().any(|item| item.done)
        {
            todos.retain(|item| !item.done);
            save_todos(&self.todos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Pos2;

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
