//! ARGVUS calculator application. Computation stays in the pure engine module.
mod engine;
mod history;
mod preferences;

use engine::{Angle, evaluate, format_result};
use gtk::prelude::*;
use gtk::{gdk, glib};
use relm4::gtk;
use std::cell::RefCell;
use std::rc::Rc;

fn text(key: &str) -> &'static str {
  argvus_i18n::tr(
    argvus_i18n::Lang::for_domain(&argvus_i18n::user_locale(), "calculator"),
    key,
  )
}

struct Calculator {
  entry: gtk::Entry,
  history: gtk::Box,
  angle: gtk::CheckButton,
  pages: gtk::Stack,
  records: RefCell<Vec<history::Record>>,
  history_path: Option<std::path::PathBuf>,
}
impl Calculator {
  fn calculate(&self) {
    let expression = self.entry.text();
    let angle = if self.angle.is_active() {
      Angle::Radians
    } else {
      Angle::Degrees
    };
    match evaluate(&expression, angle) {
      Ok(value) => {
        let formatted = format_result(value);
        self.entry.set_text(&formatted);
        self.entry.grab_focus_without_selecting();
        self.entry.select_region(-1, -1);
        self.entry.set_position(-1);
        self.add_history(&expression, &formatted);
        let mut records = self.records.borrow_mut();
        records.push(history::Record {
          expression: expression.to_string(),
          result: formatted,
        });
        if records.len() > 100 {
          records.remove(0);
        }
        if let Some(path) = &self.history_path
          && let Err(error) = history::save(path, &records)
        {
          self.history_error(error);
        }
      }
      Err(error) => {
        self.entry.set_icon_from_icon_name(
          gtk::EntryIconPosition::Secondary,
          Some("dialog-error-symbolic"),
        );
        self.entry.set_icon_tooltip_text(
          gtk::EntryIconPosition::Secondary,
          Some(text(match error {
            engine::Error::Syntax => "error.syntax",
            engine::Error::Domain => "error.domain",
            engine::Error::Limit => "error.limit",
          })),
        );
      }
    }
  }
  fn history_error(&self, error: impl std::fmt::Display) {
    eprintln!("calculator history: {error}");
    self.entry.set_icon_from_icon_name(
      gtk::EntryIconPosition::Secondary,
      Some("dialog-error-symbolic"),
    );
    self.entry.set_icon_tooltip_text(
      gtk::EntryIconPosition::Secondary,
      Some(text("error.history")),
    );
  }
  fn add_history(&self, expression: &str, result: &str) {
    let button = gtk::Button::with_label(&format!("{expression} = {result}"));
    let entry = self.entry.clone();
    let pages = self.pages.clone();
    let result = result.to_owned();
    button.connect_clicked(move |_| {
      pages.set_visible_child_name("calculator");
      entry.set_text(&result);
      entry.grab_focus_without_selecting();
      entry.select_region(-1, -1);
      entry.set_position(-1);
    });
    self.history.prepend(&button);
    let mut child = self.history.first_child();
    let mut count = 0;
    while let Some(widget) = child {
      child = widget.next_sibling();
      count += 1;
      if count > 100 {
        self.history.remove(&widget);
      }
    }
  }
  fn insert(&self, value: &str) {
    if let Some((start, end)) = self.entry.selection_bounds() {
      self.entry.delete_text(start, end);
      self.entry.set_position(start);
    }
    let mut position = self.entry.position();
    self.entry.insert_text(value, &mut position);
    self.entry.set_position(position);
    self.entry.grab_focus_without_selecting();
  }
}
fn build(app: &gtk::Application) {
  build_with_history(app, history::path().ok(), preferences::path().ok());
}

