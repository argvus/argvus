//! Layer-shell carousel. Workers return plain data; GTK objects stay on the main loop.
use crate::{
  model::{self, Catalog, Filter},
  preview,
};
use argvus_i18n::I18n;
use argvus_theme_core::{layout::Variant, manifest::ThemeManifest};
use gtk::{gdk, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::{
  cell::{Cell, RefCell},
  collections::HashMap,
  rc::Rc,
  sync::{Arc, mpsc},
  time::Duration,
};

type Shared = Rc<RefCell<Catalog>>;

pub(crate) const BRAND: &str = "ARGVUS";
const SELECTOR_NAME: &str = "Theme Selector";

#[derive(Clone)]
struct Widgets {
  window: gtk::ApplicationWindow,
  provider: gtk::CssProvider,
  controls: gtk::Box,
  search: gtk::SearchEntry,
  filter: gtk::DropDown,
  drawing: gtk::DrawingArea,
  swatches: gtk::DrawingArea,
  title: gtk::Label,
  detail: gtk::Label,
  previous: gtk::Button,
  apply: gtk::Button,
  next: gtk::Button,
  close: gtk::Button,
  status: gtk::Label,
  spaced: Rc<Cell<bool>>,
  applying: gtk::Box,
  spinner: gtk::Spinner,
}

pub(crate) fn selector_header(name: &str, left: i32, top: i32) -> gtk::Box {
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 14);
  header.set_direction(gtk::TextDirection::Ltr);
  header.set_halign(gtk::Align::Start);
  header.set_valign(gtk::Align::Start);
  header.set_margin_start(left);
  header.set_margin_top(top);
  header.add_css_class("selector-header");
  let brand = gtk::Label::new(Some(BRAND));
  brand.add_css_class("selector-brand");
  header.append(&brand);
  header.append(&gtk::Label::new(Some(name)));
  header
}

