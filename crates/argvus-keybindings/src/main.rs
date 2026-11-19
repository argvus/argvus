//! Read-only desktop shortcut reference.
use argvus_config_core::keybindings::{display_keys, load};
use argvus_i18n::{I18n, TextDirection};
use glib::Unichar;
use gtk::{gdk, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::{cell::Cell, collections::BTreeMap, rc::Rc};

#[derive(serde::Deserialize)]
struct Reference {
  category: String,
  keys: String,
  description: String,
}

fn main() -> glib::ExitCode {
  match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
    [] => {}
    [argument] if argument == "--help" => {
      println!("usage: argvus keybindings");
      return glib::ExitCode::SUCCESS;
    }
    _ => {
      eprintln!("usage: argvus keybindings");
      return glib::ExitCode::FAILURE;
    }
  }
  let i18n = match I18n::new("keybindings") {
    Ok(i18n) => i18n,
    Err(error) => {
      eprintln!("argvus keybindings: {error}");
      return glib::ExitCode::FAILURE;
    }
  };
  let app = gtk::Application::builder()
    .application_id("sh.argvus.Keybindings")
    .build();
  app.connect_activate(move |app| {
    if let Some(window) = app.active_window() {
      window.present();
    } else {
      open(app, &i18n);
    }
  });
  app.run_with_args::<&str>(&[])
}

fn search_key(text: &str) -> String {
  glib::normalize(glib::casefold(text), glib::NormalizeMode::Default)
    .chars()
    .filter(|character| character.combining_class() == 0)
    .collect()
}

fn search_backdrop_opacity(window_opacity: f64) -> f64 {
  let target = argvus_theme_selector::render::SELECTOR_BACKDROP_OPACITY;
  // Source-over composition of the tint with the existing window background.
  if window_opacity < target {
    (target - window_opacity) / (1.0 - window_opacity)
  } else {
    0.0
  }
}

fn apply_theme(window: &gtk::ApplicationWindow, locale: &str) {
  use argvus_config_core::ConfigStore;
  use argvus_theme_core::color::Color;
  use argvus_theme_selector::model::Catalog;

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
  let radius = document
    .as_ref()
    .map_or(0, |document| document.window_rounding());
  let search_style =
    argvus_theme_selector::render::search_css(".keybindings", &fg, &accent, &format!("{radius}px"));
  let search_backdrop = search_backdrop_opacity(opacity);
  // GTK 4.22+ derives the Wayland blur region from this filter. Hyprland
  // uses its own blur settings for that region.
  let blur = if opacity < 1.0 && (gtk::major_version(), gtk::minor_version()) >= (4, 22) {
    "backdrop-filter: blur(20px);"
  } else {
    ""
  };
  let provider = gtk::CssProvider::new();
  provider.load_from_string(&format!(
    "window.keybindings {{ background: transparent; color: {fg}; }}
     .keybindings .window-backdrop {{ background: alpha({bg}, {opacity}); {blur} }}
     .keybindings .panel {{ border: 1px solid alpha({fg}, .38); border-radius: {radius}px; padding: 20px; }}
     .keybindings .header, .keybindings .browser {{ background: alpha({bg}, .68); border-radius: {radius}px; font: 15px monospace; }}
     .keybindings list, .keybindings scrolledwindow, .keybindings viewport {{ background: transparent; color: {fg}; }}
     .keybindings row {{ border-bottom: 1px solid alpha({fg}, .14); padding: 8px 12px; }}
     .keybindings row:selected {{ background: alpha({accent}, .21); color: {fg}; border-radius: {radius}px; }}
     .keybindings row:focus-visible {{ outline: 1px solid {accent}; outline-offset: -2px; }}
     .keybindings .categories {{ border-right: 1px solid alpha({fg}, .22); }}
     .keybindings .heading, .keybindings .brand {{ font-size: 22px; font-weight: bold; }}
     .keybindings .brand {{ background: {accent}; color: {on_accent}; padding: 2px 8px; }}
     .keybindings dropdown button {{ border-radius: {radius}px; }}
     .keybindings .category-title {{ font-size: 21px; font-weight: bold; }}
     .keybindings .keycap {{ border: 1px solid alpha({fg}, .52); border-radius: {radius}px; padding: 2px 7px; }}
     .keybindings .footer {{ border-top: 1px solid {accent}; padding-top: 12px; }}
     .keybindings .search-backdrop {{ background: alpha({bg}, {search_backdrop}); border-radius: {radius}px; }}
     {search_style}"
  ));
  gtk::style_context_add_provider_for_display(
    &gtk::prelude::WidgetExt::display(window),
    &provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );
}

