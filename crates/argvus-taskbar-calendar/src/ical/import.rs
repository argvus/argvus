//! iCalendar (`.ics`) import.

use std::path::Path;

use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, TimeZone, Utc};
use icalendar::{Calendar as ICalendar, CalendarDateTime, Component, DatePerhapsTime, EventLike};

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
    let start_value = event
      .get_start()
      .ok_or_else(|| ArgvusError::Ics(format!("unsupported datetime: {start_raw}")))?;
    let all_day = matches!(start_value, DatePerhapsTime::Date(_));
    let start = to_utc(&start_value)?;
    // RFC 5545: without DTEND an all-day event lasts one day.
    let end = match event.get_end() {
      Some(end) => to_utc(&end)?,
      None if all_day => start + Duration::days(1),
      None => start + Duration::hours(1),
    };
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

/// Converts a `DTSTART`/`DTEND` value to UTC. A date (all-day) starts at the
/// local midnight, like the events created in the popup; a floating time is a
/// local time (RFC 5545 form #1); a time with `TZID` uses that zone, or the
/// local zone when the identifier is unknown.
fn to_utc(value: &DatePerhapsTime) -> Result<DateTime<Utc>> {
  let local = |naive: &NaiveDateTime| {
    Local
      .from_local_datetime(naive)
      .earliest()
      .map(|date_time| date_time.with_timezone(&Utc))
      .ok_or_else(|| ArgvusError::Ics(format!("nonexistent local time: {naive}")))
  };
  match value {
    DatePerhapsTime::Date(date) => local(&date.and_time(NaiveTime::MIN)),
    DatePerhapsTime::DateTime(CalendarDateTime::Utc(date_time)) => Ok(*date_time),
    DatePerhapsTime::DateTime(CalendarDateTime::Floating(naive)) => local(naive),
    DatePerhapsTime::DateTime(zoned @ CalendarDateTime::WithTimezone { date_time, .. }) => {
      zoned.try_into_utc().map_or_else(|| local(date_time), Ok)
    }
  }
}

/// Minutes before the start of a `TRIGGER` relative to it, such as `-PT15M`,
/// `-PT1H30M`, `-PT900S`, `-P1D` or `PT0S`. A positive offset (after the
/// start) becomes a negative value; unsupported forms are skipped.
fn trigger_to_minutes(trigger: &str) -> Option<i64> {
  let value = trigger.trim();
  let (sign, duration) = match value.strip_prefix('-') {
    Some(rest) => (1, rest),
    None => (-1, value.strip_prefix('+').unwrap_or(value)),
  };
  let duration = duration.strip_prefix('P')?;
  let (date_part, time_part) = duration.split_once('T').unwrap_or((duration, ""));
  let mut seconds = 0_i64;
  for (part, units) in [
    (date_part, &[('W', 7 * 86_400), ('D', 86_400)][..]),
    (time_part, &[('H', 3_600), ('M', 60), ('S', 1)][..]),
  ] {
    let mut rest = part;
    for (unit, seconds_per_unit) in units {
      if let Some((number, tail)) = rest.split_once(*unit) {
        seconds += number.parse::<i64>().ok()? * seconds_per_unit;
        rest = tail;
      }
    }
    if !rest.is_empty() {
      return None;
    }
  }
  Some(sign * seconds / 60)
}

#[cfg(test)]
mod tests {
  use chrono::{NaiveDate, Timelike};

  use super::*;

  fn single_event(body: &str) -> CalendarEvent {
    let contents = format!(
      "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:test\r\nBEGIN:VEVENT\r\nUID:test@argvus\r\nSUMMARY:Test\r\n{body}END:VEVENT\r\nEND:VCALENDAR\r\n"
    );
    parse_ics(&contents, 1).unwrap().remove(0)
  }

  #[test]
  fn all_day_dates_start_at_the_local_midnight() {
    let event = single_event("DTSTART;VALUE=DATE:20261013\r\n");
    assert!(event.all_day);
    let start = event.start.with_timezone(&Local);
    assert_eq!(
      start.date_naive(),
      NaiveDate::from_ymd_opt(2026, 10, 13).unwrap()
    );
    assert_eq!((start.hour(), start.minute()), (0, 0));
    assert_eq!(event.end - event.start, Duration::days(1));
  }

  #[test]
  fn utc_floating_and_zoned_times_are_converted() {
    let event = single_event("DTSTART:20261013T120000Z\r\nDTEND:20261013T130000Z\r\n");
    assert_eq!(event.start.to_rfc3339(), "2026-10-13T12:00:00+00:00");
    assert!(!event.all_day);

    let event = single_event("DTSTART;TZID=America/Sao_Paulo:20261013T090000\r\n");
    assert_eq!(event.start.to_rfc3339(), "2026-10-13T12:00:00+00:00");
    assert_eq!(event.end - event.start, Duration::hours(1));

    let event = single_event("DTSTART:20261013T090000\r\n");
    let start = event.start.with_timezone(&Local);
    assert_eq!((start.hour(), start.minute()), (9, 0));
  }

  #[test]
  fn alarm_triggers_become_minutes_before() {
    assert_eq!(trigger_to_minutes("-PT15M"), Some(15));
    assert_eq!(trigger_to_minutes("-PT1H"), Some(60));
    assert_eq!(trigger_to_minutes("-PT1H30M"), Some(90));
    assert_eq!(trigger_to_minutes("-P1D"), Some(1440));
    assert_eq!(trigger_to_minutes("-P1W"), Some(10080));
    assert_eq!(trigger_to_minutes("PT0S"), Some(0));
    assert_eq!(trigger_to_minutes("-PT3600S"), Some(60));
    assert_eq!(trigger_to_minutes("PT30M"), Some(-30));
    assert_eq!(trigger_to_minutes("19980101T050000Z"), None);
    assert_eq!(trigger_to_minutes("-PTxM"), None);
  }
}
