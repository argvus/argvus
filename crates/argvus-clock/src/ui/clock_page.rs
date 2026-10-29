//! The clock: local time, date, time zone and the next alarm.

use std::rc::Rc;

use argvus_time_core::world::{format_utc_offset, local_zone_name, utc_offset_seconds};
use chrono::{Local, Utc};
use gtk::prelude::*;

use super::{page_box, relative_text};
use crate::app::ClockApp;

pub struct ClockPage {
  pub root: gtk::Box,
  time: gtk::Label,
  period: gtk::Label,
  date: gtk::Label,
  zone: gtk::Label,
  next_alarm: gtk::Label,
}

impl ClockPage {
  pub fn build(_app: &Rc<ClockApp>) -> Self {
    let root = page_box();
    root.set_valign(gtk::Align::Center);
    root.set_vexpand(true);
    let time_row = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(6)
      .halign(gtk::Align::Center)
      .build();
    let time = gtk::Label::builder()
      .css_classes(["display-large", "digits"])
      .build();
    let period = gtk::Label::builder()
      .css_classes(["period"])
      .valign(gtk::Align::Baseline)
      .build();
    time_row.append(&time);
    time_row.append(&period);
    let date = gtk::Label::builder()
      .css_classes(["date"])
      .wrap(true)
      .justify(gtk::Justification::Center)
      .build();
    let zone = gtk::Label::builder().css_classes(["caption"]).build();
    let next_alarm = gtk::Label::builder()
      .css_classes(["caption", "next-alarm"])
      .wrap(true)
      .justify(gtk::Justification::Center)
      .margin_top(18)
      .build();
    root.append(&time_row);
    root.append(&date);
    root.append(&zone);
    root.append(&next_alarm);
    Self {
      root,
      time,
      period,
      date,
      zone,
      next_alarm,
    }
  }

  pub fn update(&self, app: &ClockApp) {
    let settings = app.settings();
    let now = Local::now();
    let time = app
      .calendar
      .time(&now, settings.hour_cycle, settings.show_seconds);
    self.time.set_label(&time.digits);
    self
      .period
      .set_label(time.period.as_deref().unwrap_or_default());
    self.period.set_visible(time.period.is_some());
    self.date.set_label(&app.calendar.long_date(&now));

    let zone_name = local_zone_name().unwrap_or_else(|| app.texts.tr("clock.local_zone"));
    let offset = format_utc_offset(utc_offset_seconds(&Local, Utc::now()));
    self
      .zone
      .set_label(&format!("{} · {offset}", zone_name.replace('_', " ")));

    let next = app.alarms().next_trigger(&Local);
    let text = match next {
      Some((_, trigger)) => {
        let at = trigger.at.with_timezone(&Local);
        let when = if at.date_naive() == now.date_naive() {
          app.local_time(trigger.at)
        } else {
          format!(
            "{} {}",
            app.calendar.weekday_short(chrono::Datelike::weekday(&at)),
            app.local_time(trigger.at)
          )
        };
        let until = (trigger.at - Utc::now()).to_std().unwrap_or_default();
        app.texts.tr_args(
          "clock.next_alarm",
          [
            ("when", when),
            ("relative", relative_text(&app.texts, until)),
          ],
        )
      }
      None => app.texts.tr("clock.no_alarm"),
    };
    self.next_alarm.set_label(&text);
  }
}