fn build_with_history(
  app: &gtk::Application,
  history_path: Option<std::path::PathBuf>,
  preferences_path: Option<std::path::PathBuf>,
) {
  if argvus_i18n::Lang::for_domain(&argvus_i18n::user_locale(), "calculator").direction()
    == argvus_i18n::TextDirection::RightToLeft
  {
    gtk::Widget::set_default_direction(gtk::TextDirection::Rtl);
  }
  let style = argvus_time_ui::theme::ThemeStyle::install(include_str!("style.css"));
  let window = gtk::ApplicationWindow::builder()
    .application(app)
    .title(text("app.name"))
    .resizable(true)
    .default_width(370)
    .default_height(420)
    .build();
  window.set_size_request(370, 420);
  window.add_css_class("argvus-time");
  window.add_css_class("calculator");
  style.watch(&window);
  let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
  content.set_margin_top(12);
  content.set_margin_bottom(12);
  content.set_margin_start(16);
  content.set_margin_end(16);
  let pages = gtk::Stack::builder()
    .vexpand(true)
    .hhomogeneous(false)
    .vhomogeneous(false)
    .build();
  let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
  root.append(&pages);
  let settings_hint = gtk::Button::with_label(&format!("s {}", text("settings")));
  settings_hint.add_css_class("footer-hint");
  settings_hint.set_halign(gtk::Align::Start);
  let navigation = pages.clone();
  settings_hint.connect_clicked(move |_| navigation.set_visible_child_name("menu"));
  root.append(&settings_hint);
  pages.add_named(&content, Some("calculator"));
  let settings = gtk::Box::new(gtk::Orientation::Vertical, 8);
  let history_page = gtk::Box::new(gtk::Orientation::Vertical, 8);
  for page in [&settings, &history_page] {
    page.set_margin_top(12);
    page.set_margin_bottom(12);
    page.set_margin_start(16);
    page.set_margin_end(16);
    let back = gtk::Button::from_icon_name("go-previous-symbolic");
    back.set_halign(gtk::Align::Start);
    back.set_tooltip_text(Some(text("action.back")));
    back.update_property(&[gtk::accessible::Property::Label(text("action.back"))]);
    let navigation = pages.clone();
    // Back always returns to calculation without changing the expression.
    back.connect_clicked(move |_| navigation.set_visible_child_name("calculator"));
    page.append(&back);
  }
  pages.add_named(&settings, Some("menu"));
  pages.add_named(&history_page, Some("history"));
  let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  let scientific = gtk::CheckButton::with_label(text("mode.scientific"));
  let angle = gtk::CheckButton::with_label(text("angle.radians"));
  angle.set_tooltip_text(Some(text("angle.hint")));
  controls.append(&scientific);
  controls.append(&angle);
  angle.set_visible(false);
  let entry = gtk::Entry::builder().hexpand(true).max_length(4096).build();
  entry.set_direction(gtk::TextDirection::Ltr);
  entry.set_placeholder_text(Some(text("expression")));
  entry.update_property(&[gtk::accessible::Property::Label(text("expression"))]);
  entry.connect_changed(|entry| {
    entry.set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, None);
    entry.set_icon_tooltip_text(gtk::EntryIconPosition::Secondary, None);
  });
  let history = gtk::Box::new(gtk::Orientation::Vertical, 4);
  let calculator = Rc::new(Calculator {
    entry: entry.clone(),
    history: history.clone(),
    angle: angle.clone(),
    pages: pages.clone(),
    records: RefCell::new(Vec::new()),
    history_path: history_path.clone(),
  });
  match history_path
    .as_deref()
    .map(history::load)
    .unwrap_or_else(|| Ok(Vec::new()))
  {
    Ok(records) => {
      for record in &records {
        calculator.add_history(&record.expression, &record.result);
      }
      *calculator.records.borrow_mut() = records;
    }
    Err(error) => calculator.history_error(error),
  }
  let state = calculator.clone();
  entry.connect_activate(move |_| state.calculate());
  controls.set_visible(false);
  settings.append(&controls);
  content.append(&entry);
  let advanced = gtk::Grid::builder()
    .column_spacing(6)
    .row_spacing(6)
    .column_homogeneous(true)
    .build();
  for (index, (label, value)) in [
    ("sin", "sin("),
    ("cos", "cos("),
    ("tan", "tan("),
    ("π", "pi"),
    ("asin", "asin("),
    ("acos", "acos("),
    ("atan", "atan("),
    ("e", "e"),
    ("√", "sqrt("),
    ("log", "log("),
    ("ln", "ln("),
    ("exp", "exp("),
    ("x²", "^2"),
    ("xʸ", "^"),
    ("1/x", "1/("),
    ("(", "("),
    (")", ")"),
  ]
  .iter()
  .enumerate()
  {
    let button = gtk::Button::with_label(label);
    let state = calculator.clone();
    let value = *value;
    button.connect_clicked(move |_| {
      if value == "1/(" {
        let expression = state.entry.text();
        state.entry.set_text(&format!("1/({expression})"));
        state.entry.set_position(-1);
        state.entry.grab_focus_without_selecting();
      } else {
        state.insert(value);
      }
    });
    advanced.attach(&button, (index % 4) as i32, (index / 4) as i32, 1, 1);
  }
  advanced.set_visible(false);
  content.append(&advanced);
  let expanded = advanced.clone();
  let window_weak = window.downgrade();
  scientific.connect_toggled(move |button| {
    expanded.set_visible(button.is_active());
    angle.set_visible(button.is_active());
    if let Some(window) = window_weak.upgrade() {
      let size = if button.is_active() {
        (440, 640)
      } else {
        (370, 420)
      };
      // Wait for GTK to discard the previous mode's minimum allocation before
      // requesting the smaller surface. A default-size change in this signal
      // alone is clamped to the scientific grid's previous minimum height.
      window.set_size_request(size.0, size.1);
      window.set_resizable(false);
      let weak = window.downgrade();
      let frames = std::cell::Cell::new(0);
      window.add_tick_callback(move |_, _| {
        frames.set(frames.get() + 1);
        if frames.get() < 2 {
          return glib::ControlFlow::Continue;
        }
        let Some(window) = weak.upgrade() else {
          return glib::ControlFlow::Break;
        };
        if frames.get() == 2 {
          window.set_default_size(size.0, size.1);
        }
        if frames.get() >= 4 && ((window.width(), window.height()) == size || frames.get() >= 30) {
          window.set_resizable(true);
          glib::ControlFlow::Break
        } else {
          glib::ControlFlow::Continue
        }
      });
    }
  });
  let grid = gtk::Grid::builder()
    .vexpand(true)
    .column_spacing(6)
    .row_spacing(6)
    .column_homogeneous(true)
    .row_homogeneous(true)
    .build();
  for (index, label) in [
    "C", "⌫", "%", "/", "7", "8", "9", "*", "4", "5", "6", "-", "1", "2", "3", "+", "±", "0", ".",
    "=",
  ]
  .iter()
  .enumerate()
  {
    let button = gtk::Button::with_label(label);
    let state = calculator.clone();
    let label = *label;
    if label.bytes().all(|byte| byte.is_ascii_digit()) {
      button.add_css_class("number-key");
    }
    if label == "=" {
      button.add_css_class("primary");
    }
    if label == "C" {
      button.set_tooltip_text(Some(text("action.clear")));
    }
    if label == "⌫" {
      button.set_tooltip_text(Some(text("action.backspace")));
    }
    button.connect_clicked(move |_| match label {
      "C" => {
        state.entry.set_text("");
        state.entry.grab_focus_without_selecting();
      }
      "⌫" => {
        if let Some((start, end)) = state.entry.selection_bounds() {
          state.entry.delete_text(start, end);
        } else {
          let end = state.entry.position();
          if end > 0 {
            state.entry.delete_text(end - 1, end);
          }
        }
        state.entry.grab_focus_without_selecting();
      }
      "±" => {
        let expression = state.entry.text();
        state.entry.set_text(&format!("-({expression})"));
        state.entry.set_position(-1);
        state.entry.grab_focus_without_selecting();
      }
      "=" => state.calculate(),
      _ => state.insert(label),
    });
    grid.attach(&button, (index % 4) as i32, (index / 4) as i32, 1, 1);
  }
  content.append(&grid);
  let options = gtk::ListBox::new();
  options.add_css_class("settings-list");
  options.set_selection_mode(gtk::SelectionMode::Single);
  options.set_activate_on_single_click(true);
  let mut option_labels = Vec::new();
  for key in ["mode.scientific", "angle.radians", "action.copy", "history"] {
    let label = gtk::Label::new(Some(text(key)));
    label.set_xalign(0.0);
    options.append(&label);
    option_labels.push(label);
  }
  let scientific_label = option_labels[0].clone();
  scientific_label.set_text(&format!("[ ] {}", text("mode.scientific")));
  scientific.connect_toggled(move |control| {
    scientific_label.set_text(&format!(
      "[{}] {}",
      if control.is_active() { "x" } else { " " },
      text("mode.scientific")
    ))
  });
  let angle_label = option_labels[1].clone();
  angle_label.set_text(&format!("[ ] {}", text("angle.radians")));
  let angle_row = options.row_at_index(1).unwrap();
  angle_row.set_visible(false);
  scientific.connect_toggled(move |control| angle_row.set_visible(control.is_active()));
  let angle_state = calculator.angle.clone();
  angle_state.connect_toggled(move |control| {
    angle_label.set_text(&format!(
      "[{}] {}",
      if control.is_active() { "x" } else { " " },
      text("angle.radians")
    ))
  });
  let state = calculator.clone();
  let scientific_state = scientific.clone();
  options.connect_row_activated(move |_, row| match row.index() {
    0 => scientific_state.set_active(!scientific_state.is_active()),
    1 => state.angle.set_active(!state.angle.is_active()),
    2 => state.entry.clipboard().set_text(&state.entry.text()),
    3 => state.pages.set_visible_child_name("history"),
    _ => {}
  });
  let list = options.clone();
  pages.connect_visible_child_name_notify(move |pages| {
    if pages.visible_child_name().as_deref() == Some("menu")
      && let Some(row) = list.row_at_index(0)
    {
      list.select_row(Some(&row));
      row.grab_focus();
    }
  });
  settings.append(&options);
  history_page.append(&gtk::Label::new(Some(text("history"))));
  let scroll = gtk::ScrolledWindow::builder()
    .min_content_height(70)
    .vexpand(true)
    .child(&history)
    .build();
  history_page.append(&scroll);
  let clear_history = gtk::Button::with_label(text("action.clear_history"));
  let history_state = calculator.clone();
  clear_history.connect_clicked(move |_| {
    if let Some(path) = &history_state.history_path
      && let Err(error) = history::save(path, &[])
    {
      history_state.history_error(error);
      return;
    }
    history_state.records.borrow_mut().clear();
    while let Some(child) = history.first_child() {
      history.remove(&child);
    }
  });
  history_page.append(&clear_history);
  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  let state = calculator;
  keys.connect_key_pressed(move |_, key, _, modifiers| {
    if key == gdk::Key::s
      && !modifiers.intersects(
        gdk::ModifierType::CONTROL_MASK
          | gdk::ModifierType::ALT_MASK
          | gdk::ModifierType::SUPER_MASK,
      )
      && state.pages.visible_child_name().as_deref() == Some("calculator")
    {
      state.pages.set_visible_child_name("menu");
      return glib::Propagation::Stop;
    }
    if key == gdk::Key::Escape {
      if state.pages.visible_child_name().as_deref() != Some("calculator") {
        state.pages.set_visible_child_name("calculator");
        state.entry.grab_focus_without_selecting();
        return glib::Propagation::Stop;
      }
      state.entry.set_text("");
      state.entry.grab_focus_without_selecting();
      glib::Propagation::Stop
    } else {
      glib::Propagation::Proceed
    }
  });
  window.add_controller(keys);
  window.set_child(Some(&root));
  match preferences_path
    .as_deref()
    .map(preferences::load)
    .unwrap_or_else(|| Ok(preferences::Preferences::default()))
  {
    Ok(saved) => scientific.set_active(saved.scientific),
    Err(error) => {
      eprintln!("calculator preferences: {error}");
      entry.set_icon_from_icon_name(
        gtk::EntryIconPosition::Secondary,
        Some("dialog-error-symbolic"),
      );
      entry.set_icon_tooltip_text(
        gtk::EntryIconPosition::Secondary,
        Some(text("error.settings")),
      );
    }
  }
  let display = entry.clone();
  scientific.connect_toggled(move |control| {
    if let Some(path) = &preferences_path
      && let Err(error) = history::save_json(
        path,
        &preferences::Preferences {
          scientific: control.is_active(),
        },
      )
    {
      eprintln!("calculator preferences: {error}");
      display.set_icon_from_icon_name(
        gtk::EntryIconPosition::Secondary,
        Some("dialog-error-symbolic"),
      );
      display.set_icon_tooltip_text(
        gtk::EntryIconPosition::Secondary,
        Some(text("error.settings")),
      );
    }
  });
  window.present();
  entry.grab_focus_without_selecting();
}
fn main() -> glib::ExitCode {
  let app = gtk::Application::builder()
    .application_id("org.argvus.Calculator")
    .build();
  // GTK owns the single application instance and presents it on repeated launch.
  let retained = Rc::new(RefCell::new(None::<gtk::Window>));
  app.connect_activate(move |app| {
    if let Some(window) = retained.borrow().as_ref() {
      window.present();
      return;
    }
    build(app);
    *retained.borrow_mut() = app.active_window();
  });
  app.run()
}

