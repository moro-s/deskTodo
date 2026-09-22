use super::text::text_metrics;
use crate::ui::theme::{lighten, Theme};
use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, Sense, Vec2};
use eguidev::{WidgetMeta, WidgetRole, WidgetRoleMeta, WidgetValue};

/// 向 eguidev 注册一个按钮控件的自动化元数据。
pub(crate) fn track_button(id: &str, response: &egui::Response, label: &str) {
    eguidev::track_response(
        id,
        response,
        WidgetMeta {
            role: WidgetRoleMeta::Button { selected: None },
            label: Some(label.to_string()),
            visible: true,
            ..Default::default()
        },
    );
}

/// 向 eguidev 注册一个带选中态的分段按钮自动化元数据。
pub(crate) fn track_select_button(
    id: &str,
    response: &egui::Response,
    label: &str,
    selected: bool,
) {
    eguidev::track_response(
        id,
        response,
        WidgetMeta {
            role: WidgetRoleMeta::Button {
                selected: Some(selected),
            },
            label: Some(label.to_string()),
            visible: true,
            ..Default::default()
        },
    );
}

/// 向 eguidev 注册一个文本输入控件的自动化元数据。
pub(crate) fn track_text_edit(id: &str, response: &egui::Response, label: &str, value: &str) {
    eguidev::track_response(
        id,
        response,
        WidgetMeta {
            role: WidgetRoleMeta::Plain(WidgetRole::TextEdit),
            label: Some(label.to_string()),
            value: Some(WidgetValue::Text(value.to_string())),
            visible: true,
            ..Default::default()
        },
    );
}

