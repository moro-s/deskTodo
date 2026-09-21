use crate::models::{date_key, TodoStore};
use chrono::{DateTime, Local, Timelike};
use std::collections::HashSet;
use std::time::Duration;

pub(crate) struct DueReminder {
    pub(crate) trigger_id: String,
    pub(crate) message: String,
}

pub(crate) fn parse_hhmm(value: &str) -> Option<u32> {
    let (hour, minute) = value.split_once(':')?;
    let hour: u32 = hour.parse().ok()?;
    let minute: u32 = minute.parse().ok()?;
    if hour < 24 && minute < 60 {
        Some(hour * 3600 + minute * 60)
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
            let Some(remind_secs) = parse_hhmm(remind_text) else {
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
            let Some(remind_secs) = parse_hhmm(remind_text) else {
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
