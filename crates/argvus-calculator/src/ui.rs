//! GTK profile views, keyboard-first settings and persisted application state.
mod calculation;
mod records;
mod settings;
use crate::{
  conversion,
  engine::{self, Angle, Error, format_result},
  financial, history,
  preferences::{self, Preferences, Profile},
  programming, text,
};
use gtk::{gdk, glib, prelude::*};
use relm4::gtk;
use std::{
  cell::{Cell, RefCell},
  path::PathBuf,
  rc::Rc,
};

#[derive(Clone, Copy)]
enum Action {
  Profile,
  Radians,
  Base,
  Bits,
  Signed,
  Financial,
  Category,
  From,
  To,
  Copy,
  History,
}

pub struct Calculator {
  pub window: gtk::ApplicationWindow,
  entry: gtk::Entry,
  pages: gtk::Stack,
  summary: gtk::Label,
  scientific: gtk::Grid,
  programming: gtk::Grid,
  keypad: gtk::Grid,
  parameter_box: gtk::Box,
  parameters: [gtk::Entry; 3],
  parameter_labels: [gtk::Label; 3],
  settings: gtk::ListBox,
  profiles: gtk::ListBox,
  history_box: gtk::Box,
  preferences: RefCell<Preferences>,
  drafts: RefCell<[String; 5]>,
  records: RefCell<Vec<history::Record>>,
  actions: RefCell<Vec<Action>>,
  history_path: Option<PathBuf>,
  preferences_path: Option<PathBuf>,
  conversion_result: Cell<bool>,
  resize_generation: Cell<u64>,
  load_error: RefCell<Option<&'static str>>,
}
impl Calculator {
  pub fn new(
    application: &gtk::Application,
    history_path: Option<PathBuf>,
    preferences_path: Option<PathBuf>,
  ) -> Rc<Self> {
    let saved = preferences_path
      .as_deref()
      .map(preferences::load)
      .unwrap_or_else(|| Ok(Preferences::default()));
    let (preferences, settings_error) = match saved {
      Ok(saved) => (saved, None),
      Err(error) => {
        eprintln!("calculator preferences: {error}");
        (Preferences::default(), Some("error.settings"))
      }
    };
    if argvus_i18n::Lang::for_domain(&argvus_i18n::user_locale(), "calculator").direction()
      == argvus_i18n::TextDirection::RightToLeft
    {
      gtk::Widget::set_default_direction(gtk::TextDirection::Rtl);
    }
    let size = preferences.profile.size();
    let window = gtk::ApplicationWindow::builder()
      .application(application)
      .default_width(size.0)
      .default_height(size.1)
      .resizable(true)
      .build();
    window.add_css_class("argvus-time");
    window.add_css_class("calculator");
    window.set_size_request(size.0, size.1);
    let style = argvus_time_ui::theme::ThemeStyle::install(include_str!("style.css"));
    style.watch(&window);
    let pages = gtk::Stack::builder()
      .hhomogeneous(false)
      .vhomogeneous(false)
      .vexpand(true)
      .build();
    let content = page_box();
    pages.add_named(&content, Some("calculator"));
    let summary = gtk::Label::new(None);
    summary.set_wrap(true);
    summary.set_xalign(0.0);
    summary.add_css_class("caption");
    content.append(&summary);
    let entry = gtk::Entry::builder().hexpand(true).max_length(4096).build();
    entry.set_direction(gtk::TextDirection::Ltr);
    entry.set_placeholder_text(Some(text("expression")));
    entry.update_property(&[gtk::accessible::Property::Label(text("expression"))]);
    content.append(&entry);
    let parameter_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
    parameter_box.add_css_class("financial-fields");
    let parameters = std::array::from_fn(|_| gtk::Entry::builder().max_length(4096).build());
    let parameter_labels = std::array::from_fn(|_| gtk::Label::new(None));
    for (label, field) in parameter_labels.iter().zip(&parameters) {
      label.set_xalign(0.0);
      parameter_box.append(label);
      field.set_direction(gtk::TextDirection::Ltr);
      parameter_box.append(field);
    }
    parameters[0].set_text("0");
    parameters[1].set_text("12");
    parameters[2].set_text("0");
    content.append(&parameter_box);
    let scientific = grid(false);
    let programming = grid(false);
    let keypad = grid(true);
    content.append(&scientific);
    content.append(&programming);
    content.append(&keypad);
    let settings_page = page_box();
    let settings = gtk::ListBox::new();
    configure_list(&settings);
    let profile_page = page_box();
    let profiles = gtk::ListBox::new();
    configure_list(&profiles);
    let history_page = page_box();
    let history_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    pages.add_named(&settings_page, Some("menu"));
    pages.add_named(&profile_page, Some("profiles"));
    pages.add_named(&history_page, Some("history"));
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    header.add_css_class("calculator-header");
    let brand = gtk::Label::new(Some("ARGVUS"));
    brand.add_css_class("calculator-brand");
    brand.set_valign(gtk::Align::Center);
    let title = text("app.name").replace("ARGVUS", "");
    let heading = gtk::Label::new(Some(title.trim()));
    heading.add_css_class("calculator-heading");
    heading.set_xalign(0.0);
    header.append(&brand);
    header.append(&heading);
    root.append(&header);
    root.append(&pages);
    let footer = gtk::Button::with_label(&format!("s {}", text("settings")));
    footer.set_halign(gtk::Align::Start);
    footer.add_css_class("footer-hint");
    let footer_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    footer_bar.add_css_class("calculator-footer");
    let hint = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let shortcut = gtk::Label::new(Some("s"));
    shortcut.add_css_class("shortcut-key");
    hint.append(&shortcut);
    hint.append(&gtk::Label::new(Some(text("settings"))));
    footer.set_child(Some(&hint));
    footer_bar.append(&footer);
    root.append(&footer_bar);
    window.set_child(Some(&root));
    let calculator = Rc::new(Self {
      window,
      entry,
      pages,
      summary,
      scientific,
      programming,
      keypad,
      parameter_box,
      parameters,
      parameter_labels,
      settings,
      profiles,
      history_box,
      preferences: RefCell::new(preferences),
      drafts: RefCell::new(std::array::from_fn(|_| String::new())),
      records: RefCell::new(Vec::new()),
      actions: RefCell::new(Vec::new()),
      history_path,
      preferences_path,
      conversion_result: Cell::new(false),
      resize_generation: Cell::new(0),
      load_error: RefCell::new(settings_error),
    });
    for (page, target) in [
      (&settings_page, "calculator"),
      (&profile_page, "menu"),
      (&history_page, "menu"),
    ] {
      let back = gtk::Button::from_icon_name("go-previous-symbolic");
      back.set_halign(gtk::Align::Start);
      back.set_tooltip_text(Some(text("action.back")));
      back.update_property(&[gtk::accessible::Property::Label(text("action.back"))]);
      let weak = Rc::downgrade(&calculator);
      back.connect_clicked(move |_| {
        if let Some(state) = weak.upgrade() {
          state.navigate(target);
        }
      });
      page.append(&back);
    }
    settings_page.append(&calculator.settings);
    profile_page.append(&calculator.profiles);
    history_page.append(&gtk::Label::new(Some(text("history"))));
    let scroll = gtk::ScrolledWindow::builder()
      .min_content_height(70)
      .vexpand(true)
      .child(&calculator.history_box)
      .build();
    history_page.append(&scroll);
    let clear = gtk::Button::with_label(text("action.clear_history"));
    history_page.append(&clear);
    let weak = Rc::downgrade(&calculator);
    clear.connect_clicked(move |_| {
      if let Some(state) = weak.upgrade() {
        state.clear_history();
      }
    });
    let weak = Rc::downgrade(&calculator);
    footer.connect_clicked(move |_| {
      if let Some(state) = weak.upgrade() {
        state.navigate("menu");
      }
    });
    calculator.build_keypads();
    calculator.connect_signals();
    calculator.refresh(false);
    calculator.load_history();
    if let Some(key) = calculator.load_error.borrow().as_ref() {
      calculator.error(key);
    }
    calculator.entry.grab_focus_without_selecting();
    calculator
  }
  fn connect_signals(self: &Rc<Self>) {
    let weak = Rc::downgrade(self);
    self.entry.connect_activate(move |_| {
      if let Some(state) = weak.upgrade() {
        state.calculate();
      }
    });
    let weak = Rc::downgrade(self);
    self.entry.connect_changed(move |entry| {
      entry.set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, None);
      entry.set_icon_tooltip_text(gtk::EntryIconPosition::Secondary, None);
      if let Some(state) = weak.upgrade() {
        state.conversion_result.set(false);
      }
    });
    for field in &self.parameters {
      let weak = Rc::downgrade(self);
      field.connect_activate(move |_| {
        if let Some(state) = weak.upgrade() {
          state.calculate();
        }
      });
    }
    let weak = Rc::downgrade(self);
    self.settings.connect_row_activated(move |_, row| {
      if let Some(state) = weak.upgrade() {
        let action = state
          .actions
          .borrow()
          .get(usize::try_from(row.index()).unwrap_or(usize::MAX))
          .copied();
        if let Some(action) = action {
          state.activate(action);
        }
      }
    });
    let weak = Rc::downgrade(self);
    self.profiles.connect_row_activated(move |_, row| {
      if let Some(state) = weak.upgrade()
        && let Ok(index) = usize::try_from(row.index())
        && let Some(profile) = Profile::ALL.get(index)
      {
        let mut preferences = state.preferences.borrow().clone();
        preferences.profile = *profile;
        state.apply_preferences(preferences);
        state.navigate("calculator");
      }
    });
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = Rc::downgrade(self);
    keys.connect_key_pressed(move |_, key, _, modifiers| {
      let Some(state) = weak.upgrade() else {
        return glib::Propagation::Proceed;
      };
      if key == gdk::Key::Escape {
        let current = state.pages.visible_child_name();
        match current.as_deref() {
          Some("profiles") | Some("history") => state.navigate("menu"),
          Some("menu") => state.navigate("calculator"),
          _ => state.entry.set_text(""),
        };
        return glib::Propagation::Stop;
      }
      if key == gdk::Key::s
        && !modifiers.intersects(
          gdk::ModifierType::CONTROL_MASK
            | gdk::ModifierType::ALT_MASK
            | gdk::ModifierType::SUPER_MASK,
        )
        && state.pages.visible_child_name().as_deref() == Some("calculator")
      {
        state.navigate("menu");
        return glib::Propagation::Stop;
      }
      glib::Propagation::Proceed
    });
    self.window.add_controller(keys);
  }
  fn build_keypads(self: &Rc<Self>) {
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
    .into_iter()
    .enumerate()
    {
      self.add_key(&self.scientific, index, label, value);
    }
    for (index, label) in [
      "A", "B", "C", "D", "E", "F", "AND", "OR", "XOR", "NOT", "<<", ">>", "(", ")", "%",
    ]
    .into_iter()
    .enumerate()
    {
      let value = if ["AND", "OR", "XOR", "NOT"].contains(&label) {
        format!(" {label} ")
      } else {
        label.into()
      };
      self.add_key(&self.programming, index, label, &value);
    }
    for (index, label) in [
      "C", "⌫", "%", "/", "7", "8", "9", "*", "4", "5", "6", "-", "1", "2", "3", "+", "±", "0",
      ".", "=",
    ]
    .into_iter()
    .enumerate()
    {
      self.add_key(&self.keypad, index, label, label);
    }
  }
  fn add_key(self: &Rc<Self>, grid: &gtk::Grid, index: usize, label: &str, value: &str) {
    let display = match label {
      "*" => "×",
      "/" => "÷",
      "-" => "−",
      _ => label,
    };
    let button = gtk::Button::with_label(display);
    button.add_css_class("keypad-key");
    if ["+", "-", "*", "/", "=", "±", "%", "⌫", "√", "π"].contains(&label) {
      button.add_css_class("symbol-key");
    }
    if label.bytes().all(|byte| byte.is_ascii_digit()) {
      button.add_css_class("number-key");
    }
    if label == "=" {
      button.add_css_class("primary");
    }
    if label == "C" && grid == &self.keypad {
      button.set_tooltip_text(Some(text("action.clear")));
    }
    if label == "⌫" {
      button.set_tooltip_text(Some(text("action.backspace")));
    }
    let value = value.to_owned();
    let clear = label == "C" && grid == &self.keypad;
    let weak = Rc::downgrade(self);
    button.connect_clicked(move |_| {
      if let Some(state) = weak.upgrade() {
        if clear {
          state.entry.set_text("");
        } else {
          match value.as_str() {
            "=" => {
              state.calculate();
              return;
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
            }
            "±" => state.entry.set_text(&format!("-({})", state.entry.text())),
            "1/(" => state.entry.set_text(&format!("1/({})", state.entry.text())),
            _ => state.insert(&value),
          }
        }
        state.entry.grab_focus_without_selecting();
      }
    });
    grid.attach(
      &button,
      i32::try_from(index % 4).unwrap_or(0),
      i32::try_from(index / 4).unwrap_or(0),
      1,
      1,
    );
  }
  fn insert(&self, value: &str) {
    if let Some((start, end)) = self.entry.selection_bounds() {
      self.entry.delete_text(start, end);
      self.entry.set_position(start);
    }
    let mut position = self.entry.position();
    self.entry.insert_text(value, &mut position);
    self.entry.set_position(position);
  }
  fn navigate(&self, page: &str) {
    self.pages.set_visible_child_name(page);
    match page {
      "menu" => focus_first(&self.settings),
      "profiles" => {
        if let Some(row) = self
          .profiles
          .row_at_index(i32::try_from(self.preferences.borrow().profile.index()).unwrap_or(0))
        {
          self.profiles.select_row(Some(&row));
          row.grab_focus();
        }
      }
      "calculator" => {
        self.entry.grab_focus_without_selecting();
      }
      _ => {}
    }
  }
  fn error(&self, key: &str) {
    self.entry.set_icon_from_icon_name(
      gtk::EntryIconPosition::Secondary,
      Some("dialog-error-symbolic"),
    );
    self
      .entry
      .set_icon_tooltip_text(gtk::EntryIconPosition::Secondary, Some(text(key)));
  }
  fn storage_error(&self, key: &str, error: impl std::fmt::Display) {
    eprintln!("calculator: {error}");
    self.error(key);
  }
  fn save_preferences(&self) {
    if let Some(path) = &self.preferences_path
      && let Err(error) = history::save_json(path, &*self.preferences.borrow())
    {
      self.storage_error("error.settings", error);
    }
  }
  fn apply_preferences(self: &Rc<Self>, mut preferences: Preferences) {
    preferences.validate();
    let previous = self.preferences.borrow().profile;
    if previous != preferences.profile {
      self.drafts.borrow_mut()[previous.index()] = self.entry.text().to_string();
      self
        .entry
        .set_text(&self.drafts.borrow()[preferences.profile.index()]);
    }
    *self.preferences.borrow_mut() = preferences;
    self.conversion_result.set(false);
    self.refresh(previous != self.preferences.borrow().profile);
    self.save_preferences();
  }
  fn refresh(self: &Rc<Self>, resize: bool) {
    let preferences = self.preferences.borrow();
    let profile = preferences.profile;
    self.window.set_title(Some(&format!(
      "{} — {}",
      text("app.name"),
      text(profile.key())
    )));
    self.scientific.set_visible(profile == Profile::Scientific);
    self
      .programming
      .set_visible(profile == Profile::Programming);
    self
      .parameter_box
      .set_visible(profile == Profile::Financial);
    let summary = match profile {
      Profile::Basic => String::new(),
      Profile::Scientific => text(if preferences.radians {
        "angle.radians"
      } else {
        "angle.degrees"
      })
      .into(),
      Profile::Programming => format!(
        "{}: {} · {}: {} · {}",
        text("program.base"),
        preferences.programming.base.radix(),
        text("program.bits"),
        preferences.programming.bits,
        text(if preferences.programming.signed {
          "program.signed"
        } else {
          "program.unsigned"
        })
      ),
      Profile::Financial => text(preferences.financial.key()).into(),
      Profile::Conversion => format!(
        "{}: {} → {}",
        text(preferences.category.key()),
        preferences.category.units()[preferences.from].symbol,
        preferences.category.units()[preferences.to].symbol
      ),
    };
    self.summary.set_text(&summary);
    self.summary.set_visible(!summary.is_empty());
    for grid in [&self.keypad, &self.programming] {
      let mut child = grid.first_child();
      while let Some(widget) = child {
        child = widget.next_sibling();
        if let Ok(button) = widget.downcast::<gtk::Button>() {
          if grid == &self.keypad && matches!(button.label().as_deref(), Some("C" | "AC")) {
            button.set_label(if profile == Profile::Programming {
              "AC"
            } else {
              "C"
            });
          }
          let label = button.label().unwrap_or_default();
          let allowed = if profile != Profile::Programming {
            true
          } else if label == "." {
            false
          } else if label == "%" {
            true
          } else if label.len() == 1
            && label.chars().all(|character| character.is_ascii_hexdigit())
            && !(label == "C" && grid == &self.keypad)
          {
            label
              .chars()
              .next()
              .and_then(|character| character.to_digit(16))
              .is_some_and(|digit| digit < preferences.programming.base.radix())
          } else {
            true
          };
          button.set_sensitive(allowed);
        }
      }
    }
    let financial = preferences.financial;
    let label_keys = [
      if financial == financial::Operation::Change {
        "finance.final"
      } else {
        "finance.rate"
      },
      "finance.periods",
      "finance.periodic_payment",
    ];
    for (index, key) in label_keys.into_iter().enumerate() {
      self.parameter_labels[index].set_text(text(key));
      self.parameters[index].update_property(&[gtk::accessible::Property::Label(text(key))]);
      let visible = index == 0
        || (index == 1 && financial != financial::Operation::Change)
        || (index == 2
          && [financial::Operation::Present, financial::Operation::Future].contains(&financial));
      self.parameters[index].set_visible(visible);
      self.parameter_labels[index].set_visible(visible);
    }
    let primary_label = match profile {
      Profile::Financial if financial == financial::Operation::Present => "finance.future_amount",
      Profile::Financial if financial == financial::Operation::Change => "finance.original",
      Profile::Financial => "finance.principal",
      _ => "expression",
    };
    self.entry.set_placeholder_text(Some(text(primary_label)));
    self
      .entry
      .update_property(&[gtk::accessible::Property::Label(text(primary_label))]);
    drop(preferences);
    self.rebuild_settings();
    self.rebuild_profiles();
    if resize {
      self.resize(profile.size());
    }
  }
  fn resize(self: &Rc<Self>, size: (i32, i32)) {
    let generation = self.resize_generation.get().wrapping_add(1);
    self.resize_generation.set(generation);
    self.window.set_size_request(size.0, size.1);
    self.window.set_resizable(false);
    let weak = Rc::downgrade(self);
    let frames = Cell::new(0);
    self.window.add_tick_callback(move |_, _| {
      let Some(state) = weak.upgrade() else {
        return glib::ControlFlow::Break;
      };
      if state.resize_generation.get() != generation {
        return glib::ControlFlow::Break;
      }
      frames.set(frames.get() + 1);
      if frames.get() == 2 {
        state.window.set_default_size(size.0, size.1);
      }
      if frames.get() >= 4
        && ((state.window.width(), state.window.height()) == size || frames.get() >= 30)
      {
        state.window.set_resizable(true);
        glib::ControlFlow::Break
      } else {
        glib::ControlFlow::Continue
      }
    });
  }
}
fn cycle<T: Copy + PartialEq>(value: T, values: &[T]) -> T {
  values[(values
    .iter()
    .position(|candidate| *candidate == value)
    .unwrap_or(0)
    + 1)
    % values.len()]
}
fn grid(expand: bool) -> gtk::Grid {
  gtk::Grid::builder()
    .column_spacing(6)
    .row_spacing(6)
    .column_homogeneous(true)
    .row_homogeneous(true)
    .vexpand(expand)
    .build()
}
fn page_box() -> gtk::Box {
  let page = gtk::Box::new(gtk::Orientation::Vertical, 8);
  page.set_margin_top(12);
  page.set_margin_bottom(12);
  page.set_margin_start(16);
  page.set_margin_end(16);
  page
}
fn configure_list(list: &gtk::ListBox) {
  list.add_css_class("settings-list");
  list.set_selection_mode(gtk::SelectionMode::Single);
  list.set_activate_on_single_click(true);
}
fn append_row(list: &gtk::ListBox, value: &str) {
  let label = gtk::Label::new(Some(value));
  label.set_xalign(0.0);
  label.set_wrap(true);
  list.append(&label);
}
fn clear_list(list: &gtk::ListBox) {
  while let Some(row) = list.first_child() {
    list.remove(&row);
  }
}
fn focus_first(list: &gtk::ListBox) {
  if let Some(row) = list.row_at_index(0) {
    list.select_row(Some(&row));
    row.grab_focus();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn settle() {
    let context = glib::MainContext::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(300);
    while std::time::Instant::now() < deadline {
      while context.pending() {
        context.iteration(false);
      }
      std::thread::sleep(std::time::Duration::from_millis(5));
    }
  }
  fn choose(state: &Rc<Calculator>, profile: Profile) {
    state.navigate("menu");
    state
      .settings
      .emit_by_name::<()>("row-activated", &[&state.settings.row_at_index(0).unwrap()]);
    assert_eq!(
      state.pages.visible_child_name().as_deref(),
      Some("profiles")
    );
    state.profiles.emit_by_name::<()>(
      "row-activated",
      &[&state
        .profiles
        .row_at_index(i32::try_from(profile.index()).unwrap())
        .unwrap()],
    );
    settle();
    assert_eq!(state.preferences.borrow().profile, profile);
    assert_eq!(state.window.size_request(), profile.size());
  }
  #[test]
  #[ignore = "requires a GTK display, calculator catalogs and floating CalculatorTest window rule"]
  fn profiles_history_and_restart() {
    gtk::init().unwrap();
    let application = gtk::Application::builder()
      .application_id("org.argvus.CalculatorTest")
      .build();
    application
      .register(None::<&gtk::gio::Cancellable>)
      .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let history_path = temp.path().join("history.json");
    let settings_path = temp.path().join("settings.json");
    // Old records without context must remain readable and usable.
    std::fs::write(&history_path, r#"[{"expression":"1+1","result":"2"}]"#).unwrap();
    let state = Calculator::new(
      &application,
      Some(history_path.clone()),
      Some(settings_path.clone()),
    );
    state.window.present();
    settle();
    // Applications Font is delivered through GtkSettings; no calculator CSS
    // may pin a family or prevent live updates on an already visible entry.
    let gtk_settings = gtk::Settings::default().unwrap();
    let previous_font = gtk_settings.property::<String>("gtk-font-name");
    for family in ["Serif", "Sans"] {
      gtk_settings.set_property("gtk-font-name", format!("{family} 12"));
      settle();
      let actual = state
        .entry
        .pango_context()
        .font_description()
        .unwrap()
        .family()
        .unwrap()
        .to_string();
      assert_eq!(actual.to_ascii_lowercase(), family.to_ascii_lowercase());
    }
    gtk_settings.set_property("gtk-font-name", previous_font);
    settle();
    let shell = state.window.child().unwrap();
    let header = shell.first_child().unwrap();
    assert!(header.has_css_class("calculator-header"));
    assert_eq!(
      header
        .first_child()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap()
        .text(),
      "ARGVUS"
    );
    let footer = shell.last_child().unwrap();
    assert!(footer.has_css_class("calculator-footer"));
    footer
      .first_child()
      .unwrap()
      .downcast::<gtk::Button>()
      .unwrap()
      .emit_clicked();
    assert_eq!(state.pages.visible_child_name().as_deref(), Some("menu"));
    state.navigate("calculator");
    assert_eq!(state.records.borrow().len(), 1);
    state
      .history_box
      .first_child()
      .unwrap()
      .downcast::<gtk::Button>()
      .unwrap()
      .emit_clicked();
    assert_eq!(state.entry.text(), "2");
    assert_eq!(state.preferences.borrow().profile, Profile::Basic);
    assert_eq!(
      state.pages.visible_child_name().as_deref(),
      Some("calculator")
    );
    state.entry.set_text("2+2");
    state.entry.emit_activate();
    assert_eq!(state.entry.text(), "4");
    assert!(state.entry.selection_bounds().is_none());
    assert_eq!(state.entry.position(), 1);
    let mut position = state.entry.position();
    state.entry.insert_text("+2", &mut position);
    state.entry.set_position(position);
    state.calculate();
    assert_eq!(state.entry.text(), "6");
    choose(&state, Profile::Scientific);
    assert!(state.scientific.is_visible());
    assert!(state.entry.text().is_empty());
    state.entry.set_text("sin(30)");
    state.calculate();
    assert_eq!(state.entry.text(), "0.5");
    choose(&state, Profile::Programming);
    assert!(state.programming.is_visible());
    state.entry.set_text("9007199254740993+2");
    state.calculate();
    assert_eq!(state.entry.text(), "9007199254740995");
    state.entry.set_text("255");
    state.activate(Action::Base);
    assert_eq!(state.entry.text(), "FF");
    assert_eq!(
      state.preferences.borrow().programming.base,
      programming::Base::Hexadecimal
    );
    choose(&state, Profile::Financial);
    state.entry.set_text("1200");
    state.parameters[0].set_text("0");
    state.parameters[1].set_text("12");
    state.calculate();
    assert_eq!(state.entry.text(), "100");
    // All financial forms, including the third parameter, must fit at the default size.
    for operation in financial::Operation::ALL {
      let mut preferences = state.preferences.borrow().clone();
      preferences.financial = operation;
      state.apply_preferences(preferences);
      settle();
      assert!(
        state.window.width() <= 440 && state.window.height() <= 640,
        "financial allocation: {}x{}",
        state.window.width(),
        state.window.height()
      );
    }
    choose(&state, Profile::Conversion);
    state.entry.set_text("1500");
    state.calculate();
    assert_eq!(state.entry.text(), "1.5");
    let count = state.records.borrow().len();
    state.calculate();
    assert_eq!(state.entry.text(), "1.5");
    assert_eq!(state.records.borrow().len(), count);
    state
      .history_box
      .first_child()
      .unwrap()
      .downcast::<gtk::Button>()
      .unwrap()
      .emit_clicked();
    assert_eq!(state.preferences.borrow().profile, Profile::Conversion);
    assert_eq!(state.preferences.borrow().from, 1);
    assert_eq!(state.entry.text(), "1.5");
    state.calculate();
    assert_eq!(state.entry.text(), "1500");
    choose(&state, Profile::Basic);
    assert_eq!(state.entry.text(), "6");
    settle();
    assert_eq!((state.window.width(), state.window.height()), (370, 420));
    assert!(state.window.is_resizable());
    let key = state.keypad.first_child().unwrap();
    let original = key.height();
    state.window.set_size_request(520, 680);
    settle();
    assert!(key.height() > original);
    state.window.set_size_request(370, 420);
    // Saved records carry the exact profile and options, and reopening restores each profile.
    assert!(history::load(&history_path).unwrap().iter().any(|record| {
      record
        .context
        .as_ref()
        .is_some_and(|context| context.preferences.profile == Profile::Programming)
    }));
    state.window.close();
    drop(state);
    settle();
    for profile in Profile::ALL {
      let preferences = Preferences {
        profile,
        ..Preferences::default()
      };
      history::save_json(&settings_path, &preferences).unwrap();
      let reopened = Calculator::new(
        &application,
        Some(history_path.clone()),
        Some(settings_path.clone()),
      );
      reopened.window.present();
      settle();
      assert_eq!(reopened.preferences.borrow().profile, profile);
      assert_eq!(reopened.window.size_request(), profile.size());
      assert!(!reopened.records.borrow().is_empty());
      reopened.window.close();
      drop(reopened);
      settle();
    }
    let reopened = Calculator::new(
      &application,
      Some(history_path.clone()),
      Some(settings_path),
    );
    reopened.clear_history();
    assert!(history::load(&history_path).unwrap().is_empty());
    reopened.window.close();
  }
}
