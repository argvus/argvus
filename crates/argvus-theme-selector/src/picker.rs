//! Shared GTK layer-shell styling and widgets for keyboard-first pickers.

use argvus_config_core::ConfigDocument;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};

pub fn window(namespace: &str, title: &str, locale: &str) -> gtk::Window {
  let (width, height) = picker_size();
  let window = gtk::Window::builder()
    .title(title)
    .decorated(false)
    .default_width(width)
    .default_height(height)
    .build();
  if gtk4_layer_shell::is_supported() {
    window.init_layer_shell();
    window.set_namespace(Some(namespace));
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_exclusive_zone(-1);
  }
  window.add_css_class("argvus-picker");
  apply_theme(&window, locale);
  window
}

fn picker_size() -> (i32, i32) {
  const DEFAULT: (i32, i32) = (1120, 590);
  let Ok(output) = std::process::Command::new("hyprctl")
    .args(["-j", "monitors"])
    .output()
  else {
    return DEFAULT;
  };
  let Ok(monitors) = serde_json::from_slice::<Vec<serde_json::Value>>(&output.stdout) else {
    return DEFAULT;
  };
  let Some(monitor) = monitors
    .iter()
    .find(|monitor| monitor["focused"].as_bool() == Some(true))
  else {
    return DEFAULT;
  };
  let scale = monitor["scale"]
    .as_f64()
    .filter(|scale| *scale > 0.0)
    .unwrap_or(1.0);
  let rotated = monitor["transform"].as_u64().unwrap_or_default() % 2 == 1;
  let (width, height) = if rotated {
    ("height", "width")
  } else {
    ("width", "height")
  };
  let available = |key: &str, limit: i32| {
    monitor[key].as_f64().map_or(limit, |pixels| {
      ((pixels * 0.92 / scale) as i32).clamp(1, limit)
    })
  };
  (available(width, DEFAULT.0), available(height, DEFAULT.1))
}

pub fn panel(window: &gtk::Window, title: &str) -> gtk::Box {
  let panel = gtk::Box::new(gtk::Orientation::Vertical, 12);
  panel.add_css_class("panel");
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  header.add_css_class("header");
  let brand = gtk::Label::new(Some("ARGVUS"));
  brand.add_css_class("brand");
  header.append(&brand);
  let title = gtk::Label::new(Some(title));
  title.add_css_class("heading");
  title.set_ellipsize(gtk::pango::EllipsizeMode::End);
  header.append(&title);
  panel.append(&header);
  window.set_child(Some(&panel));
  panel
}

pub fn scroll(list: &gtk::ListBox) -> gtk::ScrolledWindow {
  list.set_activate_on_single_click(true);
  gtk::ScrolledWindow::builder()
    .hscrollbar_policy(gtk::PolicyType::Never)
    .vexpand(true)
    .child(list)
    .build()
}

pub fn row(icon: &str, title: &str, detail: &str, shortcut: &str) -> gtk::ListBoxRow {
  let icon = gtk::Image::from_icon_name(icon);
  icon.add_css_class("row-icon");
  let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  content.append(&icon);
  let labels = gtk::Box::new(gtk::Orientation::Vertical, 3);
  labels.set_hexpand(true);
  for (text, class) in [(title, "row-title"), (detail, "detail")] {
    if !text.is_empty() {
      let label = gtk::Label::new(Some(text));
      label.set_halign(gtk::Align::Start);
      label.set_ellipsize(gtk::pango::EllipsizeMode::End);
      label.set_tooltip_text(Some(text));
      label.add_css_class(class);
      labels.append(&label);
    }
  }
  content.append(&labels);
  if !shortcut.is_empty() {
    let label = gtk::Label::new(Some(shortcut));
    label.add_css_class("keycap");
    label.set_valign(gtk::Align::Center);
    content.append(&label);
  }
  gtk::ListBoxRow::builder().child(&content).build()
}

pub fn footer(panel: &gtk::Box, hints: &[(&str, &str)]) {
  let footer = gtk::FlowBox::builder()
    .selection_mode(gtk::SelectionMode::None)
    .column_spacing(16)
    .row_spacing(6)
    .max_children_per_line(8)
    .build();
  footer.add_css_class("footer");
  for (key, action) in hints.iter().filter(|(key, _)| !key.is_empty()) {
    let hint = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let key = gtk::Label::new(Some(key));
    key.add_css_class("keycap");
    hint.append(&key);
    if !action.is_empty() {
      hint.append(&gtk::Label::new(Some(action)));
    }
    footer.insert(&hint, -1);
  }
  panel.append(&footer);
}

