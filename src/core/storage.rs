use crate::core::config::{default_log_level, Config};
use crate::core::models::TodoStore;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

pub(crate) fn default_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("deskTodo"))
}

fn config_file() -> Option<PathBuf> {
    default_data_dir().map(|dir| dir.join("config.json"))
}

fn read_custom_data_dir() -> Option<PathBuf> {
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

static CUSTOM_DIR_CACHE: RwLock<Option<Option<PathBuf>>> = RwLock::new(None);

fn custom_data_dir() -> Option<PathBuf> {
    if let Some(cached) = &*CUSTOM_DIR_CACHE.read().ok()? {
        return cached.clone();
    }
    let computed = read_custom_data_dir();
    if let Ok(mut slot) = CUSTOM_DIR_CACHE.write() {
        *slot = Some(computed.clone());
    }
    computed
}

pub(crate) fn invalidate_data_dir_cache() {
    if let Ok(mut slot) = CUSTOM_DIR_CACHE.write() {
        *slot = None;
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

fn write_json(path: Option<PathBuf>, json: String) -> bool {
    let Some(path) = path else {
        crate::log_error!("storage", "数据目录不可用，写入被跳过");
        return false;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::write(&path, json) {
        Ok(()) => true,
        Err(err) => {
            crate::log_error!("storage", "写入失败 {}：{err}", path.display());
            false
        }
    }
}

pub(crate) fn load_todos() -> TodoStore {
    for path in [data_file(), default_data_file()].into_iter().flatten() {
        if let Ok(raw) = fs::read_to_string(&path)
            && let Ok(store) = serde_json::from_str(&raw)
        {
            return store;
        }
        if path.exists() {
            crate::log_warn!("storage", "待办数据文件损坏，已跳过：{}", path.display());
        }
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
        && let Ok(raw) = fs::read_to_string(&path)
        && let Ok(config) = serde_json::from_str(&raw)
    {
        return config;
    }
    if let Some(path) = config_file()
        && path.exists()
    {
        crate::log_warn!("storage", "配置文件损坏，已使用默认配置：{}", path.display());
    }
    Config {
        theme: 0,
        font_scale: 1.0,
        opacity: 1.0,
        hotkey: Default::default(),
        data_dir: None,
        log_level: default_log_level(),
    }
}

pub(crate) fn save_config(config: &Config) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        write_json(config_file(), json);
    }
}
