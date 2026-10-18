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
  // hub) now live inside Taskbar, between Transparency and Icons.
  assert_eq!(app(AppearancePage::Taskbar).page_rows().len(), 6);
  // "Borders" is entered directly from the Hyprland category (crate
  // principal) and just groups the two pre-existing border screens.
  assert_eq!(app(AppearancePage::Borders).page_rows().len(), 2);
  assert_eq!(
    app(AppearancePage::TaskbarIcons).page_rows().len(),
    2 + TaskbarUtilityWidget::ALL.len() + 1
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
fn mode_page_lists_sticky_and_float_independent_of_theme() {
  let rows = labels(&app(AppearancePage::Mode));
  assert_eq!(rows.len(), 2);
  assert!(rows[0].contains("Sticky"));
  assert!(rows[1].contains("Float"));

  let mut ui = app(AppearancePage::Desktop);
  assert_eq!(selected(&ui), Some(Item::Themes));
  select(&mut ui, Item::Mode);
  ui.handle(KeyCode::Enter);
  assert_eq!(ui.page, AppearancePage::Mode);
}

#[test]
fn take_theme_dirty_clears_after_first_read() {
  let mut ui = app(AppearancePage::Home);
  assert!(!ui.take_theme_dirty());
  ui.theme_dirty = true;
  assert!(ui.take_theme_dirty());
  assert!(!ui.take_theme_dirty());
}

#[test]
fn set_theme_replaces_the_semantic_palette() {
  let mut ui = app(AppearancePage::Home);
  let mut theme = Theme::load();
  theme.name = "marker-theme".into();
  ui.set_theme(&theme);
  assert_eq!(ui.theme.name, "marker-theme");
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
fn theme_rows_use_friendly_official_names() {
  let rows = labels(&app(AppearancePage::Themes));
  // Official, Custom, export profile, import profile.
  assert_eq!(rows.len(), 4);
  assert!(rows[0].contains("Official"));
  assert!(rows[1].contains("Custom"));
  assert!(rows[2].contains("Export theme profile"));
  assert!(rows[3].contains("Import theme profile"));
}

#[test]
fn official_theme_rows_expose_both_categories() {
  let rows = app(AppearancePage::OfficialThemes).rows();
  assert_eq!(
    rows.iter().map(|row| row.label()).collect::<Vec<_>>(),
    vec!["Dark", "Light"]
  );
  assert!(rows.iter().all(|row| row.kind() == RowKind::Submenu));
}

#[test]
fn wallpaper_rows_use_nerd_font_icon_and_nested_categories() {
  let rows = app(AppearancePage::Wallpapers).rows();
  assert_eq!(rows.len(), 3);
  assert!(rows[0].label().contains("Choose image from HOME"));
  assert_eq!(rows[0].kind(), RowKind::Action);
  assert_eq!(rows[0].icon_glyph(), Some(icons::FOLDER));
  assert!(!rows[0].label().contains("📂"));
  assert_eq!(
    rows[1].id(),
    Some(&Item::Collection(WallpaperCollection::Abstract))
  );
  assert_eq!(
    rows[2].id(),
    Some(&Item::Collection(WallpaperCollection::Landscape))
  );

  let modes = app(AppearancePage::WallpaperModes {
    collection: WallpaperCollection::Abstract,
  })
  .rows();
  assert_eq!(
    modes[0].id(),
    Some(&Item::WallpaperMode(WallpaperMode::Dark))
  );
  assert_eq!(
    modes[1].id(),
    Some(&Item::WallpaperMode(WallpaperMode::Light))
  );
}

#[test]
fn wallpaper_navigation_returns_from_items_to_collection() {
  let mut application = app(AppearancePage::Wallpapers);
  select(
    &mut application,
    Item::Collection(WallpaperCollection::Abstract),
  );
  application.handle(KeyCode::Enter);
  application.job = None;
  assert_eq!(
    application.page,
    AppearancePage::WallpaperModes {
      collection: WallpaperCollection::Abstract
    }
  );
  application.handle(KeyCode::Right);
  application.job = None;
  assert_eq!(
    application.page,
    AppearancePage::WallpaperItems {
      collection: WallpaperCollection::Abstract,
      mode: WallpaperMode::Dark
    }
  );
  application.handle(KeyCode::Left);
  assert_eq!(
    application.page,
    AppearancePage::WallpaperModes {
      collection: WallpaperCollection::Abstract
    }
  );
  application.handle(KeyCode::Esc);
  assert_eq!(application.page, AppearancePage::Wallpapers);
}

#[test]
fn wallpaper_items_filter_by_collection_and_mode() {
  let mut application = app(AppearancePage::WallpaperItems {
    collection: WallpaperCollection::Landscape,
    mode: WallpaperMode::Light,
  });
  application.state.wallpapers = vec![
    crate::WallpaperEntry {
      path: "abstract/dark/gruvbox-abstract-dark.jxl".into(),
      collection: WallpaperCollection::Abstract,
      mode: WallpaperMode::Dark,
    },
    crate::WallpaperEntry {
      path: "landscape/light/gruvbox-landscape-light.jxl".into(),
      collection: WallpaperCollection::Landscape,
      mode: WallpaperMode::Light,
    },
  ];
  application.state.wallpaper_active = Some("landscape/light/gruvbox-landscape-light.jxl".into());
  let rows = application.rows();
  assert_eq!(labels(&application), vec!["gruvbox-landscape-light.jxl"]);
  assert_eq!(
    rows[0].id(),
    Some(&Item::WallpaperFile(1)),
    "keeps its identity"
  );
  assert_eq!(rows[0].kind(), RowKind::Choice { current: true });
}

// TODO (Fase 3): Refactor to test with dynamic theme discovery
// Currently themes are discovered dynamically from /usr/share/argvus/appearance/themes.d/
// Test needs a temporary directory with sample theme.toml files to verify grouping
#[test]
fn theme_families_are_grouped_by_identifier_category() {
  let dark_rows = app(AppearancePage::ThemeFamilies {
    category: ThemeCategory::Dark,
  })
  .rows();
  let light_rows = app(AppearancePage::ThemeFamilies {
    category: ThemeCategory::Light,
  })
  .rows();
  // Without drop-in packages installed, only built-in themes appear
  assert_eq!(dark_rows.len(), 1); // argvus-dark
  assert_eq!(light_rows.len(), 1); // argvus-light
}

#[test]
fn builtin_themes_always_present() {
  let report = argvus_theme::discovery::discover_themes(std::path::Path::new("/tmp"));
  assert!(report.themes.iter().any(|t| t.id == "argvus-dark"));
  assert!(report.themes.iter().any(|t| t.id == "argvus-light"));
  // Verify categories are correct
  let dark = report
    .themes
    .iter()
    .find(|t| t.id == "argvus-dark")
    .unwrap();
  assert_eq!(dark.category, argvus_theme::discovery::ThemeCategory::Dark);
  let light = report
    .themes
    .iter()
    .find(|t| t.id == "argvus-light")
    .unwrap();
  assert_eq!(
    light.category,
    argvus_theme::discovery::ThemeCategory::Light
  );
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

#[test]
fn value_rows_adjust_with_arrows_and_footer_says_esc_goes_back() {
  let mut editor = app(AppearancePage::Blur);
  select(&mut editor, Item::BlurField(BlurField::Size));
  let rows = editor.rows();
  let footer = editor.hints(&rows);
  assert!(footer.contains("←/→ "), "{footer}");
  assert!(footer.contains("Esc "), "{footer}");
  assert!(!footer.contains("←/Esc"), "← adjusts here: {footer}");

  editor.handle(KeyCode::Right);
  assert_eq!(blur_value(&editor, BlurField::Size), 7.0);
  editor.handle(KeyCode::Left);
  editor.handle(KeyCode::Left);
  assert_eq!(blur_value(&editor, BlurField::Size), 5.0);
  assert_eq!(
    editor.page,
    AppearancePage::Blur,
    "← adjusted, did not leave"
  );
  editor.handle(KeyCode::Char('h'));
  editor.handle(KeyCode::Char('l'));
  editor.handle(KeyCode::Char('l'));
  assert_eq!(blur_value(&editor, BlurField::Size), 6.0);
  assert!(!editor.blur_draft_has_changes(), "back to the stored value");
}

/// Blur value as the page currently shows it (draft first, then stored).
fn blur_value(application: &AppearanceApp, field: BlurField) -> f64 {
  application.blur_editor().values.get(field)
}

#[test]
fn blur_switch_and_values_stay_in_the_draft_until_apply() {
  let mut blur = app(AppearancePage::Blur);
  select(&mut blur, Item::BlurEnabled);
  blur.handle(KeyCode::Enter);
  assert_eq!(
    blur.blur_draft.map(|draft| draft.enabled),
    Some(!blur.state.blur)
  );
  assert!(blur.has_pending_changes(), "Apply is enabled");
  assert!(blur.action.is_none(), "the switch is not written yet");

  select(&mut blur, Item::BlurField(BlurField::Passes));
  blur.handle(KeyCode::Right);
  assert_eq!(blur_value(&blur, BlurField::Passes), 3.0);
  assert_eq!(
    blur.state.blur_values.get(BlurField::Passes),
    2.0,
    "stored value unchanged"
  );
  assert!(blur.action.is_none(), "nothing applied");
}

#[test]
fn theme_shortcuts_are_kept_and_shown_in_the_footer() {
  let mut custom = app(AppearancePage::CustomThemes);
  custom.state.custom_themes = vec![CustomTheme {
    id: "mine".into(),
    name: "Mine".into(),
    base_theme: "argvus-dark".into(),
    profile_path: "/tmp/mine".into(),
    wallpaper_path: None,
  }];
  let rows = custom.rows();
  custom.menu.normalize(&rows);
  let footer = custom.hints(&rows);
  for key in ["d ", "e ", "i "] {
    assert!(footer.contains(key), "{key} missing: {footer}");
  }
  custom.handle(KeyCode::Char('d'));
  assert_eq!(custom.page, AppearancePage::CustomThemes);
  assert_eq!(
    custom.confirm.map(|(c, _)| c),
    Some(Confirmation::DeleteTheme)
  );
  assert_eq!(custom.delete_theme.as_ref().unwrap().id, "mine");

  let mut themes = app(AppearancePage::Themes);
  themes.handle(KeyCode::Char('i'));
  assert_eq!(themes.page, AppearancePage::ThemeImport);
}

#[test]
fn draft_pages_end_with_an_apply_row_enabled_only_with_changes() {
  let pages = [
    AppearancePage::Taskbar,
    AppearancePage::TaskbarIcons,
    AppearancePage::TaskbarDate,
    AppearancePage::TaskbarDateFormat,
    AppearancePage::TaskbarTime,
    AppearancePage::TaskbarTimeFormat,
    AppearancePage::WidgetTelemetry,
    AppearancePage::ControlPanel,
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::UtilityIcons,
    },
    AppearancePage::SurfaceSection {
      surface: EffectSurface::WidgetTelemetry,
      section: SurfaceSection::Sessions,
    },
    AppearancePage::SurfaceSection {
      surface: EffectSurface::ControlPanel,
      section: SurfaceSection::Transparency,
    },
  ];
  for page in pages {
    let mut application = draft_app(page);
    let rows = application.rows();
    let (apply, separator) = (&rows[rows.len() - 1], &rows[rows.len() - 2]);
    assert_eq!(apply.id(), Some(&Item::Apply), "{page:?}");
    assert_eq!(separator.kind(), RowKind::Separator, "{page:?}");
    assert_eq!(apply.icon_glyph(), Some(icons::APPLY));
    assert!(
      !apply.is_selectable(),
      "clean draft: Apply is skipped ({page:?})"
    );

    application
      .surface_draft
      .as_mut()
      .unwrap()
      .transparency_enabled ^= true;
    let rows = application.rows();
    let apply = rows.last().unwrap();
    assert!(
      apply.is_selectable(),
      "dirty draft: Apply is enabled ({page:?})"
    );
    assert!(
      apply.detail_text().is_some(),
      "pending indicator ({page:?})"
    );
  }

  for page in [
    AppearancePage::Blur,
    AppearancePage::TerminalTransparency,
    AppearancePage::TransparencySurface {
      surface: EffectSurface::Launchers,
    },
  ] {
    let mut application = app(page);
    if page == AppearancePage::Blur {
      select(&mut application, Item::BlurField(BlurField::Size));
    }
    assert!(!application.rows().last().unwrap().is_selectable());
    application.handle(KeyCode::Right);
    assert!(
      application.rows().last().unwrap().is_selectable(),
      "{page:?}"
    );
  }
}

#[test]
fn apply_row_runs_the_same_apply_and_keeps_the_page() {
  let mut application = draft_app(AppearancePage::TaskbarTime);
  select(&mut application, Item::Seconds);
  application.handle(KeyCode::Enter);
  application.handle(KeyCode::End);
  assert_eq!(selected(&application), Some(Item::Apply));
  application.handle(KeyCode::Enter);
  assert!(
    application.action.is_some(),
    "apply_surface_changes started"
  );
  assert_eq!(application.page, AppearancePage::TaskbarTime);

  // Effect editors apply and return where they did before.
  let mut terminal = app(AppearancePage::TerminalTransparency);
  terminal.handle(KeyCode::Left);
  select(&mut terminal, Item::Apply);
  terminal.handle(KeyCode::Enter);
  assert!(terminal.action.is_some(), "apply_effect_changes started");
  assert_eq!(terminal.page, AppearancePage::Terminal);
  assert_eq!(terminal.effect_draft, None);
}

#[test]
fn draft_pages_render_apply_as_a_row_without_button_labels() {
  let mut application = draft_app(AppearancePage::TaskbarTime);
  application
    .surface_draft
    .as_mut()
    .unwrap()
    .time_seconds_enabled ^= true;
  let screen = render(&mut application, 80, 24);
  let apply = line_with(&screen, "Apply");
  assert!(apply.contains(icons::APPLY), "{apply:?}");
  assert!(
    !screen.iter().any(|line| line.contains("[ Apply ]")),
    "{screen:#?}"
  );
}

#[test]
fn esc_with_a_pending_draft_asks_before_discarding() {
  // Clean draft: Esc leaves without asking. Taskbar is an entry point
  // reached from the Control Center Home (crate principal), so leaving
  // signals the host instead of navigating to a parent page here.
  let mut clean = draft_app(AppearancePage::Taskbar);
  assert!(clean.handle(KeyCode::Esc), "signals leaving the crate");
  assert_eq!(clean.page, AppearancePage::Taskbar);
  assert!(clean.confirm.is_none());

  // Pending draft inside the Taskbar pages: going up keeps the draft.
  let mut nested = draft_app(AppearancePage::TaskbarTime);
  nested.surface_draft.as_mut().unwrap().time_seconds_enabled ^= true;
  nested.handle(KeyCode::Esc);
  assert_eq!(nested.page, AppearancePage::Taskbar);
  assert!(nested.confirm.is_none());
  assert!(nested.surface_draft_has_changes(), "draft survives");

  // Leaving the Taskbar would drop it: confirmation, focus on Cancel.
  nested.handle(KeyCode::Esc);
  assert_eq!(nested.page, AppearancePage::Taskbar);
  let (confirmation, state) = nested.confirm.unwrap();
  assert_eq!(confirmation, Confirmation::DiscardDraft);
  assert!(!state.is_confirm_focused());
  let rows = nested.rows();
  assert_eq!(nested.hints(&rows), confirm_hints(nested.lang));
  nested.handle(KeyCode::Enter);
  assert!(nested.confirm.is_none(), "Enter on Cancel closes");
  assert_eq!(nested.page, AppearancePage::Taskbar);
  assert!(nested.surface_draft_has_changes());

  nested.handle(KeyCode::Left);
  nested.handle(KeyCode::Char('n'));
  assert_eq!(nested.page, AppearancePage::Taskbar);
  nested.handle(KeyCode::Esc);
  assert!(
    nested.handle(KeyCode::Char('y')),
    "discarding on an exit point signals leaving the crate"
  );
  assert_eq!(nested.page, AppearancePage::Taskbar);
  assert!(nested.surface_draft.is_none(), "discarded");

  // Effect editors drop their value on every Esc. Blur is now an entry
  // point reached from the Hyprland category (crate principal), so
  // confirming here signals the host to leave instead of navigating to a
  // parent page inside this crate.
  let mut editor = app(AppearancePage::Blur);
  select(&mut editor, Item::BlurField(BlurField::Size));
  editor.handle(KeyCode::Right);
  editor.handle(KeyCode::Esc);
  assert!(editor.confirm.is_some());
  editor.handle(KeyCode::Up);
  assert!(editor.handle(KeyCode::Enter), "signals leaving the crate");
  assert_eq!(editor.page, AppearancePage::Blur);
  assert_eq!(editor.blur_draft, None);
}

#[test]
fn enter_on_a_draft_value_types_it_without_applying() {
  let mut editor = app(AppearancePage::Blur);
  select(&mut editor, Item::BlurField(BlurField::Contrast));
  editor.handle(KeyCode::Enter);
  assert_eq!(
    editor.page,
    AppearancePage::Prompt {
      goal: PromptGoal::BlurValue(BlurField::Contrast)
    }
  );
  assert!(editor.captures_text());
  for key in ['0', '.', '5'] {
    editor.handle(KeyCode::Char(key));
  }
  editor.handle(KeyCode::Enter);
  assert_eq!(editor.page, AppearancePage::Blur);
  assert_eq!(blur_value(&editor, BlurField::Contrast), 0.5);
  assert_eq!(
    selected(&editor),
    Some(Item::BlurField(BlurField::Contrast))
  );
  assert!(editor.action.is_none(), "nothing applied");

  let mut section = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::Transparency,
  });
  select(&mut section, Item::SectionValue);
  section.handle(KeyCode::Enter);
  section.handle(KeyCode::Char('0'));
  section.handle(KeyCode::Enter);
  assert_eq!(section.surface_draft().unwrap().transparency, 0);
  assert_eq!(selected(&section), Some(Item::SectionValue));
  assert!(section.action.is_none());

  // Out of range keeps the prompt open with an error.
  // Size accepts 1–64: 200 keeps the prompt open and leaves the draft alone.
  let mut invalid = app(AppearancePage::Blur);
  select(&mut invalid, Item::BlurField(BlurField::Size));
  invalid.handle(KeyCode::Enter);
  for key in ['2', '0', '0'] {
    invalid.handle(KeyCode::Char(key));
  }
  invalid.handle(KeyCode::Enter);
  assert!(invalid.prompt_error.is_some());
  assert_eq!(invalid.blur_draft, None);
}

