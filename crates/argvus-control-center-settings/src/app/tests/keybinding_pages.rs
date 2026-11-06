//! Keyboard shortcuts pages: search, editing, restoring and conflicts.

use super::*;

#[test]
fn font_size_is_a_value_row_with_arrows_and_shortcuts() {
  let mut app = app(Page::Fonts);
  app.navigation.push(Page::FontSelector(FontTarget::Apps));
  app.pending_size = 13;
  let rows = app.rows();
  assert_eq!(rows[0].id(), Some(&Item::FontSize));
  assert_eq!(rows[0].kind(), RowKind::Value { step: Some(1) });
  assert!(
    app
      .navigation
      .current_mut()
      .menu
      .select(&rows, &Item::FontSize)
  );
  press(&mut app, KeyCode::Right);
  assert_eq!(app.pending_size, 14);
  press(&mut app, KeyCode::Left);
  press(&mut app, KeyCode::Char('-'));
  assert_eq!(app.pending_size, 12);
  assert_eq!(app.page(), Page::FontSelector(FontTarget::Apps));
  press(&mut app, KeyCode::Enter);
  assert!(app.admin.editor.is_some(), "Enter opens the size prompt");
}

#[test]
fn r_on_shortcuts_restores_the_selected_shortcut_after_confirmation() {
  let mut app = app(Page::AllShortcuts);
  let Some(first) = app.visible_keybindings().first().copied() else {
    return; // no keybinding catalog in this environment
  };
  let rows = app.rows();
  assert!(
    app
      .navigation
      .current_mut()
      .menu
      .select(&rows, &Item::Keybinding(first))
  );
  press(&mut app, KeyCode::Char('r'));
  let id = app.keybindings[first].id.clone();
  assert!(matches!(
    &app.confirm,
    Some(PendingAction::RestoreKeybinding { id: pending, leave: false }) if *pending == id
  ));
  assert_eq!(
    app.rows().last().and_then(|row| row.id().copied()),
    Some(Item::RestoreAllShortcuts)
  );
}

#[test]
fn keybindings_search_sits_on_top_without_an_action_bar() {
  let mut app = app(Page::AllShortcuts);
  assert!(app.search_on_top());
  assert!(!app.has_action_bar());
  assert!(app.action_rows().is_empty());
  press(&mut app, KeyCode::Char('/'));
  assert!(app.searching, "/ focuses the search box");
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::RestoreAllShortcuts)),
    "the list keeps its danger zone"
  );
}

#[test]
fn keybinding_edit_footer_has_no_change_or_restore_hints() {
  let app = app(Page::KeybindingEdit);
  let footer = app.footer();
  assert!(
    !footer.contains(tr(app.lang, "control_center.change_shortcut")),
    "{footer}"
  );
  assert!(
    !footer.contains(tr(app.lang, "control_center.restore_default")),
    "{footer}"
  );
}

#[test]
fn keybindings_footer_keeps_restore_default() {
  let app = app(Page::AllShortcuts);
  assert!(
    app
      .footer()
      .contains(tr(app.lang, "control_center.restore_default")),
    "{}",
    app.footer()
  );
}

#[test]
fn shortcut_conflict_is_a_confirmation_that_esc_cancels() {
  let mut app = app(Page::AllShortcuts);
  app.keybinding_conflict = Some(("a".into(), "SUPER + Q".into(), vec!["b".into()]));
  let (title, message, confirm, danger) = app.confirm_dialog().unwrap();
  assert_eq!(
    title,
    tr(app.lang, "control_center.keybindings_conflict_title")
  );
  assert!(message.contains("SUPER + Q"));
  assert_eq!(confirm, tr(app.lang, "control_center.replace"));
  assert!(!danger);
  assert!(app.footer().contains("r "), "{}", app.footer());
  press(&mut app, KeyCode::Enter);
  assert!(!app.has_keybinding_conflict(), "Enter on Cancel cancels");
  app.keybinding_conflict = Some(("a".into(), "SUPER + Q".into(), vec!["b".into()]));
  press(&mut app, KeyCode::Esc);
  assert!(!app.has_keybinding_conflict());
  assert_eq!(app.page(), Page::AllShortcuts);
}

