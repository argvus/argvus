//! ARGVUS desktop emoji picker.
mod catalog;
mod clipboard;
mod state;
mod style;
mod wayland;

use argvus_i18n::{I18n, TextDirection};
use gtk::{gdk, glib, prelude::*};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::{
  cell::{Cell, RefCell},
  env,
  fs::OpenOptions,
  rc::Rc,
};

const CATEGORIES: [&str; 10] = [
  "recent",
  "favorites",
  "all",
  "smileys",
  "people",
  "nature",
  "food",
  "travel",
  "objects",
  "symbols",
];

const ALL_CATEGORY: usize = 2;

fn main() -> glib::ExitCode {
  match env::args().skip(1).collect::<Vec<_>>().as_slice() {
    [command, selection] if clipboard::is_owner_command(command) => {
      return match clipboard::serve(selection == "primary") {
        Ok(()) => glib::ExitCode::SUCCESS,
        Err(error) => {
          eprintln!("argvus emoji-picker: {error}");
          glib::ExitCode::FAILURE
        }
      };
    }
    [] => {}
    [argument] if argument == "--help" => {
      println!("usage: argvus emoji-picker");
      return glib::ExitCode::SUCCESS;
    }
    _ => return glib::ExitCode::FAILURE,
  }
  let Some(runtime) = env::var_os("XDG_RUNTIME_DIR") else {
    return glib::ExitCode::FAILURE;
  };
  let lock = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(false)
    .open(std::path::Path::new(&runtime).join("argvus-emoji.lock"));
  let lock = match lock {
    Ok(lock) => lock,
    Err(error) => {
      eprintln!("argvus emoji-picker: {error}");
      return glib::ExitCode::FAILURE;
    }
  };
  if lock.try_lock().is_err() {
    return glib::ExitCode::SUCCESS;
  }
  let target = wayland::active_window();
  let i18n = match I18n::new("emoji") {
    Ok(i18n) => Rc::new(i18n),
    Err(error) => {
      eprintln!("argvus emoji-picker: {error}");
      return glib::ExitCode::FAILURE;
    }
  };
  let app = gtk::Application::builder()
    .application_id("sh.argvus.Emoji")
    .build();
  app.connect_activate(move |app| open(app, &i18n, target.clone()));
  app.run_with_args::<&str>(&[])
}

struct Picker {
  catalog: Vec<catalog::Emoji>,
  history: RefCell<state::History>,
  history_readable: bool,
  category: Cell<usize>,
  results: RefCell<Vec<usize>>,
  window: glib::WeakRef<gtk::ApplicationWindow>,
  grid: gtk::FlowBox,
  search: gtk::Entry,
  name: gtk::Label,
  code: gtk::Label,
  preview: gtk::Label,
  status: gtk::Label,
  insert: gtk::Button,
  copy: gtk::Button,
  busy: Cell<bool>,
  i18n: Rc<I18n>,
  target: Option<String>,
}

impl Picker {
  fn selected(&self) -> Option<&catalog::Emoji> {
    let child = self.grid.selected_children().into_iter().next()?;
    let index = *self.results.borrow().get(child.index() as usize)?;
    self.catalog.get(index)
  }

  fn details(&self) {
    let item = self.selected();
    self
      .preview
      .set_text(item.map_or("", |item| &item.sequence));
    self.name.set_text(item.map_or("", |item| &item.name));
    self.code.set_text(item.map_or("", |item| &item.unicode));
    self.insert.set_sensitive(item.is_some());
    self.copy.set_sensitive(item.is_some());
  }

