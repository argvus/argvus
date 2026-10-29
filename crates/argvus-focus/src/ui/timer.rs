//! The main page: mode, phase, the timer inside a progress ring, the cycle
//! and the session controls.

use std::cell::Cell;
use std::f64::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

use argvus_time_core::duration::{format_remaining, rounded_minutes};
use argvus_time_core::pomodoro::{Mode, Phase};
use gtk::prelude::*;

use super::icon_button;
use crate::app::FocusApp;
use crate::engine::Status;

const RING_SIZE: i32 = 200;
const RING_WIDTH: f64 = 10.0;
const PHASE_CLASSES: [&str; 3] = ["focus", "short-break", "long-break"];

pub struct TimerPage {
  pub root: gtk::Box,
  pomodoro_mode: gtk::ToggleButton,
  single_mode: gtk::ToggleButton,
  phase: gtk::Label,
  ring: gtk::DrawingArea,
  progress: Rc<Cell<f64>>,
  time: gtk::Label,
  status: gtk::Label,
  dots: gtk::Box,
  cycle: gtk::Label,
  notice: gtk::Box,
  notice_text: gtk::Label,
  cancel: gtk::Button,
  primary: gtk::Button,
  secondary: gtk::Button,
  today: gtk::Label,
  /// Set while the page writes its own widgets, so mode toggles do not
  /// report the change back.
  updating: Rc<Cell<bool>>,
}

impl TimerPage {
  pub fn build(app: &Rc<FocusApp>) -> Self {
    let texts = &app.texts;
    let root = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(12)
      .margin_top(16)
      .margin_bottom(16)
      .margin_start(18)
      .margin_end(18)
      .css_classes(["page"])
      .build();

    let modes = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Center)
      .css_classes(["linked", "mode-switch"])
      .build();
    modes.update_property(&[gtk::accessible::Property::Label(&texts.tr("a11y.mode"))]);
    let pomodoro_mode = gtk::ToggleButton::with_label(&texts.tr("mode.pomodoro"));
    let single_mode = gtk::ToggleButton::with_label(&texts.tr("mode.single"));
    single_mode.set_group(Some(&pomodoro_mode));
    modes.append(&pomodoro_mode);
    modes.append(&single_mode);

    let phase = gtk::Label::builder()
      .halign(gtk::Align::Center)
      .css_classes(["phase-pill"])
      .build();

    let progress = Rc::new(Cell::new(0.0));
    let ring = gtk::DrawingArea::builder()
      .content_width(RING_SIZE)
      .content_height(RING_SIZE)
      .halign(gtk::Align::Center)
      .css_classes(["focus-ring"])
      .accessible_role(gtk::AccessibleRole::ProgressBar)
      .build();
    ring.set_draw_func({
      let progress = Rc::clone(&progress);
      move |area, cairo, width, height| draw_ring(area, cairo, width, height, progress.get())
    });

    let time = gtk::Label::builder()
      .css_classes(["display-large", "digits"])
      .build();
    let status = gtk::Label::builder().css_classes(["caption"]).build();
    let center = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .halign(gtk::Align::Center)
      .valign(gtk::Align::Center)
      .build();
    center.append(&time);
    center.append(&status);
    let overlay = gtk::Overlay::builder()
      .child(&ring)
      .halign(gtk::Align::Center)
      .build();
    overlay.add_overlay(&center);

    let dots = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(8)
      .halign(gtk::Align::Center)
      .build();
    let cycle = gtk::Label::builder().css_classes(["caption"]).build();

    let notice = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(8)
      .css_classes(["banner"])
      .visible(false)
      .build();
    let notice_text = gtk::Label::builder()
      .wrap(true)
      .xalign(0.0)
      .hexpand(true)
      .build();
    let dismiss = icon_button("window-close-symbolic", &texts.tr("action.dismiss"));
    dismiss.add_css_class("flat");
    notice.append(&notice_text);
    notice.append(&dismiss);

