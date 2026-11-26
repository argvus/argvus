//! Command-line actions that do not open the popup: iCalendar import and export, one sync pass and the status report.

use std::path::PathBuf;

use crate::config::{Paths, Settings, service_events_enabled};
use crate::error::Result;
use crate::ical::{export_ics_file, import_ics_file};
use crate::storage::Database;

pub fn import_file(paths: &Paths, file: PathBuf) -> Result<()> {
  let count = import_events(paths, file)?;
  println!("Imported {count} events into the Local calendar");
  Ok(())
}

pub fn import_events(paths: &Paths, file: PathBuf) -> Result<usize> {
  let mut db = Database::open(&paths.database)?;
  let calendar_id = db.default_calendar_id()?;
  let mut events = import_ics_file(&file, calendar_id)?;
  let count = events.len();
  for event in &mut events {
    db.upsert_event(event)?;
  }
  Ok(count)
}

pub fn export_file(paths: &Paths, output: PathBuf) -> Result<()> {
  let count = export_events(paths, &output)?;
  println!("Exported {count} events to {}", output.display());
  Ok(())
}

pub fn export_events(paths: &Paths, output: &std::path::Path) -> Result<usize> {
  let db = Database::open(&paths.database)?;
  let events = db.all_events()?;
  export_ics_file(output, &events)?;
  Ok(events.len())
}

pub fn sync_once(paths: &Paths, settings: &Settings) -> Result<()> {
  println!("{}", sync_report(paths, settings)?);
  Ok(())
}

pub fn sync_report(paths: &Paths, settings: &Settings) -> Result<String> {
  let db = Database::open(&paths.database)?;
  let calendars = db.calendars()?;
  let remote = calendars
    .iter()
    .filter(|calendar| calendar.enabled && calendar.remote_url.is_some())
    .count();
  Ok(format!(
    "Sync scheduler configured for every {} minutes; {remote} remote calendars configured",
    settings.calendar.sync_interval_minutes
  ))
}

pub fn status(paths: &Paths) -> Result<()> {
  println!("{}", status_report(paths)?);
  Ok(())
}

pub fn status_report(paths: &Paths) -> Result<String> {
  let db = Database::open(&paths.database)?;
  let events = if service_events_enabled(paths) {
    "enabled"
  } else {
    "disabled"
  };
  Ok(
    [
      format!("Database: OK ({})", paths.database.display()),
      format!("Config: {}", paths.config_file.display()),
      format!("Style: {}", paths.user_style.display()),
      format!("Theme: {}", paths.theme_file.display()),
      format!("Theme cache: {}", paths.cache_theme_file.display()),
      format!(
        "Active ARGVUS theme: {}",
        paths.active_argvus_theme_file.display()
      ),
      format!(
        "Events/reminders: {events} ({})",
        paths.events_enabled_file.display()
      ),
      format!("Data: {}", paths.data_dir.display()),
      format!("State: {}", paths.state_dir.display()),
      format!("Cache: {}", paths.cache_dir.display()),
      format!("Calendars: {}", db.calendars()?.len()),
      format!("Events: {}", db.all_events()?.len()),
      "Service: use `argvus taskbar-calendar service` or install the user systemd unit".into(),
    ]
    .join("\n"),
  )
}
