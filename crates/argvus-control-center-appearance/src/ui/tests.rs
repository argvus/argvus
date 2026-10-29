use super::drafts::recreates_surface;
use super::*;
use crate::animations::{
  AnimationCurve, AnimationField, AnimationGroup, AnimationLeaf, AnimationStyle,
};
use crate::model::{
  ControlPanelCard, IconThemeEntry, IconThemeSource, IconThemeVariant, TaskbarPosition,
  TaskbarUtilityWidget, WallpaperCollection, WallpaperMode, WidgetTelemetryBlock,
};
use argvus_tui::{
  components::{StatusKind, StatusMessage},
  icons,
  menu::RowKind,
};
use ratatui::{Terminal, backend::TestBackend};

mod animation_pages;
mod drafts;
mod icon_pages;
mod loading;
mod taskbar_pages;
mod theme_pages;

fn app(page: AppearancePage) -> AppearanceApp {
  AppearanceApp {
    page,
    loaded: true,
    ..AppearanceApp::new(Lang::for_locale("en-US"), Theme::load())
  }
}

/// A page whose draft was loaded the way `go` loads it.
fn draft_app(page: AppearancePage) -> AppearanceApp {
  let mut application = app(page);
  let surface = AppearanceApp::surface_for_page(page).expect("draft page");
  application.surface_draft = Some(application.make_surface_draft(surface));
  application
}

fn labels(application: &AppearanceApp) -> Vec<String> {
  application
    .rows()
    .iter()
    .map(|row| row.label().to_owned())
    .collect()
}

/// Moves the cursor to `item`, failing when the row is not selectable.
fn select(application: &mut AppearanceApp, item: Item) {
  let rows = application.rows();
  application.menu.normalize(&rows);
  assert!(
    application.menu.select(&rows, &item),
    "{item:?} is not selectable on {:?}",
    application.page
  );
}

fn selected(application: &AppearanceApp) -> Option<Item> {
  let rows = application.rows();
  let mut menu = application.menu;
  menu.normalize(&rows);
  menu.selected_id(&rows)
}

fn render(application: &mut AppearanceApp, width: u16, height: u16) -> Vec<String> {
  let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
  terminal.draw(|frame| application.draw(frame)).unwrap();
  terminal
    .backend()
    .buffer()
    .content
    .chunks(usize::from(width))
    .map(|line| line.iter().map(|cell| cell.symbol()).collect())
    .collect()
}

fn line_with<'screen>(screen: &'screen [String], needle: &str) -> &'screen str {
  screen
    .iter()
    .find(|line| line.contains(needle))
    .unwrap_or_else(|| panic!("{needle} missing: {screen:#?}"))
}

#[test]
fn home_has_categories_and_spacing_is_nested() {
  // The Appearance root only groups Desktop and Application Style; the
  // principal Home lists both groups directly. Taskbar/Control Panel/Widget
  // Telemetry are entered from the Desktop Shell card, not from here.
  assert_eq!(app(AppearancePage::Home).page_rows().len(), 2);
  assert_eq!(app(AppearancePage::Desktop).page_rows().len(), 6);
  assert_eq!(app(AppearancePage::ApplicationStyle).page_rows().len(), 2);
  // Position/Spaces (moved from the removed "Spaces, Borders & Position"
  // hub) now live inside Taskbar, between Transparency and Icons. Height and
  // Border sit after Spaces.
  assert_eq!(app(AppearancePage::Taskbar).page_rows().len(), 8);
  // "Borders" is entered directly from the Hyprland category (crate
  // principal) and just groups the two pre-existing border screens.
  assert_eq!(app(AppearancePage::Borders).page_rows().len(), 3);
  assert_eq!(
    app(AppearancePage::TaskbarIcons).page_rows().len(),
    3 + TaskbarUtilityWidget::ALL.len() + 1
  );
  assert_eq!(app(AppearancePage::TaskbarDate).page_rows().len(), 1);
  assert_eq!(app(AppearancePage::TaskbarDateFormat).page_rows().len(), 4);
  assert_eq!(app(AppearancePage::TaskbarTime).page_rows().len(), 2);
  assert_eq!(app(AppearancePage::TaskbarTimeFormat).page_rows().len(), 2);
  assert_eq!(app(AppearancePage::WidgetTelemetry).page_rows().len(), 3);
  assert_eq!(app(AppearancePage::ControlPanel).page_rows().len(), 3);
  // Blur: the "Enable" switch, the "Values" heading and the seven parameters.
  assert_eq!(app(AppearancePage::Blur).page_rows().len(), 9);
  assert_eq!(app(AppearancePage::Terminal).page_rows().len(), 2);
}

