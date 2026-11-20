//! GTK front end for the session's cliphist database.

use std::{
  cell::{Cell, RefCell},
  ffi::OsString,
  io::Write,
  process::{Command, ExitCode, Stdio},
  rc::Rc,
};

use argvus_i18n::{Lang, TextDirection, tr};
use argvus_theme_selector::picker;
use gtk::{gdk, gio, glib, prelude::*};

use crate::exec::{Failure, Outcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
  Text,
  Link,
  Image,
  Binary,
}

impl Kind {
  fn label(self) -> &'static str {
    match self {
      Self::Text => "kind.text",
      Self::Link => "kind.link",
      Self::Image => "kind.image",
      Self::Binary => "kind.binary",
    }
  }

  fn icon(self) -> &'static str {
    match self {
      Self::Text => "text-x-generic-symbolic",
      Self::Link => "insert-link-symbolic",
      Self::Image => "image-x-generic-symbolic",
      Self::Binary => "application-x-generic-symbolic",
    }
  }
}

#[derive(Debug)]
struct Record {
  line: Vec<u8>,
  description: String,
  kind: Kind,
}

impl Record {
  fn parse(line: &[u8]) -> Option<Self> {
    let tab = line.iter().position(|byte| *byte == b'\t')?;
    let id = std::str::from_utf8(&line[..tab])
      .ok()?
      .parse::<u64>()
      .ok()?;
    if id == 0 {
      return None;
    }
    let description = String::from_utf8_lossy(&line[tab + 1..]).into_owned();
    let kind = if let Some(metadata) = description
      .strip_prefix("[[ binary data ")
      .and_then(|text| text.strip_suffix(" ]]"))
    {
      let dimensions = metadata
        .split_whitespace()
        .next_back()
        .and_then(|size| size.split_once('x'))
        .is_some_and(|(width, height)| {
          width.parse::<u32>().is_ok() && height.parse::<u32>().is_ok()
        });
      if dimensions {
        Kind::Image
      } else {
        Kind::Binary
      }
    } else if ["https://", "http://"]
      .iter()
      .any(|prefix| description.starts_with(prefix))
      && !description.chars().any(char::is_whitespace)
    {
      Kind::Link
    } else {
      Kind::Text
    };
    Some(Self {
      line: line.to_vec(),
      description,
      kind,
    })
  }

  fn display(&self, lang: Lang) -> String {
    match self.kind {
      Kind::Image | Kind::Binary => {
        let metadata = self
          .description
          .trim_start_matches("[[ binary data ")
          .trim_end_matches(" ]]");
        format!("{} · {metadata}", tr(lang, self.kind.label()))
      }
      _ if self.description.trim().is_empty() => tr(lang, "empty_description").to_owned(),
      _ => self.description.clone(),
    }
  }
}

fn subprocess(
  program: &str,
  args: &[&str],
  input: Option<&[u8]>,
  capture: bool,
) -> Result<Vec<u8>, String> {
  let mut child = Command::new(program)
    .args(args)
    .stdin(if input.is_some() {
      Stdio::piped()
    } else {
      Stdio::null()
    })
    .stdout(if capture {
      Stdio::piped()
    } else {
      Stdio::null()
    })
    .stderr(if capture {
      Stdio::piped()
    } else {
      Stdio::null()
    })
    .spawn()
    .map_err(|error| format!("{program}: {error}"))?;
  let written = input
    .map(|bytes| {
      child
        .stdin
        .take()
        .ok_or_else(|| format!("{program}: missing stdin"))?
        .write_all(bytes)
        .map_err(|error| format!("{program}: {error}"))
    })
    .transpose();
  let output = child
    .wait_with_output()
    .map_err(|error| format!("{program}: {error}"))?;
  // cliphist exits with this diagnostic before the first item and after wipe.
  if program == "cliphist"
    && args == ["list"]
    && String::from_utf8_lossy(&output.stderr).trim() == "opening db: please store something first"
  {
    return Ok(Vec::new());
  }
  if !output.status.success() {
    return Err(format!("{program}: {}", output.status));
  }
  written?;
  Ok(output.stdout)
}

fn read_history() -> Result<Vec<Record>, String> {
  Ok(
    subprocess("cliphist", &["list"], None, true)?
      .split(|byte| *byte == b'\n')
      .filter_map(Record::parse)
      .collect(),
  )
}

