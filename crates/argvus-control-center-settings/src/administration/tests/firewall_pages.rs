//! Firewall page: fields, toggles, protection level and the actions that confirm before running.

use super::*;

#[test]
fn every_firewall_field_is_editable_without_writing() {
  let mut app = app(Page::Firewall);
  app.admin.firewall = json!({"active":false,"enabled":false});
  app.admin.config = json!({});
  for (index, (key, _)) in FIREWALL_FIELDS.iter().enumerate() {
    app.admin.config[*key] = json!(if *key == "PROTECTION_LEVEL" {
      "high"
    } else {
      ""
    });
    app.admin_activate(Item::FirewallField(index));
    if *key == "PROTECTION_LEVEL" {
      assert_eq!(app.admin.config[*key], "paranoid");
    } else {
      assert!(app.admin.editor.take().is_some());
    }
  }
  app.admin.config["ALLOW_SSH"] = json!("n");
  let ssh = FIREWALL_FIELDS
    .iter()
    .position(|(key, _)| *key == "ALLOW_SSH")
    .unwrap();
  app.admin_activate(Item::FirewallField(ssh));
  assert_eq!(app.admin.config["ALLOW_SSH"], "y");
  assert!(!app.admin.busy());
}

#[test]
fn firewall_rows_are_toggles_values_and_a_protection_level_step() {
  let mut app = app(Page::Firewall);
  app.admin.firewall = json!({"active":true,"enabled":true});
  app.admin.config = json!({"ALLOW_SSH":"y","PROTECTION_LEVEL":"low","SSH_PORT":"22"});
  let rows = app.rows();
  let kind = |key: &str| {
    let index = FIREWALL_FIELDS.iter().position(|(k, _)| *k == key).unwrap();
    rows
      .iter()
      .find(|row| row.id() == Some(&Item::FirewallField(index)))
      .map(|row| row.kind())
      .unwrap()
  };
  assert_eq!(kind("ALLOW_SSH"), RowKind::Toggle { on: true });
  assert_eq!(kind("PROTECTION_LEVEL"), RowKind::Value { step: Some(1) });
  assert_eq!(kind("SSH_PORT"), RowKind::Value { step: None });
  let level = FIREWALL_FIELDS
    .iter()
    .position(|(key, _)| *key == "PROTECTION_LEVEL")
    .unwrap();
  select(&mut app, Item::FirewallField(level));
  press(&mut app, KeyCode::Left);
  assert_eq!(
    app.admin.config["PROTECTION_LEVEL"], "paranoid",
    "wraps around"
  );
  press(&mut app, KeyCode::Right);
  assert_eq!(app.admin.config["PROTECTION_LEVEL"], "low");
  assert_eq!(
    app.page(),
    Page::Firewall,
    "← adjusts instead of going back"
  );
}

#[test]
fn firewall_actions_are_in_the_bottom_bar() {
  let mut app = app(Page::Firewall);
  app.admin.firewall = json!({"active":false,"enabled":false});
  app.admin.config = json!({});
  let rows = app
    .admin
    .rows(Page::Firewall, Lang::for_locale("pt-BR"), "");
  let position = |item: Item| rows.iter().position(|row| row.id() == Some(&item)).unwrap();
  assert!(position(Item::FirewallField(FIREWALL_FIELDS.len() - 1)) < position(Item::AddRules));
  for item in [Item::SaveFirewall, Item::DiscardFirewall, Item::ApplyRules] {
    assert!(!rows.iter().any(|row| row.id() == Some(&item)));
  }
  assert_eq!(
    app
      .action_rows()
      .iter()
      .filter_map(|row| row.id().copied())
      .collect::<Vec<_>>(),
    [Item::SaveFirewall, Item::DiscardFirewall, Item::ApplyRules],
  );
  assert_eq!(
    rows[position(Item::FirewallBoot)].kind(),
    RowKind::Toggle { on: false }
  );
  assert!(rows.iter().all(|row| !row.label().contains("[ ")));
}

