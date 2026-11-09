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
  Windows,
  WindowMode,
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
  Windows,
  Info,
}

/// State kept by the Hyprland category across frames.
#[derive(Debug, Default)]
pub struct HyprlandState {
  menu: MenuState,
  windows_menu: MenuState,
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
  rows.push(
    Row::submenu(Item::Windows, label("control_center.hyprland_windows")).icon(icons::WINDOW_RULES),
  );
  #[cfg(feature = "appearance")]
  {
    rows.push(
      Row::submenu(Item::Animations, label("control_center.animations")).icon(icons::ANIMATION),
    );
    rows.push(Row::submenu(Item::Blur, label("control_center.blur")).icon(icons::BLUR));
    rows.push(Row::submenu(Item::Borders, label("control_center.borders")).icon(icons::BORDER));
  }
  rows.push(Row::submenu(Item::Info, label("control_center.info")).icon(icons::INFO));
  rows
}

fn windows_rows(app: &App) -> Vec<Row<Item>> {
  vec![
    #[cfg(feature = "apps")]
    Row::submenu(
      Item::WindowRules,
      tr(app.lang, "control_center.hyprland_windows_rules"),
    )
    .icon(icons::WINDOW_RULES),
    #[cfg(feature = "appearance")]
    Row::submenu(
      Item::WindowSpaces,
      tr(app.lang, "control_center.hyprland_windows_spaces"),
    )
    .icon(icons::WINDOW_STICKY),
    Row::submenu(
      Item::WindowMode,
      tr(app.lang, "control_center.hyprland_windows_mode"),
    )
    .icon(icons::LAYOUT),
  ]
}

/// Shared entry preparation for the menu, global search and direct CLI routes.
/// Separate menu states preserve selection when a child returns to Windows.
pub(crate) fn prepare_windows(app: &mut App, item: Option<Item>) {
  app.hyprland.view = View::Windows;
  let root = rows(app);
  app.hyprland.menu.select(&root, &Item::Windows);
  if let Some(item) = item {
    let children = windows_rows(app);
    app.hyprland.windows_menu.select(&children, &item);
  }
  app.return_route = Route::Hyprland;
}

