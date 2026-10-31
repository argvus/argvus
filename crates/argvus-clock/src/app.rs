//! The GTK application: alarms, timer and stopwatch scheduling, ringing,
//! persistence and the window lifecycle.
//!
//! Reliability rules (documented for users in the Clock page of the docs):
//! - Alarms ring only while this process runs in the user's session. Closing
//!   the window keeps the process (and the alarms) running while anything is
//!   pending, unless "Keep running in the background" is off. "Start with the
//!   session" enables `argvus-clock.service`, which starts it hidden at login.
//! - A single instance runs per session (`sh.argvus.Clock` on the session
//!   bus), so there is only one scheduler.
//! - An occurrence is marked handled and saved before it rings, so reopening
//!   the application never rings it again.
//! - An alarm noticed more than five minutes late (the process was not
//!   running, or the system was asleep or off) does not ring: it is reported
//!   as missed. A timer that ended while the system slept rings on wake-up and
//!   says how late it is.

use std::cell::{Cell, OnceCell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use argvus_taskbar_popup::{PopupAnchor, PopupApp, close_competing_surfaces};
use argvus_time_core::alarm::{AlarmBook, AlarmDraft, AlarmId, Due, DueKind, SoundChoice};
use argvus_time_core::elapsed::Stopwatch;
use argvus_time_core::store::{AppDirs, Loaded, StoreError};
use argvus_time_core::ticks::Ticks;
use argvus_time_ui::sound::{Playback, SoundError, SoundPlayer};
use argvus_time_ui::text::{Calendar, Catalog};
use argvus_time_ui::theme::ThemeStyle;
use chrono::{DateTime, Local, Utc};
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::cli::{Cli, PageArg};
use crate::ringing::{self, Entry, RingQueue, Source};
use crate::settings::{self, Settings};
use crate::timers::{self, Timer, TimersState};
use crate::ui::{ClockWindow, Page, RingWindow};
use crate::{alarms, autostart};

pub const APP_ID: &str = "sh.argvus.Clock";
const DOMAIN: &str = "clock";
/// Wake slightly after the second changes, so the display never lags.
const TICK_SLACK: Duration = Duration::from_millis(15);
/// Longest sleep between alarm checks while the window is closed.
const MAX_IDLE_WAIT: Duration = Duration::from_secs(30);
const SETTINGS_SAVE_DELAY: Duration = Duration::from_millis(400);
/// A sound test plays for this long at most.
const SOUND_TEST_LENGTH: Duration = Duration::from_secs(4);
const APP_CSS: &str = include_str!("ui/clock.css");

type ActionHandler = fn(&Rc<ClockApp>);
type AlarmHandler = fn(&Rc<ClockApp>, AlarmId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
  TimersRestored,
  SoundFailed(String),
  CustomSoundMissing,
  SaveFailed(String),
  Recovered(PathBuf),
  AutostartFailed(String),
}

/// An alarm that was due while it could not ring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissedAlarm {
  pub title: String,
  pub due_at: DateTime<Utc>,
}

struct Paths {
  settings: PathBuf,
  alarms: PathBuf,
  timers: PathBuf,
}

pub struct ClockApp {
  pub application: gtk::Application,
  pub texts: Catalog,
  pub calendar: Calendar,
  paths: Option<Paths>,
  settings: RefCell<Settings>,
  alarms: RefCell<AlarmBook>,
  timer: RefCell<Timer>,
  stopwatch: RefCell<Stopwatch>,
  ringing: RefCell<RingQueue>,
  missed: RefCell<Vec<MissedAlarm>>,
  notice: RefCell<Option<Notice>>,
  sound: SoundPlayer,
  /// The choice the ring sound is playing, so a refresh does not restart it.
  ring_sound: RefCell<Option<SoundChoice>>,
  sound_test: RefCell<Option<glib::SourceId>>,
  heartbeat: RefCell<Option<glib::SourceId>>,
  settings_save: RefCell<Option<glib::SourceId>>,
  started_in_background: Cell<bool>,
  /// Set once the first command line was handled. Before that the window
  /// has not been shown yet, so the process must not decide it is idle.
  launched: Cell<bool>,
  autostart: Cell<Option<bool>>,
  theme: OnceCell<Rc<ThemeStyle>>,
  pending_presentations: Cell<usize>,
  window: OnceCell<ClockWindow>,
  ring_window: OnceCell<RingWindow>,
}

