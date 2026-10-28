//! Confirmations and drafts: cancel by default, danger style and rendering sizes.

use super::*;

#[test]
fn destructive_actions_default_to_cancel_and_never_run_on_selection() {
  let mut app = app(Page::User);
  // Save, Cancel, then Delete user.
  press(&mut app, KeyCode::Right);
  press(&mut app, KeyCode::Right);
  press(&mut app, KeyCode::Right);
  assert!(
    app.confirm.is_none(),
    "moving the focus does not run anything"
  );
  press(&mut app, KeyCode::Enter);
  assert_eq!(
    app.page(),
    Page::UserDelete,
    "Delete asks Keep or Delete home first"
  );
  select(&mut app, Item::DeleteUserAndHome);
  press(&mut app, KeyCode::Enter);
  assert!(app.confirm.is_some());
  assert!(!app.confirm_focus.is_confirm_focused());
  assert!(!app.admin.busy());
  press(&mut app, KeyCode::Enter);
  assert!(app.confirm.is_none(), "Enter right after opening cancels");
  assert!(app.admin.pending.is_none());
  assert!(!app.admin.busy());
}

#[test]
fn multiline_editor_moves_between_lines_and_accepts_paste() {
  let mut app = app(Page::Firewall);
  app.admin.editor = Some(Editor::new(
    "Rules".into(),
    "abc\nxy".into(),
    EditTarget::Rules,
    true,
  ));
  app.admin_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
  assert_eq!(app.admin.editor.as_ref().unwrap().cursor, 2);
  app.admin_paste("q?\niptables -A INPUT -j ACCEPT");
  assert!(
    app
      .admin
      .editor
      .as_ref()
      .unwrap()
      .value
      .iter()
      .collect::<String>()
      .contains("q?\niptables")
  );
  app.admin_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
  assert!(app.admin.editor.is_none());
  assert!(app.admin.pending.is_none());
}

#[test]
fn save_and_delete_rows_confirm_without_running() {
  for (page, item) in [
    (Page::User, Item::SaveUser),
    (Page::User, Item::ToggleAutoLogin),
    (Page::UserDelete, Item::DeleteUser),
    (Page::UserDelete, Item::DeleteUserAndHome),
  ] {
    let mut app = app(page);
    app.admin.user["name"] = json!("Alice Liddell");
    app.admin_activate(item);
    assert!(app.confirm.is_some(), "{item:?} asks first");
    assert!(!app.admin.busy());
  }
}

#[test]
fn confirmation_uses_the_single_component_keys() {
  let mut app = app(Page::User);
  app.admin.user["name"] = json!("Changed");
  press(&mut app, KeyCode::Esc);
  assert!(app.confirm.is_some());
  press(&mut app, KeyCode::Char('n'));
  assert!(app.confirm.is_none(), "n cancels");
  press(&mut app, KeyCode::Esc);
  press(&mut app, KeyCode::Down);
  assert!(
    !app.confirm_focus.is_confirm_focused(),
    "Cancel is the lower row"
  );
  press(&mut app, KeyCode::Char('k'));
  assert!(app.confirm_focus.is_confirm_focused());
  press(&mut app, KeyCode::Tab);
  assert!(!app.confirm_focus.is_confirm_focused());
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  assert!(app.confirm.is_none());
  assert_eq!(
    app.page(),
    Page::Main,
    "Enter on Confirm discards and leaves"
  );
  assert_eq!(text(&app.admin.user, "name"), "Alice");
}

#[test]
fn deletions_use_the_danger_style_and_other_actions_do_not() {
  let mut user = app(Page::User);
  for (item, danger) in [(Item::DeleteUser, true), (Item::DeleteUserAndHome, true)] {
    user.admin_activate(item);
    let (_, message, confirm, is_danger) = user.confirm_dialog().unwrap();
    assert_eq!(is_danger, danger, "{item:?}");
    if danger {
      assert!(message.contains("irreversible"), "{message}");
      assert_eq!(confirm, tr(user.lang, "control_center.yes"));
    } else {
      assert!(message.ends_with("alice?"), "{message}");
    }
    user.cancel_modal();
  }
  let mut group = app(Page::Group);
  group.admin.group = json!({"name":"wheel","original":"wheel","gid":998,"members":[]});
  group.admin_activate(Item::DeleteGroup);
  assert!(group.confirm_dialog().unwrap().3);
}

#[test]
fn confirmation_renders_vertical_rows_without_button_labels() {
  use ratatui::{Terminal, backend::TestBackend};
  let mut app = app(Page::User);
  app.admin_activate(Item::DeleteUser);
  let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
  terminal
    .draw(|frame| crate::ui::draw(&mut app, frame))
    .unwrap();
  let rendered: String = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect();
  assert!(rendered.contains(tr(app.lang, "control_center.yes")));
  assert!(rendered.contains(tr(app.lang, "control_center.no")));
  assert!(rendered.contains("irreversible"));
  assert!(!rendered.contains(tr(app.lang, "control_center.cancel_f8378f")));
}