    let controls = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(18)
      .halign(gtk::Align::Center)
      .margin_top(4)
      .build();
    let cancel = icon_button("media-playback-stop-symbolic", &texts.tr("action.cancel"));
    cancel.set_action_name(Some("app.cancel"));
    cancel.add_css_class("round");
    let primary = gtk::Button::builder()
      .css_classes(["primary", "pill-button"])
      .build();
    primary.set_action_name(Some("app.toggle"));
    let secondary = icon_button("object-select-symbolic", &texts.tr("action.complete"));
    secondary.add_css_class("round");
    controls.append(&cancel);
    controls.append(&primary);
    controls.append(&secondary);

    let today = gtk::Label::builder()
      .css_classes(["caption"])
      .margin_top(6)
      .build();
    let hint = gtk::Label::builder()
      .label(texts.tr("hint.shortcuts"))
      .css_classes(["caption", "hint"])
      .wrap(true)
      .justify(gtk::Justification::Center)
      .build();

    root.append(&modes);
    root.append(&phase);
    root.append(&overlay);
    root.append(&dots);
    root.append(&cycle);
    root.append(&notice);
    root.append(&controls);
    root.append(&today);
    root.append(&hint);

    let page = Self {
      root,
      pomodoro_mode,
      single_mode,
      phase,
      ring,
      progress,
      time,
      status,
      dots,
      cycle,
      notice,
      notice_text,
      cancel,
      primary,
      secondary,
      today,
      updating: Rc::new(Cell::new(false)),
    };
    page.connect(app, &dismiss);
    page
  }

  fn connect(&self, app: &Rc<FocusApp>, dismiss: &gtk::Button) {
    for (button, mode) in [
      (&self.pomodoro_mode, Mode::Pomodoro),
      (&self.single_mode, Mode::Single),
    ] {
      let app = Rc::downgrade(app);
      let updating = Rc::clone(&self.updating);
      button.connect_toggled(move |button| {
        if button.is_active()
          && !updating.get()
          && let Some(app) = app.upgrade()
        {
          app.set_mode(mode);
        }
      });
    }

    let weak = Rc::downgrade(app);
    dismiss.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.dismiss_notice();
      }
    });

    let weak = Rc::downgrade(app);
    self.secondary.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        if app.view().phase.is_break() {
          app.skip_break();
        } else {
          app.complete();
        }
      }
    });
  }

  pub fn focus_primary(&self) {
    self.primary.grab_focus();
  }

  pub fn update(&self, app: &FocusApp) {
    let texts = &app.texts;
    let view = app.view();
    let started = matches!(view.status, Status::Running | Status::Paused(_));

    self.updating.set(true);
    self.pomodoro_mode.set_active(view.mode == Mode::Pomodoro);
    self.single_mode.set_active(view.mode == Mode::Single);
    self.updating.set(false);
    self.pomodoro_mode.set_sensitive(!started);
    self.single_mode.set_sensitive(!started);

    let (phase_key, phase_class) = match view.phase {
      Phase::Focus => ("phase.focus", PHASE_CLASSES[0]),
      Phase::ShortBreak => ("phase.short_break", PHASE_CLASSES[1]),
      Phase::LongBreak => ("phase.long_break", PHASE_CLASSES[2]),
    };
    self.phase.set_label(&texts.tr(phase_key));
    for widget in [
      self.phase.upcast_ref::<gtk::Widget>(),
      self.ring.upcast_ref(),
    ] {
      for class in PHASE_CLASSES {
        widget.remove_css_class(class);
      }
      widget.add_css_class(phase_class);
    }

    let remaining = format_remaining(view.remaining);
    self.time.set_label(&remaining);
    let status = match view.status {
      Status::Ready => texts.tr("status.ready"),
      Status::Running => texts.tr("status.running"),
      Status::Paused(_) => texts.tr("status.paused"),
      Status::AwaitingNext { .. } => texts.tr("status.up_next"),
    };
    self.status.set_label(&status);
    self.progress.set(view.progress);
    self.ring.queue_draw();
    self.ring.update_property(&[
      gtk::accessible::Property::Label(&texts.tr("a11y.ring")),
      gtk::accessible::Property::ValueNow(view.progress * 100.0),
      gtk::accessible::Property::ValueText(&format!("{remaining} · {status}")),
    ]);

    self.update_cycle(app, view.mode, view.cycle);

    let primary_key = match (view.status, view.phase.is_break()) {
      (Status::Running, _) => "action.pause",
      (Status::Paused(_), _) => "action.resume",
      (_, true) => "action.start_break",
      (_, false) => "action.start_focus",
    };
    self.primary.set_label(&texts.tr(primary_key));
    self.cancel.set_sensitive(started);
    let (secondary_icon, secondary_key, secondary_enabled) = if view.phase.is_break() {
      ("media-skip-forward-symbolic", "action.skip_break", true)
    } else {
      ("object-select-symbolic", "action.complete", started)
    };
    let secondary_label = texts.tr(secondary_key);
    self.secondary.set_icon_name(secondary_icon);
    self.secondary.set_tooltip_text(Some(&secondary_label));
    self
      .secondary
      .update_property(&[gtk::accessible::Property::Label(&secondary_label)]);
    self.secondary.set_sensitive(secondary_enabled);

    match app.notice_text() {
      Some((text, warning)) => {
        self.notice_text.set_label(&text);
        if warning {
          self.notice.add_css_class("warning");
        } else {
          self.notice.remove_css_class("warning");
        }
        self.notice.set_visible(true);
      }
      None => self.notice.set_visible(false),
    }

    let today = app.today();
    self.today.set_label(&texts.tr_args(
      "today.summary",
      [
        (
          "sessions",
          texts.tr_plural("today.sessions", i64::from(today.sessions)),
        ),
        ("minutes", rounded_minutes(today.focused).to_string()),
      ],
    ));
  }

  fn update_cycle(&self, app: &FocusApp, mode: Mode, (done, length): (u32, u32)) {
    let single = mode == Mode::Single;
    self.dots.set_visible(!single);
    if single {
      self.cycle.set_label(&app.texts.tr("status.single"));
      return;
    }
    while let Some(child) = self.dots.first_child() {
      self.dots.remove(&child);
    }
    for index in 0..length {
      let dot = gtk::Box::builder().css_classes(["cycle-dot"]).build();
      if index < done {
        dot.add_css_class("done");
      }
      self.dots.append(&dot);
    }
    let current = (done + 1).min(length);
    self.cycle.set_label(&app.texts.tr_args(
      "status.cycle",
      [
        ("current", current.to_string()),
        ("total", length.to_string()),
      ],
    ));
  }
}

