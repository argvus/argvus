//! The popup, opened next to the taskbar like the calendar. Five tabs (clock, world, stopwatch, timer, alarms) share a
//! header and a notice banner; settings, the alarm editor and the city search
//! are full pages reached from them. `Esc` returns from a full page.

mod alarm_editor;
mod alarms_page;
mod clock_page;
mod ring_window;
mod settings_page;
mod stopwatch_page;
mod timer_page;
mod world_page;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use argvus_taskbar_popup::{PopupAnchor, PopupController, PopupSize};
use argvus_time_core::alarm::AlarmId;
use argvus_time_ui::text::Catalog;
use gtk::prelude::*;
use gtk::{gdk, glib};
pub use ring_window::RingWindow;

use crate::app::{APP_ID, ClockApp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
  Clock,
  World,
  Stopwatch,
  Timer,
  Alarms,
  Settings,
  AlarmEditor,
  AddCity,
}

impl Page {
  const TABS: [Page; 5] = [
    Page::Clock,
    Page::World,
    Page::Stopwatch,
    Page::Timer,
    Page::Alarms,
  ];

  fn name(self) -> &'static str {
    match self {
      Self::Clock => "clock",
      Self::World => "world",
      Self::Stopwatch => "stopwatch",
      Self::Timer => "timer",
      Self::Alarms => "alarms",
      Self::Settings => "settings",
      Self::AlarmEditor => "alarm-editor",
      Self::AddCity => "add-city",
    }
  }

  fn title_key(self) -> &'static str {
    match self {
      Self::Clock => "page.clock",
      Self::World => "page.world",
      Self::Stopwatch => "page.stopwatch",
      Self::Timer => "page.timer",
      Self::Alarms => "page.alarms",
      Self::Settings => "page.settings",
      Self::AlarmEditor => "page.alarm_editor",
      Self::AddCity => "page.add_city",
    }
  }

  fn is_tab(self) -> bool {
    Self::TABS.contains(&self)
  }

  fn from_name(name: &str) -> Option<Self> {
    [Self::Settings, Self::AlarmEditor, Self::AddCity]
      .into_iter()
      .chain(Self::TABS)
      .find(|page| page.name() == name)
  }
}

/// Width of the popup, the same as the taskbar calendar.
const POPUP_WIDTH: i32 = 398;
/// Height estimate used before the first frame gives the real one.
const POPUP_HEIGHT_ESTIMATE: i32 = 580;
const POPUP_SIZE: PopupSize = PopupSize {
  width: POPUP_WIDTH,
  height_estimate: POPUP_HEIGHT_ESTIMATE,
};
const POINTER_OUTSIDE_DISMISS_DELAY: Duration = Duration::from_secs(5);
/// Fixed height of the page area, so every page has the same popup size.
const PAGES_HEIGHT: i32 = 490;

pub struct ClockWindow {
  pub window: gtk::Window,
  controller: Rc<PopupController>,
  card: gtk::Box,
  /// Where the popup was last asked to open; reused when it is shown again
  /// without a click position (shortcut, notification).
  anchor: Cell<Option<PopupAnchor>>,
  outer: gtk::Stack,
  tabs: gtk::Stack,
  tab_buttons: Vec<(Page, gtk::ToggleButton)>,
  title: gtk::Label,
  back: gtk::Button,
  notice: gtk::Box,
  notice_text: gtk::Label,
  clock: clock_page::ClockPage,
  world: world_page::WorldPage,
  stopwatch: stopwatch_page::StopwatchPage,
  timer: timer_page::TimerPage,
  alarms: alarms_page::AlarmsPage,
  editor: alarm_editor::AlarmEditor,
  settings: settings_page::SettingsPage,
  /// The tab to return to from a full page.
  last_tab: Rc<Cell<Page>>,
}

