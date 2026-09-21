#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;
use eframe::egui::{
    Align2, Color32, CornerRadius, FontId, Key, Pos2, RichText, Sense, Vec2, ViewportCommand,
    WindowLevel,
};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const WEEKDAY_LABELS: [&str; 7] = ["一", "二", "三", "四", "五", "六", "日"];

#[derive(Serialize, Deserialize, Clone)]
struct TodoItem {
    text: String,
    done: bool,
}

type TodoStore = BTreeMap<String, Vec<TodoItem>>;

struct App {
    todos: TodoStore,
    input: String,
    view_year: i32,
    view_month: u32,
    selected: NaiveDate,
    pinned: bool,
    #[allow(dead_code)]
    hotkey_manager: GlobalHotKeyManager,
}

fn date_key(date: NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

fn data_file() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("deskTodo").join("todos.json"))
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

fn install_cjk_font(ctx: &egui::Context) {
    if std::env::var_os("DESKTODO_NO_CJK").is_some() {
        return;
    }
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

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_cjk_font(&cc.egui_ctx);
        let today = Local::now().date_naive();
        let hotkey_manager = GlobalHotKeyManager::new().expect("无法创建全局热键管理器");
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT);
        if let Err(err) = hotkey_manager.register(hotkey) {
            eprintln!("注册热键失败: {err}");
        }
        Self {
            todos: load_todos(),
            input: String::new(),
            view_year: today.year(),
            view_month: today.month(),
            selected: today,
            pinned: false,
            hotkey_manager,
        }
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

    fn add_todo(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        let key = date_key(self.selected);
        self.todos
            .entry(key)
            .or_default()
            .push(TodoItem { text, done: false });
        self.input.clear();
        save_todos(&self.todos);
    }

    fn draw_titlebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Panel::top("titlebar")
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(30, 32, 38)))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(6.0);
                    if ui.button("‹").clicked() {
                        self.shift_month(-1);
                    }
                    if ui.button("›").clicked() {
                        self.shift_month(1);
                    }
                    if ui.button("今天").clicked() {
                        let today = Local::now().date_naive();
                        self.view_year = today.year();
                        self.view_month = today.month();
                        self.selected = today;
                    }

                    let month_text = format!("{}年{}月", self.view_year, self.view_month);
                    let title_rect = ui.available_rect_before_wrap();
                    let (rect, response) =
                        ui.allocate_exact_size(title_rect.size(), Sense::drag());
                    if response.dragged() {
                        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                    }
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        month_text,
                        FontId::proportional(15.0),
                        Color32::from_gray(230),
                    );

                    let pin_label = if self.pinned { "置顶✓" } else { "置顶" };
                    if ui
                        .button(RichText::new(pin_label).small())
                        .on_hover_text("快捷键 Ctrl+Alt+T")
                        .clicked()
                    {
                        self.set_pinned(ctx, !self.pinned);
                    }
                    if ui.button("—").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                    }
                    if ui
                        .button(RichText::new("✕").color(Color32::from_rgb(230, 90, 90)))
                        .clicked()
                    {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                    ui.add_space(6.0);
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
        let cell_h = 74.0;

        egui::Grid::new("weekday_header")
            .num_columns(7)
            .spacing([spacing, 4.0])
            .show(ui, |ui| {
                for (idx, label) in WEEKDAY_LABELS.iter().enumerate() {
                    let color = if idx >= 5 {
                        Color32::from_rgb(235, 130, 120)
                    } else {
                        Color32::from_gray(150)
                    };
                    ui.add_sized(
                        [cell_w, 14.0],
                        egui::Label::new(
                            RichText::new(*label).color(color).font(FontId::proportional(12.0)),
                        )
                        .selectable(false)
                        .halign(egui::Align::Center),
                    );
                }
            });

        ui.add_space(4.0);

        let first = NaiveDate::from_ymd_opt(self.view_year, self.view_month, 1)
            .expect("非法日期");
        let lead = first.weekday().num_days_from_monday() as usize;
        let today = Local::now().date_naive();
        let mut clicked: Option<NaiveDate> = None;

        for row in 0..6 {
            ui.horizontal(|ui| {
                for col in 0..7 {
                    let date = first - chrono::Duration::days(lead as i64 - (row as i64 * 7 + col as i64));
                    let in_month = date.month() == self.view_month;
                    let is_today = date == today;
                    let is_selected = date == self.selected;
                    let key = date_key(date);
                    let todos = self.todos.get(&key);

                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(cell_w, cell_h), Sense::click());
                    if response.clicked() {
                        clicked = Some(date);
                    }

                    let bg = if is_selected {
                        Color32::from_rgb(64, 105, 190)
                    } else if is_today {
                        Color32::from_rgb(58, 64, 78)
                    } else if in_month {
                        Color32::from_rgb(44, 47, 54)
                    } else {
                        Color32::from_rgb(32, 34, 39)
                    };
                    ui.painter()
                        .rect_filled(rect, CornerRadius::same(6), bg);

                    let day_color = if is_today {
                        Color32::from_rgb(240, 200, 90)
                    } else if in_month {
                        Color32::from_gray(210)
                    } else {
                        Color32::from_gray(95)
                    };
                    ui.painter().text(
                        rect.left_top() + Vec2::new(7.0, 5.0),
                        Align2::LEFT_TOP,
                        date.day().to_string(),
                        FontId::proportional(12.0),
                        day_color,
                    );

                    if let Some(items) = todos {
                        let mut y = rect.left_top().y + 22.0;
                        for item in items.iter().take(2) {
                            let prefix = if item.done { "✓ " } else { "• " };
                            let text = format!("{}{}", prefix, truncate_chars(&item.text, 8));
                            let color = if item.done {
                                Color32::from_gray(110)
                            } else {
                                Color32::from_gray(200)
                            };
                            ui.painter().text(
                                Pos2::new(rect.left() + 7.0, y),
                                Align2::LEFT_TOP,
                                text,
                                FontId::proportional(10.0),
                                color,
                            );
                            y += 13.0;
                        }
                        if items.len() > 2 {
                            ui.painter().text(
                                Pos2::new(rect.right() - 7.0, rect.bottom() - 5.0),
                                Align2::RIGHT_BOTTOM,
                                format!("+{}", items.len() - 2),
                                FontId::proportional(10.0),
                                Color32::from_rgb(240, 200, 90),
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
        ui.add_space(8.0);
        let weekday = WEEKDAY_LABELS[self.selected.weekday().num_days_from_monday() as usize];
        ui.label(
            RichText::new(format!(
                "{}月{}日 周{}",
                self.selected.month(),
                self.selected.day(),
                weekday
            ))
            .size(15.0)
            .color(Color32::from_rgb(240, 200, 90)),
        );
        ui.add_space(6.0);

        let mut add_clicked = false;
        ui.horizontal(|ui| {
            let response = ui.add_sized(
                [ui.available_width() - 64.0, 24.0],
                egui::TextEdit::singleline(&mut self.input)
                    .hint_text("添加待办，回车确认…")
                    .desired_width(f32::INFINITY),
            );
            if ui.button("添加").clicked() {
                add_clicked = true;
            }
            let enter = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if enter || add_clicked {
                self.add_todo();
                response.request_focus();
            }
        });

        ui.add_space(6.0);
        egui::ScrollArea::vertical()
            .max_height(150.0)
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
                                .color(Color32::from_gray(120)),
                        );
                    });
                } else {
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
                            let mut text = RichText::new(todos[index].text.clone()).color(
                                if todos[index].done {
                                    Color32::from_gray(120)
                                } else {
                                    Color32::from_gray(220)
                                },
                            );
                            if todos[index].done {
                                text = text.strikethrough();
                            }
                            ui.label(text);
                            if ui
                                .button(RichText::new("✕").small().color(Color32::from_gray(140)))
                                .clicked()
                            {
                                todos.remove(index);
                                changed = true;
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new("清除已完成").small())
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

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state() == HotKeyState::Pressed {
                self.set_pinned(&ctx, !self.pinned);
            }
        }

        self.draw_titlebar(&ctx, ui);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(24, 26, 30))
                    .inner_margin(8.0),
            )
            .show(ui, |ui| {
                self.draw_calendar(ui);
                ui.add_space(2.0);
                egui::Frame::NONE
                    .fill(Color32::from_rgb(44, 47, 54))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.draw_editor(ui);
                    });
            });

        ctx.request_repaint_after(std::time::Duration::from_secs(30));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(false)
            .with_inner_size([452.0, 768.0])
            .with_title("桌面日历待办"),
        ..Default::default()
    };
    eframe::run_native(
        "deskTodo",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