fn custom_binding() -> keybindings::Binding {
  keybindings::custom_binding(
    "custom_editor".into(),
    keybindings::CustomShortcut {
      description: "My editor".into(),
      keys: "SUPER + CTRL + E".into(),
      enabled: true,
      action: keybindings::ShortcutAction::Command {
        command: "editor --new-window".into(),
      },
    },
  )
}

#[test]
fn shortcuts_hub_opens_create_and_all_and_cancel_discards_the_draft() {
  let mut app = app(Page::Keybindings);
  let rows = app.rows();
  assert_eq!(rows.len(), 2);
  assert_eq!(rows[0].id(), Some(&Item::Page(Page::CustomShortcut)));
  assert_eq!(rows[1].id(), Some(&Item::Page(Page::AllShortcuts)));
  app.activate(Item::Page(Page::CustomShortcut));
  let id = app.custom_shortcut.id.clone();
  assert!(id.starts_with("custom_"));
  app.custom_shortcut.description = "Unsaved".into();
  app.activate(Item::CancelCustomShortcut);
  assert_eq!(app.page(), Page::Keybindings);
  assert!(app.custom_shortcut.description.is_empty());
  assert!(app.shortcut_job.is_none());
  app.activate(Item::Page(Page::CustomShortcut));
  assert_ne!(app.custom_shortcut.id, id);
}

#[test]
fn custom_form_actions_and_capture_only_change_a_draft() {
  let mut app = app(Page::CustomShortcut);
  assert!(app.has_action_bar());
  assert!(!app.search_on_top());
  assert!(app.custom_shortcut.id.starts_with("custom_"));
  press(&mut app, KeyCode::Tab);
  assert_eq!(app.action_cursor(), Some(0));
  press(&mut app, KeyCode::Right);
  assert_eq!(app.action_cursor(), Some(1));
  app.begin_custom_shortcut_capture();
  assert!(app.captures_text());
  crate::event::handle(
    &mut app,
    Event::Key(KeyEvent::new(
      KeyCode::Char('q'),
      crossterm::event::KeyModifiers::CONTROL,
    )),
  );
  assert_eq!(app.custom_shortcut.keys, "CTRL + Q");
  assert!(app.shortcut_job.is_none());
  app.begin_custom_shortcut_capture();
  crate::event::handle(
    &mut app,
    Event::Key(KeyEvent::new(
      KeyCode::Enter,
      crossterm::event::KeyModifiers::ALT,
    )),
  );
  assert_eq!(app.custom_shortcut.keys, "ALT + Return");
  app.begin_custom_shortcut_capture();
  press(&mut app, KeyCode::Esc);
  assert_eq!(app.custom_shortcut.keys, "ALT + Return");
}

#[test]
fn custom_list_searches_command_and_edits_all_fields_without_restore() {
  let mut app = app(Page::AllShortcuts);
  app.keybindings = vec![custom_binding()];
  app.search = "new-window".into();
  assert_eq!(app.visible_keybindings(), vec![0]);
  app.activate(Item::Keybinding(0));
  assert_eq!(app.page(), Page::CustomShortcut);
  assert_eq!(app.custom_shortcut.description, "My editor");
  assert_eq!(app.custom_shortcut.command, "editor --new-window");
  assert!(
    !app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::RestoreShortcut))
  );
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::DeleteCustomShortcut))
  );
  let rows = app.rows();
  assert!(
    app
      .navigation
      .current_mut()
      .menu
      .select(&rows, &Item::DeleteCustomShortcut)
  );
  press(&mut app, KeyCode::Enter);
  assert!(app.confirm.is_some());
  press(&mut app, KeyCode::Enter);
  assert!(app.confirm.is_none(), "delete starts focused on Cancel");
  assert!(app.shortcut_job.is_none());
}

