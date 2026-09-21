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
    pub(crate) reminder_banner: Color32,
    pub(crate) separator: Color32,
}

pub(crate) const THEMES: [Theme; 3] = [
    Theme {
        name: "深色",
        icon: "🌙",
        dark: true,
        bg: Color32::from_rgb(24, 26, 30),
        titlebar: Color32::from_rgb(30, 32, 38),
        card: Color32::from_rgb(44, 47, 54),
        cell_selected: Color32::from_rgb(64, 105, 190),
        cell_today: Color32::from_rgb(58, 64, 78),
        cell_in_month: Color32::from_rgb(44, 47, 54),
        cell_out_month: Color32::from_rgb(32, 34, 39),
        text_title: Color32::from_gray(230),
        text_primary: Color32::from_gray(210),
        text_secondary: Color32::from_gray(200),
        text_muted: Color32::from_gray(150),
        text_done: Color32::from_gray(110),
        text_dim: Color32::from_gray(95),
        accent: Color32::from_rgb(240, 200, 90),
        weekend: Color32::from_rgb(235, 130, 120),
        danger: Color32::from_rgb(230, 90, 90),
        reminder_banner: Color32::from_rgb(180, 60, 60),
        separator: Color32::from_rgb(62, 66, 74),
    },
    Theme {
        name: "浅色",
        icon: "☀",
        dark: false,
        bg: Color32::from_rgb(238, 240, 244),
        titlebar: Color32::from_rgb(226, 229, 235),
        card: Color32::from_rgb(255, 255, 255),
        cell_selected: Color32::from_rgb(64, 105, 190),
        cell_today: Color32::from_rgb(255, 232, 160),
        cell_in_month: Color32::from_rgb(255, 255, 255),
        cell_out_month: Color32::from_rgb(232, 234, 238),
        text_title: Color32::from_gray(45),
        text_primary: Color32::from_gray(55),
        text_secondary: Color32::from_gray(65),
        text_muted: Color32::from_gray(120),
        text_done: Color32::from_gray(150),
        text_dim: Color32::from_gray(170),
        accent: Color32::from_rgb(190, 140, 20),
        weekend: Color32::from_rgb(200, 85, 75),
        danger: Color32::from_rgb(200, 70, 70),
        reminder_banner: Color32::from_rgb(224, 82, 82),
        separator: Color32::from_rgb(224, 227, 233),
    },
    Theme {
        name: "墨绿",
        icon: "🍃",
        dark: true,
        bg: Color32::from_rgb(18, 28, 25),
        titlebar: Color32::from_rgb(24, 36, 32),
        card: Color32::from_rgb(34, 50, 44),
        cell_selected: Color32::from_rgb(42, 125, 100),
        cell_today: Color32::from_rgb(52, 74, 64),
        cell_in_month: Color32::from_rgb(34, 50, 44),
        cell_out_month: Color32::from_rgb(26, 38, 33),
        text_title: Color32::from_gray(225),
        text_primary: Color32::from_gray(205),
        text_secondary: Color32::from_gray(195),
        text_muted: Color32::from_gray(140),
        text_done: Color32::from_gray(105),
        text_dim: Color32::from_gray(90),
        accent: Color32::from_rgb(230, 190, 90),
        weekend: Color32::from_rgb(225, 130, 115),
        danger: Color32::from_rgb(225, 95, 95),
        reminder_banner: Color32::from_rgb(170, 70, 60),
        separator: Color32::from_rgb(50, 70, 62),
    },
];

pub(crate) fn apply_style(ctx: &egui::Context, theme: &Theme) {
    ctx.all_styles_mut(|style| {
        let radius = CornerRadius::same(6);
        style.visuals.widgets.inactive.corner_radius = radius;
        style.visuals.widgets.hovered.corner_radius = radius;
        style.visuals.widgets.active.corner_radius = radius;
        style.visuals.widgets.open.corner_radius = radius;
        style.visuals.window_corner_radius = CornerRadius::same(8);

        if theme.dark {
            style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(60, 60, 60);
            style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(76, 76, 76);
            style.visuals.widgets.active.bg_fill = Color32::from_rgb(80, 80, 80);
            style.visuals.widgets.inactive.fg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(224, 224, 224));
            style.visuals.widgets.hovered.fg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(230, 230, 230));
            style.visuals.widgets.active.fg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(235, 235, 235));
            style.visuals.widgets.noninteractive.bg_stroke =
                egui::Stroke::new(0.0, Color32::TRANSPARENT);
            style.visuals.widgets.inactive.bg_stroke =
                egui::Stroke::new(0.0, Color32::TRANSPARENT);
            style.visuals.selection.bg_fill =
                Color32::from_rgba_unmultiplied(128, 128, 128, 110);
            style.visuals.selection.stroke =
                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(128, 128, 128, 110));
        } else {
            style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(238, 238, 238);
            style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(225, 225, 225);
            style.visuals.widgets.active.bg_fill = Color32::from_rgb(215, 215, 215);
            style.visuals.widgets.inactive.fg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 60));
            style.visuals.widgets.hovered.fg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(45, 45, 45));
            style.visuals.widgets.active.fg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(35, 35, 35));
            style.visuals.widgets.inactive.bg_stroke =
                egui::Stroke::new(0.0, Color32::TRANSPARENT);
            style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 0, 0, 26);
            style.visuals.selection.stroke =
                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 40));
        }

        style.spacing.button_padding = Vec2::new(14.0, 9.0);
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