#[test]
fn reload_keeps_a_changed_draft_and_rebuilds_a_clean_one() {
  let mut application = draft_app(AppearancePage::TaskbarTime);
  let mut loaded = application.state.clone();
  loaded.taskbar_time_seconds_enabled = !loaded.taskbar_time_seconds_enabled;

  // Clean: the draft follows the reloaded state.
  application.on_loaded(AppearancePage::TaskbarTime, loaded.clone());
  assert_eq!(
    application.surface_draft().unwrap().time_seconds_enabled,
    loaded.taskbar_time_seconds_enabled
  );

  // Changed: the user's edit survives the reload.
  application.activate(Item::DateFormat(TaskbarDateFormat::WeekdayDayMonthYear));
  application.on_loaded(AppearancePage::TaskbarTime, loaded);
  assert_eq!(
    application.surface_draft().unwrap().date_format,
    TaskbarDateFormat::WeekdayDayMonthYear
  );
}

#[test]
fn surface_value_navigation_keeps_vertical_focus_and_ends_with_apply() {
  for surface in EffectSurface::ALL {
    let mut application = draft_app(AppearancePage::SurfaceSection {
      surface,
      section: SurfaceSection::Transparency,
    });

    application.handle(KeyCode::Down);
    assert_eq!(selected(&application), Some(Item::SectionValue));
    application.handle(KeyCode::Up);
    assert_eq!(selected(&application), Some(Item::SectionEnabled));

    select(&mut application, Item::SectionValue);
    let value = |application: &AppearanceApp| application.surface_draft().unwrap().transparency;
    let initial = value(&application);
    application.handle(KeyCode::Char('+'));
    assert_eq!(selected(&application), Some(Item::SectionValue));
    assert_eq!(value(&application), (initial + 5).min(100));

    application.handle(KeyCode::End);
    assert_eq!(selected(&application), Some(Item::Apply));
    application.handle(KeyCode::Enter);
    assert!(application.action.is_some());
  }
}