fn build(app: &gtk::Application, i18n: &I18n) -> Widgets {
  gtk::Widget::set_default_direction(
    if i18n.direction() == argvus_i18n::TextDirection::RightToLeft {
      gtk::TextDirection::Rtl
    } else {
      gtk::TextDirection::Ltr
    },
  );
  let window = gtk::ApplicationWindow::builder()
    .application(app)
    .title(format!("{BRAND} {SELECTOR_NAME}"))
    .build();
  window.init_layer_shell();
  window.set_namespace(Some("argvus-theme-selector"));
  window.set_layer(Layer::Overlay);
  window.set_keyboard_mode(KeyboardMode::Exclusive);
  window.set_exclusive_zone(-1);
  for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
    window.set_anchor(edge, true);
  }
  let provider = gtk::CssProvider::new();
  provider.load_from_string(&crate::render::css(None, false));
  if let Some(display) = gdk::Display::default() {
    gtk::style_context_add_provider_for_display(
      &display,
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
  }
  window.add_css_class("theme-selector");
  let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
  content.set_valign(gtk::Align::Center);
  content.set_margin_top(24);
  content.set_margin_bottom(24);
  content.set_margin_start(24);
  content.set_margin_end(24);
  let controls = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  controls.set_halign(gtk::Align::Center);
  let search = gtk::SearchEntry::new();
  search.add_css_class("argvus-search");
  search.set_hexpand(true);
  search.set_placeholder_text(Some(&i18n.tr("selector.search")));
  controls.append(&search);
  let filter = gtk::DropDown::from_strings(&[
    &i18n.tr("selector.all"),
    &i18n.tr("theme.category.dark"),
    &i18n.tr("theme.category.light"),
    &i18n.tr("selector.custom"),
  ]);
  filter.set_size_request(152, -1);
  filter.set_tooltip_text(Some(&i18n.tr("selector.filter")));
  controls.append(&filter);
  content.append(&controls);
  let drawing = gtk::DrawingArea::new();
  drawing.set_hexpand(true);
  drawing.set_focusable(true);
  drawing.set_content_height(320);
  drawing.set_vexpand(true);
  let preview_overlay = gtk::Overlay::new();
  preview_overlay.set_child(Some(&drawing));
  let applying = gtk::Box::new(gtk::Orientation::Vertical, 16);
  applying.set_halign(gtk::Align::Center);
  applying.set_valign(gtk::Align::Center);
  applying.add_css_class("applying-preview");
  let spinner = gtk::Spinner::new();
  spinner.set_size_request(64, 64);
  let applying_content = gtk::Box::new(gtk::Orientation::Vertical, 16);
  applying_content.set_valign(gtk::Align::Center);
  applying_content.set_vexpand(true);
  applying_content.append(&spinner);
  applying_content.append(&gtk::Label::new(Some(&i18n.tr("theme.applying"))));
  applying.append(&applying_content);
  applying.set_visible(false);
  preview_overlay.add_overlay(&applying);
  drawing.add_tick_callback({
    let applying = applying.downgrade();
    move |drawing, _| {
      if let Some(applying) = applying.upgrade() {
        let width =
          crate::render::center_width(f64::from(drawing.width()), f64::from(drawing.height()))
            .round() as i32;
        if applying.width_request() != width {
          applying.set_size_request(width, (f64::from(width) * 9.0 / 16.0).round() as i32);
        }
      }
      glib::ControlFlow::Continue
    }
  });
  content.append(&preview_overlay);
  let swatches = gtk::DrawingArea::new();
  swatches.set_content_width(200);
  swatches.set_content_height(14);
  swatches.set_halign(gtk::Align::Center);
  content.append(&swatches);
  let title = gtk::Label::new(None);
  title.add_css_class("title");
  content.append(&title);
  let detail = gtk::Label::new(None);
  detail.set_wrap(true);
  detail.add_css_class("muted");
  content.append(&detail);
  let footer = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  footer.set_halign(gtk::Align::Center);
  let previous = gtk::Button::with_label("←");
  previous.set_tooltip_text(Some(&i18n.tr("selector.previous")));
  let apply = gtk::Button::with_label(&i18n.tr("selector.apply"));
  apply.add_css_class("suggested-action");
  apply.set_sensitive(false);
  let next = gtk::Button::with_label("→");
  next.set_tooltip_text(Some(&i18n.tr("selector.next")));
  let close = gtk::Button::with_label(&i18n.tr("selector.close"));
  for button in [&previous, &apply, &next, &close] {
    footer.append(button);
  }
  content.append(&footer);
  let status = gtk::Label::new(Some(&i18n.tr("selector.loading")));
  status.set_wrap(true);
  content.append(&status);
  let hint = gtk::Label::new(Some(&i18n.tr("selector.hint")));
  hint.add_css_class("muted");
  hint.set_wrap(true);
  content.append(&hint);
  let header = selector_header(SELECTOR_NAME, 35, 30);

  let overlay = gtk::Overlay::new();
  overlay.set_child(Some(&content));
  overlay.add_overlay(&header);
  window.set_child(Some(&overlay));
  Widgets {
    window,
    provider,
    controls,
    search,
    filter,
    drawing,
    swatches,
    title,
    detail,
    previous,
    apply,
    next,
    close,
    status,
    spaced: Rc::new(Cell::new(false)),
    applying,
    spinner,
  }
}

