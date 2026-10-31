//! The GTK application: lifecycle, timers, persistence, notifications and
//! sounds around the [`Engine`].
//!
//! Lifecycle:
//! - Closing the window while a cycle is under way only hides it; the session
//!   keeps running and notifies when a phase ends. With nothing under way,
//!   closing quits.
//! - The session is checkpointed on every change and every
//!   [`CHECKPOINT_INTERVAL`] while a phase runs. After the process ends (logout,
//!   crash, `Ctrl+Q`), the next launch restores it paused.
//! - The display is refreshed when the shown second changes, not on a fixed
//!   tick, and only while a phase runs.

use std::cell::{Cell, OnceCell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use argvus_taskbar_popup::{PopupAnchor, PopupApp, close_competing_surfaces};
use argvus_time_core::duration::rounded_minutes;
use argvus_time_core::focus_status::{self, FocusStatus};
use argvus_time_core::pomodoro::{Mode, Phase};
use argvus_time_core::store::{AppDirs, Loaded, StoreError};
use argvus_time_core::ticks::Ticks;
use argvus_time_ui::sound::{self, Playback, SoundPlayer};
use argvus_time_ui::text::{Calendar, Catalog};
use argvus_time_ui::theme::ThemeStyle;
use chrono::Utc;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::cli::Cli;
use crate::engine::{Engine, Event, PauseReason, SNAPSHOT_FILE_NAME, Snapshot, Status, View};
use crate::history::{self, DaySummary, History};
use crate::settings::{self, Settings};
use crate::ui::{FocusWindow, Page};

pub const APP_ID: &str = "sh.argvus.Focus";
/// Translation domain of the application strings.
const DOMAIN: &str = "focus";
/// How often a running phase is written to disk.
pub const CHECKPOINT_INTERVAL: Duration = Duration::from_secs(30);
/// Wake slightly after the second changes, so the display never lags.
const TICK_SLACK: Duration = Duration::from_millis(20);
/// Settings changes are written once the user stops adjusting.
const SETTINGS_SAVE_DELAY: Duration = Duration::from_millis(400);
const NOTIFICATION_ID: &str = "phase";
const APP_CSS: &str = include_str!("ui/focus.css");

type ActionHandler = fn(&Rc<FocusApp>);

/// A message shown above the controls until the user dismisses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
  Restored,
  Suspended { minutes: u64 },
  SoundFailed(String),
  SaveFailed(String),
  Recovered(PathBuf),
}

struct Paths {
  /// Status file for the taskbar indicator, in the runtime directory.
  status: Option<PathBuf>,
  settings: PathBuf,
  history: PathBuf,
  session: PathBuf,
}

pub struct FocusApp {
  pub application: gtk::Application,
  pub texts: Catalog,
  pub calendar: Calendar,
  paths: Option<Paths>,
  settings: RefCell<Settings>,
  history: RefCell<History>,
  engine: RefCell<Engine>,
  last_ticks: Cell<Ticks>,
  last_checkpoint: Cell<Duration>,
  tick_source: RefCell<Option<glib::SourceId>>,
  settings_save: RefCell<Option<glib::SourceId>>,
  sound: SoundPlayer,
  notice: RefCell<Option<Notice>>,
  theme: OnceCell<Rc<ThemeStyle>>,
  pending_presentations: Cell<usize>,
  window: OnceCell<FocusWindow>,
}

