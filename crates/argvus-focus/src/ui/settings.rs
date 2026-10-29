//! Settings page: durations, presets, cycle, notifications and sounds.

use std::cell::Cell;
use std::rc::Rc;

use argvus_time_core::pomodoro::{
  MAX_PHASE_MINUTES, MAX_SESSIONS_BEFORE_LONG_BREAK, MIN_PHASE_MINUTES, PRESETS,
};
use gtk::prelude::*;

use super::{labeled_row, section_title};
use crate::app::FocusApp;
use crate::settings::Settings;

type NumberSetter = fn(&mut Settings, u32);
type FlagSetter = fn(&mut Settings, bool);

pub struct SettingsPage {
  pub root: gtk::ScrolledWindow,
  preset: gtk::DropDown,
  focus: gtk::SpinButton,
  short_break: gtk::SpinButton,
  long_break: gtk::SpinButton,
  sessions: gtk::SpinButton,
  single: gtk::SpinButton,
  auto_breaks: gtk::Switch,
  auto_focus: gtk::Switch,
  notifications: gtk::Switch,
  sounds: gtk::Switch,
  volume: gtk::Scale,
  updating: Rc<Cell<bool>>,
}

fn minutes_spin(max: u32) -> gtk::SpinButton {
  let spin = gtk::SpinButton::with_range(f64::from(MIN_PHASE_MINUTES), f64::from(max), 1.0);
  spin.set_numeric(true);
  spin
}

fn spin_value(spin: &gtk::SpinButton) -> u32 {
  u32::try_from(spin.value_as_int()).unwrap_or(MIN_PHASE_MINUTES)
}

