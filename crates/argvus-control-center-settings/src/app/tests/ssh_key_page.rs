//! SSH keys page: generating, loading, copying, deleting and searching keys.

use super::*;

#[test]
fn ssh_keys_page_offers_unload_only_for_loaded_keys() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![
      ssh_key_entry("alpha", AgentState::Loaded),
      ssh_key_entry("beta", AgentState::NotLoaded),
    ],
  };
  let rows = app.rows();
  assert!(row_of(&rows, Item::UnloadSshKey(0)).is_selectable());
  assert!(row_of(&rows, Item::LoadSshKey(1)).is_selectable());
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::LoadSshKey(0)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::UnloadSshKey(1)))
  );
}

#[test]
fn ssh_keys_delete_is_a_destructive_row_that_only_asks_for_confirmation() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![ssh_key_entry("alpha", AgentState::NotLoaded)],
  };
  let rows = app.rows();
  assert_eq!(
    row_of(&rows, Item::DeleteSshKey(0)).kind(),
    RowKind::Destructive
  );
  // The delete row is the last selectable row, inside the Danger zone.
  press(&mut app, KeyCode::End);
  assert_eq!(app.selected_item(), Some(Item::DeleteSshKey(0)));
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::DeleteSshKey(ref name)) if name == "alpha"
  ));
}

#[test]
fn ssh_keys_block_copies_the_public_key_before_load_and_both_rows_have_icons() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![ssh_key_entry("alpha", AgentState::NotLoaded)],
  };
  let rows = app.rows();
  let copy = row_of(&rows, Item::CopySshPublicKey(0));
  assert_eq!(copy.kind(), RowKind::Action);
  assert_eq!(copy.icon_glyph(), Some(argvus_tui::icons::COPY));
  assert_eq!(
    row_of(&rows, Item::DeleteSshKey(0)).icon_glyph(),
    Some(argvus_tui::icons::DELETE)
  );
  let position = |item: Item| rows.iter().position(|row| row.id() == Some(&item));
  assert!(position(Item::CopySshPublicKey(0)) < position(Item::LoadSshKey(0)));
}

#[test]
fn ssh_keys_form_rows_respond_to_enter() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![ssh_key_entry("alpha", AgentState::NotLoaded)],
  };
  assert_eq!(app.selected_item(), Some(Item::NewSshKeyName));
  press(&mut app, KeyCode::Enter);
  assert!(app.admin.editor.is_some(), "Enter on Name opens the editor");
  press(&mut app, KeyCode::Esc);
  press(&mut app, KeyCode::Down);
  assert_eq!(app.selected_item(), Some(Item::SshKeyType));
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.new_ssh_key_type, crate::system::ssh_keys::KeyType::Rsa);
}

#[test]
fn ssh_keys_search_matches_name_or_comment_and_keeps_the_listing_index() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![
      ssh_key_entry("alpha", AgentState::NotLoaded),
      ssh_key_entry("beta", AgentState::NotLoaded),
    ],
  };
  // "beta@host" is the comment of the second pair, so only its block remains,
  // and its Delete row still points at index 1 of the full listing.
  app.search = "beta".into();
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.id() == Some(&Item::DeleteSshKey(1)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::DeleteSshKey(0)))
  );
}

#[test]
fn ssh_keys_action_bar_holds_refresh_and_unload_all() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![ssh_key_entry("alpha", AgentState::Loaded)],
  };
  assert!(app.has_action_bar());
  let actions: Vec<Option<Item>> = app
    .action_rows()
    .iter()
    .map(|row| row.id().copied())
    .collect();
  assert_eq!(
    actions,
    vec![Some(Item::RefreshSshKeys), Some(Item::UnloadAllSshKeys)]
  );
}

#[test]
fn ssh_keys_page_without_the_launcher_only_informs() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys::default();
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
}