  fn refresh(&self, preserve: Option<&str>) {
    let query = catalog::normalize(&self.search.text());
    let history = self.history.borrow();
    let category = self.category.get();
    let results: Vec<usize> = if query.trim().is_empty() && category < ALL_CATEGORY {
      let list = if category == 0 {
        &history.recent
      } else {
        &history.favorites
      };
      list
        .iter()
        .filter_map(|sequence| {
          self
            .catalog
            .iter()
            .position(|item| item.sequence == *sequence)
        })
        .collect()
    } else {
      self
        .catalog
        .iter()
        .enumerate()
        .filter(|(_, item)| {
          !item.sequence.contains(['🏻', '🏼', '🏽', '🏾', '🏿'])
            && if query.trim().is_empty() {
              category == ALL_CATEGORY || item.category == CATEGORIES[category]
            } else {
              query
                .split_whitespace()
                .all(|word| item.search.contains(word))
            }
        })
        .map(|(index, _)| index)
        .collect()
    };
    let selected = preserve
      .and_then(|sequence| {
        results
          .iter()
          .position(|index| self.catalog[*index].sequence == sequence)
      })
      .unwrap_or(0);
    while let Some(child) = self.grid.first_child() {
      self.grid.remove(&child);
    }
    for index in &results {
      let item = &self.catalog[*index];
      let label = gtk::Label::new(Some(&item.sequence));
      label.set_tooltip_text(Some(&item.name));
      label.set_size_request(62, 54);
      let child = gtk::FlowBoxChild::new();
      child.add_css_class("emoji-cell");
      child.update_property(&[gtk::accessible::Property::Label(&item.name)]);
      child.set_child(Some(&label));
      self.grid.insert(&child, -1);
    }
    let is_empty = results.is_empty();
    self.results.replace(results);
    if let Some(child) = self.grid.child_at_index(selected as i32) {
      self.grid.select_child(&child);
    }
    self.status.set_text(&if !self.history_readable {
      self.i18n.tr("state_error")
    } else if is_empty {
      self.i18n.tr("empty")
    } else {
      String::new()
    });
    self.details();
  }

  fn focus_grid(&self) {
    if let Some(child) = self.grid.selected_children().first() {
      child.grab_focus();
    } else {
      self.grid.grab_focus();
    }
  }

  fn move_selection(&self, key: gdk::Key) {
    let Some(selected) = self.grid.selected_children().into_iter().next() else {
      return;
    };
    let len = self.results.borrow().len() as i32;
    let first_row = self.grid.child_at_index(0).and_then(|child| {
      child
        .compute_bounds(&self.grid)
        .map(|bounds| bounds.y() as i32)
    });
    let columns = (0..len)
      .take_while(|index| {
        self.grid.child_at_index(*index).and_then(|child| {
          child
            .compute_bounds(&self.grid)
            .map(|bounds| bounds.y() as i32)
        }) == first_row
      })
      .count()
      .max(1) as i32;
    let rtl = self.grid.direction() == gtk::TextDirection::Rtl;
    let step = match key {
      gdk::Key::Up => -columns,
      gdk::Key::Down => columns,
      gdk::Key::Left if !rtl => -1,
      gdk::Key::Right if rtl => -1,
      _ => 1,
    };
    let index = (selected.index() + step).clamp(0, len - 1);
    if let Some(child) = self.grid.child_at_index(index) {
      self.grid.select_child(&child);
      child.grab_focus();
    }
  }

  fn favorite(&self) {
    let Some(item) = self.selected() else { return };
    let sequence = item.sequence.clone();
    let mut history = self.history.borrow_mut();
    let favorites = &mut history.favorites;
    let added = if let Some(index) = favorites.iter().position(|entry| *entry == sequence) {
      favorites.remove(index);
      false
    } else {
      favorites.push(sequence.clone());
      true
    };
    let saved = self.history_readable && history.save().is_ok();
    drop(history);
    self.refresh(Some(&sequence));
    self.status.set_text(&self.i18n.tr(if !saved {
      "state_error"
    } else if added {
      "favorite_added"
    } else {
      "favorite_removed"
    }));
    self.focus_grid();
  }

  fn deliver(self: &Rc<Self>, insert: bool) {
    let Some(item) = self.selected() else { return };
    let Some(window) = self.window.upgrade() else {
      return;
    };
    if self.busy.replace(true) {
      return;
    }
    let sequence = item.sequence.clone();
    let target = self.target.clone();
    if insert {
      window.set_visible(false);
      gtk::prelude::WidgetExt::display(&window).sync();
    }
    let picker = Rc::clone(self);
    glib::spawn_future_local(async move {
      let sent_sequence = sequence.clone();
      let delivery = gtk::gio::spawn_blocking(move || {
        wayland::deliver(&sent_sequence, insert, target.as_deref())
      })
      .await;
      picker.busy.set(false);
      match delivery {
        Ok(Ok(())) => {
          let mut history = picker.history.borrow_mut();
          history.record(sequence);
          if picker.history_readable && history.save().is_ok() {
            window.close();
          } else {
            picker.status.set_text(&picker.i18n.tr("state_error"));
            window.present();
          }
        }
        result => {
          let key = result.ok().and_then(Result::err).unwrap_or(if insert {
            "insert_error"
          } else {
            "copy_error"
          });
          picker.status.set_text(&picker.i18n.tr(key));
          window.present();
          picker.focus_grid();
        }
      }
    });
  }
}

