use argvus_taskbar_popup::morphing::Server;
use argvus_time_core::duration::format_remaining;
use argvus_time_core::pomodoro::{MAX_PHASE_MINUTES, MAX_SESSIONS_BEFORE_LONG_BREAK, PRESETS};
use argvus_time_ui::text::HourCycle;
use chrono::Local;
use serde::Deserialize;
use serde_json::{Value, json};

use super::*;

const STARTUP_GRACE: Duration = Duration::from_secs(5);
const LABELS: &[&str] = &[
  "action.cancel",
  "action.clear_history",
  "action.complete",
  "action.dismiss",
  "action.history",
  "action.pause",
  "action.resume",
  "action.settings",
  "action.skip_break",
  "action.start_break",
  "action.start_focus",
  "action.test_sound",
  "a11y.ring",
  "history.clear.confirm",
  "history.clear.detail",
  "history.empty",
  "history.recent_title",
  "mode.pomodoro",
  "mode.single",
  "phase.focus",
  "phase.long_break",
  "phase.short_break",
  "preset.classic",
  "preset.custom",
  "preset.deep",
  "preset.extended",
  "settings.auto_start_breaks",
  "settings.auto_start_focus",
  "settings.focus_minutes",
  "settings.long_break_minutes",
  "settings.notifications",
  "settings.preset",
  "settings.section.alerts",
  "settings.section.cycle",
  "settings.section.durations",
  "settings.sessions_before_long_break",
  "settings.short_break_minutes",
  "settings.single_minutes",
  "settings.sounds",
  "settings.volume",
  "status.paused",
  "status.ready",
  "status.running",
  "status.up_next",
];

#[derive(Deserialize)]
#[serde(
  tag = "field",
  content = "value",
  rename_all = "snake_case",
  deny_unknown_fields
)]
enum Preference {
  Preset(String),
  FocusMinutes(u32),
  ShortBreakMinutes(u32),
  LongBreakMinutes(u32),
  SingleSessionMinutes(u32),
  SessionsBeforeLongBreak(u32),
  AutoStartBreaks(bool),
  AutoStartFocus(bool),
  Notifications(bool),
  Sounds(bool),
  Volume(u8),
}

impl Preference {
  fn validate(&self) -> bool {
    match self {
      Self::Preset(id) => PRESETS.iter().any(|preset| preset.id == id),
      Self::FocusMinutes(value)
      | Self::ShortBreakMinutes(value)
      | Self::LongBreakMinutes(value)
      | Self::SingleSessionMinutes(value) => (1..=MAX_PHASE_MINUTES).contains(value),
      Self::SessionsBeforeLongBreak(value) => (1..=MAX_SESSIONS_BEFORE_LONG_BREAK).contains(value),
      Self::Volume(value) => *value <= 100,
      _ => true,
    }
  }

  fn apply(self, settings: &mut Settings) {
    let config = &mut settings.pomodoro;
    match self {
      Self::Preset(id) => {
        if let Some(preset) = PRESETS.into_iter().find(|preset| preset.id == id) {
          *config = config.with_preset(preset);
        }
      }
      Self::FocusMinutes(value) => config.focus_minutes = value,
      Self::ShortBreakMinutes(value) => config.short_break_minutes = value,
      Self::LongBreakMinutes(value) => config.long_break_minutes = value,
      Self::SingleSessionMinutes(value) => config.single_session_minutes = value,
      Self::SessionsBeforeLongBreak(value) => config.sessions_before_long_break = value,
      Self::AutoStartBreaks(value) => config.auto_start_breaks = value,
      Self::AutoStartFocus(value) => config.auto_start_focus = value,
      Self::Notifications(value) => settings.notifications = value,
      Self::Sounds(value) => settings.sounds = value,
      Self::Volume(value) => settings.volume = value,
    }
  }
}

impl FocusApp {
  pub(super) fn start_morphing(self: &Rc<Self>) -> glib::ExitCode {
    if self.morphing_server.borrow().is_none() {
      let opened = Rc::downgrade(self);
      let operation = Rc::downgrade(self);
      let server = Server::start(
        "focus",
        move || {
          let app = opened.upgrade()?;
          app.morphing_clients.set(app.morphing_clients.get() + 1);
          app.morphing_starting.set(false);
          let hold = app.application.hold();
          if let Some(window) = app.window.get() {
            window.hide();
          }
          Some(Box::new(move || {
            let _hold = hold;
            app.morphing_clients.set(app.morphing_clients.get() - 1);
            app.quit_if_idle();
          }))
        },
        move |action, args| {
          operation
            .upgrade()
            .ok_or_else(|| "unavailable".to_string())?
            .morphing_request(action, args)
        },
      );
      match server {
        Ok(server) => {
          *self.morphing_server.borrow_mut() = Some(server);
        }
        Err(error) => {
          log::warn!("cannot start Focus Morphing session: {error}");
          self.quit_if_idle();
          return glib::ExitCode::FAILURE;
        }
      }
    }
    self.morphing_starting.set(true);
    let app = Rc::downgrade(self);
    let hold = self.application.hold();
    glib::timeout_add_local_once(STARTUP_GRACE, move || {
      let _hold = hold;
      if let Some(app) = app.upgrade() {
        app.morphing_starting.set(false);
        app.quit_if_idle();
      }
    });
    glib::ExitCode::SUCCESS
  }

