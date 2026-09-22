use crate::core::config::HotkeySpec;
use crate::platform::tray::{push_tray_action, TrayAction};
use eframe::egui;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

const KEY_TABLE: &[(egui::Key, Code, &str)] = &[
    (egui::Key::A, Code::KeyA, "A"),
    (egui::Key::B, Code::KeyB, "B"),
    (egui::Key::C, Code::KeyC, "C"),
    (egui::Key::D, Code::KeyD, "D"),
    (egui::Key::E, Code::KeyE, "E"),
    (egui::Key::F, Code::KeyF, "F"),
    (egui::Key::G, Code::KeyG, "G"),
    (egui::Key::H, Code::KeyH, "H"),
    (egui::Key::I, Code::KeyI, "I"),
    (egui::Key::J, Code::KeyJ, "J"),
    (egui::Key::K, Code::KeyK, "K"),
    (egui::Key::L, Code::KeyL, "L"),
    (egui::Key::M, Code::KeyM, "M"),
    (egui::Key::N, Code::KeyN, "N"),
    (egui::Key::O, Code::KeyO, "O"),
    (egui::Key::P, Code::KeyP, "P"),
    (egui::Key::Q, Code::KeyQ, "Q"),
    (egui::Key::R, Code::KeyR, "R"),
    (egui::Key::S, Code::KeyS, "S"),
    (egui::Key::T, Code::KeyT, "T"),
    (egui::Key::U, Code::KeyU, "U"),
    (egui::Key::V, Code::KeyV, "V"),
    (egui::Key::W, Code::KeyW, "W"),
    (egui::Key::X, Code::KeyX, "X"),
    (egui::Key::Y, Code::KeyY, "Y"),
    (egui::Key::Z, Code::KeyZ, "Z"),
    (egui::Key::Num0, Code::Digit0, "0"),
    (egui::Key::Num1, Code::Digit1, "1"),
    (egui::Key::Num2, Code::Digit2, "2"),
    (egui::Key::Num3, Code::Digit3, "3"),
    (egui::Key::Num4, Code::Digit4, "4"),
    (egui::Key::Num5, Code::Digit5, "5"),
    (egui::Key::Num6, Code::Digit6, "6"),
    (egui::Key::Num7, Code::Digit7, "7"),
    (egui::Key::Num8, Code::Digit8, "8"),
    (egui::Key::Num9, Code::Digit9, "9"),
    (egui::Key::F1, Code::F1, "F1"),
    (egui::Key::F2, Code::F2, "F2"),
    (egui::Key::F3, Code::F3, "F3"),
    (egui::Key::F4, Code::F4, "F4"),
    (egui::Key::F5, Code::F5, "F5"),
    (egui::Key::F6, Code::F6, "F6"),
    (egui::Key::F7, Code::F7, "F7"),
    (egui::Key::F8, Code::F8, "F8"),
    (egui::Key::F9, Code::F9, "F9"),
    (egui::Key::F10, Code::F10, "F10"),
    (egui::Key::F11, Code::F11, "F11"),
    (egui::Key::F12, Code::F12, "F12"),
];

pub(crate) fn name_from_egui_key(key: egui::Key) -> Option<&'static str> {
    KEY_TABLE
        .iter()
        .find(|(table_key, _, _)| *table_key == key)
        .map(|(_, _, name)| *name)
}

fn code_from_name(name: &str) -> Option<Code> {
    KEY_TABLE
        .iter()
        .find(|(_, _, table_name)| *table_name == name)
        .map(|(_, code, _)| *code)
}

pub(crate) fn spec_to_hotkey(spec: &HotkeySpec) -> Option<HotKey> {
    let code = code_from_name(&spec.key)?;
    let mut mods = Modifiers::empty();
    if spec.ctrl {
        mods |= Modifiers::CONTROL;
    }
    if spec.alt {
        mods |= Modifiers::ALT;
    }
    if spec.shift {
        mods |= Modifiers::SHIFT;
    }
    if spec.meta {
        mods |= Modifiers::META;
    }
    Some(HotKey::new(Some(mods), code))
}

pub(crate) fn register_hotkey(
    ctx: &egui::Context,
    spec: &HotkeySpec,
) -> (GlobalHotKeyManager, HotKey) {
    let manager = GlobalHotKeyManager::new().expect("无法创建全局热键管理器");
    let fallback = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT);
    let hotkey = spec_to_hotkey(spec).unwrap_or(fallback);
    match manager.register(hotkey) {
        Ok(()) => crate::log_info!("hotkey", "全局热键已注册：{}", spec.display()),
        Err(err) => crate::log_error!("hotkey", "注册热键失败（{}）：{err}", spec.display()),
    }

    let ctx = ctx.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.state() == HotKeyState::Pressed {
            push_tray_action(TrayAction::TogglePin);
        }
        ctx.request_repaint();
    }));

    (manager, hotkey)
}

pub(crate) fn reregister(
    manager: &GlobalHotKeyManager,
    old: HotKey,
    spec: &HotkeySpec,
) -> Result<HotKey, String> {
    let _ = manager.unregister(old);
    let hotkey = spec_to_hotkey(spec).ok_or_else(|| "无效的快捷键组合".to_string())?;
    match manager.register(hotkey) {
        Ok(()) => {
            crate::log_info!("hotkey", "全局热键已更新：{}", spec.display());
            Ok(hotkey)
        }
        Err(err) => {
            let message = format!("快捷键可能被其他程序占用（{err}）");
            crate::log_error!("hotkey", "更新热键失败：{message}");
            Err(message)
        }
    }
}