pub fn run() -> glib::ExitCode {
  let application = gtk::Application::builder()
    .application_id(APP_ID)
    .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
    .build();
  let state: Rc<OnceCell<Rc<ClockApp>>> = Rc::default();

  application.connect_startup({
    let state = Rc::clone(&state);
    move |application| {
      let app = ClockApp::new(application);
      app.install();
      let _ = state.set(app);
    }
  });
  application.connect_command_line({
    let state = Rc::clone(&state);
    move |_, command_line| match state.get() {
      Some(app) => app.handle_command_line(command_line),
      None => glib::ExitCode::FAILURE,
    }
  });
  application.connect_activate({
    let state = Rc::clone(&state);
    move |_| {
      if let Some(app) = state.get() {
        app.present(None, None);
      }
    }
  });
  application.connect_shutdown({
    let state = Rc::clone(&state);
    move |_| {
      if let Some(app) = state.get() {
        app.shutdown();
      }
    }
  });
  application.run()
}

impl ClockApp {
  fn new(application: &gtk::Application) -> Rc<Self> {
    let mut notice = None;
    let paths = match AppDirs::for_app("clock") {
      Ok(directories) => Some(Paths {
        settings: directories.config.join(settings::FILE_NAME),
        alarms: directories.data.join(alarms::FILE_NAME),
        timers: directories.state.join(timers::FILE_NAME),
      }),
      Err(error) => {
        log::error!("{error}");
        notice = Some(Notice::SaveFailed(error.to_string()));
        None
      }
    };
    let settings = load_or_default(
      paths.as_ref().map(|paths| Settings::load(&paths.settings)),
      &mut notice,
    );
    let alarms = load_or_default(
      paths.as_ref().map(|paths| alarms::load(&paths.alarms)),
      &mut notice,
    );
    let timers_state = load_or_default(
      paths.as_ref().map(|paths| TimersState::load(&paths.timers)),
      &mut notice,
    );
    let (timer, stopwatch, restored) = timers_state.restore();
    if restored.timer_paused || restored.stopwatch_paused {
      notice = Some(Notice::TimersRestored);
    }

    Rc::new(Self {
      application: application.clone(),
      texts: Catalog::load(DOMAIN),
      calendar: Calendar::load(),
      paths,
      settings: RefCell::new(settings),
      alarms: RefCell::new(alarms),
      timer: RefCell::new(timer),
      stopwatch: RefCell::new(stopwatch),
      ringing: RefCell::default(),
      missed: RefCell::default(),
      notice: RefCell::new(notice),
      sound: SoundPlayer::new(),
      ring_sound: RefCell::new(None),
      sound_test: RefCell::new(None),
      heartbeat: RefCell::new(None),
      settings_save: RefCell::new(None),
      started_in_background: Cell::new(false),
      launched: Cell::new(false),
      autostart: Cell::new(None),
      theme: OnceCell::new(),
      pending_presentations: Cell::new(0),
      window: OnceCell::new(),
      ring_window: OnceCell::new(),
    })
  }

  fn install(self: &Rc<Self>) {
    let _ = self.theme.set(ThemeStyle::install(APP_CSS));
    self.install_actions();
    let window = ClockWindow::build(self);
    if let Some(theme) = self.theme.get() {
      theme.watch(&window.window);
    }
    let _ = self.window.set(window);
    let _ = self.ring_window.set(RingWindow::new());
    self.refresh_autostart();
    // Check the alarms at once: missed ones are reported right after start.
    self.on_heartbeat();
  }

