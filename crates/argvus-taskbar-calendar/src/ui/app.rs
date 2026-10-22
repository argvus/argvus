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
use gtk4_layer_shell::LayerShell;
use relm4::{ComponentParts, ComponentSender, RelmApp, SimpleComponent};

use crate::calendar::{
  CalendarEvent, first_of_month, next_month, previous_month, shift_year_clamped, today,
};
use crate::config::{
  Paths, Settings, active_theme_name, list_theme_names, load_events_enabled, save_events_enabled,
  write_active_theme,
};
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

const POPUP_WIDTH: i32 = 398;
const POPUP_HEIGHT_ESTIMATE: i32 = 520;
const POPUP_GAP_Y: i32 = 4;
// Layer namespaces that identify the horizontal bar. ARGVUS taskbar runs Waybar
// with the `argvus-taskbar` namespace; plain Waybar uses `waybar`.
const WAYBAR_LAYER_NAMESPACES: [&str; 2] = ["argvus-taskbar", "waybar"];

type PopupBounds = (i32, i32, i32, i32);

#[derive(Default)]
struct PopupState {
  bounds: Cell<Option<PopupBounds>>,
  dismiss_armed: Cell<bool>,
  presentation_id: Cell<u64>,
  // Tracks whether the window has actually gained keyboard focus since the
  // current presentation. Guards against a spurious `has-focus-notify`
  // firing before the compositor ever granted focus (e.g. right after the
  // keyboard mode switches from Exclusive to OnDemand), which would
  // otherwise be mistaken for the user clicking outside the popup.
  focused_once: Cell<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeFingerprint {
  cache_mtime: i64,
  cache_len: u64,
  active_mtime: i64,
  active_len: u64,
  accent_mtime: i64,
  accent_len: u64,
}

fn file_fingerprint(path: &Path) -> (i64, u64) {
  let Ok(metadata) = std::fs::metadata(path) else {
    return (0, 0);
  };
  let mtime = metadata
    .modified()
    .ok()
    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
    .map_or(0, |duration| duration.as_nanos() as i64);
  (mtime, metadata.len())
}

fn theme_fingerprint(paths: &Paths) -> ThemeFingerprint {
  let (cache_mtime, cache_len) = file_fingerprint(&paths.cache_theme_file);
  let (active_mtime, active_len) = file_fingerprint(&paths.active_argvus_theme_file);
  let accent_file = paths
    .active_argvus_theme_file
    .parent()
    .map(|parent| parent.join(".accent-color"))
    .unwrap_or_default();
  let (accent_mtime, accent_len) = file_fingerprint(&accent_file);
  ThemeFingerprint {
    cache_mtime,
    cache_len,
    active_mtime,
    active_len,
    accent_mtime,
    accent_len,
  }
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
  SettingsTheme(String),
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
  popup_state: Rc<PopupState>,
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
    let popup_state = Rc::new(PopupState::default());
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

    let watch_paths = model.paths.clone();
    let watch_sender = sender.clone();
    gtk::glib::timeout_add_local(Duration::from_millis(700), move || {
      let current = theme_fingerprint(&watch_paths);
      if current != theme_watcher.get() {
        theme_watcher.set(current);
        watch_sender.input(AppMsg::ReloadTheme);
      }
      gtk::glib::ControlFlow::Continue
    });

    if let Some(listener) = crate::ipc::listen(&model.paths) {
      let sender = sender.clone();
      crate::ipc::serve(listener, move |message| {
        let msg = match message.command {
          IpcCommand::Toggle => AppMsg::Toggle(message.position),
          IpcCommand::Show => AppMsg::Show(message.position),
          IpcCommand::Hide => AppMsg::Hide,
          IpcCommand::ReloadTheme => AppMsg::ReloadTheme,
          IpcCommand::ReloadConfig => AppMsg::ReloadConfig,
        };
        sender.input(msg);
      });
    }

    window.add_css_class("argvus-calendar-window");

    let stack = gtk::Stack::new();
    stack.add_css_class("content-stack");
    stack.set_hhomogeneous(true);
    stack.set_vhomogeneous(false);

    let calendar_root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    calendar_root.add_css_class("calendar-view");
    calendar_root.set_vexpand(false);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    header.add_css_class("calendar-header");
    let prev_year = gtk::Button::with_label("‹‹");
    prev_year.add_css_class("nav-button");
    prev_year.set_tooltip_text(Some(model.i18n.text(Text::PreviousYear)));
    let prev = gtk::Button::with_label("‹");
    prev.add_css_class("nav-button");
    prev.set_tooltip_text(Some(model.i18n.text(Text::PreviousMonth)));
    let title = gtk::Label::new(None);
    title.set_xalign(0.0);
    title.set_hexpand(true);
    title.add_css_class("calendar-title");
    let today_button = gtk::Button::with_label("●");
    today_button.add_css_class("nav-button");
    today_button.add_css_class("today-button");
    let next = gtk::Button::with_label("›");
    next.add_css_class("nav-button");
    next.set_tooltip_text(Some(model.i18n.text(Text::NextMonth)));
    let next_year = gtk::Button::with_label("››");
    next_year.add_css_class("nav-button");
    next_year.set_tooltip_text(Some(model.i18n.text(Text::NextYear)));
    let settings_button = gtk::Button::from_icon_name("preferences-system-symbolic");
    settings_button.add_css_class("nav-button");
    settings_button.add_css_class("settings-button");
    settings_button.set_tooltip_text(Some(model.i18n.text(Text::Settings)));
    today_button.set_tooltip_text(Some(model.i18n.text(Text::Today)));
    header.append(&prev_year);
    header.append(&prev);
    header.append(&title);
    header.append(&today_button);
    header.append(&next);
    header.append(&next_year);
    header.append(&settings_button);
    calendar_root.append(&header);

    let grid = gtk::Grid::builder()
      .row_spacing(2)
      .column_spacing(2)
      .column_homogeneous(true)
      .build();
    grid.add_css_class("calendar-grid");
    for (idx, label) in weekday_labels(model.i18n.weekday_headers(), model.settings.week_start())
      .into_iter()
      .enumerate()
    {
      let weekday = gtk::Label::new(Some(label));
      weekday.add_css_class("weekday");
      weekday.set_xalign(0.5);
      grid.attach(&weekday, idx as i32, 0, 1, 1);
    }
    let mut day_buttons = Vec::new();
    for index in 0..42 {
      let button = gtk::Button::new();
      button.add_css_class("day");
      let sender = sender.clone();
      button.connect_clicked(move |button| {
        if let Some(date) = unsafe { button.data::<NaiveDate>("date") } {
          sender.input(AppMsg::SelectDay(unsafe { *date.as_ref() }));
        }
      });
      grid.attach(&button, index % 7, index / 7 + 1, 1, 1);
      day_buttons.push(button);
    }
    calendar_root.append(&grid);

    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
    separator.add_css_class("agenda-separator");
    calendar_root.append(&separator);

    let agenda_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    agenda_header.add_css_class("agenda-header");
    let agenda_title = gtk::Label::new(None);
    agenda_title.add_css_class("agenda-title");
    agenda_title.set_xalign(0.0);
    agenda_title.set_hexpand(true);
    let add = gtk::Button::with_label(model.i18n.text(Text::AddEvent));
    add.add_css_class("add-event");
    add.set_tooltip_text(Some(model.i18n.text(Text::AddEvent)));
    agenda_header.append(&agenda_title);
    agenda_header.append(&add);
    calendar_root.append(&agenda_header);

    let agenda = gtk::ListBox::new();
    agenda.add_css_class("agenda");
    agenda.set_selection_mode(gtk::SelectionMode::None);
    let agenda_scroll = gtk::ScrolledWindow::builder()
      .hscrollbar_policy(gtk::PolicyType::Never)
      .vscrollbar_policy(gtk::PolicyType::Automatic)
      .min_content_height(74)
      .max_content_height(128)
      .child(&agenda)
      .build();
    agenda_scroll.add_css_class("agenda-scroll");
    calendar_root.append(&agenda_scroll);

    let editor_root = build_editor();
    editor_root.add_css_class("event-editor");
    let editor_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    editor_header.add_css_class("editor-header");
    let back = gtk::Button::with_label("‹");
    back.add_css_class("nav-button");
    back.set_tooltip_text(Some(model.i18n.text(Text::Back)));
    let editor_title = gtk::Label::new(None);
    editor_title.add_css_class("editor-title");
    editor_title.set_xalign(0.0);
    editor_title.set_hexpand(true);
    editor_header.append(&back);
    editor_header.append(&editor_title);
    editor_root.append(&editor_header);

    let title_label = editor_label();
    let title_entry = editor_entry();
    editor_root.append(&title_label);
    editor_root.append(&title_entry);

    let date_row = editor_row();
    let date_label = editor_label();
    let date_value = gtk::Label::new(None);
    date_value.add_css_class("editor-value");
    date_value.set_xalign(0.0);
    date_row.append(&date_label);
    date_row.append(&date_value);
    editor_root.append(&date_row);

    let start_time_row = editor_row();
    let start_time_label = editor_label();
    start_time_label.set_width_chars(12);
    let start_hour = editor_number_spin(23);
    let start_separator = gtk::Label::new(Some(":"));
    start_separator.add_css_class("time-separator");
    let start_minute = editor_number_spin(59);
    start_time_row.append(&start_time_label);
    start_time_row.append(&start_hour);
    start_time_row.append(&start_separator);
    start_time_row.append(&start_minute);
    editor_root.append(&start_time_row);

    let end_time_row = editor_row();
    let end_time_label = editor_label();
    end_time_label.set_width_chars(12);
    let end_hour = editor_number_spin(23);
    let end_separator = gtk::Label::new(Some(":"));
    end_separator.add_css_class("time-separator");
    let end_minute = editor_number_spin(59);
    end_time_row.append(&end_time_label);
    end_time_row.append(&end_hour);
    end_time_row.append(&end_separator);
    end_time_row.append(&end_minute);
    editor_root.append(&end_time_row);

    let all_day = gtk::CheckButton::new();
    all_day.add_css_class("editor-check");
    editor_root.append(&all_day);

    let location_label = editor_label();
    let location_entry = editor_entry();
    editor_root.append(&location_label);
    editor_root.append(&location_entry);

    let description_label = editor_label();
    let description_entry = editor_entry();
    editor_root.append(&description_label);
    editor_root.append(&description_entry);

    let reminder_row = editor_row();
    let reminder_enabled = gtk::CheckButton::new();
    reminder_enabled.add_css_class("editor-check");
    reminder_enabled.set_width_request(102);
    let reminder_hour_label = gtk::Label::new(None);
    reminder_hour_label.add_css_class("editor-value");
    let reminder_hours = editor_number_spin(23);
    let reminder_minute_label = gtk::Label::new(None);
    reminder_minute_label.add_css_class("editor-value");
    let reminder_minutes = editor_number_spin(59);
    let reminder_before = gtk::Label::new(None);
    reminder_before.add_css_class("editor-value");
    reminder_row.append(&reminder_enabled);
    reminder_row.append(&reminder_hour_label);
    reminder_row.append(&reminder_hours);
    reminder_row.append(&reminder_minute_label);
    reminder_row.append(&reminder_minutes);
    reminder_row.append(&reminder_before);
    editor_root.append(&reminder_row);

    let repeat_row = editor_row();
    let repeat_all_day = gtk::CheckButton::new();
    repeat_all_day.add_css_class("editor-check");
    repeat_all_day.set_width_request(102);
    let repeat_hour_label = gtk::Label::new(None);
    repeat_hour_label.add_css_class("editor-value");
    let repeat_hours = editor_number_spin(23);
    let repeat_minute_label = gtk::Label::new(None);
    repeat_minute_label.add_css_class("editor-value");
    let repeat_minutes = editor_number_spin(59);
    repeat_row.append(&repeat_all_day);
    repeat_row.append(&repeat_hour_label);
    repeat_row.append(&repeat_hours);
    repeat_row.append(&repeat_minute_label);
    repeat_row.append(&repeat_minutes);
    editor_root.append(&repeat_row);

    let editor_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    editor_actions.add_css_class("editor-actions");
    let delete = gtk::Button::new();
    delete.add_css_class("editor-action");
    delete.add_css_class("editor-delete");
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    let cancel = gtk::Button::new();
    cancel.add_css_class("editor-action");
    let save = gtk::Button::new();
    save.add_css_class("editor-action");
    save.add_css_class("editor-save");
    editor_actions.append(&delete);
    editor_actions.append(&spacer);
    editor_actions.append(&cancel);
    editor_actions.append(&save);
    editor_root.append(&editor_actions);

    let settings_root = build_settings();
    settings_root.add_css_class("event-editor");
    settings_root.add_css_class("settings-view");
    let settings_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    settings_header.add_css_class("editor-header");
    let settings_back = gtk::Button::with_label("‹");
    settings_back.add_css_class("nav-button");
    settings_back.set_tooltip_text(Some(model.i18n.text(Text::Back)));
    let settings_title = gtk::Label::new(None);
    settings_title.add_css_class("editor-title");
    settings_title.set_xalign(0.0);
    settings_title.set_hexpand(true);
    settings_header.append(&settings_back);
    settings_header.append(&settings_title);
    settings_root.append(&settings_header);

    let font_family_label = settings_label();
    let font_family = editor_entry();
    settings_root.append(&font_family_label);
    settings_root.append(&font_family);

    let font_size_label = settings_label();
    let font_size = settings_number_spin(32, 2);
    settings_root.append(&font_size_label);
    settings_root.append(&font_size);

    let theme_label = settings_label();
    let theme_names: Vec<String> = list_theme_names(&model.paths);
    let theme_refs: Vec<&str> = theme_names.iter().map(String::as_str).collect();
    let theme_model = gtk::StringList::new(&theme_refs);
    let theme = gtk::DropDown::new(Some(theme_model.clone()), None::<gtk::Expression>);
    theme.add_css_class("editor-dropdown");
    settings_root.append(&theme_label);
    settings_root.append(&theme);

    let language_label = settings_label();
    let language_model = gtk::StringList::new(&["auto", "en-US", "pt-BR"]);
    let language = gtk::DropDown::new(Some(language_model.clone()), None::<gtk::Expression>);
    language.add_css_class("editor-dropdown");
    settings_root.append(&language_label);
    settings_root.append(&language);

    let week_start_label = settings_label();
    let week_start_model = gtk::StringList::new(&["monday", "sunday"]);
    let week_start = gtk::DropDown::new(Some(week_start_model.clone()), None::<gtk::Expression>);
    week_start.add_css_class("editor-dropdown");
    settings_root.append(&week_start_label);
    settings_root.append(&week_start);

    let show_events_label = settings_label();
    let show_events_experimental = gtk::Label::new(Some(model.i18n.text(Text::Experimental)));
    show_events_experimental.add_css_class("experimental-badge");
    show_events_experimental
      .set_tooltip_text(Some(model.i18n.text(Text::EventsExperimentalTooltip)));
    let show_events = gtk::Switch::new();
    show_events.add_css_class("editor-switch");
    show_events.set_tooltip_text(Some(model.i18n.text(Text::EventsExperimentalTooltip)));
    let show_events_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    show_events_spacer.set_hexpand(true);
    let show_events_row = editor_row();
    show_events_row.append(&show_events_label);
    show_events_row.append(&show_events_spacer);
    show_events_row.append(&show_events_experimental);
    show_events_row.append(&show_events);
    settings_root.append(&show_events_row);

    let default_duration_label = settings_label();
    let default_duration = settings_number_spin(1440, 4);
    let default_duration_row = editor_row();
    default_duration_row.append(&default_duration_label);
    default_duration_row.append(&default_duration);
    settings_root.append(&default_duration_row);

    let default_reminder_label = settings_label();
    let default_reminder = settings_number_spin(1440, 4);
    let default_reminder_row = editor_row();
    default_reminder_row.append(&default_reminder_label);
    default_reminder_row.append(&default_reminder);
    settings_root.append(&default_reminder_row);

    let sync_interval_label = settings_label();
    let sync_interval = settings_number_spin(1440, 4);
    let sync_interval_suffix = gtk::Label::new(Some(model.i18n.text(Text::MinuteShort)));
    sync_interval_suffix.add_css_class("editor-value");
    let sync_interval_row = editor_row();
    sync_interval_row.append(&sync_interval_label);
    sync_interval_row.append(&sync_interval);
    sync_interval_row.append(&sync_interval_suffix);
    settings_root.append(&sync_interval_row);

    settings_root.append(&settings_section_label(model.i18n.text(Text::Terminal)));
    let terminal_command_label = settings_label();
    let terminal_command = editor_entry();
    settings_root.append(&terminal_command_label);
    settings_root.append(&terminal_command);
    let terminal_args_label = settings_label();
    let terminal_args = editor_entry();
    settings_root.append(&terminal_args_label);
    settings_root.append(&terminal_args);

    settings_root.append(&settings_section_label(model.i18n.text(Text::Editor)));
    let editor_command_label = settings_label();
    let editor_command = editor_entry();
    settings_root.append(&editor_command_label);
    settings_root.append(&editor_command);
    let editor_args_label = settings_label();
    let editor_args = editor_entry();
    settings_root.append(&editor_args_label);
    settings_root.append(&editor_args);

    let edit_file = gtk::Button::new();
    edit_file.add_css_class("editor-action");
    settings_root.append(&edit_file);

    let settings_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    settings_actions.add_css_class("editor-actions");
    let settings_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    settings_spacer.set_hexpand(true);
    let settings_cancel = gtk::Button::new();
    settings_cancel.add_css_class("editor-action");
    let settings_save = gtk::Button::new();
    settings_save.add_css_class("editor-action");
    settings_save.add_css_class("editor-save");
    settings_actions.append(&settings_spacer);
    settings_actions.append(&settings_cancel);
    settings_actions.append(&settings_save);
    settings_root.append(&settings_actions);

    let settings_scroll = gtk::ScrolledWindow::new();
    settings_scroll.add_css_class("settings-scroll");
    settings_scroll.set_overlay_scrolling(false);
    settings_scroll.set_child(Some(&settings_root));
    settings_scroll.set_min_content_height(620);
    settings_scroll.set_max_content_height(640);
    settings_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    stack.add_named(&calendar_root, Some("calendar"));
    stack.add_named(&editor_root, Some("editor"));
    stack.add_named(&settings_scroll, Some("settings"));
    popup.append(&stack);
    window.set_child(Some(&popup));

    window.connect_has_focus_notify(clone!(
      #[strong]
      sender,
      #[strong]
      popup_state,
      move |window| {
        if window.has_focus() {
          popup_state.focused_once.set(true);
        } else if window.is_visible()
          && popup_state.dismiss_armed.get()
          && popup_state.focused_once.get()
          && pointer_is_outside_popup(popup_state.bounds.get())
        {
          sender.input(AppMsg::ClickOutside);
        }
      }
    ));
    prev_year.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| sender.input(AppMsg::PreviousYear)
    ));
    prev.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| sender.input(AppMsg::PreviousMonth)
    ));
    next.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| sender.input(AppMsg::NextMonth)
    ));
    next_year.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| sender.input(AppMsg::NextYear)
    ));
    today_button.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| sender.input(AppMsg::Today)
    ));
    add.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::OpenNewEvent);
      }
    ));
    back.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::CancelEditor);
      }
    ));
    cancel.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::CancelEditor);
      }
    ));
    save.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::SaveEditor);
      }
    ));
    delete.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::DeleteEvent);
      }
    ));
    settings_button.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::OpenSettings);
      }
    ));
    settings_back.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::CancelSettings);
      }
    ));
    settings_cancel.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::CancelSettings);
      }
    ));
    settings_save.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::SaveSettings);
      }
    ));
    edit_file.connect_clicked(clone!(
      #[strong]
      sender,
      move |_| {
        sender.input(AppMsg::OpenConfig);
      }
    ));

    let font_family_handler = font_family.connect_changed(clone!(
      #[strong]
      sender,
      move |entry: &gtk::Entry| {
        sender.input(AppMsg::SettingsFontFamily(entry.text().to_string()));
      }
    ));
    let font_size_handler = font_size.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::SettingsFontSize(spin.value() as u32));
      }
    ));
    let theme_handler = theme.connect_selected_notify(clone!(
      #[strong]
      sender,
      move |dropdown: &gtk::DropDown| {
        sender.input(AppMsg::SettingsTheme(dropdown_value(dropdown)));
      }
    ));
    let language_handler = language.connect_selected_notify(clone!(
      #[strong]
      sender,
      move |dropdown: &gtk::DropDown| {
        sender.input(AppMsg::SettingsLanguage(dropdown_value(dropdown)));
      }
    ));
    let week_start_handler = week_start.connect_selected_notify(clone!(
      #[strong]
      sender,
      move |dropdown: &gtk::DropDown| {
        sender.input(AppMsg::SettingsWeekStart(dropdown_value(dropdown)));
      }
    ));
    let show_events_handler = show_events.connect_active_notify(clone!(
      #[strong]
      sender,
      move |switch: &gtk::Switch| {
        sender.input(AppMsg::SettingsShowEvents(switch.is_active()));
      }
    ));
    let default_duration_handler = default_duration.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::SettingsDefaultDuration(spin.value() as i64));
      }
    ));
    let default_reminder_handler = default_reminder.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::SettingsDefaultReminder(spin.value() as i64));
      }
    ));
    let sync_interval_handler = sync_interval.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::SettingsSyncInterval(spin.value() as u64));
      }
    ));
    let editor_command_handler = editor_command.connect_changed(clone!(
      #[strong]
      sender,
      move |entry: &gtk::Entry| {
        sender.input(AppMsg::SettingsEditorCommand(entry.text().to_string()));
      }
    ));
    let editor_args_handler = editor_args.connect_changed(clone!(
      #[strong]
      sender,
      move |entry: &gtk::Entry| {
        sender.input(AppMsg::SettingsEditorArgs(entry.text().to_string()));
      }
    ));
    let terminal_command_handler = terminal_command.connect_changed(clone!(
      #[strong]
      sender,
      move |entry: &gtk::Entry| {
        sender.input(AppMsg::SettingsTerminalCommand(entry.text().to_string()));
      }
    ));
    let terminal_args_handler = terminal_args.connect_changed(clone!(
      #[strong]
      sender,
      move |entry: &gtk::Entry| {
        sender.input(AppMsg::SettingsTerminalArgs(entry.text().to_string()));
      }
    ));

    let key = gtk::EventControllerKey::new();
    key.connect_key_pressed(clone!(
      #[strong]
      sender,
      #[strong]
      window,
      move |_, key, _, modifier| {
        if key != gtk::gdk::Key::Escape && editor_control_has_focus(&window) {
          return gtk::glib::Propagation::Proceed;
        }
        match key {
          gtk::gdk::Key::Escape => {
            sender.input(AppMsg::Hide);
          }
          gtk::gdk::Key::t | gtk::gdk::Key::T => sender.input(AppMsg::Today),
          gtk::gdk::Key::n | gtk::gdk::Key::N => sender.input(AppMsg::OpenNewEvent),
          gtk::gdk::Key::Left => sender.input(AppMsg::MoveDays(-1)),
          gtk::gdk::Key::Right => sender.input(AppMsg::MoveDays(1)),
          gtk::gdk::Key::Page_Up if modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK) => {
            sender.input(AppMsg::PreviousYear)
          }
          gtk::gdk::Key::Page_Down if modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK) => {
            sender.input(AppMsg::NextYear)
          }
          gtk::gdk::Key::Page_Up => sender.input(AppMsg::PreviousMonth),
          gtk::gdk::Key::Page_Down => sender.input(AppMsg::NextMonth),
          _ => return gtk::glib::Propagation::Proceed,
        }
        gtk::glib::Propagation::Stop
      }
    ));
    window.add_controller(key);

    let title_handler = title_entry.connect_changed(clone!(
      #[strong]
      sender,
      move |entry: &gtk::Entry| {
        sender.input(AppMsg::EditorTitle(entry.text().to_string()));
      }
    ));
    let location_handler = location_entry.connect_changed(clone!(
      #[strong]
      sender,
      move |entry: &gtk::Entry| {
        sender.input(AppMsg::EditorLocation(entry.text().to_string()));
      }
    ));
    let description_handler = description_entry.connect_changed(clone!(
      #[strong]
      sender,
      move |entry: &gtk::Entry| {
        sender.input(AppMsg::EditorDescription(entry.text().to_string()));
      }
    ));
    let start_hour_handler = start_hour.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::EditorStartHour(spin.value() as u32));
      }
    ));
    let start_minute_handler = start_minute.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::EditorStartMinute(spin.value() as u32));
      }
    ));
    let end_hour_handler = end_hour.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::EditorEndHour(spin.value() as u32));
      }
    ));
    let end_minute_handler = end_minute.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::EditorEndMinute(spin.value() as u32));
      }
    ));
    let all_day_handler = all_day.connect_toggled(clone!(
      #[strong]
      sender,
      move |check: &gtk::CheckButton| {
        sender.input(AppMsg::EditorAllDay(check.is_active()));
      }
    ));
    let reminder_enabled_handler = reminder_enabled.connect_toggled(clone!(
      #[strong]
      sender,
      move |check: &gtk::CheckButton| {
        sender.input(AppMsg::EditorReminderEnabled(check.is_active()));
      }
    ));
    let reminder_hours_handler = reminder_hours.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::EditorReminderHours(spin.value() as u32));
      }
    ));
    let reminder_minutes_handler = reminder_minutes.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::EditorReminderMinutes(spin.value() as u32));
      }
    ));
    let repeat_all_day_handler = repeat_all_day.connect_toggled(clone!(
      #[strong]
      sender,
      move |check: &gtk::CheckButton| {
        sender.input(AppMsg::EditorRepeatAllDay(check.is_active()));
      }
    ));
    let repeat_hours_handler = repeat_hours.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::EditorRepeatHours(spin.value() as u32));
      }
    ));
    let repeat_minutes_handler = repeat_minutes.connect_value_changed(clone!(
      #[strong]
      sender,
      move |spin: &gtk::SpinButton| {
        sender.input(AppMsg::EditorRepeatMinutes(spin.value() as u32));
      }
    ));

    let mut widgets = AppWidgets {
      popup,
      stack,
      title,
      day_buttons,
      agenda_separator: separator,
      agenda_header,
      agenda_title,
      add_button: add,
      prev_year,
      prev_month: prev,
      today_button,
      next_month: next,
      next_year,
      settings_button,
      agenda,
      agenda_scroll,
      editor: EditorWidgets {
        header: editor_title,
        title_label,
        title: title_entry,
        date_label,
        date_value,
        start_time_label,
        end_time_label,
        start_hour,
        start_minute,
        end_hour,
        end_minute,
        all_day,
        location_label,
        location: location_entry,
        description_label,
        description: description_entry,
        reminder_enabled,
        reminder_hour_label,
        reminder_hours,
        reminder_minute_label,
        reminder_minutes,
        reminder_before,
        repeat_row,
        repeat_all_day,
        repeat_hour_label,
        repeat_hours,
        repeat_minute_label,
        repeat_minutes,
        delete,
        cancel,
        save,
        changed_handlers: EditorChangedHandlers {
          title: title_handler,
          location: location_handler,
          description: description_handler,
          start_hour: start_hour_handler,
          start_minute: start_minute_handler,
          end_hour: end_hour_handler,
          end_minute: end_minute_handler,
          all_day: all_day_handler,
          reminder_enabled: reminder_enabled_handler,
          reminder_hours: reminder_hours_handler,
          reminder_minutes: reminder_minutes_handler,
          repeat_all_day: repeat_all_day_handler,
          repeat_hours: repeat_hours_handler,
          repeat_minutes: repeat_minutes_handler,
        },
      },
      settings: SettingsWidgets {
        header: settings_title,
        font_family_label,
        font_family,
        font_size_label,
        font_size,
        theme_label,
        theme,
        theme_model,
        language_label,
        language,
        language_model,
        week_start_label,
        week_start,
        week_start_model,
        show_events_label,
        show_events_experimental,
        show_events,
        default_duration_label,
        default_duration,
        default_reminder_label,
        default_reminder,
        sync_interval_label,
        sync_interval,
        editor_command_label,
        editor_command,
        editor_args_label,
        editor_args,
        terminal_command_label,
        terminal_command,
        terminal_args_label,
        terminal_args,
        edit_file,
        cancel: settings_cancel,
        save: settings_save,
        changed_handlers: SettingsChangedHandlers {
          font_family: font_family_handler,
          font_size: font_size_handler,
          theme: theme_handler,
          language: language_handler,
          week_start: week_start_handler,
          show_events: show_events_handler,
          default_duration: default_duration_handler,
          default_reminder: default_reminder_handler,
          sync_interval: sync_interval_handler,
          editor_command: editor_command_handler,
          editor_args: editor_args_handler,
          terminal_command: terminal_command_handler,
          terminal_args: terminal_args_handler,
        },
      },
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
    let should_reload = !matches!(
      &msg,
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
        | AppMsg::SettingsTheme(_)
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
    );
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
            self.settings.calendar.default_reminder_minutes,
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
      AppMsg::EditorTitle(value) => update_draft(&mut self.view, |draft| draft.title = value),
      AppMsg::EditorLocation(value) => {
        update_draft(&mut self.view, |draft| draft.location = value);
      }
      AppMsg::EditorDescription(value) => {
        update_draft(&mut self.view, |draft| draft.description = value);
      }
      AppMsg::EditorStartHour(value) => {
        update_draft(&mut self.view, |draft| draft.start_hour = value);
      }
      AppMsg::EditorStartMinute(value) => {
        update_draft(&mut self.view, |draft| draft.start_minute = value);
      }
      AppMsg::EditorEndHour(value) => {
        update_draft(&mut self.view, |draft| draft.end_hour = value);
      }
      AppMsg::EditorEndMinute(value) => {
        update_draft(&mut self.view, |draft| draft.end_minute = value);
      }
      AppMsg::EditorAllDay(value) => update_draft(&mut self.view, |draft| draft.all_day = value),
      AppMsg::EditorReminderEnabled(value) => {
        update_draft(&mut self.view, |draft| draft.reminder_enabled = value)
      }
      AppMsg::EditorReminderHours(value) => {
        update_draft(&mut self.view, |draft| draft.reminder_hours = value)
      }
      AppMsg::EditorReminderMinutes(value) => {
        update_draft(&mut self.view, |draft| draft.reminder_minutes = value)
      }
      AppMsg::EditorRepeatAllDay(value) => {
        update_draft(&mut self.view, |draft| draft.repeat_all_day = value)
      }
      AppMsg::EditorRepeatHours(value) => {
        update_draft(&mut self.view, |draft| draft.repeat_hours = value)
      }
      AppMsg::EditorRepeatMinutes(value) => {
        update_draft(&mut self.view, |draft| draft.repeat_minutes = value)
      }
      AppMsg::SaveEditor => {
        if let ViewState::Editor(draft) = &self.view {
          let draft = *draft.clone();
          let _ = save_editor(&self.paths.database, &self.settings, draft);
          self.view = ViewState::Calendar;
        }
      }
      AppMsg::DeleteEvent => {
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
      AppMsg::CancelEditor => {
        self.view = ViewState::Calendar;
      }
      AppMsg::OpenSettings => {
        let theme = active_theme_name(&self.paths)
          .or_else(|| list_theme_names(&self.paths).into_iter().next())
          .unwrap_or_default();
        self.view = ViewState::Settings(Box::new(SettingsDraft::from_config(
          &self.settings,
          &theme,
          self.events_enabled,
        )));
      }
      AppMsg::CancelSettings => {
        self.view = ViewState::Calendar;
      }
      AppMsg::SettingsFontFamily(value) => {
        update_settings_draft(&mut self.view, |draft| draft.font_family = value)
      }
      AppMsg::SettingsFontSize(value) => {
        update_settings_draft(&mut self.view, |draft| draft.font_size = value)
      }
      AppMsg::SettingsTheme(value) => {
        update_settings_draft(&mut self.view, |draft| draft.theme = value)
      }
      AppMsg::SettingsLanguage(value) => {
        update_settings_draft(&mut self.view, |draft| draft.language = value)
      }
      AppMsg::SettingsWeekStart(value) => {
        update_settings_draft(&mut self.view, |draft| draft.week_start = value)
      }
      AppMsg::SettingsShowEvents(value) => {
        update_settings_draft(&mut self.view, |draft| draft.show_events = value)
      }
      AppMsg::SettingsDefaultDuration(value) => update_settings_draft(&mut self.view, |draft| {
        draft.default_event_duration_minutes = value
      }),
      AppMsg::SettingsDefaultReminder(value) => update_settings_draft(&mut self.view, |draft| {
        draft.default_reminder_minutes = value
      }),
      AppMsg::SettingsSyncInterval(value) => {
        update_settings_draft(&mut self.view, |draft| draft.sync_interval_minutes = value)
      }
      AppMsg::SettingsEditorCommand(value) => {
        update_settings_draft(&mut self.view, |draft| draft.editor_command = value)
      }
      AppMsg::SettingsEditorArgs(value) => {
        update_settings_draft(&mut self.view, |draft| draft.editor_args = value)
      }
      AppMsg::SettingsTerminalCommand(value) => {
        update_settings_draft(&mut self.view, |draft| draft.terminal_command = value)
      }
      AppMsg::SettingsTerminalArgs(value) => {
        update_settings_draft(&mut self.view, |draft| draft.terminal_args = value)
      }
      AppMsg::SaveSettings => {
        if let ViewState::Settings(draft) = &self.view {
          let draft = *draft.clone();
          let theme = draft.theme.clone();
          if active_theme_name(&self.paths).as_deref() != Some(theme.as_str()) {
            let _ = write_active_theme(&self.paths, &theme);
          }
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
      AppMsg::Toggle(fixed_position) => {
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
      AppMsg::Show(fixed_position) => {
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

fn render_model(model: &AppModel, widgets: &mut AppWidgets, sender: ComponentSender<AppModel>) {
  widgets
    .title
    .set_label(&model.i18n.month_title(model.month));
  let show_events = model.events_enabled;
  for (button, (date, in_month)) in widgets
    .day_buttons
    .iter()
    .zip(day_cells(model.month, model.settings.week_start()))
  {
    let has_event = show_events && model.event_days.contains(&date);
    let label = if has_event {
      format!("{:02}•", date.day())
    } else {
      format!("{:02}", date.day())
    };
    button.set_label(&label);
    unsafe {
      button.set_data("date", date);
    }
    for class in [
      "day-outside-month",
      "day-current",
      "day-selected",
      "day-has-event",
    ] {
      button.remove_css_class(class);
    }
    if !in_month {
      button.add_css_class("day-outside-month");
    }
    if date == today() {
      button.add_css_class("day-current");
    }
    if date == model.selected {
      button.add_css_class("day-selected");
    }
    if has_event {
      button.add_css_class("day-has-event");
    }
  }
  unsafe {
    widgets.add_button.set_data("selected-date", model.selected);
  }
  widgets
    .agenda_title
    .set_label(&model.i18n.agenda_date(model.selected));
  widgets
    .add_button
    .set_label(model.i18n.text(Text::AddEvent));
  let can_add_event = show_events && can_create_event(model.selected);
  widgets.add_button.set_sensitive(can_add_event);
  widgets.add_button.set_tooltip_text(
    (!can_add_event && model.selected < today()).then_some(model.i18n.text(Text::PastDate)),
  );
  widgets
    .prev_year
    .set_tooltip_text(Some(model.i18n.text(Text::PreviousYear)));
  widgets
    .prev_month
    .set_tooltip_text(Some(model.i18n.text(Text::PreviousMonth)));
  widgets
    .today_button
    .set_tooltip_text(Some(model.i18n.text(Text::Today)));
  widgets
    .next_month
    .set_tooltip_text(Some(model.i18n.text(Text::NextMonth)));
  widgets
    .next_year
    .set_tooltip_text(Some(model.i18n.text(Text::NextYear)));
  widgets
    .settings_button
    .set_tooltip_text(Some(model.i18n.text(Text::Settings)));
  widgets.agenda_separator.set_visible(show_events);
  widgets.agenda_header.set_visible(show_events);
  widgets.agenda_scroll.set_visible(show_events);
  if show_events {
    widgets.agenda_scroll.set_min_content_height(74);
    widgets.agenda_scroll.set_max_content_height(128);
    rebuild_agenda(&widgets.agenda, &model.events, model.i18n, move |id| {
      sender.input(AppMsg::OpenEditEvent(id));
    });
  } else {
    widgets.agenda_scroll.set_min_content_height(0);
    widgets.agenda_scroll.set_max_content_height(0);
    while let Some(child) = widgets.agenda.first_child() {
      widgets.agenda.remove(&child);
    }
  }
  if let ViewState::Editor(draft) = &model.view {
    widgets.stack.set_visible_child_name("editor");
    populate_editor(&widgets.editor, draft, model.i18n);
  } else if let ViewState::Settings(draft) = &model.view {
    widgets.stack.set_visible_child_name("settings");
    populate_settings(&widgets.settings, draft, model.i18n);
  } else {
    widgets.stack.set_visible_child_name("calendar");
  }
}

fn editor_row() -> gtk::Box {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  row.add_css_class("editor-row");
  row
}

fn editor_label() -> gtk::Label {
  let label = gtk::Label::new(None);
  label.add_css_class("editor-label");
  label.set_xalign(0.0);
  label.set_width_chars(9);
  label
}

fn editor_entry() -> gtk::Entry {
  let entry = gtk::Entry::new();
  entry.add_css_class("editor-entry");
  entry
}

fn editor_control_has_focus(window: &gtk::Window) -> bool {
  gtk::prelude::RootExt::focus(window).is_some_and(|widget| {
    widget.is::<gtk::Entry>() || widget.ancestor(gtk::Entry::static_type()).is_some()
  })
}

fn can_create_event(date: NaiveDate) -> bool {
  date >= today()
}

fn editor_number_spin(max: u32) -> gtk::SpinButton {
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

fn settings_label() -> gtk::Label {
  let label = gtk::Label::new(None);
  label.add_css_class("editor-label");
  label.set_xalign(0.0);
  label.set_width_chars(14);
  label
}

fn settings_section_label(text: &str) -> gtk::Label {
  let label = gtk::Label::new(Some(text));
  label.add_css_class("editor-section");
  label.set_xalign(0.0);
  label
}

fn settings_number_spin(max: u32, width: i32) -> gtk::SpinButton {
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

fn cancel_editor(view: &mut ViewState) {
  *view = ViewState::Calendar;
}

fn hide_popup(view: &mut ViewState, window: &gtk::Window, popup_state: &PopupState) {
  popup_state
    .presentation_id
    .set(popup_state.presentation_id.get().wrapping_add(1));
  popup_state.dismiss_armed.set(false);
  popup_state.bounds.set(None);
  cancel_editor(view);
  window.set_visible(false);
  relm4::main_application().quit();
}

fn update_draft<F>(view: &mut ViewState, update: F)
where
  F: FnOnce(&mut EditorDraft),
{
  if let ViewState::Editor(draft) = view {
    update(draft.as_mut());
  }
}

fn update_settings_draft<F>(view: &mut ViewState, update: F)
where
  F: FnOnce(&mut SettingsDraft),
{
  if let ViewState::Settings(draft) = view {
    update(draft.as_mut());
  }
}

fn control_reminder_service(enabled: bool) {
  if enabled {
    let _ = Command::new("systemctl")
      .args(["--user", "reset-failed", "argvus-taskbar-calendar.service"])
      .status();
  }
  let systemctl_args: &[&str] = if enabled {
    &[
      "--user",
      "enable",
      "--now",
      "argvus-taskbar-calendar.service",
    ]
  } else {
    &[
      "--user",
      "disable",
      "--now",
      "argvus-taskbar-calendar.service",
    ]
  };
  let _ = Command::new("systemctl").args(systemctl_args).status();
  if !enabled {
    // Only this user's reminder service: the uid comes from /proc/self, as
    // this crate does not link libc. Without it nothing is signalled.
    if let Ok(metadata) = std::fs::metadata("/proc/self") {
      use std::os::unix::fs::MetadataExt as _;
      let _ = Command::new("pkill")
        .args([
          "-u",
          metadata.uid().to_string().as_str(),
          "-f",
          "argvus taskbar-calendar service",
        ])
        .status();
    }
  }
}

fn present_popup(
  window: &gtk::Window,
  popup: &gtk::Box,
  popup_state: &Rc<PopupState>,
  fixed_position: Option<PopupAnchor>,
) {
  let presentation_id = popup_state.presentation_id.get().wrapping_add(1);
  popup_state.presentation_id.set(presentation_id);
  popup_state.dismiss_armed.set(false);
  popup_state.focused_once.set(false);
  set_monitor_at_pointer(window, fixed_position);
  popup_state
    .bounds
    .set(Some(position_popup(window, fixed_position)));
  window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
  popup.queue_resize();
  window.queue_resize();
  window.present();
  let measured_state = popup_state.clone();
  window.add_tick_callback(move |window, _| {
    if !window.is_visible() || measured_state.presentation_id.get() != presentation_id {
      return gtk::glib::ControlFlow::Break;
    }
    let width = window.width();
    let height = window.height();
    if width > 0
      && height > 0
      && let Some((x, y, old_width, old_height)) = measured_state.bounds.get()
    {
      let y = if window.is_anchor(gtk4_layer_shell::Edge::Bottom) {
        y + old_height - height
      } else {
        y
      };
      measured_state
        .bounds
        .set(Some((x + old_width - width, y, width, height)));
    }
    gtk::glib::ControlFlow::Continue
  });
  window.grab_focus();
  window.queue_draw();
  let click_outside = window.clone();
  let popup_state = popup_state.clone();
  gtk::glib::timeout_add_local(Duration::from_millis(800), move || {
    if click_outside.is_visible() && popup_state.presentation_id.get() == presentation_id {
      click_outside.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
      popup_state.dismiss_armed.set(true);
    }
    gtk::glib::ControlFlow::Break
  });
}

fn set_monitor_at_pointer(window: &gtk::Window, fixed_position: Option<PopupAnchor>) {
  let Some(anchor) = fixed_position.or_else(|| {
    current_pointer_position().map(|(x, y)| PopupAnchor {
      x,
      y,
      bar_bottom: None,
      bar_right: None,
    })
  }) else {
    return;
  };
  let (pointer_x, pointer_y) = (anchor.x, anchor.y);
  let Some(display) = gtk::gdk::Display::default() else {
    return;
  };
  let monitors = display.monitors();
  for index in 0..monitors.n_items() {
    let Some(item) = monitors.item(index) else {
      continue;
    };
    let Ok(monitor) = item.downcast::<gtk::gdk::Monitor>() else {
      continue;
    };
    let geometry = monitor.geometry();
    let contains = pointer_x >= geometry.x()
      && pointer_x < geometry.x() + geometry.width()
      && pointer_y >= geometry.y()
      && pointer_y < geometry.y() + geometry.height();
    if contains {
      window.set_monitor(Some(&monitor));
      return;
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

fn apply_layer_shell(window: &gtk::Window) {
  window.init_layer_shell();
  window.set_namespace(Some("argvus-taskbar-calendar"));
  window.set_layer(gtk4_layer_shell::Layer::Overlay);
  window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
  // Margins below are relative to the full monitor. With zone 0 the
  // compositor adds the panels' reserved area again, pushing us down.
  window.set_exclusive_zone(-1);
  for edge in [
    gtk4_layer_shell::Edge::Top,
    gtk4_layer_shell::Edge::Right,
    gtk4_layer_shell::Edge::Bottom,
    gtk4_layer_shell::Edge::Left,
  ] {
    window.set_anchor(edge, false);
    window.set_margin(edge, 0);
  }
  window.set_anchor(gtk4_layer_shell::Edge::Top, true);
  window.set_anchor(gtk4_layer_shell::Edge::Right, true);
}

fn position_popup(window: &gtk::Window, fixed_position: Option<PopupAnchor>) -> PopupBounds {
  let position = popup_position(fixed_position);
  window.set_anchor(gtk4_layer_shell::Edge::Top, !position.above);
  window.set_anchor(gtk4_layer_shell::Edge::Bottom, position.above);
  window.set_margin(gtk4_layer_shell::Edge::Bottom, position.margin_bottom);
  window.set_margin(gtk4_layer_shell::Edge::Left, 0);
  window.set_margin(gtk4_layer_shell::Edge::Top, position.margin_top);
  window.set_margin(gtk4_layer_shell::Edge::Right, position.margin_right);
  position.bounds
}

struct PopupPosition {
  above: bool,
  margin_bottom: i32,
  margin_top: i32,
  margin_right: i32,
  bounds: PopupBounds,
}

fn popup_position(fixed_position: Option<PopupAnchor>) -> PopupPosition {
  let anchor = fixed_position.unwrap_or_else(|| {
    current_pointer_position()
      .map(|(x, y)| PopupAnchor {
        x,
        y,
        bar_bottom: None,
        bar_right: None,
      })
      .unwrap_or(PopupAnchor {
        x: 0,
        y: 0,
        bar_bottom: None,
        bar_right: None,
      })
  });
  let (pointer_x, pointer_y) = (anchor.x, anchor.y);
  let monitor = current_monitor_geometry(pointer_x, pointer_y).unwrap_or((0, 0, 1920, 1080, 0));
  let (origin_x, origin_y, width, height, reserved_top) = monitor;
  let waybar = current_waybar_geometry(pointer_x, pointer_y, monitor);
  let bar_bottom = waybar
    .map(|geometry| geometry.bottom)
    .or(anchor.bar_bottom)
    .unwrap_or(origin_y + reserved_top);
  let bar_right = waybar
    .map(|geometry| geometry.right)
    .or(anchor.bar_right)
    .unwrap_or(origin_x + width);
  let (popup_x, popup_y) = popup_origin(bar_right, bar_bottom, origin_x, origin_y, width, height);
  let above = waybar.is_some_and(|bar| bar.top - origin_y > height / 2);
  let margin_bottom = waybar
    .filter(|_| above)
    .map_or(0, |bar| (origin_y + height - bar.top + POPUP_GAP_Y).max(0));

  PopupPosition {
    above,
    margin_bottom,
    margin_top: if above { 0 } else { popup_y.max(0) },
    margin_right: (width - popup_x - POPUP_WIDTH).max(0),
    bounds: (
      origin_x + popup_x,
      origin_y
        + if above {
          height - margin_bottom - POPUP_HEIGHT_ESTIMATE
        } else {
          popup_y
        },
      POPUP_WIDTH,
      POPUP_HEIGHT_ESTIMATE,
    ),
  }
}

fn popup_origin(
  bar_right: i32,
  bar_bottom: i32,
  origin_x: i32,
  origin_y: i32,
  width: i32,
  height: i32,
) -> (i32, i32) {
  let max_x = (width - POPUP_WIDTH).max(0);
  let max_y = (height - POPUP_HEIGHT_ESTIMATE).max(0);
  let popup_x = (bar_right - origin_x - POPUP_WIDTH).clamp(0, max_x);
  let popup_y = (bar_bottom - origin_y + POPUP_GAP_Y).clamp(0, max_y);
  (popup_x, popup_y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WaybarGeometry {
  top: i32,
  right: i32,
  bottom: i32,
}

fn current_waybar_geometry(
  pointer_x: i32,
  pointer_y: i32,
  monitor: (i32, i32, i32, i32, i32),
) -> Option<WaybarGeometry> {
  let output = Command::new("hyprctl")
    .args(["layers", "-j"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let outputs = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
  select_waybar(&outputs, pointer_x, pointer_y, monitor)
}

fn select_waybar(
  outputs: &serde_json::Value,
  pointer_x: i32,
  pointer_y: i32,
  monitor: (i32, i32, i32, i32, i32),
) -> Option<WaybarGeometry> {
  let (mx, my, mw, mh, _) = monitor;
  outputs
    .as_object()?
    .values()
    .filter_map(|output| output.get("levels")?.as_object())
    .flat_map(|levels| levels.values())
    .filter_map(serde_json::Value::as_array)
    .flatten()
    .filter_map(|layer| {
      let namespace = layer.get("namespace")?.as_str()?;
      let x = layer.get("x")?.as_i64()? as i32;
      let y = layer.get("y")?.as_i64()? as i32;
      let w = layer.get("w")?.as_i64()? as i32;
      let h = layer.get("h")?.as_i64()? as i32;
      let horizontal = WAYBAR_LAYER_NAMESPACES.contains(&namespace) && w > h.saturating_mul(4);
      let same_monitor = x >= mx && x < mx + mw && y >= my && y < my + mh;
      let distance = i64::from(pointer_x.clamp(x, x + w) - pointer_x).abs()
        + i64::from(pointer_y.clamp(y, y + h) - pointer_y).abs();
      (horizontal && same_monitor).then_some((
        distance,
        WaybarGeometry {
          top: y,
          right: x.checked_add(w)?,
          bottom: y.checked_add(h)?,
        },
      ))
    })
    .min_by_key(|(distance, _)| *distance)
    .map(|(_, bar)| bar)
}

fn pointer_is_outside_popup(bounds: Option<PopupBounds>) -> bool {
  let Some((x, y, width, height)) = bounds else {
    return true;
  };
  let Some((pointer_x, pointer_y)) = current_pointer_position() else {
    return true;
  };
  pointer_x < x || pointer_x >= x + width || pointer_y < y || pointer_y >= y + height
}

pub(crate) fn current_pointer_position() -> Option<(i32, i32)> {
  let output = Command::new("hyprctl")
    .args(["cursorpos", "-j"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let value = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
  let x = value.get("x")?.as_i64()? as i32;
  let y = value.get("y")?.as_i64()? as i32;
  Some((x, y))
}

fn current_monitor_geometry(pointer_x: i32, pointer_y: i32) -> Option<(i32, i32, i32, i32, i32)> {
  let output = Command::new("hyprctl")
    .args(["monitors", "-j"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let monitors = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
  monitors.as_array()?.iter().find_map(|monitor| {
    let x = monitor.get("x")?.as_i64()? as i32;
    let y = monitor.get("y")?.as_i64()? as i32;
    let width = monitor.get("width")?.as_i64()? as i32;
    let height = monitor.get("height")?.as_i64()? as i32;
    let reserved_top = monitor
      .get("reserved")
      .and_then(|reserved| reserved.get(1))
      .and_then(serde_json::Value::as_i64)
      .unwrap_or(0) as i32;
    let contains =
      pointer_x >= x && pointer_y >= y && pointer_x < x + width && pointer_y < y + height;
    contains.then_some((x, y, width, height, reserved_top))
  })
}

fn load_css(paths: &Paths, providers: &Rc<RefCell<Vec<gtk::CssProvider>>>) {
  if let Some(display) = gtk::gdk::Display::default() {
    for old in providers.borrow_mut().drain(..) {
      gtk::style_context_remove_provider_for_display(&display, &old);
    }

    let mut new_providers = Vec::new();
    for (priority, stylesheet) in stylesheet_paths(paths).into_iter().enumerate() {
      if stylesheet.exists() {
        let provider = gtk::CssProvider::new();
        provider.load_from_path(stylesheet);
        gtk::style_context_add_provider_for_display(
          &display,
          &provider,
          gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + priority as u32,
        );
        new_providers.push(provider);
      }
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_string(&runtime_css(&paths.config_file, paths));
    gtk::style_context_add_provider_for_display(
      &display,
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 10,
    );
    new_providers.push(provider);

    *providers.borrow_mut() = new_providers;
  }
}

fn stylesheet_paths(paths: &Paths) -> Vec<std::path::PathBuf> {
  let mut stylesheets = vec![paths.user_style.clone(), paths.theme_file.clone()];
  let active_theme_id = active_theme_id(paths);
  let active_theme = active_theme_name(paths);
  if let Some(active_theme) = &active_theme {
    stylesheets.push(paths.theme_dir.join(format!("{active_theme}.css")));
  }
  if cache_theme_is_current(paths, active_theme_id.as_deref()) {
    stylesheets.push(paths.cache_theme_file.clone());
  }
  stylesheets
}

fn active_theme_id(paths: &Paths) -> Option<String> {
  let theme = std::fs::read_to_string(&paths.active_argvus_theme_file).ok()?;
  let theme = theme.trim();
  (!theme.is_empty()).then(|| theme.to_string())
}

fn cache_theme_is_current(paths: &Paths, active_theme: Option<&str>) -> bool {
  let Ok(cache_css) = std::fs::read_to_string(&paths.cache_theme_file) else {
    return false;
  };
  let Some(marker) = cache_css.lines().next().and_then(|line| {
    line
      .trim()
      .strip_prefix("/* argvus-theme: ")
      .and_then(|value| value.strip_suffix(" */"))
  }) else {
    return active_theme.is_none();
  };
  active_theme.is_some_and(|theme| theme == marker)
}

fn runtime_css(_config_path: &std::path::Path, paths: &Paths) -> String {
  let settings = Settings::load(paths).unwrap_or_default();
  let family = css_string(&settings.appearance.font_family);
  let size = settings.appearance.font_size.clamp(8, 32);
  format!(
    ".argvus-calendar, .event-editor {{ font-family: \"{family}\", monospace; font-size: {size}px; }}\n\
         window.argvus-calendar-window, .argvus-calendar-window {{ background-color: transparent; background-image: none; box-shadow: none; }}\n{}",
    accent_runtime_css(paths)
  )
}

fn accent_runtime_css(paths: &Paths) -> String {
  let accent_file = paths
    .active_argvus_theme_file
    .parent()
    .map(|parent| parent.join(".accent-color"));
  let Some(accent_file) = accent_file else {
    return String::new();
  };
  let Ok(value) = std::fs::read_to_string(accent_file) else {
    return String::new();
  };
  let value = value.trim();
  let hex = value.strip_prefix('#').unwrap_or(value);
  if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
    return String::new();
  }
  let Ok(red) = u8::from_str_radix(&hex[0..2], 16) else {
    return String::new();
  };
  let Ok(green) = u8::from_str_radix(&hex[2..4], 16) else {
    return String::new();
  };
  let Ok(blue) = u8::from_str_radix(&hex[4..6], 16) else {
    return String::new();
  };
  let luminance = (299 * u32::from(red) + 587 * u32::from(green) + 114 * u32::from(blue)) / 1000;
  let text = if luminance >= 128 {
    "#000000"
  } else {
    "#FFFFFF"
  };
  format!(
    ".today-button, .add-event, .weekday, .day-current, .editor-label, .editor-save, .events-toggle:checked {{ color: {value}; }}\n\
     .nav-button:hover, .add-event:hover, .day:hover, .event-button:hover, .editor-action:hover, .agenda row:hover, .event-row:hover {{ background-color: rgba({red}, {green}, {blue}, 0.16); border-color: {value}; }}\n\
     .nav-button:focus, .add-event:focus, .day:focus, .event-button:focus, .editor-action:focus, .editor-entry:focus, .event-editor entry:focus, .day-selected {{ border-color: {value}; }}\n\
     .day-current.day-selected {{ background-color: rgba({red}, {green}, {blue}, 0.22); }}\n\
     .event-divider {{ color: rgba({red}, {green}, {blue}, 0.54); }}\n\
     .day-selected, .editor-save {{ color: {text}; }}"
  )
}

fn css_string(value: &str) -> String {
  value
    .chars()
    .filter(|ch| !matches!(ch, '"' | '\\' | '\n' | '\r'))
    .collect()
}

fn load_day_and_month(
  paths: &Paths,
  selected: NaiveDate,
  month: NaiveDate,
) -> (Vec<CalendarEvent>, HashSet<NaiveDate>) {
  let Ok(db) = Database::open(&paths.database) else {
    return (Vec::new(), HashSet::new());
  };
  let _ = db.purge_expired_local_single_events(chrono::Utc::now());
  let events = db.events_for_day(selected).unwrap_or_default();
  let first = first_of_month(month);
  let last = next_month(month)
    .checked_sub_days(Days::new(1))
    .unwrap_or(first);
  let start = first.and_hms_opt(0, 0, 0).unwrap().and_utc();
  let end = last.and_hms_opt(23, 59, 59).unwrap().and_utc();
  let event_days = db
    .events_between(start, end)
    .unwrap_or_default()
    .into_iter()
    .map(|event| event.start.with_timezone(&Local).date_naive())
    .collect();
  (events, event_days)
}

#[cfg(test)]
mod tests {
  use chrono::NaiveDate;

  use super::*;

  #[test]
  fn view_transitions_calendar_new_cancel_calendar() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
    let view = ViewState::Editor(Box::new(EditorDraft::new(date, 10)));
    assert!(matches!(view, ViewState::Editor(_)));
    let view = ViewState::Calendar;
    assert!(matches!(view, ViewState::Calendar));
  }

  #[test]
  fn draft_updates_are_applied() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
    let mut view = ViewState::Editor(Box::new(EditorDraft::new(date, 10)));
    update_draft(&mut view, |draft| draft.title = "Meeting".to_string());
    let ViewState::Editor(draft) = view else {
      panic!("expected editor");
    };
    assert_eq!(draft.title, "Meeting");
  }

  #[test]
  fn cancel_editor_restores_calendar_view() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    let mut view = ViewState::Editor(Box::new(EditorDraft::new(date, 10)));
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
  fn popup_opens_below_the_fixed_click_position() {
    assert_eq!(popup_origin(1902, 46, 0, 0, 1920, 1080), (1504, 50));
    assert_eq!(popup_origin(1920, 1060, 0, 0, 1920, 1080), (1522, 560));
  }

  #[test]
  fn popup_right_edge_follows_waybar_not_click_position() {
    let layers = serde_json::json!({"HDMI-A-1": {"levels": {"1": [
      {"namespace":"waybar", "x":18, "y":18, "w":1884, "h":28},
      {"namespace":"waybar", "x":18, "y":64, "w":384, "h":998}
    ]}}});
    for y in [0, 7, 17, 18, 30, 45, 46] {
      let bar = select_waybar(&layers, 1600, y, (0, 0, 1920, 1080, 64)).unwrap();
      assert_eq!(
        bar,
        WaybarGeometry {
          top: 18,
          right: 1902,
          bottom: 46
        }
      );
      assert_eq!(
        popup_origin(bar.right, bar.bottom, 0, 0, 1920, 1080),
        (1504, 50)
      );
    }
  }

  #[test]
  fn argvus_taskbar_layer_is_recognized_at_top_and_bottom() {
    let top = serde_json::json!({"HDMI-A-1": {"levels": {"1": [
      {"namespace":"argvus-taskbar", "x":18, "y":18, "w":1884, "h":28},
      {"namespace":"argvus-widget-telemetry", "x":18, "y":64, "w":384, "h":998}
    ]}}});
    let bar = select_waybar(&top, 1600, 30, (0, 0, 1920, 1080, 64)).unwrap();
    assert_eq!(
      bar,
      WaybarGeometry {
        top: 18,
        right: 1902,
        bottom: 46
      }
    );
    assert_eq!(
      popup_origin(bar.right, bar.bottom, 0, 0, 1920, 1080),
      (1504, 50)
    );

    let bottom = serde_json::json!({"HDMI-A-1": {"levels": {"1": [
      {"namespace":"argvus-taskbar", "x":18, "y":1034, "w":1884, "h":28}
    ]}}});
    let bar = select_waybar(&bottom, 1600, 1040, (0, 0, 1920, 1080, 0)).unwrap();
    assert_eq!(bar.top, 1034);
    assert_eq!(bar.right, 1902);
    assert_eq!(bar.bottom, 1062);
  }

  #[test]
  fn bottom_bar_is_found_even_with_surface_local_click_coordinates() {
    let layers = serde_json::json!({"HDMI-A-1": {"levels": {"1": [
      {"namespace":"waybar", "x":0, "y":1050, "w":1920, "h":28},
      {"namespace":"waybar", "x":0, "y":0, "w":384, "h":1050}
    ]}}});
    for y in [7, 1050, 1077] {
      let bar = select_waybar(&layers, 1600, y, (0, 0, 1920, 1080, 0)).unwrap();
      assert_eq!(bar.top, 1050);
      assert_eq!(bar.right, 1920);
    }
  }

  #[test]
  fn current_calendar_theme_cache_is_loaded() {
    let root = std::env::temp_dir().join(format!("argvus-calendar-theme-{}", uuid::Uuid::new_v4()));
    let theme_dir = root.join("themes");
    std::fs::create_dir_all(&theme_dir).unwrap();
    let paths = test_theme_paths(&root, "gruvbox-dark-float");
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
    let paths = test_theme_paths(&root, "sunset");
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
  fn current_cache_is_loaded_before_new_theme_asset_is_installed() {
    let root = std::env::temp_dir().join(format!("argvus-calendar-theme-{}", uuid::Uuid::new_v4()));
    let paths = test_theme_paths(&root, "sunset");
    std::fs::remove_file(paths.theme_dir.join("sunset.css")).unwrap();
    std::fs::create_dir_all(paths.active_argvus_theme_file.parent().unwrap()).unwrap();
    std::fs::write(&paths.active_argvus_theme_file, "sunset\n").unwrap();
    std::fs::create_dir_all(paths.cache_theme_file.parent().unwrap()).unwrap();
    std::fs::write(&paths.cache_theme_file, "/* argvus-theme: sunset */\n").unwrap();

    let stylesheets = stylesheet_paths(&paths);
    assert!(stylesheets.contains(&paths.cache_theme_file));
    std::fs::remove_dir_all(root).unwrap();
  }

  fn test_theme_paths(root: &std::path::Path, theme: &str) -> Paths {
    let paths = Paths {
      database: root.join("data/calendar.db"),
      config_file: root.join("config/config.toml"),
      legacy_settings_file: root.join("config/settings.toml"),
      user_config_file: root.join("user/config.toml"),
      user_style: root.join("config/style.css"),
      theme_file: root.join("config/theme.css"),
      theme_dir: root.join("themes"),
      cache_theme_file: root.join("cache/theme.css"),
      active_argvus_theme_file: root.join("config/argvus/data/.active-theme"),
      events_enabled_file: root.join("state/events-enabled"),
      config_dir: root.join("config"),
      data_dir: root.join("data"),
      state_dir: root.join("state"),
      cache_dir: root.join("cache"),
    };
    std::fs::create_dir_all(&paths.theme_dir).unwrap();
    std::fs::write(paths.theme_dir.join(format!("{theme}.css")), "").unwrap();
    paths
  }
}
