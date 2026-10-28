//! Theme and wallpaper pages: official and custom themes, families, modes and wallpapers.

use super::*;

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
fn theme_rows_use_friendly_official_names() {
  let rows = labels(&app(AppearancePage::Themes));
  // Official, Custom, export, import.
  assert_eq!(rows.len(), 4);
  assert!(rows[0].contains("Official"));
  assert!(rows[1].contains("Custom"));
  assert!(rows[2].contains("Export theme"));
  assert!(rows[3].contains("Import theme"));
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

// The test state lists only the built-in themes; installed packages are
// discovered from /usr/share/argvus/themes at runtime.
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
fn theme_shortcuts_are_kept_and_shown_in_the_footer() {
  let mut custom = app(AppearancePage::CustomThemes);
  custom.state.custom_themes = vec![CustomTheme {
    id: "mine".into(),
    name: "Mine".into(),
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