pub fn open(app: &gtk::Application, i18n: Arc<I18n>) {
  let widgets = build(app, &i18n);
  let Widgets {
    window,
    provider: _,
    controls,
    search,
    filter,
    drawing,
    swatches,
    title,
    detail,
    previous,
    apply,
    next,
    close,
    status,
    spaced: _,
    applying: _,
    spinner: _,
  } = widgets.clone();
  let catalog: Shared = Rc::default();
  let images = Rc::new(RefCell::new(HashMap::<
    String,
    (gtk::cairo::ImageSurface, bool),
  >::new()));
  let busy = Rc::new(Cell::new(false));
  let spaced = widgets.spaced.clone();
  let (preview_sender, preview_receiver) = mpsc::channel::<Vec<ThemeManifest>>();
  let refresh: Rc<dyn Fn()> = Rc::new({
    let (catalog, title, detail, drawing, apply, i18n) = (
      catalog.clone(),
      title.clone(),
      detail.clone(),
      drawing.clone(),
      apply.clone(),
      i18n.clone(),
    );
    let swatches = swatches.clone();
    let (images, preview_sender, busy) = (images.clone(), preview_sender.clone(), busy.clone());
    move || {
      let catalog = catalog.borrow();
      let nearby = catalog.nearby();
      images
        .borrow_mut()
        .retain(|id, _| nearby.iter().any(|theme| &theme.id == id));
      let requests = nearby
        .into_iter()
        .filter(|theme| !images.borrow().contains_key(&theme.id))
        .cloned()
        .collect();
      let _ = preview_sender.send(requests);
      if let Some(theme) = catalog.current() {
        title.set_text(model::name(theme, i18n.locale()));
        drawing.update_property(&[gtk::accessible::Property::Label(model::name(
          theme,
          i18n.locale(),
        ))]);
        let active = if theme.id == catalog.active {
          i18n.tr("selector.active")
        } else {
          String::new()
        };
        detail.set_text(&format!(
          "{} · {} / {}  {}  · {}",
          i18n.tr(&format!("theme.category.{}", theme.category.as_str())),
          catalog.selected + 1,
          catalog.visible.len(),
          active,
          if images
            .borrow()
            .get(&theme.id)
            .is_some_and(|(_, illustrative)| !illustrative)
          {
            String::new()
          } else {
            i18n.tr("selector.illustrative")
          }
        ));
        apply.set_sensitive(!busy.get());
      } else {
        title.set_text(&i18n.tr("selector.empty"));
        detail.set_text("");
        apply.set_sensitive(false);
      }
      drawing.queue_draw();
      swatches.queue_draw();
    }
  });
  let animation = Rc::new(Cell::new(None::<std::time::Instant>));
  connect_drawing(&drawing, &swatches, &catalog, &images, &animation, &spaced);
  let navigate: Rc<dyn Fn(isize)> = Rc::new({
    let (catalog, refresh, busy) = (catalog.clone(), refresh.clone(), busy.clone());
    let (animation, drawing) = (animation.clone(), drawing.clone());
    move |offset| {
      if !busy.get() {
        if animation.replace(Some(std::time::Instant::now())).is_none() {
          let animation = animation.clone();
          drawing.add_tick_callback(move |drawing, _| {
            drawing.queue_draw();
            if animation
              .get()
              .is_none_or(|start| start.elapsed() >= Duration::from_millis(160))
            {
              animation.set(None);
              glib::ControlFlow::Break
            } else {
              glib::ControlFlow::Continue
            }
          });
        }
        catalog.borrow_mut().step(offset);
        refresh();
      }
    }
  });
  previous.connect_clicked({
    let navigate = navigate.clone();
    move |_| navigate(-1)
  });
  next.connect_clicked({
    let navigate = navigate.clone();
    move |_| navigate(1)
  });
  let weak = window.downgrade();
  close.connect_clicked(move |_| {
    if let Some(window) = weak.upgrade() {
      window.close();
    }
  });
  let refilter: Rc<dyn Fn()> = Rc::new({
    let (catalog, search, filter, i18n, refresh) = (
      catalog.clone(),
      search.clone(),
      filter.clone(),
      i18n.clone(),
      refresh.clone(),
    );
    move || {
      catalog.borrow_mut().filter(
        search.text().as_str(),
        selected_filter(filter.selected()),
        i18n.locale(),
      );
      refresh();
    }
  });
  search.connect_text_notify({
    let refilter = refilter.clone();
    move |_| refilter()
  });
  filter.connect_selected_notify(move |_| refilter());
  let scroll = gtk::EventControllerScroll::new(
    gtk::EventControllerScrollFlags::BOTH_AXES | gtk::EventControllerScrollFlags::DISCRETE,
  );
  scroll.connect_scroll({
    let navigate = navigate.clone();
    move |_, x, y| {
      navigate(if x + y < 0.0 { -1 } else { 1 });
      glib::Propagation::Stop
    }
  });
  drawing.add_controller(scroll);
  let click = gtk::GestureClick::new();
  click.connect_released({
    let (weak, busy) = (window.downgrade(), busy.clone());
    let catalog = catalog.clone();
    let (navigate, drawing) = (navigate.clone(), drawing.clone());
    move |_, _, x, y| {
      if busy.get() {
        return;
      }
      let width = f64::from(drawing.width());
      let height = f64::from(drawing.height());
      let count = catalog.borrow().visible.len();
      match crate::render::hit_test(width, height, count, x, y) {
        Some(0) => {}
        Some(offset) => navigate(offset),
        None => {
          if let Some(window) = weak.upgrade() {
            window.close();
          }
        }
      }
    }
  });
  drawing.add_controller(click);
  let background = gtk::GestureClick::new();
  background.connect_released({
    let (weak, busy) = (window.downgrade(), busy.clone());
    move |_, _, x, y| {
      if busy.get() {
        return;
      }
      if let Some(window) = weak.upgrade()
        && let Some(bounds) = window
          .child()
          .and_then(|child| child.compute_bounds(&window))
        && (x < f64::from(bounds.x())
          || x > f64::from(bounds.x() + bounds.width())
          || y < f64::from(bounds.y())
          || y > f64::from(bounds.y() + bounds.height()))
      {
        window.close();
      }
    }
  });
  window.add_controller(background);
  connect_keys(&window, &search, &drawing, &apply, &busy, &navigate);
  let (sender, receiver) = mpsc::channel();
  apply.connect_clicked({
    let applying = widgets.applying.clone();
    let spinner = widgets.spinner.clone();
    let (catalog, busy, status, sender, i18n, controls) = (
      catalog.clone(),
      busy.clone(),
      status.clone(),
      sender.clone(),
      i18n.clone(),
      controls.clone(),
    );
    let (apply_button, close_button) = (apply.clone(), close.clone());
    move |_| {
      if busy.replace(true) {
        return;
      }
      let Some(id) = catalog.borrow().current().map(|theme| theme.id.clone()) else {
        busy.set(false);
        return;
      };
      applying.set_visible(true);
      spinner.start();
      status.set_text(&i18n.tr("theme.applying"));
      controls.set_sensitive(false);
      apply_button.set_sensitive(false);
      close_button.set_sensitive(false);
      let sender = sender.clone();
      std::thread::spawn(move || {
        let result = std::process::Command::new("argvus")
          .args(["theme-switch", &id])
          .env("ARGVUS_SELECTOR_APPLY", "1")
          .env("ARGVUS_ACCENT_OFFICIAL", "1")
          .output()
          .map_err(|error| error.to_string())
          .and_then(|output| {
            if output.status.success() {
              Ok(())
            } else {
              Err(String::from_utf8_lossy(&output.stderr).into_owned())
            }
          });
        let _ = sender.send(Event::Applied(result, active_theme()));
      });
    }
  });
  search.connect_activate({
    let apply = apply.clone();
    move |_| {
      if apply.is_sensitive() {
        apply.emit_clicked();
      }
    }
  });
  let image_sender = sender.clone();
  std::thread::spawn(move || {
    while let Ok(mut themes) = preview_receiver.recv() {
      loop {
        if let Some(latest) = preview_receiver.try_iter().last() {
          themes = latest;
        }
        if themes.is_empty() {
          break;
        }
        let theme = themes.remove(0);
        let image = preview::image(&theme);
        if image_sender.send(Event::Image(theme.id, image)).is_err() {
          return;
        }
      }
    }
  });
  let worker = sender.clone();
  let locale = i18n.locale().to_owned();
  std::thread::spawn(move || {
    let _ = worker.send(Event::Monitor(focused_monitor()));
    let _ = worker.send(Event::Layout(layout_variant()));
    let _ = worker.send(Event::Catalog(Catalog::load(&locale, active_theme())));
  });
  let layout_sender = sender.clone();
  std::thread::spawn(move || {
    let mut previous = layout_variant();
    loop {
      std::thread::sleep(Duration::from_millis(500));
      let current = layout_variant();
      if current != previous {
        previous = current.clone();
        if layout_sender.send(Event::Layout(current)).is_err() {
          break;
        }
      }
    }
  });
  poll_events(
    widgets,
    catalog,
    images,
    busy,
    refresh.clone(),
    receiver,
    i18n,
  );
}