#[test]
fn taskbar_icons_date_time_pages_toggle_and_end_with_apply() {
  let mut icons_page = draft_app(AppearancePage::TaskbarIcons);
  let draft = |application: &AppearanceApp| application.surface_draft().unwrap().clone();

  let before = draft(&icons_page);
  select(&mut icons_page, Item::AudioPlayer);
  icons_page.handle(KeyCode::Enter);
  let widget = TaskbarUtilityWidget::GpuTemperature;
  select(&mut icons_page, Item::UtilityWidget(widget));
  icons_page.handle(KeyCode::Enter);
  let after = draft(&icons_page);
  assert_eq!(after.audio_player_enabled, !before.audio_player_enabled);
  assert_eq!(
    after.utility_widgets.enabled(widget),
    !before.utility_widgets.enabled(widget)
  );

  // "Launcher ›" opens the shared `SurfaceSection{Taskbar, Launcher}` page
  // (Enable + Custom), instead of toggling inline like the other rows.
  select(&mut icons_page, Item::LauncherIcon);
  icons_page.handle(KeyCode::Enter);
  assert_eq!(
    icons_page.page,
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::Launcher,
    }
  );

  let mut launcher = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::Launcher,
  });
  let before_enabled = launcher.surface_draft().unwrap().launcher_enabled;
  select(&mut launcher, Item::LauncherEnabled);
  launcher.handle(KeyCode::Enter);
  assert_eq!(
    launcher.surface_draft().unwrap().launcher_enabled,
    !before_enabled
  );

  // Back to the Icons page before exercising the next submenu, since
  // navigating into Launcher moved `icons_page.page` away from TaskbarIcons.
  // `go()` schedules a refresh job that nothing polls in this test, so clear
  // it or the next Enter would be swallowed by the "busy" guard in `handle`.
  icons_page.go(AppearancePage::TaskbarIcons);
  icons_page.job = None;

  // "Utilities ›" opens the shared `SurfaceSection{Taskbar, UtilityIcons}`
  // page (Always expanded / Expand on hover).
  select(&mut icons_page, Item::Utilities);
  icons_page.handle(KeyCode::Enter);
  assert_eq!(
    icons_page.page,
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::UtilityIcons,
    }
  );

  let mut utility_group = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::UtilityIcons,
  });
  for mode in [
    TaskbarUtilityGroupMode::AlwaysExpanded,
    TaskbarUtilityGroupMode::Auto,
  ] {
    select(&mut utility_group, Item::UtilityGroup(mode));
    utility_group.handle(KeyCode::Enter);
    assert_eq!(utility_group.surface_draft().unwrap().utility_group, mode);
  }

  let mut date_format = draft_app(AppearancePage::TaskbarDateFormat);
  let format = TaskbarDateFormat::WeekdayDayMonthYear;
  select(&mut date_format, Item::DateFormat(format));
  date_format.handle(KeyCode::Enter);
  assert_eq!(date_format.surface_draft().unwrap().date_format, format);

  let mut time_format = draft_app(AppearancePage::TaskbarTimeFormat);
  select(
    &mut time_format,
    Item::TimeFormat(TaskbarTimeFormat::TwelveHour),
  );
  time_format.handle(KeyCode::Enter);
  assert_eq!(
    time_format.surface_draft().unwrap().time_format,
    TaskbarTimeFormat::TwelveHour
  );

  // Apply is the last row and runs the Taskbar apply.
  icons_page.go(AppearancePage::TaskbarIcons);
  icons_page.job = None;
  icons_page.handle(KeyCode::End);
  assert_eq!(selected(&icons_page), Some(Item::Apply));
  icons_page.handle(KeyCode::Enter);
  assert!(icons_page.action.is_some());
}

