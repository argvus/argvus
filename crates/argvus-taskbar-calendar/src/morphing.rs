//! Calendar session for the QML frontend. The caller holds the legacy
//! presentation socket for the lifetime of this UI, including handover to GTK.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use argvus_taskbar_popup::morphing::Server;
use chrono::{Datelike, Days, Local, NaiveDate};
use gtk::{gio, glib, prelude::FileExt};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::calendar::{
  CalendarEvent, first_of_month, next_month, previous_month, shift_year_clamped, today,
};
use crate::config::{self, Paths, Settings};
use crate::error::{ArgvusError, Result};
use crate::i18n::{I18n, Language, Text};
use crate::ipc::{self, IpcCommand, IpcMessage};
use crate::storage::Database;
use crate::ui::app::{PopupCommand, drafts, render};
use crate::ui::calendar::{day_cells, weekday_labels};
use crate::ui::event_editor::{EditorDraft, EditorFormValues, EditorKind, save_editor};
use crate::ui::settings::SettingsDraft;

mod file_picker;
pub use file_picker::FilePicker;

const COMMAND: &str = "taskbar-calendar";

pub fn run(paths: Paths, settings: Settings) -> Result<()> {
  let listener = match ipc::listen(&paths) {
    Some(listener) => listener,
    None => {
      // The GTK popup handles hide by releasing its ownership and exiting.
      ipc::notify(
        &paths,
        IpcMessage {
          command: IpcCommand::Hide,
          position: None,
        },
      );
      (0..20)
        .find_map(|_| {
          std::thread::sleep(Duration::from_millis(25));
          ipc::listen(&paths)
        })
        .ok_or_else(|| ArgvusError::Service("Calendar presentation is already active".into()))?
    }
  };
  // Keep the legacy listener alive until the presentation ends. The thread
  // ends with this process, or when the wake-up below asks it to stop.
  let (commands, pending) = mpsc::channel();
  let stop_ipc = ipc::serve_until(listener, move |message| commands.send(message).is_ok());
  let loop_ = glib::MainLoop::new(None, false);
  let session = Rc::new(RefCell::new(Session::new(paths, settings)));
  let attached = Rc::new(Cell::new(false));
  let fallback = Rc::new(Cell::new(None));
  let closing = Rc::new(Cell::new(false));
  let server = Server::start(
    COMMAND,
    {
      let loop_ = loop_.clone();
      let attached = Rc::clone(&attached);
      let closing = Rc::clone(&closing);
      move || {
        if closing.get() || attached.replace(true) {
          return None;
        }
        let loop_ = loop_.clone();
        let closing = Rc::clone(&closing);
        Some(Box::new(move || {
          closing.set(true);
          loop_.quit();
        }))
      }
    },
    {
      let session = Rc::clone(&session);
      move |action, args| session.borrow_mut().request(action, args)
    },
  )
  .map_err(|error| ArgvusError::Service(error.to_string()))?;
  let timer = glib::timeout_add_local(Duration::from_millis(50), {
    let closing = Rc::clone(&closing);
    let session = Rc::clone(&session);
    let fallback = Rc::clone(&fallback);
    let loop_ = loop_.clone();
    move || {
      for message in pending.try_iter() {
        match message.command {
          IpcCommand::Hide | IpcCommand::Toggle => {
            closing.set(true);
            loop_.quit();
          }
          IpcCommand::Show => {
            closing.set(true);
            fallback.set(Some(message));
            loop_.quit();
          }
          IpcCommand::ReloadTheme => {}
          IpcCommand::ReloadConfig => session.borrow_mut().reload(),
        }
      }
      glib::ControlFlow::Continue
    }
  });
  let startup = glib::timeout_add_local(Duration::from_secs(5), {
    let loop_ = loop_.clone();
    move || {
      if !attached.get() {
        loop_.quit();
      }
      glib::ControlFlow::Continue
    }
  });
  loop_.run();
  timer.remove();
  startup.remove();
  drop(server);
  stop_ipc();
  if let Some(message) = fallback.take() {
    let session = session.borrow();
    crate::ui::run_ui(
      session.paths.clone(),
      session.settings.clone(),
      PopupCommand::Show,
      message.position,
    );
  }
  Ok(())
}