pub fn focus_search(window: &gtk::Window) {
  if let Some(search) = window
    .child()
    .and_downcast::<gtk::Overlay>()
    .and_then(|overlay| overlay.child())
    .and_then(|content| content.first_child())
    .and_then(|heading| heading.next_sibling())
    .and_then(|controls| controls.first_child())
    .and_downcast::<gtk::SearchEntry>()
  {
    search.grab_focus();
  }
}

enum Event {
  Monitor(Option<String>),
  Catalog(Result<Catalog, String>),
  Layout(String),
  Image(String, Option<preview::Image>),
  Applied(Result<(), String>, String),
}

fn selected_filter(selected: u32) -> Filter {
  match selected {
    1 => Filter::Dark,
    2 => Filter::Light,
    3 => Filter::Custom,
    _ => Filter::All,
  }
}

fn layout_variant() -> String {
  match argvus_config_core::ConfigStore::from_environment().and_then(|store| store.load_effective())
  {
    Ok(document) => document
      .get("/layout/variant")
      .and_then(|value| value.as_str())
      .and_then(Variant::parse)
      .unwrap_or(Variant::Sticky)
      .as_str()
      .to_owned(),
    Err(error) => {
      eprintln!("argvus-theme-selector: reading layout variant: {error}");
      Variant::Sticky.as_str().to_owned()
    }
  }
}

