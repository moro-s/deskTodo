use eframe::egui::{Color32, FontId, Ui, Vec2};
use std::borrow::Cow;
use std::sync::OnceLock;

static PADDED_NUMBERS: OnceLock<Box<[String; 100]>> = OnceLock::new();
static NUMBERS: OnceLock<Box<[String; 100]>> = OnceLock::new();

pub(crate) fn padded_number(value: u32) -> &'static str {
    let cache = PADDED_NUMBERS.get_or_init(|| {
        let mut list = Vec::with_capacity(100);
        for n in 0..100 {
            list.push(format!("{n:02}"));
        }
        list.into_boxed_slice().try_into().ok().unwrap()
    });
    cache
        .get(value as usize)
        .map(String::as_str)
        .unwrap_or_default()
}

pub(crate) fn number(value: u32) -> &'static str {
    let cache = NUMBERS.get_or_init(|| {
        let mut list = Vec::with_capacity(100);
        for n in 0..100 {
            list.push(n.to_string());
        }
        list.into_boxed_slice().try_into().ok().unwrap()
    });
    cache
        .get(value as usize)
        .map(String::as_str)
        .unwrap_or_default()
}

pub(crate) struct TextMetrics {
    pub(crate) width: f32,
    pub(crate) visual_offset: Vec2,
}

pub(crate) fn text_metrics(ui: &Ui, text: &str, font_id: &FontId) -> TextMetrics {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font_id.clone(), Color32::WHITE);
    let visual_offset = if galley.mesh_bounds.is_positive() {
        galley.mesh_bounds.center() - galley.rect.center()
    } else {
        Vec2::ZERO
    };
    TextMetrics {
        width: galley.size().x,
        visual_offset,
    }
}

pub(crate) fn truncate_chars(text: &str, max_chars: usize) -> Cow<'_, str> {
    if text.chars().count() <= max_chars {
        Cow::Borrowed(text)
    } else {
        let cut: String = text.chars().take(max_chars).collect();
        Cow::Owned(format!("{cut}…"))
    }
}