  fn install_actions(self: &Rc<Self>) {
    let actions: [(&str, &[&str], ActionHandler); 9] = [
      ("show", &[], |app| app.present(None, None)),
      ("close", &["<Control>w"], |app| app.close_window()),
      ("settings", &["<Control>comma"], |app| {
        app.show_page(Page::Settings)
      }),
      ("new-alarm", &["<Control>n"], |app| app.edit_alarm(None)),
      ("page-clock", &["<Control>1"], |app| {
        app.show_page(Page::Clock)
      }),
      ("page-world", &["<Control>2"], |app| {
        app.show_page(Page::World)
      }),
      ("page-stopwatch", &["<Control>3"], |app| {
        app.show_page(Page::Stopwatch)
      }),
      ("page-timer", &["<Control>4"], |app| {
        app.show_page(Page::Timer)
      }),
      ("page-alarms", &["<Control>5"], |app| {
        app.show_page(Page::Alarms)
      }),
    ];
    for (name, accels, handler) in actions {
      let action = gio::SimpleAction::new(name, None);
      let app = Rc::downgrade(self);
      action.connect_activate(move |_, _| {
        if let Some(app) = app.upgrade() {
          handler(&app);
        }
      });
      self.application.add_action(&action);
      if !accels.is_empty() {
        self
          .application
          .set_accels_for_action(&format!("app.{name}"), accels);
      }
    }

    let alarm_actions: [(&str, AlarmHandler); 2] = [
      ("dismiss-alarm", |app, id| app.dismiss(Source::Alarm(id))),
      ("snooze-alarm", |app, id| app.snooze(id)),
    ];
    for (name, handler) in alarm_actions {
      let action = gio::SimpleAction::new(name, Some(glib::VariantTy::UINT64));
      let app = Rc::downgrade(self);
      action.connect_activate(move |_, parameter| {
        if let (Some(app), Some(id)) = (app.upgrade(), parameter.and_then(u64::from_variant)) {
          handler(&app, AlarmId(id));
        }
      });
      self.application.add_action(&action);
    }
    let dismiss_timer = gio::SimpleAction::new("dismiss-timer", None);
    let app = Rc::downgrade(self);
    dismiss_timer.connect_activate(move |_, _| {
      if let Some(app) = app.upgrade() {
        app.dismiss(Source::Timer);
      }
    });
    self.application.add_action(&dismiss_timer);

    let quit = gio::SimpleAction::new("quit", None);
    let application = self.application.downgrade();
    quit.connect_activate(move |_, _| {
      if let Some(application) = application.upgrade() {
        application.quit();
      }
    });
    self.application.add_action(&quit);
    self
      .application
      .set_accels_for_action("app.quit", &["<Control>q"]);
  }

  fn handle_command_line(
    self: &Rc<Self>,
    command_line: &gio::ApplicationCommandLine,
  ) -> glib::ExitCode {
    // `main` already rejected invalid arguments in the launching process.
    let cli = match Cli::parse_arguments(command_line.arguments()) {
      Ok(cli) => cli,
      Err(error) => {
        log::error!("ignoring invalid arguments: {error}");
        return glib::ExitCode::FAILURE;
      }
    };
    let page = cli.page.map(Page::from);
    self.launched.set(true);
    if cli.background {
      // Only the first instance can start hidden; a later launch with
      // --background has nothing to do.
      if !command_line.is_remote() {
        self.started_in_background.set(true);
      }
    } else if cli.toggle {
      self.toggle_window(cli.anchor(), page);
    } else {
      self.present(cli.anchor(), page);
    }
    self.schedule_heartbeat();
    self.quit_if_idle();
    glib::ExitCode::SUCCESS
  }

  // Heartbeat: alarms, timer, ring limits and the displayed time.

  /// Whether the process must keep running with the window closed.
  fn has_pending_work(&self) -> bool {
    let now = Ticks::now().boottime;
    self.alarms.borrow().has_pending()
      || self.timer.borrow().is_running(now)
      || self.stopwatch.borrow().is_running()
      || !self.ringing.borrow().is_empty()
  }

  fn window_visible(&self) -> bool {
    self
      .window
      .get()
      .is_some_and(|window| window.window.is_visible())
  }

  fn schedule_heartbeat(self: &Rc<Self>) {
    if let Some(source) = self.heartbeat.borrow_mut().take() {
      source.remove();
    }
    if !self.window_visible() && !self.has_pending_work() {
      return;
    }
    let wall = Utc::now();
    let until_timer = self.timer.borrow().until_next_second(Ticks::now().boottime);
    let only_alarms =
      !self.window_visible() && self.ringing.borrow().is_empty() && until_timer.is_none();
    let delay = if only_alarms {
      // Nothing is shown: wake for the next alarm, and at least every
      // MAX_IDLE_WAIT so a clock change or a wake-up is noticed soon.
      let until_alarm = self
        .alarms
        .borrow()
        .next_trigger(&Local)
        .and_then(|(_, trigger)| (trigger.at - wall).to_std().ok())
        .unwrap_or(MAX_IDLE_WAIT);
      until_alarm.min(MAX_IDLE_WAIT) + TICK_SLACK
    } else {
      let until_wall_second = Duration::from_nanos(u64::from(
        1_000_000_000 - wall.timestamp_subsec_nanos().min(999_999_999),
      ));
      until_timer.map_or(until_wall_second, |timer| timer.min(until_wall_second)) + TICK_SLACK
    };
    let app = Rc::downgrade(self);
    let source = glib::timeout_add_local_once(delay, move || {
      if let Some(app) = app.upgrade() {
        app.heartbeat.borrow_mut().take();
        app.on_heartbeat();
      }
    });
    *self.heartbeat.borrow_mut() = Some(source);
  }

