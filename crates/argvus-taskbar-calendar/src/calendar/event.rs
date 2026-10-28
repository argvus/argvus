//! Calendar, event and reminder types stored in the database.

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
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

  /// The days this event occupies in `tz`. The end is exclusive, so an
  /// all-day event that ends at the next midnight covers a single day.
  pub fn local_days<Tz: TimeZone>(&self, tz: &Tz) -> Vec<NaiveDate> {
    let first = self.start.with_timezone(tz).date_naive();
    let last_instant = if self.end > self.start {
      self.end - Duration::nanoseconds(1)
    } else {
      self.start
    };
    let last = last_instant.with_timezone(tz).date_naive();
    first.iter_days().take_while(|day| *day <= last).collect()
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

#[cfg(test)]
mod tests {
  use chrono::{FixedOffset, TimeZone};

  use super::*;

  fn sao_paulo() -> FixedOffset {
    FixedOffset::west_opt(3 * 3600).unwrap()
  }

  fn day(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
  }

  #[test]
  fn all_day_event_covers_only_its_own_day() {
    let tz = sao_paulo();
    let start = tz.with_ymd_and_hms(2026, 10, 13, 0, 0, 0).unwrap();
    let mut event = CalendarEvent::new_local(1, "All day", start.with_timezone(&Utc));
    event.end = tz
      .with_ymd_and_hms(2026, 10, 14, 0, 0, 0)
      .unwrap()
      .with_timezone(&Utc);
    event.all_day = true;
    assert_eq!(event.local_days(&tz), vec![day(13)]);
  }

  #[test]
  fn late_and_overnight_events_use_local_days() {
    let tz = sao_paulo();
    let late = tz.with_ymd_and_hms(2026, 10, 13, 22, 0, 0).unwrap();
    let mut event = CalendarEvent::new_local(1, "Late", late.with_timezone(&Utc));
    event.end = event.start + Duration::minutes(30);
    assert_eq!(event.local_days(&tz), vec![day(13)]);

    event.end = event.start + Duration::hours(3);
    assert_eq!(event.local_days(&tz), vec![day(13), day(14)]);

    event.end = event.start;
    assert_eq!(event.local_days(&tz), vec![day(13)]);
  }
}