fn open(app: &gtk::Application, i18n: &Rc<I18n>, target: Option<String>) {
  let catalog = match catalog::load(i18n.locale()) {
    Ok(entries) => entries,
    Err(error) => {
      eprintln!("argvus emoji-picker: {error}");
      return;
    }
  };
  let history = state::History::load();
  let history_readable = history.is_ok();
  gtk::Widget::set_default_direction(match i18n.direction() {
    TextDirection::RightToLeft => gtk::TextDirection::Rtl,
    TextDirection::LeftToRight => gtk::TextDirection::Ltr,
  });
  let (width, height) = wayland::picker_size();
  let window = gtk::ApplicationWindow::builder()
    .application(app)
    .title("ARGVUS Emoji")
    .default_width(width)
    .default_height(height)
    .decorated(false)
    .build();
  if gtk4_layer_shell::is_supported() {
    window.init_layer_shell();
    window.set_namespace(Some("argvus-emoji"));
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_exclusive_zone(-1);
  }
  window.add_css_class("emoji-picker");
  style::apply_theme(&window, i18n.locale());
  let panel = gtk::Box::new(gtk::Orientation::Vertical, 12);
  panel.add_css_class("panel");
  window.set_child(Some(&panel));
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  let brand = gtk::Label::new(Some("ARGVUS"));
  brand.add_css_class("brand");
  let title = gtk::Label::new(Some("Emoji"));
  title.add_css_class("heading");
  header.append(&brand);
  header.append(&title);
  panel.append(&header);
  let search = gtk::Entry::builder()
    .placeholder_text(i18n.tr("search"))
    .primary_icon_name("system-search-symbolic")
    .build();
  panel.append(&search);
  let categories = gtk::Box::new(gtk::Orientation::Horizontal, 6);
  let category_scroll = gtk::ScrolledWindow::builder()
    .hscrollbar_policy(gtk::PolicyType::Automatic)
    .vscrollbar_policy(gtk::PolicyType::Never)
    .child(&categories)
    .build();
  panel.append(&category_scroll);
  let grid = gtk::FlowBox::builder()
    .selection_mode(gtk::SelectionMode::Single)
    .max_children_per_line(12)
    .min_children_per_line(1)
    .homogeneous(true)
    .valign(gtk::Align::Start)
    .focusable(true)
    .row_spacing(6)
    .column_spacing(6)
    .activate_on_single_click(false)
    .build();
  let scroll = gtk::ScrolledWindow::builder()
    .hscrollbar_policy(gtk::PolicyType::Never)
    .vexpand(true)
    .child(&grid)
    .build();
  grid.set_vadjustment(&scroll.vadjustment());
  panel.append(&scroll);
  let details = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  details.add_css_class("details");
  let preview = gtk::Label::new(None);
  preview.add_css_class("preview");
  details.append(&preview);
  let text = gtk::Box::new(gtk::Orientation::Vertical, 4);
  text.set_hexpand(true);
  let name = gtk::Label::builder().xalign(0.0).wrap(true).build();
  let code = gtk::Label::builder().xalign(0.0).wrap(true).build();
  text.append(&name);
  text.append(&code);
  details.append(&text);
  let insert = gtk::Button::with_label(&i18n.tr("insert"));
  insert.add_css_class("primary");
  let copy = gtk::Button::with_label(&i18n.tr("copy"));
  details.append(&insert);
  details.append(&copy);
  panel.append(&details);
  let status = gtk::Label::builder().xalign(0.0).wrap(true).build();
  panel.append(&status);
  let hints = gtk::Label::builder()
    .label(i18n.tr("hints"))
    .wrap(true)
    .xalign(0.0)
    .build();
  hints.add_css_class("footer");
  panel.append(&hints);
  let picker = Rc::new(Picker {
    catalog,
    history: RefCell::new(history.unwrap_or_default()),
    history_readable,
    category: Cell::new(ALL_CATEGORY),
    results: RefCell::new(Vec::new()),
    window: window.downgrade(),
    grid: grid.clone(),
    search: search.clone(),
    name,
    code,
    preview,
    status,
    insert: insert.clone(),
    copy: copy.clone(),
    busy: Cell::new(false),
    i18n: Rc::clone(i18n),
    target,
  });
  connect(&window, &categories, &picker);
  picker.refresh(None);
  window.present();
  picker.focus_grid();
}