  fn on_heartbeat(self: &Rc<Self>) {
    let wall = Utc::now();
    let boot = Ticks::now().boottime;

    let due = self.alarms.borrow_mut().take_due(wall, &Local);
    if !due.is_empty() {
      // Saved before ringing: a restart never rings these occurrences again.
      self.save_alarms();
      for due in due {
        self.on_alarm_due(due, boot);
      }
    }

    let finished = self.timer.borrow_mut().take_finished(boot);
    if let Some(finished) = finished {
      self.save_timers();
      let sound = self.settings.borrow().timer_sound.clone();
      self.ring(Entry {
        source: Source::Timer,
        title: self.texts.tr("timer.finished"),
        due_at: wall,
        started: boot,
        sound,
        late_by: finished.late_by,
      });
    }

    let limit = Duration::from_secs(u64::from(self.settings.borrow().ring_limit_minutes) * 60);
    let expired = self.ringing.borrow_mut().take_expired(boot, limit);
    for entry in expired {
      self.notify_not_dismissed(&entry);
    }
    if !self.ringing.borrow().is_empty() || self.ring_sound.borrow().is_some() {
      self.update_ring();
    }

    self.refresh_live();
    self.schedule_heartbeat();
    self.quit_if_idle();
  }

  fn on_alarm_due(self: &Rc<Self>, due: Due, boot: Duration) {
    let (title, sound) = match self.alarms.borrow().get(due.id) {
      Some(alarm) if !alarm.label.is_empty() => (alarm.label.clone(), alarm.sound.clone()),
      Some(alarm) => (self.texts.tr("alarm.default_title"), alarm.sound.clone()),
      None => return,
    };
    match due.kind {
      DueKind::OnTime => self.ring(Entry {
        source: Source::Alarm(due.id),
        title,
        due_at: due.occurrence,
        started: boot,
        sound,
        late_by: None,
      }),
      DueKind::Missed => {
        let missed = MissedAlarm {
          title,
          due_at: due.occurrence,
        };
        self.notify_missed(&missed);
        self.missed.borrow_mut().push(missed);
        self.refresh();
      }
    }
  }

  // Ringing.

  fn ring(self: &Rc<Self>, entry: Entry) {
    self.send_ring_notification(&entry);
    self.ringing.borrow_mut().push(entry);
    self.update_ring();
    self.schedule_heartbeat();
  }

  /// Starts, keeps or stops the ring sound and shows or hides the ring
  /// window to match the queue.
  fn update_ring(self: &Rc<Self>) {
    let wanted = self.ringing.borrow().sound().cloned();
    let playing = self.ring_sound.borrow().clone();
    if wanted != playing {
      self.stop_sound_test();
      self.sound.stop();
      *self.ring_sound.borrow_mut() = None;
      if let Some(choice) = wanted {
        self.play_ring_sound(&choice);
        *self.ring_sound.borrow_mut() = Some(choice);
      }
    }
    if let Some(ring_window) = self.ring_window.get() {
      ring_window.update(self);
    }
    self.refresh();
  }

  fn play_ring_sound(self: &Rc<Self>, choice: &SoundChoice) {
    let Some((path, fallback)) = ringing::sound_path(choice) else {
      return;
    };
    if fallback {
      self.set_notice(Notice::CustomSoundMissing);
    }
    let volume = self.settings.borrow().volume_fraction();
    let app = Rc::downgrade(self);
    let result = self
      .sound
      .play(&path, Playback::Loop, volume, move |error| {
        if let Some(app) = app.upgrade() {
          app.on_sound_failed(error);
        }
      });
    if let Err(error) = result {
      self.on_sound_failed(&error);
    }
  }

  fn on_sound_failed(self: &Rc<Self>, error: &SoundError) {
    log::error!("alarm sound failed: {error}");
    self.set_notice(Notice::SoundFailed(error.to_string()));
    // Keep the choice so the failed sound is not retried every second.
    if let Some(ring_window) = self.ring_window.get() {
      ring_window.update(self);
    }
  }