#[test]
fn launcher_custom_row_shows_the_current_icon_path_as_its_detail() {
  let mut launcher = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::Launcher,
  });
  let rows = launcher.rows();
  let custom_row = rows
    .iter()
    .find(|row| row.label() == tr(launcher.lang, "control_center.custom"))
    .expect("Custom row must be present");
  assert_eq!(
    custom_row.detail_text(),
    Some(tr(launcher.lang, "control_center.none"))
  );

  launcher.state.taskbar_launcher_custom_icon_path = Some("/home/user/icon.svg".into());
  let rows = launcher.rows();
  let custom_row = rows
    .iter()
    .find(|row| row.label() == tr(launcher.lang, "control_center.custom"))
    .expect("Custom row must be present");
  assert_eq!(custom_row.detail_text(), Some("/home/user/icon.svg"));
}

#[test]
fn apply_keeps_rendering_the_just_applied_draft_until_refresh_replaces_it() {
  // Regression test: pressing Apply used to clear `surface_draft`
  // immediately, which made every toggle/radio on the page flash back to
  // its pre-edit (unchecked/default) state for the whole apply+refresh
  // round trip, since the rows all render from `surface_draft()`.
  let mut application = draft_app(AppearancePage::TaskbarIcons);
  let draft = application.surface_draft.as_mut().unwrap();
  draft.audio_player_enabled = false;
  draft.launcher_enabled = false;

  application.apply_surface_changes();

  let draft = application
    .surface_draft()
    .expect("the just-edited draft must still be present right after Apply");
  assert!(!draft.audio_player_enabled);
  assert!(!draft.launcher_enabled);
  assert!(application.action.is_some());
}