  fn morphing_request(self: &Rc<Self>, action: &str, args: Value) -> Result<Value, String> {
    match action {
      "state" => return Ok(self.morphing_state()),
      "toggle" => self.toggle_session(),
      "complete" => self.complete(),
      "cancel" => self.cancel(),
      "skip_break" => self.skip_break(),
      "dismiss_notice" => self.dismiss_notice(),
      "test_sound" => self.test_sound(),
      "clear_history" if args == Value::Bool(true) => self.clear_history(),
      "mode" => {
        let mode = serde_json::from_value(args).map_err(|_| "invalid_mode")?;
        self.set_mode(mode);
      }
      "preference" => {
        let preference: Preference =
          serde_json::from_value(args).map_err(|_| "invalid_preference")?;
        if !preference.validate() {
          return Err("invalid_preference".into());
        }
        self.update_settings(|settings| preference.apply(settings));
      }
      "quit" => {
        let application = self.application.clone();
        glib::idle_add_local_once(move || application.quit());
        return Ok(Value::Null);
      }
      _ => return Err("invalid_action".into()),
    }
    Ok(self.morphing_state())
  }

  fn morphing_state(&self) -> Value {
    let view = self.view();
    let settings = self.settings();
    let today = self.today();
    let history = self.history();
    let texts = &self.texts;
    let labels: serde_json::Map<_, _> = LABELS
      .iter()
      .map(|key| ((*key).into(), texts.tr(key).into()))
      .collect();
    let today = texts.tr_args(
      "today.summary",
      [
        (
          "sessions",
          texts.tr_plural("today.sessions", i64::from(today.sessions)),
        ),
        ("minutes", rounded_minutes(today.focused).to_string()),
      ],
    );
    let total_minutes = rounded_minutes(history.total_focused());
    let total = texts.tr_args(
      "history.total",
      [
        (
          "sessions",
          texts.tr_plural(
            "history.sessions",
            i64::try_from(history.total_sessions()).unwrap_or(i64::MAX),
          ),
        ),
        ("hours", (total_minutes / 60).to_string()),
        ("minutes", (total_minutes % 60).to_string()),
      ],
    );
    let recent: Vec<_> = history
      .recent(20)
      .map(|session| {
        let started = session.started_at.with_timezone(&Local);
        let ended = session.ended_at.with_timezone(&Local);
        let date = self.calendar.short_date(&started);
        let start = self.calendar.time(&started, HourCycle::H24, false).joined();
        let end = self.calendar.time(&ended, HourCycle::H24, false).joined();
        let length = texts.tr_args(
          "history.minutes",
          [("minutes", rounded_minutes(session.focused()).to_string())],
        );
        json!({
          "when": format!("{date} · {start}–{end}"),
          "length": length,
          "early": session.completed_early,
        })
      })
      .collect();
    let (done, length) = view.cycle;
    let cycle = if view.mode == Mode::Single {
      texts.tr("status.single")
    } else {
      texts.tr_args(
        "status.cycle",
        [
          ("current", (done + 1).min(length).to_string()),
          ("total", length.to_string()),
        ],
      )
    };
    let status = match view.status {
      Status::Ready => "ready",
      Status::Running => "running",
      Status::Paused(_) => "paused",
      Status::AwaitingNext { .. } => "up_next",
    };
    json!({
      "labels": labels,
      "rtl": texts.is_rtl(),
      "settings": settings,
      "preset": settings.pomodoro.matching_preset().map_or("custom", |preset| preset.id),
      "status": status,
      "mode": view.mode,
      "phase": view.phase,
      "remaining": format_remaining(view.remaining),
      "progress": view.progress,
      "cycle": cycle,
      "done": done,
      "cycle_length": length,
      "today": today,
      "total": total,
      "recent": recent,
      "notice": self.notice_text(),
    })
  }
}
