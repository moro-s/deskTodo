use eframe::egui::{self, Color32, CornerRadius, Vec2};

pub(crate) struct Theme {
    pub(crate) name: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) dark: bool,
    pub(crate) bg: Color32,
    pub(crate) titlebar: Color32,
    pub(crate) card: Color32,
    pub(crate) cell_selected: Color32,
    pub(crate) cell_today: Color32,
    pub(crate) cell_in_month: Color32,
    pub(crate) cell_out_month: Color32,
    pub(crate) text_title: Color32,
    pub(crate) text_primary: Color32,
    pub(crate) text_secondary: Color32,
    pub(crate) text_muted: Color32,
    pub(crate) text_done: Color32,
    pub(crate) text_dim: Color32,
    pub(crate) accent: Color32,
    pub(crate) weekend: Color32,
    pub(crate) danger: Color32,
    pub(crate) separator: Color32,
    pub(crate) border: Color32,
    pub(crate) hover_fill: Color32,
    pub(crate) danger_fill: Color32,
}

pub(crate) const THEMES: [Theme; 3] = [
    Theme {
        name: "深色",
        icon: "🌙",
        dark: true,
        bg: Color32::from_rgb(20, 20, 20),
        titlebar: Color32::from_rgb(29, 29, 29),
        card: Color32::from_rgb(29, 29, 29),
        cell_selected: Color32::from_rgb(64, 158, 255),
        cell_today: Color32::from_rgb(29, 43, 58),
        cell_in_month: Color32::from_rgb(29, 29, 29),
        cell_out_month: Color32::from_rgb(20, 20, 20),
        text_title: Color32::from_rgb(229, 234, 243),
        text_primary: Color32::from_rgb(229, 234, 243),
        text_secondary: Color32::from_rgb(207, 211, 220),
        text_muted: Color32::from_rgb(163, 166, 173),
        text_done: Color32::from_rgb(108, 110, 114),
        text_dim: Color32::from_rgb(108, 110, 114),
        accent: Color32::from_rgb(64, 158, 255),
        weekend: Color32::from_rgb(248, 152, 152),
        danger: Color32::from_rgb(245, 108, 108),
        separator: Color32::from_rgb(65, 66, 67),
        border: Color32::from_rgb(76, 77, 79),
        hover_fill: Color32::from_rgb(38, 52, 71),
        danger_fill: Color32::from_rgb(58, 36, 38),
    },
    Theme {
        name: "浅色",
        icon: "☀",
        dark: false,
        bg: Color32::from_rgb(245, 247, 250),
        titlebar: Color32::from_rgb(255, 255, 255),
        card: Color32::from_rgb(255, 255, 255),
        cell_selected: Color32::from_rgb(64, 158, 255),
        cell_today: Color32::from_rgb(236, 245, 255),
        cell_in_month: Color32::from_rgb(255, 255, 255),
        cell_out_month: Color32::from_rgb(245, 247, 250),
        text_title: Color32::from_rgb(48, 49, 51),
        text_primary: Color32::from_rgb(48, 49, 51),
        text_secondary: Color32::from_rgb(96, 98, 102),
        text_muted: Color32::from_rgb(144, 147, 153),
        text_done: Color32::from_rgb(192, 196, 204),
        text_dim: Color32::from_rgb(192, 196, 204),
        accent: Color32::from_rgb(64, 158, 255),
        weekend: Color32::from_rgb(245, 108, 108),
        danger: Color32::from_rgb(245, 108, 108),
        separator: Color32::from_rgb(228, 231, 237),
        border: Color32::from_rgb(220, 223, 230),
        hover_fill: Color32::from_rgb(236, 245, 255),
        danger_fill: Color32::from_rgb(254, 240, 240),
    },
    Theme {
        name: "墨绿",
        icon: "🍃",
        dark: true,
        bg: Color32::from_rgb(16, 22, 20),
        titlebar: Color32::from_rgb(20, 27, 24),
        card: Color32::from_rgb(24, 36, 32),
        cell_selected: Color32::from_rgb(42, 174, 118),
        cell_today: Color32::from_rgb(30, 51, 41),
        cell_in_month: Color32::from_rgb(24, 36, 32),
        cell_out_month: Color32::from_rgb(18, 25, 22),
        text_title: Color32::from_rgb(224, 234, 228),
        text_primary: Color32::from_rgb(215, 227, 219),
        text_secondary: Color32::from_rgb(183, 200, 189),
        text_muted: Color32::from_rgb(126, 145, 134),
        text_done: Color32::from_rgb(90, 106, 96),
        text_dim: Color32::from_rgb(90, 106, 96),
        accent: Color32::from_rgb(47, 191, 143),
        weekend: Color32::from_rgb(241, 147, 140),
        danger: Color32::from_rgb(239, 108, 108),
        separator: Color32::from_rgb(42, 56, 48),
        border: Color32::from_rgb(58, 74, 65),
        hover_fill: Color32::from_rgb(30, 51, 41),
        danger_fill: Color32::from_rgb(51, 32, 29),
    },
];

pub(crate) fn apply_style(ctx: &egui::Context, theme: &Theme) {
    ctx.all_styles_mut(|style| {
        let radius = CornerRadius::same(4);
        style.visuals.widgets.inactive.corner_radius = radius;
        style.visuals.widgets.hovered.corner_radius = radius;
        style.visuals.widgets.active.corner_radius = radius;
        style.visuals.widgets.open.corner_radius = radius;
        style.visuals.window_corner_radius = CornerRadius::same(12);
        style.visuals.menu_corner_radius = CornerRadius::same(8);

        style.visuals.widgets.inactive.bg_fill = theme.card;
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, theme.border);
        style.visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0, theme.text_primary);
        style.visuals.widgets.hovered.bg_fill = theme.card;
        style.visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0, theme.text_muted);
        style.visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0, theme.text_primary);
        style.visuals.widgets.active.bg_fill = theme.card;
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, theme.accent);
        style.visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0, theme.text_primary);
        style.visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(0.0, Color32::TRANSPARENT);
        style.visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;

        style.visuals.selection.bg_fill =
            Color32::from_rgba_unmultiplied(64, 158, 255, 40);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, theme.accent);

        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        style.spacing.interact_size.y = 28.0;
    });
}

pub(crate) fn lighten(color: Color32) -> Color32 {
    let f = |channel: u8| channel as u32 + ((255 - channel as u32) * 12 / 100);
    Color32::from_rgb(
        f(color.r()) as u8,
        f(color.g()) as u8,
        f(color.b()) as u8,
    )
}