#[test]
/// Entry points reached from the Hyprland category (crate principal):
/// Animations is its own page with a single "Enable" toggle, and Window
/// Spaces is a hub with "Inner"/"Outer" submenus instead of a flat list.
fn hyprland_entry_points_match_the_approved_design() {
  assert_eq!(app(AppearancePage::Animations).page_rows().len(), 8);
  assert_eq!(app(AppearancePage::WindowSpaces).page_rows().len(), 2);
  assert_eq!(app(AppearancePage::WindowSpacesInner).page_rows().len(), 1);
  assert_eq!(app(AppearancePage::WindowSpacesOuter).page_rows().len(), 4);

  let animations = app(AppearancePage::Animations).rows();
  assert_eq!(animations[0].kind(), RowKind::Toggle { on: true });

  let blur = app(AppearancePage::Blur).rows();
  assert_eq!(blur[0].kind(), RowKind::Toggle { on: true });
  assert_eq!(blur[1].kind(), RowKind::Separator, "Values heading");
  assert_eq!(blur[2].kind(), RowKind::Value { step: Some(1) });
  assert_eq!(blur[2].label(), "Size");
  assert_eq!(blur[2].detail_text(), Some("6"));
  assert_eq!(blur[5].detail_text(), Some("0.00"), "noise");
  assert_eq!(blur[6].detail_text(), Some("0.900000"), "contrast");
  assert_eq!(blur[7].detail_text(), Some("0.100000"), "vibrancy");
}

#[test]
fn rows_have_the_kind_of_what_they_do() {
  let home = app(AppearancePage::Home).rows();
  assert!(home.iter().all(|row| row.kind() == RowKind::Submenu));

  let position = app(AppearancePage::TaskbarPosition).rows();
  assert_eq!(position[0].kind(), RowKind::Choice { current: true });
  assert_eq!(position[1].kind(), RowKind::Choice { current: false });

  let spaces = app(AppearancePage::TaskbarSpaces).rows();
  assert!(
    spaces
      .iter()
      .all(|row| row.kind() == RowKind::Value { step: None })
  );

  let editor = app(AppearancePage::Blur).rows();
  assert_eq!(editor[2].kind(), RowKind::Value { step: Some(1) });
}

#[test]
fn every_navigable_row_owns_a_semantic_icon() {
  let root = app(AppearancePage::Home).rows();
  assert_eq!(root[0].icon_glyph(), Some(icons::WALLPAPER));
  assert_eq!(root[1].icon_glyph(), Some(icons::TERMINAL));

  let desktop = app(AppearancePage::Desktop).rows();
  let glyphs: Vec<_> = desktop.iter().map(|row| row.icon_glyph()).collect();
  assert_eq!(glyphs[0], Some(icons::PALETTE));
  assert_eq!(glyphs[2], Some(icons::ACCENT));
  let unique: std::collections::HashSet<_> = glyphs.iter().collect();
  assert_eq!(
    unique.len(),
    glyphs.len(),
    "Desktop siblings repeat an icon"
  );
  assert!(
    !glyphs
      .iter()
      .any(|glyph| matches!(*glyph, Some(icons::STORAGE | icons::INFO | icons::SUCCESS))),
    "wildcard icons are gone"
  );

  let mode = app(AppearancePage::Mode).rows();
  assert_eq!(mode[0].icon_glyph(), Some(icons::WINDOW_STICKY));
  assert_eq!(mode[1].icon_glyph(), Some(icons::WINDOW_FLOAT));

  let position = app(AppearancePage::TaskbarPosition).rows();
  assert_eq!(position[0].icon_glyph(), Some(icons::ARROW_UP));
  assert_eq!(position[1].icon_glyph(), Some(icons::ARROW_DOWN));
}