fn copy(mut line: Vec<u8>) -> Result<(), String> {
  line.push(b'\n');
  let bytes = subprocess("cliphist", &["decode"], Some(&line), true)?;
  subprocess("wl-copy", &[], Some(&bytes), false)?;
  Ok(())
}

struct Clipboard {
  lang: Lang,
  records: RefCell<Vec<Record>>,
  descriptions: RefCell<Vec<String>>,
  stack: gtk::Stack,
  list: gtk::ListBox,
  search: gtk::SearchEntry,
  empty: gtk::Label,
  status: gtk::Label,
  clear: gtk::Button,
  cancel: gtk::Button,
  busy: Cell<bool>,
  running: glib::MainLoop,
}

impl Clipboard {
  fn replace(&self, records: Vec<Record>) {
    self.list.remove_all();
    let descriptions: Vec<_> = records
      .iter()
      .map(|entry| entry.display(self.lang))
      .collect();
    let display: Vec<_> = descriptions
      .iter()
      .map(|description| glib::casefold(description).to_string())
      .collect();
    self.descriptions.replace(display);
    self.records.replace(records);
    for (entry, description) in self.records.borrow().iter().zip(&descriptions) {
      self.list.append(&picker::row(
        entry.kind.icon(),
        description,
        "",
        tr(self.lang, entry.kind.label()),
      ));
    }
    self.clear.set_sensitive(!descriptions.is_empty());
    self.select_first();
  }

  fn select_first(&self) {
    let visible = (0..)
      .map_while(|index| self.list.row_at_index(index))
      .find(WidgetExt::is_child_visible);
    self.list.select_row(visible.as_ref());
    self.empty.set_label(tr(
      self.lang,
      if self.records.borrow().is_empty() {
        "empty"
      } else {
        "no_matches"
      },
    ));
  }

  fn move_selection(&self, step: i32) {
    let start = self.list.selected_row().map_or(-1, |row| row.index());
    let mut index = start + step;
    while let Some(row) = self.list.row_at_index(index) {
      if row.is_child_visible() {
        self.list.select_row(Some(&row));
        row.grab_focus();
        return;
      }
      index += step;
    }
  }

  fn focus_list(&self) {
    if let Some(row) = self.list.selected_row() {
      row.grab_focus();
    } else {
      self.search.grab_focus();
    }
  }

  fn show_confirmation(&self) {
    if !self.records.borrow().is_empty() {
      self.stack.set_visible_child_name("confirm");
      self.cancel.grab_focus();
    }
  }

  fn dismiss_confirmation(&self) {
    self.stack.set_visible_child_name("history");
    self.focus_list();
  }

  fn error(&self, key: &str, error: &str) {
    eprintln!("argvus clipboard: {error}");
    if key == "error.read" {
      self.empty.set_label(tr(self.lang, key));
      return;
    }
    self.status.set_label(tr(self.lang, key));
    self.status.set_visible(true);
  }

  fn load(self: &Rc<Self>) {
    self.busy.set(true);
    let state = self.clone();
    glib::spawn_future_local(async move {
      let result = gio::spawn_blocking(read_history).await;
      state.busy.set(false);
      match result {
        Ok(Ok(records)) => {
          state.replace(records);
          state.focus_list();
        }
        Ok(Err(error)) => state.error("error.read", &error),
        Err(_) => state.error("error.read", "history worker failed"),
      }
    });
  }

  fn copy_row(self: &Rc<Self>, row: &gtk::ListBoxRow) {
    let Some(line) = self
      .records
      .borrow()
      .get(row.index() as usize)
      .map(|entry| entry.line.clone())
    else {
      return;
    };
    if self.busy.get() {
      return;
    }
    self.busy.set(true);
    self.status.set_visible(false);
    let state = self.clone();
    glib::spawn_future_local(async move {
      let result = gio::spawn_blocking(move || copy(line)).await;
      state.busy.set(false);
      match result {
        Ok(Ok(())) => state.running.quit(),
        Ok(Err(error)) => state.error("error.copy", &error),
        Err(_) => state.error("error.copy", "copy worker failed"),
      }
    });
  }