  /// Whether the ring sound failed (shown on the ring window).
  pub fn sound_failed(&self) -> Option<String> {
    match self.notice.borrow().as_ref() {
      Some(Notice::SoundFailed(error)) if self.ring_sound.borrow().is_some() => Some(error.clone()),
      _ => None,
    }
  }

  pub fn dismiss(self: &Rc<Self>, source: Source) {
    self.ringing.borrow_mut().remove(source);
    self
      .application
      .withdraw_notification(&notification_id(source));
    if let Source::Alarm(id) = source {
      let changed = self
        .alarms
        .borrow_mut()
        .get_mut(id)
        .map(|alarm| alarm.dismiss())
        .is_some();
      if changed {
        self.save_alarms();
      }
    }
    self.update_ring();
    self.schedule_heartbeat();
    self.quit_if_idle();
  }

  pub fn snooze(self: &Rc<Self>, id: AlarmId) {
    self.ringing.borrow_mut().remove(Source::Alarm(id));
    self
      .application
      .withdraw_notification(&notification_id(Source::Alarm(id)));
    let snoozed = self
      .alarms
      .borrow_mut()
      .get_mut(id)
      .map(|alarm| alarm.snooze(Utc::now()))
      .is_some();
    if snoozed {
      self.save_alarms();
    }
    self.update_ring();
    self.schedule_heartbeat();
  }

