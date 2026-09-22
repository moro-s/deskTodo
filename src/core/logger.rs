use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    pub(crate) fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" => Self::Off,
            "error" => Self::Error,
            "warn" => Self::Warn,
            "debug" => Self::Debug,
            _ => Self::Info,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }

    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::Off => "关闭",
            Self::Error => "错误",
            Self::Warn => "警告",
            Self::Info => "信息",
            Self::Debug => "调试",
        }
    }
}

pub(crate) const LOG_FILE_NAME: &str = "deskTodo.log";
const MAX_LOG_SIZE: u64 = 1024 * 1024;
const LOG_HEADER: &str = "===== deskTodo 日志 =====";

struct LoggerState {
    level: LogLevel,
    file: Option<Arc<Mutex<File>>>,
    path: Option<PathBuf>,
}

static STATE: RwLock<Option<LoggerState>> = RwLock::new(None);

fn open_state(dir: Option<PathBuf>, level: LogLevel) -> LoggerState {
    let Some(dir) = dir else {
        return LoggerState {
            level,
            file: None,
            path: None,
        };
    };
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(LOG_FILE_NAME);
    if let Ok(meta) = std::fs::metadata(&path)
        && meta.len() > MAX_LOG_SIZE
    {
        let _ = std::fs::rename(&path, dir.join(format!("{LOG_FILE_NAME}.old")));
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();
    if file.is_none() {
        eprintln!("无法打开日志文件：{}", path.display());
    }
    LoggerState {
        level,
        file: file.map(|handle| Arc::new(Mutex::new(handle))),
        path: Some(path),
    }
}

pub(crate) fn init(level: LogLevel) {
    let dir = crate::core::storage::current_data_dir()
        .or_else(crate::core::storage::default_data_dir);
    init_at(dir, level);
}

fn init_at(dir: Option<PathBuf>, level: LogLevel) {
    let state = open_state(dir, level);
    if let Some(file) = &state.file
        && let Ok(mut handle) = file.lock()
    {
        let _ = writeln!(
            handle,
            "\n{LOG_HEADER} 启动于 {}",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );
    }
    if let Ok(mut guard) = STATE.write() {
        *guard = Some(state);
    }
}

pub(crate) fn set_level(level: LogLevel) {
    if let Ok(mut guard) = STATE.write()
        && let Some(state) = guard.as_mut()
    {
        state.level = level;
    }
}

pub(crate) fn reopen(level: LogLevel) {
    let dir = crate::core::storage::current_data_dir()
        .or_else(crate::core::storage::default_data_dir);
    let state = open_state(dir, level);
    if let Ok(mut guard) = STATE.write() {
        *guard = Some(state);
    }
}

pub(crate) fn enabled(level: LogLevel) -> bool {
    STATE
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(|state| level <= state.level))
        .unwrap_or(false)
}

pub(crate) fn log(level: LogLevel, target: &str, message: &str) {
    if !enabled(level) {
        return;
    }
    let line = format!(
        "{} [{:<5}] [{target}] {message}",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        level.as_str().to_uppercase()
    );
    eprintln!("{line}");
    let file = STATE
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|state| state.file.clone()));
    if let Some(file) = file
        && let Ok(mut handle) = file.lock()
    {
        let _ = writeln!(handle, "{line}");
    }
}

pub(crate) fn log_file_path() -> Option<PathBuf> {
    STATE
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|state| state.path.clone()))
}

pub(crate) fn clear() {
    if let Some(path) = log_file_path() {
        let _ = File::create(&path);
    }
}

pub(crate) fn open_in_editor(path: &Path) {
    if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("notepad").arg(path).spawn();
    } else if cfg!(target_os = "macos") {
        let _ = std::process::Command::new("open")
            .arg("-t")
            .arg(path)
            .spawn();
    } else {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

#[macro_export]
macro_rules! log_error {
    ($target:expr, $($arg:tt)+) => {{
        let level = $crate::core::logger::LogLevel::Error;
        if $crate::core::logger::enabled(level) {
            $crate::core::logger::log(level, $target, &format!($($arg)+));
        }
    }};
}

#[macro_export]
macro_rules! log_warn {
    ($target:expr, $($arg:tt)+) => {{
        let level = $crate::core::logger::LogLevel::Warn;
        if $crate::core::logger::enabled(level) {
            $crate::core::logger::log(level, $target, &format!($($arg)+));
        }
    }};
}

#[macro_export]
macro_rules! log_info {
    ($target:expr, $($arg:tt)+) => {{
        let level = $crate::core::logger::LogLevel::Info;
        if $crate::core::logger::enabled(level) {
            $crate::core::logger::log(level, $target, &format!($($arg)+));
        }
    }};
}

#[macro_export]
macro_rules! log_debug {
    ($target:expr, $($arg:tt)+) => {{
        let level = $crate::core::logger::LogLevel::Debug;
        if $crate::core::logger::enabled(level) {
            $crate::core::logger::log(level, $target, &format!($($arg)+));
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "desktodo-log-test-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn reset() {
        *STATE.write().unwrap() = None;
    }

    #[test]
    fn parse_level_names() {
        assert_eq!(LogLevel::parse("off"), LogLevel::Off);
        assert_eq!(LogLevel::parse("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::parse("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::parse(" debug "), LogLevel::Debug);
        assert_eq!(LogLevel::parse("info"), LogLevel::Info);
        assert_eq!(LogLevel::parse("未知"), LogLevel::Info);
    }

    #[test]
    fn log_respects_level_and_runtime_switch() {
        let _guard = TEST_LOCK.lock().unwrap();
        let dir = temp_dir("level");
        init_at(Some(dir.clone()), LogLevel::Warn);
        log_info!("test", "这条信息级别不足，不应写入");
        log_warn!("test", "警告消息");
        set_level(LogLevel::Debug);
        log_info!("test", "级别提升后的信息");
        log_debug!("test", "调试消息");
        let content = std::fs::read_to_string(dir.join(LOG_FILE_NAME)).unwrap();
        assert!(content.contains("[WARN ] [test] 警告消息"));
        assert!(content.contains("[INFO ] [test] 级别提升后的信息"));
        assert!(content.contains("[DEBUG] [test] 调试消息"));
        assert!(!content.contains("这条信息级别不足"));
        assert!(content.contains(LOG_HEADER));
        reset();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn log_before_init_is_noop() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        log_error!("test", "未初始化时不应输出");
        assert!(log_file_path().is_none());
    }

    #[test]
    fn clear_truncates_file() {
        let _guard = TEST_LOCK.lock().unwrap();
        let dir = temp_dir("clear");
        init_at(Some(dir.clone()), LogLevel::Info);
        log_info!("test", "将被清空的内容");
        clear();
        let content = std::fs::read_to_string(dir.join(LOG_FILE_NAME)).unwrap();
        assert!(content.is_empty());
        log_info!("test", "清空后可以继续写入");
        let content = std::fs::read_to_string(dir.join(LOG_FILE_NAME)).unwrap();
        assert!(content.contains("清空后可以继续写入"));
        reset();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