impl ClockWindow {
  pub fn build(app: &Rc<ClockApp>) -> Self {
    let texts = &app.texts;
    let window = gtk::Window::builder()
      .application(&app.application)
      .title(texts.tr("app.name"))
      .icon_name(APP_ID)
      .default_width(POPUP_WIDTH)
      .resizable(false)
      .decorated(false)
      .focusable(true)
      .css_classes(["argvus-time", "argvus-clock", "argvus-popup"])
      .build();
    PopupController::apply_layer_shell(&window, "argvus-clock");

    let header = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(4)
      .css_classes(["popup-header"])
      .build();
    let back = icon_button("go-previous-symbolic", &texts.tr("action.back"));
    back.set_visible(false);
    let title = gtk::Label::builder()
      .label(texts.tr("app.name"))
      .xalign(0.0)
      .hexpand(true)
      .css_classes(["popup-title"])
      .build();
    let settings_button = icon_button("emblem-system-symbolic", &texts.tr("page.settings"));
    settings_button.set_action_name(Some("app.settings"));
    for button in [&back, &settings_button] {
      button.add_css_class("flat");
    }
    header.append(&back);
    header.append(&title);
    header.append(&settings_button);

    let clock = clock_page::ClockPage::build(app);
    let world = world_page::WorldPage::build(app);
    let stopwatch = stopwatch_page::StopwatchPage::build(app);
    let timer = timer_page::TimerPage::build(app);
    let alarms = alarms_page::AlarmsPage::build(app);
    let editor = alarm_editor::AlarmEditor::build(app);
    let settings = settings_page::SettingsPage::build(app);

    let tabs = gtk::Stack::builder()
      .transition_type(gtk::StackTransitionType::Crossfade)
      .transition_duration(120)
      .vexpand(true)
      .hhomogeneous(false)
      .build();
    for (page, root) in [
      (Page::Clock, clock.root.upcast_ref::<gtk::Widget>()),
      (Page::World, world.root.upcast_ref()),
      (Page::Stopwatch, stopwatch.root.upcast_ref()),
      (Page::Timer, timer.root.upcast_ref()),
      (Page::Alarms, alarms.root.upcast_ref()),
    ] {
      tabs.add_named(root, Some(page.name()));
    }
    // A linked row of toggle buttons, each as wide as its own label (a
    // StackSwitcher makes every button as wide as the longest one).
    let switcher = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Center)
      .margin_top(10)
      .css_classes(["linked", "tabs"])
      .build();
    switcher.update_property(&[gtk::accessible::Property::Label(&texts.tr("a11y.tabs"))]);
    let mut tab_buttons: Vec<(Page, gtk::ToggleButton)> = Vec::new();
    for page in Page::TABS {
      let button = gtk::ToggleButton::with_label(&texts.tr(page.title_key()));
      match tab_buttons.first() {
        Some((_, first)) => button.set_group(Some(first)),
        // The clock tab is shown first; set before the handler is connected.
        None => button.set_active(true),
      }
      let weak = Rc::downgrade(app);
      button.connect_toggled(move |button| {
        if button.is_active()
          && let Some(app) = weak.upgrade()
        {
          app.show_page(page);
        }
      });
      switcher.append(&button);
      tab_buttons.push((page, button));
    }

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

    let main = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .css_classes(["page"])
      .build();
    main.append(&switcher);
    main.append(&tabs);

    let outer = gtk::Stack::builder()
      .transition_type(gtk::StackTransitionType::SlideLeftRight)
      .transition_duration(160)
      .vhomogeneous(true)
      .hhomogeneous(false)
      .build();
    outer.set_size_request(-1, PAGES_HEIGHT);
    outer.add_named(&main, Some("main"));
    outer.add_named(&settings.root, Some(Page::Settings.name()));
    outer.add_named(&editor.root, Some(Page::AlarmEditor.name()));
    outer.add_named(&world.search_root, Some(Page::AddCity.name()));

    let card = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(6)
      .css_classes(["time-card"])
      .build();
    card.set_size_request(POPUP_WIDTH, -1);
    card.append(&header);
    card.append(&notice);
    card.append(&outer);
    window.set_child(Some(&card));

