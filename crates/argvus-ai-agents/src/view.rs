use std::{cell::Cell, process::ExitCode, rc::Rc};

use argvus_i18n::{I18n, TextDirection};
use gtk::{gdk, gio, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::{Agent, matching, move_selection, style};

pub(super) fn select(agents: Vec<Agent>, i18n: &I18n) -> Result<Option<&'static str>, ExitCode> {
  let app = gtk::Application::builder()
    .application_id("sh.argvus.AiAgents")
    .flags(gio::ApplicationFlags::NON_UNIQUE)
    .build();
  let selected = Rc::new(Cell::new(None));
  let i18n = i18n.clone();
  app.connect_activate({
    let agents = Rc::new(agents);
    let selected = selected.clone();
    move |app| open(app, agents.clone(), &i18n, selected.clone())
  });
  if app.run_with_args::<&str>(&[]) == glib::ExitCode::SUCCESS {
    Ok(selected.get())
  } else {
    Err(ExitCode::FAILURE)
  }
}

fn open(
  app: &gtk::Application,
  agents: Rc<Vec<Agent>>,
  i18n: &I18n,
  result: Rc<Cell<Option<&'static str>>>,
) {
  let rtl = i18n.direction() == TextDirection::RightToLeft;
  gtk::Widget::set_default_direction(if rtl {
    gtk::TextDirection::Rtl
  } else {
    gtk::TextDirection::Ltr
  });
  let window = gtk::ApplicationWindow::builder()
    .application(app)
    .title(format!("ARGVUS {}", i18n.tr("title")))
    .default_width(1120)
    .default_height(650)
    .decorated(false)
    .build();
  if gtk4_layer_shell::is_supported() {
    window.init_layer_shell();
    window.set_namespace(Some("argvus-ai-agents"));
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_exclusive_zone(-1);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
      window.set_anchor(edge, true);
    }
  }
  window.add_css_class("ai-agents");
  style::apply_theme(&window, i18n.locale());
  let panel = gtk::Box::new(gtk::Orientation::Vertical, 14);
  panel.add_css_class("panel");
  panel.set_halign(gtk::Align::Center);
  panel.set_valign(gtk::Align::Center);
  let surface = gtk::Overlay::new();
  surface.set_child(Some(&gtk::Box::new(gtk::Orientation::Vertical, 0)));
  surface.add_overlay(&panel);
  window.set_child(Some(&surface));

  let header = gtk::Box::new(gtk::Orientation::Horizontal, 14);
  let brand = gtk::Label::new(Some("ARGVUS"));
  brand.add_css_class("brand");
  brand.set_valign(gtk::Align::Center);
  header.append(&brand);
  let heading = gtk::Box::new(gtk::Orientation::Vertical, 4);
  let title = label(&i18n.tr("title"));
  title.add_css_class("heading");
  heading.append(&title);
  heading.append(&label(&i18n.tr("selector.subtitle")));
  header.append(&heading);
  panel.append(&header);

  let search = gtk::SearchEntry::new();
  search.set_placeholder_text(Some(&i18n.tr("selector.search")));
  search.add_css_class("argvus-search");
  panel.append(&search);

  let grid = gtk::FlowBox::builder()
    .selection_mode(gtk::SelectionMode::Single)
    .activate_on_single_click(true)
    .homogeneous(true)
    .column_spacing(10)
    .row_spacing(10)
    .valign(gtk::Align::Start)
    .build();
  grid.add_css_class("agent-grid");
  let cards: Rc<Vec<_>> = Rc::new(
    agents
      .iter()
      .map(|agent| {
        let child = gtk::FlowBoxChild::new();
        child.add_css_class("agent-card");
        child.set_child(Some(&card(agent)));
        grid.insert(&child, -1);
        child
      })
      .collect(),
  );
  let scroll = gtk::ScrolledWindow::builder()
    .hscrollbar_policy(gtk::PolicyType::Never)
    .vexpand(true)
    .child(&grid)
    .build();
  let empty = label(&i18n.tr(if agents.is_empty() {
    "empty"
  } else {
    "selector.no_matches"
  }));
  empty.set_wrap(true);
  let results = gtk::Stack::new();
  results.set_vexpand(true);
  results.add_named(&scroll, Some("grid"));
  results.add_named(&empty, Some("empty"));
  panel.append(&results);

  let footer = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  footer.add_css_class("footer");
  let hints = label(&i18n.tr("selector.keys"));
  hints.set_wrap(true);
  hints.set_hexpand(true);
  footer.append(&hints);
  let count = gtk::Label::new(None);
  footer.append(&count);
  panel.append(&footer);

  let columns = Rc::new(Cell::new(3));
  set_columns(&grid, columns.get());
  window.add_tick_callback({
    let (panel, grid, columns) = (panel.clone(), grid.clone(), columns.clone());
    move |window, _| {
      let width = (window.width() - 32).clamp(1, 1120);
      let height = (window.height() - 32).clamp(1, 580);
      if panel.width_request() != width || panel.height_request() != height {
        panel.set_size_request(width, height);
      }
      let next = column_count(width);
      if columns.replace(next) != next {
        set_columns(&grid, next);
      }
      glib::ControlFlow::Continue
    }
  });

  let refresh = {
    let (grid, cards, agents, i18n) = (grid.clone(), cards.clone(), agents.clone(), i18n.clone());
    move |query: &str| {
      let matches = matching(&agents, query);
      let previous = grid.selected_children().first().cloned();
      for (index, child) in cards.iter().enumerate() {
        child.set_visible(matches.contains(&index));
      }
      grid.unselect_all();
      if let Some(selected) = previous
        .filter(|child| child.is_visible())
        .or_else(|| matches.first().map(|index| cards[*index].clone()))
      {
        grid.select_child(&selected);
      }
      results.set_visible_child_name(if matches.is_empty() { "empty" } else { "grid" });
      count.set_label(&i18n.tr_args("selector.count", [("count", matches.len().to_string())]));
    }
  };
  refresh("");
  search.connect_changed(move |entry| refresh(entry.text().as_str()));

  let weak = window.downgrade();
  grid.connect_child_activated(move |_, child| {
    if let Some(window) = weak.upgrade()
      && child.is_visible()
      && let Some(agent) = agents.get(child.index() as usize)
    {
      result.set(Some(agent.id));
      window.close();
    }
  });
  let keyboard = gtk::EventControllerKey::new();
  keyboard.set_propagation_phase(gtk::PropagationPhase::Capture);
  let weak = window.downgrade();
  let grid_for_keys = grid.clone();
  keyboard.connect_key_pressed(move |_, mut key, _, modifiers| {
    let Some(window) = weak.upgrade() else {
      return glib::Propagation::Proceed;
    };
    let grid = &grid_for_keys;
    let in_search = gtk::prelude::GtkWindowExt::focus(&window).is_some_and(|focus| {
      focus == search.clone().upcast::<gtk::Widget>() || focus.is_ancestor(&search)
    });
    if rtl {
      key = match key {
        gdk::Key::Left => gdk::Key::Right,
        gdk::Key::Right => gdk::Key::Left,
        _ => key,
      };
    }
    match key {
      gdk::Key::Escape => window.close(),
      gdk::Key::slash
        if !in_search
          && !modifiers.intersects(
            gdk::ModifierType::CONTROL_MASK
              | gdk::ModifierType::ALT_MASK
              | gdk::ModifierType::SUPER_MASK,
          ) =>
      {
        search.grab_focus();
      }
      gdk::Key::Return | gdk::Key::KP_Enter => {
        if let Some(child) = grid.selected_children().first() {
          grid.emit_by_name::<()>("child-activated", &[child]);
        }
      }
      gdk::Key::Left | gdk::Key::Right | gdk::Key::Up | gdk::Key::Down => {
        let visible: Vec<_> = cards.iter().filter(|child| child.is_visible()).collect();
        let current = grid
          .selected_children()
          .first()
          .and_then(|selected| visible.iter().position(|child| *child == selected))
          .unwrap_or(0);
        let next = move_selection(current, visible.len(), columns.get(), key);
        if let Some(child) = visible.get(next).copied() {
          grid.select_child(child);
          if !in_search {
            child.grab_focus();
          }
          if let Some(bounds) = child.compute_bounds(grid) {
            scroll.vadjustment().clamp_page(
              f64::from(bounds.y()),
              f64::from(bounds.y() + bounds.height()),
            );
          }
        }
      }
      _ => return glib::Propagation::Proceed,
    }
    glib::Propagation::Stop
  });
  window.add_controller(keyboard);
  if let Some(first) = grid.selected_children().first() {
    gtk::prelude::GtkWindowExt::set_focus(&window, Some(first));
  }
  window.present();
}