fn open(app: &gtk::Application, i18n: &I18n) {
  gtk::Widget::set_default_direction(match i18n.direction() {
    TextDirection::RightToLeft => gtk::TextDirection::Rtl,
    TextDirection::LeftToRight => gtk::TextDirection::Ltr,
  });
  let window = gtk::ApplicationWindow::builder()
    .application(app)
    .title(i18n.tr("title"))
    .build();
  window.add_css_class("keybindings");
  apply_theme(&window, i18n.locale());
  window.set_default_size(760, 560);
  if gtk4_layer_shell::is_supported() {
    window.init_layer_shell();
    window.set_namespace(Some("argvus-keybindings"));
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_exclusive_zone(-1);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
      window.set_anchor(edge, true);
    }
  }
  let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
  content.add_css_class("panel");
  let surface = gtk::Overlay::new();
  let backdrop = gtk::Box::new(gtk::Orientation::Vertical, 0);
  backdrop.add_css_class("window-backdrop");
  surface.set_child(Some(&backdrop));
  surface.add_overlay(&content);
  surface.set_measure_overlay(&content, true);
  window.set_child(Some(&surface));
  window.add_tick_callback({
    let content = content.downgrade();
    move |window, _| {
      if let Some(content) = content.upgrade() {
        let horizontal = (window.width() / 18).clamp(0, 100);
        let vertical = (window.height() / 14).clamp(0, 72);
        if content.margin_start() != horizontal {
          content.set_margin_start(horizontal);
          content.set_margin_end(horizontal);
        }
        if content.margin_top() != vertical {
          content.set_margin_top(vertical);
          content.set_margin_bottom(vertical);
        }
      }
      glib::ControlFlow::Continue
    }
  });
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 14);
  header.add_css_class("header");
  let brand = label("ARGVUS");
  brand.add_css_class("brand");
  brand.set_valign(gtk::Align::Center);
  header.append(&brand);
  let title = i18n.tr("title");
  let heading = label(title.strip_prefix("ARGVUS ").unwrap_or(&title));
  heading.add_css_class("heading");
  header.append(&heading);
  content.append(&header);
  let search = gtk::SearchEntry::new();
  search.add_css_class("argvus-search");
  search.set_hexpand(true);
  search.set_placeholder_text(Some(&i18n.tr("search")));
  let search_backdrop = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  search_backdrop.add_css_class("search-backdrop");
  search_backdrop.append(&search);
  content.append(&search_backdrop);
  let (bindings, load_error) = match load() {
    Ok(bindings) => (bindings, false),
    Err(error) => {
      eprintln!("argvus keybindings: {error}");
      (Vec::new(), true)
    }
  };
  let mut categories = BTreeMap::<String, Vec<Shortcut>>::new();
  for binding in bindings {
    let description = binding.custom.as_ref().map_or_else(
      || {
        let description = i18n.tr(&binding.description_key);
        match binding.description_key.as_str() {
          "keybindings.action.workspace_switch"
          | "keybindings.action.workspace_move"
          | "keybindings.action.project_slot" => {
            let number = binding.id.rsplit(['.', '_']).next().unwrap_or_default();
            format!("{description} {number}")
          }
          _ => description,
        }
      },
      |shortcut| shortcut.description.clone(),
    );
    categories
      .entry(format!("category.{}", binding.category))
      .or_default()
      .push(Shortcut {
        keys: display_keys(&binding.keys),
        description,
        action: binding.action,
        disabled: (!binding.enabled).then(|| i18n.tr("disabled")),
      });
  }
  if !load_error {
    let references: Vec<Reference> = serde_json::from_str(include_str!("reference.json"))
      .expect("the bundled reference is valid JSON");
    for reference in references {
      categories
        .entry(reference.category)
        .or_default()
        .push(Shortcut {
          keys: reference.keys,
          description: i18n.tr(&reference.description),
          action: String::new(),
          disabled: None,
        });
    }
  }
  let body = gtk::Box::new(gtk::Orientation::Horizontal, 16);
  body.set_vexpand(true);
  let sidebar = gtk::ListBox::new();
  let sidebar_scroll = scroll(&sidebar);
  sidebar_scroll.add_css_class("categories");
  body.append(&sidebar_scroll);
  let results = gtk::Box::new(gtk::Orientation::Vertical, 10);
  results.set_hexpand(true);
  let category_title = label(&i18n.tr("all"));
  category_title.add_css_class("category-title");
  results.append(&category_title);
  let list = gtk::ListBox::new();
  list.set_placeholder(Some(&label(&i18n.tr(if load_error {
    "load_error"
  } else {
    "empty"
  }))));
  results.append(&scroll(&list));
  body.append(&results);

  let mut titles = vec![i18n.tr("all")];
  let total = categories.values().map(Vec::len).sum::<usize>();
  sidebar.append(&category_row("view-grid-symbolic", &titles[0], total));
  let mut index = Vec::new();
  for (group, (category, shortcuts)) in categories.into_iter().enumerate() {
    let title = i18n.tr(&category);
    sidebar.append(&category_row(
      category_icon(&category),
      &title,
      shortcuts.len(),
    ));
    titles.push(title.clone());
    for shortcut in shortcuts {
      index.push((
        group + 1,
        search_key(&format!(
          "{} {} {} {}",
          shortcut.description, shortcut.keys, shortcut.action, title
        )),
      ));
      list.append(&shortcut_row(&shortcut));
    }
  }

  let compact = gtk::DropDown::from_strings(&titles.iter().map(String::as_str).collect::<Vec<_>>());
  let browser = gtk::Box::new(gtk::Orientation::Vertical, 14);
  browser.add_css_class("browser");
  browser.append(&compact);
  browser.append(&body);
  content.append(&browser);
  window.add_tick_callback({
    let compact = compact.downgrade();
    let sidebar = sidebar_scroll.downgrade();
    move |window, _| {
      if let (Some(compact), Some(sidebar)) = (compact.upgrade(), sidebar.upgrade()) {
        let narrow = window.width() < 720;
        let sidebar_width = (window.width() / 5).clamp(160, 330);
        if sidebar.width_request() != sidebar_width {
          sidebar.set_width_request(sidebar_width);
        }
        if compact.is_visible() != narrow {
          compact.set_visible(narrow);
          sidebar.set_visible(!narrow);
        }
      }
      glib::ControlFlow::Continue
    }
  });

  let footer = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  footer.add_css_class("footer");
  let hints = gtk::Box::new(gtk::Orientation::Horizontal, 14);
  hints.set_hexpand(true);
  for (key, action) in [
    ("↑↓", "navigate"),
    ("Tab", "switch_panel"),
    ("/", "search"),
    ("Esc", "close"),
  ] {
    let item = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let cap = gtk::Label::new(Some(key));
    cap.add_css_class("keycap");
    item.append(&cap);
    item.append(&label(&i18n.tr(action)));
    hints.append(&item);
  }
  footer.append(&hints);
  let count = label("");
  footer.append(&count);
  browser.append(&footer);

  let selected = Rc::new(Cell::new(0));
  let words = Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
  let index = Rc::new(index);
  let refresh = Rc::new({
    let list = list.downgrade();
    let count = count.downgrade();
    let count_label = i18n.tr("shortcuts");
    let selected = selected.clone();
    let words = words.clone();
    let index = index.clone();
    move || {
      let matches = index
        .iter()
        .filter(|(group, text)| matches_filter(*group, selected.get(), text, &words.borrow()))
        .count();
      if let Some(count) = count.upgrade() {
        count.set_text(&format!("{matches} {count_label}"));
      }
      if let Some(list) = list.upgrade() {
        list.invalidate_filter();
        if list
          .selected_row()
          .is_none_or(|row| !row.is_child_visible())
        {
          list.select_row(first_visible(&list).as_ref());
        }
      }
    }
  });
  list.set_filter_func({
    let selected = selected.clone();
    let words = words.clone();
    move |row| {
      let (group, text) = &index[row.index() as usize];
      matches_filter(*group, selected.get(), text, &words.borrow())
    }
  });
  search.connect_search_changed({
    let refresh = refresh.clone();
    move |search| {
      *words.borrow_mut() = search_key(&search.text())
        .split_whitespace()
        .map(str::to_owned)
        .collect();
      refresh();
    }
  });
  sidebar.connect_row_selected({
    let compact = compact.downgrade();
    let refresh = refresh.clone();
    move |_, row| {
      if let Some(row) = row {
        let category = row.index() as usize;
        selected.set(category);
        category_title.set_text(&titles[category]);
        if let Some(compact) = compact.upgrade() {
          compact.set_selected(category as u32);
        }
        refresh();
      }
    }
  });
  compact.connect_selected_notify({
    let sidebar = sidebar.downgrade();
    move |compact| {
      if let Some(sidebar) = sidebar.upgrade() {
        sidebar.select_row(sidebar.row_at_index(compact.selected() as i32).as_ref());
      }
    }
  });
  sidebar.select_row(sidebar.row_at_index(0).as_ref());
  refresh();
  install_keyboard(&window, &search, &sidebar, &compact, &list);
  window.present();
  focus_list(&list);
}