fn active_theme() -> String {
  match argvus_config_core::ConfigStore::from_environment().and_then(|store| store.load_effective())
  {
    Ok(document) => document
      .get("/appearance/theme")
      .and_then(|value| value.as_str())
      .unwrap_or("argvus-dark")
      .trim_end_matches("-spaced")
      .to_owned(),
    Err(error) => {
      eprintln!("argvus-theme-selector: reading active theme: {error}");
      String::new()
    }
  }
}

/// A stale compositor socket must not prevent the selector from opening.
pub(crate) fn focused_monitor() -> Option<String> {
  use std::{
    process::{Command, Stdio},
    time::Instant,
  };
  let mut child = Command::new("hyprctl")
    .args(["monitors", "-j"])
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()
    .ok()?;
  let deadline = Instant::now() + Duration::from_millis(500);
  loop {
    match child.try_wait() {
      Ok(Some(status)) if status.success() => break,
      Ok(Some(_)) => return None,
      Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
      _ => {
        let _ = child.kill();
        let _ = child.wait();
        return None;
      }
    }
  }
  let output = child.wait_with_output().ok()?;
  let monitors: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
  monitors
    .as_array()?
    .iter()
    .find(|monitor| monitor["focused"] == true)?["name"]
    .as_str()
    .map(str::to_owned)
}

type Images = Rc<RefCell<HashMap<String, (gtk::cairo::ImageSurface, bool)>>>;

fn connect_drawing(
  drawing: &gtk::DrawingArea,
  swatches: &gtk::DrawingArea,
  catalog: &Shared,
  images: &Images,
  animation: &Rc<Cell<Option<std::time::Instant>>>,
  spaced: &Rc<Cell<bool>>,
) {
  swatches.set_draw_func({
    let catalog = catalog.clone();
    move |_, context, _, _| {
      if let Some(theme) = catalog.borrow().current() {
        for (index, token) in ["bg", "fg", "accent", "ok", "warn", "danger"]
          .iter()
          .enumerate()
        {
          if let Ok(color) = theme.token(token) {
            context.set_source_rgb(
              f64::from(color.red) / 255.0,
              f64::from(color.green) / 255.0,
              f64::from(color.blue) / 255.0,
            );
            context.arc(
              10.0 + index as f64 * 34.0,
              7.0,
              6.0,
              0.0,
              std::f64::consts::TAU,
            );
            let _ = context.fill();
          }
        }
      }
    }
  });
  drawing.set_draw_func({
    let animation = animation.clone();
    let (catalog, images, spaced) = (catalog.clone(), images.clone(), spaced.clone());
    move |_, context, width, height| {
      let catalog = catalog.borrow();
      let count = catalog.visible.len();
      if count == 0 {
        return;
      }
      for card in crate::render::cards(f64::from(width), f64::from(height), count) {
        let position =
          (catalog.selected as isize + card.offset).rem_euclid(count as isize) as usize;
        let theme = &catalog.themes[catalog.visible[position]];
        if context.save().is_err() {
          continue;
        }
        let progress = animation
          .get()
          .map(|start| (start.elapsed().as_secs_f64() / 0.16).min(1.0))
          .unwrap_or(1.0);
        let scale = 0.97 + 0.03 * progress;
        context.translate(
          card.x + (1.0 - progress) * 12.0,
          card.y + card.height * (1.0 - scale) / 2.0,
        );
        context.scale(card.width * scale / 960.0, card.height * scale / 540.0);
        let images = images.borrow();
        let image = images.get(&theme.id);
        crate::render::paint(
          context,
          theme,
          image.map(|(pixbuf, _)| pixbuf),
          image.is_none_or(|(_, illustrative)| *illustrative),
          spaced.get(),
        );
        if card.offset != 0 {
          context.set_source_rgba(0.0, 0.0, 0.0, 0.55);
          let _ = context.paint();
        }
        let _ = context.restore();
      }
    }
  });
}

