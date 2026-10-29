//! Relm4 component of the calendar popup: model, messages and their handling.

use std::collections::HashSet;
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};
use std::{
  cell::{Cell, RefCell},
  path::Path,
  rc::Rc,
};

use chrono::{Datelike, Days, Local, NaiveDate};
use gtk::glib::clone;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmApp, SimpleComponent};

use crate::calendar::{
  CalendarEvent, day_bounds, first_of_month, next_month, previous_month, shift_year_clamped, today,
};
use crate::config::{Paths, Settings, load_events_enabled, save_events_enabled};
use crate::i18n::{I18n, Language, Text};
use crate::ipc::{IpcCommand, PopupAnchor};
use crate::storage::Database;

use super::agenda::rebuild_agenda;
use super::calendar::{day_cells, weekday_labels};
use super::event_editor::{
  EditorChangedHandlers, EditorDraft, EditorWidgets, build_editor, populate_editor, save_editor,
};
use super::settings::{
  SettingsChangedHandlers, SettingsDraft, SettingsWidgets, build_settings, dropdown_value,
  populate_settings,
};

mod build;
mod drafts;
mod form_widgets;
mod popup;
mod render;
mod styling;

pub(crate) use argvus_taskbar_popup::geometry::current_pointer_position;
use argvus_taskbar_popup::{PopupController, PopupSize};
use build::{
  build_calendar_view, build_editor_view, build_settings_view, connect_focus_dismiss,
  install_keyboard_shortcuts, serve_ipc, watch_theme,
};
use drafts::{
  apply_editor_field, apply_settings_field, can_create_event, cancel_editor,
  control_reminder_service,
};
use form_widgets::{
  editor_control_has_focus, editor_entry, editor_label, editor_number_spin, editor_row,
  settings_label, settings_number_spin, settings_section_label,
};
use popup::{apply_layer_shell, hide_popup, present_popup};
use render::{load_day_and_month, render_model};
use styling::{load_css, theme_fingerprint};

