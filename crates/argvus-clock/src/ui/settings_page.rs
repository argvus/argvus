//! Settings: time format, alarm sound and volume, ring limit, background
//! behaviour and start with the session.

use std::cell::Cell;
use std::rc::Rc;

use argvus_time_core::alarm::{BundledSound, SoundChoice};
use argvus_time_ui::text::HourCycle;
use gtk::prelude::*;

use super::{card, labeled_row, page_box, section_title};
use crate::app::ClockApp;
use crate::settings::{MIN_ALARM_VOLUME, RING_LIMIT_CHOICES};

pub struct SettingsPage {
  pub root: gtk::ScrolledWindow,
  hour_cycle: gtk::DropDown,
  seconds: gtk::Switch,
  volume: gtk::Scale,
  timer_sound: gtk::DropDown,
  ring_limit: gtk::DropDown,
  background: gtk::Switch,
  autostart: gtk::Switch,
  autostart_hint: gtk::Label,
  updating: Rc<Cell<bool>>,
}

impl SettingsPage {
  pub fn build(app: &Rc<ClockApp>) -> Self {
    let texts = &app.texts;
    let content = page_box();
    let hour_names = [texts.tr("settings.hour_24"), texts.tr("settings.hour_12")];
    let hour_cycle =
      gtk::DropDown::from_strings(&hour_names.iter().map(String::as_str).collect::<Vec<_>>());
    let sound_names = [texts.tr("sound.chime"), texts.tr("sound.beacon")];
    let timer_sound =
      gtk::DropDown::from_strings(&sound_names.iter().map(String::as_str).collect::<Vec<_>>());
    let limit_names: Vec<String> = RING_LIMIT_CHOICES
      .iter()
      .map(|minutes| {
        texts.tr_args(
          "settings.ring_limit_choice",
          [("minutes", minutes.to_string())],
        )
      })
      .collect();
    let ring_limit =
      gtk::DropDown::from_strings(&limit_names.iter().map(String::as_str).collect::<Vec<_>>());
    let volume = gtk::Scale::with_range(
      gtk::Orientation::Horizontal,
      f64::from(MIN_ALARM_VOLUME),
      100.0,
      5.0,
    );
    volume.set_width_request(160);
    let test_sound = gtk::Button::with_label(&texts.tr("action.test_sound"));

    let page = Self {
      root: gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&content)
        .css_classes(["page"])
        .build(),
      hour_cycle,
      seconds: gtk::Switch::new(),
      volume,
      timer_sound,
      ring_limit,
      background: gtk::Switch::new(),
      autostart: gtk::Switch::new(),
      autostart_hint: gtk::Label::builder()
        .xalign(0.0)
        .wrap(true)
        .css_classes(["caption"])
        .build(),
      updating: Rc::new(Cell::new(false)),
    };

    let display = card();
    display.append(&labeled_row(
      &texts.tr("settings.hour_cycle"),
      &page.hour_cycle,
    ));
    display.append(&labeled_row(
      &texts.tr("settings.show_seconds"),
      &page.seconds,
    ));
    let sounds = card();
    sounds.append(&labeled_row(&texts.tr("settings.volume"), &page.volume));
    sounds.append(&labeled_row(&texts.tr("settings.sound_test"), &test_sound));
    sounds.append(&labeled_row(
      &texts.tr("settings.timer_sound"),
      &page.timer_sound,
    ));
    sounds.append(&labeled_row(
      &texts.tr("settings.ring_limit"),
      &page.ring_limit,
    ));
    let running = card();
    running.append(&labeled_row(
      &texts.tr("settings.background"),
      &page.background,
    ));
    running.append(&labeled_row(
      &texts.tr("settings.autostart"),
      &page.autostart,
    ));
    running.append(&page.autostart_hint);
    running.append(
      &gtk::Label::builder()
        .label(texts.tr("settings.reliability"))
        .xalign(0.0)
        .wrap(true)
        .css_classes(["caption"])
        .build(),
    );

    content.append(&section_title(&texts.tr("settings.section.display")));
    content.append(&display);
    content.append(&section_title(&texts.tr("settings.section.sounds")));
    content.append(&sounds);
    content.append(&section_title(&texts.tr("settings.section.running")));
    content.append(&running);

