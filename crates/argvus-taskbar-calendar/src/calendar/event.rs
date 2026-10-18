use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Recurrence;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CalendarSource {
  Local,
  CalDav,
  ImportedIcs,
}

impl CalendarSource {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Local => "local",
      Self::CalDav => "caldav",
      Self::ImportedIcs => "ics",
    }
  }
}

impl From<&str> for CalendarSource {
  fn from(value: &str) -> Self {
    match value {
      "caldav" => Self::CalDav,
      "ics" => Self::ImportedIcs,
      _ => Self::Local,
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Calendar {
  pub id: i64,
  pub name: String,
  pub source: CalendarSource,
  pub remote_url: Option<String>,
  pub color: Option<String>,
  pub enabled: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Reminder {
  pub id: Option<i64>,
  pub event_id: Option<i64>,
  pub minutes_before: i64,
  pub fired_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CalendarEvent {
  pub id: Option<i64>,
  pub uid: String,
  pub calendar_id: i64,
  pub title: String,
  pub description: Option<String>,
  pub location: Option<String>,
  pub start: DateTime<Utc>,
  pub end: DateTime<Utc>,
  pub all_day: bool,
  pub timezone: Option<String>,
  pub recurrence: Recurrence,
  pub reminders: Vec<Reminder>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub source: CalendarSource,
  pub external_etag: Option<String>,
  pub remote_url: Option<String>,
  pub dirty: bool,
  pub deleted: bool,
}

impl CalendarEvent {
  pub fn new_local(calendar_id: i64, title: impl Into<String>, start: DateTime<Utc>) -> Self {
    let now = Utc::now();
    Self {
      id: None,
      uid: format!("{}@argvus-calendar", Uuid::new_v4()),
      calendar_id,
      title: title.into(),
      description: None,
      location: None,
      start,
      end: start + Duration::hours(1),
      all_day: false,
      timezone: Some("UTC".to_string()),
      recurrence: Recurrence::default(),
      reminders: vec![Reminder {
        id: None,
        event_id: None,
        minutes_before: 10,
        fired_at: None,
      }],
      created_at: now,
      updated_at: now,
      source: CalendarSource::Local,
      external_etag: None,
      remote_url: None,
      dirty: true,
      deleted: false,
    }
  }

  pub fn validate(&self) -> crate::error::Result<()> {
    if self.title.trim().is_empty() {
      return Err(crate::error::ArgvusError::Configuration(
        "event title cannot be empty".to_string(),
      ));
    }
    if self.end < self.start {
      return Err(crate::error::ArgvusError::Configuration(
        "event end must be equal to or after start".to_string(),
      ));
    }
    Ok(())
  }
}