fn apply_theme(window: &gtk::Window, locale: &str) {
  use crate::model::Catalog;
  use argvus_config_core::ConfigStore;
  use argvus_theme_core::color::Color;

  let document = ConfigStore::from_environment()
    .and_then(|store| store.load_effective())
    .ok();
  let theme_id = document
    .as_ref()
    .and_then(|document| document.get("/appearance/theme"))
    .and_then(serde_json::Value::as_str)
    .unwrap_or("argvus-dark");
  let theme = Catalog::load(locale, theme_id.to_owned())
    .ok()
    .and_then(|catalog| {
      catalog
        .themes
        .into_iter()
        .find(|theme| theme.id == theme_id)
    });
  let accent = document
    .as_ref()
    .and_then(|document| document.get("/appearance/accent"))
    .and_then(serde_json::Value::as_str)
    .and_then(Color::parse)
    .or_else(|| theme.as_ref().and_then(|theme| theme.token("accent").ok()))
    .unwrap_or_else(|| Color::opaque(53, 144, 189));
  let effect = |name: &str| document.as_ref().and_then(|doc| doc.get(name));
  let enabled = |name| {
    effect(name)
      .and_then(serde_json::Value::as_bool)
      .unwrap_or(true)
  };
  let opacity = if enabled("/effects/transparency_control-center_enabled")
    && enabled("/effects/blur_control-center_enabled")
    && enabled("/effects/blur_global_enabled")
  {
    1.0
      - effect("/effects/transparency_control-center_value")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(15)
        .min(100) as f64
        / 100.0
  } else {
    1.0
  };
  let token = |name, fallback| {
    theme
      .as_ref()
      .and_then(|theme| theme.token(name).ok())
      .unwrap_or(fallback)
      .hex()
  };
  let bg = token("bg", Color::opaque(17, 19, 22));
  let fg = token("fg", Color::opaque(238, 238, 238));
  let on_accent = accent.contrast().hex();
  let accent = accent.hex();
  let (radius, border) = document.as_ref().map_or((0, 1), panel_geometry);
  let search_style =
    crate::render::search_css(".argvus-picker", &fg, &accent, &format!("{radius}px"));

  // GTK 4.22+ derives the Wayland blur region from this filter. Hyprland
  // uses its own blur settings for that region.
  let blur = if opacity < 1.0 && (gtk::major_version(), gtk::minor_version()) >= (4, 22) {
    "backdrop-filter: blur(20px);"
  } else {
    ""
  };
  let provider = gtk::CssProvider::new();
  provider.load_from_string(&format!(
    "window.argvus-picker {{ background: transparent; color: {fg}; font: 15px monospace; }}
     .argvus-picker .panel {{ background: alpha({bg}, {opacity}); {blur} border: {border}px solid alpha({fg}, .38); border-radius: {radius}px; padding: 20px; }}
     .argvus-picker list {{ background: transparent; color: {fg}; }}
     .argvus-picker list row {{ border: 1px solid transparent; border-bottom-color: alpha({fg}, .22); border-radius: {radius}px; padding: 8px 12px; }}
     .argvus-picker list row:selected {{ background: alpha({accent}, .23); border-color: {accent}; color: {fg}; }}
     .argvus-picker list.inactive-selection row:selected {{ background: transparent; border-color: transparent; border-bottom-color: alpha({fg}, .22); }}
     .argvus-picker list row:focus-visible {{ outline: 1px solid {accent}; outline-offset: -2px; }}
     .argvus-picker .header {{ border-bottom: 1px solid alpha({fg}, .22); padding-bottom: 10px; }}
     .argvus-picker .row-title {{ font-weight: bold; }}
     .argvus-picker .detail {{ color: alpha({fg}, .8); }}
     .argvus-picker .row-icon {{ color: {accent}; }}
     .argvus-picker .keycap {{ border: 1px solid alpha({fg}, .45); border-radius: {radius}px; padding: 2px 7px; }}
     .argvus-picker .heading, .argvus-picker .brand {{ font-size: 22px; font-weight: bold; }}
     .argvus-picker .brand {{ background: {accent}; color: {on_accent}; border-radius: {radius}px; padding: 4px 10px; }}
     .argvus-picker .footer {{ border-top: 1px solid {accent}; padding-top: 12px; }}
     .argvus-picker .accent-editor button, .argvus-picker .accent-editor entry {{ border: 1px solid alpha({fg}, .35); border-radius: {radius}px; color: {fg}; background: alpha({bg}, .80); }}
     .argvus-picker .accent-editor button.suggested-action {{ background: {accent}; color: {on_accent}; }}
     .argvus-picker .accent-editor button:disabled {{ opacity: .5; }}
     .argvus-picker .accent-editor drawingarea:focus-visible, .argvus-picker .accent-editor button:focus-visible, .argvus-picker .accent-editor entry:focus-within {{ outline: 2px solid {accent}; outline-offset: 1px; }}
     .argvus-picker .accent-editor entry.error {{ border-color: #e06666; }}
     .argvus-picker .weather-browser {{ background: alpha({bg}, .68); border-radius: {radius}px; }}
     .argvus-picker .weather-tabs {{ border-right: 1px solid alpha({fg}, .22); }}
     .argvus-picker .weather-tabs:dir(rtl) {{ border-right: 0; border-left: 1px solid alpha({fg}, .22); }}
     .argvus-picker .weather-tabs row {{ border-color: transparent; border-bottom-color: alpha({fg}, .14); }}
     .argvus-picker .weather-tabs row:selected {{ background: alpha({accent}, .21); border-color: transparent; }}
     .argvus-picker .clipboard-picker button {{ border-radius: {radius}px; }}
     .argvus-picker .clipboard-picker button:focus-visible {{ outline: 2px solid {accent}; outline-offset: 1px; }}
     .argvus-picker scrollbar slider {{ border-radius: {radius}px; }}
     {search_style}"
  ));
  gtk::style_context_add_provider_for_display(
    &WidgetExt::display(window),
    &provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );
}

fn panel_geometry(document: &ConfigDocument) -> (u64, u64) {
  let radius = if document
    .get("/layout/variant")
    .and_then(serde_json::Value::as_str)
    == Some("sticky")
  {
    0
  } else {
    document.window_rounding()
  };
  let border = document
    .get("/layout/window/border_size")
    .and_then(serde_json::Value::as_u64)
    .unwrap_or(1);
  (radius, border)
}

/// A searchable row. Its position is returned when the user activates it.
pub struct Choice<'a> {
  pub icon: &'a str,
  pub title: &'a str,
  pub detail: &'a str,
  pub shortcut: &'a str,
}

/// Localized labels and Wayland identity of a searchable picker.
pub struct SearchPicker<'a> {
  pub namespace: &'a str,
  pub title: &'a str,
  pub prompt: &'a str,
  pub empty: &'a str,
  pub no_results: &'a str,
  pub activate: &'a str,
}