struct Session {
  paths: Paths,
  settings: Settings,
  language: Language,
  i18n: I18n,
  selected: NaiveDate,
  month: NaiveDate,
  events: Vec<CalendarEvent>,
  event_days: HashSet<NaiveDate>,
  errors: Vec<Text>,
  last_read: Option<(NaiveDate, NaiveDate)>,
  draft: Option<EditorDraft>,
  enabled: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StartChange {
  hour: u32,
  minute: u32,
  values: EditorFormValues,
}

impl Session {
  fn new(paths: Paths, settings: Settings) -> Self {
    let selected = today();
    let language = Language::resolve(&settings.locale.language);
    let enabled = config::load_events_enabled(&paths, &settings);
    Self {
      paths,
      settings,
      language,
      enabled,
      i18n: I18n::new(language),
      selected,
      month: first_of_month(selected),
      events: Vec::new(),
      event_days: HashSet::new(),
      errors: Vec::new(),
      last_read: None,
      draft: None,
    }
  }

  fn reload(&mut self) {
    if let Ok(settings) = Settings::load(&self.paths) {
      self.settings = settings;
    }
    self.language = Language::resolve(&self.settings.locale.language);
    self.i18n = I18n::new(self.language);
    self.enabled = config::load_events_enabled(&self.paths, &self.settings);
    self.read_events(true);
  }

  fn read_events(&mut self, purge_expired: bool) {
    if !self.enabled {
      self.last_read = None;
      self.events.clear();
      self.event_days.clear();
      self.errors.clear();
      return;
    }
    let same_day = self
      .last_read
      .is_some_and(|(date, _)| date == self.selected);
    let same_month = self.last_read.is_some_and(|(_, month)| month == self.month);
    self.errors = render::apply_read(
      render::load_calendar(&self.paths, self.selected, self.month, purge_expired),
      &mut self.events,
      &mut self.event_days,
      same_day,
      same_month,
    );
    self.last_read = Some((self.selected, self.month));
  }

  fn request(&mut self, action: &str, args: Value) -> std::result::Result<Value, String> {
    self.execute(action, args).map_err(|error| {
      log::warn!("Calendar Morphing {action}: {error}");
      error.to_string()
    })
  }

  fn execute(&mut self, action: &str, args: Value) -> Result<Value> {
    match action {
      "state" => {
        self.read_events(self.last_read.is_none());
        return Ok(self.snapshot());
      }
      "today" => self.select(today())?,
      "select" => self.select(parse(args)?)?,
      "month" => {
        let month = match parse::<i32>(args)? {
          -1 => previous_month(self.month),
          1 => next_month(self.month),
          _ => return Err(invalid("month direction")),
        };
        self.month = checked_date(month)?;
      }
      "year" => {
        let offset = parse::<i32>(args)?;
        if !matches!(offset, -1 | 1) {
          return Err(invalid("year direction"));
        }
        self.select(shift_year_clamped(self.selected, offset))?;
      }
      "day" => {
        let offset = parse::<i32>(args)?;
        let next = match offset {
          -7 | -1 => self
            .selected
            .checked_sub_days(Days::new(offset.unsigned_abs().into())),
          1 | 7 => self
            .selected
            .checked_add_days(Days::new(offset.unsigned_abs().into())),
          _ => return Err(invalid("day direction")),
        };
        self.select(next.ok_or_else(|| invalid("date"))?)?;
      }
      "new" => {
        if !self.enabled || !drafts::can_create_event(self.selected) {
          return Err(invalid(self.i18n.text(Text::PastDate)));
        }
        let draft = EditorDraft::new(self.selected, &self.settings.calendar);
        return Ok(self.open_editor(draft));
      }
      "edit" => {
        let id = parse::<String>(args)?
          .parse()
          .map_err(|_| invalid("event ID"))?;
        let event = self.event(id)?;
        return Ok(self.open_editor(EditorDraft::from_event(event)));
      }
      "editor_start" => {
        let change: StartChange = parse(args)?;
        validate_values(&change.values)?;
        if change.hour > 23 || change.minute > 59 {
          return Err(invalid("start time"));
        }
        let mut draft = self.draft.clone().ok_or_else(|| invalid("editor"))?;
        draft.apply_form_values(change.values);
        draft.set_start_time(change.hour, change.minute);
        return Ok(
          json!({ "result": { "end_hour": draft.end_hour, "end_minute": draft.end_minute } }),
        );
      }
      "save" => {
        let values = parse(args)?;
        validate_values(&values)?;
        let mut draft = self.current_draft()?;
        draft.apply_form_values(values);
        save_editor(&self.paths.database, &self.settings, draft, self.i18n)?;
        self.draft = None;
      }
      "delete" => {
        let draft = self.current_draft()?;
        let EditorKind::Edit(event) = draft.kind else {
          return Err(invalid("event ID"));
        };
        let id = event.id.ok_or_else(|| invalid("event ID"))?;
        Database::open(&self.paths.database)?.mark_deleted(id)?;
        self.draft = None;
      }
      "cancel" => self.draft = None,
      "open_config" => config::open_config(&self.paths, &self.settings)?,
      "settings" => {
        let draft: SettingsDraft = parse(args)?;
        let config = draft.into_config();
        config.save(&self.paths)?;
        if config.calendar.show_events != self.enabled {
          config::save_events_enabled(&self.paths, config.calendar.show_events)?;
          drafts::control_reminder_service(config.calendar.show_events);
        }
        self.reload();
      }
      "import" => {
        crate::app::import_events(&self.paths, file_path(args)?)?;
      }
      "export" => {
        crate::app::export_events(&self.paths, &file_path(args)?)?;
      }
      "status" => return Ok(json!({ "result": crate::app::status_report(&self.paths)? })),
      "sync" => {
        return Ok(json!({ "result": crate::app::sync_report(&self.paths, &self.settings)? }));
      }
      _ => return Err(invalid("action")),
    }
    self.read_events(true);
    Ok(self.snapshot())
  }

