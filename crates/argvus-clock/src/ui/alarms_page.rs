//! The alarm list: switches, the next alarm, missed alarms and the button to
//! add one. Activating a row opens the editor.

use std::cell::RefCell;
use std::rc::Rc;

use argvus_time_core::alarm::{Alarm, AlarmBook, WeekdaySet};
use chrono::{Local, Utc};
use gtk::prelude::*;

use super::{clear_box, page_box, relative_text};
use crate::app::ClockApp;

pub struct AlarmsPage {
  pub root: gtk::Box,
  missed: gtk::Box,
  missed_list: gtk::Box,
  next: gtk::Label,
  list: gtk::ListBox,
  empty: gtk::Label,
  /// The alarms the list was built from, to rebuild only on change.
  shown: RefCell<Option<AlarmBook>>,
}

impl AlarmsPage {
  pub fn build(app: &Rc<ClockApp>) -> Self {
    let texts = &app.texts;
    let root = page_box();

    let missed = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(6)
      .css_classes(["banner", "warning"])
      .visible(false)
      .build();
    let missed_header = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(8)
      .build();
    missed_header.append(
      &gtk::Label::builder()
        .label(texts.tr("alarms.missed_title"))
        .xalign(0.0)
        .hexpand(true)
        .css_classes(["heading"])
        .build(),
    );
    let clear_missed = gtk::Button::builder()
      .label(texts.tr("action.dismiss"))
      .css_classes(["flat"])
      .build();
    missed_header.append(&clear_missed);
    let missed_list = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(2)
      .build();
    missed.append(&missed_header);
    missed.append(&missed_list);

    let next = gtk::Label::builder()
      .css_classes(["caption"])
      .xalign(0.0)
      .wrap(true)
      .build();
    let list = gtk::ListBox::builder()
      .selection_mode(gtk::SelectionMode::None)
      .activate_on_single_click(true)
      .css_classes(["alarm-list"])
      .build();
    list.update_property(&[gtk::accessible::Property::Label(&texts.tr("page.alarms"))]);
    let empty = gtk::Label::builder()
      .label(texts.tr("alarms.empty"))
      .wrap(true)
      .justify(gtk::Justification::Center)
      .css_classes(["empty-state"])
      .build();
    let scrolled = gtk::ScrolledWindow::builder()
      .hscrollbar_policy(gtk::PolicyType::Never)
      .vexpand(true)
      .child(&list)
      .build();
    let add = gtk::Button::builder()
      .label(texts.tr("alarms.add"))
      .action_name("app.new-alarm")
      .css_classes(["primary"])
      .halign(gtk::Align::Center)
      .build();

    root.append(&missed);
    root.append(&next);
    root.append(&empty);
    root.append(&scrolled);
    root.append(&add);

    let weak = Rc::downgrade(app);
    clear_missed.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.clear_missed();
      }
    });
    let weak = Rc::downgrade(app);
    list.connect_row_activated(move |_, row| {
      if let (Some(app), Ok(id)) = (weak.upgrade(), row.widget_name().parse::<u64>()) {
        app.edit_alarm(Some(argvus_time_core::alarm::AlarmId(id)));
      }
    });

    Self {
      root,
      missed,
      missed_list,
      next,
      list,
      empty,
      shown: RefCell::default(),
    }
  }

  pub fn update(&self, app: &Rc<ClockApp>) {
    let missed = app.missed();
    clear_box(&self.missed_list);
    for alarm in missed.iter() {
      let text = app.texts.tr_args(
        "alarms.missed_item",
        [
          ("title", alarm.title.clone()),
          ("time", app.local_time(alarm.due_at)),
        ],
      );
      self.missed_list.append(
        &gtk::Label::builder()
          .label(text)
          .xalign(0.0)
          .wrap(true)
          .build(),
      );
    }
    self.missed.set_visible(!missed.is_empty());
    drop(missed);

    let book = app.alarms().clone();
    if self.shown.borrow().as_ref() != Some(&book) {
      self.list.remove_all();
      for alarm in book.alarms() {
        self.list.append(&alarm_row(app, alarm));
      }
      self.empty.set_visible(book.alarms().is_empty());
      self.list.set_visible(!book.alarms().is_empty());
      *self.shown.borrow_mut() = Some(book);
    }
    self.update_live(app);
  }

  pub fn update_live(&self, app: &ClockApp) {
    let text = match app.alarms().next_trigger(&Local) {
      Some((_, trigger)) => {
        let until = (trigger.at - Utc::now()).to_std().unwrap_or_default();
        app.texts.tr_args(
          "alarms.next",
          [
            ("time", app.local_time(trigger.at)),
            ("relative", relative_text(&app.texts, until)),
          ],
        )
      }
      None => app.texts.tr("clock.no_alarm"),
    };
    self.next.set_label(&text);
  }
}

