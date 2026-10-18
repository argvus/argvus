use chrono::{DateTime, Days, Duration, Local, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use gtk::glib::SignalHandlerId;
use gtk::prelude::*;

use crate::calendar::{CalendarEvent, Reminder, today};
use crate::config::Settings;
use crate::i18n::{I18n, Text};
use crate::storage::Database;

#[derive(Debug, Clone)]
pub enum EditorKind {
  New,
  Edit(Box<CalendarEvent>),
}

#[derive(Debug, Clone)]
pub struct EditorDraft {
  pub kind: EditorKind,
  pub date: NaiveDate,
  pub title: String,
  pub location: String,
  pub description: String,
  pub start_hour: u32,
  pub start_minute: u32,
  pub end_hour: u32,
  pub end_minute: u32,
  pub all_day: bool,
  pub reminder_enabled: bool,
  pub reminder_hours: u32,
  pub reminder_minutes: u32,
  pub repeat_all_day: bool,
  pub repeat_hours: u32,
  pub repeat_minutes: u32,
}

pub struct EditorWidgets {
  pub header: gtk::Label,
  pub title_label: gtk::Label,
  pub title: gtk::Entry,
  pub date_label: gtk::Label,
  pub date_value: gtk::Label,
  pub start_time_label: gtk::Label,
  pub end_time_label: gtk::Label,
  pub start_hour: gtk::SpinButton,
  pub start_minute: gtk::SpinButton,
  pub end_hour: gtk::SpinButton,
  pub end_minute: gtk::SpinButton,
  pub all_day: gtk::CheckButton,
  pub location_label: gtk::Label,
  pub location: gtk::Entry,
  pub description_label: gtk::Label,
  pub description: gtk::Entry,
  pub reminder_enabled: gtk::CheckButton,
  pub reminder_hour_label: gtk::Label,
  pub reminder_hours: gtk::SpinButton,
  pub reminder_minute_label: gtk::Label,
  pub reminder_minutes: gtk::SpinButton,
  pub reminder_before: gtk::Label,
  pub repeat_row: gtk::Box,
  pub repeat_all_day: gtk::CheckButton,
  pub repeat_hour_label: gtk::Label,
  pub repeat_hours: gtk::SpinButton,
  pub repeat_minute_label: gtk::Label,
  pub repeat_minutes: gtk::SpinButton,
  pub delete: gtk::Button,
  pub cancel: gtk::Button,
  pub save: gtk::Button,
  pub changed_handlers: EditorChangedHandlers,
}

pub struct EditorChangedHandlers {
  pub title: SignalHandlerId,
  pub location: SignalHandlerId,
  pub description: SignalHandlerId,
  pub start_hour: SignalHandlerId,
  pub start_minute: SignalHandlerId,
  pub end_hour: SignalHandlerId,
  pub end_minute: SignalHandlerId,
  pub all_day: SignalHandlerId,
  pub reminder_enabled: SignalHandlerId,
  pub reminder_hours: SignalHandlerId,
  pub reminder_minutes: SignalHandlerId,
  pub repeat_all_day: SignalHandlerId,
  pub repeat_hours: SignalHandlerId,
  pub repeat_minutes: SignalHandlerId,
}

impl EditorDraft {
  pub fn new(date: NaiveDate, default_reminder_minutes: i64) -> Self {
    let reminder = default_reminder_minutes.max(0) as u32;
    Self {
      kind: EditorKind::New,
      date,
      title: String::new(),
      location: String::new(),
      description: String::new(),
      start_hour: 9,
      start_minute: 0,
      end_hour: 10,
      end_minute: 0,
      all_day: false,
      reminder_enabled: true,
      reminder_hours: reminder / 60,
      reminder_minutes: reminder % 60,
      repeat_all_day: false,
      repeat_hours: 2,
      repeat_minutes: 0,
    }
  }

  pub fn from_event(event: CalendarEvent) -> Self {
    let start = event.start.with_timezone(&Local);
    let end = event.end.with_timezone(&Local);
    let reminder = event
      .reminders
      .iter()
      .find(|reminder| reminder.minutes_before >= 0)
      .map(|reminder| reminder.minutes_before as u32);
    let repeat = event
      .reminders
      .iter()
      .filter(|reminder| reminder.minutes_before < 0)
      .map(|reminder| reminder.minutes_before.unsigned_abs() as u32)
      .min();
    Self {
      kind: EditorKind::Edit(Box::new(event.clone())),
      date: start.date_naive(),
      title: event.title,
      location: event.location.unwrap_or_default(),
      description: event.description.unwrap_or_default(),
      start_hour: start.hour(),
      start_minute: start.minute(),
      end_hour: end.hour(),
      end_minute: end.minute(),
      all_day: event.all_day,
      reminder_enabled: reminder.is_some(),
      reminder_hours: reminder.unwrap_or_default() / 60,
      reminder_minutes: reminder.unwrap_or_default() % 60,
      repeat_all_day: repeat.is_some(),
      repeat_hours: repeat.unwrap_or(120) / 60,
      repeat_minutes: repeat.unwrap_or(120) % 60,
    }
  }
}

pub fn build_editor() -> gtk::Box {
  gtk::Box::new(gtk::Orientation::Vertical, 10)
}

pub fn populate_editor(widgets: &EditorWidgets, draft: &EditorDraft, i18n: I18n) {
  let handlers = &widgets.changed_handlers;
  for (widget, id) in [
    (widgets.title.upcast_ref::<gtk::Widget>(), &handlers.title),
    (
      widgets.location.upcast_ref::<gtk::Widget>(),
      &handlers.location,
    ),
    (
      widgets.description.upcast_ref::<gtk::Widget>(),
      &handlers.description,
    ),
    (
      widgets.start_hour.upcast_ref::<gtk::Widget>(),
      &handlers.start_hour,
    ),
    (
      widgets.start_minute.upcast_ref::<gtk::Widget>(),
      &handlers.start_minute,
    ),
    (
      widgets.end_hour.upcast_ref::<gtk::Widget>(),
      &handlers.end_hour,
    ),
    (
      widgets.end_minute.upcast_ref::<gtk::Widget>(),
      &handlers.end_minute,
    ),
    (
      widgets.all_day.upcast_ref::<gtk::Widget>(),
      &handlers.all_day,
    ),
    (
      widgets.reminder_enabled.upcast_ref::<gtk::Widget>(),
      &handlers.reminder_enabled,
    ),
    (
      widgets.reminder_hours.upcast_ref::<gtk::Widget>(),
      &handlers.reminder_hours,
    ),
    (
      widgets.reminder_minutes.upcast_ref::<gtk::Widget>(),
      &handlers.reminder_minutes,
    ),
    (
      widgets.repeat_all_day.upcast_ref::<gtk::Widget>(),
      &handlers.repeat_all_day,
    ),
    (
      widgets.repeat_hours.upcast_ref::<gtk::Widget>(),
      &handlers.repeat_hours,
    ),
    (
      widgets.repeat_minutes.upcast_ref::<gtk::Widget>(),
      &handlers.repeat_minutes,
    ),
  ] {
    widget.block_signal(id);
  }

  widgets.header.set_label(match draft.kind {
    EditorKind::New => i18n.text(Text::NewEvent),
    EditorKind::Edit(_) => i18n.text(Text::EditEvent),
  });
  widgets.title_label.set_label(i18n.text(Text::Title));
  set_entry_text(&widgets.title, &draft.title);
  widgets.date_label.set_label(i18n.text(Text::Date));
  widgets
    .date_value
    .set_label(&draft.date.format("%Y-%m-%d").to_string());
  widgets
    .start_time_label
    .set_label(i18n.text(Text::StartTime));
  widgets.end_time_label.set_label(i18n.text(Text::EndTime));
  set_number(&widgets.start_hour, draft.start_hour);
  set_number(&widgets.start_minute, draft.start_minute);
  set_number(&widgets.end_hour, draft.end_hour);
  set_number(&widgets.end_minute, draft.end_minute);
  for entry in [
    &widgets.start_hour,
    &widgets.start_minute,
    &widgets.end_hour,
    &widgets.end_minute,
  ] {
    entry.set_sensitive(!draft.all_day);
  }
  widgets.all_day.set_label(Some(i18n.text(Text::AllDay)));
  widgets.all_day.set_active(draft.all_day);
  widgets.location_label.set_label(i18n.text(Text::Location));
  set_entry_text(&widgets.location, &draft.location);
  widgets
    .description_label
    .set_label(i18n.text(Text::Description));
  set_entry_text(&widgets.description, &draft.description);
  widgets
    .reminder_enabled
    .set_label(Some(i18n.text(Text::Reminder)));
  widgets.reminder_enabled.set_active(draft.reminder_enabled);
  widgets
    .reminder_hour_label
    .set_label(i18n.text(Text::HourShort));
  set_number(&widgets.reminder_hours, draft.reminder_hours);
  widgets
    .reminder_minute_label
    .set_label(i18n.text(Text::MinuteShort));
  set_number(&widgets.reminder_minutes, draft.reminder_minutes);
  widgets.reminder_before.set_label(i18n.text(Text::Before));
  for widget in [
    widgets.reminder_hours.upcast_ref::<gtk::Widget>(),
    widgets.reminder_minutes.upcast_ref::<gtk::Widget>(),
  ] {
    widget.set_sensitive(draft.reminder_enabled);
  }
  widgets.repeat_row.set_visible(draft.all_day);
  widgets
    .repeat_all_day
    .set_label(Some(i18n.text(Text::Repeat)));
  widgets.repeat_all_day.set_active(draft.repeat_all_day);
  widgets
    .repeat_hour_label
    .set_label(i18n.text(Text::HourShort));
  set_number(&widgets.repeat_hours, draft.repeat_hours);
  widgets
    .repeat_minute_label
    .set_label(i18n.text(Text::MinuteShort));
  set_number(&widgets.repeat_minutes, draft.repeat_minutes);
  for widget in [
    widgets.repeat_hours.upcast_ref::<gtk::Widget>(),
    widgets.repeat_minutes.upcast_ref::<gtk::Widget>(),
  ] {
    widget.set_sensitive(draft.repeat_all_day);
  }
  widgets.delete.set_label(i18n.text(Text::Delete));
  widgets
    .delete
    .set_visible(matches!(draft.kind, EditorKind::Edit(_)));
  widgets.cancel.set_label(i18n.text(Text::Cancel));
  widgets.save.set_label(i18n.text(Text::Save));

  for (widget, id) in [
    (widgets.title.upcast_ref::<gtk::Widget>(), &handlers.title),
    (
      widgets.location.upcast_ref::<gtk::Widget>(),
      &handlers.location,
    ),
    (
      widgets.description.upcast_ref::<gtk::Widget>(),
      &handlers.description,
    ),
    (
      widgets.start_hour.upcast_ref::<gtk::Widget>(),
      &handlers.start_hour,
    ),
    (
      widgets.start_minute.upcast_ref::<gtk::Widget>(),
      &handlers.start_minute,
    ),
    (
      widgets.end_hour.upcast_ref::<gtk::Widget>(),
      &handlers.end_hour,
    ),
    (
      widgets.end_minute.upcast_ref::<gtk::Widget>(),
      &handlers.end_minute,
    ),
    (
      widgets.all_day.upcast_ref::<gtk::Widget>(),
      &handlers.all_day,
    ),
    (
      widgets.reminder_enabled.upcast_ref::<gtk::Widget>(),
      &handlers.reminder_enabled,
    ),
    (
      widgets.reminder_hours.upcast_ref::<gtk::Widget>(),
      &handlers.reminder_hours,
    ),
    (
      widgets.reminder_minutes.upcast_ref::<gtk::Widget>(),
      &handlers.reminder_minutes,
    ),
    (
      widgets.repeat_all_day.upcast_ref::<gtk::Widget>(),
      &handlers.repeat_all_day,
    ),
    (
      widgets.repeat_hours.upcast_ref::<gtk::Widget>(),
      &handlers.repeat_hours,
    ),
    (
      widgets.repeat_minutes.upcast_ref::<gtk::Widget>(),
      &handlers.repeat_minutes,
    ),
  ] {
    widget.unblock_signal(id);
  }
}

pub fn save_editor(
  db_path: &std::path::Path,
  settings: &Settings,
  draft: EditorDraft,
) -> crate::error::Result<()> {
  if draft.date < today() {
    return Err(crate::error::ArgvusError::Configuration(
      "cannot create or update an event in the past".to_string(),
    ));
  }
  let mut db = Database::open(db_path)?;
  let calendar_id = db.default_calendar_id()?;
  let (start, end) = if draft.all_day {
    let start = local_datetime(draft.date, 0, 0).expect("valid all-day start");
    let next_day = draft
      .date
      .checked_add_days(Days::new(1))
      .expect("valid next day");
    let end = local_datetime(next_day, 0, 0).expect("valid all-day end");
    (start, end)
  } else {
    let start = local_datetime(draft.date, draft.start_hour, draft.start_minute)
      .unwrap_or_else(|| local_datetime(draft.date, 9, 0).expect("valid default"));
    let end = local_datetime(draft.date, draft.end_hour, draft.end_minute).unwrap_or_else(|| {
      start + Duration::minutes(settings.calendar.default_event_duration_minutes)
    });
    (start, end.max(start))
  };
  let reminders = build_reminders(&draft, None, start, end, Utc::now());
  let mut event = match draft.kind {
    EditorKind::New => CalendarEvent::new_local(
      calendar_id,
      if draft.title.is_empty() {
        "New event"
      } else {
        &draft.title
      },
      start,
    ),
    EditorKind::Edit(event) => {
      let mut event = *event;
      event.title = if draft.title.is_empty() {
        event.title
      } else {
        draft.title.clone()
      };
      event
    }
  };
  event.location = empty_to_none(draft.location);
  event.description = empty_to_none(draft.description);
  event.start = start;
  event.end = end;
  event.all_day = draft.all_day;
  event.dirty = true;
  event.reminders = reminders;
  db.upsert_event(&mut event)?;
  Ok(())
}

fn build_reminders(
  draft: &EditorDraft,
  event_id: Option<i64>,
  start: DateTime<Utc>,
  end: DateTime<Utc>,
  now: DateTime<Utc>,
) -> Vec<Reminder> {
  let mut reminders = Vec::new();
  if draft.reminder_enabled {
    let configured_minutes_before =
      i64::from(draft.reminder_hours) * 60 + i64::from(draft.reminder_minutes);
    let minutes_before = if configured_minutes_before > 0
      && start - Duration::minutes(configured_minutes_before) <= now
    {
      0
    } else {
      configured_minutes_before
    };
    reminders.push(Reminder {
      id: None,
      event_id,
      minutes_before,
      fired_at: None,
    });
  }

  let interval = i64::from(draft.repeat_hours) * 60 + i64::from(draft.repeat_minutes);
  if draft.all_day && draft.repeat_all_day && interval > 0 {
    let duration = (end - start).num_minutes();
    let mut elapsed = interval;
    while elapsed < duration {
      reminders.push(Reminder {
        id: None,
        event_id,
        minutes_before: -elapsed,
        fired_at: None,
      });
      elapsed += interval;
    }
  }
  reminders
}

fn local_datetime(date: NaiveDate, hour: u32, minute: u32) -> Option<DateTime<Utc>> {
  let time = NaiveTime::from_hms_opt(hour, minute, 0)?;
  Local
    .from_local_datetime(&date.and_time(time))
    .single()
    .map(|dt| dt.with_timezone(&Utc))
}

fn empty_to_none(value: String) -> Option<String> {
  if value.is_empty() { None } else { Some(value) }
}

fn set_entry_text(entry: &gtk::Entry, value: &str) {
  if entry.text().as_str() != value {
    entry.set_text(value);
    entry.set_position(-1);
  }
}

fn set_number(spin: &gtk::SpinButton, value: u32) {
  if !spin.has_focus() {
    spin.set_value(f64::from(value));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn local_datetime_accepts_valid_time() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    let datetime = local_datetime(date, 23, 59).unwrap();
    let local = datetime.with_timezone(&Local);
    assert_eq!((local.hour(), local.minute()), (23, 59));
  }

  #[test]
  fn local_datetime_rejects_out_of_range_time() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    assert!(local_datetime(date, 24, 0).is_none());
    assert!(local_datetime(date, 23, 60).is_none());
  }

  #[test]
  fn all_day_repeat_builds_reminders_during_the_day() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    let mut draft = EditorDraft::new(date, 90);
    draft.all_day = true;
    draft.repeat_all_day = true;
    draft.repeat_hours = 6;
    let start = local_datetime(date, 0, 0).unwrap();
    let end = local_datetime(date.checked_add_days(Days::new(1)).unwrap(), 0, 0).unwrap();

    let now = start - Duration::hours(12);
    let values: Vec<_> = build_reminders(&draft, None, start, end, now)
      .into_iter()
      .map(|reminder| reminder.minutes_before)
      .collect();
    assert_eq!(values, vec![90, -360, -720, -1080]);
  }

  #[test]
  fn disabled_reminder_produces_no_schedule() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    let mut draft = EditorDraft::new(date, 10);
    draft.reminder_enabled = false;
    let start = local_datetime(date, 9, 0).unwrap();
    assert!(
      build_reminders(
        &draft,
        None,
        start,
        start + Duration::hours(1),
        start - Duration::hours(1)
      )
      .is_empty()
    );
  }

  #[test]
  fn short_notice_events_remind_at_start_instead_of_in_the_past() {
    let now = Utc::now();
    let start = now + Duration::minutes(1);
    let mut draft = EditorDraft::new(start.with_timezone(&Local).date_naive(), 10);
    draft.reminder_hours = 0;
    draft.reminder_minutes = 10;

    let reminders = build_reminders(&draft, None, start, start + Duration::hours(1), now);
    assert_eq!(reminders.len(), 1);
    assert_eq!(reminders[0].minutes_before, 0);
  }

  #[test]
  fn save_editor_persists_a_short_notice_start_reminder() {
    let test_dir = std::env::temp_dir().join(format!(
      "argvus-calendar-save-reminder-{}",
      uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&test_dir).unwrap();
    let database_path = test_dir.join("calendar.db");
    let local_start = Local::now() + Duration::minutes(1);
    let local_end = local_start + Duration::hours(1);
    let mut draft = EditorDraft::new(local_start.date_naive(), 10);
    draft.title = "Short notice".to_string();
    draft.start_hour = local_start.hour();
    draft.start_minute = local_start.minute();
    draft.end_hour = local_end.hour();
    draft.end_minute = local_end.minute();

    save_editor(&database_path, &Settings::default(), draft).unwrap();

    let db = Database::open(&database_path).unwrap();
    let events = db.all_events().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].reminders.len(), 1);
    assert_eq!(events[0].reminders[0].minutes_before, 0);

    drop(db);
    std::fs::remove_dir_all(test_dir).unwrap();
  }
}