struct Shortcut {
  keys: String,
  description: String,
  action: String,
  disabled: Option<String>,
}

fn label(text: &str) -> gtk::Label {
  gtk::Label::builder()
    .label(text)
    .wrap(true)
    .xalign(0.0)
    .build()
}

fn scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
  gtk::ScrolledWindow::builder()
    .hscrollbar_policy(gtk::PolicyType::Never)
    .vexpand(true)
    .child(child)
    .build()
}

fn category_icon(category: &str) -> &'static str {
  match category {
    "category.appearance" => "applications-graphics-symbolic",
    "category.applications" => "view-app-grid-symbolic",
    "category.windows" => "window-restore-symbolic",
    "category.workspaces" => "view-grid-symbolic",
    "category.media" => "media-playback-start-symbolic",
    _ => "preferences-system-symbolic",
  }
}

fn category_row(icon: &str, title: &str, count: usize) -> gtk::ListBoxRow {
  let line = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  line.append(&gtk::Image::from_icon_name(icon));
  let title = label(title);
  title.set_hexpand(true);
  line.append(&title);
  let count = label(&count.to_string());
  count.add_css_class("keycap");
  line.append(&count);
  gtk::ListBoxRow::builder()
    .activatable(false)
    .child(&line)
    .build()
}

fn shortcut_row(shortcut: &Shortcut) -> gtk::ListBoxRow {
  let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  let description = label(&shortcut.description);
  description.set_hexpand(true);
  line.append(&description);
  if !shortcut.keys.contains(" + ") && shortcut.keys.chars().count() > 18 {
    let reference = label(&shortcut.keys);
    reference.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    reference.set_max_width_chars(26);
    line.append(&reference);
    return gtk::ListBoxRow::builder()
      .activatable(false)
      .child(&line)
      .build();
  }
  let keys = gtk::Box::new(gtk::Orientation::Horizontal, 5);
  keys.set_halign(gtk::Align::End);
  keys.set_valign(gtk::Align::Center);
  for (index, key) in shortcut.keys.split(" + ").enumerate() {
    if index > 0 {
      keys.append(&gtk::Label::new(Some("+")));
    }
    let cap = label(key);
    cap.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    cap.add_css_class("keycap");
    keys.append(&cap);
  }
  if let Some(disabled) = &shortcut.disabled {
    let state = label(disabled);
    state.add_css_class("dim-label");
    keys.append(&state);
  }
  line.append(&keys);
  gtk::ListBoxRow::builder()
    .activatable(false)
    .child(&line)
    .build()
}

