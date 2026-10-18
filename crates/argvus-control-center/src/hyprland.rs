//! "Hyprland" category: a router page that reuses pages already owned by
//! the `settings` and `appearance` crates (Shortcuts, Window rules, Window
//! Spaces, Animations, Blur, Borders), plus a readonly Hyprland version
//! line.
//!
//! Every row here is a submenu into a page owned by another crate; none of
//! them read or mirror that crate's state in place, so navigating into
//! Animations/Blur/Window Spaces/Borders always lands on the real page that
//! owns the toggle/value and reloads correctly on `Apply`/immediate change.
//!
//! This page is new, so it uses the shared `argvus-tui` menu components
//! directly instead of the Home's older `HomeRow` list.
use crate::app::{App, Route};
#[cfg(feature = "appearance")]
use argvus_control_center_appearance::AppearancePage;
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_settings::Page;
use argvus_i18n::tr;
use argvus_tui::{
  hints::{HintContext, hints},
  icons,
  menu::{MenuEvent, MenuState, MenuStyle, Row, draw_menu},
  page::shell,
};
use crossterm::event::KeyCode;
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
  Shortcuts,
  #[cfg(feature = "apps")]
  WindowRules,
  #[cfg(feature = "appearance")]
  WindowSpaces,
  #[cfg(feature = "appearance")]
  Animations,
  /// Global blur (not a per-surface transparency/blur section).
  #[cfg(feature = "appearance")]
  Blur,
  #[cfg(feature = "appearance")]
  Borders,
  Info,
}

/// Whether the page shows its main list or the "Info" sub-page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum View {
  #[default]
  List,
  Info,
}

/// State kept by the Hyprland category across frames.
#[derive(Debug, Default)]
pub struct HyprlandState {
  menu: MenuState,
  list_height: u16,
  view: View,
  /// `hyprctl version`'s first field, read once at startup.
  pub version: String,
}

impl HyprlandState {
  /// Constructs `new` with the Hyprland version read once at startup (see
  /// [`read_version`]), or empty when `hyprctl` is unavailable.
  pub fn new(version: String) -> Self {
    Self {
      version,
      ..Default::default()
    }
  }
}

/// Reads the first field of `hyprctl version` (e.g. `0.56.2`), the same
/// value `argvus-widget-telemetry`'s `machine-info.sh` shows as the
/// Hyprland version. Runs synchronously once at startup, mirroring how
/// `argvus-about` reads `pacman -Q` for its own version rows.
pub fn read_version() -> Option<String> {
  use argvus_control_center_core::process::{ProcessRequest, ProcessRunner, SystemProcessRunner};
  use std::time::Duration;
  let request = ProcessRequest::new("hyprctl")
    .arg("version")
    .timeout(Duration::from_secs(2));
  let output = SystemProcessRunner.run(&request).ok()?;
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .next()?
    .split_whitespace()
    .nth(1)
    .map(str::to_string)
}

/// Rows of the main Hyprland list, in display order. Every row is a plain
/// submenu; the pages they open own their own state and apply mechanism.
fn rows(app: &App) -> Vec<Row<Item>> {
  let label = |key: &str| tr(app.lang, key);
  let mut rows = vec![
    Row::submenu(Item::Shortcuts, label("control_center.keyboard_shortcuts")).icon(icons::KEYBOARD),
  ];
  #[cfg(feature = "apps")]
  rows.push(
    Row::submenu(Item::WindowRules, label("control_center.window_rules")).icon(icons::WINDOW_RULES),
  );
  #[cfg(feature = "appearance")]
  {
    rows.push(
      Row::submenu(Item::WindowSpaces, label("control_center.window_spaces"))
        .icon(icons::WINDOW_STICKY),
    );
    rows.push(
      Row::submenu(Item::Animations, label("control_center.animations")).icon(icons::ANIMATION),
    );
    rows.push(Row::submenu(Item::Blur, label("control_center.blur")).icon(icons::BLUR));
    rows.push(Row::submenu(Item::Borders, label("control_center.borders")).icon(icons::BORDER));
  }
  rows.push(Row::submenu(Item::Info, label("control_center.info")).icon(icons::INFO));
  rows
}

