//! GPG keys pages: the key form, the keyring list, editing, signing, revoking, import and export.

use super::*;

#[test]
fn gpg_sections_start_expanded_and_collapse_on_enter() {
  use crate::item::GpgSection;

  let mut app = app(Page::GpgKeys);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::GpgFullName))
  );
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::ImportGpgKey))
  );
  select_gpg(&mut app, Item::ToggleGpgSection(GpgSection::Generate));
  press(&mut app, KeyCode::Enter);
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::GpgFullName))
  );
  press(&mut app, KeyCode::Enter);
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::GpgFullName))
  );
}

#[test]
fn gpg_keys_page_offers_the_form_and_a_link_to_the_keyring_only() {
  let mut app = app(Page::GpgKeys);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![gpg_key_entry(
      "Alice",
      "AAAA111122223333AAAA111122223333AAAAAAAA",
    )],
    ..Default::default()
  };
  let rows = app.rows();
  assert!(rows.iter().any(|row| row.id() == Some(&Item::OpenGpgKeys)));
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ExportGpgKey(0))),
    "keys are listed on their own page"
  );
}

#[test]
fn gpg_keys_generate_stays_disabled_until_name_and_email_are_typed() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  let rows = app.rows();
  assert!(!row_of(&rows, Item::GenerateGpgKey).is_selectable());
  app.gpg_form.full_name = "Alice".into();
  let rows = app.rows();
  assert!(!row_of(&rows, Item::GenerateGpgKey).is_selectable());
  app.gpg_form.email = "alice@example.com".into();
  let rows = app.rows();
  assert!(row_of(&rows, Item::GenerateGpgKey).is_selectable());
}

#[test]
fn gpg_keys_generate_reports_a_bad_form_without_running() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  app.gpg_form.full_name = "Alice".into();
  app.gpg_form.email = "not-an-email".into();
  select_gpg(&mut app, Item::GenerateGpgKey);
  press(&mut app, KeyCode::Enter);
  assert!(
    app.confirm.is_none(),
    "a bad form never asks for confirmation"
  );
  assert!(app.gpg_job.is_none(), "a bad form never starts gpg");
}

#[test]
fn gpg_keys_generate_without_passphrase_asks_before_running() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  app.gpg_form.full_name = "Alice".into();
  app.gpg_form.email = "alice@example.com".into();
  app.gpg_form.protect_with_passphrase = false;
  select_gpg(&mut app, Item::GenerateGpgKey);
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::GenerateGpgNoPassphrase)
  ));
  assert!(
    app.gpg_job.is_none(),
    "nothing runs before the confirmation"
  );
}

#[test]
fn gpg_keys_cancel_clears_every_field_of_the_form() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  app.gpg_form.full_name = "Alice".into();
  app.gpg_form.email = "alice@example.com".into();
  app.gpg_form.comment = "work".into();
  select_gpg(&mut app, Item::CancelGpgForm);
  press(&mut app, KeyCode::Enter);
  let order = crate::system::date::DateOrder::for_locale(&app.lang.locale());
  assert_eq!(
    app.gpg_form,
    crate::system::gpg_keys::GpgForm::new(order),
    "Cancel returns the form to its defaults"
  );
}

#[test]
fn gpg_keys_algorithm_and_size_rows_change_on_enter() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  select_gpg(&mut app, Item::GpgAlgorithm);
  press(&mut app, KeyCode::Enter);
  assert_eq!(
    app.gpg_form.algorithm,
    crate::system::gpg_keys::KeyAlgorithm::Dsa
  );
  assert_eq!(app.gpg_form.key_size, 2048, "DSA starts at 2048 bits");
  select_gpg(&mut app, Item::GpgKeySize);
  press(&mut app, KeyCode::Enter);
  assert_eq!(
    app.gpg_form.key_size, 3072,
    "DSA offers 3072 instead of 4096"
  );
}

#[test]
fn gpg_keys_protect_toggle_clears_the_passphrases_when_turned_off() {
  let mut app = app(Page::GpgFormPassphrase);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  app.gpg_form.passphrase = "correct horse".into();
  app.gpg_form.passphrase_confirm = "correct horse".into();
  select_gpg(&mut app, Item::GpgProtectPassphrase);
  press(&mut app, KeyCode::Enter);
  assert!(!app.gpg_form.protect_with_passphrase);
  assert!(app.gpg_form.passphrase.is_empty());
  assert!(app.gpg_form.passphrase_confirm.is_empty());
}

