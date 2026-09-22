use crate::app::App;
use crate::hotkey;
use crate::models::HotkeySpec;
use crate::storage::current_data_dir;
use crate::theme::{Theme, THEMES};
use crate::ui::{el_button, el_button_ex, ButtonKind};
use eframe::egui::{self, CornerRadius, Key, RichText, Vec2};

const FONT_SCALES: [(&str, f32); 4] = [("小", 0.85), ("标准", 1.0), ("大", 1.15), ("特大", 1.3)];

const OPACITY_STEPS: [(&str, f32); 6] = [
    ("100%", 1.0),
    ("90%", 0.9),
    ("80%", 0.8),
    ("70%", 0.7),
    ("60%", 0.6),
    ("50%", 0.5),
];

fn card<R>(ui: &mut egui::Ui, title: &str, theme: &Theme, body: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::NONE
        .fill(theme.card)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(title)
                    .size(15.0)
                    .color(theme.text_title)
                    .strong(),
            );
            ui.add_space(6.0);
            body(ui)
        })
        .inner
}

fn section_button(
    ui: &mut egui::Ui,
    label: String,
    active: bool,
    theme: &Theme,
) -> bool {
    let kind = if active {
        ButtonKind::Primary
    } else {
        ButtonKind::Default
    };
    el_button_ex(ui, &label, kind, theme, 13.0, Vec2::new(14.0, 6.0)).clicked()
}

impl App {
    pub(crate) fn draw_settings(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let theme = self.theme();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme.bg).inner_margin(10.0))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    self.draw_hotkey_section(ui);
                    ui.add_space(10.0);
                    self.draw_theme_section(&ctx, ui);
                    ui.add_space(10.0);
                    self.draw_font_section(&ctx, ui);
                    ui.add_space(10.0);
                    self.draw_opacity_section(ui);
                    ui.add_space(10.0);
                    self.draw_storage_section(ui);
                    ui.add_space(6.0);
                });
            });
    }

    fn draw_hotkey_section(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme();
        card(ui, "快捷键配置", theme, |ui| {
            ui.label(
                RichText::new("全局热键：切换窗口置顶")
                    .size(13.5)
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("当前组合：{}", self.hotkey_spec.display()))
                        .size(15.0)
                        .color(theme.text_primary)
                        .strong(),
                );
                if !self.recording_hotkey
                    && el_button(ui, "修改快捷键", ButtonKind::Default, theme)
                        .clicked()
                {
                    self.recording_hotkey = true;
                    self.hotkey_status = None;
                }
            });
            if self.recording_hotkey {
                ui.add_space(6.0);
                egui::Frame::NONE
                    .fill(theme.cell_today)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(
                                "请按下新的快捷键组合（需含 Ctrl 或 Alt，Esc 取消）",
                            )
                            .size(13.0)
                            .color(theme.accent),
                        );
                    });
                let mut key_events: Vec<(Key, egui::Modifiers)> = Vec::new();
                ui.input(|i| {
                    for event in &i.events {
                        if let egui::Event::Key {
                            key, pressed: true, ..
                        } = event
                        {
                            key_events.push((*key, i.modifiers));
                        }
                    }
                });
                for (key, mods) in key_events {
                    if key == Key::Escape {
                        self.recording_hotkey = false;
                        self.hotkey_status = None;
                    } else if !(mods.ctrl || mods.alt) {
                        self.hotkey_status =
                            Some("快捷键需包含 Ctrl 或 Alt 修饰键".to_string());
                    } else if let Some(name) = hotkey::name_from_egui_key(key) {
                        let spec = HotkeySpec {
                            ctrl: mods.ctrl,
                            alt: mods.alt,
                            shift: mods.shift,
                            meta: false,
                            key: name.to_string(),
                        };
                        self.recording_hotkey = false;
                        self.apply_hotkey(spec);
                    } else {
                        self.hotkey_status =
                            Some("暂不支持该按键，请使用字母 / 数字 / F1-F12".to_string());
                    }
                }
            }
            if let Some(status) = &self.hotkey_status {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(status.clone())
                        .size(12.5)
                        .color(theme.text_muted),
                );
            }
        });
    }

    fn draw_theme_section(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let theme = self.theme();
        let mut picked: Option<usize> = None;
        card(ui, "主题设置", theme, |ui| {
            ui.label(
                RichText::new("选择界面配色")
                    .size(13.5)
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                for (index, item) in THEMES.iter().enumerate() {
                    let active = index == self.theme_index;
                    if section_button(ui, format!("{} {}", item.icon, item.name), active, theme) {
                        picked = Some(index);
                    }
                }
            });
        });
        if let Some(index) = picked {
            self.set_theme(ctx, index);
        }
    }

    fn draw_font_section(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let theme = self.theme();
        let mut picked: Option<f32> = None;
        card(ui, "字体大小设置", theme, |ui| {
            ui.label(
                RichText::new("整体界面缩放（含字体与控件）")
                    .size(13.5)
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                for (name, scale) in FONT_SCALES {
                    let active = (self.font_scale - scale).abs() < 0.01;
                    if section_button(ui, name.to_string(), active, theme) {
                        picked = Some(scale);
                    }
                }
            });
        });
        if let Some(scale) = picked {
            self.set_font_scale(ctx, scale);
        }
    }

    fn draw_opacity_section(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme();
        let mut picked: Option<f32> = None;
        card(ui, "透明度调节", theme, |ui| {
            ui.label(
                RichText::new("调整窗口整体透明度，实时生效")
                    .size(13.5)
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                for (name, value) in OPACITY_STEPS {
                    let active = (self.opacity - value).abs() < 0.005;
                    if section_button(ui, name.to_string(), active, theme) {
                        picked = Some(value);
                    }
                }
            });
        });
        if let Some(value) = picked {
            self.set_opacity(value);
        }
    }

    fn draw_storage_section(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme();
        let mut apply_clicked = false;
        let mut reset_clicked = false;
        let mut open_clicked = false;
        card(ui, "存储位置设置", theme, |ui| {
            ui.label(
                RichText::new("待办数据（todos.json）的保存目录")
                    .size(13.5)
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);
            let current = current_data_dir()
                .map(|dir| dir.display().to_string())
                .unwrap_or_else(|| "未设置".to_string());
            ui.label(
                RichText::new(format!("当前：{current}"))
                    .size(12.5)
                    .color(theme.text_muted),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let response = ui.add_sized(
                    [ui.available_width() - 90.0, 30.0],
                    egui::TextEdit::singleline(&mut self.pending_data_dir)
                        .hint_text("输入新的文件夹路径"),
                );
                if response.lost_focus()
                    && ui.input(|i| i.key_pressed(Key::Enter))
                {
                    apply_clicked = true;
                }
                if el_button(ui, "应用", ButtonKind::Primary, theme).clicked() {
                    apply_clicked = true;
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if el_button(ui, "恢复默认", ButtonKind::Default, theme).clicked() {
                    reset_clicked = true;
                }
                if el_button(ui, "打开文件夹", ButtonKind::Default, theme).clicked() {
                    open_clicked = true;
                }
            });
            if let Some(status) = &self.storage_status {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(status.clone())
                        .size(12.5)
                        .color(theme.text_muted),
                );
            }
        });
        if apply_clicked {
            let input = self.pending_data_dir.clone();
            self.apply_data_dir(&input);
        }
        if reset_clicked {
            self.pending_data_dir.clear();
            self.apply_data_dir("");
        }
        if open_clicked {
            self.open_data_folder();
        }
    }
}
