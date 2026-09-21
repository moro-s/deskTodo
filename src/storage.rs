use crate::models::{Config, TodoStore};
use std::fs;
use std::path::PathBuf;

fn app_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("deskTodo"))
}

fn data_file() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("todos.json"))
}

fn config_file() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("config.json"))
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
    Config { theme: 0 }
}

pub(crate) fn save_config(config: &Config) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        write_json(config_file(), json);
    }
}