pub fn run() -> glib::ExitCode {
  let application = gtk::Application::builder()
    .application_id(APP_ID)
    .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
    .build();
  let state: Rc<OnceCell<Rc<FocusApp>>> = Rc::default();

  application.connect_startup({
    let state = Rc::clone(&state);
    move |application| {
      let app = FocusApp::new(application);
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
        app.present(None);
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

impl FocusApp {
  fn new(application: &gtk::Application) -> Rc<Self> {
    let mut notice = None;
    let paths = match AppDirs::for_app("focus") {
      Ok(directories) => Some(Paths {
        status: focus_status::status_path(),
        settings: directories.config.join(settings::FILE_NAME),
        history: directories.data.join(history::FILE_NAME),
        session: directories.state.join(SNAPSHOT_FILE_NAME),
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
    let history = load_or_default(
      paths.as_ref().map(|paths| History::load(&paths.history)),
      &mut notice,
    );
    let snapshot = paths
      .as_ref()
      .and_then(|paths| {
        Snapshot::load(&paths.session)
          .inspect_err(|error| log::error!("{error}"))
          .ok()
      })
      .flatten();
    let engine = match snapshot {
      Some(snapshot) => Engine::restore(&snapshot, settings.pomodoro),
      None => Engine::new(settings.pomodoro, settings.mode),
    };
    if engine.status() == Status::Paused(PauseReason::Restored) {
      notice = Some(Notice::Restored);
    }

    Rc::new(Self {
      application: application.clone(),
      texts: Catalog::load(DOMAIN),
      calendar: Calendar::load(),
      paths,
      settings: RefCell::new(settings),
      history: RefCell::new(history),
      engine: RefCell::new(engine),
      last_ticks: Cell::new(Ticks::now()),
      last_checkpoint: Cell::new(Duration::ZERO),
      tick_source: RefCell::new(None),
      settings_save: RefCell::new(None),
      sound: SoundPlayer::new(),
      notice: RefCell::new(notice),
      theme: OnceCell::new(),
      pending_presentations: Cell::new(0),
      window: OnceCell::new(),
    })
  }

  /// Styles, actions, shortcuts and the (hidden) window.
  fn install(self: &Rc<Self>) {
    let _ = self.theme.set(ThemeStyle::install(APP_CSS));
    self.install_actions();
    let window = FocusWindow::build(self);
    if let Some(theme) = self.theme.get() {
      theme.watch(&window.window);
    }
    let _ = self.window.set(window);
    self.write_status();
    self.refresh();
  }

  fn install_actions(self: &Rc<Self>) {
    let actions: [(&str, &[&str], ActionHandler); 10] = [
      ("start", &[], |app| app.start()),
      ("pause", &[], |app| app.pause()),
      ("toggle", &[], |app| app.toggle_session()),
      ("complete", &[], |app| app.complete()),
      ("cancel", &[], |app| app.cancel()),
      ("skip-break", &[], |app| app.skip_break()),
      ("show", &[], |app| app.present(None)),
      ("settings", &["<Control>comma"], |app| {
        app.show_page(Page::Settings)
      }),
      ("history", &["<Control>h"], |app| {
        app.show_page(Page::History)
      }),
      ("close", &["<Control>w"], |app| app.close_window()),
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
    // `main` already rejected invalid arguments in the launching process, so
    // a failure here only comes from a different binary version.
    let cli = match Cli::parse_arguments(command_line.arguments()) {
      Ok(cli) => cli,
      Err(error) => {
        log::error!("ignoring invalid arguments: {error}");
        return glib::ExitCode::FAILURE;
      }
    };
    if cli.start {
      self.start();
    }
    if cli.pause {
      self.pause();
    }
    let anchor = cli.anchor();
    if cli.toggle {
      self.toggle_window(anchor);
    } else if !cli.start && !cli.pause {
      self.present(anchor);
    }
    self.quit_if_idle();
    glib::ExitCode::SUCCESS
  }

  // Session control.

  pub fn start(self: &Rc<Self>) {
    let now = Ticks::now();
    self.engine.borrow_mut().start(now.boottime, Utc::now());
    self.last_ticks.set(now);
    self.dismiss_session_notice();
    self.application.withdraw_notification(NOTIFICATION_ID);
    self.after_change();
  }

  pub fn pause(self: &Rc<Self>) {
    self.engine.borrow_mut().pause(Ticks::now().boottime);
    self.after_change();
  }

  pub fn toggle_session(self: &Rc<Self>) {
    if self.engine.borrow().is_running() {
      self.pause();
    } else {
      self.start();
    }
  }

  pub fn complete(self: &Rc<Self>) {
    let event = self
      .engine
      .borrow_mut()
      .complete(Ticks::now().boottime, Utc::now());
    if let Some(event) = event {
      self.handle_event(event, false);
    }
    self.after_change();
  }

  pub fn cancel(self: &Rc<Self>) {
    self.engine.borrow_mut().cancel();
    self.dismiss_session_notice();
    self.after_change();
  }

  pub fn skip_break(self: &Rc<Self>) {
    self.engine.borrow_mut().skip_break();
    self.application.withdraw_notification(NOTIFICATION_ID);
    self.after_change();
  }

  pub fn set_mode(self: &Rc<Self>, mode: Mode) {
    if self.engine.borrow_mut().set_mode(mode) {
      self.update_settings(|settings| settings.mode = mode);
    }
    self.refresh();
  }

  /// Publishes the state for the taskbar indicator.
  fn write_status(&self) {
    let Some(path) = self.paths.as_ref().and_then(|paths| paths.status.as_ref()) else {
      return;
    };
    let status: FocusStatus =
      self
        .engine
        .borrow()
        .status_report(Ticks::now().boottime, Utc::now(), std::process::id());
    if let Err(error) = status.save(path) {
      log::warn!("cannot write the status file: {error}");
    }
  }

  fn after_change(self: &Rc<Self>) {
    self.write_status();
    self.save_checkpoint();
    self.schedule_tick();
    self.refresh();
    self.quit_if_idle();
  }

  fn schedule_tick(self: &Rc<Self>) {
    if let Some(source) = self.tick_source.borrow_mut().take() {
      source.remove();
    }
    let engine = self.engine.borrow();
    if !engine.is_running() {
      return;
    }
    let delay = engine.until_next_second(Ticks::now().boottime) + TICK_SLACK;
    let app = Rc::downgrade(self);
    let source = glib::timeout_add_local_once(delay, move || {
      if let Some(app) = app.upgrade() {
        app.on_tick();
      }
    });
    *self.tick_source.borrow_mut() = Some(source);
  }

  fn on_tick(self: &Rc<Self>) {
    // The one-shot source has already been removed by GLib.
    self.tick_source.borrow_mut().take();
    let now = Ticks::now();
    let event = self
      .engine
      .borrow_mut()
      .tick(self.last_ticks.get(), now, Utc::now());
    self.last_ticks.set(now);
    match event {
      Some(event) => {
        self.handle_event(event, true);
        self.write_status();
        self.save_checkpoint();
      }
      None if now.boottime.saturating_sub(self.last_checkpoint.get()) >= CHECKPOINT_INTERVAL => {
        self.save_checkpoint();
      }
      None => {}
    }
    self.schedule_tick();
    self.refresh();
    self.quit_if_idle();
  }

  /// `alert` is false when the user ended the phase, so no notification or
  /// sound is needed.
  fn handle_event(self: &Rc<Self>, event: Event, alert: bool) {
    match event {
      Event::PhaseFinished {
        finished,
        next,
        record,
        started_next,
      } => {
        if let Some(record) = record {
          let saved = {
            let mut history = self.history.borrow_mut();
            history.record(record, Utc::now());
            self
              .paths
              .as_ref()
              .map(|paths| history.save(&paths.history))
          };
          self.report_save(saved);
        }
        if alert {
          self.alert_phase_end(finished, next, started_next);
        }
        if started_next {
          self.last_ticks.set(Ticks::now());
        }
      }
      Event::PausedBySuspend { asleep } => {
        self.set_notice(Notice::Suspended {
          minutes: rounded_minutes(asleep).max(1),
        });
      }
    }
  }

  fn alert_phase_end(self: &Rc<Self>, finished: Phase, next: Phase, started_next: bool) {
    let settings = *self.settings.borrow();
    if settings.notifications {
      let notification = self.phase_notification(finished, next, started_next, &settings);
      self
        .application
        .send_notification(Some(NOTIFICATION_ID), &notification);
    }
    if settings.sounds {
      self.play_bell(settings.volume_fraction());
    }
  }

  fn phase_notification(
    &self,
    finished: Phase,
    next: Phase,
    started_next: bool,
    settings: &Settings,
  ) -> gio::Notification {
    let mode = self.engine.borrow().mode();
    let minutes = |phase| (settings.pomodoro.duration(mode, phase).as_secs() / 60).to_string();
    let (title, body, button) = match (mode, finished, next) {
      (Mode::Single, ..) => (
        self.texts.tr("notify.focus_done.title"),
        self.texts.tr_args(
          "notify.single_done.body",
          [("minutes", minutes(Phase::Focus))],
        ),
        "action.start_focus",
      ),
      (_, Phase::Focus, Phase::LongBreak) => (
        self.texts.tr("notify.focus_done.title"),
        self
          .texts
          .tr_args("notify.focus_done.long_break", [("minutes", minutes(next))]),
        "action.start_break",
      ),
      (_, Phase::Focus, _) => (
        self.texts.tr("notify.focus_done.title"),
        self.texts.tr_args(
          "notify.focus_done.short_break",
          [("minutes", minutes(next))],
        ),
        "action.start_break",
      ),
      _ => (
        self.texts.tr("notify.break_done.title"),
        self.texts.tr("notify.break_done.body"),
        "action.start_focus",
      ),
    };
    let notification = gio::Notification::new(&title);
    notification.set_body(Some(&body));
    notification.set_icon(&gio::ThemedIcon::new(APP_ID));
    notification.set_priority(gio::NotificationPriority::High);
    notification.set_default_action("app.show");
    if !started_next {
      notification.add_button(&self.texts.tr(button), "app.start");
    }
    notification
  }

  fn play_bell(self: &Rc<Self>, volume: f64) {
    let app = Rc::downgrade(self);
    let on_failure = move |error: &sound::SoundError| {
      if let Some(app) = app.upgrade() {
        app.set_notice(Notice::SoundFailed(error.to_string()));
      }
    };
    let path = sound::bundled_sound(sound::BELL_FILE);
    if let Err(error) = self.sound.play(&path, Playback::Once, volume, on_failure) {
      log::error!("{error}");
      self.set_notice(Notice::SoundFailed(error.to_string()));
    }
  }

  pub fn test_sound(self: &Rc<Self>) {
    let volume = self.settings.borrow().volume_fraction();
    self.play_bell(volume);
  }

  // Persistence.

  fn save_checkpoint(self: &Rc<Self>) {
    let now = Ticks::now().boottime;
    self.last_checkpoint.set(now);
    let saved = self
      .paths
      .as_ref()
      .map(|paths| self.engine.borrow().snapshot(now).save(&paths.session));
    self.report_save(saved);
  }

  /// Applies a settings change at once and writes it shortly after.
  pub fn update_settings(self: &Rc<Self>, change: impl FnOnce(&mut Settings)) {
    let config = {
      let mut settings = self.settings.borrow_mut();
      change(&mut settings);
      *settings = settings.normalized();
      settings.pomodoro
    };
    self.engine.borrow_mut().set_config(config);
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

  pub fn clear_history(self: &Rc<Self>) {
    let saved = {
      let mut history = self.history.borrow_mut();
      history.clear();
      self
        .paths
        .as_ref()
        .map(|paths| history.save(&paths.history))
    };
    self.report_save(saved);
    self.refresh();
  }

  fn report_save(self: &Rc<Self>, result: Option<Result<(), StoreError>>) {
    if let Some(Err(error)) = result {
      log::error!("{error}");
      self.set_notice(Notice::SaveFailed(error.to_string()));
    }
  }

  fn shutdown(self: &Rc<Self>) {
    if self.settings_save.borrow_mut().take().is_some() {
      // The pending source dies with the main loop; write now instead.
      self.save_settings();
    }
    self.save_checkpoint();
    self.sound.stop();
    if let Some(path) = self.paths.as_ref().and_then(|paths| paths.status.as_ref()) {
      FocusStatus::remove(path);
    }
  }

  // Window.

  /// Shows the popup next to the taskbar; `anchor` is the click position the
  /// taskbar passed, if any.
  pub fn present(self: &Rc<Self>, anchor: Option<PopupAnchor>) {
    self
      .pending_presentations
      .set(self.pending_presentations.get() + 1);
    let hold = self.application.hold();
    let app = Rc::clone(self);
    close_competing_surfaces(PopupApp::Focus, move || {
      let _hold = hold;
      app
        .pending_presentations
        .set(app.pending_presentations.get() - 1);
      if let Some(window) = app.window.get() {
        window.present(anchor);
      }
      app.refresh();
    });
  }

  fn toggle_window(self: &Rc<Self>, anchor: Option<PopupAnchor>) {
    let Some(window) = self.window.get() else {
      return;
    };
    if window.window.is_visible() {
      self.close_window();
    } else if !window.reopen_suppressed() {
      // A click on the icon that dismissed the popup by moving the focus
      // away is the same click: it must not reopen the popup.
      self.present(anchor);
    }
  }

  /// Hides the popup. The process keeps running while a cycle is under way and
  /// quits otherwise.
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
    let hidden = self
      .window
      .get()
      .is_none_or(|window| !window.window.is_visible());
    if hidden && !self.engine.borrow().is_active() {
      self.application.quit();
    }
  }

  pub fn show_page(self: &Rc<Self>, page: Page) {
    let visible = self
      .window
      .get()
      .is_some_and(|window| window.window.is_visible());
    if let Some(window) = self.window.get() {
      window.show_page(page);
    }
    if visible {
      self.refresh();
    } else {
      self.present(None);
    }
  }

  pub fn refresh(self: &Rc<Self>) {
    if let Some(window) = self.window.get() {
      window.update(self);
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

  /// Restored and suspended notices describe the paused phase; they go away
  /// when the user acts on it.
  fn dismiss_session_notice(&self) {
    let mut notice = self.notice.borrow_mut();
    if matches!(*notice, Some(Notice::Restored | Notice::Suspended { .. })) {
      *notice = None;
    }
  }

  pub fn notice_text(&self) -> Option<(String, bool)> {
    let notice = self.notice.borrow();
    let text = match notice.as_ref()? {
      Notice::Restored => (self.texts.tr("notice.restored"), false),
      Notice::Suspended { minutes } => (
        self
          .texts
          .tr_args("notice.suspended", [("minutes", minutes.to_string())]),
        false,
      ),
      Notice::SoundFailed(error) => (
        self
          .texts
          .tr_args("notice.sound_failed", [("error", error)]),
        true,
      ),
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
    };
    Some(text)
  }

  // Read access for the window.

  pub fn view(&self) -> View {
    self.engine.borrow().view(Ticks::now().boottime)
  }

  pub fn settings(&self) -> Settings {
    *self.settings.borrow()
  }

  pub fn today(&self) -> DaySummary {
    self
      .history
      .borrow()
      .day_summary(Utc::now(), &chrono::Local)
  }

  pub fn history(&self) -> std::cell::Ref<'_, History> {
    self.history.borrow()
  }
}

/// The loaded value, or the defaults when there is no place to load from or
/// the file could not be read. Problems become a notice.
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