/// A track circle and the elapsed arc, clockwise from the top, in the
/// widget's CSS colour (the phase colour).
fn draw_ring(
  area: &gtk::DrawingArea,
  cairo: &gtk::cairo::Context,
  width: i32,
  height: i32,
  progress: f64,
) {
  let color = area.color();
  let size = f64::from(width.min(height));
  let radius = (size - RING_WIDTH) / 2.0;
  let (center_x, center_y) = (f64::from(width) / 2.0, f64::from(height) / 2.0);
  cairo.set_line_width(RING_WIDTH);
  cairo.set_line_cap(gtk::cairo::LineCap::Round);

  cairo.set_source_rgba(
    f64::from(color.red()),
    f64::from(color.green()),
    f64::from(color.blue()),
    0.16,
  );
  cairo.arc(center_x, center_y, radius, 0.0, TAU);
  let _ = cairo.stroke();

  if progress > 0.0 {
    cairo.set_source_rgba(
      f64::from(color.red()),
      f64::from(color.green()),
      f64::from(color.blue()),
      f64::from(color.alpha()),
    );
    cairo.arc(
      center_x,
      center_y,
      radius,
      -FRAC_PI_2,
      -FRAC_PI_2 + TAU * progress.min(1.0),
    );
    let _ = cairo.stroke();
  }
}
