//! The ringing alert: a card at the top of the screen, above other windows,
//! with Snooze and Dismiss for each alarm or timer that is ringing.
//!
//! It is a layer-shell overlay where the compositor supports it, and a plain
//! window otherwise. The keyboard is taken only on demand, so text typed in
//! another window when an alarm starts cannot dismiss it by accident. When it
//! has the focus, Enter dismisses and S snoozes the first entry.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use super::clear_box;
use crate::app::{APP_ID, ClockApp};
use crate::ringing::Source;

const TOP_MARGIN: i32 = 56;

/// The alert, created when something starts ringing and destroyed when the
/// queue empties, so no surface stays allocated between alarms.
#[derive(Default)]
pub struct RingWindow {
  surface: RefCell<Option<Surface>>,
}

struct Surface {
  window: gtk::Window,
  entries: gtk::Box,
  sound_error: gtk::Label,
}

impl RingWindow {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn update(&self, app: &Rc<ClockApp>) {
    if app.ringing().is_empty() {
      if let Some(surface) = self.surface.borrow_mut().take() {
        surface.window.destroy();
      }
      return;
    }
    let created = self.surface.borrow().is_none();
    if created {
      *self.surface.borrow_mut() = Some(Surface::build(app));
    }
    if let Some(surface) = self.surface.borrow().as_ref() {
      surface.fill(app);
      if created {
        surface.window.present();
      }
    }
  }
}

impl Surface {
  fn build(app: &Rc<ClockApp>) -> Self {
    let window = gtk::Window::builder()
      .application(&app.application)
      .title(app.texts.tr("ring.window_title"))
      .icon_name(APP_ID)
      .resizable(false)
      .decorated(false)
      .css_classes(["argvus-time", "argvus-clock", "argvus-ring"])
      .build();
    if gtk4_layer_shell::is_supported() {
      window.init_layer_shell();
      window.set_namespace(Some("argvus-clock-alarm"));
      window.set_layer(Layer::Overlay);
      window.set_anchor(Edge::Top, true);
      window.set_margin(Edge::Top, TOP_MARGIN);
      window.set_keyboard_mode(KeyboardMode::OnDemand);
    }

    let card = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(10)
      .css_classes(["ring-card"])
      .build();
    let entries = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(12)
      .build();
    let sound_error = gtk::Label::builder()
      .wrap(true)
      .xalign(0.0)
      .css_classes(["ring-sound-error"])
      .visible(false)
      .build();
    card.append(&entries);
    card.append(&sound_error);
    window.set_child(Some(&card));

    let keys = gtk::EventControllerKey::new();
    let weak = Rc::downgrade(app);
    keys.connect_key_pressed(move |_, key, _, _| {
      let Some(app) = weak.upgrade() else {
        return glib::Propagation::Proceed;
      };
      let first = app.ringing().entries().first().map(|entry| entry.source);
      match (key, first) {
        (gdk::Key::Return | gdk::Key::KP_Enter, Some(source)) => {
          app.dismiss(source);
          glib::Propagation::Stop
        }
        (gdk::Key::s | gdk::Key::S, Some(Source::Alarm(id))) => {
          app.snooze(id);
          glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
      }
    });
    window.add_controller(keys);
    // Closing it with the compositor counts as dismissing everything.
    let weak = Rc::downgrade(app);
    window.connect_close_request(move |_| {
      if let Some(app) = weak.upgrade() {
        let sources: Vec<Source> = app
          .ringing()
          .entries()
          .iter()
          .map(|entry| entry.source)
          .collect();
        for source in sources {
          app.dismiss(source);
        }
      }
      glib::Propagation::Stop
    });

    Self {
      window,
      entries,
      sound_error,
    }
  }

  fn fill(&self, app: &Rc<ClockApp>) {
    clear_box(&self.entries);
    for entry in app.ringing().entries() {
      self.entries.append(&self.entry_row(app, entry));
    }
    match app.sound_failed() {
      Some(error) => {
        self
          .sound_error
          .set_label(&app.texts.tr_args("ring.sound_failed", [("error", error)]));
        self.sound_error.set_visible(true);
      }
      None => self.sound_error.set_visible(false),
    }
  }

  fn entry_row(&self, app: &Rc<ClockApp>, entry: &crate::ringing::Entry) -> gtk::Box {
    let texts = &app.texts;
    let row = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(14)
      .build();
    let icon = gtk::Image::builder()
      .icon_name(APP_ID)
      .pixel_size(40)
      .css_classes(["ring-icon"])
      .build();
    let text = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .hexpand(true)
      .valign(gtk::Align::Center)
      .build();
    text.append(
      &gtk::Label::builder()
        .label(&entry.title)
        .xalign(0.0)
        .css_classes(["ring-title"])
        .build(),
    );
    let mut detail = match entry.source {
      Source::Alarm(_) => app.local_time(entry.due_at),
      Source::Timer => texts.tr("notify.timer.body"),
    };
    if let Some(late_by) = entry.late_by {
      detail = format!("{detail} · {}", app.late_text(late_by));
    }
    text.append(
      &gtk::Label::builder()
        .label(detail)
        .xalign(0.0)
        .wrap(true)
        .css_classes(["caption"])
        .build(),
    );

    let buttons = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(8)
      .valign(gtk::Align::Center)
      .build();
    if let Source::Alarm(id) = entry.source {
      let minutes = app.snooze_minutes(id).unwrap_or_default();
      let snooze =
        gtk::Button::with_label(&texts.tr_args("ring.snooze", [("minutes", minutes.to_string())]));
      let weak = Rc::downgrade(app);
      snooze.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
          app.snooze(id);
        }
      });
      buttons.append(&snooze);
    }
    let dismiss = gtk::Button::builder()
      .label(texts.tr("action.dismiss"))
      .css_classes(["primary"])
      .build();
    let weak = Rc::downgrade(app);
    let source = entry.source;
    dismiss.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.dismiss(source);
      }
    });
    buttons.append(&dismiss);

    row.append(&icon);
    row.append(&text);
    row.append(&buttons);
    row
  }
}