    page.connect(app, &test_sound);
    page
  }

  fn connect(&self, app: &Rc<ClockApp>, test_sound: &gtk::Button) {
    let updating = &self.updating;
    let weak = Rc::downgrade(app);
    let guard = Rc::clone(updating);
    self.hour_cycle.connect_selected_notify(move |dropdown| {
      let cycle = if dropdown.selected() == 1 {
        HourCycle::H12
      } else {
        HourCycle::H24
      };
      if let (false, Some(app)) = (guard.get(), weak.upgrade()) {
        app.update_settings(|settings| settings.hour_cycle = cycle);
      }
    });
    let weak = Rc::downgrade(app);
    let guard = Rc::clone(updating);
    self.seconds.connect_active_notify(move |switch| {
      if let (false, Some(app)) = (guard.get(), weak.upgrade()) {
        let on = switch.is_active();
        app.update_settings(|settings| settings.show_seconds = on);
      }
    });
    let weak = Rc::downgrade(app);
    let guard = Rc::clone(updating);
    self.volume.connect_value_changed(move |scale| {
      if let (false, Some(app)) = (guard.get(), weak.upgrade()) {
        let volume = scale.value().round().clamp(0.0, 100.0) as u8;
        app.update_settings(|settings| settings.alarm_volume = volume);
      }
    });
    let weak = Rc::downgrade(app);
    let guard = Rc::clone(updating);
    self.timer_sound.connect_selected_notify(move |dropdown| {
      let sound = if dropdown.selected() == 1 {
        BundledSound::Beacon
      } else {
        BundledSound::Chime
      };
      if let (false, Some(app)) = (guard.get(), weak.upgrade()) {
        app.update_settings(|settings| settings.timer_sound = SoundChoice::Bundled { sound });
      }
    });
    let weak = Rc::downgrade(app);
    let guard = Rc::clone(updating);
    self.ring_limit.connect_selected_notify(move |dropdown| {
      let limit = usize::try_from(dropdown.selected())
        .ok()
        .and_then(|index| RING_LIMIT_CHOICES.get(index).copied());
      if let (false, Some(app), Some(limit)) = (guard.get(), weak.upgrade(), limit) {
        app.update_settings(|settings| settings.ring_limit_minutes = limit);
      }
    });
    let weak = Rc::downgrade(app);
    let guard = Rc::clone(updating);
    self.background.connect_active_notify(move |switch| {
      if let (false, Some(app)) = (guard.get(), weak.upgrade()) {
        let on = switch.is_active();
        app.update_settings(|settings| settings.keep_in_background = on);
      }
    });
    let weak = Rc::downgrade(app);
    let guard = Rc::clone(updating);
    self.autostart.connect_active_notify(move |switch| {
      if let (false, Some(app)) = (guard.get(), weak.upgrade()) {
        app.set_autostart(switch.is_active());
      }
    });
    let weak = Rc::downgrade(app);
    test_sound.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.test_sound(&SoundChoice::default());
      }
    });
  }

  pub fn update(&self, app: &ClockApp) {
    let settings = app.settings();
    self.updating.set(true);
    self
      .hour_cycle
      .set_selected(u32::from(settings.hour_cycle == HourCycle::H12));
    self.seconds.set_active(settings.show_seconds);
    self.volume.set_value(f64::from(settings.alarm_volume));
    let beacon = settings.timer_sound
      == SoundChoice::Bundled {
        sound: BundledSound::Beacon,
      };
    self.timer_sound.set_selected(u32::from(beacon));
    let limit_index = RING_LIMIT_CHOICES
      .iter()
      .position(|minutes| *minutes == settings.ring_limit_minutes)
      .and_then(|index| u32::try_from(index).ok())
      .unwrap_or(0);
    self.ring_limit.set_selected(limit_index);
    self.background.set_active(settings.keep_in_background);
    match app.autostart() {
      Some(enabled) => {
        self.autostart.set_active(enabled);
        self.autostart.set_sensitive(true);
        self
          .autostart_hint
          .set_label(&app.texts.tr("settings.autostart_hint"));
      }
      None => {
        self.autostart.set_active(false);
        self.autostart.set_sensitive(false);
        self
          .autostart_hint
          .set_label(&app.texts.tr("settings.autostart_unavailable"));
      }
    }
    self.updating.set(false);
  }
}
