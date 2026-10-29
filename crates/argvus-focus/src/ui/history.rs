//! History page: today's summary, totals and the recent sessions.

use std::rc::Rc;

use argvus_time_core::duration::rounded_minutes;
use argvus_time_ui::text::HourCycle;
use chrono::Local;
use gtk::prelude::*;

use super::section_title;
use crate::app::FocusApp;

/// Sessions listed on the page; older ones only count in the totals.
const RECENT_LIMIT: usize = 50;

pub struct HistoryPage {
  pub root: gtk::Box,
  today: gtk::Label,
  total: gtk::Label,
  list: gtk::ListBox,
  empty: gtk::Label,
  clear: gtk::Button,
  confirm: gtk::Box,
}

impl HistoryPage {
  pub fn build(app: &Rc<FocusApp>) -> Self {
    let texts = &app.texts;
    let root = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(10)
      .margin_top(14)
      .margin_bottom(16)
      .margin_start(18)
      .margin_end(18)
      .css_classes(["page"])
      .build();

    let summary = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(4)
      .css_classes(["card"])
      .build();
    let today = gtk::Label::builder()
      .xalign(0.0)
      .css_classes(["summary-today"])
      .wrap(true)
      .build();
    let total = gtk::Label::builder()
      .xalign(0.0)
      .css_classes(["caption"])
      .wrap(true)
      .build();
    summary.append(&today);
    summary.append(&total);

    let list = gtk::ListBox::builder()
      .selection_mode(gtk::SelectionMode::None)
      .css_classes(["history-list"])
      .build();
    list.update_property(&[gtk::accessible::Property::Label(
      &texts.tr("history.recent"),
    )]);
    let empty = gtk::Label::builder()
      .label(texts.tr("history.empty"))
      .wrap(true)
      .justify(gtk::Justification::Center)
      .css_classes(["empty-state"])
      .build();
    let scrolled = gtk::ScrolledWindow::builder()
      .hscrollbar_policy(gtk::PolicyType::Never)
      .vexpand(true)
      .child(&list)
      .build();

    let clear = gtk::Button::builder()
      .label(texts.tr("action.clear_history"))
      .css_classes(["destructive-action"])
      .halign(gtk::Align::End)
      .build();

    // "Clear history" asks inline: a dialog would be a second window, and
    // the popup closes when it loses the focus.
    let confirm = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(8)
      .css_classes(["card"])
      .visible(false)
      .build();
    confirm.append(
      &gtk::Label::builder()
        .label(texts.tr("history.clear.confirm"))
        .xalign(0.0)
        .css_classes(["heading"])
        .build(),
    );
    confirm.append(
      &gtk::Label::builder()
        .label(texts.tr("history.clear.detail"))
        .xalign(0.0)
        .wrap(true)
        .css_classes(["caption"])
        .build(),
    );
    let confirm_buttons = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(8)
      .halign(gtk::Align::End)
      .build();
    let keep = gtk::Button::with_label(&texts.tr("action.cancel"));
    let erase = gtk::Button::builder()
      .label(texts.tr("action.clear_history"))
      .css_classes(["destructive-action"])
      .build();
    confirm_buttons.append(&keep);
    confirm_buttons.append(&erase);
    confirm.append(&confirm_buttons);

    root.append(&summary);
    root.append(&section_title(&texts.tr("history.recent_title")));
    root.append(&empty);
    root.append(&scrolled);
    root.append(&confirm);
    root.append(&clear);

    let (clear_button, confirm_box) = (clear.clone(), confirm.clone());
    clear.connect_clicked(move |button| {
      button.set_visible(false);
      confirm_box.set_visible(true);
    });
    let (clear_button_back, confirm_box_back) = (clear_button.clone(), confirm.clone());
    keep.connect_clicked(move |_| {
      confirm_box_back.set_visible(false);
      clear_button_back.set_visible(true);
    });
    let weak = Rc::downgrade(app);
    let (clear_button_done, confirm_box_done) = (clear_button, confirm.clone());
    erase.connect_clicked(move |_| {
      confirm_box_done.set_visible(false);
      clear_button_done.set_visible(true);
      if let Some(app) = weak.upgrade() {
        app.clear_history();
      }
    });

    Self {
      root,
      today,
      total,
      list,
      empty,
      clear,
      confirm,
    }
  }

  pub fn update(&self, app: &FocusApp) {
    let texts = &app.texts;
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

    let history = app.history();
    let total_minutes = rounded_minutes(history.total_focused());
    self.total.set_label(&texts.tr_args(
      "history.total",
      [
        (
          "sessions",
          texts.tr_plural(
            "history.sessions",
            i64::try_from(history.total_sessions()).unwrap_or(i64::MAX),
          ),
        ),
        ("hours", (total_minutes / 60).to_string()),
        ("minutes", (total_minutes % 60).to_string()),
      ],
    ));

    while let Some(row) = self.list.first_child() {
      self.list.remove(&row);
    }
    for session in history.recent(RECENT_LIMIT) {
      let started = session.started_at.with_timezone(&Local);
      let ended = session.ended_at.with_timezone(&Local);
      let when = format!(
        "{} · {}–{}",
        app.calendar.short_date(&started),
        app.calendar.time(&started, HourCycle::H24, false).joined(),
        app.calendar.time(&ended, HourCycle::H24, false).joined(),
      );
      let mut length = texts.tr_args(
        "history.minutes",
        [("minutes", rounded_minutes(session.focused()).to_string())],
      );
      if session.completed_early {
        length = format!("{length} · {}", texts.tr("history.early"));
      }
      let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(4)
        .margin_end(4)
        .build();
      row.append(
        &gtk::Label::builder()
          .label(&when)
          .xalign(0.0)
          .hexpand(true)
          .build(),
      );
      row.append(
        &gtk::Label::builder()
          .label(&length)
          .css_classes(["caption", "digits"])
          .build(),
      );
      self.list.append(&row);
    }
    let empty = history.is_empty();
    self.empty.set_visible(empty);
    self.list.set_visible(!empty);
    self.clear.set_sensitive(!empty);
    if empty {
      self.confirm.set_visible(false);
      self.clear.set_visible(true);
    }
  }
}