/// Opens a picker and returns the original index of the selected row.
pub fn choose(
  locale: &str,
  direction: argvus_i18n::TextDirection,
  options: SearchPicker<'_>,
  rows: &[Choice<'_>],
) -> Result<Option<usize>, gtk::glib::BoolError> {
  use gtk::{gdk, glib};
  use std::{cell::RefCell, rc::Rc};

  gtk::init()?;
  gtk::Widget::set_default_direction(match direction {
    argvus_i18n::TextDirection::RightToLeft => gtk::TextDirection::Rtl,
    argvus_i18n::TextDirection::LeftToRight => gtk::TextDirection::Ltr,
  });
  let window = window(options.namespace, options.title, locale);
  let panel = panel(&window, options.title);
  let search = gtk::SearchEntry::builder()
    .placeholder_text(options.prompt)
    .build();
  search.add_css_class("argvus-search");
  search.set_sensitive(!rows.is_empty());
  panel.append(&search);
  let list = gtk::ListBox::new();
  let empty = gtk::Label::new(Some(if rows.is_empty() {
    options.empty
  } else {
    options.no_results
  }));
  empty.set_wrap(true);
  list.set_placeholder(Some(&empty));
  panel.append(&scroll(&list));
  footer(
    &panel,
    &[("↑ ↓", ""), ("Enter", options.activate), ("Esc", "")],
  );

  for entry in rows {
    list.append(&row(entry.icon, entry.title, entry.detail, entry.shortcut));
  }
  let query = Rc::new(RefCell::new(String::new()));
  let filter_query = query.clone();
  let filter_paths: Vec<_> = rows
    .iter()
    .map(|entry| format!("{}\n{}", entry.title, entry.detail).to_lowercase())
    .collect();
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
    *chosen.borrow_mut() = usize::try_from(row.index()).ok();
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn panel_respects_the_layout_and_user_border_settings() {
    let mut document = ConfigDocument::default();
    document.set("/layout/variant", "sticky".into()).unwrap();
    document.set("/layout/window/rounded", true.into()).unwrap();
    document.set("/layout/window/rounding", 8.into()).unwrap();
    document
      .set("/layout/window/border_size", 3.into())
      .unwrap();
    assert_eq!(panel_geometry(&document), (0, 3));

    document.set("/layout/variant", "spaced".into()).unwrap();
    assert_eq!(panel_geometry(&document), (8, 3));

    document
      .set("/layout/window/rounded", false.into())
      .unwrap();
    document
      .set("/layout/window/border_size", 0.into())
      .unwrap();
    assert_eq!(panel_geometry(&document), (0, 0));
  }
}
