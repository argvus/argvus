//! World clocks: the saved cities and the search to add one.

use std::cell::RefCell;
use std::rc::Rc;

use argvus_time_core::world::{self, DayRelation, WorldClock, region_name};
use chrono::{Local, Utc};
use chrono_tz::Tz;
use gtk::prelude::*;

use super::{Page, icon_button, page_box};
use crate::app::ClockApp;

/// Search results shown at once; typing narrows them.
const SEARCH_LIMIT: usize = 60;

struct LiveRow {
  zone: Tz,
  time: gtk::Label,
  relative: gtk::Label,
}

pub struct WorldPage {
  pub root: gtk::Box,
  pub search_root: gtk::Box,
  list: gtk::ListBox,
  empty: gtk::Label,
  search: gtk::SearchEntry,
  results: gtk::ListBox,
  rows: RefCell<Vec<LiveRow>>,
  /// The clocks the list was built from, to rebuild only on change.
  shown: RefCell<Option<Vec<WorldClock>>>,
}

impl WorldPage {
  pub fn build(app: &Rc<ClockApp>) -> Self {
    let texts = &app.texts;
    let root = page_box();
    let list = gtk::ListBox::builder()
      .selection_mode(gtk::SelectionMode::None)
      .build();
    list.update_property(&[gtk::accessible::Property::Label(&texts.tr("page.world"))]);
    let empty = gtk::Label::builder()
      .label(texts.tr("world.empty"))
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
      .label(texts.tr("world.add"))
      .css_classes(["primary"])
      .halign(gtk::Align::Center)
      .build();
    root.append(&empty);
    root.append(&scrolled);
    root.append(&add);

    let search_root = page_box();
    let search = gtk::SearchEntry::builder()
      .placeholder_text(texts.tr("world.search_placeholder"))
      .build();
    search.update_property(&[gtk::accessible::Property::Label(
      &texts.tr("world.search_placeholder"),
    )]);
    let results = gtk::ListBox::builder()
      .selection_mode(gtk::SelectionMode::Single)
      .build();
    let results_scrolled = gtk::ScrolledWindow::builder()
      .hscrollbar_policy(gtk::PolicyType::Never)
      .vexpand(true)
      .child(&results)
      .build();
    search_root.append(&search);
    search_root.append(&results_scrolled);

    let page = Self {
      root,
      search_root,
      list,
      empty,
      search,
      results,
      rows: RefCell::default(),
      shown: RefCell::default(),
    };

    let weak = Rc::downgrade(app);
    add.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.show_page(Page::AddCity);
      }
    });
    let results = page.results.clone();
    page
      .search
      .connect_search_changed(move |entry| fill_results(&results, &entry.text()));
    // Enter in the search field adds the first result.
    let results = page.results.clone();
    page.search.connect_activate(move |_| {
      if let Some(row) = results.row_at_index(0) {
        row.emit_activate();
      }
    });
    let weak = Rc::downgrade(app);
    page.results.connect_row_activated(move |_, row| {
      let zone = row.widget_name();
      if let (Some(app), Some(zone)) = (weak.upgrade(), world::parse_zone(&zone)) {
        app.update_settings(|settings| {
          if !settings
            .world_clocks
            .iter()
            .any(|clock| clock.zone == zone.name())
          {
            settings.world_clocks.push(WorldClock::new(zone));
          }
        });
        app.show_page(Page::World);
      }
    });
    fill_results(&page.results, "");
    page
  }

  pub fn focus_search(&self) {
    self.search.set_text("");
    self.search.grab_focus();
  }

  pub fn update(&self, app: &Rc<ClockApp>) {
    let clocks = app.settings().world_clocks;
    if self.shown.borrow().as_ref() == Some(&clocks) {
      return;
    }
    self.list.remove_all();
    let mut rows = Vec::new();
    let count = clocks.len();
    for (index, clock) in clocks.iter().enumerate() {
      let Some(zone) = clock.time_zone() else {
        continue;
      };
      let (row, live) = self.clock_row(app, clock, zone, index, count);
      self.list.append(&row);
      rows.push(live);
    }
    self.empty.set_visible(clocks.is_empty());
    self.list.set_visible(!clocks.is_empty());
    *self.rows.borrow_mut() = rows;
    *self.shown.borrow_mut() = Some(clocks);
    self.update_live(app);
  }

  fn clock_row(
    &self,
    app: &Rc<ClockApp>,
    clock: &WorldClock,
    zone: Tz,
    index: usize,
    count: usize,
  ) -> (gtk::Box, LiveRow) {
    let texts = &app.texts;
    let row = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(10)
      .margin_top(8)
      .margin_bottom(8)
      .margin_start(4)
      .margin_end(4)
      .build();
    let names = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .hexpand(true)
      .build();
    names.append(
      &gtk::Label::builder()
        .label(clock.display_name())
        .xalign(0.0)
        .css_classes(["city"])
        .build(),
    );
    let relative = gtk::Label::builder()
      .xalign(0.0)
      .css_classes(["caption"])
      .build();
    names.append(&relative);
    let time = gtk::Label::builder()
      .css_classes(["world-time", "digits"])
      .build();

    let up = icon_button("go-up-symbolic", &texts.tr("world.move_up"));
    let down = icon_button("go-down-symbolic", &texts.tr("world.move_down"));
    let remove = icon_button("edit-delete-symbolic", &texts.tr("world.remove"));
    for button in [&up, &down, &remove] {
      button.add_css_class("flat");
    }
    up.set_sensitive(index > 0);
    down.set_sensitive(index + 1 < count);
    let moves: [(&gtk::Button, Option<usize>); 3] = [
      (&up, index.checked_sub(1)),
      (&down, (index + 1 < count).then_some(index + 1)),
      (&remove, None),
    ];
    for (button, target) in moves {
      let weak = Rc::downgrade(app);
      let is_remove = button == &remove;
      button.connect_clicked(move |_| {
        let Some(app) = weak.upgrade() else {
          return;
        };
        app.update_settings(|settings| {
          if is_remove {
            settings.world_clocks.remove(index);
          } else if let Some(target) = target {
            settings.world_clocks.swap(index, target);
          }
        });
      });
    }

    row.append(&names);
    row.append(&time);
    row.append(&up);
    row.append(&down);
    row.append(&remove);
    (
      row,
      LiveRow {
        zone,
        time,
        relative,
      },
    )
  }

  pub fn update_live(&self, app: &ClockApp) {
    let settings = app.settings();
    let now = Utc::now();
    let local_date = now.with_timezone(&Local).date_naive();
    let local_offset = world::utc_offset_seconds(&Local, now);
    for row in self.rows.borrow().iter() {
      let there = now.with_timezone(&row.zone);
      row.time.set_label(
        &app
          .calendar
          .time(&there, settings.hour_cycle, false)
          .joined(),
      );
      let difference = world::utc_offset_seconds(&row.zone, now) - local_offset;
      let day = match world::day_relation(local_date, there.date_naive()) {
        DayRelation::Yesterday => app.texts.tr("world.yesterday"),
        DayRelation::Today => app.texts.tr("world.today"),
        DayRelation::Tomorrow => app.texts.tr("world.tomorrow"),
      };
      row
        .relative
        .set_label(&format!("{day} · {}", offset_text(app, difference)));
    }
  }
}

