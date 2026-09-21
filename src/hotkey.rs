use crate::tray::{push_tray_action, TrayAction};
use eframe::egui;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

pub(crate) fn register_pin_toggle(ctx: &egui::Context) -> GlobalHotKeyManager {
    let manager = GlobalHotKeyManager::new().expect("无法创建全局热键管理器");
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT);
    if let Err(err) = manager.register(hotkey) {
        eprintln!("注册热键失败: {err}");
    }

    let ctx = ctx.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.state() == HotKeyState::Pressed {
            push_tray_action(TrayAction::TogglePin);
        }
        ctx.request_repaint();
    }));

    manager
}
