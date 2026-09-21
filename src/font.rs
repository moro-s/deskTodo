use eframe::egui;
use std::fs;

fn cjk_font_candidates() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
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
    }
}

fn symbol_font_candidates() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
        &[r"C:\Windows\Fonts\seguisym.ttf"]
    } else if cfg!(target_os = "macos") {
        &["/System/Library/Fonts/Apple Symbols.ttf"]
    } else {
        &[
            "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
            "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
        ]
    }
}

pub(crate) fn install_cjk_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut cjk_loaded = false;

    for path in cjk_font_candidates() {
        if let Ok(bytes) = fs::read(path) {
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
            cjk_loaded = true;
            break;
        }
    }

    for path in symbol_font_candidates() {
        if let Ok(bytes) = fs::read(path) {
            fonts
                .font_data
                .insert("symbols".into(), egui::FontData::from_owned(bytes).into());
            let insert_at = usize::from(cjk_loaded);
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(insert_at, "symbols".into());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(insert_at, "symbols".into());
            break;
        }
    }

    ctx.set_fonts(fonts);
}