fn column_count(width: i32) -> usize {
  match width {
    940.. => 3,
    640.. => 2,
    _ => 1,
  }
}

fn set_columns(grid: &gtk::FlowBox, count: usize) {
  grid.set_min_children_per_line(1);
  grid.set_max_children_per_line(count as u32);
  grid.set_min_children_per_line(count as u32);
}

fn label(text: &str) -> gtk::Label {
  let label = gtk::Label::new(Some(text));
  label.set_halign(gtk::Align::Start);
  label.set_xalign(0.0);
  label
}

fn card(agent: &Agent) -> gtk::Box {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  let slot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  slot.add_css_class("agent-icon");
  slot.set_size_request(64, 64);
  slot.set_hexpand(false);
  slot.set_valign(gtk::Align::Center);
  let image = load_icon(&agent.icon)
    .map(|texture| gtk::Image::from_paintable(Some(&texture)))
    .unwrap_or_else(|| gtk::Image::from_icon_name("application-x-executable-symbolic"));
  image.set_pixel_size(48);
  image.set_size_request(48, 48);
  image.set_halign(gtk::Align::Center);
  image.set_valign(gtk::Align::Center);
  image.set_hexpand(true);
  slot.append(&image);
  row.append(&slot);

  let text = gtk::Box::new(gtk::Orientation::Vertical, 4);
  text.set_valign(gtk::Align::Center);
  let name = label(agent.name);
  name.add_css_class("agent-name");
  name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  text.append(&name);
  let description = label(&agent.description);
  description.set_wrap(true);
  description.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  description.set_lines(3);
  description.set_ellipsize(gtk::pango::EllipsizeMode::End);
  description.set_max_width_chars(24);
  text.append(&description);
  row.append(&text);
  row
}