#[test]
fn invalid_form_and_protected_conflict_never_start_a_save() {
  let mut app = app(Page::CustomShortcut);
  let id = app.custom_shortcut.id.clone();
  app.save_custom_shortcut();
  assert!(app.error_modal.is_some());
  assert_eq!(app.custom_shortcut.id, id);
  assert!(app.shortcut_job.is_none());
  app.error_modal = None;
  app.custom_shortcut.description = "Example".into();
  app.custom_shortcut.command = "editor".into();
  app.custom_shortcut.keys = "SUPER + CTRL + E".into();
  let mut protected = custom_binding();
  protected.configurable = false;
  app.keybindings = vec![protected];
  app.save_custom_shortcut();
  assert!(app.error_modal.is_some());
  assert!(!app.has_keybinding_conflict());
  assert!(app.shortcut_job.is_none());
}

#[test]
fn custom_conflict_cancels_without_writing_or_losing_the_draft() {
  let mut app = app(Page::CustomShortcut);
  app.custom_shortcut.description = "Another editor".into();
  app.custom_shortcut.command = "other-editor".into();
  app.custom_shortcut.keys = "CTRL + SUPER + E".into();
  app.keybindings = vec![custom_binding()];
  app.save_custom_shortcut();
  assert!(app.has_keybinding_conflict());
  assert!(app.shortcut_job.is_none());
  press(&mut app, KeyCode::Esc);
  assert!(!app.has_keybinding_conflict());
  assert_eq!(app.custom_shortcut.command, "other-editor");
  assert_eq!(app.page(), Page::CustomShortcut);
}

fn finish_shortcut_job(app: &mut App) {
  let deadline = Instant::now() + Duration::from_secs(2);
  while app.shortcut_job.is_some() && Instant::now() < deadline {
    app.poll_shortcut_job();
    std::thread::yield_now();
  }
  assert!(app.shortcut_job.is_none());
}

#[test]
fn saved_apply_failure_preserves_the_id_and_draft_for_retry() {
  let mut app = app(Page::CustomShortcut);
  app.custom_shortcut.description = "My editor".into();
  app.custom_shortcut.command = "editor".into();
  let id = app.custom_shortcut.id.clone();
  app.shortcut_job = Some(super::super::custom_shortcut::ShortcutJob {
    id: id.clone(),
    leave: true,
    deleted: false,
    handle: app
      .jobs
      .spawn(|_| Ok((vec![custom_binding()], Some("reload failed".into())))),
  });
  finish_shortcut_job(&mut app);
  assert_eq!(app.page(), Page::CustomShortcut);
  assert_eq!(app.custom_shortcut.id, id);
  assert!(app.custom_shortcut.existing);
  assert_eq!(app.custom_shortcut.command, "editor");
  assert!(app.error_modal.as_ref().unwrap().contains("Saved"));
}

#[test]
fn saved_shortcut_opens_all_clears_search_and_selects_the_new_item() {
  let mut app = app(Page::Keybindings);
  app.activate(Item::Page(Page::CustomShortcut));
  app.search = "unrelated".into();
  app.shortcut_job = Some(super::super::custom_shortcut::ShortcutJob {
    id: "custom_editor".into(),
    leave: true,
    deleted: false,
    handle: app.jobs.spawn(|_| Ok((vec![custom_binding()], None))),
  });
  finish_shortcut_job(&mut app);
  assert_eq!(app.page(), Page::AllShortcuts);
  assert!(app.search.is_empty());
  assert_eq!(
    app.selected_keybinding().map(|binding| binding.id.as_str()),
    Some("custom_editor")
  );
  app.back();
  assert_eq!(app.page(), Page::Keybindings);
}

#[test]
fn persistence_failure_preserves_the_form_and_stable_id() {
  let mut app = app(Page::CustomShortcut);
  app.custom_shortcut.description = "Not yet saved".into();
  let id = app.custom_shortcut.id.clone();
  app.shortcut_job = Some(super::super::custom_shortcut::ShortcutJob {
    id: id.clone(),
    leave: true,
    deleted: false,
    handle: app.jobs.spawn(|_| Err("permission denied".into())),
  });
  finish_shortcut_job(&mut app);
  assert_eq!(app.page(), Page::CustomShortcut);
  assert_eq!(app.custom_shortcut.id, id);
  assert_eq!(app.custom_shortcut.description, "Not yet saved");
  assert!(!app.custom_shortcut.existing);
  assert!(app.error_modal.as_ref().unwrap().contains("Could not save"));
}

