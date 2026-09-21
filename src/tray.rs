use crate::icon::{calendar_icon_data, tray_icon_from};
use eframe::egui;
use std::sync::{Mutex, OnceLock};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Clone, Copy)]
pub(crate) enum TrayAction {
    ToggleVisibility,
    TogglePin,
    OpenSettings,
    Exit,
}

static TRAY_ACTIONS: OnceLock<Mutex<Vec<TrayAction>>> = OnceLock::new();

pub(crate) fn push_tray_action(action: TrayAction) {
    let queue = TRAY_ACTIONS.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut queue) = queue.lock() {
        queue.push(action);
    }
}

pub(crate) fn take_tray_actions() -> Vec<TrayAction> {
    TRAY_ACTIONS
        .get()
        .and_then(|queue| queue.lock().ok())
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default()
}

pub(crate) fn build_tray(ctx: &egui::Context) -> Option<TrayIcon> {
    let menu = Menu::new();
    let settings_item = MenuItem::with_id("settings", "设置", true, None);
    let quit_item = MenuItem::with_id("quit", "退出", true, None);
    if let Err(err) = menu.append_items(&[&settings_item, &quit_item]) {
        eprintln!("构建托盘菜单失败: {err}");
        return None;
    }

    let icon_data = calendar_icon_data();
    let Some(icon) = tray_icon_from(&icon_data) else {
        eprintln!("创建托盘图标失败");
        return None;
    };

    let tray = match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip("桌面日历待办")
        .with_icon(icon)
        .build()
    {
        Ok(tray) => tray,
        Err(err) => {
            eprintln!("创建托盘失败: {err}");
            return None;
        }
    };

    let ctx_menu = ctx.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        match event.id().as_ref() {
            "settings" => push_tray_action(TrayAction::OpenSettings),
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

    Some(tray)
}
