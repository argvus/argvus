//! Stopwatch with laps. The display follows the frame clock only while the
//! stopwatch runs and the page is visible; the time itself comes from boot
//! time, not from the redraws.

use std::cell::Cell;
use std::rc::Rc;

use argvus_time_core::duration::format_precise;
use argvus_time_core::ticks::Ticks;
use gtk::glib;
use gtk::prelude::*;

use super::page_box;
use crate::app::ClockApp;

pub struct StopwatchPage {
  pub root: gtk::Box,
  display: gtk::Label,
  primary: gtk::Button,
  secondary: gtk::Button,
  laps: gtk::ListBox,
  /// Number of laps listed, to rebuild only when it changes.
  listed_laps: Cell<Option<usize>>,
  /// Whether a tick callback is registered.
  ticking: Rc<Cell<bool>>,
}

impl StopwatchPage {
  pub fn build(app: &Rc<ClockApp>) -> Self {
    let root = page_box();
    let display = gtk::Label::builder()
      .label(format_precise(std::time::Duration::ZERO))
      .css_classes(["display-large", "digits"])
      .margin_top(18)
      .build();
    let controls = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(14)
      .halign(gtk::Align::Center)
      .build();
    let secondary = gtk::Button::builder().css_classes(["pill-button"]).build();
    let primary = gtk::Button::builder()
      .css_classes(["primary", "pill-button"])
      .build();
    controls.append(&secondary);
    controls.append(&primary);
    let laps = gtk::ListBox::builder()
      .selection_mode(gtk::SelectionMode::None)
      .css_classes(["laps"])
      .build();
    laps.update_property(&[gtk::accessible::Property::Label(
      &app.texts.tr("stopwatch.laps"),
    )]);
    let scrolled = gtk::ScrolledWindow::builder()
      .hscrollbar_policy(gtk::PolicyType::Never)
      .vexpand(true)
      .child(&laps)
      .build();
    root.append(&display);
    root.append(&controls);
    root.append(&scrolled);

    let weak = Rc::downgrade(app);
    primary.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.change_stopwatch(|stopwatch, now| {
          if stopwatch.is_running() {
            stopwatch.pause(now);
          } else {
            stopwatch.start(now);
          }
        });
      }
    });
    let weak = Rc::downgrade(app);
    secondary.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.change_stopwatch(|stopwatch, now| {
          if stopwatch.is_running() {
            stopwatch.lap(now);
          } else {
            stopwatch.reset();
          }
        });
      }
    });

    Self {
      root,
      display,
      primary,
      secondary,
      laps,
      listed_laps: Cell::new(None),
      ticking: Rc::new(Cell::new(false)),
    }
  }

  pub fn update(&self, app: &Rc<ClockApp>) {
    let texts = &app.texts;
    let now = Ticks::now().boottime;
    let stopwatch = app.stopwatch();
    let running = stopwatch.is_running();
    let started = stopwatch.has_started(now);
    self
      .display
      .set_label(&format_precise(stopwatch.elapsed(now)));
    self.primary.set_label(&texts.tr(match (running, started) {
      (true, _) => "action.pause",
      (false, true) => "action.resume",
      (false, false) => "action.start",
    }));
    self.secondary.set_label(&texts.tr(if running {
      "stopwatch.lap"
    } else {
      "action.reset"
    }));
    self.secondary.set_sensitive(started);

    if self.listed_laps.get() != Some(stopwatch.laps().len()) {
      self.laps.remove_all();
      for lap in stopwatch.laps().iter().rev() {
        let row = gtk::Box::builder()
          .orientation(gtk::Orientation::Horizontal)
          .spacing(12)
          .margin_top(4)
          .margin_bottom(4)
          .margin_start(6)
          .margin_end(6)
          .build();
        let name = texts.tr_args("stopwatch.lap_number", [("number", lap.number.to_string())]);
        row.append(
          &gtk::Label::builder()
            .label(name)
            .xalign(0.0)
            .hexpand(true)
            .build(),
        );
        row.append(
          &gtk::Label::builder()
            .label(format_precise(lap.split()))
            .css_classes(["digits"])
            .build(),
        );
        row.append(
          &gtk::Label::builder()
            .label(format_precise(lap.total()))
            .css_classes(["digits", "caption"])
            .build(),
        );
        self.laps.append(&row);
      }
      self.listed_laps.set(Some(stopwatch.laps().len()));
    }
    drop(stopwatch);

    if running {
      self.start_ticking(app);
    }
  }

  /// Redraws the display on every frame while the stopwatch runs. GTK only
  /// calls the callback while the label is mapped.
  fn start_ticking(&self, app: &Rc<ClockApp>) {
    if self.ticking.replace(true) {
      return;
    }
    let weak = Rc::downgrade(app);
    let ticking = Rc::clone(&self.ticking);
    self.display.add_tick_callback(move |label, _| {
      let running = weak.upgrade().is_some_and(|app| {
        let stopwatch = app.stopwatch();
        label.set_label(&format_precise(stopwatch.elapsed(Ticks::now().boottime)));
        stopwatch.is_running()
      });
      if running {
        glib::ControlFlow::Continue
      } else {
        ticking.set(false);
        glib::ControlFlow::Break
      }
    });
  }
}
