//! Saved profiles and configurations: pages, actions, names and replayed rules.

use super::*;

#[test]
fn home_lists_monitors_profiles_and_refresh_as_rows() {
  let mut app = app_with(false);
  app.state.profiles.push(profile("Trabalho"));
  let rows = app.rows();
  assert_eq!(
    ids(&app),
    vec![
      Some(Item::Monitor(0)),
      Some(Item::Profiles),
      None,
      Some(Item::Refresh)
    ]
  );
  assert_eq!(rows[0].label(), "eDP-1");
  assert_eq!(rows[0].icon_glyph(), Some(icons::MONITOR));
  assert!(!rows[0].detail_text().unwrap().contains("Trabalho"));
  assert!(
    !rows[0].detail_text().unwrap().contains("escala"),
    "no fixed Portuguese text"
  );
  assert_eq!(rows[1].detail_text(), Some("1"));
}

#[test]
fn profiles_row_is_visible_without_profiles() {
  let app = app_with(false);
  assert!(app.state.profiles.is_empty());
  assert!(ids(&app).contains(&Some(Item::Profiles)));
}

#[test]
fn each_saved_configuration_opens_its_own_detail() {
  let mut app = app_with(false);
  app
    .config
    .set_monitor("HDMI-A-1", PersistedMonitor::default());
  app.config.set_monitor("DP-2", PersistedMonitor::default());
  let names = app.stale_names();
  assert_eq!(names.len(), 2);
  select(&mut app, Item::Stale(1));
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, DisplayPage::Detail(2));
  assert_eq!(app.stale_name(2).as_deref(), Some(names[1].as_str()));
  assert_eq!(
    ids(&app),
    vec![None, None, Some(Item::RemoveConfig)],
    "Info, Danger zone, Remove configuration"
  );
  assert_eq!(app.rows()[2].kind(), RowKind::Destructive);
}

#[test]
fn profile_rule_replays_persisted_fields() {
  let persisted = PersistedMonitor {
    mode: Some("1920x1080@144".into()),
    position: Some("0x0".into()),
    scale: Some(1.25),
    transform: Some(1),
    vrr: Some(1),
    hdr: Some(1),
    bitdepth: Some(10),
    disabled: Some(false),
    ..Default::default()
  };
  let rule = rule_from_persisted("eDP-1", &persisted);
  assert!(
    rule.starts_with("eDP-1, 1920x1080@144, 0x0, 1.25, transform, 1"),
    "{rule}"
  );
  assert!(rule.contains("vrr, 1"), "{rule}");
  assert!(rule.contains("bitdepth, 10"), "{rule}");
  assert!(rule.contains("supports_hdr, 1"), "{rule}");
  let disabled = PersistedMonitor {
    disabled: Some(true),
    ..Default::default()
  };
  assert_eq!(rule_from_persisted("eDP-1", &disabled), "eDP-1, disabled");
}

#[test]
fn profiles_open_a_profile_page_with_its_actions() {
  let mut app = app_with(false);
  app.state.profiles = vec![profile("Casa"), profile("Trabalho")];
  app.state.active_profile = Some("Trabalho".into());
  app.page = DisplayPage::Profiles;
  assert_eq!(
    ids(&app),
    vec![
      Some(Item::Profile(0)),
      Some(Item::Profile(1)),
      None,
      Some(Item::NewProfile)
    ]
  );
  assert_eq!(app.rows()[1].detail_text(), Some("Active"));
  select(&mut app, Item::Profile(1));
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, DisplayPage::Profile(1));
  let rows = app.rows();
  let last = rows.last().unwrap();
  assert_eq!(last.id(), Some(&Item::DeleteProfile));
  assert_eq!(last.kind(), RowKind::Destructive);
  assert_eq!(selected(&app), Some(Item::ApplyProfile));
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Enter);
  assert_eq!(
    app.page,
    DisplayPage::Prompt {
      goal: PromptGoal::ProfileRename(1)
    }
  );
  assert_eq!(app.prompt_buffer, "Trabalho");
  app.handle(KeyCode::Esc);
  assert_eq!(
    app.page,
    DisplayPage::Profile(1),
    "rename returns to the profile"
  );
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, DisplayPage::Profiles);
}

#[test]
fn new_profile_row_opens_the_name_prompt() {
  let mut app = app_with(false);
  app.page = DisplayPage::Profiles;
  assert_eq!(selected(&app), Some(Item::NewProfile));
  app.handle(KeyCode::Enter);
  assert_eq!(
    app.page,
    DisplayPage::Prompt {
      goal: PromptGoal::ProfileCreate
    }
  );
}

#[test]
fn removing_a_configuration_and_deleting_a_profile_ask_first() {
  let mut app = app_with(false);
  app
    .config
    .set_monitor("HDMI-A-1", PersistedMonitor::default());
  app.page = DisplayPage::Detail(app.stale_start());
  app.handle(KeyCode::Enter);
  assert!(matches!(app.confirm, Some((Confirmation::RemoveConfig, _))));
  app.handle(KeyCode::Enter);
  assert!(app.confirm.is_none(), "Enter right after opening cancels");
  assert_eq!(app.config.monitors.len(), 1, "nothing removed");

  app.state.profiles.push(profile("Casa"));
  app.page = DisplayPage::Profile(0);
  select(&mut app, Item::DeleteProfile);
  app.handle(KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some((Confirmation::DeleteProfile(0), _))
  ));
  assert!(app.footer_hints(&app.rows()).contains("y "));
  app.handle(KeyCode::Char('n'));
  assert!(app.confirm.is_none());
  assert_eq!(app.state.profiles.len(), 1, "nothing deleted");
}