const POPUP_WIDTH: i32 = 398;
const POPUP_HEIGHT_ESTIMATE: i32 = 520;
const POPUP_SIZE: PopupSize = PopupSize {
  width: POPUP_WIDTH,
  height_estimate: POPUP_HEIGHT_ESTIMATE,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeFingerprint {
  cache_mtime: i64,
  cache_len: u64,
  active_mtime: i64,
  active_len: u64,
  accent_mtime: i64,
  accent_len: u64,
}

#[derive(Debug, Clone)]
pub struct AppInit {
  pub paths: Paths,
  pub settings: Settings,
  pub command: PopupCommand,
  /// Pointer position captured by the Waybar launcher before application
  /// startup, so later pointer movement cannot relocate the popup.
  pub fixed_position: Option<PopupAnchor>,
}

#[derive(Debug, Clone, Copy)]
pub enum PopupCommand {
  Show,
  Toggle,
}

#[derive(Debug)]
pub enum AppMsg {
  PreviousMonth,
  NextMonth,
  PreviousYear,
  NextYear,
  Today,
  MoveDays(i64),
  SelectDay(NaiveDate),
  OpenNewEvent,
  OpenEditEvent(i64),
  EditorTitle(String),
  EditorLocation(String),
  EditorDescription(String),
  EditorStartHour(u32),
  EditorStartMinute(u32),
  EditorEndHour(u32),
  EditorEndMinute(u32),
  EditorAllDay(bool),
  EditorReminderEnabled(bool),
  EditorReminderHours(u32),
  EditorReminderMinutes(u32),
  EditorRepeatAllDay(bool),
  EditorRepeatHours(u32),
  EditorRepeatMinutes(u32),
  SaveEditor,
  DeleteEvent,
  CancelEditor,
  OpenConfig,
  OpenSettings,
  CancelSettings,
  SaveSettings,
  SettingsFontFamily(String),
  SettingsFontSize(u32),
  SettingsLanguage(String),
  SettingsWeekStart(String),
  SettingsShowEvents(bool),
  SettingsDefaultDuration(i64),
  SettingsDefaultReminder(i64),
  SettingsSyncInterval(u64),
  SettingsEditorCommand(String),
  SettingsEditorArgs(String),
  SettingsTerminalCommand(String),
  SettingsTerminalArgs(String),
  ReloadConfig,
  Toggle(Option<PopupAnchor>),
  Show(Option<PopupAnchor>),
  Hide,
  ClickOutside,
  ReloadTheme,
}

impl AppMsg {
  /// Whether this message moves around the month grid or opens a new event.
  /// Those come from the calendar page and its shortcuts.
  fn is_calendar_navigation(&self) -> bool {
    matches!(
      self,
      AppMsg::PreviousMonth
        | AppMsg::NextMonth
        | AppMsg::PreviousYear
        | AppMsg::NextYear
        | AppMsg::Today
        | AppMsg::MoveDays(_)
        | AppMsg::SelectDay(_)
        | AppMsg::OpenNewEvent
    )
  }

  /// Whether the message applies to `view`. Calendar shortcuts are ignored on
  /// the editor and settings pages, where they would discard the draft.
  fn applies_to(&self, view: &ViewState) -> bool {
    !self.is_calendar_navigation() || matches!(view, ViewState::Calendar)
  }

  /// Whether handling this message can change the selected day, the visible
  /// month or the stored events, so the events must be read again. Popup,
  /// theme, settings and in-progress editor messages leave them unchanged.
  fn reloads_events(&self) -> bool {
    !matches!(
      self,
      AppMsg::Toggle(_)
        | AppMsg::Show(_)
        | AppMsg::Hide
        | AppMsg::ClickOutside
        | AppMsg::ReloadTheme
        | AppMsg::OpenConfig
        | AppMsg::OpenSettings
        | AppMsg::CancelSettings
        | AppMsg::SaveSettings
        | AppMsg::ReloadConfig
        | AppMsg::OpenEditEvent(_)
        | AppMsg::EditorTitle(_)
        | AppMsg::EditorLocation(_)
        | AppMsg::EditorDescription(_)
        | AppMsg::EditorStartHour(_)
        | AppMsg::EditorStartMinute(_)
        | AppMsg::EditorEndHour(_)
        | AppMsg::EditorEndMinute(_)
        | AppMsg::EditorAllDay(_)
        | AppMsg::EditorReminderEnabled(_)
        | AppMsg::EditorReminderHours(_)
        | AppMsg::EditorReminderMinutes(_)
        | AppMsg::EditorRepeatAllDay(_)
        | AppMsg::EditorRepeatHours(_)
        | AppMsg::EditorRepeatMinutes(_)
        | AppMsg::SettingsFontFamily(_)
        | AppMsg::SettingsFontSize(_)
        | AppMsg::SettingsLanguage(_)
        | AppMsg::SettingsWeekStart(_)
        | AppMsg::SettingsShowEvents(_)
        | AppMsg::SettingsDefaultDuration(_)
        | AppMsg::SettingsDefaultReminder(_)
        | AppMsg::SettingsSyncInterval(_)
        | AppMsg::SettingsEditorCommand(_)
        | AppMsg::SettingsEditorArgs(_)
        | AppMsg::SettingsTerminalCommand(_)
        | AppMsg::SettingsTerminalArgs(_)
    )
  }
}

#[derive(Debug, Clone)]
pub enum ViewState {
  Calendar,
  Editor(Box<EditorDraft>),
  Settings(Box<SettingsDraft>),
}

pub struct AppModel {
  paths: Paths,
  settings: Settings,
  window: gtk::Window,
  popup: gtk::Box,
  month: NaiveDate,
  selected: NaiveDate,
  events: Vec<CalendarEvent>,
  event_days: HashSet<NaiveDate>,
  events_enabled: bool,
  fixed_position: Option<PopupAnchor>,
  view: ViewState,
  i18n: I18n,
  popup_state: Rc<PopupController>,
  _theme_watcher: Rc<Cell<ThemeFingerprint>>,
  css_providers: Rc<RefCell<Vec<gtk::CssProvider>>>,
}

pub struct AppWidgets {
  popup: gtk::Box,
  stack: gtk::Stack,
  title: gtk::Label,
  day_buttons: Vec<gtk::Button>,
  agenda_separator: gtk::Separator,
  agenda_header: gtk::Box,
  agenda_title: gtk::Label,
  add_button: gtk::Button,
  prev_year: gtk::Button,
  prev_month: gtk::Button,
  today_button: gtk::Button,
  next_month: gtk::Button,
  next_year: gtk::Button,
  settings_button: gtk::Button,
  agenda: gtk::ListBox,
  agenda_scroll: gtk::ScrolledWindow,
  editor: EditorWidgets,
  settings: SettingsWidgets,
}

impl SimpleComponent for AppModel {
  type Init = AppInit;
  type Input = AppMsg;
  type Output = ();
  type Root = gtk::Window;
  type Widgets = AppWidgets;

  fn init_root() -> Self::Root {
    gtk::Window::builder()
      .title("Argvus Calendar")
      .default_width(POPUP_WIDTH)
      .resizable(false)
      .decorated(false)
      .focusable(true)
      .build()
  }

  fn init(
    init: Self::Init,
    window: Self::Root,
    sender: ComponentSender<Self>,
  ) -> ComponentParts<Self> {
    apply_layer_shell(&window);
    let css_providers = Rc::new(RefCell::new(Vec::new()));
    load_css(&init.paths, &css_providers);

    let selected = today();
    let month = first_of_month(selected);
    let (events, event_days) = load_day_and_month(&init.paths, selected, month);
    let i18n = I18n::new(Language::resolve(&init.settings.locale.language));
    let events_enabled = load_events_enabled(&init.paths, &init.settings);
    let popup = gtk::Box::new(gtk::Orientation::Vertical, 0);
    popup.add_css_class("argvus-calendar");
    popup.set_size_request(POPUP_WIDTH, -1);
    popup.set_hexpand(true);
    popup.set_vexpand(false);
    let popup_state = PopupController::new();
    let theme_watcher = Rc::new(Cell::new(theme_fingerprint(&init.paths)));
    let model = AppModel {
      window: window.clone(),
      popup: popup.clone(),
      paths: init.paths,
      settings: init.settings,
      fixed_position: init.fixed_position,
      month,
      selected,
      events,
      event_days,
      events_enabled,
      view: ViewState::Calendar,
      i18n,
      popup_state: popup_state.clone(),
      _theme_watcher: theme_watcher.clone(),
      css_providers,
    };

    watch_theme(model.paths.clone(), theme_watcher, sender.clone());
    serve_ipc(&model.paths, &sender);

    window.add_css_class("argvus-calendar-window");

    let stack = gtk::Stack::new();
    stack.add_css_class("content-stack");
    stack.set_hhomogeneous(true);
    stack.set_vhomogeneous(false);

    let calendar = build_calendar_view(&model.i18n, &model.settings, &sender);
    let (editor_root, editor) = build_editor_view(&model.i18n, &sender);
    let (settings_scroll, settings) = build_settings_view(&model.i18n, &sender);

    stack.add_named(&calendar.root, Some("calendar"));
    stack.add_named(&editor_root, Some("editor"));
    stack.add_named(&settings_scroll, Some("settings"));
    popup.append(&stack);
    window.set_child(Some(&popup));

    connect_focus_dismiss(&window, &sender, &popup_state);
    install_keyboard_shortcuts(&window, &sender);

    let mut widgets = AppWidgets {
      popup,
      stack,
      title: calendar.title,
      day_buttons: calendar.day_buttons,
      agenda_separator: calendar.agenda_separator,
      agenda_header: calendar.agenda_header,
      agenda_title: calendar.agenda_title,
      add_button: calendar.add_button,
      prev_year: calendar.prev_year,
      prev_month: calendar.prev_month,
      today_button: calendar.today_button,
      next_month: calendar.next_month,
      next_year: calendar.next_year,
      settings_button: calendar.settings_button,
      agenda: calendar.agenda,
      agenda_scroll: calendar.agenda_scroll,
      editor,
      settings,
    };
    render_model(&model, &mut widgets, sender.clone());
    if matches!(init.command, PopupCommand::Show | PopupCommand::Toggle) {
      present_popup(
        &window,
        &widgets.popup,
        &model.popup_state,
        model.fixed_position,
      );
    }

    ComponentParts { model, widgets }
  }

  fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
    if !msg.applies_to(&self.view) {
      return;
    }
    let should_reload = msg.reloads_events();
    match msg {
      AppMsg::PreviousMonth => self.month = previous_month(self.month),
      AppMsg::NextMonth => self.month = next_month(self.month),
      AppMsg::PreviousYear => {
        self.selected = shift_year_clamped(self.selected, -1);
        self.month = first_of_month(self.selected);
      }
      AppMsg::NextYear => {
        self.selected = shift_year_clamped(self.selected, 1);
        self.month = first_of_month(self.selected);
      }
      AppMsg::Today => {
        self.selected = today();
        self.month = first_of_month(self.selected);
      }
      AppMsg::MoveDays(days) => {
        let next = if days.is_negative() {
          self
            .selected
            .checked_sub_days(Days::new(days.unsigned_abs()))
        } else {
          self.selected.checked_add_days(Days::new(days as u64))
        };
        if let Some(next) = next {
          self.selected = next;
          self.month = first_of_month(next);
        }
      }
      AppMsg::SelectDay(date) => {
        self.selected = date;
        self.month = first_of_month(date);
      }
      AppMsg::OpenNewEvent => {
        if self.events_enabled && can_create_event(self.selected) {
          self.view = ViewState::Editor(Box::new(EditorDraft::new(
            self.selected,
            &self.settings.calendar,
          )));
        }
      }
      AppMsg::OpenEditEvent(id) => {
        if self.events_enabled
          && let Ok(db) = Database::open(&self.paths.database)
          && let Ok(Some(event)) = db.event_by_id(id)
        {
          self.view = ViewState::Editor(Box::new(EditorDraft::from_event(event)));
        }
      }
      field @ (AppMsg::EditorTitle(_)
      | AppMsg::EditorLocation(_)
      | AppMsg::EditorDescription(_)
      | AppMsg::EditorStartHour(_)
      | AppMsg::EditorStartMinute(_)
      | AppMsg::EditorEndHour(_)
      | AppMsg::EditorEndMinute(_)
      | AppMsg::EditorAllDay(_)
      | AppMsg::EditorReminderEnabled(_)
      | AppMsg::EditorReminderHours(_)
      | AppMsg::EditorReminderMinutes(_)
      | AppMsg::EditorRepeatAllDay(_)
      | AppMsg::EditorRepeatHours(_)
      | AppMsg::EditorRepeatMinutes(_)) => apply_editor_field(&mut self.view, field),
      AppMsg::SaveEditor => self.save_editor(),
      AppMsg::DeleteEvent => self.delete_edited_event(),
      AppMsg::CancelEditor => {
        self.view = ViewState::Calendar;
      }
      AppMsg::OpenSettings => {
        self.view = ViewState::Settings(Box::new(SettingsDraft::from_config(
          &self.settings,
          self.events_enabled,
        )));
      }
      AppMsg::CancelSettings => {
        self.view = ViewState::Calendar;
      }
      field @ (AppMsg::SettingsFontFamily(_)
      | AppMsg::SettingsFontSize(_)
      | AppMsg::SettingsLanguage(_)
      | AppMsg::SettingsWeekStart(_)
      | AppMsg::SettingsShowEvents(_)
      | AppMsg::SettingsDefaultDuration(_)
      | AppMsg::SettingsDefaultReminder(_)
      | AppMsg::SettingsSyncInterval(_)
      | AppMsg::SettingsEditorCommand(_)
      | AppMsg::SettingsEditorArgs(_)
      | AppMsg::SettingsTerminalCommand(_)
      | AppMsg::SettingsTerminalArgs(_)) => apply_settings_field(&mut self.view, field),
      AppMsg::SaveSettings => self.save_settings(),
      AppMsg::ReloadConfig => {
        if let Ok(settings) = Settings::load(&self.paths) {
          self.settings = settings;
        }
        self.view = ViewState::Calendar;
        self.reapply_runtime();
      }
      AppMsg::OpenConfig => {
        let _ = crate::config::open_config(&self.paths, &self.settings);
      }
      AppMsg::Toggle(fixed_position) => self.toggle_popup(fixed_position),
      AppMsg::Show(fixed_position) => self.show_popup(fixed_position),
      AppMsg::Hide => {
        hide_popup(&mut self.view, &self.window, &self.popup_state);
      }
      AppMsg::ClickOutside => {
        hide_popup(&mut self.view, &self.window, &self.popup_state);
      }
      AppMsg::ReloadTheme => {
        load_css(&self.paths, &self.css_providers);
      }
    }
    if should_reload {
      let loaded = load_day_and_month(&self.paths, self.selected, self.month);
      self.events = loaded.0;
      self.event_days = loaded.1;
    }
  }

  fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
    render_model(self, widgets, sender);
  }
}