#[test]
fn enabling_an_official_shortcut_checks_custom_conflicts_first() {
  let mut app = app(Page::AllShortcuts);
  let mut official = custom_binding();
  official.id = "window.close".into();
  official.custom = None;
  official.enabled = false;
  app.keybindings = vec![official, custom_binding()];
  let rows = app.rows();
  app
    .navigation
    .current_mut()
    .menu
    .select(&rows, &Item::Keybinding(0));
  app.toggle_selected_keybinding();
  assert!(app.has_keybinding_conflict());
  assert_eq!(app.page(), Page::AllShortcuts);
  assert!(!app.keybindings[0].enabled);
  assert!(app.shortcut_job.is_none());
}

#[test]
fn dispatcher_null_arguments_reach_conflict_confirmation_without_saving() {
  let mut app = app(Page::CustomShortcut);
  app.custom_shortcut.description = "Example".into();
  app.custom_shortcut.keys = "SUPER + CTRL + E".into();
  app.custom_shortcut.dispatcher_action = true;
  app.custom_shortcut.dispatcher = "window.close".into();
  app.custom_shortcut.arguments = "null".into();
  app.keybindings = vec![custom_binding()];
  app.save_custom_shortcut();
  assert!(app.error_modal.is_none());
  assert!(app.has_keybinding_conflict());
  assert!(app.shortcut_job.is_none());
}

#[test]
fn current_shortcut_captures_pressed_keys_instead_of_opening_a_text_editor() {
  for existing in [false, true] {
    let mut app = app(Page::CustomShortcut);
    if existing {
      app.open_custom_shortcut(Some(&custom_binding()));
    }
    let original = app.custom_shortcut.keys.clone();
    let rows = app.rows();
    assert_eq!(row_of(&rows, Item::CustomKeys).kind(), RowKind::Action);
    assert!(
      app
        .navigation
        .current_mut()
        .menu
        .select(&rows, &Item::CustomKeys)
    );
    press(&mut app, KeyCode::Enter);
    assert!(app.is_keybinding_capturing());
    assert!(app.admin.editor.is_none());
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.custom_shortcut.keys, original);
    assert_eq!(app.page(), Page::CustomShortcut);
    press(&mut app, KeyCode::Enter);
    crate::event::handle(
      &mut app,
      Event::Key(KeyEvent::new(
        KeyCode::Char('k'),
        crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT,
      )),
    );
    assert_eq!(app.custom_shortcut.keys, "CTRL + ALT + K");
    assert!(!app.is_keybinding_capturing());
    assert!(app.admin.editor.is_none());
    assert!(
      app.shortcut_job.is_none(),
      "capturing only changes the draft"
    );
    app.activate(Item::CaptureCustomShortcut);
    assert!(
      app.is_keybinding_capturing(),
      "Change shortcut uses the same capture"
    );
  }
}

#[test]
fn capture_reports_a_duplicate_before_description_or_command_are_filled() {
  let mut app = app(Page::CustomShortcut);
  app.keybindings = vec![custom_binding()];
  app.begin_custom_shortcut_capture();
  crate::event::handle(
    &mut app,
    Event::Key(KeyEvent::new(
      KeyCode::Char('e'),
      crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::SUPER,
    )),
  );
  assert!(
    app.has_keybinding_conflict(),
    "the warning belongs to capture, not Create"
  );
  assert!(app.error_modal.is_none());
  assert!(app.custom_shortcut.description.is_empty());
  assert!(app.custom_shortcut.command.is_empty());
  assert!(
    app.custom_shortcut.keys.is_empty(),
    "a conflicting draft waits for confirmation"
  );
  assert!(app.confirm_dialog().unwrap().1.contains("My editor"));
  press(&mut app, KeyCode::Char('y'));
  assert!(!app.has_keybinding_conflict());
  assert_eq!(app.custom_shortcut.keys, "SUPER + CTRL + E");
  assert_eq!(app.custom_shortcut.accepted_conflicts, ["custom_editor"]);
  assert!(
    app.keybindings[0].enabled,
    "accepting the combination must not disable anything yet"
  );
  assert!(
    app.shortcut_job.is_none(),
    "accepting capture must not create the shortcut"
  );
  app.custom_shortcut.description = "New editor".into();
  app.custom_shortcut.command = "other-editor".into();
  app.save_custom_shortcut();
  assert!(
    !app.has_keybinding_conflict(),
    "the acknowledged conflict is not asked twice"
  );
  assert!(app.shortcut_job.is_some());
}

