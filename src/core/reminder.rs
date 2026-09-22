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

fn pending_reminders<'a>(
    todos: &'a TodoStore,
    key: &str,
) -> impl Iterator<Item = (&'a str, &'a str, u32)> + 'a {
    todos
        .get(key)
        .into_iter()
        .flat_map(|items| items.iter())
        .filter(|item| !item.done)
        .filter_map(|item| {
            item.remind_at
                .as_deref()
                .and_then(parse_hms)
                .map(|secs| (item.text.as_str(), item.remind_at.as_deref().unwrap(), secs))
        })
}

pub(crate) fn due_reminders(
    todos: &TodoStore,
    triggered: &HashSet<String>,
    now: DateTime<Local>,
) -> Vec<DueReminder> {
    let now_secs = now.time().num_seconds_from_midnight();
    let key = date_key(now.date_naive());
    let mut due = Vec::new();
    for (text, remind_text, remind_secs) in pending_reminders(todos, &key) {
        if now_secs < remind_secs {
            continue;
        }
        let trigger_id = format!("{key}|{text}|{remind_text}");
        if triggered.contains(&trigger_id) {
            continue;
        }
        due.push(DueReminder {
            trigger_id,
            message: format!("{text}（{remind_text}）"),
        });
    }
    due
}

pub(crate) fn next_delay(todos: &TodoStore, now: DateTime<Local>) -> Duration {
    let now_secs = now.time().num_seconds_from_midnight();
    let key = date_key(now.date_naive());
    let mut next = 30u64;
    for (_, _, remind_secs) in pending_reminders(todos, &key) {
        if now_secs >= remind_secs {
            continue;
        }
        next = next.min((remind_secs - now_secs).max(1) as u64);
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