    let controller = PopupController::new();
    let weak = Rc::downgrade(app);
    controller.connect_pointer_dismiss(&window, POINTER_OUTSIDE_DISMISS_DELAY, move || {
      if let Some(app) = weak.upgrade() {
        app.close_window();
      }
    });

    let clock_window = Self {
      window,
      controller,
      card,
      anchor: Cell::new(None),
      outer,
      tabs,
      tab_buttons,
      title,
      back,
      notice,
      notice_text,
      clock,
      world,
      stopwatch,
      timer,
      alarms,
      editor,
      settings,
      last_tab: Rc::new(Cell::new(Page::Clock)),
    };
    clock_window.connect_signals(app, &dismiss);
    clock_window
  }

  /// Shows the popup, at the click position when there is one and where it
  /// last opened otherwise.
  pub fn present(&self, anchor: Option<PopupAnchor>) {
    if anchor.is_some() {
      self.anchor.set(anchor);
    }
    self
      .controller
      .present(&self.window, &self.card, self.anchor.get(), POPUP_SIZE);
  }

  pub fn hide(&self) {
    self.controller.hide(&self.window);
  }

  pub fn reopen_suppressed(&self) -> bool {
    self.controller.reopen_suppressed()
  }

  fn connect_signals(&self, app: &Rc<ClockApp>, dismiss: &gtk::Button) {
    let weak = Rc::downgrade(app);
    dismiss.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.dismiss_notice();
      }
    });

    let weak = Rc::downgrade(app);
    let last_tab = Rc::clone(&self.last_tab);
    self.back.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.show_page(last_tab.get());
      }
    });

    let weak = Rc::downgrade(app);
    self.window.connect_close_request(move |_| {
      if let Some(app) = weak.upgrade() {
        app.close_window();
      }
      glib::Propagation::Stop
    });

    // Switching tabs with the mouse refreshes the newly shown page.
    let weak = Rc::downgrade(app);
    self.tabs.connect_visible_child_name_notify(move |_| {
      if let Some(app) = weak.upgrade() {
        app.refresh();
      }
    });

    let keys = gtk::EventControllerKey::new();
    let weak = Rc::downgrade(app);
    let outer = self.outer.clone();
    let last_tab = Rc::clone(&self.last_tab);
    keys.connect_key_pressed(move |_, key, _, _| {
      let Some(app) = weak.upgrade() else {
        return glib::Propagation::Proceed;
      };
      if key == gdk::Key::Escape {
        // Back from a full page; from a tab, hide the popup.
        if outer.visible_child_name().as_deref() == Some("main") {
          app.close_window();
        } else {
          app.show_page(last_tab.get());
        }
        return glib::Propagation::Stop;
      }
      glib::Propagation::Proceed
    });
    self.window.add_controller(keys);
  }

  pub fn current_page(&self) -> Page {
    match self.outer.visible_child_name().as_deref() {
      Some("main") | None => self
        .tabs
        .visible_child_name()
        .and_then(|name| Page::from_name(&name))
        .unwrap_or(Page::Clock),
      Some(name) => Page::from_name(name).unwrap_or(Page::Clock),
    }
  }

  pub fn show_page(&self, page: Page) {
    // The settings button of the settings page goes back.
    let page = if page == Page::Settings && self.current_page() == Page::Settings {
      self.last_tab.get()
    } else {
      page
    };
    if page.is_tab() {
      self.last_tab.set(page);
      self.tabs.set_visible_child_name(page.name());
      if let Some((_, button)) = self.tab_buttons.iter().find(|(tab, _)| *tab == page) {
        button.set_active(true);
      }
      self.outer.set_visible_child_name("main");
    } else {
      if self.current_page().is_tab() {
        self.last_tab.set(self.current_page());
      }
      self.outer.set_visible_child_name(page.name());
    }
    self.back.set_visible(!page.is_tab());
    if page == Page::AddCity {
      self.world.focus_search();
    }
  }

  pub fn edit_alarm(&self, app: &Rc<ClockApp>, id: Option<AlarmId>) {
    self.editor.load(app, id);
    self.show_page(Page::AlarmEditor);
    self.update(app);
  }

  /// Everything, after a change.
  pub fn update(&self, app: &Rc<ClockApp>) {
    let page = self.current_page();
    let title = if page.is_tab() {
      app.texts.tr("app.name")
    } else {
      app.texts.tr(page.title_key())
    };
    self.title.set_label(&title);

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

    self.world.update(app);
    self.stopwatch.update(app);
    self.timer.update(app);
    self.alarms.update(app);
    self.settings.update(app);
    self.editor.update(app);
    self.update_live(app);
  }

  /// The parts that change every second.
  pub fn update_live(&self, app: &ClockApp) {
    match self.current_page() {
      Page::Clock => self.clock.update(app),
      Page::World => self.world.update_live(app),
      Page::Timer => self.timer.update_live(app),
      Page::Alarms => self.alarms.update_live(app),
      Page::AlarmEditor => self.editor.update(app),
      Page::Stopwatch | Page::Settings | Page::AddCity => {}
    }
  }
}