fn custom_theme_app() -> AppearanceApp {
  let mut custom = app(AppearancePage::CustomThemes);
  custom.state.custom_themes = vec![CustomTheme {
    id: "mine".into(),
    name: "Mine".into(),
    base_theme: "argvus-dark".into(),
    profile_path: "/tmp/mine".into(),
    wallpaper_path: None,
  }];
  custom
}

#[test]
fn theme_delete_uses_the_single_confirmation_starting_on_cancel() {
  // Enter right after opening runs the focused row: Cancel. Cancelling
  // returns to Themes, as the old confirmation page did.
  let mut custom = custom_theme_app();
  custom.handle(KeyCode::Char('d'));
  let screen = render(&mut custom, 80, 24);
  assert!(line_with(&screen, "Mine").contains("Mine"));
  assert!(
    line_with(&screen, "Cancel").contains("> Cancel"),
    "{screen:#?}"
  );
  custom.handle(KeyCode::Enter);
  assert!(custom.confirm.is_none());
  assert!(custom.action.is_none(), "nothing deleted");
  assert!(custom.delete_theme.is_none());
  assert_eq!(custom.page, AppearancePage::Themes);

  // `y` confirms from any focus and deletes, then shows Themes.
  let mut custom = custom_theme_app();
  custom.handle(KeyCode::Char('d'));
  custom.handle(KeyCode::Char('y'));
  assert!(custom.action.is_some(), "delete started");
  assert_eq!(custom.page, AppearancePage::Themes);

  // Keys other than the dialog's do not reach the page underneath.
  let mut custom = custom_theme_app();
  custom.handle(KeyCode::Char('d'));
  custom.handle(KeyCode::Char('e'));
  assert_eq!(custom.page, AppearancePage::CustomThemes);
  assert!(custom.confirm.is_some());
}