  fn wipe(self: &Rc<Self>) {
    if self.busy.replace(true) {
      return;
    }
    let state = self.clone();
    glib::spawn_future_local(async move {
      let result = gio::spawn_blocking(|| subprocess("cliphist", &["wipe"], None, false)).await;
      state.busy.set(false);
      state.dismiss_confirmation();
      match result {
        Ok(Ok(_)) => {
          state.search.set_text("");
          state.replace(Vec::new());
          state.status.set_visible(false);
          state.load();
        }
        Ok(Err(error)) => state.error("error.clear", &error),
        Err(_) => state.error("error.clear", "clear worker failed"),
      }
    });
  }
}

pub fn run(args: &[OsString]) -> Outcome {
  if matches!(args, [arg] if arg == "--help" || arg == "-h") {
    println!("usage: argvus clipboard");
    return Ok(ExitCode::SUCCESS);
  }
  if !args.is_empty() {
    return Err(Failure::new("usage: argvus clipboard"));
  }
  let locale = argvus_i18n::user_locale();
  let lang = Lang::for_domain(&locale, "clipboard");
  let config = argvus_config_core::ConfigStore::from_environment()
    .and_then(|store| store.load_effective())
    .map_err(Failure::new)?;
  if config
    .get("/taskbar/icons/clipboard_enabled")
    .and_then(serde_json::Value::as_bool)
    == Some(false)
  {
    return Err(Failure::new(tr(lang, "disabled")));
  }
  gtk::init().map_err(|error| Failure::new(format!("GTK: {error}")))?;
  gtk::Widget::set_default_direction(match lang.direction() {
    TextDirection::LeftToRight => gtk::TextDirection::Ltr,
    TextDirection::RightToLeft => gtk::TextDirection::Rtl,
  });
  let title = tr(lang, "title");
  let window = picker::window("argvus-clipboard", &format!("ARGVUS {title}"), &locale);
  let panel = picker::panel(&window, title);
  panel.add_css_class("clipboard-picker");

  let stack = gtk::Stack::new();
  stack.set_vexpand(true);
  let history = gtk::Box::new(gtk::Orientation::Vertical, 12);
  let search = gtk::SearchEntry::builder()
    .placeholder_text(tr(lang, "search"))
    .build();
  search.add_css_class("argvus-search");
  history.append(&search);

  let list = gtk::ListBox::new();
  let empty = gtk::Label::new(Some(tr(lang, "loading")));
  empty.set_wrap(true);
  list.set_placeholder(Some(&empty));
  let scroll = picker::scroll(&list);
  list.set_activate_on_single_click(false);
  history.append(&scroll);

  let status = gtk::Label::new(None);
  status.set_wrap(true);
  status.set_xalign(0.0);
  status.set_visible(false);
  history.append(&status);

  let clear = gtk::Button::with_label(tr(lang, "clear"));
  clear.set_halign(gtk::Align::End);
  clear.set_sensitive(false);
  history.append(&clear);
  picker::footer(
    &history,
    &[
      ("↑ ↓", tr(lang, "navigate")),
      ("Enter", tr(lang, "copy")),
      ("/", tr(lang, "search_hint")),
      ("Delete", tr(lang, "clear")),
      ("Esc", tr(lang, "close")),
    ],
  );
  stack.add_named(&history, Some("history"));

  let confirm = gtk::Box::new(gtk::Orientation::Vertical, 12);
  confirm.set_valign(gtk::Align::Center);
  let message = gtk::Label::new(Some(tr(lang, "confirm_clear")));
  message.set_wrap(true);
  confirm.append(&message);
  let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  buttons.set_halign(gtk::Align::Center);
  let cancel = gtk::Button::with_label(tr(lang, "cancel"));
  let wipe = gtk::Button::with_label(tr(lang, "clear"));
  wipe.add_css_class("destructive-action");
  buttons.append(&cancel);
  buttons.append(&wipe);
  confirm.append(&buttons);
  stack.add_named(&confirm, Some("confirm"));
  panel.append(&stack);

  let state = Rc::new(Clipboard {
    lang,
    records: RefCell::new(Vec::new()),
    descriptions: RefCell::new(Vec::new()),
    stack,
    list,
    search,
    empty,
    status,
    clear,
    cancel,
    busy: Cell::new(false),
    running: glib::MainLoop::new(None, false),
  });
  connect(&window, &state, &wipe);
  window.present();
  state.load();
  state.running.run();
  window.destroy();
  Ok(ExitCode::SUCCESS)
}