  fn select(&mut self, date: NaiveDate) -> Result<()> {
    self.selected = checked_date(date)?;
    self.month = first_of_month(date);
    Ok(())
  }

  fn event(&self, id: i64) -> Result<CalendarEvent> {
    if !self.enabled {
      return Err(invalid("events disabled"));
    }
    Database::open(&self.paths.database)?
      .event_by_id(id)?
      .filter(|event| !event.deleted)
      .ok_or_else(|| invalid("event no longer exists"))
  }

  fn current_draft(&self) -> Result<EditorDraft> {
    let mut draft = self
      .draft
      .clone()
      .filter(EditorDraft::can_save)
      .ok_or_else(|| invalid(self.i18n.text(Text::PastDate)))?;
    if let EditorKind::Edit(original) = &mut draft.kind {
      let latest = self.event(original.id.ok_or_else(|| invalid("event ID"))?)?;
      if latest.updated_at != original.updated_at {
        return Err(invalid("event changed after opening"));
      }
      // The reminder service can acknowledge alarm delivery while editing.
      **original = latest;
    }
    Ok(draft)
  }

  fn open_editor(&mut self, draft: EditorDraft) -> Value {
    let editing = matches!(draft.kind, EditorKind::Edit(_));
    let values = EditorFormValues {
      title: draft.title.clone(),
      location: draft.location.clone(),
      description: draft.description.clone(),
      start_hour: draft.start_hour,
      start_minute: draft.start_minute,
      end_hour: draft.end_hour,
      end_minute: draft.end_minute,
      all_day: draft.all_day,
      reminder_enabled: draft.reminder_enabled,
      reminder_hours: draft.reminder_hours,
      reminder_minutes: draft.reminder_minutes,
      repeat_all_day: draft.repeat_all_day,
      repeat_hours: draft.repeat_hours,
      repeat_minutes: draft.repeat_minutes,
    };
    let reply = json!({ "result": {
      "date": self.i18n.agenda_date(draft.date),
      "can_save": draft.can_save(),
      "editing": editing,
      "values": values,
    } });
    self.draft = Some(draft);
    reply
  }

