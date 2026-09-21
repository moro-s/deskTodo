#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{Datelike, Local, NaiveDate, Timelike};
use eframe::egui;
use eframe::egui::{
    Align2, Color32, CornerRadius, DragValue, FontId, Key, Pos2, RichText, Sense, Vec2,
    ViewportCommand, WindowLevel,
};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

const WEEKDAY_LABELS: [&str; 7] = ["一", "二", "三", "四", "五", "六", "日"];

struct Theme {
    name: &'static str,
    icon: &'static str,
    dark: bool,
    bg: Color32,
    titlebar: Color32,
    card: Color32,
    cell_selected: Color32,
    cell_today: Color32,
    cell_in_month: Color32,
    cell_out_month: Color32,
    text_title: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    text_muted: Color32,
    text_done: Color32,
    text_dim: Color32,
    accent: Color32,
    weekend: Color32,
    danger: Color32,
    reminder_banner: Color32,
}

const THEMES: [Theme; 3] = [
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
    },
    Theme {
        name: "浅色",
        icon: "☀️",
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
    },
];

#[derive(Serialize, Deserialize, Clone)]
struct TodoItem {
    text: String,
    done: bool,
    #[serde(default)]
    remind_at: Option<String>,
}

type TodoStore = BTreeMap<String, Vec<TodoItem>>;

#[derive(Clone, Copy)]
enum TrayAction {
    ToggleVisibility,
    TogglePin,
    Exit,
}

static TRAY_ACTIONS: OnceLock<Mutex<Vec<TrayAction>>> = OnceLock::new();

fn push_tray_action(action: TrayAction) {
    let queue = TRAY_ACTIONS.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut queue) = queue.lock() {
        queue.push(action);
    }
}

fn take_tray_actions() -> Vec<TrayAction> {
    TRAY_ACTIONS
        .get()
        .and_then(|queue| queue.lock().ok())
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default()
}

#[derive(Serialize, Deserialize)]
struct Config {
    theme: usize,
}

struct App {
    todos: TodoStore,
    input: String,
    view_year: i32,
    view_month: u32,
    selected: NaiveDate,
    pinned: bool,
    hidden: bool,
    theme_index: usize,
    remind_enabled: bool,
    remind_hour: i32,
    remind_minute: i32,
    triggered_reminders: HashSet<String>,
    active_reminder: Option<String>,
    tray: Option<TrayIcon>,
    #[allow(dead_code)]
    hotkey_manager: GlobalHotKeyManager,
}

fn date_key(date: NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

fn data_file() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("deskTodo").join("todos.json"))
}

fn config_file() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("deskTodo").join("config.json"))
}

fn load_config() -> Config {
    if let Some(path) = config_file() {
        if let Ok(raw) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&raw) {
                return config;
            }
        }
    }
    Config { theme: 0 }
}

fn save_config(config: &Config) {
    if let Some(path) = config_file() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(config) {
            let _ = fs::write(path, json);
        }
    }
}

fn load_todos() -> TodoStore {
    if let Some(path) = data_file() {
        if let Ok(raw) = fs::read_to_string(path) {
            if let Ok(store) = serde_json::from_str(&raw) {
                return store;
            }
        }
    }
    TodoStore::new()
}

fn save_todos(todos: &TodoStore) {
    if let Some(path) = data_file() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(todos) {
            let _ = fs::write(path, json);
        }
    }
}

fn apply_style(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        let radius = CornerRadius::same(7);
        style.visuals.widgets.inactive.corner_radius = radius;
        style.visuals.widgets.hovered.corner_radius = radius;
        style.visuals.widgets.active.corner_radius = radius;
        style.visuals.widgets.open.corner_radius = radius;
        style.spacing.button_padding = Vec2::new(10.0, 6.0);
    });
}

fn install_cjk_font(ctx: &egui::Context) {
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            r"C:\Windows\Fonts\msyh.ttc",
            r"C:\Windows\Fonts\msyh.ttf",
            r"C:\Windows\Fonts\simhei.ttf",
            r"C:\Windows\Fonts\simsun.ttc",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
        ]
    } else {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        ]
    };

    for path in candidates {
        if let Ok(bytes) = fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("cjk".into(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "cjk".into());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("cjk".into());
            ctx.set_fonts(fonts);
            return;
        }
    }
}