fn connect(window: &gtk::Window, state: &Rc<Clipboard>, wipe: &gtk::Button) {
  let query = Rc::new(RefCell::new(String::new()));
  let filter = query.clone();
  let weak = Rc::downgrade(state);
  state.list.set_filter_func(move |row| {
    weak.upgrade().is_some_and(|state| {
      state
        .descriptions
        .borrow()
        .get(row.index() as usize)
        .is_some_and(|text| text.contains(filter.borrow().as_str()))
    })
  });
  let weak = Rc::downgrade(state);
  state.search.connect_changed(move |search| {
    if let Some(state) = weak.upgrade() {
      *query.borrow_mut() = glib::casefold(search.text().trim()).to_string();
      state.list.invalidate_filter();
      state.select_first();
    }
  });
  let weak = Rc::downgrade(state);
  state.list.connect_row_activated(move |_, row| {
    if let Some(state) = weak.upgrade() {
      state.copy_row(row);
    }
  });
  let weak = Rc::downgrade(state);
  state.clear.connect_clicked(move |_| {
    if let Some(state) = weak.upgrade().filter(|state| !state.busy.get()) {
      state.show_confirmation();
    }
  });
  let weak = Rc::downgrade(state);
  state.cancel.connect_clicked(move |_| {
    if let Some(state) = weak.upgrade().filter(|state| !state.busy.get()) {
      state.dismiss_confirmation();
    }
  });
  let weak = Rc::downgrade(state);
  wipe.connect_clicked(move |_| {
    if let Some(state) = weak.upgrade() {
      state.wipe();
    }
  });
  let weak = Rc::downgrade(state);
  window.connect_close_request(move |_| {
    if let Some(state) = weak.upgrade().filter(|state| !state.busy.get()) {
      state.running.quit();
    }
    glib::Propagation::Stop
  });
  let search_focus = gtk::EventControllerFocus::new();
  state.search.add_controller(search_focus.clone());
  let list_focus = gtk::EventControllerFocus::new();
  state.list.add_controller(list_focus.clone());
  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  let weak = Rc::downgrade(state);
  keys.connect_key_pressed(move |_, key, _, modifiers| {
    let Some(state) = weak.upgrade() else {
      return glib::Propagation::Proceed;
    };
    if state.busy.get() {
      return glib::Propagation::Stop;
    }
    if state.stack.visible_child_name().as_deref() == Some("confirm") {
      if key == gdk::Key::Escape {
        state.dismiss_confirmation();
        return glib::Propagation::Stop;
      }
      return glib::Propagation::Proceed;
    }
    match key {
      gdk::Key::Escape => state.running.quit(),
      gdk::Key::slash
        if !search_focus.contains_focus()
          && !modifiers.intersects(
            gdk::ModifierType::CONTROL_MASK
              | gdk::ModifierType::ALT_MASK
              | gdk::ModifierType::SUPER_MASK,
          ) =>
      {
        state.search.grab_focus();
      }
      gdk::Key::Up => state.move_selection(-1),
      gdk::Key::Down => state.move_selection(1),
      gdk::Key::Delete if list_focus.contains_focus() => state.show_confirmation(),
      gdk::Key::Return | gdk::Key::KP_Enter
        if search_focus.contains_focus() || list_focus.contains_focus() =>
      {
        if let Some(row) = state.list.selected_row() {
          state.copy_row(&row);
        }
      }
      _ => return glib::Propagation::Proceed,
    }
    glib::Propagation::Stop
  });
  window.add_controller(keys);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn records_keep_original_bytes_and_distinguish_identical_descriptions() {
    let one = Record::parse(b"21\tDuplicate").unwrap();
    let two = Record::parse(b"20\tDuplicate").unwrap();
    assert_eq!(one.description, two.description);
    assert_ne!(one.line, two.line);
    assert!(Record::parse(b"invalid\tpreview").is_none());
  }

  #[test]
  fn understands_link_and_binary_previews() {
    assert_eq!(
      Record::parse(b"4\thttps://example.org").unwrap().kind,
      Kind::Link
    );
    assert_eq!(
      Record::parse(b"3\t[[ binary data 69 B png 5x6 ]]")
        .unwrap()
        .kind,
      Kind::Image
    );
    assert_eq!(
      Record::parse(b"2\t[[ binary data 15 B ]]").unwrap().kind,
      Kind::Binary
    );
    assert_eq!(
      Record::parse("1\tAção".as_bytes()).unwrap().description,
      "Ação"
    );
  }
}