  fn snapshot(&self) -> Value {
    let days: Vec<_> = day_cells(self.month, self.settings.week_start()).into_iter().map(|(date, in_month)| {
      json!({
        "date": date, "day": date.day(), "selected": date == self.selected, "current": date == today(),
        "has_event": self.event_days.contains(&date), "in_month": in_month,
        "label": self.i18n.agenda_date(date),
      })
    }).collect();
    let events: Vec<_> = self
      .events
      .iter()
      .map(|event| {
        let time = if event.all_day {
          self.i18n.text(Text::AllDay).to_owned()
        } else {
          event
            .start
            .with_timezone(&Local)
            .format("%H:%M")
            .to_string()
        };
        json!({ "id": event.id.map(|id| id.to_string()), "title": event.title, "time": time })
      })
      .collect();
    let errors: Vec<_> = self
      .errors
      .iter()
      .map(|error| self.i18n.text(*error))
      .collect();
    json!({
      "labels": labels(self.i18n), "rtl": self.language.is_rtl(),
      "title": self.i18n.month_title(self.month),
      "agenda": self.i18n.agenda_date(self.selected),
      "weekdays": weekday_labels(self.i18n.weekday_headers(), self.settings.week_start()),
      "days": days, "events": events, "events_enabled": self.enabled,
      "errors": errors, "daily_error": self.errors.iter().any(|error| *error != Text::MonthlyReadFailed),
      "can_add": self.enabled && drafts::can_create_event(self.selected),
      "settings": SettingsDraft::from_config(&self.settings, self.enabled),
      "locales": argvus_i18n::installed_locales().into_iter()
        .map(|locale| json!({ "tag": locale.locale, "name": locale.native_name }))
        .collect::<Vec<_>>(),
      "monday": self.i18n.weekday_headers()[0],
      "sunday": self.i18n.weekday_headers()[6],
    })
  }
}

fn parse<T: DeserializeOwned>(args: Value) -> Result<T> {
  serde_json::from_value(args).map_err(|error| invalid(&error.to_string()))
}

fn invalid(detail: &str) -> ArgvusError {
  ArgvusError::Configuration(detail.into())
}

fn checked_date(date: NaiveDate) -> Result<NaiveDate> {
  (1..=9999)
    .contains(&date.year())
    .then_some(date)
    .ok_or_else(|| invalid("date out of range"))
}

fn file_path(args: Value) -> Result<PathBuf> {
  let uri: String = parse(args)?;
  if !uri.starts_with("file:///") {
    return Err(invalid("local file required"));
  }
  gio::File::for_uri(&uri)
    .path()
    .ok_or_else(|| invalid("local file required"))
}

fn validate_values(values: &EditorFormValues) -> Result<()> {
  let hours = [
    values.start_hour,
    values.end_hour,
    values.reminder_hours,
    values.repeat_hours,
  ];
  let minutes = [
    values.start_minute,
    values.end_minute,
    values.reminder_minutes,
    values.repeat_minutes,
  ];
  if hours.into_iter().any(|hour| hour > 23) || minutes.into_iter().any(|minute| minute > 59) {
    return Err(invalid("time out of range"));
  }
  Ok(())
}

fn labels(i18n: I18n) -> Value {
  let labels = [
    ("Today", i18n.text(Text::Today)),
    ("AddEvent", i18n.text(Text::AddEvent)),
    ("NewEvent", i18n.text(Text::NewEvent)),
    ("EditEvent", i18n.text(Text::EditEvent)),
    ("Save", i18n.text(Text::Save)),
    ("Cancel", i18n.text(Text::Cancel)),
    ("Title", i18n.text(Text::Title)),
    ("Location", i18n.text(Text::Location)),
    ("Description", i18n.text(Text::Description)),
    ("Date", i18n.text(Text::Date)),
    ("StartTime", i18n.text(Text::StartTime)),
    ("EndTime", i18n.text(Text::EndTime)),
    ("Reminder", i18n.text(Text::Reminder)),
    ("Repeat", i18n.text(Text::Repeat)),
    ("Before", i18n.text(Text::Before)),
    ("HourShort", i18n.text(Text::HourShort)),
    ("MinuteShort", i18n.text(Text::MinuteShort)),
    ("AllDay", i18n.text(Text::AllDay)),
    ("NoEvents", i18n.text(Text::NoEvents)),
    ("Events", i18n.text(Text::Events)),
    ("NotifyAt", i18n.text(Text::NotifyAt)),
    ("Delete", i18n.text(Text::Delete)),
    ("Settings", i18n.text(Text::Settings)),
    ("PreviousMonth", i18n.text(Text::PreviousMonth)),
    ("NextMonth", i18n.text(Text::NextMonth)),
    ("PreviousYear", i18n.text(Text::PreviousYear)),
    ("NextYear", i18n.text(Text::NextYear)),
    ("Back", i18n.text(Text::Back)),
    ("PastDate", i18n.text(Text::PastDate)),
    ("FontFamily", i18n.text(Text::FontFamily)),
    ("FontSize", i18n.text(Text::FontSize)),
    ("Language", i18n.text(Text::Language)),
    ("WeekStart", i18n.text(Text::WeekStart)),
    ("DefaultDuration", i18n.text(Text::DefaultDuration)),
    ("DefaultReminder", i18n.text(Text::DefaultReminder)),
    ("SyncEvery", i18n.text(Text::SyncEvery)),
    ("Command", i18n.text(Text::Command)),
    ("Args", i18n.text(Text::Args)),
    ("EditConfigFile", i18n.text(Text::EditConfigFile)),
    ("Terminal", i18n.text(Text::Terminal)),
    ("Editor", i18n.text(Text::Editor)),
    ("Import", i18n.extra("morphing.import")),
    ("Export", i18n.extra("morphing.export")),
    ("Sync", i18n.extra("morphing.sync")),
    ("Status", i18n.extra("morphing.status")),
    ("Automatic", i18n.extra("morphing.automatic")),
  ];
  Value::Object(
    labels
      .into_iter()
      .map(|(key, value)| (key.into(), value.into()))
      .collect(),
  )
}