#[cfg(test)]
mod ui_tests {
  use super::*;

  fn find<T: IsA<gtk::Widget> + glib::types::StaticType>(widget: &gtk::Widget) -> Option<T> {
    if let Ok(found) = widget.clone().downcast::<T>() {
      return Some(found);
    }
    let mut child = widget.first_child();
    while let Some(widget) = child {
      if let Some(found) = find::<T>(&widget) {
        return Some(found);
      }
      child = widget.next_sibling();
    }
    None
  }
  fn settle() {
    let context = glib::MainContext::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(250);
    while std::time::Instant::now() < deadline {
      while context.pending() {
        context.iteration(false);
      }
      std::thread::sleep(std::time::Duration::from_millis(5));
    }
  }
  #[test]
  #[ignore = "requires GTK display and calculator catalogs"]
  fn mode_round_trip_shrinks_and_history_has_its_own_page() {
    gtk::init().unwrap();
    let app = gtk::Application::builder()
      .application_id("org.argvus.CalculatorTest")
      .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    build_with_history(
      &app,
      Some(temporary.path().join("history.json")),
      Some(temporary.path().join("settings.json")),
    );
    let window = app.active_window().unwrap();
    let pages = find::<gtk::Stack>(window.upcast_ref()).unwrap();
    assert_eq!(pages.visible_child_name().as_deref(), Some("calculator"));
    assert!(!pages.is_vhomogeneous());
    let footer = window
      .child()
      .unwrap()
      .last_child()
      .unwrap()
      .downcast::<gtk::Button>()
      .unwrap();
    assert_eq!(
      footer.label().as_deref(),
      Some(format!("s {}", text("settings")).as_str())
    );
    footer.emit_clicked();
    assert_eq!(pages.visible_child_name().as_deref(), Some("menu"));
    pages.set_visible_child_name("calculator");
    let main = pages.child_by_name("calculator").unwrap();
    assert!(find::<gtk::ScrolledWindow>(&main).is_none());
    let menu = pages.child_by_name("menu").unwrap();
    let scientific = find::<gtk::CheckButton>(&menu).unwrap();
    let entry = find::<gtk::Entry>(&main).unwrap();
    entry.set_text("2+3");
    pages.set_visible_child_name("menu");
    let options = find::<gtk::ListBox>(&menu).unwrap();
    options.emit_by_name::<()>("row-activated", &[&options.row_at_index(0).unwrap()]);
    assert!(scientific.is_active());
    settle();
    pages.set_visible_child_name("calculator");
    settle();
    assert_eq!(window.default_size(), (440, 640));
    assert_eq!(window.width(), 440);
    pages.set_visible_child_name("menu");
    options.emit_by_name::<()>("row-activated", &[&options.row_at_index(0).unwrap()]);
    assert!(!scientific.is_active());
    settle();
    pages.set_visible_child_name("calculator");
    settle();
    assert_eq!(window.default_size(), (370, 420));
    assert_eq!((window.width(), window.height()), (370, 420));
    assert!(window.is_resizable());
    let keypad = main.last_child().unwrap().downcast::<gtk::Grid>().unwrap();
    let key = keypad.first_child().unwrap();
    let original_height = key.height();
    window.set_size_request(520, 680);
    settle();
    assert!(key.height() > original_height);
    window.set_size_request(370, 420);
    window.set_default_size(200, 200);
    settle();
    assert!(window.width() >= 370 && window.height() >= 420);
    assert_eq!(window.size_request(), (370, 420));
    let back = menu
      .first_child()
      .unwrap()
      .downcast::<gtk::Button>()
      .unwrap();
    assert_eq!(back.icon_name().as_deref(), Some("go-previous-symbolic"));
    assert!(back.label().is_none());
    assert_eq!(entry.text(), "2+3");
    entry.emit_activate();
    assert_eq!(entry.text(), "5");
    entry.set_text("2+2");
    entry.emit_activate();
    assert_eq!(entry.text(), "4");
    assert!(entry.selection_bounds().is_none());
    assert_eq!(entry.position(), 1);
    // Append at the actual caret, just like continuing from the keyboard.
    let mut position = entry.position();
    entry.insert_text("+2", &mut position);
    entry.set_position(position);
    entry.emit_activate();
    assert_eq!(entry.text(), "6");
    assert!(entry.selection_bounds().is_none());
    assert_eq!(entry.position(), 1);
    entry.set_text("4*3");
    entry.emit_activate();
    assert_eq!(entry.text(), "12");
    entry.set_text("1/0");
    entry.emit_activate();
    assert_eq!(entry.text(), "1/0");
    assert!(entry.icon_name(gtk::EntryIconPosition::Secondary).is_some());
    entry.set_text("2+3");
    assert!(entry.icon_name(gtk::EntryIconPosition::Secondary).is_none());
    entry.emit_activate();
    pages.set_visible_child_name("history");
    let history = pages.child_by_name("history").unwrap();
    assert!(find::<gtk::ScrolledWindow>(&history).is_some());
    let scroll = find::<gtk::ScrolledWindow>(&history).unwrap();
    let button = find::<gtk::Button>(&scroll.upcast()).unwrap();
    button.emit_clicked();
    assert_eq!(pages.visible_child_name().as_deref(), Some("calculator"));
    assert_eq!(entry.text(), "5");
    assert!(entry.selection_bounds().is_none());
    assert_eq!(entry.position(), 1);
    assert!(
      !history::load(&temporary.path().join("history.json"))
        .unwrap()
        .is_empty()
    );
    scientific.set_active(true);
    window.close();
    settle();
    build_with_history(
      &app,
      Some(temporary.path().join("history.json")),
      Some(temporary.path().join("settings.json")),
    );
    settle();
    let reopened = app.active_window().unwrap();
    let restored_pages = find::<gtk::Stack>(reopened.upcast_ref()).unwrap();
    let restored_menu = restored_pages.child_by_name("menu").unwrap();
    assert!(
      find::<gtk::CheckButton>(&restored_menu)
        .unwrap()
        .is_active()
    );
    assert_eq!(reopened.default_size(), (440, 640));
    reopened.close();
  }
}
