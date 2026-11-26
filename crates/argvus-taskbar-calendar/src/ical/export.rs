//! iCalendar (`.ics`) export.

use std::path::Path;

use chrono::{DateTime, Local, Utc};
use icalendar::{
  Alarm, Calendar as ICalendar, CalendarDateTime, Component, Event, EventLike, Property,
};

use crate::calendar::CalendarEvent;
use crate::config::validate_export_path;
use crate::error::Result;

pub fn export_ics_file(path: &Path, events: &[CalendarEvent]) -> Result<()> {
  validate_export_path(path)?;
  std::fs::write(path, export_ics(events)?)?;
  Ok(())
}

pub fn export_ics(events: &[CalendarEvent]) -> Result<String> {
  let mut calendar = ICalendar::new();
  calendar.name("Argvus Calendar");
  for source in events {
    let mut event = Event::new();
    event.uid(&source.uid).summary(&source.title);
    if source.all_day {
      // All-day events are stored from local midnight to local midnight and
      // go out as DATE values, which the import reads back the same way.
      event
        .starts(source.start.with_timezone(&Local).date_naive())
        .ends(source.end.with_timezone(&Local).date_naive());
    } else {
      event
        .starts(export_time(source.start, source.timezone.as_deref()))
        .ends(export_time(source.end, source.timezone.as_deref()));
    }
    if let Some(description) = &source.description {
      event.description(description);
    }
    if let Some(location) = &source.location {
      event.location(location);
    }
    if let Some(rrule) = &source.recurrence.rrule {
      event.append_property(Property::new("RRULE", rrule));
    }
    for raw in &source.recurrence.raw_properties {
      if let Some((key, value)) = raw.split_once(':') {
        event.append_property(Property::new(key, value));
      }
    }
    for reminder in &source.reminders {
      let duration = chrono::Duration::minutes(-reminder.minutes_before);
      event.append_component(Alarm::display(&source.title, duration));
    }
    calendar.push(event);
  }
  Ok(calendar.done().to_string())
}

fn export_time(instant: DateTime<Utc>, timezone: Option<&str>) -> CalendarDateTime {
  if let Some(tzid) = timezone
    && let Ok(zone) = tzid.parse::<chrono_tz::Tz>()
  {
    let zoned = CalendarDateTime::WithTimezone {
      date_time: instant.with_timezone(&zone).naive_local(),
      tzid: tzid.to_owned(),
    };
    // Use UTC when a repeated wall time would lose which occurrence was
    // recorded. The exported event must retain its original instant.
    if zoned.try_into_utc() == Some(instant) {
      return zoned;
    }
  }
  CalendarDateTime::Utc(instant)
}

#[cfg(test)]
mod tests {
  use chrono::{Duration, Local, NaiveDate, TimeZone, Utc};

  use super::*;
  use crate::calendar::{CalendarSource, Reminder};
  use crate::ical::import::parse_ics;

  #[test]
  fn exported_events_import_back_unchanged() {
    let local_midnight = Local
      .from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 10, 13)
          .unwrap()
          .and_hms_opt(0, 0, 0)
          .unwrap(),
      )
      .earliest()
      .unwrap()
      .with_timezone(&Utc);
    let mut all_day = CalendarEvent::new_local(1, "All day", local_midnight);
    all_day.end = local_midnight + Duration::days(1);
    all_day.all_day = true;
    all_day.reminders = vec![Reminder {
      id: None,
      event_id: None,
      minutes_before: -360,
      fired_at: None,
    }];
    let timed = CalendarEvent::new_local(1, "Timed", local_midnight + Duration::hours(9));

    let imported = parse_ics(&export_ics(&[all_day.clone(), timed.clone()]).unwrap(), 1).unwrap();
    for (original, imported) in [all_day, timed].iter().zip(&imported) {
      assert_eq!(imported.title, original.title);
      assert_eq!(imported.start, original.start);
      assert_eq!(imported.end, original.end);
      assert_eq!(imported.all_day, original.all_day);
      assert_eq!(imported.source, CalendarSource::ImportedIcs);
      assert_eq!(
        imported
          .reminders
          .iter()
          .map(|reminder| reminder.minutes_before)
          .collect::<Vec<_>>(),
        original
          .reminders
          .iter()
          .map(|reminder| reminder.minutes_before)
          .collect::<Vec<_>>()
      );
    }
  }
  #[test]
  fn timed_events_round_trip_tzid_recurrence_and_alarms() {
    let fixture = concat!(
      "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\n",
      "UID:zoned@example.invalid\r\nSUMMARY:Coffee\r\n",
      "DTSTART;TZID=Europe/Berlin:20261013T093000\r\n",
      "DTEND;TZID=Europe/Berlin:20261013T103000\r\n",
      "RRULE:FREQ=WEEKLY;COUNT=2\r\n",
      "BEGIN:VALARM\r\nACTION:DISPLAY\r\nDESCRIPTION:Coffee\r\nTRIGGER:-PT10M\r\nEND:VALARM\r\n",
      "END:VEVENT\r\nEND:VCALENDAR\r\n",
    );
    let original = parse_ics(fixture, 1).unwrap();
    let output = export_ics(&original).unwrap();
    assert!(output.contains("DTSTART;TZID=Europe/Berlin:20261013T093000"));
    let reimported = parse_ics(&output, 1).unwrap();
    assert_eq!(reimported[0].timezone.as_deref(), Some("Europe/Berlin"));
    assert_eq!(reimported[0].start, original[0].start);
    assert_eq!(reimported[0].end, original[0].end);
    assert_eq!(reimported[0].recurrence, original[0].recurrence);
    assert_eq!(reimported[0].reminders, original[0].reminders);
  }
}
