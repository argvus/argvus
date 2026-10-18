use std::path::Path;

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
    event
      .uid(&source.uid)
      .summary(&source.title)
      .starts(source.start)
      .ends(source.end);
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