fn matches_filter(group: usize, selected: usize, text: &str, words: &[String]) -> bool {
  (selected == 0 || group == selected) && words.iter().all(|word| text.contains(word))
}

fn first_visible(list: &gtk::ListBox) -> Option<gtk::ListBoxRow> {
  (0..)
    .map_while(|index| list.row_at_index(index))
    .find(gtk::prelude::WidgetExt::is_child_visible)
}

fn focus_list(list: &gtk::ListBox) {
  if let Some(row) = list
    .selected_row()
    .filter(gtk::prelude::WidgetExt::is_child_visible)
    .or_else(|| first_visible(list))
  {
    row.grab_focus();
  } else {
    list.grab_focus();
  }
}

fn install_keyboard(
  window: &gtk::ApplicationWindow,
  search: &gtk::SearchEntry,
  sidebar: &gtk::ListBox,
  compact: &gtk::DropDown,
  list: &gtk::ListBox,
) {
  let keyboard = gtk::EventControllerKey::new();
  let weak_window = window.downgrade();
  let (search, sidebar, compact, list) = (
    search.downgrade(),
    sidebar.downgrade(),
    compact.downgrade(),
    list.downgrade(),
  );
  keyboard.set_propagation_phase(gtk::PropagationPhase::Capture);
  keyboard.connect_key_pressed(move |_, key, _, modifiers| {
    let (Some(window), Some(search), Some(sidebar), Some(compact), Some(list)) = (
      weak_window.upgrade(),
      search.upgrade(),
      sidebar.upgrade(),
      compact.upgrade(),
      list.upgrade(),
    ) else {
      return glib::Propagation::Proceed;
    };
    let focus = gtk::prelude::GtkWindowExt::focus(&window);
    let inside = |widget: &gtk::Widget| {
      focus
        .as_ref()
        .is_some_and(|focus| focus == widget || focus.is_ancestor(widget))
    };
    match key {
      gdk::Key::Escape => window.close(),
      gdk::Key::slash
        if !inside(search.upcast_ref())
          && !modifiers.intersects(
            gdk::ModifierType::CONTROL_MASK
              | gdk::ModifierType::ALT_MASK
              | gdk::ModifierType::SUPER_MASK,
          ) =>
      {
        search.grab_focus();
      }
      gdk::Key::Tab | gdk::Key::ISO_Left_Tab => {
        if inside(list.upcast_ref()) {
          if compact.is_visible() {
            compact.grab_focus();
          } else {
            focus_list(&sidebar);
          }
        } else {
          focus_list(&list);
        }
      }
      _ => return glib::Propagation::Proceed,
    }
    glib::Propagation::Stop
  });
  window.add_controller(keyboard);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn search_backdrop_matches_selector_without_reducing_window_opacity() {
    for window in [0.0, 0.50, 0.85] {
      let tint = search_backdrop_opacity(window);
      assert!((tint + window * (1.0 - tint) - 0.94).abs() < 1e-10);
    }
    assert_eq!(search_backdrop_opacity(0.94), 0.0);
    assert_eq!(search_backdrop_opacity(0.98), 0.0);
    assert_eq!(search_backdrop_opacity(1.0), 0.0);
  }

  #[test]
  fn filters_by_category_and_every_search_term() {
    let words = ["super".into(), "/".into()];
    assert!(matches_filter(2, 0, "super + /", &words));
    assert!(matches_filter(2, 2, "super + /", &words));
    assert!(!matches_filter(2, 1, "super + /", &words));
    assert!(!matches_filter(2, 2, "super + t", &words));
  }

  #[test]
  fn search_normalizes_accents_case_and_keeps_symbols() {
    assert_eq!(search_key("Área SUPER + ?"), "area super + ?");
    assert_eq!(search_key("Über"), "uber");
  }
}
