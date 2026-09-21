use crate::models::{Config, TodoStore};
use std::fs;
use std::path::PathBuf;

fn default_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("deskTodo"))
}

fn config_file() -> Option<PathBuf> {
    default_data_dir().map(|dir| dir.join("config.json"))
}

fn custom_data_dir() -> Option<PathBuf> {
    let path = config_file()?;
    let raw = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let dir = value.get("data_dir")?.as_str()?;
    if dir.is_empty() {
        None
    } else {
        Some(PathBuf::from(dir))
    }
}

fn app_data_dir() -> Option<PathBuf> {
    custom_data_dir().or_else(default_data_dir)
}

fn data_file() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("todos.json"))
}

fn default_data_file() -> Option<PathBuf> {
    default_data_dir().map(|dir| dir.join("todos.json"))
}

pub(crate) fn current_data_dir() -> Option<PathBuf> {
    app_data_dir()
}

fn write_json(path: Option<PathBuf>, json: String) {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, json);
    }
}

pub(crate) fn load_todos() -> TodoStore {
    if let Some(path) = data_file()
        && let Ok(raw) = fs::read_to_string(path)
        && let Ok(store) = serde_json::from_str(&raw)
    {
        return store;
    }
    if let Some(path) = default_data_file()
        && let Ok(raw) = fs::read_to_string(path)
        && let Ok(store) = serde_json::from_str(&raw)
    {
        return store;
    }
    TodoStore::new()
}

pub(crate) fn save_todos(todos: &TodoStore) {
    if let Ok(json) = serde_json::to_string_pretty(todos) {
        write_json(data_file(), json);
    }
}

pub(crate) fn load_config() -> Config {
    if let Some(path) = config_file()
        && let Ok(raw) = fs::read_to_string(path)
        && let Ok(config) = serde_json::from_str(&raw)
    {
        return config;
    }
    Config {
        theme: 0,
        font_scale: 1.0,
        hotkey: Default::default(),
        data_dir: None,
    }
}

pub(crate) fn save_config(config: &Config) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        write_json(config_file(), json);
    }
}
