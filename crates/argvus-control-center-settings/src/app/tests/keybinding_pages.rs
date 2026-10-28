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
  let mut app = app(Page::Keybindings);
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
  let mut app = app(Page::Keybindings);
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
  let app = app(Page::Keybindings);
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
  let mut app = app(Page::Keybindings);
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
  assert_eq!(app.page(), Page::Keybindings);
}