#[test]
fn data_lists_carry_no_icon_so_the_column_disappears() {
  let mut application = app(AppearancePage::WallpaperItems {
    collection: WallpaperCollection::Abstract,
    mode: WallpaperMode::Dark,
  });
  application.state.wallpapers = vec![crate::WallpaperEntry {
    path: "abstract/dark/one.jxl".into(),
    collection: WallpaperCollection::Abstract,
    mode: WallpaperMode::Dark,
  }];
  assert!(
    application
      .rows()
      .iter()
      .all(|row| row.icon_glyph().is_none())
  );
}

#[test]
fn surface_pages_keep_settings_nested_and_values_bounded() {
  let widget_sessions = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::WidgetTelemetry,
    section: SurfaceSection::Sessions,
  });
  assert_eq!(
    widget_sessions.page_rows().len(),
    WidgetTelemetryBlock::ALL.len()
  );
  let control_sessions = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::ControlPanel,
    section: SurfaceSection::Sessions,
  });
  assert_eq!(
    control_sessions.page_rows().len(),
    ControlPanelCard::ALL.len()
  );

  let mut application = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::Transparency,
  });
  application.surface_draft.as_mut().unwrap().transparency = 100;
  select(&mut application, Item::SectionValue);
  application.handle(KeyCode::Char('+'));
  application.handle(KeyCode::Right);
  assert_eq!(application.surface_draft().unwrap().transparency, 100);
  application.surface_draft.as_mut().unwrap().transparency = 0;
  application.handle(KeyCode::Char('-'));
  application.handle(KeyCode::Left);
  assert_eq!(application.surface_draft().unwrap().transparency, 0);
}

#[test]
fn control_panel_sessions_toggle_the_card_on_the_row() {
  // Regression: the cursor used to index `ControlPanelCard::ALL` while the
  // page lists only available cards, so with a hidden card the toggle hit
  // the card below the cursor.
  let mut application = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::ControlPanel,
    section: SurfaceSection::Sessions,
  });
  let draft = application.surface_draft.as_mut().unwrap();
  draft
    .control_panel_cards
    .set_available(ControlPanelCard::User, false);
  let target = ControlPanelCard::Notifications;
  let before = draft.control_panel_cards.enabled(target);
  let neighbour = draft
    .control_panel_cards
    .enabled(ControlPanelCard::Calendar);

  let rows = application.page_rows();
  assert_eq!(rows.len(), ControlPanelCard::ALL.len() - 1);
  assert_eq!(rows[0].id(), Some(&Item::PanelCard(target)));
  application.handle(KeyCode::Enter);

  let cards = &application.surface_draft().unwrap().control_panel_cards;
  assert_eq!(cards.enabled(target), !before);
  assert_eq!(cards.enabled(ControlPanelCard::Calendar), neighbour);
}

#[test]
fn home_rows_follow_the_global_icon_setting() {
  AppConfig::set_session_icons(true);
  let screen = render(&mut app(AppearancePage::Desktop), 80, 24);
  assert!(line_with(&screen, "Theme").contains(icons::PALETTE));
  let screen = render(&mut app(AppearancePage::ApplicationStyle), 80, 24);
  assert!(line_with(&screen, "Terminal").contains(icons::TERMINAL));

  AppConfig::set_session_icons(false);
  let screen = render(&mut app(AppearancePage::Desktop), 80, 24);
  assert!(!line_with(&screen, "Theme").contains(icons::PALETTE));
  let screen = render(&mut app(AppearancePage::ApplicationStyle), 80, 24);
  assert!(!line_with(&screen, "Terminal").contains(icons::TERMINAL));
  AppConfig::set_session_icons(true);
}

