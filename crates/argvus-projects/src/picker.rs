//! Interactive project selection. Opening and configuration belong to the command.

use std::{cell::RefCell, path::Path, rc::Rc};

use argvus_i18n::{Lang, TextDirection, tr};
use argvus_theme_selector::picker;
use gtk::{gdk, glib, prelude::*};

pub fn choose(projects: &[String], prompt: &str) -> Result<Option<String>, glib::BoolError> {
  gtk::init()?;
  let locale = argvus_i18n::user_locale();
  let lang = Lang::for_domain(&locale, "launcher");
  let t = |key| tr(lang, key);
  gtk::Widget::set_default_direction(match lang.direction() {
    TextDirection::RightToLeft => gtk::TextDirection::Rtl,
    TextDirection::LeftToRight => gtk::TextDirection::Ltr,
  });

  let title = t("project.picker_title");
  let window = picker::window("argvus-projects", title, &locale);
  let panel = picker::panel(&window, title);
  let search = gtk::SearchEntry::builder()
    .placeholder_text(if prompt.is_empty() {
      t("project.picker_search")
    } else {
      prompt
    })
    .build();
  search.add_css_class("argvus-search");
  search.set_sensitive(!projects.is_empty());
  panel.append(&search);
  let list = gtk::ListBox::new();
  let empty = gtk::Label::new(Some(if projects.is_empty() {
    t("project.picker_empty")
  } else {
    t("project.picker_no_results")
  }));
  empty.set_wrap(true);
  list.set_placeholder(Some(&empty));
  panel.append(&picker::scroll(&list));
  picker::footer(
    &panel,
    &[("↑ ↓", ""), ("Enter", t("search.project")), ("Esc", "")],
  );

  let paths = projects.to_vec();
  for (index, path) in paths.iter().enumerate() {
    let name = Path::new(path)
      .file_name()
      .and_then(|name| name.to_str())
      .unwrap_or(path);
    let shortcut = if index < 9 {
      format!("Super + Ctrl + {}", index + 1)
    } else {
      String::new()
    };
    list.append(&picker::row("folder-symbolic", name, path, &shortcut));
  }
  let query = Rc::new(RefCell::new(String::new()));
  let filter_query = query.clone();
  let filter_paths: Vec<_> = paths.iter().map(|path| path.to_lowercase()).collect();
  list.set_filter_func(move |row| {
    filter_paths[row.index() as usize].contains(filter_query.borrow().as_str())
  });
  let select_first = |list: &gtk::ListBox| {
    let first = (0..)
      .map_while(|index| list.row_at_index(index))
      .find(gtk::prelude::WidgetExt::is_child_visible);
    list.select_row(first.as_ref());
  };
  select_first(&list);
  let filtered_list = list.clone();
  search.connect_changed(move |entry| {
    *query.borrow_mut() = entry.text().trim().to_lowercase();
    filtered_list.invalidate_filter();
    select_first(&filtered_list);
  });

  let result = Rc::new(RefCell::new(None));
  let chosen = result.clone();
  let main_loop = glib::MainLoop::new(None, false);
  let finish = main_loop.clone();
  list.connect_row_activated(move |_, row| {
    *chosen.borrow_mut() = paths.get(row.index() as usize).cloned();
    finish.quit();
  });
  let finish = main_loop.clone();
  window.connect_close_request(move |_| {
    finish.quit();
    glib::Propagation::Stop
  });
  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  let finish = main_loop.clone();
  keys.connect_key_pressed(move |_, key, _, _| {
    match key {
      gdk::Key::Escape => finish.quit(),
      gdk::Key::Return | gdk::Key::KP_Enter => {
        if let Some(row) = list.selected_row() {
          list.emit_by_name::<()>("row-activated", &[&row]);
        }
      }
      gdk::Key::Up | gdk::Key::Down => {
        let step = if key == gdk::Key::Up { -1 } else { 1 };
        let mut index = list.selected_row().map_or(-1, |row| row.index()) + step;
        while let Some(row) = list.row_at_index(index) {
          if row.is_child_visible() {
            list.select_row(Some(&row));
            row.grab_focus();
            break;
          }
          index += step;
        }
      }
      _ => return glib::Propagation::Proceed,
    }
    glib::Propagation::Stop
  });
  window.add_controller(keys);
  window.present();
  search.grab_focus();
  main_loop.run();
  window.destroy();
  Ok(result.take())
}
