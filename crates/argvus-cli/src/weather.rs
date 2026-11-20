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

#[derive(Clone, Deserialize, Serialize)]
struct Place {
  id: Option<u64>,
  name: String,
  location: String,
  used: Option<u64>,
}

struct Files {
  location: PathBuf,
  recent: PathBuf,
}

impl Files {
  fn new() -> Result<Self, Failure> {
    let data = crate::paths::Layout::from_environment()?
      .config_home()
      .join("argvus/data");
    Ok(Self {
      location: data.join(".weather-location"),
      recent: data.join("weather-recent.json"),
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

struct Picker {
  files: Files,
  lang: Lang,
  list: gtk::ListBox,
  entry: gtk::SearchEntry,
  message: gtk::Label,
  clear: gtk::Button,
  auto: gtk::Button,
  window: gtk::Window,
  done: glib::MainLoop,
  rows: RefCell<Vec<Place>>,
  recent_count: gtk::Label,
  confirmation: gtk::Box,
  cancel: gtk::Button,
  generation: Cell<u64>,
  detecting: Cell<bool>,
  debounce: RefCell<Option<glib::SourceId>>,
}

impl Picker {
  fn fill(&self, places: Vec<Place>) {
    while let Some(child) = self.list.first_child() {
      self.list.remove(&child);
    }
    let key = Lang::for_domain(&self.lang.locale(), "weather-picker");
    for place in &places {
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
      self.list.append(&picker::row(
        "mark-location-symbolic",
        &place.name,
        "",
        &age,
      ));
    }
    self.list.select_row(self.list.row_at_index(0).as_ref());
    self.rows.replace(places);
  }

  fn recent(&self) {
    let lang = Lang::for_domain(&self.lang.locale(), "weather-picker");
    match self.files.recent() {
      Ok(places) => {
        self
          .recent_count
          .set_label(&Lang::for_domain(&self.lang.locale(), "weather-picker").tr("recent"));
        self.clear.set_sensitive(!places.is_empty());
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
        if state.generation.get() != generation {
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

  fn select(&self, row: &gtk::ListBoxRow) {
    if self.detecting.get() || self.confirmation.is_visible() {
      return;
    }
    let Some(place) = self.rows.borrow().get(row.index() as usize).cloned() else {
      return;
    };
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
    if self.confirmation.is_visible() || self.detecting.replace(true) {
      return;
    }
    self.auto.set_sensitive(false);
    self.message.set_label(&self.lang.tr("weather.loading"));
    let state = self.clone();
    glib::MainContext::default().spawn_local(async move {
      let result = gio::spawn_blocking(detect).await;
      state.detecting.set(false);
      state.auto.set_sensitive(true);
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

  fn clear_recent(self: &Rc<Self>) {
    if !self.clear.is_sensitive() {
      return;
    }
    if self.confirmation.is_visible() || self.detecting.get() {
      return;
    }
    self.confirmation.set_visible(true);
    self.cancel.grab_focus();
  }

  fn finish_clear(&self, accept: bool) {
    self.confirmation.set_visible(false);
    self.clear.grab_focus();
    if !accept {
      return;
    }
    if let Err(error) = write_atomic(&self.files.recent, b"[]\n") {
      self.error(error);
      return;
    }
    if self.entry.text().is_empty() {
      self.recent();
    } else {
      self.entry.set_text("");
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
  panel.add_css_class("clipboard-picker");
  let entry = gtk::SearchEntry::builder()
    .placeholder_text(lang.tr("weather.location.prompt"))
    .build();
  entry.add_css_class("argvus-search");
  panel.append(&entry);
  let actions = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  let auto = gtk::Button::with_label(&lang.tr("weather.location.auto"));
  auto.set_hexpand(true);
  let recent_count = gtk::Label::new(None);
  let clear = gtk::Button::with_label(&strings.tr("clear"));
  actions.append(&auto);
  actions.append(&recent_count);
  actions.append(&clear);
  panel.append(&actions);
  let list = gtk::ListBox::new();
  panel.append(&picker::scroll(&list));
  list.set_activate_on_single_click(false);
  let message = gtk::Label::new(None);
  message.set_wrap(true);
  panel.append(&message);
  let confirmation = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  let prompt = gtk::Label::new(Some(&strings.tr("confirm")));
  prompt.set_wrap(true);
  let cancel = gtk::Button::with_label(&hints.tr("cancel"));
  let confirm = gtk::Button::with_label(&strings.tr("clear"));
  confirmation.append(&prompt);
  confirmation.append(&cancel);
  confirmation.append(&confirm);
  confirmation.set_visible(false);
  panel.append(&confirmation);
  picker::footer(
    &panel,
    &[
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
    entry,
    message,
    clear,
    auto,
    window,
    done: glib::MainLoop::new(None, false),
    rows: RefCell::new(Vec::new()),
    recent_count,
    confirmation,
    cancel,
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
  let weak = Rc::downgrade(&state);
  state.list.connect_row_activated(move |_, row| {
    if let Some(state) = weak.upgrade() {
      state.select(row);
    }
  });
  let weak = Rc::downgrade(&state);
  state.auto.connect_clicked(move |_| {
    if let Some(state) = weak.upgrade() {
      state.automatic();
    }
  });
  let weak = Rc::downgrade(&state);
  state.clear.connect_clicked(move |_| {
    if let Some(state) = weak.upgrade() {
      state.clear_recent();
    }
  });
  let weak = Rc::downgrade(&state);
  state.cancel.connect_clicked(move |_| {
    if let Some(state) = weak.upgrade() {
      state.finish_clear(false);
    }
  });
  let weak = Rc::downgrade(&state);
  confirm.connect_clicked(move |_| {
    if let Some(state) = weak.upgrade() {
      state.finish_clear(true);
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
    if state.confirmation.is_visible() {
      if key == gdk::Key::Escape {
        state.finish_clear(false);
        return glib::Propagation::Stop;
      }
      return glib::Propagation::Proceed;
    }
    let focus = gtk::prelude::GtkWindowExt::focus(&state.window);
    let in_entry = focus.as_ref().is_some_and(|focus| {
      focus == state.entry.upcast_ref::<gtk::Widget>() || focus.is_ancestor(&state.entry)
    });
    let on_button = focus
      .as_ref()
      .is_some_and(|focus| focus.is::<gtk::Button>());
    match key {
      gdk::Key::Escape => state.done.quit(),
      gdk::Key::slash if !in_entry => {
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
      gdk::Key::Delete if !in_entry => state.clear_recent(),
      gdk::Key::Return | gdk::Key::KP_Enter if !on_button => {
        if let Some(row) = state.list.selected_row() {
          state.select(&row);
        }
      }
      gdk::Key::Up | gdk::Key::Down => {
        let step = if key == gdk::Key::Up { -1 } else { 1 };
        let index = state
          .list
          .selected_row()
          .map_or(0, |row| row.index() + step);
        if let Some(row) = state.list.row_at_index(index) {
          state.list.select_row(Some(&row));
          row.grab_focus();
        }
      }
      _ => return glib::Propagation::Proceed,
    }
    glib::Propagation::Stop
  });
  state.window.add_controller(keys);
  state.recent();
  state.window.present();
  if let Some(row) = state.list.selected_row() {
    row.grab_focus();
  }
  state.done.run();
  state.window.destroy();
  Ok(())
}