pub(crate) fn open_windows(app: &mut App) {
  prepare_windows(app, None);
  app.return_route = Route::Home;
  app.route = Route::Hyprland;
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
    View::Windows => format!(
      "{root} > {}",
      tr(app.lang, "control_center.hyprland_windows")
    ),
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
    View::List | View::Windows => hints(
      app.lang,
      &HintContext {
        row: if app.hyprland.view == View::Windows {
          app.hyprland.windows_menu.selected_kind(rows)
        } else {
          app.hyprland.menu.selected_kind(rows)
        },
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
  let rows = if app.hyprland.view == View::Windows {
    windows_rows(app)
  } else {
    rows(app)
  };
  let mut menu = if app.hyprland.view == View::Windows {
    app.hyprland.windows_menu
  } else {
    app.hyprland.menu
  };
  menu.normalize(&rows);
  let event = menu.handle(key, &rows, usize::from(app.hyprland.list_height));
  if app.hyprland.view == View::Windows {
    app.hyprland.windows_menu = menu;
  } else {
    app.hyprland.menu = menu;
  }
  match event {
    MenuEvent::Back => {
      if app.hyprland.view == View::Windows {
        app.hyprland.view = View::List;
      } else {
        return true;
      }
    }
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
    Item::Windows => open_windows(app),
    Item::WindowMode => app.open_settings(Page::WindowMode),
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
pub(crate) fn open_appearance_page(app: &mut App, page: AppearancePage) {
  if matches!(
    page,
    AppearancePage::WindowSpaces
      | AppearancePage::WindowSpacesInner
      | AppearancePage::WindowSpacesOuter
  ) {
    prepare_windows(app, Some(Item::WindowSpaces));
  }
  app.return_route = Route::Hyprland;
  app.appearance.open(page);
  app.route = Route::Appearance;
}

/// Renders the current Hyprland view.
pub fn draw(app: &mut App, frame: &mut Frame) {
  let rows = match app.hyprland.view {
    View::List => rows(app),
    View::Windows => windows_rows(app),
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
  let mut menu = if app.hyprland.view == View::Windows {
    app.hyprland.windows_menu
  } else {
    app.hyprland.menu
  };
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
  match app.hyprland.view {
    View::List => app.hyprland.menu = menu,
    View::Windows => app.hyprland.windows_menu = menu,
    View::Info => {}
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::InitialRoute;

  #[test]
  fn windows_hub_and_mode_restore_both_menu_selections_on_back() {
    let mut app = App::with_lang(
      InitialRoute::Hyprland,
      argvus_i18n::Lang::for_locale("en-US"),
    );
    let root = rows(&app);
    assert!(root.iter().any(|row| row.id() == Some(&Item::Windows)));
    assert!(!root.iter().any(|row| row.id() == Some(&Item::WindowMode)));
    activate(&mut app, Item::Windows);
    assert!(breadcrumb(&app).ends_with("Hyprland > Windows"));
    activate(&mut app, Item::WindowMode);
    assert_eq!(app.settings.page(), Page::WindowMode);
    app.back();
    assert_eq!(app.route, Route::Hyprland);
    assert_eq!(app.hyprland.view, View::Windows);
    assert_eq!(
      app.hyprland.windows_menu.selected_id(&windows_rows(&app)),
      Some(Item::WindowMode)
    );
    app.back();
    assert_eq!(app.hyprland.view, View::List);
    assert_eq!(
      app.hyprland.menu.selected_id(&rows(&app)),
      Some(Item::Windows)
    );
    app.back();
    assert_eq!(app.route, Route::Home);
  }

  #[cfg(all(feature = "apps", feature = "appearance"))]
  #[test]
  fn windows_hub_orders_rules_spaces_and_mode() {
    let app = App::new(InitialRoute::HyprlandWindows);
    assert_eq!(
      windows_rows(&app)
        .iter()
        .filter_map(Row::id)
        .copied()
        .collect::<Vec<_>>(),
      vec![Item::WindowRules, Item::WindowSpaces, Item::WindowMode]
    );
    assert!(
      !rows(&app)
        .iter()
        .any(|row| matches!(row.id(), Some(Item::WindowRules | Item::WindowSpaces)))
    );
  }

  #[test]
  fn mode_search_and_cli_open_the_same_page_and_parent() {
    let mut app = App::new(InitialRoute::Home);
    app.open_search_route("hyprland/windows-mode");
    assert_eq!(app.settings.page(), Page::WindowMode);
    let direct = App::new(InitialRoute::Settings(Page::WindowMode));
    assert_eq!(app.settings.breadcrumb(), direct.settings.breadcrumb());
    assert_eq!(app.return_route, Route::Hyprland);
    assert_eq!(direct.return_route, Route::Hyprland);
    assert_eq!(direct.hyprland.view, View::Windows);
    assert!(
      app
        .search_registry
        .entries()
        .iter()
        .any(|entry| entry.id == "hyprland.window_mode")
    );
  }

  #[test]
  fn leaving_the_hub_does_not_change_unrelated_page_parents() {
    let mut app = App::new(InitialRoute::HyprlandWindows);
    app.back();
    app.back();
    assert_eq!(app.route, Route::Home);
    app.open_settings(Page::MouseTouchpad);
    app.back();
    assert_eq!(app.route, Route::Home);
  }

  #[cfg(feature = "appearance")]
  #[test]
  fn spaces_returns_to_windows_from_search_and_cli() {
    let mut app = App::new(InitialRoute::Home);
    app.open_search_route("hyprland/windows-spaces");
    assert_eq!(app.appearance.page, AppearancePage::WindowSpaces);
    app.back();
    assert_eq!(app.route, Route::Hyprland);
    assert_eq!(app.hyprland.view, View::Windows);
    let mut direct = App::new(InitialRoute::Appearance(AppearancePage::WindowSpaces));
    direct.back();
    assert_eq!(direct.route, Route::Hyprland);
    assert_eq!(direct.hyprland.view, View::Windows);
  }
}
