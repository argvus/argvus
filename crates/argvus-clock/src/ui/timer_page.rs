//! Countdown timer: hours, minutes and seconds, presets, and the running
//! view with pause, restart and cancel. Not related to ARGVUS Focus sessions.

use std::rc::Rc;
use std::time::Duration;

use argvus_time_core::duration::{Hms, format_remaining};
use argvus_time_core::elapsed::CountdownState;
use argvus_time_core::ticks::Ticks;
use gtk::prelude::*;

use super::{icon_button, page_box};
use crate::app::ClockApp;
use crate::timers::Timer;

const PRESET_MINUTES: [u64; 6] = [5, 10, 15, 25, 30, 60];

pub struct TimerPage {
  pub root: gtk::Box,
  setup: gtk::Box,
  hours: gtk::SpinButton,
  minutes: gtk::SpinButton,
  seconds: gtk::SpinButton,
  start: gtk::Button,
  running: gtk::Box,
  display: gtk::Label,
  progress: gtk::ProgressBar,
  pause: gtk::Button,
}

fn spin(max: f64) -> gtk::SpinButton {
  let spin = gtk::SpinButton::with_range(0.0, max, 1.0);
  spin.set_numeric(true);
  spin.set_wrap(true);
  spin.set_orientation(gtk::Orientation::Vertical);
  spin.add_css_class("digits");
  // Two digits, like a clock.
  spin.connect_output(|spin| {
    spin.set_text(&format!("{:02}", spin.value_as_int()));
    gtk::glib::Propagation::Stop
  });
  spin
}

impl TimerPage {
  pub fn build(app: &Rc<ClockApp>) -> Self {
    let texts = &app.texts;
    let root = page_box();

    let setup = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(14)
      .build();
    let fields = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(8)
      .halign(gtk::Align::Center)
      .margin_top(18)
      .css_classes(["timer-fields"])
      .build();
    let hours = spin(23.0);
    let minutes = spin(59.0);
    let seconds = spin(59.0);
    for (index, (field, key)) in [
      (&hours, "timer.hours"),
      (&minutes, "timer.minutes"),
      (&seconds, "timer.seconds"),
    ]
    .into_iter()
    .enumerate()
    {
      if index > 0 {
        fields.append(
          &gtk::Label::builder()
            .label(":")
            .css_classes(["display"])
            .build(),
        );
      }
      let column = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();
      let caption = gtk::Label::builder()
        .label(texts.tr(key))
        .css_classes(["caption"])
        .build();
      field.update_relation(&[gtk::accessible::Relation::LabelledBy(&[
        caption.upcast_ref()
      ])]);
      column.append(field);
      column.append(&caption);
      fields.append(&column);
    }
    let presets = gtk::FlowBox::builder()
      .selection_mode(gtk::SelectionMode::None)
      .max_children_per_line(3)
      .min_children_per_line(3)
      .column_spacing(8)
      .row_spacing(8)
      .homogeneous(true)
      .build();
    let start = gtk::Button::builder()
      .label(texts.tr("action.start"))
      .css_classes(["primary", "pill-button"])
      .halign(gtk::Align::Center)
      .build();
    setup.append(&fields);
    setup.append(
      &gtk::Label::builder()
        .label(texts.tr("timer.presets"))
        .css_classes(["section-title"])
        .xalign(0.0)
        .build(),
    );
    setup.append(&presets);
    setup.append(&start);

    let running = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(16)
      .margin_top(24)
      .visible(false)
      .build();
    let display = gtk::Label::builder()
      .css_classes(["display-large", "digits"])
      .build();
    let progress = gtk::ProgressBar::builder()
      .css_classes(["timer-progress"])
      .build();
    progress.update_property(&[gtk::accessible::Property::Label(
      &texts.tr("timer.progress"),
    )]);
    let controls = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(14)
      .halign(gtk::Align::Center)
      .build();
    let cancel = icon_button("media-playback-stop-symbolic", &texts.tr("action.cancel"));
    cancel.add_css_class("round");
    let pause = gtk::Button::builder()
      .css_classes(["primary", "pill-button"])
      .build();
    let restart = icon_button("view-refresh-symbolic", &texts.tr("timer.restart"));
    restart.add_css_class("round");
    controls.append(&cancel);
    controls.append(&pause);
    controls.append(&restart);
    running.append(&display);
    running.append(&progress);
    running.append(&controls);

    root.append(&setup);
    root.append(&running);

    let page = Self {
      root,
      setup,
      hours,
      minutes,
      seconds,
      start,
      running,
      display,
      progress,
      pause,
    };

    for minutes in PRESET_MINUTES {
      let button =
        gtk::Button::with_label(&texts.tr_args("timer.preset", [("minutes", minutes.to_string())]));
      let weak = Rc::downgrade(app);
      button.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
          app.start_timer(Duration::from_secs(minutes * 60));
        }
      });
      presets.append(&button);
    }
    let weak = Rc::downgrade(app);
    let fields = (
      page.hours.clone(),
      page.minutes.clone(),
      page.seconds.clone(),
    );
    page.start.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        let length = Hms {
          hours: spin_value(&fields.0),
          minutes: spin_value(&fields.1),
          seconds: spin_value(&fields.2),
        }
        .to_duration();
        app.start_timer(length);
      }
    });
    let weak = Rc::downgrade(app);
    page.pause.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.change_timer(|timer, now| {
          if timer.is_running(now) {
            timer.pause(now);
          } else {
            timer.resume(now);
          }
        });
      }
    });
    let weak = Rc::downgrade(app);
    cancel.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.change_timer(|timer, _| timer.cancel());
      }
    });
    let weak = Rc::downgrade(app);
    restart.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.change_timer(Timer::restart);
      }
    });

    // The fields start with the last length used.
    let last = Hms::from_duration(Duration::from_secs(app.settings().timer_seconds));
    page.hours.set_value(last.hours as f64);
    page.minutes.set_value(last.minutes as f64);
    page.seconds.set_value(last.seconds as f64);
    page
  }

  pub fn update(&self, app: &ClockApp) {
    let now = Ticks::now().boottime;
    let state = app.timer().state(now);
    let active = matches!(state, CountdownState::Running | CountdownState::Paused);
    self.setup.set_visible(!active);
    self.running.set_visible(active);
    let fields_set = [&self.hours, &self.minutes, &self.seconds]
      .iter()
      .any(|field| field.value_as_int() > 0);
    self.start.set_sensitive(fields_set);
    let key = if state == CountdownState::Running {
      "action.pause"
    } else {
      "action.resume"
    };
    self.pause.set_label(&app.texts.tr(key));
    self.update_live(app);
  }

  pub fn update_live(&self, app: &ClockApp) {
    let now = Ticks::now().boottime;
    let timer = app.timer();
    if let Some(countdown) = timer.countdown() {
      self
        .display
        .set_label(&format_remaining(countdown.remaining(now)));
      self.progress.set_fraction(1.0 - countdown.progress(now));
    }
  }
}

fn spin_value(spin: &gtk::SpinButton) -> u64 {
  u64::try_from(spin.value_as_int()).unwrap_or(0)
}