#[test]
fn gpg_keys_never_expires_turns_the_date_row_off() {
  let mut app = app(Page::GpgFormExpiration);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  assert!(row_of(&app.rows(), Item::GpgExpiry).is_selectable());
  select_gpg(&mut app, Item::GpgNeverExpires);
  press(&mut app, KeyCode::Enter);
  assert!(app.gpg_form.never_expires);
  assert!(!row_of(&app.rows(), Item::GpgExpiry).is_selectable());
}

#[test]
fn gpg_keys_export_opens_the_path_editor_for_the_selected_key() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![gpg_key_entry(
      "Alice",
      "AAAA111122223333AAAA111122223333AAAAAAAA",
    )],
    ..Default::default()
  };
  select_gpg(&mut app, Item::ExportGpgKey(0));
  press(&mut app, KeyCode::Enter);
  assert!(
    app.admin.editor.is_some(),
    "Enter on Export opens the path editor"
  );
}

#[test]
fn gpg_keys_list_shows_only_information_without_focus() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![gpg_key_entry(
      "Alice",
      "AAAA111122223333AAAA111122223333AAAAAAAA",
    )],
    ..Default::default()
  };
  let rows = app.rows();
  let info: Vec<_> = rows
    .iter()
    .filter(|row| row.kind() == RowKind::Info)
    .collect();
  assert!(!info.is_empty());
  assert!(info.iter().all(|row| !row.is_selectable()));
}

#[test]
fn gpg_keys_search_matches_the_identity_and_keeps_the_listing_index() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![
      gpg_key_entry(
        "Alice <alice@example.com>",
        "AAAA111122223333AAAA111122223333AAAAAAAA",
      ),
      gpg_key_entry(
        "Bob <bob@example.com>",
        "CCCC111122223333CCCC111122223333CCCCCCCC",
      ),
    ],
    ..Default::default()
  };
  app.search = "bob".into();
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.id() == Some(&Item::ExportGpgKey(1)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ExportGpgKey(0)))
  );
}

#[test]
fn gpg_keys_without_the_launcher_only_informs() {
  let app = app(Page::GpgKeys);
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
}

#[test]
fn gpg_key_list_keeps_the_search_on_top_without_an_action_bar() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![
      gpg_key_entry(
        "Alice <alice@example.com>",
        "AAAA111122223333AAAA111122223333AAAAAAAA",
      ),
      gpg_key_entry(
        "Bob <bob@example.com>",
        "CCCC111122223333CCCC111122223333CCCCCCCC",
      ),
    ],
    ..Default::default()
  };
  assert!(app.search_on_top());
  assert!(!app.has_action_bar());
  app.begin_search();
  for character in "bob".chars() {
    app.push_search(character);
  }
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.id() == Some(&Item::ExportGpgKey(1)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ExportGpgKey(0)))
  );
}

#[test]
fn gpg_key_block_offers_edit_copy_export_and_delete() {
  let mut app = app(Page::GpgKeyList);
  gpg_keys_with_two_secrets(&mut app);
  let rows = app.rows();
  for item in [
    Item::OpenGpgKeyEdit(0),
    Item::ExportGpgPrivate(0),
    Item::CopyGpgPublic(0),
    Item::CopyGpgFingerprint(0),
    Item::DeleteGpgKey(0),
  ] {
    assert!(
      rows.iter().any(|row| row.id() == Some(&item)),
      "missing {item:?}"
    );
  }
  assert_eq!(
    row_of(&rows, Item::DeleteGpgKey(0)).kind(),
    RowKind::Destructive
  );
}

#[test]
fn a_public_only_key_offers_no_private_export() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![crate::system::gpg_keys::GpgKeyEntry {
      secret: false,
      ..gpg_key_entry("Carol <carol@example.com>", FP_FIRST)
    }],
    ..Default::default()
  };
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ExportGpgPrivate(0)))
  );
}

#[test]
fn deleting_a_key_asks_before_it_runs() {
  let mut app = app(Page::GpgKeyList);
  gpg_keys_with_two_secrets(&mut app);
  select_gpg(&mut app, Item::DeleteGpgKey(0));
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::DeleteGpgKey(ref fingerprint)) if fingerprint == FP_FIRST
  ));
  assert!(
    app.gpg_action.is_none(),
    "nothing runs before the confirmation"
  );
}

