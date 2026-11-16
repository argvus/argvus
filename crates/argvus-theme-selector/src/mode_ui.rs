//! Two-column layout-mode selector; configuration and application belong to argvus-config.
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
      windows: WindowMode::Floating,
      position: TaskbarPosition::Top,
    },
    Self {
      windows: WindowMode::Tile,
      position: TaskbarPosition::Bottom,
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
enum FocusColumn {
  Modes,
  Submodes,
}

impl FocusColumn {
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
  focus: FocusColumn,
}

impl Selection {
  fn from_preset(preset: LayoutPreset) -> Self {
    Self {
      variant: preset.variant,
      windows: preset.windows,
      position: preset.position,
      cursor: WindowTaskbarChoice::from_preset(preset),
      focus: FocusColumn::Submodes,
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
      focus: FocusColumn::Submodes,
      ..self
    }
  }

  fn move_right(self, key: gdk::Key) -> Self {
    let cursor = match key {
      gdk::Key::Left => self.cursor.saturating_sub(1),
      gdk::Key::Right => (self.cursor + 1).min(WindowTaskbarChoice::ALL.len() - 1),
      _ => self.cursor,
    };
    self.choose(cursor)
  }

  fn move_mode(self, key: gdk::Key) -> Self {
    let mut selected = self;
    match key {
      gdk::Key::Up => selected.variant = Variant::Sticky,
      gdk::Key::Down => selected.variant = Variant::Spaced,
      _ => {}
    }
    selected.focus = FocusColumn::Modes;
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
}

struct State {
  theme: ThemeManifest,
  saved: usize,
  selection: Selection,
  image: Option<preview::Image>,
  pending: bool,
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
  Ok(State {
    theme,
    saved,
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

fn preview_height(width: i32, height: i32) -> i32 {
  let card_width = preview_width(width);
  (card_width * 9 / 16)
    .max(24)
    .min(((height - 250) / 2).max(24))
}

fn preview_width(width: i32) -> i32 {
  // Three previews, three button borders and padding, column gaps, and outer margins.
  ((width - 180) / 3).clamp(1, 840)
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

fn card_opacity(kind: CardKind, selection: Selection) -> f64 {
  if matches!(kind, CardKind::Variant(variant) if variant != selection.variant) {
    0.4
  } else {
    1.0
  }
}

fn refresh_cards(
  cards: &[Card],
  selection: Selection,
  theme: Option<&ThemeManifest>,
  provider: &gtk::CssProvider,
) {
  provider.load_from_string(&render::css(theme, selection.variant == Variant::Spaced));
  for card in cards {
    let selected = card_is_selected(card.kind, selection);
    card.button.set_opacity(card_opacity(card.kind, selection));
    if selected {
      card.button.add_css_class("mode-selected");
    } else {
      card.button.remove_css_class("mode-selected");
    }
    let has_cursor = matches!(card.kind, CardKind::Variant(_))
      && selection.focus == FocusColumn::Modes
      && selected
      || matches!(card.kind, CardKind::WindowTaskbar(_))
        && selection.focus == FocusColumn::Submodes
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
  provider.load_from_string(&render::css(None, false));
  gtk::style_context_add_provider_for_display(
    &gtk::prelude::WidgetExt::display(&window),
    &provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );
  window.add_css_class("theme-selector");

  let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
  content.set_valign(gtk::Align::Center);
  content.set_margin_start(24);
  content.set_margin_end(24);
  let title = gtk::Label::new(Some(&format!("ARGVUS · {}", i18n.tr("layout_mode.select"))));
  title.add_css_class("title");
  content.append(&title);

  let selection = Rc::new(Cell::new(Selection::from_preset(LayoutPreset::ALL[0])));
  let theme = Rc::new(RefCell::new(None::<ThemeManifest>));
  let image: Surface = Rc::default();
  let busy = Rc::new(Cell::new(false));
  let mut cards = Vec::with_capacity(6);
  let layout = gtk::Box::new(gtk::Orientation::Horizontal, 18);
  layout.set_halign(gtk::Align::Center);
  layout.set_direction(gtk::TextDirection::Ltr);
  let mode_column = gtk::Box::new(gtk::Orientation::Vertical, 12);
  let mode_cards = [Variant::Sticky, Variant::Spaced].map(|variant| {
    let card = make_card(
      CardKind::Variant(variant),
      &i18n,
      &selection,
      &theme,
      &image,
    );
    mode_column.append(&card.button);
    card
  });
  cards.extend(mode_cards.clone());

  let submode_grid = gtk::Grid::new();
  submode_grid.set_row_spacing(12);
  submode_grid.set_column_spacing(12);
  submode_grid.set_column_homogeneous(true);
  submode_grid.set_row_homogeneous(true);
  submode_grid.set_direction(gtk::TextDirection::Ltr);
  for (index, choice) in WindowTaskbarChoice::ALL.into_iter().enumerate() {
    let card = make_card(
      CardKind::WindowTaskbar(choice),
      &i18n,
      &selection,
      &theme,
      &image,
    );
    submode_grid.attach(&card.button, (index % 2) as i32, (index / 2) as i32, 1, 1);
    cards.push(card);
  }
  layout.append(&mode_column);
  layout.append(&submode_grid);
  content.append(&layout);

  let status = gtk::Label::new(Some(&i18n.tr("selector.loading")));
  status.set_wrap(true);
  content.append(&status);
  let footer = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  footer.set_halign(gtk::Align::Center);
  let apply = gtk::Button::with_label(&i18n.tr("layout_mode.apply"));
  apply.add_css_class("suggested-action");
  apply.set_sensitive(false);
  let close = gtk::Button::with_label(&i18n.tr("selector.close"));
  footer.append(&apply);
  footer.append(&close);
  content.append(&footer);
  let hint = gtk::Label::new(Some(&i18n.tr("layout_mode.hint")));
  hint.set_wrap(true);
  hint.add_css_class("muted");
  content.append(&hint);
  window.set_child(Some(&content));

  for card in &cards {
    connect_card(card, &cards, &selection, &theme, &provider, &status, &i18n);
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
    let theme = theme.clone();
    let provider = provider.clone();
    let status = status.clone();
    let apply = apply.clone();
    let close = close.clone();
    let weak = window.downgrade();
    let i18n = i18n.clone();
    move |_, key, _, _| {
      if busy.get() {
        return glib::Propagation::Stop;
      }
      match key {
        gdk::Key::Escape => {
          if let Some(window) = weak.upgrade() {
            window.close();
          }
        }
        gdk::Key::Tab | gdk::Key::ISO_Left_Tab => {
          let mut current = selection.get();
          current.focus = current.focus.next();
          selection.set(current);
          refresh_cards(&cards, current, theme.borrow().as_ref(), &provider);
          let focused_card = match current.focus {
            FocusColumn::Modes => cards.iter().find(
              |card| matches!(card.kind, CardKind::Variant(variant) if variant == current.variant),
            ),
            FocusColumn::Submodes => cards.iter().find(|card| {
              matches!(card.kind, CardKind::WindowTaskbar(_))
                && option_kind_for_card(card.kind) == Some(current.cursor)
            }),
          };
          if let Some(card) = focused_card {
            card.button.grab_focus();
          }
        }
        gdk::Key::Up | gdk::Key::Down => {
          let current = selection.get().move_mode(key);
          selection.set(current);
          refresh_cards(&cards, current, theme.borrow().as_ref(), &provider);
          status.set_text(&i18n.tr("layout_mode.selection"));
          if let Some(card) = cards.iter().find(
            |card| matches!(card.kind, CardKind::Variant(variant) if variant == current.variant),
          ) {
            card.button.grab_focus();
          }
        }
        gdk::Key::Left | gdk::Key::Right => {
          let current = selection.get().move_right(key);
          selection.set(current);
          refresh_cards(&cards, current, theme.borrow().as_ref(), &provider);
          status.set_text(&i18n.tr("layout_mode.selection"));
          if let Some(card) = cards.iter().find(|card| {
            option_kind_for_card(card.kind) == Some(current.cursor)
              && matches!(card.kind, CardKind::WindowTaskbar(_))
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
          connect_monitor(&window, monitor.as_deref(), &cards);
          match result {
            Ok(state) => {
              selection.set(state.selection);
              refresh_cards(&cards, state.selection, Some(&state.theme), &provider);
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
              for card in &cards {
                card.drawing.queue_draw();
              }
              status.set_text(&i18n.tr(if state.pending {
                "layout_mode.pending"
              } else if state.saved == state.selection.index() {
                "selector.illustrative"
              } else {
                "layout_mode.selection"
              }));
              apply.set_sensitive(true);
              window.present();
              if let Some(card) = cards.iter().find(|card| {
                option_kind_for_card(card.kind) == Some(state.selection.cursor)
                  && matches!(card.kind, CardKind::WindowTaskbar(_))
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
        context.scale(f64::from(width) / 960.0, f64::from(height) / 540.0);
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
  spinner.set_size_request(24, 24);
  cover.set_halign(gtk::Align::Fill);
  cover.set_valign(gtk::Align::Fill);
  cover.set_homogeneous(true);
  cover.append(&spinner);
  cover.set_visible(false);
  overlay.add_overlay(&cover);

  let label = gtk::Label::new(Some(&card_title(kind, i18n)));
  label.set_ellipsize(gtk::pango::EllipsizeMode::End);
  label.set_width_chars(1);
  label.set_justify(gtk::Justification::Center);
  let child = gtk::Box::new(gtk::Orientation::Vertical, 8);
  child.append(&overlay);
  child.append(&label);
  let button = gtk::Button::new();
  button.set_child(Some(&child));
  button.add_css_class("mode-choice");
  button.set_tooltip_text(Some(&card_title(kind, i18n)));
  button.set_hexpand(true);
  button.set_focusable(true);
  Card {
    kind,
    button,
    drawing,
    cover,
    spinner,
  }
}

fn connect_card(
  card: &Card,
  cards: &[Card],
  selection: &Rc<Cell<Selection>>,
  theme: &Rc<RefCell<Option<ThemeManifest>>>,
  provider: &gtk::CssProvider,
  status: &gtk::Label,
  i18n: &Arc<I18n>,
) {
  card.button.connect_clicked({
    let card = card.clone();
    let cards = cards.to_vec();
    let selection = selection.clone();
    let theme = theme.clone();
    let provider = provider.clone();
    let status = status.clone();
    let i18n = i18n.clone();
    move |_| {
      let next = match card.kind {
        CardKind::Variant(variant) => {
          let mut next = selection.get();
          next.variant = variant;
          next.focus = FocusColumn::Modes;
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
      card.button.grab_focus();
      refresh_cards(&cards, next, theme.borrow().as_ref(), &provider);
      status.set_text(&i18n.tr("layout_mode.selection"));
    }
  });
}

fn connect_monitor(window: &gtk::ApplicationWindow, monitor: Option<&str>, cards: &[Card]) {
  let Some(display) = gdk::Display::default() else {
    return;
  };
  let outputs = display.monitors();
  for index in 0..outputs.n_items() {
    if let Some(output) = outputs.item(index).and_downcast::<gdk::Monitor>()
      && (monitor == output.connector().as_deref() || (monitor.is_none() && index == 0))
    {
      window.set_monitor(Some(&output));
      let update = {
        let drawings = cards
          .iter()
          .map(|card| card.drawing.clone())
          .collect::<Vec<_>>();
        move |output: &gdk::Monitor| {
          let geometry = output.geometry();
          let height = preview_height(geometry.width(), geometry.height());
          for drawing in &drawings {
            drawing.set_content_width(preview_width(geometry.width()));
            drawing.set_content_height(height);
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
  fn horizontal_navigation_selects_unique_window_and_taskbar_pairs() {
    let selection = Selection::from_preset(LayoutPreset::ALL[0]);
    let mut current = selection;
    let mut seen = Vec::new();
    for _ in 0..4 {
      seen.push((current.windows, current.position));
      current = current.move_right(gdk::Key::Right);
    }
    assert_eq!(seen.len(), 4);
    assert_eq!(seen[0], (WindowMode::Tile, TaskbarPosition::Top));
    assert_eq!(seen[1], (WindowMode::Floating, TaskbarPosition::Top));
    assert_eq!(seen[2], (WindowMode::Tile, TaskbarPosition::Bottom));
    assert_eq!(seen[3], (WindowMode::Floating, TaskbarPosition::Bottom));
    assert_eq!(current, seen_selection(selection, 3));
  }

  fn seen_selection(mut selection: Selection, index: usize) -> Selection {
    for _ in 0..index {
      selection = selection.move_right(gdk::Key::Right);
    }
    selection
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
  fn inactive_mode_is_dimmed_and_tab_does_not_change_selected_values() {
    let selection = Selection::from_preset(LayoutPreset::ALL[0]);
    assert_eq!(
      card_opacity(CardKind::Variant(Variant::Sticky), selection),
      1.0
    );
    assert_eq!(
      card_opacity(CardKind::Variant(Variant::Spaced), selection),
      0.4
    );
    assert_eq!(
      card_opacity(
        CardKind::WindowTaskbar(WindowTaskbarChoice::ALL[0]),
        selection
      ),
      1.0
    );

    let focused = Selection {
      focus: selection.focus.next(),
      ..selection
    };
    assert_eq!(focused.variant, selection.variant);
    assert_eq!(focused.windows, selection.windows);
    assert_eq!(focused.position, selection.position);
    assert_eq!(focused.cursor, selection.cursor);
  }

  #[test]
  fn two_preview_rows_fit_small_and_large_outputs() {
    for (width, height) in [
      (640, 480),
      (800, 600),
      (1280, 720),
      (1920, 1080),
      (2560, 1440),
    ] {
      let preview = preview_height(width, height);
      assert!(preview >= 24);
      assert!(2 * preview + 250 <= height);
    }
  }
}