fn alarm_row(app: &Rc<ClockApp>, alarm: &Alarm) -> gtk::ListBoxRow {
  let texts = &app.texts;
  let row = gtk::ListBoxRow::builder()
    .name(alarm.id.0.to_string())
    .activatable(true)
    .build();
  let content = gtk::Box::builder()
    .orientation(gtk::Orientation::Horizontal)
    .spacing(12)
    .margin_top(8)
    .margin_bottom(8)
    .margin_start(6)
    .margin_end(6)
    .build();
  let text = gtk::Box::builder()
    .orientation(gtk::Orientation::Vertical)
    .hexpand(true)
    .build();
  let cycle = app.settings().hour_cycle;
  let time = chrono::NaiveTime::from_hms_opt(
    u32::from(alarm.time.hour()),
    u32::from(alarm.time.minute()),
    0,
  )
  .unwrap_or_default();
  let time = gtk::Label::builder()
    .label(app.calendar.time(&time, cycle, false).joined())
    .xalign(0.0)
    .css_classes(["alarm-time", "digits"])
    .build();
  let title = if alarm.label.is_empty() {
    texts.tr("alarm.default_title")
  } else {
    alarm.label.clone()
  };
  let mut details = format!("{title} · {}", repeat_text(app, alarm.repeat));
  if alarm.snoozed_until.is_some() {
    details = format!("{details} · {}", texts.tr("alarms.snoozed"));
  }
  text.append(&time);
  text.append(
    &gtk::Label::builder()
      .label(&details)
      .xalign(0.0)
      .wrap(true)
      .css_classes(["caption"])
      .build(),
  );
  if !alarm.enabled {
    row.add_css_class("disabled");
  }

  let switch = gtk::Switch::builder()
    .active(alarm.enabled)
    .valign(gtk::Align::Center)
    .build();
  switch.update_property(&[gtk::accessible::Property::Label(
    &texts.tr_args("alarms.toggle", [("title", title)]),
  )]);
  let weak = Rc::downgrade(app);
  let id = alarm.id;
  switch.connect_active_notify(move |switch| {
    if let Some(app) = weak.upgrade() {
      app.set_alarm_enabled(id, switch.is_active());
    }
  });

  content.append(&text);
  content.append(&switch);
  row.set_child(Some(&content));
  row
}

/// "Once", "Every day", "Weekdays", "Weekends" or the short day names.
pub fn repeat_text(app: &ClockApp, repeat: WeekdaySet) -> String {
  match repeat {
    WeekdaySet::ONCE => app.texts.tr("repeat.once"),
    WeekdaySet::EVERY_DAY => app.texts.tr("repeat.every_day"),
    WeekdaySet::WEEKDAYS => app.texts.tr("repeat.weekdays"),
    WeekdaySet::WEEKEND => app.texts.tr("repeat.weekends"),
    days => days
      .days()
      .map(|day| app.calendar.weekday_short(day))
      .collect::<Vec<_>>()
      .join(", "),
  }
}
