//! Small GTK builders shared by the event editor and the settings form.

use super::*;

pub(super) fn editor_row() -> gtk::Box {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  row.add_css_class("editor-row");
  row
}

pub(super) fn editor_label() -> gtk::Label {
  let label = gtk::Label::new(None);
  label.add_css_class("editor-label");
  label.set_xalign(0.0);
  label.set_width_chars(9);
  label
}

pub(super) fn editor_entry() -> gtk::Entry {
  let entry = gtk::Entry::new();
  entry.add_css_class("editor-entry");
  entry
}

pub(super) fn editor_control_has_focus(window: &gtk::Window) -> bool {
  gtk::prelude::RootExt::focus(window).is_some_and(|widget| {
    widget.is::<gtk::Entry>() || widget.ancestor(gtk::Entry::static_type()).is_some()
  })
}

pub(super) fn editor_number_spin(max: u32) -> gtk::SpinButton {
  let adjustment = gtk::Adjustment::new(0.0, 0.0, f64::from(max), 1.0, 10.0, 0.0);
  let spin = gtk::SpinButton::new(Some(&adjustment), 1.0, 0);
  spin.add_css_class("editor-entry");
  spin.add_css_class("number-entry");
  spin.set_numeric(true);
  spin.set_snap_to_ticks(true);
  spin.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
  spin.set_hexpand(false);
  spin.set_halign(gtk::Align::Start);
  spin.set_width_chars(2);
  spin.set_value(0.0);
  spin.connect_input(|spin| {
    let parsed = spin.text().trim().parse::<f64>().ok()?;
    let lower = spin.adjustment().lower();
    let upper = spin.adjustment().upper();
    Some(Ok(parsed.clamp(lower, upper)))
  });
  spin.connect_output(|spin| {
    let value = spin.value().round() as u32;
    let text = format!("{value:02}");
    if spin.text() != text {
      spin.set_text(&text);
    }
    gtk::glib::Propagation::Stop
  });
  spin
}

pub(super) fn settings_label() -> gtk::Label {
  let label = gtk::Label::new(None);
  label.add_css_class("editor-label");
  label.set_xalign(0.0);
  label.set_width_chars(14);
  label
}

pub(super) fn settings_section_label(text: &str) -> gtk::Label {
  let label = gtk::Label::new(Some(text));
  label.add_css_class("editor-section");
  label.set_xalign(0.0);
  label
}

pub(super) fn settings_number_spin(max: u32, width: i32) -> gtk::SpinButton {
  let adjustment = gtk::Adjustment::new(0.0, 0.0, f64::from(max), 1.0, 10.0, 0.0);
  let spin = gtk::SpinButton::new(Some(&adjustment), 1.0, 0);
  spin.add_css_class("editor-entry");
  spin.add_css_class("number-entry");
  spin.set_numeric(true);
  spin.set_snap_to_ticks(true);
  spin.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
  spin.set_hexpand(false);
  spin.set_halign(gtk::Align::Start);
  spin.set_width_chars(width);
  spin.set_value(0.0);
  spin.connect_input(|spin| {
    let parsed = spin.text().trim().parse::<f64>().ok()?;
    let lower = spin.adjustment().lower();
    let upper = spin.adjustment().upper();
    Some(Ok(parsed.clamp(lower, upper)))
  });
  spin.connect_output(move |spin| {
    let value = spin.value().round() as u32;
    let text = if max <= 99 {
      format!("{value:02}")
    } else {
      value.to_string()
    };
    if spin.text() != text {
      spin.set_text(&text);
    }
    gtk::glib::Propagation::Stop
  });
  spin
}