fn calendar_icon_data() -> egui::IconData {
    let size = 64u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let ix = x as i32;
            let iy = y as i32;
            let mut color = [45, 92, 180, 255];
            if (8..14).contains(&iy) && (12..52).contains(&ix) {
                color = [235, 105, 95, 255];
            }
            if (3..9).contains(&iy) && ((18..24).contains(&ix) || (40..46).contains(&ix)) {
                color = [235, 105, 95, 255];
            }
            for grid_y in 0..2 {
                for grid_x in 0..3 {
                    let cell_x = 12 + grid_x * 14;
                    let cell_y = 22 + grid_y * 14;
                    if (cell_x..cell_x + 8).contains(&ix) && (cell_y..cell_y + 8).contains(&iy) {
                        color = [255, 255, 255, 255];
                    }
                }
            }
            rgba.extend_from_slice(&color);
        }
    }
    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

fn tray_icon_from(icon_data: &egui::IconData) -> Option<tray_icon::Icon> {
    tray_icon::Icon::from_rgba(
        icon_data.rgba.clone(),
        icon_data.width,
        icon_data.height,
    )
    .ok()
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_cjk_font(&cc.egui_ctx);
        let today = Local::now().date_naive();
        let hotkey_manager = GlobalHotKeyManager::new().expect("无法创建全局热键管理器");
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT);
        if let Err(err) = hotkey_manager.register(hotkey) {
            eprintln!("注册热键失败: {err}");
        }

        let ctx_hotkey = cc.egui_ctx.clone();
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.state() == HotKeyState::Pressed {
                push_tray_action(TrayAction::TogglePin);
            }
            ctx_hotkey.request_repaint();
        }));
        let config = load_config();
        let theme_index = config.theme.min(THEMES.len() - 1);
        let visuals = if THEMES[theme_index].dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        cc.egui_ctx.set_visuals(visuals);
        apply_style(&cc.egui_ctx);
        Self {
            todos: load_todos(),
            input: String::new(),
            view_year: today.year(),
            view_month: today.month(),
            selected: today,
            pinned: false,
            hidden: false,
            theme_index,
            remind_enabled: false,
            remind_hour: 9,
            remind_minute: 0,
            triggered_reminders: HashSet::new(),
            active_reminder: None,
            tray: None,
            hotkey_manager,
        }
    }

    fn theme(&self) -> &'static Theme {
        &THEMES[self.theme_index]
    }

    fn cycle_theme(&mut self, ctx: &egui::Context) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
        let visuals = if self.theme().dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_visuals(visuals);
        apply_style(ctx);
        save_config(&Config {
            theme: self.theme_index,
        });
    }

    fn init_tray(&mut self, ctx: &egui::Context) {
        let menu = Menu::new();
        let show_item = MenuItem::with_id("toggle", "显示 / 隐藏", true, None);
        let pin_item = MenuItem::with_id("pin", "切换置顶（Ctrl+Alt+T）", true, None);
        let quit_item = MenuItem::with_id("quit", "退出", true, None);
        if let Err(err) = menu.append_items(&[&show_item, &pin_item, &quit_item]) {
            eprintln!("构建托盘菜单失败: {err}");
            return;
        }

        let icon_data = calendar_icon_data();
        let Some(icon) = tray_icon_from(&icon_data) else {
            eprintln!("创建托盘图标失败");
            return;
        };

        let tray = match TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("桌面日历待办")
            .with_icon(icon)
            .build()
        {
            Ok(tray) => tray,
            Err(err) => {
                eprintln!("创建托盘失败: {err}");
                return;
            }
        };

        let ctx_menu = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            match event.id().as_ref() {
                "toggle" => push_tray_action(TrayAction::ToggleVisibility),
                "pin" => push_tray_action(TrayAction::TogglePin),
                "quit" => push_tray_action(TrayAction::Exit),
                _ => {}
            }
            ctx_menu.request_repaint();
        }));

        let ctx_tray = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                push_tray_action(TrayAction::ToggleVisibility);
                ctx_tray.request_repaint();
            }
        }));

        self.tray = Some(tray);
    }

    fn set_pinned(&mut self, ctx: &egui::Context, pinned: bool) {
        self.pinned = pinned;
        let level = if pinned {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        };
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
    }

    fn hide_to_tray(&mut self, ctx: &egui::Context) {
        self.hidden = true;
        ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
    }

    fn show_from_tray(&mut self, ctx: &egui::Context) {
        self.hidden = false;
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }

    fn toggle_visibility(&mut self, ctx: &egui::Context) {
        if self.hidden {
            self.show_from_tray(ctx);
        } else {
            self.hide_to_tray(ctx);
        }
    }

    fn handle_tray_actions(&mut self, ctx: &egui::Context) {
        for action in take_tray_actions() {
            match action {
                TrayAction::ToggleVisibility => self.toggle_visibility(ctx),
                TrayAction::TogglePin => self.set_pinned(ctx, !self.pinned),
                TrayAction::Exit => {
                    save_todos(&self.todos);
                    std::process::exit(0);
                }
            }
        }
    }

    fn check_reminders(&mut self, ctx: &egui::Context) {
        let now = Local::now();
        let today = now.date_naive();
        let now_secs = now.time().num_seconds_from_midnight();
        let key = date_key(today);

        let mut due: Vec<(String, String)> = Vec::new();
        if let Some(items) = self.todos.get(&key) {
            for item in items {
                if item.done {
                    continue;
                }
                let Some(remind_text) = &item.remind_at else {
                    continue;
                };
                let Some(remind_secs) = parse_hhmm(remind_text) else {
                    continue;
                };
                let trigger_id = format!("{key}|{}|{remind_text}", item.text);
                if self.triggered_reminders.contains(&trigger_id) {
                    continue;
                }
                if now_secs >= remind_secs {
                    due.push((trigger_id, format!("{}（{}）", item.text, remind_text)));
                }
            }
        }
        for (trigger_id, reminder) in due {
            self.triggered_reminders.insert(trigger_id);
            self.active_reminder = Some(reminder);
            self.show_from_tray(ctx);
            ctx.send_viewport_cmd(ViewportCommand::RequestUserAttention(
                egui::UserAttentionType::Critical,
            ));
        }
    }

    fn next_repaint_delay(&self) -> std::time::Duration {
        let now = Local::now();
        let key = date_key(now.date_naive());
        let now_secs = now.time().num_seconds_from_midnight();
        let mut next = 30u64;
        if let Some(items) = self.todos.get(&key) {
            for item in items {
                if item.done {
                    continue;
                }
                let Some(remind_text) = &item.remind_at else {
                    continue;
                };
                let Some(remind_secs) = parse_hhmm(remind_text) else {
                    continue;
                };
                let trigger_id = format!("{key}|{}|{remind_text}", item.text);
                if self.triggered_reminders.contains(&trigger_id) {
                    continue;
                }
                let remain = remind_secs.saturating_sub(now_secs);
                next = next.min(remain.max(1) as u64);
            }
        }
        std::time::Duration::from_secs(next)
    }

    fn add_todo(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        let remind_at = if self.remind_enabled {
            Some(format!(
                "{:02}:{:02}",
                self.remind_hour, self.remind_minute
            ))
        } else {
            None
        };
        let key = date_key(self.selected);
        self.todos.entry(key).or_default().push(TodoItem {
            text,
            done: false,
            remind_at,
        });
        self.input.clear();
        save_todos(&self.todos);
    }

    fn draw_titlebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Panel::top("titlebar")
            .frame(egui::Frame::NONE.fill(self.theme().titlebar))
            .show(ui, |ui| {
                ui.set_min_height(46.0);
                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);
                    if ui
                        .button(RichText::new("‹").size(17.0))
                        .on_hover_text("上个月")
                        .clicked()
                    {
                        self.shift_month(-1);
                    }
                    if ui
                        .button(RichText::new("›").size(17.0))
                        .on_hover_text("下个月")
                        .clicked()
                    {
                        self.shift_month(1);
                    }
                    if ui
                        .button(RichText::new("今天").size(14.0))
                        .on_hover_text("回到今天")
                        .clicked()
                    {
                        let today = Local::now().date_naive();
                        self.view_year = today.year();
                        self.view_month = today.month();
                        self.selected = today;
                    }

                    let month_text = format!("{}年{}月", self.view_year, self.view_month);
                    let title_rect = ui.available_rect_before_wrap();
                    let (rect, response) = ui.allocate_exact_size(title_rect.size(), Sense::drag());
                    if response.dragged() {
                        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                    }
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        month_text,
                        FontId::proportional(17.0),
                        self.theme().text_title,
                    );

                    let theme = self.theme();
                    if ui
                        .button(RichText::new(theme.icon).size(15.0))
                        .on_hover_text(format!("主题：{}（点击切换）", theme.name))
                        .clicked()
                    {
                        self.cycle_theme(ctx);
                    }
                    let pin_label = if self.pinned { "置顶✓" } else { "置顶" };
                    if ui
                        .button(RichText::new(pin_label).size(13.0))
                        .on_hover_text("快捷键 Ctrl+Alt+T")
                        .clicked()
                    {
                        self.set_pinned(ctx, !self.pinned);
                    }
                    if ui
                        .button(RichText::new("—").size(15.0))
                        .on_hover_text("最小化到托盘")
                        .clicked()
                    {
                        self.hide_to_tray(ctx);
                    }
                    if ui
                        .button(RichText::new("✕").size(15.0).color(self.theme().danger))
                        .on_hover_text("关闭到托盘")
                        .clicked()
                    {
                        self.hide_to_tray(ctx);
                    }
                    ui.add_space(10.0);
                });
            });
    }

    fn shift_month(&mut self, delta: i32) {
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

    fn draw_calendar(&mut self, ui: &mut egui::Ui) {
        let spacing = ui.spacing().item_spacing.x;
        let cell_w = (ui.available_width() - spacing * 6.0) / 7.0;
        let cell_h = 82.0;
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
                    let bg = if is_hovered && !is_selected {
                        lighten(bg) // hover 提亮一档
                    } else {
                        bg
                    };
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
                        let mut y = rect.left_top().y + 26.0;
                        for item in items.iter().take(3) {
                            if y + 14.0 > rect.bottom() - 3.0 {
                                break;
                            }
                            let prefix = if item.done {
                                "✓ "
                            } else if item.remind_at.is_some() {
                                "⏰ "
                            } else {
                                "• "
                            };
                            let text = format!("{}{}", prefix, truncate_chars(&item.text, 10));
                            let color = if item.done {
                                theme.text_done
                            } else {
                                theme.text_secondary
                            };
                            painter.text(
                                Pos2::new(rect.left() + 8.0, y),
                                Align2::LEFT_TOP,
                                text,
                                FontId::proportional(11.0),
                                color,
                            );
                            y += 15.0;
                        }
                        if items.len() > 3 {
                            let badge_text = format!("+{}", items.len() - 3);
                            let badge_pos = Pos2::new(rect.right() - 6.0, rect.bottom() - 5.0);
                            let badge_rect = egui::Rect::from_center_size(
                                badge_pos,
                                Vec2::new(24.0, 14.0),
                            );
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
    }

    fn draw_editor(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let weekday = WEEKDAY_LABELS[self.selected.weekday().num_days_from_monday() as usize];
        ui.label(
            RichText::new(format!(
                "{}月{}日 周{}",
                self.selected.month(),
                self.selected.day(),
                weekday
            ))
            .size(16.0)
            .color(self.theme().accent),
        );
        ui.add_space(8.0);

        let mut add_clicked = false;
        ui.horizontal(|ui| {
            let response = ui.add_sized(
                [ui.available_width() - 72.0, 32.0],
                egui::TextEdit::singleline(&mut self.input)
                    .hint_text("添加待办，回车确认…")
                    .desired_width(f32::INFINITY),
            );
            if ui
                .button(RichText::new("添加").size(14.0).color(self.theme().accent))
                .clicked()
            {
                add_clicked = true;
            }
            let enter = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if enter || add_clicked {
                self.add_todo();
                response.request_focus();
            }
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.remind_enabled, "⏰ 到点提醒");
            if self.remind_enabled {
                ui.add_space(6.0);
                ui.add(
                    DragValue::new(&mut self.remind_hour)
                        .range(0..=23)
                        .suffix("时"),
                );
                ui.label(":");
                ui.add(
                    DragValue::new(&mut self.remind_minute)
                        .range(0..=59)
                        .suffix("分"),
                );
                if ui
                    .button(RichText::new("现在").small())
                    .on_hover_text("设为当前时间")
                    .clicked()
                {
                    let now = Local::now();
                    self.remind_hour = now.hour() as i32;
                    self.remind_minute = now.minute() as i32;
                }
            }
        });

        ui.add_space(6.0);
        egui::ScrollArea::vertical()
            .max_height(200.0)
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
                    for index in 0..todos.len() {
                        ui.horizontal(|ui| {
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
                            ui.label(text);
                            if let Some(remind_at) = &todos[index].remind_at {
                                ui.label(
                                    RichText::new(format!("⏰ {remind_at}"))
                                        .small()
                                        .color(theme.accent),
                                );
                            }
                            if ui
                                .button(RichText::new("✕").size(13.0).color(theme.danger))
                                .on_hover_text("删除")
                                .clicked()
                            {
                                todos.remove(index);
                                changed = true;
                            }
                        });
                        if index + 1 < todos.len() {
                            ui.add_space(4.0);
                        }
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new("清除已完成")
                                    .small()
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

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max_chars).collect();
        format!("{}…", cut)
    }
}