/// "+3 h", "-1 h 30 min" or "same time" relative to the local zone.
fn offset_text(app: &ClockApp, seconds: i32) -> String {
  if seconds == 0 {
    return app.texts.tr("world.same_time");
  }
  let sign = if seconds < 0 { "-" } else { "+" };
  let minutes = seconds.unsigned_abs() / 60;
  let text = match (minutes / 60, minutes % 60) {
    (hours, 0) => format!("{sign}{hours} h"),
    (0, minutes) => format!("{sign}{minutes} min"),
    (hours, minutes) => format!("{sign}{hours} h {minutes} min"),
  };
  app.texts.tr_args("world.offset", [("offset", text)])
}

fn fill_results(results: &gtk::ListBox, query: &str) {
  results.remove_all();
  for zone in world::search(query).into_iter().take(SEARCH_LIMIT) {
    let row = gtk::ListBoxRow::builder().name(zone.name()).build();
    let content = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .margin_top(6)
      .margin_bottom(6)
      .margin_start(6)
      .build();
    content.append(
      &gtk::Label::builder()
        .label(world::city_name(zone.name()))
        .xalign(0.0)
        .build(),
    );
    content.append(
      &gtk::Label::builder()
        .label(region_name(zone.name()))
        .xalign(0.0)
        .css_classes(["caption"])
        .build(),
    );
    row.set_child(Some(&content));
    results.append(&row);
  }
}
