use argvus_taskbar_popup::morphing::Server;
use argvus_time_core::alarm::{
  AlarmTime, BundledSound, MAX_SNOOZE_MINUTES, MIN_SNOOZE_MINUTES, WeekdaySet,
};
use argvus_time_core::duration::{format_precise, format_remaining};
use argvus_time_core::elapsed::CountdownState;
use argvus_time_core::world::{self, DayRelation, WorldClock};
use argvus_time_ui::text::HourCycle;
use chrono::Datelike;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use super::*;

const STARTUP_GRACE: Duration = Duration::from_secs(5);
const LABELS: &[&str] = &[
  "action.back",
  "action.cancel",
  "action.dismiss",
  "action.pause",
  "action.reset",
  "action.resume",
  "action.start",
  "action.test_sound",
  "alarms.add",
  "alarms.empty",
  "alarms.missed_title",
  "alarms.toggle",
  "editor.delete",
  "editor.hour",
  "editor.minute",
  "editor.name",
  "editor.repeat",
  "editor.rings_once",
  "editor.save",
  "editor.snooze",
  "editor.sound",
  "editor.sound_test",
  "page.add_city",
  "page.alarm_editor",
  "page.alarms",
  "page.clock",
  "page.settings",
  "page.stopwatch",
  "page.timer",
  "page.world",
  "ring.window_title",
  "settings.autostart",
  "settings.autostart_hint",
  "settings.autostart_unavailable",
  "settings.background",
  "settings.hour_12",
  "settings.hour_24",
  "settings.hour_cycle",
  "settings.reliability",
  "settings.ring_limit",
  "settings.section.display",
  "settings.section.running",
  "settings.section.sounds",
  "settings.show_seconds",
  "settings.sound_test",
  "settings.timer_sound",
  "settings.volume",
  "sound.beacon",
  "sound.chime",
  "stopwatch.lap",
  "stopwatch.laps",
  "timer.hours",
  "timer.minutes",
  "timer.presets",
  "timer.progress",
  "timer.restart",
  "timer.seconds",
  "world.add",
  "world.empty",
  "world.move_down",
  "world.move_up",
  "world.remove",
  "world.same_time",
  "world.search_placeholder",
  "world.today",
  "world.tomorrow",
  "world.yesterday",
];

