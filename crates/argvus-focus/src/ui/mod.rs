//! The popup: a card with a header and three pages (timer, settings, history)
//! in a stack, opened next to the taskbar like the calendar. Settings and
//! history are pages, not dialogs; `Esc` goes back, and hides the popup from
//! the timer page.

mod history;
mod settings;
mod timer;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use argvus_taskbar_popup::{PopupAnchor, PopupController, PopupSize};
use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::app::{APP_ID, FocusApp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
  Timer,
  Settings,
  History,
}

impl Page {
  fn name(self) -> &'static str {
    match self {
      Self::Timer => "timer",
      Self::Settings => "settings",
      Self::History => "history",
    }
  }
}

/// Width of the popup, the same as the taskbar calendar.
const POPUP_WIDTH: i32 = 398;
/// Height estimate used before the first frame gives the real one.
const POPUP_HEIGHT_ESTIMATE: i32 = 560;
const POPUP_SIZE: PopupSize = PopupSize {
  width: POPUP_WIDTH,
  height_estimate: POPUP_HEIGHT_ESTIMATE,
};
const POINTER_OUTSIDE_DISMISS_DELAY: Duration = Duration::from_secs(5);
/// Fixed height of the page area, so every page has the same popup size.
const PAGES_HEIGHT: i32 = 470;

pub struct FocusWindow {
  pub window: gtk::Window,
  controller: Rc<PopupController>,
  card: gtk::Box,
  /// Where the popup was last asked to open; reused when it is shown again
  /// without a click position (shortcut, notification).
  anchor: Cell<Option<PopupAnchor>>,
  stack: gtk::Stack,
  title: gtk::Label,
  back: gtk::Button,
  timer: timer::TimerPage,
  settings: settings::SettingsPage,
  history: history::HistoryPage,
  page: Cell<Page>,
}

impl FocusWindow {
  pub fn build(app: &Rc<FocusApp>) -> Self {
    let texts = &app.texts;
    let window = gtk::Window::builder()
      .application(&app.application)
      .title(texts.tr("app.name"))
      .icon_name(APP_ID)
      .default_width(POPUP_WIDTH)
      .resizable(false)
      .decorated(false)
      .focusable(true)
      .css_classes(["argvus-time", "argvus-focus", "argvus-popup"])
      .build();
    PopupController::apply_layer_shell(&window, "argvus-focus");

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
    let history_button = icon_button("document-open-recent-symbolic", &texts.tr("action.history"));
    history_button.set_action_name(Some("app.history"));
    let settings_button = icon_button("emblem-system-symbolic", &texts.tr("action.settings"));
    settings_button.set_action_name(Some("app.settings"));
    for button in [&back, &history_button, &settings_button] {
      button.add_css_class("flat");
    }
    header.append(&back);
    header.append(&title);
    header.append(&history_button);
    header.append(&settings_button);

    let timer = timer::TimerPage::build(app);
    let settings = settings::SettingsPage::build(app);
    let history = history::HistoryPage::build(app);
    let stack = gtk::Stack::builder()
      .transition_type(gtk::StackTransitionType::SlideLeftRight)
      .transition_duration(160)
      .vhomogeneous(true)
      .build();
    stack.set_size_request(-1, PAGES_HEIGHT);
    stack.add_named(&timer.root, Some(Page::Timer.name()));
    stack.add_named(&settings.root, Some(Page::Settings.name()));
    stack.add_named(&history.root, Some(Page::History.name()));

    let card = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(6)
      .css_classes(["time-card"])
      .build();
    card.set_size_request(POPUP_WIDTH, -1);
    card.append(&header);
    card.append(&stack);
    window.set_child(Some(&card));

    let controller = PopupController::new();
    let weak = Rc::downgrade(app);
    controller.connect_pointer_dismiss(&window, POINTER_OUTSIDE_DISMISS_DELAY, move || {
      if let Some(app) = weak.upgrade() {
        app.close_window();
      }
    });

    let focus_window = Self {
      window,
      controller,
      card,
      anchor: Cell::new(None),
      stack,
      title,
      back,
      timer,
      settings,
      history,
      page: Cell::new(Page::Timer),
    };
    focus_window.connect_signals(app);
    focus_window
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

  fn connect_signals(&self, app: &Rc<FocusApp>) {
    let weak = Rc::downgrade(app);
    self.back.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.show_page(Page::Timer);
      }
    });

    let weak = Rc::downgrade(app);
    self.window.connect_close_request(move |_| {
      if let Some(app) = weak.upgrade() {
        app.close_window();
      }
      glib::Propagation::Stop
    });

    // Space starts or pauses from anywhere on the timer page; Esc leaves a
    // sub-page, or hides the popup from the timer page. Text fields and spin
    // buttons keep their own keys.
    let keys = gtk::EventControllerKey::new();
    let weak = Rc::downgrade(app);
    let stack = self.stack.clone();
    let window = self.window.clone();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
      let Some(app) = weak.upgrade() else {
        return glib::Propagation::Proceed;
      };
      let on_timer = stack.visible_child_name().as_deref() == Some(Page::Timer.name());
      let typing = gtk::prelude::GtkWindowExt::focus(&window)
        .is_some_and(|widget| widget.is::<gtk::Editable>() || widget.is::<gtk::Text>());
      match key {
        gdk::Key::Escape if on_timer => {
          app.close_window();
          glib::Propagation::Stop
        }
        gdk::Key::Escape => {
          app.show_page(Page::Timer);
          glib::Propagation::Stop
        }
        gdk::Key::space if on_timer && !typing && modifiers.is_empty() => {
          app.toggle_session();
          glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
      }
    });
    self.window.add_controller(keys);
  }

  pub fn show_page(&self, page: Page) {
    // Pressing the button of the page already shown goes back to the timer.
    let target = if page == self.page.get() && page != Page::Timer {
      Page::Timer
    } else {
      page
    };
    self.page.set(target);
    self.stack.set_visible_child_name(target.name());
    self.back.set_visible(target != Page::Timer);
    if target == Page::Timer {
      self.timer.focus_primary();
    }
  }

  pub fn update(&self, app: &FocusApp) {
    let title = match self.page.get() {
      Page::Timer => app.texts.tr("app.name"),
      Page::Settings => app.texts.tr("settings.title"),
      Page::History => app.texts.tr("history.title"),
    };
    self.title.set_label(&title);
    self.timer.update(app);
    match self.page.get() {
      Page::Settings => self.settings.update(app),
      Page::History => self.history.update(app),
      Page::Timer => {}
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