fn lighten(color: Color32) -> Color32 {
    let f = |channel: u8| channel as u32 + ((255 - channel as u32) * 12 / 100);
    Color32::from_rgb(
        f(color.r()) as u8,
        f(color.g()) as u8,
        f(color.b()) as u8,
    )
}

fn parse_hhmm(value: &str) -> Option<u32> {
    let (hour, minute) = value.split_once(':')?;
    let hour: u32 = hour.parse().ok()?;
    let minute: u32 = minute.parse().ok()?;
    if hour < 24 && minute < 60 {
        Some(hour * 3600 + minute * 60)
    } else {
        None
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.tray.is_none() {
            self.init_tray(ctx);
        }

        self.handle_tray_actions(ctx);
        self.check_reminders(ctx);

        if ctx.input(|i| i.viewport().close_requested()) {
            self.hide_to_tray(ctx);
        }

        let delay = self.next_repaint_delay();
        ctx.request_repaint_after(delay);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.draw_titlebar(&ctx, ui);

        if let Some(reminder) = self.active_reminder.clone() {
            egui::Frame::NONE
                .fill(self.theme().reminder_banner)
                .corner_radius(CornerRadius::same(10))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("⏰ 待办提醒：{reminder}"))
                                .size(14.0)
                                .color(Color32::WHITE)
                                .strong(),
                        );
                        if ui
                            .button(RichText::new("知道了").size(13.0).color(Color32::WHITE))
                            .clicked()
                        {
                            self.active_reminder = None;
                        }
                    });
                });
            ui.add_space(4.0);
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.theme().bg)
                    .inner_margin(10.0),
            )
            .show(ui, |ui| {
                self.draw_calendar(ui);
                ui.add_space(4.0);
                egui::Frame::NONE
                    .fill(self.theme().card)
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.draw_editor(ui);
                    });
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(false)
            .with_inner_size([560.0, 952.0])
            .with_min_inner_size([520.0, 860.0])
            .with_title("桌面日历待办")
            .with_icon(Arc::new(calendar_icon_data())),
        ..Default::default()
    };
    eframe::run_native(
        "deskTodo",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
