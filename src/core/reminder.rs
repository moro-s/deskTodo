use crate::core::models::{date_key, TodoStore};
use chrono::{DateTime, Local, Timelike};
use std::collections::HashSet;
use std::time::Duration;

pub(crate) struct DueReminder {
    pub(crate) trigger_id: String,
    pub(crate) message: String,
}

pub(crate) fn parse_hms(value: &str) -> Option<u32> {
    let mut parts = value.split(':');
    let hour: u32 = parts.next()?.parse().ok()?;
    let minute: u32 = parts.next()?.parse().ok()?;
    let second: u32 = parts.next().map(|s| s.parse().ok()).unwrap_or(Some(0))?;
    if parts.next().is_none() && hour < 24 && minute < 60 && second < 60 {
        Some(hour * 3600 + minute * 60 + second)
    } else {
        None
    }
}

pub(crate) fn due_reminders(
    todos: &TodoStore,
    triggered: &HashSet<String>,
    now: DateTime<Local>,
) -> Vec<DueReminder> {
    let now_secs = now.time().num_seconds_from_midnight();
    let key = date_key(now.date_naive());
    let mut due = Vec::new();
    if let Some(items) = todos.get(&key) {
        for item in items {
            if item.done {
                continue;
            }
            let Some(remind_text) = &item.remind_at else {
                continue;
            };
            let Some(remind_secs) = parse_hms(remind_text) else {
                continue;
            };
            let trigger_id = format!("{key}|{}|{remind_text}", item.text);
            if triggered.contains(&trigger_id) {
                continue;
            }
            if now_secs >= remind_secs {
                due.push(DueReminder {
                    trigger_id,
                    message: format!("{}（{}）", item.text, remind_text),
                });
            }
        }
    }
    due
}

pub(crate) fn next_delay(
    todos: &TodoStore,
    triggered: &HashSet<String>,
    now: DateTime<Local>,
) -> Duration {
    let now_secs = now.time().num_seconds_from_midnight();
    let key = date_key(now.date_naive());
    let mut next = 30u64;
    if let Some(items) = todos.get(&key) {
        for item in items {
            if item.done {
                continue;
            }
            let Some(remind_text) = &item.remind_at else {
                continue;
            };
            let Some(remind_secs) = parse_hms(remind_text) else {
                continue;
            };
            let trigger_id = format!("{key}|{}|{remind_text}", item.text);
            if triggered.contains(&trigger_id) {
                continue;
            }
            let remain = remind_secs.saturating_sub(now_secs);
            next = next.min(remain.max(1) as u64);
        }
    }
    Duration::from_secs(next)
}

#[cfg(test)]
mod tests {
    use super::parse_hms;

    #[test]
    fn parse_hms_accepts_hhmm_legacy_format() {
        assert_eq!(parse_hms("09:05"), Some(9 * 3600 + 5 * 60));
        assert_eq!(parse_hms("00:00"), Some(0));
    }

    #[test]
    fn parse_hms_accepts_hhmmss_format() {
        assert_eq!(parse_hms("09:05:30"), Some(9 * 3600 + 5 * 60 + 30));
        assert_eq!(parse_hms("23:59:59"), Some(23 * 3600 + 59 * 60 + 59));
    }

    #[test]
    fn parse_hms_rejects_invalid_values() {
        assert_eq!(parse_hms("24:00:00"), None);
        assert_eq!(parse_hms("12:60:00"), None);
        assert_eq!(parse_hms("12:00:60"), None);
        assert_eq!(parse_hms("12:00:00:00"), None);
        assert_eq!(parse_hms("abc"), None);
        assert_eq!(parse_hms(""), None);
    }
}
