use crate::core::config::HotkeySpec;
use crate::platform::tray::{push_tray_action, TrayAction};
use eframe::egui;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

pub(crate) fn name_from_egui_key(key: egui::Key) -> Option<&'static str> {
    use egui::Key as K;
    match key {
        K::A => Some("A"),
        K::B => Some("B"),
        K::C => Some("C"),
        K::D => Some("D"),
        K::E => Some("E"),
        K::F => Some("F"),
        K::G => Some("G"),
        K::H => Some("H"),
        K::I => Some("I"),
        K::J => Some("J"),
        K::K => Some("K"),
        K::L => Some("L"),
        K::M => Some("M"),
        K::N => Some("N"),
        K::O => Some("O"),
        K::P => Some("P"),
        K::Q => Some("Q"),
        K::R => Some("R"),
        K::S => Some("S"),
        K::T => Some("T"),
        K::U => Some("U"),
        K::V => Some("V"),
        K::W => Some("W"),
        K::X => Some("X"),
        K::Y => Some("Y"),
        K::Z => Some("Z"),
        K::Num0 => Some("0"),
        K::Num1 => Some("1"),
        K::Num2 => Some("2"),
        K::Num3 => Some("3"),
        K::Num4 => Some("4"),
        K::Num5 => Some("5"),
        K::Num6 => Some("6"),
        K::Num7 => Some("7"),
        K::Num8 => Some("8"),
        K::Num9 => Some("9"),
        K::F1 => Some("F1"),
        K::F2 => Some("F2"),
        K::F3 => Some("F3"),
        K::F4 => Some("F4"),
        K::F5 => Some("F5"),
        K::F6 => Some("F6"),
        K::F7 => Some("F7"),
        K::F8 => Some("F8"),
        K::F9 => Some("F9"),
        K::F10 => Some("F10"),
        K::F11 => Some("F11"),
        K::F12 => Some("F12"),
        _ => None,
    }
}

fn code_from_name(name: &str) -> Option<Code> {
    match name {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => None,
    }
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
    let fallback =
        HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT);
    let hotkey = spec_to_hotkey(spec).unwrap_or(fallback);
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

    (manager, hotkey)
}

pub(crate) fn reregister(
    manager: &GlobalHotKeyManager,
    old: HotKey,
    spec: &HotkeySpec,
) -> Result<HotKey, String> {
    let _ = manager.unregister(old);
    let hotkey = spec_to_hotkey(spec).ok_or_else(|| "无效的快捷键组合".to_string())?;
    manager
        .register(hotkey)
        .map_err(|err| format!("快捷键可能被其他程序占用（{err}）"))?;
    Ok(hotkey)
}