#[test]
fn the_edit_page_follows_the_fingerprint_of_the_key() {
  let mut app = app(Page::GpgKeyList);
  gpg_keys_with_two_secrets(&mut app);
  select_gpg(&mut app, Item::OpenGpgKeyEdit(1));
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::GpgKeyEdit);
  assert_eq!(app.gpg_edit_fingerprint.as_deref(), Some(FP_SECOND));
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::GpgEditPasswd))
  );
}

#[test]
fn the_edit_page_of_a_vanished_key_offers_no_action() {
  let mut app = app(Page::GpgKeyEdit);
  gpg_keys_with_two_secrets(&mut app);
  app.gpg_edit_fingerprint = Some("FFFF000000000000FFFF000000000000FFFF0000".into());
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
}

#[test]
fn a_public_only_key_cannot_change_its_passphrase() {
  let mut app = app(Page::GpgKeyEdit);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![crate::system::gpg_keys::GpgKeyEntry {
      secret: false,
      ..gpg_key_entry("Carol <carol@example.com>", FP_FIRST)
    }],
    ..Default::default()
  };
  app.gpg_edit_fingerprint = Some(FP_FIRST.into());
  assert!(!row_of(&app.rows(), Item::GpgEditPasswd).is_selectable());
}

#[test]
fn sign_choices_are_the_other_keys_with_a_private_part() {
  let mut app = app(Page::GpgEditSign);
  gpg_keys_with_two_secrets(&mut app);
  app.gpg_edit_fingerprint = Some(FP_FIRST.into());
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.id() == Some(&Item::GpgSignWith(1)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::GpgSignWith(0)))
  );
}

#[test]
fn revoking_asks_for_a_reason_and_confirms_it() {
  let mut app = app(Page::GpgEditRevoke);
  gpg_keys_with_two_secrets(&mut app);
  app.gpg_edit_fingerprint = Some(FP_FIRST.into());
  select_gpg(
    &mut app,
    Item::GpgRevokeWith(crate::system::gpg_keys::RevokeReason::Unused),
  );
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::RevokeGpgKey(ref fingerprint, crate::system::gpg_keys::RevokeReason::Unused))
      if fingerprint == FP_FIRST
  ));
  assert!(app.gpg_action.is_none());
}

#[test]
fn an_expiry_in_the_past_is_refused_before_gpg_runs() {
  let mut app = app(Page::GpgEditExpiration);
  gpg_keys_with_two_secrets(&mut app);
  app.gpg_edit_fingerprint = Some(FP_FIRST.into());
  app.gpg_edit_never = false;
  app.gpg_edit_expiry = "2001/01/01".into();
  select_gpg(&mut app, Item::ApplyGpgExpiry);
  press(&mut app, KeyCode::Enter);
  assert!(app.gpg_action.is_none(), "a past date never starts gpg");
}

#[test]
fn the_main_page_offers_the_private_key_import_next_to_the_public_one() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Import);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  select_gpg(&mut app, Item::ImportGpgPrivate);
  press(&mut app, KeyCode::Enter);
  assert!(matches!(app.confirm, Some(PendingAction::ImportGpgPrivate)));
}

#[test]
fn a_launcher_older_than_the_page_offers_no_key_action() {
  let mut edit = app(Page::GpgKeyEdit);
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    outdated: true,
    entries: vec![gpg_key_entry("Alice", FP_FIRST)],
    ..Default::default()
  };
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
  edit.gpg_keys = app.gpg_keys.clone();
  edit.gpg_edit_fingerprint = Some(FP_FIRST.into());
  assert!(edit.rows().iter().all(|row| row.id().is_none()));
}

#[test]
fn exports_propose_the_fingerprint_as_the_file_name() {
  let mut app = app(Page::GpgKeyList);
  gpg_keys_with_two_secrets(&mut app);
  select_gpg(&mut app, Item::ExportGpgKey(0));
  press(&mut app, KeyCode::Enter);
  let public = app
    .admin
    .editor
    .take()
    .expect("public export opens the path editor");
  assert!(
    public.text().ends_with(&format!("/{FP_FIRST}.asc")),
    "{}",
    public.text()
  );

  app.open_confirm(PendingAction::ExportGpgPrivate(FP_FIRST.into()));
  press(&mut app, KeyCode::Char('y'));
  let private = app
    .admin
    .editor
    .take()
    .expect("private export opens the path editor");
  assert!(
    private
      .text()
      .ends_with(&format!("/{FP_FIRST}-private.asc")),
    "{}",
    private.text()
  );
}
