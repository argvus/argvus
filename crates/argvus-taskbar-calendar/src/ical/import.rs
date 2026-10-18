use std::path::Path;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use icalendar::{Calendar as ICalendar, Component, EventLike};

use crate::calendar::{CalendarEvent, CalendarSource, Recurrence, Reminder};
use crate::error::{ArgvusError, Result};

pub fn import_ics_file(path: &Path, calendar_id: i64) -> Result<Vec<CalendarEvent>> {
  let contents = std::fs::read_to_string(path)?;
  parse_ics(&contents, calendar_id)
}

pub fn parse_ics(contents: &str, calendar_id: i64) -> Result<Vec<CalendarEvent>> {
  let calendar: ICalendar = contents
    .parse()
    .map_err(|err| ArgvusError::Ics(format!("failed to parse ICS: {err}")))?;
  let now = Utc::now();
  let mut events = Vec::new();
  for event in calendar.events() {
    let Some(start_raw) = event.property_value("DTSTART") else {
      continue;
    };
    let start = parse_ics_datetime(start_raw)?;
    let end = event
      .property_value("DTEND")
      .map(parse_ics_datetime)
      .transpose()?
      .unwrap_or_else(|| start + Duration::hours(1));
    let all_day = event
      .properties()
      .get("DTSTART")
      .and_then(|property| property.params().get("VALUE"))
      .map(|param| param.value() == "DATE")
      .unwrap_or_else(|| start_raw.len() == 8);
    let uid = event
      .get_uid()
      .map(ToOwned::to_owned)
      .unwrap_or_else(|| format!("{}@import.argvus-calendar", uuid::Uuid::new_v4()));
    let recurrence = Recurrence {
      rrule: event.property_value("RRULE").map(ToOwned::to_owned),
      raw_properties: event
        .multi_properties()
        .iter()
        .filter(|(key, _)| key.starts_with("RDATE") || key.starts_with("EXDATE"))
        .flat_map(|(_, props)| {
          props
            .iter()
            .map(|prop| format!("{}:{}", prop.key(), prop.value()))
        })
        .collect(),
    };
    let reminders = event
      .components()
      .iter()
      .filter(|component| component.component_kind() == "VALARM")
      .filter_map(|component| component.property_value("TRIGGER"))
      .filter_map(trigger_to_minutes)
      .map(|minutes_before| Reminder {
        id: None,
        event_id: None,
        minutes_before,
        fired_at: None,
      })
      .collect::<Vec<_>>();
    events.push(CalendarEvent {
      id: None,
      uid,
      calendar_id,
      title: event.get_summary().unwrap_or("Untitled event").to_string(),
      description: event.get_description().map(ToOwned::to_owned),
      location: event.get_location().map(ToOwned::to_owned),
      start,
      end,
      all_day,
      timezone: event
        .properties()
        .get("DTSTART")
        .and_then(|property| property.params().get("TZID"))
        .map(|param| param.value().to_string()),
      recurrence,
      reminders,
      created_at: now,
      updated_at: now,
      source: CalendarSource::ImportedIcs,
      external_etag: None,
      remote_url: None,
      dirty: false,
      deleted: false,
    });
  }
  Ok(events)
}

fn parse_ics_datetime(value: &str) -> Result<DateTime<Utc>> {
  if value.len() == 8 {
    let date = NaiveDate::parse_from_str(value, "%Y%m%d")
      .map_err(|err| ArgvusError::Ics(format!("invalid date {value}: {err}")))?;
    return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
  }
  if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
    return Ok(dt.with_timezone(&Utc));
  }
  if let Ok(dt) = NaiveDateTime::parse_from_str(value.trim_end_matches('Z'), "%Y%m%dT%H%M%S") {
    return Ok(dt.and_utc());
  }
  Err(ArgvusError::Ics(format!("unsupported datetime: {value}")))
}

fn trigger_to_minutes(trigger: &str) -> Option<i64> {
  let value = trigger.trim();
  if !value.starts_with("-PT") || !value.ends_with('M') {
    return None;
  }
  value
    .trim_start_matches("-PT")
    .trim_end_matches('M')
    .parse::<i64>()
    .ok()
}

#[allow(dead_code)]
fn midnight() -> NaiveTime {
  NaiveTime::from_hms_opt(0, 0, 0).unwrap()
}
