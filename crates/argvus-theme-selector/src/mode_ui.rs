//! Layout-mode selector; configuration and application belong to argvus-config.
use crate::{model::Catalog, preview, render};
use argvus_config_core::{
  layout_preset::{self, ApplyOutcome, LayoutPreset, TaskbarPosition},
  window_mode::{self, WindowMode},
};
use argvus_i18n::I18n;
use argvus_theme_core::{layout::Variant, manifest::ThemeManifest};
use gtk::{gdk, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::{
  cell::{Cell, RefCell},
  rc::Rc,
  sync::{Arc, mpsc},
  time::Duration,
};

type Surface = Rc<RefCell<Option<gtk::cairo::ImageSurface>>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WindowTaskbarChoice {
  windows: WindowMode,
  position: TaskbarPosition,
}

impl WindowTaskbarChoice {
  const ALL: [Self; 4] = [
    Self {
      windows: WindowMode::Tile,
      position: TaskbarPosition::Top,
    },
    Self {
      windows: WindowMode::Tile,
      position: TaskbarPosition::Bottom,
    },
    Self {
      windows: WindowMode::Floating,
      position: TaskbarPosition::Top,
    },
    Self {
      windows: WindowMode::Floating,
      position: TaskbarPosition::Bottom,
    },
  ];

  fn from_preset(preset: LayoutPreset) -> usize {
    Self::ALL
      .iter()
      .position(|choice| choice.windows == preset.windows && choice.position == preset.position)
      .unwrap_or(0)
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FocusGroup {
  Modes,
  Submodes,
}

impl FocusGroup {
  fn next(self) -> Self {
    match self {
      Self::Modes => Self::Submodes,
      Self::Submodes => Self::Modes,
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Selection {
  variant: Variant,
  windows: WindowMode,
  position: TaskbarPosition,
  cursor: usize,
  focus: FocusGroup,
}

impl Selection {
  fn from_preset(preset: LayoutPreset) -> Self {
    Self {
      variant: preset.variant,
      windows: preset.windows,
      position: preset.position,
      cursor: WindowTaskbarChoice::from_preset(preset),
      focus: FocusGroup::Modes,
    }
  }

  fn preset(self) -> LayoutPreset {
    LayoutPreset {
      variant: self.variant,
      windows: self.windows,
      position: self.position,
    }
  }

  fn index(self) -> usize {
    LayoutPreset::ALL
      .iter()
      .position(|preset| *preset == self.preset())
      .unwrap_or(0)
  }

  fn choose(self, cursor: usize) -> Self {
    let choice = WindowTaskbarChoice::ALL[cursor];
    Self {
      windows: choice.windows,
      position: choice.position,
      cursor,
      focus: FocusGroup::Submodes,
      ..self
    }
  }

  fn navigate(self, key: gdk::Key, columns: usize) -> Self {
    if self.focus == FocusGroup::Modes {
      return self.move_mode(key);
    }
    let cursor = match key {
      gdk::Key::Left if !self.cursor.is_multiple_of(columns) => self.cursor - 1,
      gdk::Key::Right if self.cursor % columns + 1 < columns => self.cursor + 1,
      gdk::Key::Up if self.cursor >= columns => self.cursor - columns,
      gdk::Key::Down if self.cursor + columns < WindowTaskbarChoice::ALL.len() => {
        self.cursor + columns
      }
      _ => self.cursor,
    };
    self.choose(cursor)
  }

  fn move_mode(self, key: gdk::Key) -> Self {
    let mut selected = self;
    match key {
      gdk::Key::Up | gdk::Key::Left => selected.variant = Variant::Sticky,
      gdk::Key::Down | gdk::Key::Right => selected.variant = Variant::Spaced,
      _ => {}
    }
    selected.focus = FocusGroup::Modes;
    selected
  }
}

#[derive(Clone, Copy)]
enum CardKind {
  Variant(Variant),
  WindowTaskbar(WindowTaskbarChoice),
}

#[derive(Clone)]
struct Card {
  kind: CardKind,
  button: gtk::Button,
  drawing: gtk::DrawingArea,
  cover: gtk::Box,
  spinner: gtk::Spinner,
  indicator: Option<gtk::Label>,
}

struct State {
  theme: ThemeManifest,
  selection: Selection,
  image: Option<preview::Image>,
  pending: bool,
  accent: argvus_theme_core::color::Color,
  radius: u64,
}

enum Event {
  Loaded(Result<Box<State>, String>, Option<String>),
  Applied(Result<ApplyOutcome, String>, Result<usize, String>),
}

fn document() -> Result<argvus_config_core::ConfigDocument, String> {
  argvus_config_core::ConfigStore::from_environment()
    .and_then(|store| store.load_effective())
    .map_err(|error| error.to_string())
}

fn active_mode() -> Result<usize, String> {
  LayoutPreset::from_document(&document()?)
    .map(LayoutPreset::index)
    .map_err(|error| error.to_string())
}

fn load(locale: &str) -> Result<State, String> {
  let document = document()?;
  let id = document
    .get("/appearance/theme")
    .and_then(|value| value.as_str())
    .unwrap_or("argvus-dark")
    .trim_end_matches("-spaced")
    .trim_end_matches("-float");
  let catalog = Catalog::load(locale, id.to_owned())?;
  let mut theme = catalog
    .themes
    .into_iter()
    .find(|theme| theme.id == id)
    .ok_or_else(|| format!("active theme not installed: {id}"))?;
  theme.preview = None;
  if let Some(wallpaper) = document
    .get("/appearance/wallpaper")
    .and_then(|value| value.as_str())
    .filter(|value| !value.is_empty())
  {
    let path = std::path::PathBuf::from(wallpaper);
    if path.is_absolute() && path.is_file() {
      theme.wallpaper = Some(path);
    }
  }
  let image = preview::image(&theme);
  let saved = active_mode()?;
  let preset = LayoutPreset::ALL
    .get(saved)
    .copied()
    .ok_or_else(|| "saved layout preset is invalid".to_owned())?;
  let accent = document
    .get("/appearance/accent")
    .and_then(|value| value.as_str())
    .and_then(argvus_theme_core::color::Color::parse)
    .unwrap_or_else(|| {
      theme
        .token("accent")
        .unwrap_or(argvus_theme_core::color::Color::opaque(53, 144, 189))
    });
  let radius = document.window_rounding();
  Ok(State {
    accent,
    radius,
    theme,
    selection: Selection::from_preset(preset),
    image,
    pending: window_mode::status().map_or(true, |status| status.pending || !status.available),
  })
}

fn apply_mode(index: usize) -> Result<ApplyOutcome, String> {
  let preset = LayoutPreset::ALL
    .get(index)
    .ok_or("unknown layout preset")?;
  layout_preset::apply(*preset).map_err(|error| error.to_string())
}

fn grid_columns(width: i32) -> usize {
  if width >= 1200 { 4 } else { 2 }
}

fn preset_for_card(kind: CardKind, selection: Selection) -> LayoutPreset {
  let mut preset = selection.preset();
  match kind {
    CardKind::Variant(variant) => preset.variant = variant,
    CardKind::WindowTaskbar(choice) => {
      preset.windows = choice.windows;
      preset.position = choice.position;
    }
  }
  preset
}

fn card_is_selected(kind: CardKind, selection: Selection) -> bool {
  match kind {
    CardKind::Variant(variant) => selection.variant == variant,
    CardKind::WindowTaskbar(choice) => {
      selection.windows == choice.windows && selection.position == choice.position
    }
  }
}

fn refresh_cards(cards: &[Card], selection: Selection) {
  for card in cards {
    let selected = card_is_selected(card.kind, selection);
    if let Some(indicator) = &card.indicator {
      indicator.set_text(if selected { "✓" } else { "" });
      indicator.set_css_classes(&[if selected {
        "mode-checked"
      } else {
        "mode-unchecked"
      }]);
    }
    card
      .button
      .update_state(&[gtk::accessible::State::Pressed(if selected {
        gtk::AccessibleTristate::True
      } else {
        gtk::AccessibleTristate::False
      })]);
    if selected {
      card.button.add_css_class("mode-selected");
    } else {
      card.button.remove_css_class("mode-selected");
    }
    let has_cursor =
      matches!(card.kind, CardKind::Variant(_)) && selection.focus == FocusGroup::Modes && selected
        || matches!(card.kind, CardKind::WindowTaskbar(_))
          && selection.focus == FocusGroup::Submodes
          && Some(selection.cursor) == option_kind_for_card(card.kind);
    if has_cursor {
      card.button.add_css_class("mode-cursor");
    } else {
      card.button.remove_css_class("mode-cursor");
    }
    card.drawing.queue_draw();
  }
}

// Card kinds are compared through their domain values to keep the keyboard cursor precise.
fn option_kind_for_card(kind: CardKind) -> Option<usize> {
  match kind {
    CardKind::Variant(_) => None,
    CardKind::WindowTaskbar(choice) => WindowTaskbarChoice::ALL
      .iter()
      .position(|item| item.windows == choice.windows && item.position == choice.position),
  }
}

fn card_title(kind: CardKind, i18n: &I18n) -> String {
  match kind {
    CardKind::Variant(Variant::Sticky) => i18n.tr("theme.model.sticky"),
    CardKind::Variant(Variant::Spaced) => i18n.tr("theme.model.spaced"),
    CardKind::WindowTaskbar(choice) => format!(
      "{} · {}",
      i18n.tr(if choice.windows == WindowMode::Tile {
        "layout_mode.tile"
      } else {
        "layout_mode.floating"
      }),
      i18n.tr(if choice.position == TaskbarPosition::Top {
        "layout_mode.top"
      } else {
        "layout_mode.bottom"
      })
    ),
  }
}

fn mode_icon(size: i32) -> gtk::Box {
  let icon = gtk::Box::new(gtk::Orientation::Horizontal, 2);
  icon.set_valign(gtk::Align::Center);
  for _ in 0..2 {
    let pane = gtk::Box::new(gtk::Orientation::Vertical, 0);
    pane.set_size_request(size, size + 1);
    pane.add_css_class("mode-icon-pane");
    icon.append(&pane);
  }
  icon
}

pub fn open(app: &gtk::Application, i18n: Arc<I18n>) {
  gtk::Widget::set_default_direction(
    if i18n.direction() == argvus_i18n::TextDirection::RightToLeft {
      gtk::TextDirection::Rtl
    } else {
      gtk::TextDirection::Ltr
    },
  );
  let window = gtk::ApplicationWindow::builder()
    .application(app)
    .title(i18n.tr("layout_mode.select"))
    .build();
  window.init_layer_shell();
  window.set_namespace(Some("argvus-layout-mode-selector"));
  window.set_layer(Layer::Overlay);
  window.set_keyboard_mode(KeyboardMode::Exclusive);
  window.set_exclusive_zone(-1);
  for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
    window.set_anchor(edge, true);
  }
  let provider = gtk::CssProvider::new();
  provider.load_from_string(&render::mode_css(
    None,
    argvus_theme_core::color::Color::opaque(53, 144, 189),
    0,
  ));
  gtk::style_context_add_provider_for_display(
    &gtk::prelude::WidgetExt::display(&window),
    &provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );
  window.add_css_class("layout-mode-selector");

  let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
  content.set_valign(gtk::Align::Center);
  content.set_margin_start(24);
  content.set_margin_end(24);
  let brand = gtk::Label::new(Some("A R G V U S"));
  brand.add_css_class("mode-brand");
  content.append(&brand);
  let title = gtk::Label::new(Some(&i18n.tr("layout_mode.select")));
  title.add_css_class("mode-title");
  content.append(&title);
  let instruction = gtk::Label::new(Some(&i18n.tr("layout_mode.intro")));
  instruction.set_wrap(true);
  instruction.set_justify(gtk::Justification::Center);
  instruction.add_css_class("mode-muted");
  content.append(&instruction);

  let selection = Rc::new(Cell::new(Selection::from_preset(LayoutPreset::ALL[0])));
  let theme = Rc::new(RefCell::new(None::<ThemeManifest>));
  let image: Surface = Rc::default();
  let busy = Rc::new(Cell::new(false));
  let mut cards = Vec::with_capacity(6);
  let segments = gtk::Box::new(gtk::Orientation::Horizontal, 2);
  segments.set_homogeneous(true);
  segments.set_halign(gtk::Align::Center);
  segments.set_direction(gtk::TextDirection::Ltr);
  segments.add_css_class("mode-segments");
  for variant in [Variant::Sticky, Variant::Spaced] {
    let card = make_card(
      CardKind::Variant(variant),
      &i18n,
      &selection,
      &theme,
      &image,
    );
    segments.append(&card.button);
    cards.push(card);
  }
  content.append(&segments);

  let panel = gtk::Box::new(gtk::Orientation::Vertical, 16);
  panel.add_css_class("mode-panel");
  let heading = gtk::Box::new(gtk::Orientation::Horizontal, 14);
  heading.append(&mode_icon(24));
  let mode_text = gtk::Box::new(gtk::Orientation::Vertical, 4);
  let mode_title = gtk::Label::new(None);
  mode_title.set_xalign(0.0);
  mode_title.add_css_class("mode-heading");
  let description = gtk::Label::new(None);
  description.set_xalign(0.0);
  description.set_wrap(true);
  description.add_css_class("mode-muted");
  mode_text.append(&mode_title);
  mode_text.append(&description);
  heading.append(&mode_text);
  panel.append(&heading);

  let submode_grid = gtk::Grid::new();
  submode_grid.set_row_spacing(12);
  submode_grid.set_column_spacing(12);
  submode_grid.set_column_homogeneous(true);
  submode_grid.set_direction(gtk::TextDirection::Ltr);
  for choice in WindowTaskbarChoice::ALL {
    cards.push(make_card(
      CardKind::WindowTaskbar(choice),
      &i18n,
      &selection,
      &theme,
      &image,
    ));
  }
  panel.append(&submode_grid);
  let scroll = gtk::ScrolledWindow::builder()
    .hscrollbar_policy(gtk::PolicyType::Never)
    .propagate_natural_height(true)
    .child(&panel)
    .build();
  content.append(&scroll);
  let columns = Rc::new(Cell::new(2));

  let status = gtk::Label::new(Some(&i18n.tr("selector.loading")));
  status.set_wrap(true);
  content.append(&status);
  let footer = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  footer.set_halign(gtk::Align::Center);
  footer.add_css_class("mode-actions");
  let apply = gtk::Button::with_label(&i18n.tr("layout_mode.apply_short"));
  apply.add_css_class("suggested-action");
  apply.set_sensitive(false);
  let close = gtk::Button::with_label(&i18n.tr("selector.close"));
  footer.append(&apply);
  footer.append(&close);
  content.append(&footer);
  let hint = gtk::Label::new(Some(&i18n.tr("layout_mode.hint")));
  hint.set_wrap(true);
  hint.add_css_class("mode-muted");
  content.append(&hint);
  let background = gtk::DrawingArea::new();
  background.set_draw_func({
    let image = image.clone();
    let theme = theme.clone();
    move |_, context, width, height| {
      render::paint_mode_background(
        context,
        theme.borrow().as_ref(),
        image.borrow().as_ref(),
        width,
        height,
      );
    }
  });
  let overlay = gtk::Overlay::new();
  overlay.set_child(Some(&background));
  overlay.add_overlay(&content);
  overlay.set_measure_overlay(&content, true);
  window.set_child(Some(&overlay));

  let refresh_heading = Rc::new({
    let i18n = i18n.clone();
    move |variant| {
      mode_title.set_text(&card_title(CardKind::Variant(variant), &i18n));
      description.set_text(&i18n.tr(if variant == Variant::Sticky {
        "layout_mode.sticky_description"
      } else {
        "layout_mode.spaced_description"
      }));
    }
  });
  refresh_heading(selection.get().variant);

  for card in &cards {
    connect_card(card, &cards, &selection, &status, &refresh_heading);
  }

  close.connect_clicked({
    let weak = window.downgrade();
    move |_| {
      if let Some(window) = weak.upgrade() {
        window.close();
      }
    }
  });
  let backdrop_click = gtk::GestureClick::new();
  backdrop_click.connect_released({
    let weak = window.downgrade();
    let content = content.clone();
    let busy = busy.clone();
    move |_, _, x, y| {
      if let Some(window) = weak.upgrade()
        && !busy.get()
        && let Some(bounds) = content.compute_bounds(&window)
        && (y < f64::from(bounds.y())
          || y > f64::from(bounds.y() + bounds.height())
          || x < f64::from(bounds.x())
          || x > f64::from(bounds.x() + bounds.width()))
      {
        window.close();
      }
    }
  });
  window.add_controller(backdrop_click);
  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  keys.connect_key_pressed({
    let busy = busy.clone();
    let selection = selection.clone();
    let cards = cards.clone();
    let status = status.clone();
    let apply = apply.clone();
    let close = close.clone();
    let weak = window.downgrade();
    let columns = columns.clone();
    let refresh_heading = refresh_heading.clone();
    move |_, key, _, modifiers| {
      if busy.get() {
        return glib::Propagation::Stop;
      }
      match key {
        gdk::Key::Escape => {
          if let Some(window) = weak.upgrade() {
            window.close();
          }
        }
        gdk::Key::ISO_Left_Tab => {}
        gdk::Key::Tab if !modifiers.contains(gdk::ModifierType::SHIFT_MASK) => {
          let mut current = selection.get();
          current.focus = current.focus.next();
          selection.set(current);
          refresh_heading(current.variant);
          refresh_cards(&cards, current);
          let focused_card = match current.focus {
            FocusGroup::Modes => cards.iter().find(
              |card| matches!(card.kind, CardKind::Variant(variant) if variant == current.variant),
            ),
            FocusGroup::Submodes => cards.iter().find(|card| {
              matches!(card.kind, CardKind::WindowTaskbar(_))
                && option_kind_for_card(card.kind) == Some(current.cursor)
            }),
          };
          if let Some(card) = focused_card {
            card.button.grab_focus();
          }
        }
        gdk::Key::Tab => {}
        gdk::Key::Up | gdk::Key::Down | gdk::Key::Left | gdk::Key::Right => {
          let current = selection.get().navigate(key, columns.get());
          selection.set(current);
          refresh_heading(current.variant);
          refresh_cards(&cards, current);
          status.set_visible(false);
          if let Some(card) = cards.iter().find(|card| {
            card_is_selected(card.kind, current)
              && matches!(
                (current.focus, card.kind),
                (FocusGroup::Modes, CardKind::Variant(_))
                  | (FocusGroup::Submodes, CardKind::WindowTaskbar(_))
              )
          }) {
            card.button.grab_focus();
          }
        }
        gdk::Key::Return | gdk::Key::KP_Enter => {
          if let Some(window) = weak.upgrade()
            && gtk::prelude::GtkWindowExt::focus(&window).is_some_and(|focus| focus == close)
          {
            return glib::Propagation::Proceed;
          }
          if apply.is_sensitive() {
            apply.emit_clicked();
          }
        }
        _ => return glib::Propagation::Proceed,
      }
      glib::Propagation::Stop
    }
  });
  window.add_controller(keys);

  let (sender, receiver) = mpsc::channel();
  apply.connect_clicked({
    let cards = cards.clone();
    let selection = selection.clone();
    let busy = busy.clone();
    let status = status.clone();
    let i18n = i18n.clone();
    let close = close.clone();
    let sender = sender.clone();
    move |apply| {
      if busy.replace(true) {
        return;
      }
      let selected = selection.get();
      let spinner_option = selected.cursor;
      if let Some(card) = cards
        .iter()
        .find(|card| option_kind_for_card(card.kind) == Some(spinner_option))
      {
        card.cover.set_visible(true);
        card.spinner.start();
      }
      for card in &cards {
        card.button.set_sensitive(false);
      }
      close.set_sensitive(false);
      apply.set_sensitive(false);
      status.set_text(&i18n.tr("layout_mode.applying"));
      status.set_visible(true);
      let sender = sender.clone();
      std::thread::spawn(move || {
        let result = apply_mode(selected.index());
        let _ = sender.send(Event::Applied(result, active_mode()));
      });
    }
  });

  let locale = i18n.locale().to_owned();
  std::thread::spawn(move || {
    let monitor = crate::ui::focused_monitor();
    let _ = sender.send(Event::Loaded(load(&locale).map(Box::new), monitor));
  });
  let weak = window.downgrade();
  glib::timeout_add_local(Duration::from_millis(30), move || {
    let Some(window) = weak.upgrade() else {
      return glib::ControlFlow::Break;
    };
    while let Ok(event) = receiver.try_recv() {
      match event {
        Event::Loaded(result, monitor) => {
          connect_monitor(
            &window,
            monitor.as_deref(),
            &cards,
            &submode_grid,
            &columns,
            &scroll,
          );
          match result {
            Ok(state) => {
              selection.set(state.selection);
              refresh_heading(state.selection.variant);
              provider.load_from_string(&render::mode_css(
                Some(&state.theme),
                state.accent,
                state.radius,
              ));
              refresh_cards(&cards, state.selection);
              *theme.borrow_mut() = Some(state.theme);
              if let Some(loaded) = state.image {
                *image.borrow_mut() = gtk::cairo::ImageSurface::create_for_data(
                  loaded.pixels,
                  gtk::cairo::Format::ARgb32,
                  loaded.width,
                  loaded.height,
                  loaded.width * 4,
                )
                .ok();
              }
              background.queue_draw();
              for card in &cards {
                card.drawing.queue_draw();
              }
              status.set_text(&if state.pending {
                i18n.tr("layout_mode.pending")
              } else {
                String::new()
              });
              status.set_visible(state.pending);
              apply.set_sensitive(true);
              window.present();
              if let Some(card) = cards.iter().find(|card| {
                matches!(card.kind, CardKind::Variant(variant) if variant == state.selection.variant)
              }) {
                card.button.grab_focus();
              }
            }
            Err(error) => {
              eprintln!("layout-mode-selector: {error}");
              status.set_text(&i18n.tr("layout_mode.failed"));
              window.present();
            }
          }
        }
        Event::Applied(result, active) => {
          if let Err(error) = active {
            eprintln!("layout-mode-selector: could not reread the saved layout: {error}");
          }
          for card in &cards {
            card.spinner.stop();
            card.cover.set_visible(false);
            card.button.set_sensitive(true);
          }
          busy.set(false);
          close.set_sensitive(true);
          match result {
            Ok(ApplyOutcome::Applied) => {
              window.close();
              return glib::ControlFlow::Break;
            }
            result => {
              let pending = matches!(result, Ok(ApplyOutcome::Pending));
              if let Err(error) = result {
                eprintln!("layout-mode-selector: {error}");
              }
              apply.set_sensitive(true);
              status.set_text(&i18n.tr(if pending {
                "layout_mode.pending"
              } else {
                "layout_mode.failed"
              }));
            }
          }
        }
      }
    }
    glib::ControlFlow::Continue
  });
}

fn make_card(
  kind: CardKind,
  i18n: &I18n,
  selection: &Rc<Cell<Selection>>,
  theme: &Rc<RefCell<Option<ThemeManifest>>>,
  image: &Surface,
) -> Card {
  let drawing = gtk::DrawingArea::new();
  drawing.set_content_height(170);
  drawing.set_content_width(260);
  drawing.set_hexpand(true);
  drawing.set_draw_func({
    let theme = theme.clone();
    let image = image.clone();
    let selection = selection.clone();
    move |_, context, width, height| {
      if let Some(theme) = theme.borrow().as_ref() {
        let preset = preset_for_card(kind, selection.get());
        let _ = context.save();
        let scale = (f64::from(width) / 960.0).min(f64::from(height) / 540.0);
        context.translate(
          (f64::from(width) - 960.0 * scale) / 2.0,
          (f64::from(height) - 540.0 * scale) / 2.0,
        );
        context.scale(scale, scale);
        render::paint_preset(
          context,
          theme,
          image.borrow().as_ref(),
          preset.variant == Variant::Spaced,
          preset.windows == WindowMode::Floating,
          preset.position == TaskbarPosition::Bottom,
        );
        let _ = context.restore();
      }
    }
  });
  let overlay = gtk::Overlay::new();
  overlay.set_child(Some(&drawing));
  let cover = gtk::Box::new(gtk::Orientation::Vertical, 8);
  cover.add_css_class("applying-preview");
  let spinner = gtk::Spinner::new();
  spinner.set_size_request(20, 20);
  spinner.set_halign(gtk::Align::Center);
  spinner.set_valign(gtk::Align::Center);
  cover.set_halign(gtk::Align::Fill);
  cover.set_valign(gtk::Align::Fill);

  let progress_center = gtk::CenterBox::new();
  progress_center.set_orientation(gtk::Orientation::Vertical);
  progress_center.set_vexpand(true);
  progress_center.set_center_widget(Some(&spinner));
  cover.append(&progress_center);
  cover.set_visible(false);
  overlay.add_overlay(&cover);

  let title = card_title(kind, i18n);
  let (primary, secondary) = title.split_once(" · ").unwrap_or((&title, ""));
  let label = gtk::Label::new(Some(primary));
  label.set_xalign(0.0);
  label.add_css_class("mode-card-title");
  label.set_wrap(true);
  label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  label.set_lines(2);
  label.set_ellipsize(gtk::pango::EllipsizeMode::End);
  label.set_width_chars(1);
  label.set_justify(gtk::Justification::Center);
  let child = gtk::Box::new(gtk::Orientation::Vertical, 8);
  let button = gtk::Button::new();
  let caption = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  caption.append(&mode_icon(12));
  let labels = gtk::Box::new(gtk::Orientation::Vertical, 3);
  labels.set_hexpand(true);
  labels.append(&label);
  caption.append(&labels);
  let indicator = if matches!(kind, CardKind::WindowTaskbar(_)) {
    let position = gtk::Label::new(Some(secondary));
    position.set_wrap(true);
    position.set_xalign(0.0);
    position.add_css_class("mode-muted");
    labels.append(&position);
    let indicator = gtk::Label::new(None);
    indicator.set_valign(gtk::Align::Center);
    caption.append(&indicator);
    child.append(&overlay);
    button.add_css_class("mode-choice");
    Some(indicator)
  } else {
    button.add_css_class("mode-segment");
    None
  };
  child.append(&caption);
  button.set_child(Some(&child));
  button.set_tooltip_text(Some(&title));
  button.set_hexpand(true);
  button.set_focusable(true);
  Card {
    kind,
    button,
    drawing,
    cover,
    spinner,
    indicator,
  }
}

fn connect_card(
  card: &Card,
  cards: &[Card],
  selection: &Rc<Cell<Selection>>,
  status: &gtk::Label,
  refresh_heading: &Rc<impl Fn(Variant) + 'static>,
) {
  card.button.connect_clicked({
    let card = card.clone();
    let cards = cards.to_vec();
    let selection = selection.clone();
    let status = status.clone();
    let refresh_heading = refresh_heading.clone();
    move |_| {
      let next = match card.kind {
        CardKind::Variant(variant) => {
          let mut next = selection.get();
          next.variant = variant;
          next.focus = FocusGroup::Modes;
          next
        }
        CardKind::WindowTaskbar(choice) => selection.get().choose(
          WindowTaskbarChoice::ALL
            .iter()
            .position(|candidate| {
              candidate.windows == choice.windows && candidate.position == choice.position
            })
            .unwrap_or(0),
        ),
      };
      selection.set(next);
      refresh_heading(next.variant);
      card.button.grab_focus();
      refresh_cards(&cards, next);
      status.set_visible(false);
    }
  });
}

fn connect_monitor(
  window: &gtk::ApplicationWindow,
  monitor: Option<&str>,
  cards: &[Card],
  grid: &gtk::Grid,
  columns: &Rc<Cell<usize>>,
  scroll: &gtk::ScrolledWindow,
) {
  let Some(display) = gdk::Display::default() else {
    return;
  };
  let outputs = display.monitors();
  let requested_is_available = monitor.is_some_and(|connector| {
    (0..outputs.n_items()).any(|index| {
      outputs
        .item(index)
        .and_downcast::<gdk::Monitor>()
        .is_some_and(|output| output.connector().as_deref() == Some(connector))
    })
  });
  for index in 0..outputs.n_items() {
    if let Some(output) = outputs.item(index).and_downcast::<gdk::Monitor>()
      && ((requested_is_available && monitor == output.connector().as_deref())
        || (!requested_is_available && index == 0))
    {
      window.set_monitor(Some(&output));
      let update = {
        let cards = cards
          .iter()
          .filter(|card| card.indicator.is_some())
          .cloned()
          .collect::<Vec<_>>();
        let grid = grid.clone();
        let columns = columns.clone();
        let scroll = scroll.clone();
        let window = window.downgrade();
        move |output: &gdk::Monitor| {
          let geometry = output.geometry();
          let count = grid_columns(geometry.width());
          columns.set(count);
          let preview = ((geometry.width() - 140) / count as i32).clamp(40, 440);
          if let Some(window) = window.upgrade() {
            if geometry.height() < 650 {
              window.add_css_class("compact-output");
            } else {
              window.remove_css_class("compact-output");
            }
          }
          scroll.set_max_content_height((geometry.height() - 290).max(90));
          let focused = cards.iter().position(|card| card.button.has_focus());
          for card in &cards {
            if card.button.parent().is_some() {
              grid.remove(&card.button);
            }
          }
          for (index, card) in cards.iter().enumerate() {
            grid.attach(
              &card.button,
              (index % count) as i32,
              (index / count) as i32,
              1,
              1,
            );
            card.drawing.set_content_width(preview.min(100));
            card.drawing.set_content_height(preview * 9 / 16);
            if focused == Some(index) {
              card.button.grab_focus();
            }
          }
        }
      };
      update(&output);
      output.connect_geometry_notify(update);
      let weak = window.downgrade();
      output.connect_invalidate(move |_| {
        if let Some(window) = weak.upgrade() {
          window.close();
        }
      });
      break;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn vertical_keys_select_modes_without_changing_submodes() {
    for preset in LayoutPreset::ALL {
      let selection = Selection::from_preset(preset);
      let sticky = selection.move_mode(gdk::Key::Up);
      assert_eq!(sticky.variant, Variant::Sticky);
      assert_eq!(sticky.windows, preset.windows);
      assert_eq!(sticky.position, preset.position);
      let spaced = selection.move_mode(gdk::Key::Down);
      assert_eq!(spaced.variant, Variant::Spaced);
      assert_eq!(spaced.windows, preset.windows);
      assert_eq!(spaced.position, preset.position);
    }
  }

  #[test]
  fn arrows_follow_focus_and_grid_boundaries() {
    let initial = Selection::from_preset(LayoutPreset::ALL[0]);
    assert_eq!(initial.focus, FocusGroup::Modes);
    assert_eq!(
      initial.navigate(gdk::Key::Right, 2).variant,
      Variant::Spaced
    );
    let selected = initial.choose(0);
    assert_eq!(selected.navigate(gdk::Key::Left, 2).cursor, 0);
    assert_eq!(selected.navigate(gdk::Key::Up, 2).cursor, 0);
    assert_eq!(selected.navigate(gdk::Key::Right, 2).cursor, 1);
    assert_eq!(selected.navigate(gdk::Key::Down, 2).cursor, 2);
    for index in 0..4 {
      let selection = initial.choose(index);
      for key in [
        gdk::Key::Up,
        gdk::Key::Down,
        gdk::Key::Left,
        gdk::Key::Right,
      ] {
        let next = selection.navigate(key, 2);
        assert_eq!(next.variant, selection.variant);
        assert_eq!(next.focus, FocusGroup::Submodes);
        assert!(next.cursor < 4);
      }
    }
  }

  #[test]
  fn every_card_preserves_unrelated_dimensions_and_covers_all_presets() {
    for preset in LayoutPreset::ALL {
      let selection = Selection::from_preset(preset);
      assert_eq!(LayoutPreset::ALL[selection.index()], preset);
      assert_eq!(
        preset_for_card(CardKind::Variant(Variant::Spaced), selection).windows,
        preset.windows
      );
      for (index, choice) in WindowTaskbarChoice::ALL.into_iter().enumerate() {
        let selected = selection.choose(index);
        assert_eq!(selected.windows, choice.windows);
        assert_eq!(selected.position, choice.position);
        assert_eq!(selected.variant, preset.variant);
        assert_eq!(
          preset_for_card(CardKind::WindowTaskbar(choice), selection),
          selected.preset()
        );
      }
    }
  }

  #[test]
  fn tab_cycles_without_changing_the_preset() {
    let mut selection = Selection::from_preset(LayoutPreset::ALL[0]);
    let preset = selection.preset();
    for group in [FocusGroup::Submodes, FocusGroup::Modes] {
      selection.focus = selection.focus.next();
      assert_eq!(selection.focus, group);
      assert_eq!(selection.preset(), preset);
    }
  }

  #[test]
  fn wide_grid_uses_a_single_row() {
    assert_eq!(grid_columns(1199), 2);
    assert_eq!(grid_columns(1200), 4);
    let selected = Selection::from_preset(LayoutPreset::ALL[0]).choose(2);
    assert_eq!(selected.navigate(gdk::Key::Down, 4), selected);
    assert_eq!(selected.navigate(gdk::Key::Right, 4).cursor, 3);
  }
}
