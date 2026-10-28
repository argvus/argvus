//! iCalendar (`.ics`) export.

use std::path::Path;

use chrono::Local;
use icalendar::{Alarm, Calendar as ICalendar, Component, Event, EventLike, Property};

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
      event.starts(source.start).ends(source.end);
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
}