fn load_icon(path: &std::path::Path) -> Option<gdk::Texture> {
  if path.extension().and_then(std::ffi::OsStr::to_str) != Some("svg") {
    let icon = image::open(path)
      .ok()?
      .resize(48, 48, image::imageops::FilterType::Lanczos3)
      .into_rgba8();
    let (width, height) = icon.dimensions();
    return Some(icon_texture(
      width,
      height,
      icon.into_raw(),
      gdk::MemoryFormat::R8g8b8a8,
    ));
  }

  let source = std::fs::read(path).ok()?;
  let tree = resvg::usvg::Tree::from_data(&source, &resvg::usvg::Options::default()).ok()?;
  let size = tree.size();
  let scale = 48.0 / size.width().max(size.height());
  let width = (size.width() * scale).round().max(1.0) as u32;
  let height = (size.height() * scale).round().max(1.0) as u32;
  let mut icon = resvg::tiny_skia::Pixmap::new(width, height)?;
  resvg::render(
    &tree,
    resvg::tiny_skia::Transform::from_scale(scale, scale),
    &mut icon.as_mut(),
  );
  Some(icon_texture(
    width,
    height,
    icon.take(),
    gdk::MemoryFormat::R8g8b8a8Premultiplied,
  ))
}

fn icon_texture(
  width: u32,
  height: u32,
  pixels: Vec<u8>,
  format: gdk::MemoryFormat,
) -> gdk::Texture {
  gdk::MemoryTexture::new(
    width as i32,
    height as i32,
    format,
    &glib::Bytes::from_owned(pixels),
    width as usize * 4,
  )
  .upcast()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn selects_columns_by_panel_width() {
    assert_eq!(column_count(1120), 3);
    assert_eq!(column_count(800), 2);
    assert_eq!(column_count(500), 1);
  }
}