impl AppModel {
  /// Saves the editor draft and returns to the calendar. On failure the
  /// editor stays open, so the typed values are not lost.
  fn save_editor(&mut self) {
    if let ViewState::Editor(draft) = &self.view {
      let draft = *draft.clone();
      match save_editor(&self.paths.database, &self.settings, draft, self.i18n) {
        Ok(()) => self.view = ViewState::Calendar,
        Err(err) => log::warn!("could not save the event: {err}"),
      }
    }
  }

  /// Soft-deletes the event open in the editor (a new event has no id yet)
  /// and returns to the calendar.
  fn delete_edited_event(&mut self) {
    if let ViewState::Editor(draft) = &self.view {
      if let super::event_editor::EditorKind::Edit(event) = &draft.kind
        && let Some(id) = event.id
        && let Ok(db) = Database::open(&self.paths.database)
      {
        let _ = db.mark_deleted(id);
      }
      self.view = ViewState::Calendar;
    }
  }

  /// Saves the settings draft. Turning events on or off is stored apart from
  /// the configuration file and also starts or stops the reminder service.
  fn save_settings(&mut self) {
    if let ViewState::Settings(draft) = &self.view {
      let draft = *draft.clone();
      let config = draft.into_config();
      if self.events_enabled != config.calendar.show_events {
        self.events_enabled = config.calendar.show_events;
        let _ = save_events_enabled(&self.paths, self.events_enabled);
        control_reminder_service(self.events_enabled);
      }
      if config.save(&self.paths).is_ok() {
        self.settings = config;
      }
      self.view = ViewState::Calendar;
      self.reapply_runtime();
    }
  }

