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

pub(crate) fn date_key(date: NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}
