use chrono::Local;
use gtk::prelude::*;

use crate::calendar::CalendarEvent;
use crate::i18n::{I18n, Text};

pub fn rebuild_agenda<F>(list: &gtk::ListBox, events: &[CalendarEvent], i18n: I18n, on_event: F)
where
  F: Fn(i64) + Clone + 'static,
{
  while let Some(child) = list.first_child() {
    list.remove(&child);
  }
  if events.is_empty() {
    let row = gtk::ListBoxRow::new();
    row.add_css_class("event-row");
    row.set_activatable(false);
    let label = gtk::Label::new(Some(i18n.text(Text::NoEvents)));
    label.add_css_class("agenda-empty");
    label.set_xalign(0.0);
    row.set_child(Some(&label));
    list.append(&row);
    return;
  }
  for event in events {
    let row = gtk::ListBoxRow::new();
    row.add_css_class("event-row");
    row.set_activatable(false);
    row.set_selectable(false);
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    line.add_css_class("event-line");
    let time = if event.all_day {
      i18n.text(Text::AllDay).to_string()
    } else {
      event
        .start
        .with_timezone(&Local)
        .format("%H:%M")
        .to_string()
    };
    let time_label = gtk::Label::new(Some(&time));
    time_label.add_css_class("event-time");
    time_label.set_width_chars(6);
    time_label.set_xalign(0.0);
    let divider = gtk::Label::new(Some("│"));
    divider.add_css_class("event-divider");
    let title = gtk::Label::new(Some(&event.title));
    title.add_css_class("event-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    line.append(&time_label);
    line.append(&divider);
    line.append(&title);
    let button = gtk::Button::new();
    button.add_css_class("event-button");
    button.set_hexpand(true);
    button.set_child(Some(&line));
    if let Some(id) = event.id {
      let on_event_click = on_event.clone();
      button.connect_clicked(move |_| on_event_click(id));
    } else {
      button.set_sensitive(false);
    }
    row.set_child(Some(&button));
    list.append(&row);
  }
}
