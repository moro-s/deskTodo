use super::widgets::{el_button, ButtonKind};
use crate::ui::theme::Theme;
use eframe::egui::{self, Align, Align2, CornerRadius, FontId, RichText, Sense, Vec2};

fn time_spinner_column(
    ui: &mut egui::Ui,
    salt: &str,
    value: &mut i32,
    max: i32,
    theme: &Theme,
    snap: bool,
) {
    let col_w = 46.0;
    let row_h = 30.0;
    let viewport_h = row_h * 5.0;
    let max_offset = (max as f32 * row_h - viewport_h).max(0.0);

    let base_id = egui::Id::new(salt);
    let offset_id = base_id.with("offset");
    let target_id = base_id.with("target");
    let rect_id = base_id.with("rect");
    let center_offset = |index: i32| (index as f32 + 0.5) * row_h - viewport_h * 0.5;

    let mut offset = ui
        .memory(|m| m.data.get_temp::<f32>(offset_id))
        .unwrap_or(0.0);
    let mut target = ui
        .memory(|m| m.data.get_temp::<f32>(target_id))
        .unwrap_or(0.0);
    let column_rect = ui.memory(|m| m.data.get_temp::<egui::Rect>(rect_id));

    if snap {
        offset = center_offset(*value).clamp(0.0, max_offset);
        target = offset;
    } else {
        let mut wheel_lines = 0.0;
        let mut wheel_pages = 0.0;
        let mut wheel_points = 0.0;
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { delta, unit, .. } = event {
                    match unit {
                        egui::MouseWheelUnit::Line => wheel_lines += delta.y,
                        egui::MouseWheelUnit::Page => wheel_pages += delta.y,
                        egui::MouseWheelUnit::Point => wheel_points += delta.y,
                    }
                }
            }
        });
        let over_column = column_rect.is_some_and(|rect| ui.rect_contains_pointer(rect));
        if over_column {
            target -= wheel_lines * row_h + wheel_pages * viewport_h + wheel_points;
            target = (target / row_h).round() * row_h;
            target = target.clamp(0.0, max_offset);
        }

        let dt = ui.input(|i| i.stable_dt).min(0.1);
        let progress = 1.0 - (-(16.0f32) * dt).exp();
        offset += (target - offset) * progress;
        if (target - offset).abs() > 0.5 {
            ui.ctx().request_repaint();
        } else {
            offset = target;
        }
    }

    ui.set_min_height(viewport_h);
    let scroll_output = egui::ScrollArea::vertical()
        .id_salt(salt)
        .max_width(col_w)
        .max_height(viewport_h)
        .scroll_source(egui::containers::scroll_area::ScrollSource::NONE)
        .scroll_bar_visibility(egui::containers::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .scroll_offset(Vec2::new(0.0, offset))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(col_w);
                for candidate in 0..max {
                    let selected = candidate == *value;
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(col_w, row_h), Sense::click());
                    let clicked = response.clicked();
                    let row_hovered = response.hovered();
                    if selected {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(6), theme.cell_today);
                    }
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        format!("{candidate:02}"),
                        FontId::proportional(14.0),
                        if selected {
                            theme.accent
                        } else if row_hovered {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        },
                    );
                    if clicked {
                        *value = candidate;
                        target = center_offset(candidate).clamp(0.0, max_offset);
                    }
                    let _ = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                }
            });
        })
        .inner_rect;

    ui.memory_mut(|m| {
        m.data.insert_temp(offset_id, offset);
        m.data.insert_temp(target_id, target);
        m.data.insert_temp(rect_id, scroll_output);
    });
}

pub(crate) fn draw_time_picker(
    ui: &mut egui::Ui,
    hour: &mut i32,
    minute: &mut i32,
    second: &mut i32,
    theme: &Theme,
) -> bool {
    let mut now_clicked = false;
    let trigger = ui.add_sized(
        [100.0, 32.0],
        egui::Button::new(RichText::new(format!(
            "⏰ {:02}:{:02}:{:02}",
            *hour, *minute, *second
        ))
        .size(13.5)),
    );
    let scroll_marker = egui::Id::new("time_picker_scroll");
    let mut snap = ui
        .memory(|m| m.data.get_temp::<bool>(scroll_marker))
        .unwrap_or(false);
    if trigger.clicked() {
        snap = true;
    }
    egui::Popup::from_toggle_button_response(&trigger)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let panel_w = 46.0 * 3.0 + ui.spacing().item_spacing.x * 2.0;
            ui.set_min_width(panel_w);
            ui.horizontal(|ui| {
                time_spinner_column(ui, "tp_hour", hour, 24, theme, snap);
                time_spinner_column(ui, "tp_minute", minute, 60, theme, snap);
                time_spinner_column(ui, "tp_second", second, 60, theme, snap);
            });
            ui.add_space(2.0);
            let (sep_rect, _) = ui.allocate_exact_size(
                Vec2::new(panel_w, 1.0),
                Sense::hover(),
            );
            ui.painter().hline(
                sep_rect.left()..=sep_rect.right(),
                sep_rect.center().y,
                egui::Stroke::new(1.0, theme.separator),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if el_button(ui, "此刻", ButtonKind::Text, theme).clicked() {
                    now_clicked = true;
                    ui.memory_mut(|m| {
                        m.data.insert_temp(scroll_marker, true)
                    });
                }
                ui.with_layout(
                    egui::Layout::right_to_left(Align::Center),
                    |ui| {
                        if el_button(ui, "确定", ButtonKind::Primary, theme).clicked() {
                            egui::Popup::close_all(ui.ctx());
                        }
                    },
                );
            });
        });
    if snap {
        ui.memory_mut(|m| m.data.insert_temp(scroll_marker, false));
    }
    now_clicked
}