#[test]
fn duplicate_import_uses_the_single_confirmation() {
  let pending = |application: &mut AppearanceApp| {
    application.pending_import = Some("/tmp/argvus/mine.zip".into());
    application.pending_import_name = Some("Mine".into());
    application.confirm = Some((Confirmation::ReplaceImport, ConfirmState::new()));
  };

  let mut import = app(AppearancePage::ThemeImport);
  pending(&mut import);
  import.handle(KeyCode::Char('n'));
  assert!(import.pending_import.is_none());
  assert!(import.action.is_none());
  assert_eq!(import.page, AppearancePage::Themes);

  let mut import = app(AppearancePage::ThemeImport);
  pending(&mut import);
  import.handle(KeyCode::Up);
  import.handle(KeyCode::Enter);
  assert!(import.action.is_some(), "replace started");
  assert!(import.pending_import.is_none(), "taken by the import");
  assert_eq!(import.page, AppearancePage::Themes);
}

#[test]
fn animations_root_lists_the_switch_options_and_the_five_categories() {
  let application = app(AppearancePage::Animations);
  let rows = application.rows();
  assert_eq!(rows[0].kind(), RowKind::Toggle { on: true }, "Enable");
  assert_eq!(rows[1].kind(), RowKind::Separator, "Options heading");
  assert_eq!(rows[2].label(), "Custom curves");
  assert_eq!(rows[3].label(), "Windows");
  assert_eq!(rows[4].label(), "Workspaces");
  assert_eq!(rows[5].label(), "Layers & popups");
  assert_eq!(rows[6].label(), "Borders & effects");
  assert_eq!(rows[7].label(), "Other animations");
  assert_eq!(
    rows.last().unwrap().id(),
    Some(&Item::Apply),
    "shared Apply row"
  );
}

#[test]
fn leaf_pages_show_only_the_rows_of_their_kind() {
  let opening = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Opening,
  })
  .page_rows();
  let labels: Vec<String> = opening.iter().map(|row| row.label().to_owned()).collect();
  assert_eq!(labels, ["Style", "Speed", "Curve", "Pop-in percentage"]);

  let moving = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Moving,
  })
  .page_rows();
  assert_eq!(moving.len(), 3, "Enable, Speed and Curve only");
  assert_eq!(moving[0].kind(), RowKind::Toggle { on: true });
}

#[test]
fn custom_curves_list_their_parameters_and_fixed_points() {
  let bezier = app(AppearancePage::AnimationsCurve {
    curve: AnimationCurve::Bezier,
  })
  .page_rows();
  assert_eq!(bezier.len(), 4, "x1, y1, x2, y2");

  let spring = app(AppearancePage::AnimationsCurve {
    curve: AnimationCurve::Spring,
  })
  .page_rows();
  assert_eq!(spring.len(), 3, "mass, stiffness, dampening");

  let linear = app(AppearancePage::AnimationsCurve {
    curve: AnimationCurve::Linear,
  })
  .page_rows();
  assert_eq!(linear.len(), 1, "fixed curve: points only");
  assert_eq!(linear[0].label(), "Control points");
}

#[test]
fn every_animations_page_ends_with_the_shared_apply_row() {
  let pages = [
    AppearancePage::Animations,
    AppearancePage::AnimationsCustomCurves,
    AppearancePage::AnimationsCurve {
      curve: AnimationCurve::Bezier,
    },
    AppearancePage::AnimationsGroup {
      group: AnimationGroup::Windows,
    },
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Opening,
    },
  ];
  for page in pages {
    let application = app(page);
    let rows = application.rows();
    let apply = rows.last().unwrap();
    assert_eq!(apply.id(), Some(&Item::Apply), "{page:?}");
    assert!(
      !apply.is_selectable(),
      "clean draft: Apply is skipped ({page:?})"
    );
  }
}

