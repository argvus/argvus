//! Weather location state and the GTK picker shared by the shortcut and panel.

use std::{
  cell::{Cell, RefCell},
  ffi::OsString,
  fs,
  io::{self, Write},
  path::{Path, PathBuf},
  process::{Command, ExitCode},
  rc::Rc,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use argvus_i18n::{Lang, TextDirection};
use argvus_theme_selector::picker;
use gtk::{gdk, gio, glib, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::exec::{Failure, Outcome};

#[derive(Clone, Deserialize, PartialEq, Eq, Serialize)]
struct Place {
  id: Option<u64>,
  name: String,
  location: String,
  used: Option<u64>,
}

struct Files {
  location: PathBuf,
  recent: PathBuf,
  temperature_unit: PathBuf,
}

impl Files {
  fn new() -> Result<Self, Failure> {
    let data = crate::paths::Layout::from_environment()?
      .config_home()
      .join("argvus/data");
    Ok(Self {
      location: data.join(".weather-location"),
      recent: data.join("weather-recent.json"),
      temperature_unit: data.join(".weather-temperature-unit"),
    })
  }

  fn current(&self) -> io::Result<String> {
    match fs::read_to_string(&self.location) {
      Ok(text) => Ok(text.lines().next().unwrap_or_default().trim().into()),
      Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
      Err(error) => Err(error),
    }
  }

  fn recent(&self) -> io::Result<Vec<Place>> {
    match fs::read(&self.recent) {
      Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
      Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(legacy_place(self.current()?)),
      Err(error) => Err(error),
    }
  }

  fn save(&self, location: &str, name: Option<&str>) -> io::Result<()> {
    let mut record = format!("{location}\n");
    if let Some(name) = name {
      record.push_str(&name.replace(['\r', '\n'], " "));
      record.push('\n');
    }
    write_atomic(&self.location, record.as_bytes())
  }

  fn remember(&self, mut place: Place) -> io::Result<()> {
    let mut recent = self.recent()?;
    recent.retain(|previous| match (place.id, previous.id) {
      (Some(id), Some(other)) => id != other,
      _ => glib::casefold(&place.location) != glib::casefold(&previous.location),
    });
    place.used = Some(now());
    recent.insert(0, place);
    recent.truncate(20);
    write_atomic(&self.recent, &serde_json::to_vec(&recent)?)
  }
}

fn legacy_place(location: String) -> Vec<Place> {
  if location.is_empty() {
    Vec::new()
  } else {
    vec![Place {
      id: None,
      name: location.clone(),
      location,
      used: None,
    }]
  }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
  let parent = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
  fs::create_dir_all(parent)?;
  let mut file = tempfile::NamedTempFile::new_in(parent)?;
  file.write_all(bytes)?;
  file.persist(path)?;
  Ok(())
}

fn now() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs()
}

fn fetch(url: &str, params: &[(&str, &str)]) -> Result<Value, String> {
  let mut command = Command::new("curl");
  command.args([
    "-q",
    "-fsS",
    "--connect-timeout",
    "8",
    "--max-time",
    "12",
    "--max-filesize",
    "262144",
    "--get",
    url,
  ]);
  for (key, value) in params {
    command
      .arg("--data-urlencode")
      .arg(format!("{key}={value}"));
  }
  let response = command.output().map_err(|error| error.to_string())?;
  if !response.status.success() {
    return Err(String::from_utf8_lossy(&response.stderr).trim().into());
  }
  serde_json::from_slice(&response.stdout).map_err(|error| error.to_string())
}

fn search(query: &str, language: &str) -> Result<Vec<Place>, String> {
  let response = fetch(
    "https://geocoding-api.open-meteo.com/v1/search",
    &[("name", query), ("count", "10"), ("language", language)],
  )?;
  if response["error"].as_bool() == Some(true) || !response.is_object() {
    return Err("Invalid geocoding response".into());
  }
  Ok(
    response["results"]
      .as_array()
      .into_iter()
      .flatten()
      .filter_map(|value| {
        let id = value["id"].as_u64()?;
        let city = value["name"].as_str()?.trim();
        let lat = value["latitude"].as_f64()?;
        let lon = value["longitude"].as_f64()?;
        if city.is_empty() || lat.abs() > 90.0 || lon.abs() > 180.0 {
          return None;
        }
        let mut parts = vec![city];
        for key in ["admin1", "country"] {
          if let Some(part) = value[key]
            .as_str()
            .filter(|part| !part.is_empty() && !parts.contains(part))
          {
            parts.push(part);
          }
        }
        Some(Place {
          id: Some(id),
          name: parts.join(", "),
          location: format!("{lat},{lon}"),
          used: None,
        })
      })
      .take(10)
      .collect(),
  )
}

fn detect() -> Result<String, String> {
  let data = fetch("https://ipinfo.io/json", &[])?;
  let city = data["city"]
    .as_str()
    .map(str::trim)
    .filter(|city| !city.is_empty())
    .ok_or("IP provider returned no city")?;
  Ok(
    match data["country"]
      .as_str()
      .filter(|country| !country.is_empty())
    {
      Some(country) => format!("{city}, {country}"),
      None => city.into(),
    },
  )
}

fn report(lang: Lang, location: &str, detected: Option<&str>) {
  let summary = lang.tr("weather.notification.title");
  let automatic = lang.tr("weather.notification.auto");
  let message = detected.unwrap_or(if location.is_empty() {
    &automatic
  } else {
    location
  });
  let _ = Command::new("notify-send")
    .args(["--", &summary, message])
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .spawn();
  println!("{location}");
}

pub fn run(args: &[OsString]) -> Outcome {
  let locale = argvus_i18n::user_locale();
  let lang = Lang::for_domain(&locale, "control-panel");
  let files = Files::new()?;
  let fail = |error: io::Error| Failure::new(error.to_string());
  match args {
    [] => show(files, &locale, lang)?,
    [arg] if arg == "status" => {
      let location = files.current().map_err(fail)?;
      if files.location.exists() {
        println!("{location}");
      }
    }
    [arg] if arg == "--help" || arg == "-h" => {
      println!("usage: argvus weather-location [status|--auto|LOCATION]")
    }
    [arg] => {
      let value = if arg == "--auto" {
        String::new()
      } else {
        arg.to_string_lossy().replace(['\r', '\n'], "")
      };
      let value = value.trim();
      if value.chars().count() > 120 {
        return Err(Failure::new(lang.tr("weather.location.too_long")));
      }
      files.save(value, None).map_err(fail)?;
      report(lang, value, None);
    }
    _ => {
      return Err(Failure::new(
        "usage: argvus weather-location [status|--auto|LOCATION]",
      ));
    }
  }
  Ok(ExitCode::SUCCESS)
}

#[derive(Clone, PartialEq, Eq)]
enum Row {
  City(Place),
  Automatic,
  Temperature,
  Clear,
  Cancel,
  Confirm,
}

struct Picker {
  files: Files,
  lang: Lang,
  list: gtk::ListBox,
  actions: gtk::ListBox,
  stack: gtk::Stack,
  entry: gtk::SearchEntry,
  message: gtk::Label,
  window: gtk::Window,
  done: glib::MainLoop,
  rows: RefCell<Vec<Row>>,
  action_rows: RefCell<Vec<Row>>,
  has_recent: Cell<bool>,
  confirming: Cell<bool>,
  fahrenheit: Cell<bool>,
  generation: Cell<u64>,
  detecting: Cell<bool>,
  debounce: RefCell<Option<glib::SourceId>>,
}

impl Picker {
  fn fill(&self, places: Vec<Place>) {
    let list = &self.list;
    let rows = &self.rows;
    let selected = self.selected(list);
    let restore_focus = self.focus_in(list);
    while let Some(child) = list.first_child() {
      list.remove(&child);
    }
    let key = Lang::for_domain(&self.lang.locale(), "weather-picker");
    let mut entries = Vec::new();
    for place in places {
      let age = place
        .used
        .map(|used| {
          let mins = now().saturating_sub(used) / 60;
          let (unit, count) = if mins >= 1440 {
            ("days", mins / 1440)
          } else if mins >= 60 {
            ("hours", mins / 60)
          } else {
            ("minutes", mins)
          };
          key.tr_args(unit, [("count", count.to_string())])
        })
        .unwrap_or_default();
      list.append(&picker::row(
        "find-location-symbolic",
        &place.name,
        "",
        &age,
      ));
      entries.push(Row::City(place));
    }
    self.finish_render(list, rows, entries, selected, restore_focus);
  }

  fn selected(&self, list: &gtk::ListBox) -> Option<Row> {
    self.row_at(list, &list.selected_row()?)
  }

  fn row_at(&self, list: &gtk::ListBox, row: &gtk::ListBoxRow) -> Option<Row> {
    let rows = if list == &self.list {
      &self.rows
    } else {
      &self.action_rows
    };
    rows.borrow().get(row.index() as usize).cloned()
  }

  fn focus_in(&self, widget: &impl IsA<gtk::Widget>) -> bool {
    gtk::prelude::GtkWindowExt::focus(&self.window)
      .is_some_and(|focus| &focus == widget.upcast_ref() || focus.is_ancestor(widget))
  }

  fn render_actions(&self, preferred: Option<Row>) {
    let list = &self.actions;
    let selected = preferred.or_else(|| self.selected(list));
    let restore_focus = self.focus_in(list);
    while let Some(child) = list.first_child() {
      list.remove(&child);
    }
    let key = Lang::for_domain(&self.lang.locale(), "weather-picker");
    let hints = Lang::for_domain(&self.lang.locale(), "clipboard");
    let mut rows = Vec::new();
    let mut append = |row, icon, title: &str, detail: &str| {
      list.append(&picker::row(icon, title, "", detail));
      rows.push(row);
    };
    if self.confirming.get() {
      append(
        Row::Cancel,
        "process-stop-symbolic",
        &hints.tr("cancel"),
        "",
      );
      append(Row::Confirm, "edit-clear-symbolic", &key.tr("clear"), "");
    } else {
      append(
        Row::Automatic,
        "find-location-symbolic",
        &self.lang.tr("weather.location.auto"),
        "A",
      );
      append(
        Row::Temperature,
        "open-menu-symbolic",
        &key.tr("temperature_unit"),
        if self.fahrenheit.get() { "°F" } else { "°C" },
      );
      if self.has_recent.get() {
        append(Row::Clear, "edit-clear-symbolic", &key.tr("clear"), "Del");
      }
    }
    self.finish_render(list, &self.action_rows, rows, selected, restore_focus);
  }

  fn finish_render(
    &self,
    list: &gtk::ListBox,
    model: &RefCell<Vec<Row>>,
    rows: Vec<Row>,
    selected: Option<Row>,
    restore_focus: bool,
  ) {
    let index = selected
      .and_then(|selected| rows.iter().position(|row| *row == selected))
      .unwrap_or(0);
    model.replace(rows);
    if let Some(row) = list.row_at_index(index as i32) {
      list.select_row(Some(&row));
      if restore_focus {
        row.grab_focus();
      }
    }
  }

  fn recent(&self) {
    let lang = Lang::for_domain(&self.lang.locale(), "weather-picker");
    match self.files.recent() {
      Ok(places) => {
        self.has_recent.set(!places.is_empty());
        let message = if places.is_empty() {
          lang.tr("empty")
        } else {
          String::new()
        };
        self.message.set_label(&message);
        self.fill(places);
      }
      Err(error) => self.error(error),
    }
  }

  fn error(&self, error: impl std::fmt::Display) {
    eprintln!("argvus weather-location: {error}");
    self.message.set_label(&self.lang.tr("weather.error"));
  }

  fn changed(self: &Rc<Self>) {
    if let Some(timer) = self.debounce.borrow_mut().take() {
      timer.remove();
    }
    let generation = self.generation.get().wrapping_add(1);
    self.generation.set(generation);
    let query = self.entry.text().trim().to_owned();
    if query.is_empty() {
      self.recent();
      return;
    }
    self.fill(Vec::new());
    if query.chars().count() < 2 {
      self.message.set_label("");
      return;
    }
    self.message.set_label(&self.lang.tr("weather.loading"));
    let state = self.clone();
    let timer = glib::timeout_add_local_once(Duration::from_millis(350), move || {
      state.debounce.borrow_mut().take();
      let locale = state.lang.locale();
      let language = locale
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_lowercase();
      glib::MainContext::default().spawn_local(async move {
        let result = gio::spawn_blocking(move || search(&query, &language)).await;
        if state.generation.get() != generation || state.confirming.get() {
          return;
        }
        match result {
          Ok(Ok(places)) => {
            let lang = Lang::for_domain(&locale, "weather-picker");
            let message = if places.is_empty() {
              lang.tr("no_results")
            } else {
              String::new()
            };
            state.message.set_label(&message);
            state.fill(places);
          }
          Ok(Err(error)) => state.error(error),
          Err(_) => state.error("Search worker failed"),
        }
      });
    });
    self.debounce.replace(Some(timer));
  }

  fn activate(self: &Rc<Self>, row: Row) {
    if self.detecting.get() {
      return;
    }
    match row {
      Row::Cancel => self.finish_clear(false),
      Row::Confirm => self.finish_clear(true),
      Row::City(place) => self.select_city(place),
      Row::Automatic => self.automatic(),
      Row::Temperature => self.toggle_temperature(),
      Row::Clear => self.clear_recent(),
    }
  }

  fn toggle_temperature(&self) {
    let fahrenheit = !self.fahrenheit.get();
    let value = if fahrenheit {
      b"fahrenheit\n".as_slice()
    } else {
      b"celsius\n".as_slice()
    };
    match write_atomic(&self.files.temperature_unit, value) {
      Ok(()) => {
        self.fahrenheit.set(fahrenheit);
        self.render_actions(Some(Row::Temperature));
      }
      Err(error) => self.error(error),
    }
  }

  fn select_city(&self, place: Place) {
    if let Err(error) = self
      .files
      .remember(place.clone())
      .and_then(|()| self.files.save(&place.location, Some(&place.name)))
    {
      self.error(error);
      return;
    }
    report(self.lang, &place.location, Some(&place.name));
    self.done.quit();
  }

  fn automatic(self: &Rc<Self>) {
    if self.confirming.get() || self.detecting.replace(true) {
      return;
    }
    self.message.set_label(&self.lang.tr("weather.loading"));
    let state = self.clone();
    glib::MainContext::default().spawn_local(async move {
      let result = gio::spawn_blocking(detect).await;
      state.detecting.set(false);
      match result {
        Ok(Ok(city)) => match state.files.save("", None) {
          Ok(()) => {
            report(state.lang, "", Some(&city));
            state.done.quit();
          }
          Err(error) => state.error(error),
        },
        Ok(Err(error)) => state.error(error),
        Err(_) => state.error("IP lookup worker failed"),
      }
    });
  }

  fn clear_recent(&self) {
    if !self.has_recent.get() || self.confirming.get() || self.detecting.get() {
      return;
    }
    self.confirming.set(true);
    self.stack.set_visible_child_name("actions");
    self
      .message
      .set_label(&Lang::for_domain(&self.lang.locale(), "weather-picker").tr("confirm"));
    self.render_actions(Some(Row::Cancel));
    if let Some(row) = self.actions.selected_row() {
      row.grab_focus();
    }
  }

  fn finish_clear(self: &Rc<Self>, accept: bool) {
    if accept {
      if let Err(error) = write_atomic(&self.files.recent, b"[]\n") {
        self.error(error);
        return;
      }
      self.has_recent.set(false);
    }
    self.confirming.set(false);
    self.message.set_label("");
    if self.entry.text().trim().is_empty() {
      self.recent();
    } else {
      self.changed();
    }
    self.render_actions(Some(if accept { Row::Automatic } else { Row::Clear }));
    if let Some(row) = self.actions.selected_row() {
      row.grab_focus();
    }
  }
}

fn show(files: Files, locale: &str, lang: Lang) -> Result<(), Failure> {
  gtk::init().map_err(|error| Failure::new(error.to_string()))?;
  gtk::Widget::set_default_direction(match lang.direction() {
    TextDirection::LeftToRight => gtk::TextDirection::Ltr,
    TextDirection::RightToLeft => gtk::TextDirection::Rtl,
  });
  let hints = Lang::for_domain(locale, "clipboard");
  let strings = Lang::for_domain(locale, "weather-picker");
  let title = lang.tr("weather.notification.title");
  let window = picker::window("argvus-weather", &format!("ARGVUS {title}"), locale);
  let panel = picker::panel(&window, &title);
  panel.add_css_class("weather-picker");
  let entry = gtk::SearchEntry::builder()
    .placeholder_text(lang.tr("weather.location.prompt"))
    .build();
  entry.add_css_class("argvus-search");
  let stack = gtk::Stack::new();
  stack.set_vexpand(true);
  stack.set_hexpand(true);
  let browser = gtk::Box::new(gtk::Orientation::Horizontal, 16);
  browser.add_css_class("weather-browser");
  browser.set_vexpand(true);
  let tabs = gtk::ListBox::new();
  tabs.add_css_class("weather-tabs");
  tabs.set_activate_on_single_click(true);
  for (icon, key) in [
    ("find-location-symbolic", "locations"),
    ("open-menu-symbolic", "actions"),
  ] {
    tabs.append(&picker::row(icon, &strings.tr(key), "", ""));
  }
  tabs.select_row(tabs.row_at_index(0).as_ref());
  browser.append(&tabs);
  browser.append(&stack);
  panel.append(&browser);
  let locations = gtk::Box::new(gtk::Orientation::Vertical, 12);
  locations.append(&entry);
  let fahrenheit =
    fs::read_to_string(&files.temperature_unit).is_ok_and(|value| value.trim() == "fahrenheit");
  let list = gtk::ListBox::new();
  locations.append(&picker::scroll(&list));
  stack.add_titled(&locations, Some("locations"), &strings.tr("locations"));
  let actions = gtk::ListBox::new();
  stack.add_titled(
    &picker::scroll(&actions),
    Some("actions"),
    &strings.tr("actions"),
  );
  let message = gtk::Label::new(None);
  message.set_wrap(true);
  panel.append(&message);
  picker::footer(
    &panel,
    &[
      ("Tab", &strings.tr("tabs")),
      ("↑ ↓", &hints.tr("navigate")),
      (
        "Enter",
        &Lang::for_domain(locale, "power").tr("idle.timeout.select"),
      ),
      ("/", &hints.tr("search_hint")),
      ("A", &lang.tr("weather.location.auto")),
      ("Del", &strings.tr("clear")),
      ("Esc", &hints.tr("close")),
    ],
  );

  let state = Rc::new(Picker {
    files,
    lang,
    list,
    actions,
    stack,
    entry,
    message,
    window,
    done: glib::MainLoop::new(None, false),
    rows: RefCell::new(Vec::new()),
    action_rows: RefCell::new(Vec::new()),
    has_recent: Cell::new(false),
    confirming: Cell::new(false),
    fahrenheit: Cell::new(fahrenheit),
    generation: Cell::new(0),
    detecting: Cell::new(false),
    debounce: RefCell::new(None),
  });
  let weak = Rc::downgrade(&state);
  state.entry.connect_changed(move |_| {
    if let Some(state) = weak.upgrade() {
      state.changed();
    }
  });
  for list in [&state.list, &state.actions] {
    let weak = Rc::downgrade(&state);
    list.connect_row_activated(move |list, row| {
      if let Some(state) = weak.upgrade()
        && let Some(action) = state.row_at(list, row)
      {
        state.activate(action);
      }
    });
  }
  let weak = Rc::downgrade(&state);
  tabs.connect_row_selected(move |_, row| {
    if let Some(state) = weak.upgrade()
      && let Some(row) = row
    {
      let page = if row.index() == 0 {
        "locations"
      } else {
        "actions"
      };
      state.stack.set_visible_child_name(page);
      if page == "locations" && state.confirming.get() {
        state.finish_clear(false);
      }
      let list = if page == "locations" {
        &state.list
      } else {
        &state.actions
      };
      if let Some(row) = list.selected_row() {
        row.grab_focus();
      } else {
        state.entry.grab_focus();
      }
    }
  });
  let done = state.done.clone();
  state.window.connect_close_request(move |_| {
    done.quit();
    glib::Propagation::Stop
  });
  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  let weak = Rc::downgrade(&state);
  keys.connect_key_pressed(move |_, key, _, mods| {
    let Some(state) = weak.upgrade() else {
      return glib::Propagation::Proceed;
    };
    let in_entry = state.focus_in(&state.entry);
    let list = if state.stack.visible_child_name().as_deref() == Some("actions") {
      &state.actions
    } else {
      &state.list
    };
    if matches!(key, gdk::Key::Tab | gdk::Key::ISO_Left_Tab)
      && !mods.intersects(
        gdk::ModifierType::CONTROL_MASK
          | gdk::ModifierType::ALT_MASK
          | gdk::ModifierType::SUPER_MASK,
      )
    {
      let index = i32::from(list == &state.list);
      tabs.select_row(tabs.row_at_index(index).as_ref());
      return glib::Propagation::Stop;
    }
    if state.confirming.get() {
      if key == gdk::Key::Escape {
        state.finish_clear(false);
        return glib::Propagation::Stop;
      }
      if !matches!(
        key,
        gdk::Key::Up | gdk::Key::Down | gdk::Key::Return | gdk::Key::KP_Enter
      ) {
        return glib::Propagation::Stop;
      }
    }
    match key {
      gdk::Key::Escape => state.done.quit(),
      gdk::Key::slash if !in_entry => {
        tabs.select_row(tabs.row_at_index(0).as_ref());
        state.entry.grab_focus();
      }
      gdk::Key::a | gdk::Key::A
        if !in_entry
          && !mods.intersects(
            gdk::ModifierType::CONTROL_MASK
              | gdk::ModifierType::ALT_MASK
              | gdk::ModifierType::SUPER_MASK,
          ) =>
      {
        state.automatic()
      }
      gdk::Key::Delete if !in_entry => {
        if state.has_recent.get() && !state.detecting.get() {
          tabs.select_row(tabs.row_at_index(1).as_ref());
          state.clear_recent();
        }
      }
      gdk::Key::Return | gdk::Key::KP_Enter => {
        if let Some(row) = state.selected(list) {
          state.activate(row);
        }
      }
      gdk::Key::Up | gdk::Key::Down => {
        let step = if key == gdk::Key::Up { -1 } else { 1 };
        let index = list.selected_row().map_or(0, |row| row.index() + step);
        if let Some(row) = list.row_at_index(index) {
          list.select_row(Some(&row));
          row.grab_focus();
        }
      }
      _ => return glib::Propagation::Proceed,
    }
    glib::Propagation::Stop
  });
  state.window.add_controller(keys);
  state.recent();
  state.render_actions(None);
  state.window.present();
  if let Some(row) = state.list.selected_row() {
    row.grab_focus();
  }
  if state.list.selected_row().is_none() {
    state.entry.grab_focus();
  }
  state.done.run();
  state.window.destroy();
  Ok(())
}
