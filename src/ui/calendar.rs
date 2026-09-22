use super::WEEKDAY_LABELS;
use super::text::{number, truncate_chars};
use crate::app::App;
use crate::core::models::date_key;
use crate::ui::theme::lighten;
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, RichText, Sense, Vec2};

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
                    let radius = CornerRadius::same(4);
                    ui.painter().rect_filled(rect, radius, bg);
                    if is_today && !is_selected {
                        ui.painter().rect_stroke(
                            rect,
                            radius,
                            egui::Stroke::new(1.0, theme.accent),
                            egui::StrokeKind::Inside,
                        );
                    }

                    let day_color = if is_selected {
                        Color32::WHITE
                    } else if is_today {
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
                        number(date.day()),
                        FontId::proportional(13.5),
                        day_color,
                    );

                    if let Some(items) = todos {
                        let pending: Vec<_> =
                            items.iter().filter(|item| !item.done).collect();
                        let mut y = rect.left_top().y + 26.0;
                        let item_color = if is_selected {
                            Color32::WHITE
                        } else {
                            theme.text_secondary
                        };
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
                                item_color,
                            );
                            y += 15.0;
                        }
                        if pending.len() > 3 {
                            let badge_text = format!("+{}", pending.len() - 3);
                            let badge_size = Vec2::new(24.0, 14.0);
                            let badge_rect = egui::Rect::from_min_size(
                                Pos2::new(
                                    rect.right() - 5.0 - badge_size.x,
                                    rect.top() + 4.0,
                                ),
                                badge_size,
                            );
                            painter.rect_filled(
                                badge_rect,
                                CornerRadius::same(7),
                                theme.danger,
                            );
                            painter.text(
                                badge_rect.center(),
                                Align2::CENTER_CENTER,
                                badge_text,
                                FontId::proportional(10.0),
                                Color32::WHITE,
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
}
