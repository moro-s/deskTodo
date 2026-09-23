use super::text::text_metrics;
use super::widgets::{el_button_ex, track_button, ButtonKind};
use crate::app::App;
use chrono::{Datelike, Local};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Sense, Vec2, ViewportCommand,
};
use eguidev::{WidgetMeta, WidgetRoleMeta, WidgetValue};

fn paint_centered_glyph(
    ui: &mut egui::Ui,
    center: Pos2,
    glyph: &str,
    font_size: f32,
    color: Color32,
) {
    let font_id = FontId::proportional(font_size);
    let offset = text_metrics(ui, glyph, &font_id).visual_offset;
    ui.painter()
        .text(center - offset, Align2::CENTER_CENTER, glyph, font_id, color);
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
                    .exact_size(222.0)
                    .show_separator_line(false)
                    .show(ui, |ui| {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.add_space(10.0);
                                let close_theme = self.theme();
                                let (close_rect, close_response) =
                                    ui.allocate_exact_size(Vec2::new(30.0, 26.0), Sense::click());
                                if close_response.hovered() {
                                    ui.painter().rect_filled(
                                        close_rect,
                                        CornerRadius::same(6),
                                        close_theme.danger_fill,
                                    );
                                }
                                paint_centered_glyph(
                                    ui,
                                    close_rect.center(),
                                    "×",
                                    16.0,
                                    close_theme.danger,
                                );
                                if close_response.clicked() {
                                    self.hide_to_tray(ctx);
                                }
                                track_button("titlebar.close", &close_response, "关闭到托盘");
                                let _ = close_response
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text("关闭到托盘");
                                let maximized =
                                    ctx.input(|i| i.viewport().maximized).unwrap_or(false);
                                let max_tip = if maximized {
                                    "还原窗口"
                                } else {
                                    "最大化"
                                };
                                let (max_btn_rect, max_btn_response) =
                                    ui.allocate_exact_size(Vec2::new(30.0, 26.0), Sense::click());
                                let theme = self.theme();
                                if max_btn_response.hovered() {
                                    ui.painter().rect_filled(
                                        max_btn_rect,
                                        CornerRadius::same(6),
                                        theme.hover_fill,
                                    );
                                }
                                let max_icon_color = if max_btn_response.hovered() {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                };
                                ui.put(
                                    egui::Rect::from_center_size(
                                        max_btn_rect.center(),
                                        Vec2::splat(15.0),
                                    ),
                                    egui::Image::new(egui::include_image!("../../assets/zoom.svg"))
                                        .fit_to_exact_size(Vec2::splat(15.0))
                                        .tint(max_icon_color),
                                );
                                if max_btn_response.clicked() {
                                    ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                                }
                                track_button("titlebar.maximize", &max_btn_response, max_tip);
                                let _ = max_btn_response
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text(max_tip);
                                let (pin_rect, pin_response) =
                                    ui.allocate_exact_size(Vec2::new(30.0, 26.0), Sense::click());
                                let pin_theme = self.theme();
                                if pin_response.hovered() {
                                    ui.painter().rect_filled(
                                        pin_rect,
                                        CornerRadius::same(6),
                                        pin_theme.hover_fill,
                                    );
                                }
                                let pin_color = if self.pinned {
                                    pin_theme.accent
                                } else if pin_response.hovered() {
                                    pin_theme.text_primary
                                } else {
                                    pin_theme.text_secondary
                                };
                                ui.put(
                                    egui::Rect::from_center_size(
                                        pin_rect.center(),
                                        Vec2::splat(15.0),
                                    ),
                                    egui::Image::new(egui::include_image!("../../assets/pin.svg"))
                                        .fit_to_exact_size(Vec2::splat(15.0))
                                        .tint(pin_color),
                                );
                                if pin_response.clicked() {
                                    self.set_pinned(ctx, !self.pinned);
                                }
                                let pin_tip = if self.pinned {
                                    "取消置顶（快捷键 Ctrl+Alt+T）"
                                } else {
                                    "窗口置顶（快捷键 Ctrl+Alt+T）"
                                };
                                eguidev::track_response(
                                    "titlebar.pin",
                                    &pin_response,
                                    WidgetMeta {
                                        role: WidgetRoleMeta::Button {
                                            selected: Some(self.pinned),
                                        },
                                        label: Some("置顶".to_string()),
                                        value: Some(WidgetValue::Bool(self.pinned)),
                                        visible: true,
                                        ..Default::default()
                                    },
                                );
                                let _ = pin_response
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text(pin_tip);
                                let switch_theme = self.theme();
                                let (switch_rect, switch_response) =
                                    ui.allocate_exact_size(Vec2::new(30.0, 26.0), Sense::click());
                                if switch_response.hovered() {
                                    ui.painter().rect_filled(
                                        switch_rect,
                                        CornerRadius::same(6),
                                        switch_theme.hover_fill,
                                    );
                                }
                                let icon_color = if switch_response.hovered() {
                                    switch_theme.accent
                                } else {
                                    switch_theme.text_muted
                                };
                                paint_centered_glyph(
                                    ui,
                                    switch_rect.center(),
                                    switch_theme.icon,
                                    16.0,
                                    icon_color,
                                );
                                if switch_response.clicked() {
                                    self.cycle_theme(ctx);
                                }
                                eguidev::track_response(
                                    "titlebar.theme",
                                    &switch_response,
                                    WidgetMeta {
                                        role: WidgetRoleMeta::Button { selected: None },
                                        label: Some("切换主题".to_string()),
                                        value: Some(WidgetValue::Text(
                                            switch_theme.name.to_string(),
                                        )),
                                        visible: true,
                                        ..Default::default()
                                    },
                                );
                                let _ = switch_response
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text(format!(
                                        "主题：{}（点击切换）",
                                        switch_theme.name
                                    ));
                                let (settings_rect, settings_response) =
                                    ui.allocate_exact_size(Vec2::new(30.0, 26.0), Sense::click());
                                let settings_theme = self.theme();
                                if settings_response.hovered() {
                                    ui.painter().rect_filled(
                                        settings_rect,
                                        CornerRadius::same(6),
                                        settings_theme.hover_fill,
                                    );
                                }
                                let settings_stroke = egui::Stroke::new(
                                    1.4,
                                    if settings_response.hovered() {
                                        settings_theme.text_primary
                                    } else {
                                        settings_theme.text_secondary
                                    },
                                );
                                let gear_center = settings_rect.center();
                                ui.painter().circle_stroke(gear_center, 3.4, settings_stroke);
                                for tooth in 0..8 {
                                    let angle = tooth as f32 * std::f32::consts::TAU / 8.0;
                                    let direction = Vec2::new(angle.cos(), angle.sin());
                                    ui.painter().line_segment(
                                        [
                                            gear_center + direction * 4.4,
                                            gear_center + direction * 6.0,
                                        ],
                                        settings_stroke,
                                    );
                                }
                                if settings_response.clicked() {
                                    self.show_settings = true;
                                }
                                track_button("titlebar.settings", &settings_response, "打开设置");
                                let _ = settings_response
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text("打开设置");
                            },
                        );
                    });

                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);
                    let nav_theme = self.theme();
                    if self.show_settings {
                        let back_response = el_button_ex(
                            ui,
                            "‹ 返回",
                            ButtonKind::Default,
                            nav_theme,
                            15.5,
                            Vec2::new(10.0, 5.0),
                        )
                        .on_hover_text("返回主界面");
                        track_button("titlebar.back", &back_response, "返回主界面");
                        if back_response.clicked() {
                            self.show_settings = false;
                        }
                    } else {
                        let prev_response = el_button_ex(
                            ui,
                            "‹",
                            ButtonKind::Icon,
                            nav_theme,
                            19.0,
                            Vec2::new(6.0, 3.0),
                        )
                        .on_hover_text("上个月");
                        track_button("titlebar.prev_month", &prev_response, "上个月");
                        if prev_response.clicked() {
                            self.shift_month(-1);
                        }
                        let next_response = el_button_ex(
                            ui,
                            "›",
                            ButtonKind::Icon,
                            nav_theme,
                            19.0,
                            Vec2::new(6.0, 3.0),
                        )
                        .on_hover_text("下个月");
                        track_button("titlebar.next_month", &next_response, "下个月");
                        if next_response.clicked() {
                            self.shift_month(1);
                        }
                        let today_response = el_button_ex(
                            ui,
                            "今天",
                            ButtonKind::Plain,
                            nav_theme,
                            15.0,
                            Vec2::new(12.0, 5.0),
                        )
                        .on_hover_text("回到今天");
                        track_button("titlebar.today", &today_response, "回到今天");
                        if today_response.clicked() {
                            let today = Local::now().date_naive();
                            self.view_year = today.year();
                            self.view_month = today.month();
                            self.selected = today;
                        }
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

                let title_text = if self.show_settings {
                    "设置".to_string()
                } else {
                    format!("{}年{}月", self.view_year, self.view_month)
                };
                let title_font = FontId::proportional(17.0);
                let title_offset = text_metrics(ui, &title_text, &title_font).visual_offset;
                ui.painter().text(
                    titlebar_rect.center() - title_offset,
                    Align2::CENTER_CENTER,
                    title_text,
                    title_font,
                    self.theme().text_title,
                );
                ui.painter().hline(
                    titlebar_rect.left()..=titlebar_rect.right(),
                    titlebar_rect.bottom(),
                    egui::Stroke::new(1.0, self.theme().separator),
                );
            });
    }
}