  fn toggle_popup(&mut self, fixed_position: Option<PopupAnchor>) {
    if self.window.is_visible() {
      hide_popup(&mut self.view, &self.window, &self.popup_state);
    } else {
      self.fixed_position = fixed_position;
      cancel_editor(&mut self.view);
      load_css(&self.paths, &self.css_providers);
      present_popup(
        &self.window,
        &self.popup,
        &self.popup_state,
        self.fixed_position,
      );
    }
  }

  /// Shows the popup at the new anchor. An already visible popup keeps the
  /// view it is on; a hidden one opens on the calendar.
  fn show_popup(&mut self, fixed_position: Option<PopupAnchor>) {
    self.fixed_position = fixed_position;
    if !self.window.is_visible() {
      cancel_editor(&mut self.view);
    }
    load_css(&self.paths, &self.css_providers);
    present_popup(
      &self.window,
      &self.popup,
      &self.popup_state,
      self.fixed_position,
    );
  }

  fn reapply_runtime(&mut self) {
    self.i18n = I18n::new(Language::resolve(&self.settings.locale.language));
    load_css(&self.paths, &self.css_providers);
    apply_layer_shell(&self.window);
    if self.window.is_visible() {
      present_popup(
        &self.window,
        &self.popup,
        &self.popup_state,
        self.fixed_position,
      );
    }
  }
}