#[test]
fn animation_edits_stay_in_the_shared_draft_until_apply() {
  let mut animations = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Opening,
  });
  select(
    &mut animations,
    Item::AnimationLeafStyle(AnimationLeaf::Opening),
  );
  animations.handle(KeyCode::Enter);
  assert_eq!(
    animations
      .animations_editor()
      .values
      .leaf(AnimationLeaf::Opening)
      .style,
    AnimationStyle::SlideFade,
    "Enter cycles the style forward"
  );
  assert_eq!(
    animations
      .state
      .animation_values
      .leaf(AnimationLeaf::Opening)
      .style,
    AnimationStyle::Popin,
    "stored value unchanged"
  );
  assert!(animations.has_pending_changes(), "Apply is enabled");
  assert!(animations.action.is_none(), "nothing applied");
}

#[test]
fn animation_switch_is_part_of_the_draft() {
  let mut animations = app(AppearancePage::Animations);
  select(&mut animations, Item::Animations);
  animations.handle(KeyCode::Enter);
  assert!(!animations.animations_editor().enabled);
  assert!(animations.state.animations, "stored switch unchanged");
  assert!(animations.has_pending_changes());
  assert!(animations.action.is_none());
}

#[test]
fn typed_animation_values_are_bounded_and_stay_in_the_draft() {
  let mut speed = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Opening,
  });
  select(
    &mut speed,
    Item::AnimationField(AnimationField::Speed(AnimationLeaf::Opening)),
  );
  speed.handle(KeyCode::Enter);
  assert!(matches!(
    speed.page,
    AppearancePage::Prompt {
      goal: PromptGoal::AnimationValue(_)
    }
  ));
  // Speed accepts 1–10: 11 keeps the prompt open.
  speed.handle(KeyCode::Char('1'));
  speed.handle(KeyCode::Char('1'));
  speed.handle(KeyCode::Enter);
  assert!(speed.prompt_error.is_some());
  assert!(matches!(speed.page, AppearancePage::Prompt { .. }));

  speed.handle(KeyCode::Backspace);
  speed.handle(KeyCode::Backspace);
  speed.handle(KeyCode::Char('7'));
  speed.handle(KeyCode::Enter);
  assert_eq!(
    speed.page,
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Opening
    }
  );
  assert_eq!(
    speed
      .animations_editor()
      .values
      .leaf(AnimationLeaf::Opening)
      .speed,
    7.0
  );
  assert_eq!(
    speed
      .state
      .animation_values
      .leaf(AnimationLeaf::Opening)
      .speed,
    4.1,
    "stored value unchanged"
  );
  assert!(speed.action.is_none(), "nothing applied");
}

#[test]
fn leaving_a_leaf_keeps_the_draft_and_leaving_the_root_asks() {
  let mut animations = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Fade,
  });
  select(
    &mut animations,
    Item::AnimationLeafEnabled(AnimationLeaf::Fade),
  );
  animations.handle(KeyCode::Enter);
  assert!(
    !animations.back_discards_draft(),
    "sub-pages keep the draft"
  );

  animations.handle(KeyCode::Esc);
  assert_eq!(
    animations.page,
    AppearancePage::AnimationsGroup {
      group: AnimationGroup::LayersPopups
    }
  );
  assert!(animations.animations_draft.is_some(), "draft survives");

  animations.handle(KeyCode::Esc);
  assert_eq!(animations.page, AppearancePage::Animations);
  assert!(animations.confirm.is_none(), "root asks only when leaving");
  assert!(
    !animations.handle(KeyCode::Esc),
    "pending draft: Esc asks first"
  );
  assert!(animations.confirm.is_some(), "leaving the root asks first");
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
fn icons_page_lists_installed_themes_and_marks_the_configured_one() {
  let mut application = app(AppearancePage::Icons);
  application.state.icon_theme = "Papirus".into();
  application.state.icon_themes = vec![
    icon_theme("Argvus Icons", IconThemeSource::System, true),
    icon_theme("Papirus", IconThemeSource::User, false),
  ];

  let rows = application.page_rows();

  assert_eq!(rows.len(), 2);
  assert_eq!(rows[0].label(), "Argvus Icons");
  assert_eq!(rows[1].label(), "Papirus");
  assert_eq!(rows[1].kind(), RowKind::Choice { current: true });
  assert_eq!(rows[0].kind(), RowKind::Choice { current: false });
  assert_eq!(rows[0].detail_text(), Some("Official · System"));
  assert_eq!(rows[1].detail_text(), Some("User"));
}

#[test]
fn icons_page_reports_a_missing_configured_theme_with_a_non_focusable_row() {
  let mut application = app(AppearancePage::Icons);
  application.state.icon_theme = "Gone".into();
  application.state.icon_themes = vec![icon_theme("Argvus Icons", IconThemeSource::System, true)];

  let rows = application.rows();
  assert_eq!(
    rows[0].kind(),
    RowKind::Info,
    "fallback notice is informational"
  );
  assert!(!rows[0].is_selectable());

  application.menu.normalize(&rows);
  assert!(
    matches!(
      application.menu.selected_id(&rows),
      Some(Item::IconTheme(0))
    ),
    "focus skips the info row and lands on the first theme"
  );
}

