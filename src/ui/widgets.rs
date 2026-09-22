use super::text::measure_width;
use crate::ui::theme::{lighten, Theme};
use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, Sense, Vec2};

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
    let text_width = measure_width(ui, label, &font_id);
    let size = Vec2::new(
        text_width + padding.x * 2.0,
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
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, label, font_id, fg);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub(crate) fn el_checkbox(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: &str,
    theme: &Theme,
) {
    let box_size = 15.0;
    let font_id = FontId::proportional(13.0);
    let text_width = if label.is_empty() {
        0.0
    } else {
        measure_width(ui, label, &font_id) + 6.0
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
            Pos2::new(box_rect.right() + 6.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font_id,
            theme.text_primary,
        );
    }
    let _ = response.on_hover_cursor(egui::CursorIcon::PointingHand);
}

pub(crate) fn el_button_centered_floating(
    ui: &mut egui::Ui,
    label: &str,
    theme: &Theme,
) -> egui::Response {
    let font_id = FontId::proportional(13.5);
    let text_width = measure_width(ui, label, &font_id);
    let size = Vec2::new(text_width + 32.0, 32.0);
    let (line_rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), size.y + 4.0),
        Sense::hover(),
    );
    let left = ((line_rect.width() - size.x) / 2.0).max(0.0);
    let rect = egui::Rect::from_min_size(
        Pos2::new(line_rect.left() + left, line_rect.top() + 2.0),
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
        rect.center(),
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