pub fn run_ui(
  paths: Paths,
  settings: Settings,
  command: PopupCommand,
  fixed_position: Option<PopupAnchor>,
) {
  let fixed_position = fixed_position.or_else(|| {
    current_pointer_position().map(|(x, y)| PopupAnchor {
      x,
      y,
      bar_bottom: None,
      bar_right: None,
    })
  });
  let app =
    RelmApp::new("sh.argvus.Calendar").with_args(vec!["argvus-taskbar-calendar".to_string()]);
  app.allow_multiple_instances(true);
  let application = relm4::main_application();
  let _application_hold = application.hold();
  app.run::<AppModel>(AppInit {
    paths,
    settings,
    command,
    fixed_position,
  });
}

#[cfg(test)]
mod tests {
  use super::drafts::update_draft;
  use super::styling::stylesheet_paths;
  use chrono::NaiveDate;

  use super::*;

  #[test]
  fn view_transitions_calendar_new_cancel_calendar() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
    let view = ViewState::Editor(Box::new(EditorDraft::new(
      date,
      &Settings::default().calendar,
    )));
    assert!(matches!(view, ViewState::Editor(_)));
    let view = ViewState::Calendar;
    assert!(matches!(view, ViewState::Calendar));
  }

  #[test]
  fn draft_updates_are_applied() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
    let mut view = ViewState::Editor(Box::new(EditorDraft::new(
      date,
      &Settings::default().calendar,
    )));
    update_draft(&mut view, |draft| draft.title = "Meeting".to_string());
    let ViewState::Editor(draft) = view else {
      panic!("expected editor");
    };
    assert_eq!(draft.title, "Meeting");
  }

  #[test]
  fn calendar_shortcuts_are_ignored_outside_the_calendar() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    let editor = ViewState::Editor(Box::new(EditorDraft::new(
      date,
      &Settings::default().calendar,
    )));
    let settings = ViewState::Settings(Box::new(SettingsDraft::from_config(
      &Settings::default(),
      true,
    )));
    for message in [
      AppMsg::OpenNewEvent,
      AppMsg::Today,
      AppMsg::MoveDays(1),
      AppMsg::NextMonth,
    ] {
      assert!(message.applies_to(&ViewState::Calendar));
      assert!(!message.applies_to(&editor));
      assert!(!message.applies_to(&settings));
    }
    assert!(AppMsg::SaveEditor.applies_to(&editor));
    assert!(AppMsg::Hide.applies_to(&settings));
  }

  #[test]
  fn cancel_editor_restores_calendar_view() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    let mut view = ViewState::Editor(Box::new(EditorDraft::new(
      date,
      &Settings::default().calendar,
    )));
    cancel_editor(&mut view);
    assert!(matches!(view, ViewState::Calendar));
  }

  #[test]
  fn event_creation_is_blocked_before_today() {
    assert!(!can_create_event(
      today().checked_sub_days(Days::new(1)).unwrap()
    ));
    assert!(can_create_event(today()));
    assert!(can_create_event(
      today().checked_add_days(Days::new(1)).unwrap()
    ));
  }

  #[test]
  fn current_calendar_theme_cache_is_loaded() {
    let root = std::env::temp_dir().join(format!("argvus-calendar-theme-{}", uuid::Uuid::new_v4()));
    let paths = test_theme_paths(&root);
    std::fs::create_dir_all(paths.active_argvus_theme_file.parent().unwrap()).unwrap();
    std::fs::create_dir_all(paths.cache_theme_file.parent().unwrap()).unwrap();
    std::fs::write(&paths.active_argvus_theme_file, "gruvbox-dark-float\n").unwrap();
    std::fs::write(
      &paths.cache_theme_file,
      "/* argvus-theme: gruvbox-dark-float */\n",
    )
    .unwrap();

    let stylesheets = stylesheet_paths(&paths);
    assert!(stylesheets.contains(&paths.cache_theme_file));
    std::fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn stale_calendar_theme_cache_is_ignored() {
    let root = std::env::temp_dir().join(format!("argvus-calendar-theme-{}", uuid::Uuid::new_v4()));
    let paths = test_theme_paths(&root);
    std::fs::create_dir_all(paths.active_argvus_theme_file.parent().unwrap()).unwrap();
    std::fs::write(&paths.active_argvus_theme_file, "sunset\n").unwrap();
    std::fs::create_dir_all(paths.cache_theme_file.parent().unwrap()).unwrap();
    std::fs::write(
      &paths.cache_theme_file,
      "/* argvus-theme: gruvbox-dark */\n",
    )
    .unwrap();

    let stylesheets = stylesheet_paths(&paths);
    assert!(!stylesheets.contains(&paths.cache_theme_file));
    std::fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn current_cache_is_loaded_without_a_packaged_theme_stylesheet() {
    let root = std::env::temp_dir().join(format!("argvus-calendar-theme-{}", uuid::Uuid::new_v4()));
    let paths = test_theme_paths(&root);
    std::fs::create_dir_all(paths.active_argvus_theme_file.parent().unwrap()).unwrap();
    std::fs::write(&paths.active_argvus_theme_file, "sunset\n").unwrap();
    std::fs::create_dir_all(paths.cache_theme_file.parent().unwrap()).unwrap();
    std::fs::write(&paths.cache_theme_file, "/* argvus-theme: sunset */\n").unwrap();

    let stylesheets = stylesheet_paths(&paths);
    assert!(stylesheets.contains(&paths.cache_theme_file));
    std::fs::remove_dir_all(root).unwrap();
  }

  fn test_theme_paths(root: &std::path::Path) -> Paths {
    Paths {
      database: root.join("data/calendar.db"),
      config_file: root.join("config/config.toml"),
      legacy_settings_file: root.join("config/settings.toml"),
      user_config_file: root.join("user/config.toml"),
      user_style: root.join("config/style.css"),
      theme_file: root.join("config/theme.css"),
      cache_theme_file: root.join("cache/theme.css"),
      active_argvus_theme_file: root.join("config/argvus/data/.active-theme"),
      events_enabled_file: root.join("state/events-enabled"),
      data_dir: root.join("data"),
      state_dir: root.join("state"),
      cache_dir: root.join("cache"),
    }
  }
}
