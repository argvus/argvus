//! Icon theme page: listing, search, draft choice and leaving with an unapplied choice.

use super::*;

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