#[test]
fn home_renders_one_vertical_list_at_80_columns() {
  let screen = render(&mut app(AppearancePage::Desktop), 80, 24);
  let theme = line_with(&screen, "Theme");
  assert!(
    theme.contains("> "),
    "first row holds the cursor: {theme:?}"
  );
  assert!(theme.contains('›'), "submenus show the marker: {theme:?}");
  assert!(
    !screen.iter().any(|line| line.contains("[ ")),
    "no button labels"
  );
}

#[test]
fn selection_bounds_follow_each_page() {
  let mut application = app(AppearancePage::WindowSpacesOuter);
  application.handle(KeyCode::End);
  assert_eq!(
    selected(&application),
    Some(Item::Spacing(PromptGoal::GapsOutBottom))
  );
  application.handle(KeyCode::Down);
  assert_eq!(
    selected(&application),
    Some(Item::Spacing(PromptGoal::GapsOutBottom))
  );
}

#[test]
fn disabled_rounding_is_skipped_and_does_not_open_prompt() {
  let mut application = app(AppearancePage::GeneralBorders);
  assert!(!application.state.rounded);
  let rows = application.rows();
  assert!(!rows[1].is_selectable(), "Rounding is disabled");
  application.handle(KeyCode::End);
  assert_eq!(selected(&application), Some(Item::Rounded));
  application.activate(Item::Rounding);
  assert!(application.prompt_back.is_none());

  application.state.rounded = true;
  select(&mut application, Item::Rounding);
  application.handle(KeyCode::Enter);
  assert_eq!(
    application.page,
    AppearancePage::Prompt {
      goal: PromptGoal::Rounding
    }
  );
}

#[test]
fn space_keeps_activating_actions_submenus_choices_and_editors() {
  // Submenu: Space opens the page, as Enter.
  let mut style = app(AppearancePage::ApplicationStyle);
  select(&mut style, Item::Terminal);
  style.handle(KeyCode::Char(' '));
  assert_eq!(style.page, AppearancePage::Terminal);

  // Choice: Space applies the option immediately, as Enter.
  let mut position = app(AppearancePage::TaskbarPosition);
  select(&mut position, Item::Position(TaskbarPosition::Bottom));
  position.handle(KeyCode::Char(' '));
  assert!(position.action.is_some());

  // Action: Space opens the export prompt.
  let mut themes = app(AppearancePage::Themes);
  select(&mut themes, Item::ExportTheme);
  themes.handle(KeyCode::Char(' '));
  assert_eq!(
    themes.page,
    AppearancePage::Prompt {
      goal: PromptGoal::ExportProfile
    }
  );

  // Value without a step: Space opens its editor.
  let mut spaces = app(AppearancePage::TaskbarSpaces);
  select(&mut spaces, Item::Spacing(PromptGoal::WaybarLeft));
  spaces.handle(KeyCode::Char(' '));
  assert_eq!(
    spaces.page,
    AppearancePage::Prompt {
      goal: PromptGoal::WaybarLeft
    }
  );

  // Toggle: Space flips it (shared menu behavior).
  let mut time = draft_app(AppearancePage::TaskbarTime);
  let before = time.surface_draft().unwrap().time_seconds_enabled;
  time.handle(KeyCode::Char(' '));
  assert_eq!(time.surface_draft().unwrap().time_seconds_enabled, !before);
}

/// Blur value as the page currently shows it (draft first, then stored).
fn blur_value(application: &AppearanceApp, field: BlurField) -> f64 {
  application.blur_editor().values.get(field)
}