impl SettingsPage {
  pub fn build(app: &Rc<FocusApp>) -> Self {
    let texts = &app.texts;
    let content = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(10)
      .margin_top(14)
      .margin_bottom(18)
      .margin_start(18)
      .margin_end(18)
      .css_classes(["page"])
      .build();

    let mut preset_names: Vec<String> = PRESETS
      .iter()
      .map(|preset| texts.tr(&format!("preset.{}", preset.id)))
      .collect();
    preset_names.push(texts.tr("preset.custom"));
    let preset_refs: Vec<&str> = preset_names.iter().map(String::as_str).collect();
    let preset = gtk::DropDown::from_strings(&preset_refs);

    let page = Self {
      root: gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&content)
        .css_classes(["page"])
        .build(),
      preset,
      focus: minutes_spin(MAX_PHASE_MINUTES),
      short_break: minutes_spin(MAX_PHASE_MINUTES),
      long_break: minutes_spin(MAX_PHASE_MINUTES),
      sessions: minutes_spin(MAX_SESSIONS_BEFORE_LONG_BREAK),
      single: minutes_spin(MAX_PHASE_MINUTES),
      auto_breaks: gtk::Switch::new(),
      auto_focus: gtk::Switch::new(),
      notifications: gtk::Switch::new(),
      sounds: gtk::Switch::new(),
      volume: gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 5.0),
      updating: Rc::new(Cell::new(false)),
    };
    page.volume.set_hexpand(true);
    page.volume.set_width_request(160);

    let test_sound = gtk::Button::with_label(&texts.tr("action.test_sound"));
    let durations = card([
      labeled_row(&texts.tr("settings.preset"), &page.preset),
      labeled_row(&texts.tr("settings.focus_minutes"), &page.focus),
      labeled_row(&texts.tr("settings.short_break_minutes"), &page.short_break),
      labeled_row(&texts.tr("settings.long_break_minutes"), &page.long_break),
      labeled_row(&texts.tr("settings.single_minutes"), &page.single),
    ]);
    let cycle = card([
      labeled_row(
        &texts.tr("settings.sessions_before_long_break"),
        &page.sessions,
      ),
      labeled_row(&texts.tr("settings.auto_start_breaks"), &page.auto_breaks),
      labeled_row(&texts.tr("settings.auto_start_focus"), &page.auto_focus),
    ]);
    let alerts = card([
      labeled_row(&texts.tr("settings.notifications"), &page.notifications),
      labeled_row(&texts.tr("settings.sounds"), &page.sounds),
      labeled_row(&texts.tr("settings.volume"), &page.volume),
      labeled_row(&texts.tr("settings.sound_hint"), &test_sound),
    ]);

    content.append(&section_title(&texts.tr("settings.section.durations")));
    content.append(&durations);
    content.append(&section_title(&texts.tr("settings.section.cycle")));
    content.append(&cycle);
    content.append(&section_title(&texts.tr("settings.section.alerts")));
    content.append(&alerts);

    page.connect(app, &test_sound);
    page.update(app);
    page
  }

  fn connect(&self, app: &Rc<FocusApp>, test_sound: &gtk::Button) {
    let spins: [(&gtk::SpinButton, NumberSetter); 5] = [
      (&self.focus, |settings, value| {
        settings.pomodoro.focus_minutes = value
      }),
      (&self.short_break, |settings, value| {
        settings.pomodoro.short_break_minutes = value
      }),
      (&self.long_break, |settings, value| {
        settings.pomodoro.long_break_minutes = value
      }),
      (&self.sessions, |settings, value| {
        settings.pomodoro.sessions_before_long_break = value
      }),
      (&self.single, |settings, value| {
        settings.pomodoro.single_session_minutes = value
      }),
    ];
    for (spin, apply) in spins {
      let app = Rc::downgrade(app);
      let updating = Rc::clone(&self.updating);
      spin.connect_value_changed(move |spin| {
        if !updating.get()
          && let Some(app) = app.upgrade()
        {
          let value = spin_value(spin);
          app.update_settings(|settings| apply(settings, value));
        }
      });
    }

    let switches: [(&gtk::Switch, FlagSetter); 4] = [
      (&self.auto_breaks, |settings, on| {
        settings.pomodoro.auto_start_breaks = on
      }),
      (&self.auto_focus, |settings, on| {
        settings.pomodoro.auto_start_focus = on
      }),
      (&self.notifications, |settings, on| {
        settings.notifications = on
      }),
      (&self.sounds, |settings, on| settings.sounds = on),
    ];
    for (switch, apply) in switches {
      let app = Rc::downgrade(app);
      let updating = Rc::clone(&self.updating);
      switch.connect_active_notify(move |switch| {
        if !updating.get()
          && let Some(app) = app.upgrade()
        {
          let on = switch.is_active();
          app.update_settings(|settings| apply(settings, on));
        }
      });
    }

    let weak = Rc::downgrade(app);
    let updating = Rc::clone(&self.updating);
    self.volume.connect_value_changed(move |scale| {
      if !updating.get()
        && let Some(app) = weak.upgrade()
      {
        let volume = scale.value().round().clamp(0.0, 100.0) as u8;
        app.update_settings(|settings| settings.volume = volume);
      }
    });

    let weak = Rc::downgrade(app);
    let updating = Rc::clone(&self.updating);
    self.preset.connect_selected_notify(move |dropdown| {
      let selected = usize::try_from(dropdown.selected()).unwrap_or(usize::MAX);
      if !updating.get()
        && let (Some(app), Some(preset)) = (weak.upgrade(), PRESETS.get(selected))
      {
        app.update_settings(|settings| settings.pomodoro = settings.pomodoro.with_preset(*preset));
      }
    });

    let weak = Rc::downgrade(app);
    test_sound.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.test_sound();
      }
    });
  }

  pub fn update(&self, app: &FocusApp) {
    let settings = app.settings();
    let config = settings.pomodoro;
    self.updating.set(true);
    let custom_index = u32::try_from(PRESETS.len()).unwrap_or(0);
    let preset_index = config
      .matching_preset()
      .and_then(|preset| {
        PRESETS
          .iter()
          .position(|candidate| candidate.id == preset.id)
      })
      .and_then(|index| u32::try_from(index).ok())
      .unwrap_or(custom_index);
    self.preset.set_selected(preset_index);
    self.focus.set_value(f64::from(config.focus_minutes));
    self
      .short_break
      .set_value(f64::from(config.short_break_minutes));
    self
      .long_break
      .set_value(f64::from(config.long_break_minutes));
    self
      .sessions
      .set_value(f64::from(config.sessions_before_long_break));
    self
      .single
      .set_value(f64::from(config.single_session_minutes));
    self.auto_breaks.set_active(config.auto_start_breaks);
    self.auto_focus.set_active(config.auto_start_focus);
    self.notifications.set_active(settings.notifications);
    self.sounds.set_active(settings.sounds);
    self.volume.set_value(f64::from(settings.volume));
    self.volume.set_sensitive(settings.sounds);
    self.updating.set(false);
  }
}

fn card<const N: usize>(rows: [gtk::Box; N]) -> gtk::Box {
  let card = gtk::Box::builder()
    .orientation(gtk::Orientation::Vertical)
    .spacing(10)
    .css_classes(["card"])
    .build();
  for row in rows {
    card.append(&row);
  }
  card
}