fn connect(window: &gtk::ApplicationWindow, categories: &gtk::Box, picker: &Rc<Picker>) {
  let mut first: Option<gtk::ToggleButton> = None;
  for (index, key) in CATEGORIES.into_iter().enumerate() {
    let button = gtk::ToggleButton::with_label(&picker.i18n.tr(key));
    if let Some(first) = &first {
      button.set_group(Some(first));
    } else {
      first = Some(button.clone());
    }
    button.set_active(index == picker.category.get());
    categories.append(&button);
    let picker = Rc::clone(picker);
    button.connect_toggled(move |button| {
      if button.is_active() {
        picker.category.set(index);
        if !picker.search.text().is_empty() {
          picker.search.set_text("");
        } else {
          picker.refresh(None);
        }
      }
    });
  }
  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  keys.connect_key_pressed({
    let picker = Rc::clone(picker);
    let categories = categories.downgrade();
    move |_, key, _, modifiers| {
      let control = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
      let Some(window) = picker.window.upgrade() else {
        return glib::Propagation::Proceed;
      };
      let focus = gtk::prelude::GtkWindowExt::focus(&window);
      let in_search = focus.as_ref().is_some_and(|widget| {
        widget == picker.search.upcast_ref::<gtk::Widget>() || widget.is_ancestor(&picker.search)
      });
      let in_grid = focus.as_ref().is_some_and(|widget| {
        widget == picker.grid.upcast_ref::<gtk::Widget>() || widget.is_ancestor(&picker.grid)
      });
      if picker.busy.get() {
        return glib::Propagation::Stop;
      }
      match key {
        gdk::Key::Tab | gdk::Key::ISO_Left_Tab if !control => {
          let Some(categories) = categories.upgrade() else {
            return glib::Propagation::Proceed;
          };
          let backwards =
            key == gdk::Key::ISO_Left_Tab || modifiers.contains(gdk::ModifierType::SHIFT_MASK);
          let next = if backwards {
            (picker.category.get() + CATEGORIES.len() - 1) % CATEGORIES.len()
          } else {
            (picker.category.get() + 1) % CATEGORIES.len()
          };
          if let Some(button) = categories
            .observe_children()
            .item(next as u32)
            .and_downcast::<gtk::ToggleButton>()
          {
            button.set_active(true);
            button.grab_focus();
            picker.focus_grid();
          }
        }
        gdk::Key::Escape => window.close(),
        gdk::Key::slash if !control && !in_search => {
          picker.search.grab_focus();
        }
        gdk::Key::Return | gdk::Key::KP_Enter if in_grid => picker.deliver(true),
        gdk::Key::Return | gdk::Key::KP_Enter | gdk::Key::Down if in_search => picker.focus_grid(),
        gdk::Key::c if control => picker.deliver(false),
        gdk::Key::d if control => picker.favorite(),
        gdk::Key::Left | gdk::Key::Right | gdk::Key::Up | gdk::Key::Down if !in_search => {
          picker.move_selection(key);
        }
        _ => return glib::Propagation::Proceed,
      }
      glib::Propagation::Stop
    }
  });
  window.add_controller(keys);
  picker.search.connect_changed({
    let picker = Rc::downgrade(picker);
    move |_| {
      if let Some(picker) = picker.upgrade() {
        picker.refresh(None);
      }
    }
  });
  picker.insert.connect_clicked({
    let picker = Rc::downgrade(picker);
    move |_| {
      if let Some(picker) = picker.upgrade() {
        picker.deliver(true);
      }
    }
  });
  picker.copy.connect_clicked({
    let picker = Rc::downgrade(picker);
    move |_| {
      if let Some(picker) = picker.upgrade() {
        picker.deliver(false);
      }
    }
  });
  picker.grid.connect_selected_children_changed({
    let picker = Rc::downgrade(picker);
    move |_| {
      if let Some(picker) = picker.upgrade() {
        picker.details();
      }
    }
  });
  picker.grid.connect_child_activated({
    let picker = Rc::downgrade(picker);
    move |_, _| {
      if let Some(picker) = picker.upgrade() {
        picker.deliver(true);
      }
    }
  });
}