/// Shadow value as the page currently shows it (draft first, then stored).
fn shadow_value(application: &AppearanceApp, field: ShadowField) -> f64 {
  application.shadow_editor().values.get(field)
}

#[test]
fn shadow_page_starts_disabled_and_follows_the_accent() {
  let shadow = app(AppearancePage::BordersShadow).rows();
  assert_eq!(shadow[0].kind(), RowKind::Toggle { on: false }, "Enable");
  assert_eq!(shadow[1].kind(), RowKind::Toggle { on: false }, "Sharp");
  assert_eq!(
    shadow[2].detail_text(),
    Some("3590bd99"),
    "accent with alpha 99"
  );
  assert_eq!(shadow[3].kind(), RowKind::Separator, "Values heading");
  assert_eq!(shadow[4].label(), "Range");
  assert_eq!(shadow[4].detail_text(), Some("6"));
  assert_eq!(shadow[5].label(), "Render power");
  assert_eq!(shadow[5].detail_text(), Some("6"));
  assert_eq!(shadow[8].detail_text(), Some("1.00"), "scale");
}

#[test]
fn shadow_values_and_color_are_committed_to_the_draft_only() {
  let mut application = app(AppearancePage::BordersShadow);
  application.commit_shadow_value(ShadowField::RenderPower, "9");
  assert_eq!(shadow_value(&application, ShadowField::RenderPower), 9.0);
  assert!(
    application.has_pending_changes(),
    "Apply is enabled by the draft"
  );

  application.commit_shadow_value(ShadowField::RenderPower, "11");
  assert_eq!(
    shadow_value(&application, ShadowField::RenderPower),
    9.0,
    "out of range keeps the draft"
  );
  assert!(application.prompt_error.is_some());

  application.commit_shadow_color("aabbcc11");
  assert_eq!(
    application.shadow_editor().color,
    ShadowColor::Custom("aabbcc11".into())
  );
  application.commit_shadow_color("aabbcc");
  assert_eq!(
    application.shadow_editor().color,
    ShadowColor::Custom("aabbcc11".into()),
    "invalid color keeps the previous value"
  );
  assert!(
    application.state.shadow == ShadowSettings::default(),
    "stored state untouched"
  );
}

fn custom_theme_app() -> AppearanceApp {
  let mut custom = app(AppearancePage::CustomThemes);
  custom.state.custom_themes = vec![CustomTheme {
    id: "mine".into(),
    name: "Mine".into(),
  }];
  custom
}

/// One installed icon theme for the Icons page tests.
fn icon_theme(directory: &str, source: IconThemeSource, official: bool) -> IconThemeEntry {
  IconThemeEntry {
    directory_name: directory.into(),
    display_name: directory.into(),
    comment: None,
    source,
    official: official.then_some(IconThemeVariant::Default),
  }
}

#[test]
fn leaving_icons_with_an_unapplied_choice_asks_before_discarding_it() {
  let mut application = app(AppearancePage::Icons);
  application.state.icon_theme = "Papirus".into();
  application.state.icon_themes = vec![
    icon_theme("Argvus Icons", IconThemeSource::System, true),
    icon_theme("Papirus", IconThemeSource::User, false),
  ];

  application.handle(KeyCode::Char(' '));
  assert!(!application.handle(KeyCode::Esc));

  assert!(application.confirm.is_some());
  assert_eq!(application.page, AppearancePage::Icons);
}

/// Polls until `done` holds, failing after a generous bound so a stuck job
/// does not hang the suite.
fn poll_until(application: &mut AppearanceApp, done: impl Fn(&AppearanceApp) -> bool) {
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
  while !done(application) {
    assert!(std::time::Instant::now() < deadline, "job did not finish");
    application.poll();
    std::thread::sleep(std::time::Duration::from_millis(5));
  }
}