/// A button showing only an icon, with a tooltip and an accessible name.
pub(crate) fn icon_button(icon: &str, label: &str) -> gtk::Button {
  let button = gtk::Button::builder()
    .icon_name(icon)
    .tooltip_text(label)
    .build();
  button.update_property(&[gtk::accessible::Property::Label(label)]);
  button
}

/// A settings row: a label on the left and its control on the right.
pub(crate) fn labeled_row(label: &str, control: &impl IsA<gtk::Widget>) -> gtk::Box {
  let row = gtk::Box::builder()
    .orientation(gtk::Orientation::Horizontal)
    .spacing(12)
    .build();
  let text = gtk::Label::builder()
    .label(label)
    .xalign(0.0)
    .hexpand(true)
    .wrap(true)
    .build();
  control
    .upcast_ref::<gtk::Widget>()
    .update_relation(&[gtk::accessible::Relation::LabelledBy(&[text.upcast_ref()])]);
  control.set_valign(gtk::Align::Center);
  row.append(&text);
  row.append(control);
  row
}

pub(crate) fn section_title(text: &str) -> gtk::Label {
  gtk::Label::builder()
    .label(text)
    .xalign(0.0)
    .css_classes(["section-title"])
    .margin_top(6)
    .build()
}

pub(crate) fn card() -> gtk::Box {
  gtk::Box::builder()
    .orientation(gtk::Orientation::Vertical)
    .spacing(10)
    .css_classes(["card"])
    .build()
}

pub(crate) fn page_box() -> gtk::Box {
  gtk::Box::builder()
    .orientation(gtk::Orientation::Vertical)
    .spacing(12)
    .margin_top(6)
    .margin_bottom(4)
    .margin_start(2)
    .margin_end(2)
    .build()
}

pub(crate) fn clear_box(container: &gtk::Box) {
  while let Some(child) = container.first_child() {
    container.remove(&child);
  }
}

/// "in 7 h 12 min", "in 12 min" or "in less than a minute".
pub(crate) fn relative_text(texts: &Catalog, until: Duration) -> String {
  let minutes = until.as_secs().div_ceil(60);
  match (minutes / 60, minutes % 60) {
    (0, 0) => texts.tr("relative.soon"),
    (0, minutes) => texts.tr_args("relative.minutes", [("minutes", minutes.to_string())]),
    (hours, minutes) => texts.tr_args(
      "relative.hours",
      [
        ("hours", hours.to_string()),
        ("minutes", minutes.to_string()),
      ],
    ),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn page_names_round_trip() {
    for page in [
      Page::Clock,
      Page::World,
      Page::Stopwatch,
      Page::Timer,
      Page::Alarms,
      Page::Settings,
      Page::AlarmEditor,
      Page::AddCity,
    ] {
      assert_eq!(Page::from_name(page.name()), Some(page));
    }
  }
}