/// The single readonly row of the "Info" sub-page.
fn info_rows(app: &App) -> Vec<Row<Item>> {
  vec![Row::info(
    tr(app.lang, "control_center.hyprland_version"),
    if app.hyprland.version.is_empty() {
      tr(app.lang, "control_center.not_available").to_string()
    } else {
      app.hyprland.version.clone()
    },
  )]
}

/// Breadcrumb of the current Hyprland view.
pub fn breadcrumb(app: &App) -> String {
  let root = format!(
    "{} > {}",
    HomeCardId::DesktopInput.label(app.lang),
    tr(app.lang, "control_center.hyprland")
  );
  match app.hyprland.view {
    View::List => root,
    View::Info => format!("{root} > {}", tr(app.lang, "control_center.info")),
  }
}

fn footer(app: &App, rows: &[Row<Item>]) -> String {
  match app.hyprland.view {
    View::Info => hints(
      app.lang,
      &HintContext {
        can_go_back: true,
        ..HintContext::default()
      },
    ),
    View::List => hints(
      app.lang,
      &HintContext {
        row: app.hyprland.menu.selected_kind(rows),
        can_go_back: true,
        ..HintContext::default()
      },
    ),
  }
}

/// Processes a key press. Returns `true` when the category should close and
/// return to the Home grid.
pub fn handle(app: &mut App, key: KeyCode) -> bool {
  if app.hyprland.view == View::Info {
    if matches!(key, KeyCode::Esc | KeyCode::Left) {
      app.hyprland.view = View::List;
    }
    return false;
  }
  let rows = rows(app);
  let mut menu = app.hyprland.menu;
  menu.normalize(&rows);
  let event = menu.handle(key, &rows, usize::from(app.hyprland.list_height));
  app.hyprland.menu = menu;
  match event {
    MenuEvent::Back => return true,
    MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item) => {
      activate(app, item);
    }
    MenuEvent::Adjust(_, _) | MenuEvent::Moved | MenuEvent::None => {}
  }
  false
}

fn activate(app: &mut App, item: Item) {
  match item {
    Item::Shortcuts => {
      app.return_route = Route::Hyprland;
      app.open_settings(Page::Keybindings);
    }
    #[cfg(feature = "apps")]
    Item::WindowRules => {
      app.return_route = Route::Hyprland;
      app.open_settings(Page::WindowRules);
    }
    #[cfg(feature = "appearance")]
    Item::WindowSpaces => open_appearance_page(app, AppearancePage::WindowSpaces),
    #[cfg(feature = "appearance")]
    Item::Animations => open_appearance_page(app, AppearancePage::Animations),
    #[cfg(feature = "appearance")]
    Item::Blur => open_appearance_page(app, AppearancePage::Blur),
    #[cfg(feature = "appearance")]
    Item::Borders => open_appearance_page(app, AppearancePage::Borders),
    Item::Info => app.hyprland.view = View::Info,
  }
}

#[cfg(feature = "appearance")]
fn open_appearance_page(app: &mut App, page: AppearancePage) {
  app.return_route = Route::Hyprland;
  app.appearance.page = page;
  app.route = Route::Appearance;
  app.appearance.reload();
}

/// Renders the current Hyprland view.
pub fn draw(app: &mut App, frame: &mut Frame) {
  let rows = match app.hyprland.view {
    View::List => rows(app),
    View::Info => info_rows(app),
  };
  let footer_text = footer(app, &rows);
  let area = shell(
    frame,
    frame.area(),
    &app.theme,
    app.lang,
    &breadcrumb(app),
    &footer_text,
  );
  app.hyprland.list_height = area.height;
  let mut menu = app.hyprland.menu;
  menu.normalize(&rows);
  draw_menu(
    frame,
    area,
    &app.theme,
    &rows,
    &mut menu,
    MenuStyle {
      icons: argvus_control_center_core::config::AppConfig::icons_enabled(),
    },
  );
  if app.hyprland.view == View::List {
    app.hyprland.menu = menu;
  }
}