#[derive(Deserialize)]
#[serde(
  tag = "field",
  content = "value",
  rename_all = "snake_case",
  deny_unknown_fields
)]
enum Preference {
  HourCycle(HourCycle),
  ShowSeconds(bool),
  AlarmVolume(u8),
  RingLimitMinutes(u32),
  TimerSound(BundledSound),
  KeepInBackground(bool),
  Autostart(bool),
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Draft {
  id: Option<String>,
  label: String,
  hour: u8,
  minute: u8,
  repeat: [bool; 7],
  sound: Option<BundledSound>,
  snooze: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Enabled {
  id: String,
  enabled: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CityMove {
  zone: String,
  direction: i8,
}

fn parse<T: DeserializeOwned>(args: Value) -> Result<T, String> {
  serde_json::from_value(args).map_err(|_| "invalid_arguments".into())
}

fn alarm_id(id: &str) -> Result<AlarmId, String> {
  id.parse().map(AlarmId).map_err(|_| "invalid_alarm".into())
}

impl ClockApp {
  pub(super) fn start_morphing(self: &Rc<Self>) -> glib::ExitCode {
    if self.morphing_server.borrow().is_none() {
      let opened = Rc::downgrade(self);
      let operation = Rc::downgrade(self);
      let server = Server::start(
        "clock",
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
          log::warn!("cannot start Clock Morphing session: {error}");
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
      "state" => {}
      "timer_start" => {
        let length = Duration::from_secs(parse(args)?);
        if length.is_zero() || length > timers::MAX_TIMER {
          return Err("invalid_duration".into());
        }
        self.start_timer(length);
      }
      "timer_toggle" => self.change_timer(|timer, now| match timer.state(now) {
        CountdownState::Running => timer.pause(now),
        CountdownState::Paused => timer.resume(now),
        _ => {}
      }),
      "timer_restart" => self.change_timer(Timer::restart),
      "timer_cancel" => self.change_timer(|timer, _| timer.cancel()),
      "stopwatch_toggle" => self.change_stopwatch(|watch, now| {
        if watch.is_running() {
          watch.pause(now);
        } else {
          watch.start(now);
        }
      }),
      "stopwatch_lap" => self.change_stopwatch(|watch, now| {
        watch.lap(now);
      }),
      "stopwatch_reset" => self.change_stopwatch(|watch, _| watch.reset()),
      "alarm_preview" => {
        let (_, draft) = self.parse_draft(parse(args)?)?;
        let time =
          chrono::NaiveTime::from_hms_opt(draft.time.hour().into(), draft.time.minute().into(), 0)
            .unwrap_or_default();
        let now = Utc::now();
        let mut book = AlarmBook::default();
        let id = book.add(draft, now);
        let relative = book
          .get(id)
          .and_then(|alarm| alarm.next_occurrence(&Local))
          .map(|at| crate::ui::relative_text(&self.texts, (at - now).to_std().unwrap_or_default()))
          .unwrap_or_default();
        let preview = self.texts.tr_args(
          "editor.preview",
          [
            (
              "time",
              self
                .calendar
                .time(&time, self.settings.borrow().hour_cycle, false)
                .joined(),
            ),
            ("relative", relative),
          ],
        );
        return Ok(json!({ "result": { "preview": preview } }));
      }
      "alarm_save" => {
        let (id, draft) = self.parse_draft(parse(args)?)?;
        self.store_alarm_draft(id, draft)?;
      }
      "alarm_enable" => {
        let Enabled { id, enabled } = parse(args)?;
        let id = alarm_id(&id)?;
        self.set_alarm_enabled(id, enabled);
      }
      "alarm_delete" => {
        let id = alarm_id(&parse::<String>(args)?)?;
        self.dismiss(Source::Alarm(id));
        if self.alarms.borrow_mut().remove(id) {
          self.save_alarms();
          self.refresh();
          self.schedule_heartbeat();
        }
      }
      "alarm_snooze" => self.snooze(alarm_id(&parse::<String>(args)?)?),
      "ring_dismiss" => {
        let id: String = parse(args)?;
        self.dismiss(if id == "timer" {
          Source::Timer
        } else {
          Source::Alarm(alarm_id(&id)?)
        });
      }
      "clear_missed" => self.clear_missed(),
      "dismiss_notice" => self.dismiss_notice(),
      "test_alarm_sound" => {
        let (_, draft) = self.parse_draft(parse(args)?)?;
        self.test_sound(&draft.sound);
      }
      "test_sound" => {
        let sound = parse::<Option<BundledSound>>(args)?.map_or_else(
          || self.settings().timer_sound,
          |sound| SoundChoice::Bundled { sound },
        );
        self.test_sound(&sound);
      }
      "preference" => self.set_morphing_preference(parse(args)?)?,
      "city_search" => {
        let query: String = parse(args)?;
        if query.len() > 256 {
          return Err("invalid_query".into());
        }
        let cities: Vec<Value> = world::search(&query).into_iter().take(60).map(|zone| {
          json!({ "zone": zone.name(), "name": world::city_name(zone.name()), "region": world::region_name(zone.name()) })
        }).collect();
        return Ok(json!({ "result": { "cities": cities } }));
      }
      "city_add" => {
        let zone: String = parse(args)?;
        let zone = world::parse_zone(&zone).ok_or("invalid_zone")?;
        self.update_settings(|settings| {
          if !settings
            .world_clocks
            .iter()
            .any(|clock| clock.zone == zone.name())
          {
            settings.world_clocks.push(WorldClock::new(zone));
          }
        });
      }
      "city_remove" => {
        let zone: String = parse(args)?;
        self.update_settings(|settings| settings.world_clocks.retain(|clock| clock.zone != zone));
      }
      "city_move" => {
        let CityMove { zone, direction } = parse(args)?;
        if !matches!(direction, -1 | 1) {
          return Err("invalid_direction".into());
        }
        self.update_settings(|settings| {
          let clocks = &mut settings.world_clocks;
          if let Some(index) = clocks.iter().position(|clock| clock.zone == zone)
            && let Some(target) = index.checked_add_signed(isize::from(direction))
            && target < clocks.len()
          {
            clocks.swap(index, target);
          }
        });
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

  fn set_morphing_preference(self: &Rc<Self>, preference: Preference) -> Result<(), String> {
    match preference {
      Preference::HourCycle(value) => self.update_settings(|settings| settings.hour_cycle = value),
      Preference::ShowSeconds(value) => {
        self.update_settings(|settings| settings.show_seconds = value)
      }
      Preference::AlarmVolume(value) if (settings::MIN_ALARM_VOLUME..=100).contains(&value) => {
        self.update_settings(|settings| settings.alarm_volume = value);
      }
      Preference::RingLimitMinutes(value) if settings::RING_LIMIT_CHOICES.contains(&value) => {
        self.update_settings(|settings| settings.ring_limit_minutes = value);
      }
      Preference::TimerSound(sound) => self.update_settings(|settings| {
        settings.timer_sound = SoundChoice::Bundled { sound };
      }),
      Preference::KeepInBackground(value) => {
        self.update_settings(|settings| settings.keep_in_background = value)
      }
      Preference::Autostart(value) => self.set_autostart(value),
      _ => return Err("invalid_preference".into()),
    }
    Ok(())
  }

  fn parse_draft(&self, draft: Draft) -> Result<(Option<AlarmId>, AlarmDraft), String> {
    if draft.label.len() > 4096
      || !(MIN_SNOOZE_MINUTES..=MAX_SNOOZE_MINUTES).contains(&draft.snooze)
    {
      return Err("invalid_alarm".into());
    }
    let id = draft.id.as_deref().map(alarm_id).transpose()?;
    let time = AlarmTime::new(draft.hour, draft.minute).ok_or("invalid_time")?;
    let repeat = WeekdaySet::ORDER
      .into_iter()
      .zip(draft.repeat)
      .fold(WeekdaySet::ONCE, |set, (day, included)| {
        set.with(day, included)
      });
    let sound = draft.sound.map_or_else(
      || {
        id.and_then(|id| {
          self
            .alarms
            .borrow()
            .get(id)
            .map(|alarm| alarm.sound.clone())
        })
        .unwrap_or_default()
      },
      |sound| SoundChoice::Bundled { sound },
    );
    Ok((
      id,
      AlarmDraft {
        label: draft.label,
        time,
        repeat,
        sound,
        snooze_minutes: draft.snooze,
      },
    ))
  }

  fn occurrence_text(&self, date: DateTime<Utc>) -> String {
    let local = date.with_timezone(&Local);
    format!(
      "{} {}",
      self.calendar.weekday_short(local.weekday()),
      self.local_time(date)
    )
  }

  fn morphing_state(&self) -> Value {
    use chrono::Timelike as _;

    let settings = self.settings();
    let wall = Utc::now();
    let local = wall.with_timezone(&Local);
    let boot = Ticks::now().boottime;
    let labels: serde_json::Map<_, _> = LABELS
      .iter()
      .map(|key| ((*key).into(), self.texts.tr(key).into()))
      .collect();
    let local_offset = world::utc_offset_seconds(&Local, wall);
    let zone = world::local_zone_name()
      .unwrap_or_else(|| self.texts.tr("clock.local_zone"))
      .replace('_', " ");
    let offset = world::format_utc_offset(local_offset);
    let cities: Vec<Value> = settings.world_clocks.iter().filter_map(|city| {
      let zone = city.time_zone()?;
      let now = wall.with_timezone(&zone);
      let difference = world::utc_offset_seconds(&zone, wall) - local_offset;
      let day = match world::day_relation(local.date_naive(), now.date_naive()) {
        DayRelation::Yesterday => "world.yesterday",
        DayRelation::Today => "world.today",
        DayRelation::Tomorrow => "world.tomorrow",
      };
      Some(json!({
        "zone": city.zone,
        "name": city.display_name(),
        "region": world::region_name(&city.zone),
        "time": self.calendar.time(&now, settings.hour_cycle, false).joined(),
        "relative": format!("{} · {}", self.texts.tr(day), crate::ui::world_offset_text(self, difference)),
      }))
    }).collect();
    let alarms = self.alarms.borrow();
    let next = alarms.next_trigger(&Local).map_or_else(
      || self.texts.tr("clock.no_alarm"),
      |(_, trigger)| {
        let until = (trigger.at - wall).to_std().unwrap_or_default();
        self.texts.tr_args(
          "clock.next_alarm",
          [
            ("when", self.occurrence_text(trigger.at)),
            ("relative", crate::ui::relative_text(&self.texts, until)),
          ],
        )
      },
    );
    let alarm_rows: Vec<_> = alarms
      .alarms()
      .iter()
      .map(|alarm| {
        let draft = Draft {
          id: Some(alarm.id.0.to_string()),
          label: alarm.label.clone(),
          hour: alarm.time.hour(),
          minute: alarm.time.minute(),
          repeat: WeekdaySet::ORDER.map(|day| alarm.repeat.contains(day)),
          sound: match alarm.sound {
            SoundChoice::Bundled { sound } => Some(sound),
            _ => None,
          },
          snooze: alarm.snooze_minutes,
        };
        let time =
          chrono::NaiveTime::from_hms_opt(draft.hour.into(), draft.minute.into(), 0).map(|time| {
            self
              .calendar
              .time(&time, settings.hour_cycle, false)
              .joined()
          });
        let title = if alarm.label.is_empty() {
          self.texts.tr("alarm.default_title")
        } else {
          alarm.label.clone()
        };
        json!({
          "draft": draft,
          "title": title,
          "repeat": crate::ui::alarm_repeat_text(self, alarm.repeat),
          "snoozed": alarm.snoozed_until.map(|_| self.texts.tr("alarms.snoozed")),
          "accessibility": self.texts.tr_args("alarms.toggle", [("title", title.clone())]),
          "enabled": alarm.enabled, "time": time,
          "next": alarm.trigger(&Local).map(|trigger| self.occurrence_text(trigger.at)),
        })
      })
      .collect();
    let ringing: Vec<_> = self.ringing.borrow().entries().iter().map(|entry| {
      let (id, snooze) = match entry.source {
        Source::Timer => ("timer".into(), None),
        Source::Alarm(id) => (id.0.to_string(), self.snooze_minutes(id)),
      };
      json!({
        "id": id, "title": entry.title, "time": self.local_time(entry.due_at),
        "snooze": snooze.map(|minutes| self.texts.tr_args("ring.snooze", [("minutes", minutes.to_string())])),
        "late": entry.late_by.map(|duration| self.late_text(duration)),
      })
    }).collect();
    let missed: Vec<_> = self
      .missed
      .borrow()
      .iter()
      .map(|alarm| {
        self.texts.tr_args(
          "alarms.missed_item",
          [
            ("title", alarm.title.clone()),
            ("time", self.local_time(alarm.due_at)),
          ],
        )
      })
      .collect();
    let timer = self.timer();
    let timer_status = match timer.state(boot) {
      CountdownState::Running => "running",
      CountdownState::Paused => "paused",
      _ => "ready",
    };
    let stopwatch = self.stopwatch();
    let laps: Vec<_> = stopwatch
      .laps()
      .iter()
      .map(|lap| {
        json!({
          "name": self.texts.tr_args("stopwatch.lap_number", [("number", lap.number.to_string())]),
          "split": format_precise(lap.split()), "total": format_precise(lap.total()),
        })
      })
      .collect();
    json!({
      "labels": labels, "rtl": self.texts.is_rtl(), "autostart": self.autostart(),
      "time": self.calendar.time(&local, settings.hour_cycle, settings.show_seconds).joined(),
      "date": self.calendar.long_date(&local), "zone": format!("{zone} · {offset}"), "next": next,
      "settings": settings, "cities": cities, "alarms": alarm_rows, "ringing": ringing, "missed": missed,
      "notice": self.notice_text(), "audio_error": self.sound_failed(),
      "timer": {
        "status": timer_status,
        "remaining": timer.countdown().map(|clock| format_remaining(clock.remaining(boot))),
        "progress": timer.countdown().map(|clock| clock.progress(boot)),
      },
      "stopwatch": {
        "running": stopwatch.is_running(), "started": stopwatch.has_started(boot),
        "time": format_precise(stopwatch.elapsed(boot)),
        "elapsed_ms": u64::try_from(stopwatch.elapsed(boot).as_millis()).unwrap_or(u64::MAX),
        "laps": laps,
      },
      "new_alarm": Draft {
        id: None, label: String::new(), hour: u8::try_from(local.hour()).unwrap_or(0),
        minute: u8::try_from(local.minute()).unwrap_or(0), repeat: [false; 7],
        sound: Some(BundledSound::Chime), snooze: argvus_time_core::alarm::DEFAULT_SNOOZE_MINUTES,
      },
      "weekdays": WeekdaySet::ORDER.map(|day| self.calendar.weekday_short(day)),
      "presets": ([5_u64, 10, 15, 25, 30, 60]).map(|minutes| json!({
        "seconds": minutes * 60,
        "label": self.texts.tr_args("timer.preset", [("minutes", minutes.to_string())]),
      })),
      "ring_limits": settings::RING_LIMIT_CHOICES.map(|minutes| json!({
        "minutes": minutes,
        "label": self.texts.tr_args("settings.ring_limit_choice", [("minutes", minutes.to_string())]),
      })),
      "snooze_max": MAX_SNOOZE_MINUTES,
    })
  }
}