#[test]
fn icons_page_without_installed_themes_shows_only_an_info_row() {
  let application = app(AppearancePage::Icons);

  let rows = application.page_rows();

  assert_eq!(rows.len(), 1);
  assert_eq!(rows[0].kind(), RowKind::Info);
}

#[test]
fn icons_page_breadcrumb_is_under_desktop_and_leaving_it_returns_there() {
  let mut application = app(AppearancePage::Icons);

  let breadcrumb = application.breadcrumb_of(AppearancePage::Icons);
  assert!(
    breadcrumb.ends_with("> Desktop Style > Icons"),
    "{breadcrumb}"
  );

  assert!(!application.navigate_back());
  assert_eq!(application.page, AppearancePage::Desktop);
}

#[test]
fn desktop_lists_icons_after_wallpaper_with_a_unique_glyph() {
  let rows = app(AppearancePage::Desktop).rows();
  let icons_row = rows
    .iter()
    .find(|row| row.id() == Some(&Item::Icons))
    .expect("Desktop lists Icons");

  assert_eq!(icons_row.icon_glyph(), Some(icons::APPS));
  // Fonts comes right after Icons, as the last Desktop entry.
  assert_eq!(rows.last().and_then(|row| row.id()), Some(&Item::Fonts));
}

#[test]
fn desktop_fonts_entry_is_hidden_when_the_fonts_page_is_not_compiled() {
  let mut application = app(AppearancePage::Desktop);
  application.fonts_available = false;

  assert!(
    application
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::Fonts))
  );
}

#[test]
fn desktop_fonts_entry_requests_the_fonts_page_from_the_host() {
  let mut application = app(AppearancePage::Desktop);
  let rows = application.rows();
  application.menu.normalize(&rows);
  assert!(application.menu.select(&rows, &Item::Fonts));

  assert!(!application.handle(KeyCode::Enter));
  assert!(application.take_fonts_request());
  assert!(!application.take_fonts_request());
}

#[test]
fn icons_space_chooses_a_draft_and_does_not_apply_it() {
  let mut application = app(AppearancePage::Icons);
  application.state.icon_theme = "Papirus".into();
  application.state.icon_themes = vec![
    icon_theme("Argvus Icons", IconThemeSource::System, true),
    icon_theme("Papirus", IconThemeSource::User, false),
  ];

  application.handle(KeyCode::Char(' '));

  assert_eq!(application.icon_draft.as_deref(), Some("Argvus Icons"));
  assert!(application.icon_changed());
  assert!(application.action.is_none(), "Space must not apply");
  // The mark follows the draft until the theme is applied.
  let rows = application.page_rows();
  assert_eq!(rows[0].kind(), RowKind::Choice { current: true });
  assert_eq!(rows[1].kind(), RowKind::Choice { current: false });
}

#[test]
fn icons_apply_button_is_reached_with_tab_only_when_the_draft_changed() {
  let mut application = app(AppearancePage::Icons);
  application.state.icon_theme = "Papirus".into();
  application.state.icon_themes = vec![
    icon_theme("Argvus Icons", IconThemeSource::System, true),
    icon_theme("Papirus", IconThemeSource::User, false),
  ];

  application.handle(KeyCode::Tab);
  assert_eq!(application.icon_actions.cursor(), None);

  application.handle(KeyCode::Char(' '));
  application.handle(KeyCode::Tab);
  assert_eq!(application.icon_actions.cursor(), Some(0));
}

#[test]
fn icons_search_filters_the_list_by_name() {
  let mut application = app(AppearancePage::Icons);
  application.state.icon_theme = "Papirus".into();
  application.state.icon_themes = vec![
    icon_theme("Argvus Icons", IconThemeSource::System, true),
    icon_theme("Papirus", IconThemeSource::User, false),
  ];

  application.handle(KeyCode::Char('/'));
  for c in "pap".chars() {
    application.handle(KeyCode::Char(c));
  }

  let labels: Vec<String> = application
    .icon_rows()
    .iter()
    .map(|row| row.label().to_string())
    .collect();
  assert_eq!(labels, vec!["Papirus".to_string()]);
  assert!(application.captures_text(), "typing must not quit the app");
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

#[test]
fn icons_page_draws_search_actions_and_a_footer_with_status_left_and_shortcuts_right() {
  let mut application = app(AppearancePage::Icons);
  application.state.icon_theme = "Papirus".into();
  application.state.icon_themes = vec![
    icon_theme("Argvus Icons", IconThemeSource::System, true),
    icon_theme("Papirus", IconThemeSource::User, false),
  ];
  application.status = Some(StatusMessage {
    kind: StatusKind::Success,
    text: "Icon theme applied".into(),
  });

  let screen = render(&mut application, 100, 30);

  assert!(line_with(&screen, "Search:").contains("Search:"));
  assert!(line_with(&screen, "Actions").contains("Actions"));
  assert!(line_with(&screen, "Apply").contains("Apply"));
  assert!(line_with(&screen, "[OK] Icon theme applied").contains("[OK]"));
  // The frame's border is part of the line; the text sits flush right.
  let shortcuts = line_with(&screen, "Esc Back").trim_matches('│');
  assert!(shortcuts.trim_end().ends_with("? Help"), "{shortcuts:?}");
  assert!(shortcuts.starts_with(' '), "shortcuts are right-aligned");
}