  pub fn ringing(&self) -> std::cell::Ref<'_, RingQueue> {
    self.ringing.borrow()
  }

  pub fn snooze_minutes(&self, id: AlarmId) -> Option<u32> {
    self
      .alarms
      .borrow()
      .get(id)
      .map(|alarm| alarm.snooze_minutes)
  }

  // Notifications.

  fn send_ring_notification(&self, entry: &Entry) {
    let notification = gio::Notification::new(&entry.title);
    let mut body = match entry.source {
      Source::Alarm(_) => self.texts.tr_args(
        "notify.alarm.body",
        [("time", self.local_time(entry.due_at))],
      ),
      Source::Timer => self.texts.tr("notify.timer.body"),
    };
    if let Some(late_by) = entry.late_by {
      body = format!("{body} {}", self.late_text(late_by));
    }
    notification.set_body(Some(&body));
    notification.set_icon(&gio::ThemedIcon::new(APP_ID));
    notification.set_priority(gio::NotificationPriority::Urgent);
    notification.set_default_action("app.show");
    match entry.source {
      Source::Alarm(id) => {
        let target = id.0.to_variant();
        notification.add_button_with_target_value(
          &self.texts.tr("action.snooze"),
          "app.snooze-alarm",
          Some(&target),
        );
        notification.add_button_with_target_value(
          &self.texts.tr("action.dismiss"),
          "app.dismiss-alarm",
          Some(&target),
        );
      }
      Source::Timer => {
        notification.add_button(&self.texts.tr("action.dismiss"), "app.dismiss-timer")
      }
    }
    self
      .application
      .send_notification(Some(&notification_id(entry.source)), &notification);
  }

  fn notify_missed(&self, missed: &MissedAlarm) {
    let notification = gio::Notification::new(&self.texts.tr("notify.missed.title"));
    notification.set_body(Some(&self.texts.tr_args(
      "notify.missed.body",
      [
        ("title", missed.title.clone()),
        ("time", self.local_time(missed.due_at)),
      ],
    )));
    notification.set_icon(&gio::ThemedIcon::new(APP_ID));
    notification.set_default_action("app.page-alarms");
    self.application.send_notification(None, &notification);
  }

  fn notify_not_dismissed(&self, entry: &Entry) {
    self
      .application
      .withdraw_notification(&notification_id(entry.source));
    let notification = gio::Notification::new(&self.texts.tr("notify.unanswered.title"));
    let minutes = self.settings.borrow().ring_limit_minutes.to_string();
    notification.set_body(Some(&self.texts.tr_args(
      "notify.unanswered.body",
      [("title", entry.title.clone()), ("minutes", minutes)],
    )));
    notification.set_icon(&gio::ThemedIcon::new(APP_ID));
    notification.set_default_action("app.show");
    self.application.send_notification(None, &notification);
  }

  pub fn late_text(&self, late_by: Duration) -> String {
    let minutes = (late_by.as_secs() / 60).max(1);
    self
      .texts
      .tr_args("ring.late", [("minutes", minutes.to_string())])
  }

  pub fn local_time(&self, at: DateTime<Utc>) -> String {
    let cycle = self.settings.borrow().hour_cycle;
    self
      .calendar
      .time(&at.with_timezone(&Local), cycle, false)
      .joined()
  }

  // Sound test.

  pub fn test_sound(self: &Rc<Self>, choice: &SoundChoice) {
    if !self.ringing.borrow().is_empty() {
      return;
    }
    let Some((path, fallback)) = ringing::sound_path(choice) else {
      return;
    };
    if fallback {
      self.set_notice(Notice::CustomSoundMissing);
    }
    self.stop_sound_test();
    let volume = self.settings.borrow().volume_fraction();
    let app = Rc::downgrade(self);
    let result = self
      .sound
      .play(&path, Playback::Once, volume, move |error| {
        if let Some(app) = app.upgrade() {
          app.set_notice(Notice::SoundFailed(error.to_string()));
        }
      });
    match result {
      Ok(()) => {
        let player = self.sound.clone();
        let app = Rc::downgrade(self);
        let stop = glib::timeout_add_local_once(SOUND_TEST_LENGTH, move || {
          if let Some(app) = app.upgrade() {
            app.sound_test.borrow_mut().take();
          }
          player.stop();
        });
        *self.sound_test.borrow_mut() = Some(stop);
      }
      Err(error) => self.set_notice(Notice::SoundFailed(error.to_string())),
    }
  }

  fn stop_sound_test(&self) {
    if let Some(source) = self.sound_test.borrow_mut().take() {
      source.remove();
    }
  }

  // Alarms.

  pub fn alarms(&self) -> std::cell::Ref<'_, AlarmBook> {
    self.alarms.borrow()
  }

  pub fn save_alarm(self: &Rc<Self>, id: Option<AlarmId>, draft: AlarmDraft) {
    {
      let mut book = self.alarms.borrow_mut();
      match id {
        Some(id) => {
          book.update(id, draft, Utc::now());
        }
        None => {
          book.add(draft, Utc::now());
        }
      }
    }
    self.save_alarms();
    self.show_page(Page::Alarms);
    self.schedule_heartbeat();
  }

  pub fn set_alarm_enabled(self: &Rc<Self>, id: AlarmId, enabled: bool) {
    let changed = self
      .alarms
      .borrow_mut()
      .get_mut(id)
      .filter(|alarm| alarm.enabled != enabled)
      .map(|alarm| alarm.set_enabled(enabled, Utc::now()))
      .is_some();
    if changed {
      self.save_alarms();
      self.refresh();
      self.schedule_heartbeat();
    }
  }

  pub fn delete_alarm(self: &Rc<Self>, id: AlarmId) {
    self.dismiss(Source::Alarm(id));
    if self.alarms.borrow_mut().remove(id) {
      self.save_alarms();
    }
    self.show_page(Page::Alarms);
  }

  pub fn edit_alarm(self: &Rc<Self>, id: Option<AlarmId>) {
    if let Some(window) = self.window.get() {
      let visible = window.window.is_visible();
      window.edit_alarm(self, id);
      if !visible {
        self.present(None, None);
      }
    }
  }

  pub fn missed(&self) -> std::cell::Ref<'_, Vec<MissedAlarm>> {
    self.missed.borrow()
  }

  pub fn clear_missed(self: &Rc<Self>) {
    self.missed.borrow_mut().clear();
    self.refresh();
  }

  // Timer and stopwatch.

  pub fn timer(&self) -> std::cell::Ref<'_, Timer> {
    self.timer.borrow()
  }

  pub fn stopwatch(&self) -> std::cell::Ref<'_, Stopwatch> {
    self.stopwatch.borrow()
  }

  pub fn change_timer(self: &Rc<Self>, change: impl FnOnce(&mut Timer, Duration)) {
    change(&mut self.timer.borrow_mut(), Ticks::now().boottime);
    self.dismiss_timers_notice();
    self.save_timers();
    self.refresh();
    self.schedule_heartbeat();
    self.quit_if_idle();
  }

  pub fn start_timer(self: &Rc<Self>, length: Duration) {
    if length.is_zero() {
      return;
    }
    self.update_settings(|settings| settings.timer_seconds = length.as_secs());
    self.change_timer(|timer, now| {
      timer.start(length, now);
    });
  }

  pub fn change_stopwatch(self: &Rc<Self>, change: impl FnOnce(&mut Stopwatch, Duration)) {
    change(&mut self.stopwatch.borrow_mut(), Ticks::now().boottime);
    self.dismiss_timers_notice();
    self.save_timers();
    self.refresh();
    self.schedule_heartbeat();
  }

  // Persistence.

  fn save_alarms(self: &Rc<Self>) {
    let saved = self
      .paths
      .as_ref()
      .map(|paths| alarms::save(&paths.alarms, &self.alarms.borrow()));
    self.report_save(saved);
  }

  fn save_timers(self: &Rc<Self>) {
    let now = Ticks::now().boottime;
    let saved = self.paths.as_ref().map(|paths| {
      TimersState::capture(&self.timer.borrow(), &self.stopwatch.borrow(), now).save(&paths.timers)
    });
    self.report_save(saved);
  }

  pub fn settings(&self) -> Settings {
    self.settings.borrow().clone()
  }

  pub fn update_settings(self: &Rc<Self>, change: impl FnOnce(&mut Settings)) {
    {
      let mut settings = self.settings.borrow_mut();
      change(&mut settings);
      *settings = settings.clone().normalized();
    }
    self
      .sound
      .set_volume(self.settings.borrow().volume_fraction());
    if let Some(source) = self.settings_save.borrow_mut().take() {
      source.remove();
    }
    let app = Rc::downgrade(self);
    let source = glib::timeout_add_local_once(SETTINGS_SAVE_DELAY, move || {
      if let Some(app) = app.upgrade() {
        app.settings_save.borrow_mut().take();
        app.save_settings();
      }
    });
    *self.settings_save.borrow_mut() = Some(source);
    self.refresh();
  }

  fn save_settings(self: &Rc<Self>) {
    let saved = self
      .paths
      .as_ref()
      .map(|paths| self.settings.borrow().save(&paths.settings));
    self.report_save(saved);
  }

  fn report_save(self: &Rc<Self>, result: Option<Result<(), StoreError>>) {
    if let Some(Err(error)) = result {
      log::error!("{error}");
      self.set_notice(Notice::SaveFailed(error.to_string()));
    }
  }

  fn shutdown(self: &Rc<Self>) {
    if self.settings_save.borrow_mut().take().is_some() {
      self.save_settings();
    }
    self.save_timers();
    self.sound.stop();
  }

  // Start with the session.

  pub fn autostart(&self) -> Option<bool> {
    self.autostart.get()
  }

  fn refresh_autostart(self: &Rc<Self>) {
    let app = Rc::downgrade(self);
    autostart::query(move |enabled| {
      if let Some(app) = app.upgrade() {
        app.autostart.set(enabled);
        app.refresh();
      }
    });
  }

  pub fn set_autostart(self: &Rc<Self>, enabled: bool) {
    if self.autostart.get() == Some(enabled) {
      return;
    }
    let app = Rc::downgrade(self);
    autostart::set(enabled, move |result| {
      if let Some(app) = app.upgrade() {
        if let Err(error) = result {
          log::error!("{error}");
          app.set_notice(Notice::AutostartFailed(error));
        }
        app.refresh_autostart();
      }
    });
  }

  // Window.

  /// Shows the popup next to the taskbar; `anchor` is the click position the
  /// taskbar passed, if any.
  pub fn present(self: &Rc<Self>, anchor: Option<PopupAnchor>, page: Option<Page>) {
    self
      .pending_presentations
      .set(self.pending_presentations.get() + 1);
    let hold = self.application.hold();
    let app = Rc::clone(self);
    close_competing_surfaces(PopupApp::Clock, move || {
      let _hold = hold;
      app
        .pending_presentations
        .set(app.pending_presentations.get() - 1);
      if let Some(window) = app.window.get() {
        if let Some(page) = page {
          window.show_page(page);
        }
        window.present(anchor);
      }
      app.refresh();
      app.schedule_heartbeat();
    });
  }

  fn toggle_window(self: &Rc<Self>, anchor: Option<PopupAnchor>, page: Option<Page>) {
    let Some(window) = self.window.get() else {
      return;
    };
    if window.window.is_visible() {
      self.close_window();
    } else if !window.reopen_suppressed() {
      // A click on the clock that dismissed the popup by moving the focus
      // away is the same click: it must not reopen the popup.
      self.present(anchor, page);
    }
  }

  /// Hides the popup. The process quits unless alarms or timers need it.
  pub fn close_window(self: &Rc<Self>) {
    if let Some(window) = self.window.get() {
      window.hide();
    }
    self.quit_if_idle();
  }

  fn quit_if_idle(self: &Rc<Self>) {
    if self.pending_presentations.get() != 0 {
      return;
    }
    if !self.launched.get() {
      return;
    }
    if self.window_visible() || !self.ringing.borrow().is_empty() {
      return;
    }
    let background_allowed =
      self.settings.borrow().keep_in_background || self.started_in_background.get();
    if !(background_allowed && self.has_pending_work()) {
      self.application.quit();
    }
  }

  /// Switches page; the popup is only shown (at its last place) when hidden.
  pub fn show_page(self: &Rc<Self>, page: Page) {
    let visible = self.window_visible();
    if let Some(window) = self.window.get() {
      window.show_page(page);
    }
    if visible {
      self.refresh();
    } else {
      self.present(None, None);
    }
  }

  /// Runs `with_dialog` while the popup is hidden, then shows the popup
  /// again. A dialog is a normal window, which a layer-shell overlay would
  /// cover, and opening it would also move the focus away from the popup.
  pub fn while_popup_hidden(self: &Rc<Self>, with_dialog: impl FnOnce(Box<dyn FnOnce()>)) {
    if let Some(window) = self.window.get() {
      window.hide();
    }
    let app = Rc::downgrade(self);
    with_dialog(Box::new(move || {
      if let Some(app) = app.upgrade() {
        app.present(None, None);
      }
    }));
  }

  pub fn refresh(self: &Rc<Self>) {
    if let Some(window) = self.window.get() {
      window.update(self);
    }
  }

  /// Updates only the parts that change every second.
  fn refresh_live(self: &Rc<Self>) {
    if let Some(window) = self
      .window
      .get()
      .filter(|window| window.window.is_visible())
    {
      window.update_live(self);
    }
  }

  // Notices.

  fn set_notice(self: &Rc<Self>, notice: Notice) {
    *self.notice.borrow_mut() = Some(notice);
    self.refresh();
  }

  pub fn dismiss_notice(self: &Rc<Self>) {
    self.notice.borrow_mut().take();
    self.refresh();
  }

  fn dismiss_timers_notice(&self) {
    let mut notice = self.notice.borrow_mut();
    if *notice == Some(Notice::TimersRestored) {
      *notice = None;
    }
  }

  pub fn notice_text(&self) -> Option<(String, bool)> {
    let notice = self.notice.borrow();
    Some(match notice.as_ref()? {
      Notice::TimersRestored => (self.texts.tr("notice.timers_restored"), false),
      Notice::SoundFailed(error) => (
        self
          .texts
          .tr_args("notice.sound_failed", [("error", error)]),
        true,
      ),
      Notice::CustomSoundMissing => (self.texts.tr("notice.custom_sound_missing"), true),
      Notice::SaveFailed(error) => (
        self.texts.tr_args("notice.save_failed", [("error", error)]),
        true,
      ),
      Notice::Recovered(path) => (
        self
          .texts
          .tr_args("notice.recovered", [("path", path.display().to_string())]),
        true,
      ),
      Notice::AutostartFailed(error) => (
        self
          .texts
          .tr_args("notice.autostart_failed", [("error", error)]),
        true,
      ),
    })
  }
}

impl From<PageArg> for Page {
  fn from(page: PageArg) -> Self {
    match page {
      PageArg::Clock => Self::Clock,
      PageArg::World => Self::World,
      PageArg::Stopwatch => Self::Stopwatch,
      PageArg::Timer => Self::Timer,
      PageArg::Alarms => Self::Alarms,
    }
  }
}

fn notification_id(source: Source) -> String {
  match source {
    Source::Alarm(id) => format!("ring-alarm-{}", id.0),
    Source::Timer => "ring-timer".to_owned(),
  }
}

fn load_or_default<T: Default>(
  loaded: Option<Result<Loaded<T>, StoreError>>,
  notice: &mut Option<Notice>,
) -> T {
  match loaded {
    Some(Ok(loaded)) => {
      if let Some(backup) = loaded.recovered_from {
        *notice = Some(Notice::Recovered(backup));
      }
      loaded.value
    }
    Some(Err(error)) => {
      log::error!("{error}");
      *notice = Some(Notice::SaveFailed(error.to_string()));
      T::default()
    }
    None => T::default(),
  }
}