#[test]
fn firewall_rows_route_to_confirm_editor_reload_and_cancel_like_before() {
  let mut app = app(Page::Firewall);
  app.admin.firewall = json!({
    "active":false,
    "enabled":false,
    "config_text":"",
    "rules":"iptables -A INPUT -j DROP"
  });
  app.admin.config = json!({"ALLOW_SSH":"n"});
  app.admin_activate(Item::SaveFirewall);
  assert!(app.admin.pending.is_some());
  app.cancel_modal();
  app.admin_activate(Item::AddRules);
  assert_eq!(app.admin.editor.as_ref().unwrap().target, EditTarget::Rules);
  app.admin_input(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
  assert!(app.admin.rules_draft.is_some());
  assert!(app.admin.pending.is_none());
  app.admin_activate(Item::ApplyRules);
  assert!(app.admin.pending.is_some());
  let Some(PendingAction::Administration { message, .. }) = app.confirm.clone() else {
    panic!("Apply saved rules must ask first");
  };
  assert!(
    message.contains(tr(app.lang, "control_center.apply_saved_rules_description")),
    "the confirmation warns that the firewall restarts: {message}"
  );
  app.cancel_modal();
  app.admin_activate(Item::FirewallService);
  assert!(
    app.admin.pending.is_some(),
    "service state change is confirmed"
  );
  app.cancel_modal();
  app.admin_activate(Item::DiscardFirewall);
  assert_eq!(app.admin.config, app.admin.firewall["config"]);
  assert!(app.admin.rules_draft.is_none());
}

#[test]
fn group_firewall_create_and_password_drafts_ask_before_leaving() {
  let mut group = app(Page::GroupList);
  group.admin_activate(Item::GroupEntry(0));
  group.admin_activate(Item::Members);
  group.admin_activate(Item::Member(0));
  press(&mut group, KeyCode::Esc);
  assert_eq!(group.page(), Page::Group, "members share the group draft");
  assert!(group.confirm.is_none());
  press(&mut group, KeyCode::Esc);
  assert!(matches!(group.confirm, Some(PendingAction::DiscardDraft)));

  let mut firewall = app(Page::Firewall);
  firewall.admin.firewall = json!({"active":true,"enabled":true,"config":{"ALLOW_SSH":"y"}});
  firewall.admin.config = json!({"ALLOW_SSH":"y"});
  let save = |app: &App| {
    app
      .action_rows()
      .into_iter()
      .find(|row| row.id() == Some(&Item::SaveFirewall))
      .unwrap()
      .is_selectable()
  };
  assert!(!save(&firewall));
  let ssh = FIREWALL_FIELDS
    .iter()
    .position(|(key, _)| *key == "ALLOW_SSH")
    .unwrap();
  firewall.admin_activate(Item::FirewallField(ssh));
  assert!(save(&firewall));
  press(&mut firewall, KeyCode::Esc);
  assert!(matches!(
    firewall.confirm,
    Some(PendingAction::DiscardDraft)
  ));
  press(&mut firewall, KeyCode::Char('y'));
  assert_eq!(firewall.admin.config["ALLOW_SSH"], "y", "discard restores");

  let mut create = app(Page::Users);
  create.admin_activate(Item::CreateUser);
  let create_row = |app: &App| {
    app
      .rows()
      .into_iter()
      .find(|row| row.id() == Some(&Item::CreateAccount))
      .unwrap()
      .is_selectable()
  };
  assert!(
    create_row(&create),
    "Create is always reachable; an empty name is refused"
  );
  press(&mut create, KeyCode::Esc);
  assert_eq!(create.page(), Page::Users);
  create.admin_activate(Item::CreateUser);
  create.admin.user["user"] = json!("bob");
  assert!(create_row(&create));
  press(&mut create, KeyCode::Esc);
  assert!(matches!(create.confirm, Some(PendingAction::DiscardDraft)));

  let mut password = app(Page::User);
  password.admin_activate(Item::ChangePassword);
  password.admin.passwords[1] = "secret".into();
  press(&mut password, KeyCode::Esc);
  assert!(matches!(
    password.confirm,
    Some(PendingAction::DiscardDraft)
  ));
  press(&mut password, KeyCode::Char('y'));
  assert_eq!(password.page(), Page::User);
  assert!(!password.admin.passwords_changed());
}