#[test]
fn cancelling_a_duplicate_capture_preserves_the_previous_combination() {
  let mut app = app(Page::CustomShortcut);
  app.custom_shortcut.keys = "CTRL + K".into();
  app.keybindings = vec![custom_binding()];
  app.begin_custom_shortcut_capture();
  crate::event::handle(
    &mut app,
    Event::Key(KeyEvent::new(
      KeyCode::Char('e'),
      crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::SUPER,
    )),
  );
  press(&mut app, KeyCode::Esc);
  assert_eq!(app.custom_shortcut.keys, "CTRL + K");
  assert!(app.custom_shortcut.accepted_conflicts.is_empty());
  assert!(app.shortcut_job.is_none());
}

#[test]
fn accepting_a_different_capture_clears_previous_replacement_permission() {
  let mut app = app(Page::CustomShortcut);
  app
    .custom_shortcut
    .accepted_conflicts
    .push("custom_editor".into());
  app.keybindings = vec![custom_binding()];
  app.begin_custom_shortcut_capture();
  crate::event::handle(
    &mut app,
    Event::Key(KeyEvent::new(
      KeyCode::Char('k'),
      crossterm::event::KeyModifiers::CONTROL,
    )),
  );
  assert_eq!(app.custom_shortcut.keys, "CTRL + K");
  assert!(app.custom_shortcut.accepted_conflicts.is_empty());
  assert!(!app.has_keybinding_conflict());
}

#[test]
fn delete_from_the_destructive_row_confirms_and_starts_a_deletion_job() {
  let mut app = app(Page::AllShortcuts);
  app.keybindings = vec![custom_binding()];
  app.activate(Item::Keybinding(0));
  let rows = app.rows();
  assert!(
    app
      .navigation
      .current_mut()
      .menu
      .select(&rows, &Item::DeleteCustomShortcut)
  );
  press(&mut app, KeyCode::Enter);
  assert!(
    matches!(&app.confirm, Some(PendingAction::DeleteCustomShortcut(id)) if id == "custom_editor")
  );
  assert!(!app.confirm_focus.is_confirm_focused());
  assert!(app.shortcut_job.is_none());
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  let job = app
    .shortcut_job
    .as_ref()
    .expect("confirmed Delete starts the backend");
  assert!(job.deleted);
  assert_eq!(job.id, "custom_editor");
  finish_shortcut_job(&mut app);
  // The process runner is disabled in unit tests: failed persistence must keep the item.
  assert!(
    app
      .error_modal
      .as_ref()
      .unwrap()
      .contains("Could not delete")
  );
  assert_eq!(app.keybindings.len(), 1);
  assert_eq!(app.page(), Page::CustomShortcut);
}

#[test]
fn successful_deletion_removes_only_the_selected_item_and_returns_to_all() {
  let mut app = app(Page::AllShortcuts);
  let mut other = custom_binding();
  other.id = "custom_other".into();
  app.keybindings = vec![custom_binding(), other.clone()];
  app.activate(Item::Keybinding(0));
  app.shortcut_job = Some(super::super::custom_shortcut::ShortcutJob {
    id: "custom_editor".into(),
    leave: true,
    deleted: true,
    handle: app.jobs.spawn(move |_| Ok((vec![other], None))),
  });
  finish_shortcut_job(&mut app);
  assert_eq!(app.page(), Page::AllShortcuts);
  assert_eq!(app.keybindings.len(), 1);
  assert_eq!(app.keybindings[0].id, "custom_other");
  assert_eq!(
    app.status.as_ref().unwrap().text,
    tr(app.lang, "control_center.shortcut_deleted")
  );
}
