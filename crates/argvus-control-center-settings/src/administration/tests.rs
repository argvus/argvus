use super::*;
use argvus_tui::menu::RowKind;
use crossterm::event::Event;

mod confirmations;
mod firewall_pages;
mod group_pages;
mod user_pages;

fn app(page: Page) -> App {
  let mut app = App::with_context(
    Page::Main,
    Lang::for_locale("en-US"),
    crate::theme::Theme::load(),
  );
  app.error_modal = None;
  app.navigation.push(page);
  app.admin.accounts = json!({"actor_uid":1000, "actor_is_admin":true, "shells":["/bin/bash", "/bin/zsh"], "groups":["users", "wheel", "audio"], "group_details":[
    {"name":"audio","gid":986,"members":["alice"]},
    {"name":"users","gid":1000,"members":[]},
    {"name":"wheel","gid":998,"members":["alice"]}
  ], "users":[
    {"user":"root", "uid":0, "gid":0, "name":"root", "shell":"/bin/bash", "primary_group":"root", "groups":[]},
    {"user":"alice", "uid":1000, "gid":1000, "name":"Alice", "shell":"/bin/bash", "primary_group":"users", "groups":["wheel"]}
  ]});
  app.admin.user = app.admin.accounts["users"][1].clone();
  app
}

fn press(app: &mut App, code: KeyCode) {
  crate::event::handle(app, Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

/// Moves the cursor to `item` and asserts that it is selectable.
fn select(app: &mut App, item: Item) {
  let rows = app.rows();
  assert!(
    app.navigation.current_mut().menu.select(&rows, &item),
    "{item:?} is not selectable"
  );
}

fn items(app: &App) -> Vec<Option<Item>> {
  app.rows().iter().map(|row| row.id().copied()).collect()
}

#[test]
/// Cancel on the focused button discards the edits and stays on the user page.
fn focused_cancel_button_discards_the_edits_and_stays() {
  let mut app = app(Page::User);
  app.admin.user["name"] = json!("Alice Liddell");
  // Save, then Cancel.
  press(&mut app, KeyCode::Right);
  press(&mut app, KeyCode::Right);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::User, "Cancel keeps the user page open");
  assert!(!app.admin.user_draft_changed(), "the edits were discarded");
  assert!(app.confirm.is_none(), "Cancel does not ask");
}

#[test]
/// Edits made on the subpages are drafts: nothing runs until Save.
fn subpage_edits_are_drafts_until_save() {
  let mut app = app(Page::UserAvatar);
  app.admin.editor = Some(Editor::new(
    "Avatar".into(),
    "/tmp/face.png".into(),
    EditTarget::Avatar,
    false,
  ));
  press(&mut app, KeyCode::Enter);
  assert!(
    app.confirm.is_none(),
    "the avatar is not saved while editing"
  );
  assert_eq!(
    text(&app.admin.user, "avatar"),
    "/tmp/face.png",
    "held in the draft"
  );
  assert!(!app.admin.busy());
}