fn connect_keys(
  window: &gtk::ApplicationWindow,
  search: &gtk::SearchEntry,
  drawing: &gtk::DrawingArea,
  apply: &gtk::Button,
  busy: &Rc<Cell<bool>>,
  navigate: &Rc<dyn Fn(isize)>,
) {
  let composing = Rc::new(Cell::new(false));
  if let Some(text) = search.delegate().and_downcast::<gtk::Text>() {
    // Compose dead keys locally because the Wayland input context drops them on this layer surface.
    text.set_im_module(Some("gtk-im-context-simple"));
    text.connect_local("preedit-changed", false, {
      let composing = composing.clone();
      move |values| {
        composing.set(values[1].get::<&str>().is_ok_and(|text| !text.is_empty()));
        None
      }
    });
  }

  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  keys.connect_key_pressed({
    let busy = busy.clone();
    let (navigate, apply, weak, search, drawing) = (
      navigate.clone(),
      apply.clone(),
      window.downgrade(),
      search.clone(),
      drawing.clone(),
    );
    move |_, key, _, modifiers| {
      if busy.get() {
        return glib::Propagation::Stop;
      }
      let focus = weak
        .upgrade()
        .and_then(|window| gtk::prelude::GtkWindowExt::focus(&window));
      let search_focused = focus
        .as_ref()
        .is_some_and(|focus| *focus == search || focus.is_ancestor(&search));
      if search_focused && composing.get() {
        return glib::Propagation::Proceed;
      }
      let plain_key = (modifiers & gtk::accelerator_get_default_mod_mask()).is_empty();
      if key == gdk::Key::Escape {
        if let Some(window) = weak.upgrade() {
          window.close();
        }
      } else if key == gdk::Key::slash || key == gdk::Key::KP_Divide {
        if search_focused {
          drawing.grab_focus();
        } else {
          search.grab_focus();
        }
      } else if key == gdk::Key::Left && (!search_focused || plain_key) {
        navigate(-1);
      } else if key == gdk::Key::Right && (!search_focused || plain_key) {
        navigate(1);
      } else if search_focused {
        return glib::Propagation::Proceed;
      } else if key == gdk::Key::Return || key == gdk::Key::KP_Enter {
        if focus.is_some_and(|focus| focus.is::<gtk::Button>() || focus.is::<gtk::DropDown>()) {
          return glib::Propagation::Proceed;
        }
        if apply.is_sensitive() {
          apply.emit_clicked();
        }
      } else {
        return glib::Propagation::Proceed;
      }
      glib::Propagation::Stop
    }
  });
  window.add_controller(keys);
}