/// 向 eguidev 注册一个复选框控件的自动化元数据。
pub(crate) fn track_checkbox(id: &str, response: &egui::Response, label: &str, checked: bool) {
    eguidev::track_response(
        id,
        response,
        WidgetMeta {
            role: WidgetRoleMeta::Plain(WidgetRole::Checkbox),
            label: Some(label.to_string()),
            value: Some(WidgetValue::Bool(checked)),
            visible: true,
            ..Default::default()
        },
    );
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ButtonKind {
    Primary,
    Default,
    Plain,
    Text,
    Danger,
    Icon,
}

pub(crate) fn el_button(
    ui: &mut egui::Ui,
    label: &str,
    kind: ButtonKind,
    theme: &Theme,
) -> egui::Response {
    el_button_ex(ui, label, kind, theme, 13.5, Vec2::new(14.0, 7.0))
}

pub(crate) fn el_button_ex(
    ui: &mut egui::Ui,
    label: &str,
    kind: ButtonKind,
    theme: &Theme,
    font_size: f32,
    padding: Vec2,
) -> egui::Response {
    let font_id = FontId::proportional(font_size);
    let metrics = text_metrics(ui, label, &font_id);
    let size = Vec2::new(
        metrics.width + padding.x * 2.0,
        font_size + padding.y * 2.0 + 2.0,
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();

    let (bg, fg, stroke) = match kind {
        ButtonKind::Primary => {
            let bg = if hovered {
                lighten(theme.accent)
            } else {
                theme.accent
            };
            (bg, Color32::WHITE, egui::Stroke::NONE)
        }
        ButtonKind::Default | ButtonKind::Plain => {
            if hovered {
                (
                    theme.hover_fill,
                    theme.accent,
                    egui::Stroke::new(1.0, theme.accent),
                )
            } else if matches!(kind, ButtonKind::Default) {
                (
                    theme.card,
                    theme.text_primary,
                    egui::Stroke::new(1.0, theme.border),
                )
            } else {
                (
                    theme.card,
                    theme.accent,
                    egui::Stroke::new(1.0, theme.accent),
                )
            }
        }
        ButtonKind::Text => {
            if hovered {
                (theme.hover_fill, theme.accent, egui::Stroke::NONE)
            } else {
                (Color32::TRANSPARENT, theme.text_muted, egui::Stroke::NONE)
            }
        }
        ButtonKind::Danger => {
            if hovered {
                (theme.danger_fill, theme.danger, egui::Stroke::NONE)
            } else {
                (Color32::TRANSPARENT, theme.danger, egui::Stroke::NONE)
            }
        }
        ButtonKind::Icon => {
            if hovered {
                (theme.hover_fill, theme.accent, egui::Stroke::NONE)
            } else {
                (Color32::TRANSPARENT, theme.text_muted, egui::Stroke::NONE)
            }
        }
    };

    if bg != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(4), bg);
    }
    if stroke.width > 0.0 {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            stroke,
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().text(
        rect.center() - metrics.visual_offset,
        Align2::CENTER_CENTER,
        label,
        font_id,
        fg,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub(crate) fn el_checkbox(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: &str,
    theme: &Theme,
) -> egui::Response {
    let box_size = 15.0;
    let font_id = FontId::proportional(13.0);
    let (text_width, label_offset_y) = if label.is_empty() {
        (0.0, 0.0)
    } else {
        let metrics = text_metrics(ui, label, &font_id);
        (metrics.width + 6.0, metrics.visual_offset.y)
    };
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(box_size + text_width, 20.0), Sense::click());
    let hovered = response.hovered();
    if response.clicked() {
        *checked = !*checked;
    }
    let box_min = Pos2::new(rect.left(), rect.center().y - box_size * 0.5);
    let box_rect = egui::Rect::from_min_size(box_min, Vec2::splat(box_size));
    if *checked {
        ui.painter()
            .rect_filled(box_rect, CornerRadius::same(3), theme.accent);
        let first = box_min + Vec2::new(3.5, 8.0);
        let corner = box_min + Vec2::new(6.5, 11.0);
        let last = box_min + Vec2::new(11.5, 4.5);
        ui.painter()
            .line_segment([first, corner], egui::Stroke::new(2.0, Color32::WHITE));
        ui.painter()
            .line_segment([corner, last], egui::Stroke::new(2.0, Color32::WHITE));
    } else {
        ui.painter()
            .rect_filled(box_rect, CornerRadius::same(3), theme.card);
        ui.painter().rect_stroke(
            box_rect,
            CornerRadius::same(3),
            egui::Stroke::new(1.2, if hovered { theme.accent } else { theme.border }),
            egui::StrokeKind::Inside,
        );
    }
    if !label.is_empty() {
        ui.painter().text(
            Pos2::new(box_rect.right() + 6.0, rect.center().y - label_offset_y),
            Align2::LEFT_CENTER,
            label,
            font_id,
            theme.text_primary,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// 在指定区域底部中央绘制悬浮按钮（绝对定位，不占布局空间）。
pub(crate) fn el_button_centered_floating(
    ui: &mut egui::Ui,
    label: &str,
    theme: &Theme,
    area: egui::Rect,
) -> egui::Response {
    let font_id = FontId::proportional(13.5);
    let metrics = text_metrics(ui, label, &font_id);
    let size = Vec2::new(metrics.width + 32.0, 32.0);
    let min_center_x = area.left() + size.x * 0.5;
    let max_center_x = area.right() - size.x * 0.5;
    let center_x = if min_center_x <= max_center_x {
        area.center().x.clamp(min_center_x, max_center_x)
    } else {
        area.center().x
    };
    let rect = egui::Rect::from_center_size(
        Pos2::new(center_x, area.bottom() - 6.0 - size.y * 0.5),
        size,
    );
    let response = ui.interact(rect, egui::Id::new(label), Sense::click());
    let hovered = response.hovered();
    let shadow = egui::Rect::from_min_size(rect.min + Vec2::new(0.0, 2.0), size);
    ui.painter().rect_filled(
        shadow,
        CornerRadius::same(5),
        Color32::from_rgba_unmultiplied(0, 0, 0, if theme.dark { 80 } else { 36 }),
    );
    let bg = if hovered {
        lighten(theme.card)
    } else {
        theme.card
    };
    ui.painter().rect_filled(rect, CornerRadius::same(5), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(5),
        egui::Stroke::new(
            1.0,
            if hovered {
                theme.danger
            } else {
                theme.border
            },
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center() - metrics.visual_offset,
        Align2::CENTER_CENTER,
        label,
        font_id,
        if hovered {
            theme.danger
        } else {
            theme.text_secondary
        },
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
