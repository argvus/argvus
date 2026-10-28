//! Hub pages: default apps, fonts, users and groups, notifications, hostname and language.

use super::*;

#[test]
fn language_page_shows_status_lines_and_selectable_choices() {
  let app = app(Page::Language);
  let rows = app.rows();
  assert_eq!(rows.len(), 6);
  assert_eq!(rows[0].label(), "Current language");
  assert_eq!(rows[0].kind(), RowKind::Info);
  assert_eq!(rows[1].label(), "Regional locale");
  assert_eq!(rows[2].label(), "Encoding");
  assert_eq!(rows[3].kind(), RowKind::Separator);
  assert_eq!(rows[4].id(), Some(&Item::Language(0)));
  assert_eq!(rows[5].id(), Some(&Item::Language(1)));
  assert!(
    rows.iter().all(|row| !row.label().contains('🇺')),
    "no flag emoji"
  );
  assert_eq!(
    rows[4].kind(),
    RowKind::Choice { current: true },
    "English is current"
  );
  assert_eq!(app.selected_item(), Some(Item::Language(0)));
}

#[test]
fn fonts_dashboard_has_sections_icons_status_and_danger_zone() {
  let app = app(Page::Fonts);
  let rows = app.rows();
  let targets: Vec<&Row<Item>> = rows
    .iter()
    .filter(|row| matches!(row.id(), Some(Item::FontTarget(_))))
    .collect();
  assert_eq!(targets.len(), FontTarget::ALL.len());
  assert_eq!(targets[0].icon_glyph(), Some(argvus_tui::icons::TASKBAR));
  assert!(targets.iter().all(|row| row.detail_text().is_some()));
  let settings = rows
    .iter()
    .filter(|row| matches!(row.id(), Some(Item::FontSetting(_))))
    .count();
  assert_eq!(settings, SettingKind::ALL.len());
  assert_eq!(rows.iter().filter(|row| row.is_section()).count(), 3);
  let last = rows.last().unwrap();
  assert_eq!(last.id(), Some(&Item::ResetDefaults));
  assert_eq!(last.kind(), RowKind::Destructive);
}

#[test]
fn fonts_dashboard_icons_are_distinct() {
  let mut seen = std::collections::HashSet::new();
  for target in FontTarget::ALL {
    let icon = font_target_icon(target);
    assert!(seen.insert(icon), "duplicate icon {icon} for {target:?}");
  }
  for setting in SettingKind::ALL {
    let icon = setting_icon(setting);
    assert!(seen.insert(icon), "duplicate icon {icon} for {setting:?}");
  }
}

#[test]
fn default_apps_dashboard_uses_icons_and_status() {
  let app = app(Page::DefaultApps);
  let rows = app.rows();
  assert_eq!(rows.len(), Category::ORDER.len());
  assert!(rows[0].label().contains("Terminal"), "{}", rows[0].label());
  assert_eq!(rows[0].icon_glyph(), Some(argvus_tui::icons::TERMINAL));
  assert_eq!(rows[0].kind(), RowKind::Submenu);
  for row in &rows {
    assert!(row.detail_text().is_some());
  }
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ResetDefaults)),
    "Restore Defaults lives in the action bar, not in the list"
  );
  let actions = app.action_rows();
  assert_eq!(actions.len(), 1);
  assert_eq!(actions[0].id(), Some(&Item::ResetDefaults));
  assert_eq!(actions[0].kind(), RowKind::Destructive);
}

#[test]
fn default_apps_dashboard_icons_are_distinct() {
  let mut seen = std::collections::HashSet::new();
  for category in Category::ORDER {
    let icon = category_icon(category);
    assert!(seen.insert(icon), "duplicate icon {icon} for {category:?}");
  }
}

#[test]
fn reset_button_and_r_open_the_same_confirmation() {
  let mut app = app(Page::DefaultApps);
  press(&mut app, KeyCode::Right);
  assert_eq!(app.action_cursor(), Some(0), "→ focuses Restore Defaults");
  press(&mut app, KeyCode::Enter);
  assert!(matches!(app.confirm, Some(PendingAction::ResetApps)));
  press(&mut app, KeyCode::Esc);
  assert!(app.confirm.is_none());
  press(&mut app, KeyCode::Char('r'));
  assert!(matches!(app.confirm, Some(PendingAction::ResetApps)));
}

#[test]
fn users_groups_hub_uses_typed_submenus_with_counts() {
  let app = app(Page::UsersGroups);
  let rows = app.rows();
  assert_eq!(rows.len(), 2);
  assert_eq!(rows[0].id(), Some(&Item::Users));
  assert_eq!(rows[0].label(), "Users");
  assert_eq!(rows[0].detail_text(), Some("N/A"));
  assert_eq!(rows[1].id(), Some(&Item::Groups));
  assert!(!rows[1].label().ends_with(':'));
  assert_eq!(rows[1].detail_text(), Some("N/A"));
}

#[test]
fn notifications_page_holds_only_do_not_disturb() {
  let app = app(Page::Notifications);
  let rows = app.rows();
  assert_eq!(rows.len(), 1);
  assert_eq!(rows[0].id(), Some(&Item::DoNotDisturb));
  assert!(matches!(rows[0].kind(), RowKind::Toggle { .. }));
  assert!(rows[0].label().contains("Do Not Disturb"));
}

#[test]
fn hostname_row_opens_the_editor_directly() {
  let mut app = app(Page::Hostname);
  app.hostname = "current-machine".into();
  press(&mut app, KeyCode::Enter);
  assert!(app.hostname_editing);
  assert_eq!(app.hostname_input, "current-machine");
  assert_eq!(app.page(), Page::Hostname);
}

#[test]
fn hostname_page_row_keeps_current_value_while_editing() {
  let mut app = app(Page::Hostname);
  app.hostname = "current-machine".into();
  app.hostname_editing = true;
  app.hostname_input = "draft-name".into();
  let rows = app.rows();
  assert_eq!(rows.len(), 1);
  assert!(rows[0].label().contains("Current hostname"));
  assert_eq!(rows[0].detail_text(), Some("current-machine"));
}

#[test]
fn users_groups_counts_match_loaded_accounts() {
  let mut app = app(Page::UsersGroups);
  app.admin.accounts = serde_json::json!({
    "actor_uid":1000,
    "shells":["/bin/bash", "/bin/zsh"],
    "groups":["users", "wheel"],
    "group_details":[
      {"name":"users","gid":1000,"members":[]},
      {"name":"wheel","gid":998,"members":["alice"]}
    ],
    "users":[
      {"user":"root","uid":0,"gid":0,"name":"root","shell":"/bin/bash","primary_group":"root","groups":[]},
      {"user":"alice","uid":1000,"gid":1000,"name":"Alice","shell":"/bin/bash","primary_group":"users","groups":["wheel"]}
    ]
  });
  let rows = app.rows();
  assert_eq!(rows[0].detail_text(), Some("1"));
  assert_eq!(rows[1].detail_text(), Some("1"));
  assert!(rows.iter().all(Row::is_selectable));
}
