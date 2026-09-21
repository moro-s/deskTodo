use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct TodoItem {
    pub(crate) text: String,
    pub(crate) done: bool,
    #[serde(default)]
    pub(crate) remind_at: Option<String>,
}

pub(crate) type TodoStore = BTreeMap<String, Vec<TodoItem>>;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct HotkeySpec {
    #[serde(default)]
    pub(crate) ctrl: bool,
    #[serde(default)]
    pub(crate) alt: bool,
    #[serde(default)]
    pub(crate) shift: bool,
    #[serde(default)]
    pub(crate) meta: bool,
    #[serde(default = "default_hotkey_key")]
    pub(crate) key: String,
}

impl Default for HotkeySpec {
    fn default() -> Self {
        Self {
            ctrl: true,
            alt: true,
            shift: false,
            meta: false,
            key: default_hotkey_key(),
        }
    }
}

fn default_hotkey_key() -> String {
    "T".to_string()
}

impl HotkeySpec {
    pub(crate) fn display(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.meta {
            parts.push("Win");
        }
        parts.push(&self.key);
        parts.join(" + ")
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) theme: usize,
    #[serde(default = "default_font_scale")]
    pub(crate) font_scale: f32,
    #[serde(default = "default_opacity")]
    pub(crate) opacity: f32,
    #[serde(default)]
    pub(crate) hotkey: HotkeySpec,
    #[serde(default)]
    pub(crate) data_dir: Option<String>,
}

fn default_font_scale() -> f32 {
    1.0
}

fn default_opacity() -> f32 {
    1.0
}

pub(crate) fn date_key(date: NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}