fn poll_events(
  widgets: Widgets,
  catalog: Shared,
  images: Images,
  busy: Rc<Cell<bool>>,
  refresh: Rc<dyn Fn()>,
  receiver: mpsc::Receiver<Event>,
  i18n: Arc<I18n>,
) {
  let Widgets {
    window,
    provider,
    controls,
    search,
    filter,
    drawing,
    close,
    status,
    spaced,
    applying,
    spinner,
    ..
  } = widgets;
  glib::timeout_add_local(Duration::from_millis(30), {
    let (weak, catalog, images, busy, status, refresh, search, filter) = (
      window.downgrade(),
      catalog.clone(),
      images.clone(),
      busy.clone(),
      status.clone(),
      refresh.clone(),
      search.clone(),
      filter.clone(),
    );
    move || {
      let Some(window) = weak.upgrade() else {
        return glib::ControlFlow::Break;
      };
      while let Ok(event) = receiver.try_recv() {
        match event {
          Event::Monitor(name) => {
            if let (Some(name), Some(display)) = (name, gdk::Display::default()) {
              let outputs = display.monitors();
              for index in 0..outputs.n_items() {
                if let Some(monitor) = outputs.item(index).and_downcast::<gdk::Monitor>()
                  && monitor.connector().as_deref() == Some(name.as_str())
                {
                  window.set_monitor(Some(&monitor));
                  let geometry = monitor.geometry();
                  let height = (geometry.height() - 330).clamp(48, 480);
                  drawing.set_content_height(height);
                  controls.set_width_request(crate::render::center_width(
                    f64::from(geometry.width() - 48),
                    f64::from(height),
                  ) as i32);
                  let weak_drawing = drawing.downgrade();
                  let weak_controls = controls.downgrade();
                  monitor.connect_geometry_notify(move |monitor| {
                    if let Some(drawing) = weak_drawing.upgrade() {
                      let geometry = monitor.geometry();
                      let height = (geometry.height() - 330).clamp(48, 480);
                      drawing.set_content_height(height);
                      if let Some(controls) = weak_controls.upgrade() {
                        controls.set_width_request(crate::render::center_width(
                          f64::from(geometry.width() - 48),
                          f64::from(height),
                        ) as i32);
                      }
                    }
                  });
                  let weak = window.downgrade();
                  monitor.connect_invalidate(move |_| {
                    if let Some(window) = weak.upgrade() {
                      window.close();
                    }
                  });
                  break;
                }
              }
            }
            window.present();
            search.grab_focus();
          }
          Event::Catalog(Ok(loaded)) => {
            provider.load_from_string(&crate::render::css(
              loaded.themes.iter().find(|theme| theme.id == loaded.active),
              spaced.get(),
            ));
            *catalog.borrow_mut() = loaded;
            catalog.borrow_mut().filter(
              search.text().as_str(),
              selected_filter(filter.selected()),
              i18n.locale(),
            );
            status.set_text("");
            refresh();
          }
          Event::Catalog(Err(error)) => {
            eprintln!("argvus-theme-selector: {error}");
            status.set_text(&i18n.tr("selector.failed"));
          }
          Event::Layout(variant) => {
            let is_spaced = variant == Variant::Spaced.as_str();
            if spaced.replace(is_spaced) != is_spaced {
              let catalog = catalog.borrow();
              provider.load_from_string(&crate::render::css(
                catalog
                  .themes
                  .iter()
                  .find(|theme| theme.id == catalog.active),
                is_spaced,
              ));
              drawing.queue_draw();
            }
          }
          Event::Image(id, Some(image)) => {
            if !catalog.borrow().nearby().iter().any(|theme| theme.id == id) {
              continue;
            }
            if let Ok(surface) = gtk::cairo::ImageSurface::create_for_data(
              image.pixels,
              gtk::cairo::Format::ARgb32,
              image.width,
              image.height,
              image.width * 4,
            ) {
              images
                .borrow_mut()
                .insert(id, (surface, image.illustrative));
              refresh();
            }
          }
          Event::Image(_, None) => {}
          Event::Applied(Ok(()), _) => {
            window.close();
            return glib::ControlFlow::Break;
          }
          Event::Applied(Err(error), active) => {
            eprintln!("argvus-theme-selector: {error}");
            spinner.stop();
            applying.set_visible(false);
            busy.set(false);
            controls.set_sensitive(true);
            close.set_sensitive(true);
            catalog.borrow_mut().active = active;
            status.set_text(&i18n.tr("selector.failed"));
            refresh();
          }
        }
      }
      glib::ControlFlow::Continue
    }
  });
}
