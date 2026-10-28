//! User pages: account lists, creating and editing users, passwords, administrator and autologin toggles.

use super::*;

#[test]
fn user_group_selection_preserves_primary_group() {
  let mut app = app(Page::UserGroups);
  let rows = app.rows();
  assert!(!rows[0].is_selectable(), "primary group row is disabled");
  assert_eq!(rows[0].kind(), RowKind::Toggle { on: true });
  app.admin_activate(Item::GroupOption(0));
  assert_eq!(strings(&app.admin.user["groups"]), vec!["wheel"]);
  app.normalize_selection();
  select(&mut app, Item::GroupOption(2));
  press(&mut app, KeyCode::Enter);
  assert_eq!(strings(&app.admin.user["groups"]), vec!["wheel", "audio"]);
  press(&mut app, KeyCode::Char(' '));
  assert_eq!(strings(&app.admin.user["groups"]), vec!["wheel"]);
}

#[test]
fn password_mismatch_stays_local_and_passwords_are_masked() {
  let mut app = app(Page::User);
  app.admin.passwords = ["old secret".into(), "new secret".into(), "different".into()];
  app.admin_activate(Item::SaveUser);
  assert!(app.error_modal.is_some());
  assert!(app.admin.pending.is_none());
  for row in app.rows() {
    assert!(!row.detail_text().unwrap_or_default().contains("secret"));
  }
}

#[test]
fn system_users_are_optional_and_own_user_is_visible() {
  let app = app(Page::Users);
  assert_eq!(app.admin.users(false).len(), 1);
  assert_eq!(app.admin.users(true).len(), 2);
}

#[test]
fn users_overview_is_separate_from_account_list() {
  let mut app = app(Page::Users);
  let labels = app
    .rows()
    .iter()
    .map(|row| row.label().to_string())
    .collect::<Vec<_>>();
  assert_eq!(labels, vec!["Create", "List", "System accounts"]);
  select(&mut app, Item::UserList);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::UserList);
  assert_eq!(items(&app), vec![Some(Item::UserEntry(0))]);
  assert_eq!(app.rows()[0].label(), "alice");
  assert_eq!(app.selected_item(), Some(Item::UserEntry(0)));
}

#[test]
fn r_reloads_the_account_lists() {
  let mut app = app(Page::SystemUsers);
  press(&mut app, KeyCode::Char('r'));
  assert!(app.admin.busy());
}

#[test]
fn user_search_keeps_the_original_index() {
  let mut app = app(Page::SystemUsers);
  app.search = "ali".into();
  assert_eq!(items(&app), vec![Some(Item::UserEntry(1))]);
  app.admin_activate(Item::UserEntry(1));
  assert_eq!(app.page(), Page::User);
  assert_eq!(text(&app.admin.user, "user"), "alice");
}

#[test]
fn create_user_screen_has_masked_password_rows_and_routes_to_editor() {
  let mut app = app(Page::CreateUser);
  let rows = app.rows();
  let labels = rows.iter().map(|row| row.label()).collect::<Vec<_>>();
  // Same two blocks as the user page: User Data, then Actions.
  assert_eq!(
    labels,
    vec![
      tr(app.lang, "control_center.avatar_image"),
      tr(app.lang, "control_center.username"),
      tr(app.lang, "control_center.full_name"),
      tr(app.lang, "control_center.shell"),
      tr(app.lang, "control_center.home_directory"),
      tr(app.lang, "control_center.password"),
      tr(app.lang, "control_center.user_groups"),
      tr(app.lang, "control_center.make_admin"),
      tr(app.lang, "control_center.actions"),
      tr(app.lang, "control_center.create"),
      tr(app.lang, "control_center.cancel"),
    ]
  );
  assert!(rows[8].is_section());
  assert_eq!(app.selected_item(), Some(Item::AvatarImage));
  // The username has its own page; the password page only asks the new one.
  select(&mut app, Item::Username);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::UserUsername);
  press(&mut app, KeyCode::Esc);
  select(&mut app, Item::ChangePassword);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::UserPassword);
  select(&mut app, Item::Password(1));
  press(&mut app, KeyCode::Enter);
  let editor = app.admin.editor.take().expect("password row opens editor");
  assert_eq!(editor.target, EditTarget::Password(1));
  assert!(editor.value.is_empty());
}

#[test]
fn create_user_button_is_dynamic_and_validates_passwords() {
  let mut app = app(Page::CreateUser);
  app.admin.user["user"] = json!("carol");
  let pt = Lang::for_locale("pt-BR");
  let create_label = |app: &App| {
    app
      .admin
      .rows(Page::CreateUser, pt, "")
      .iter()
      .find(|row| row.id() == Some(&Item::CreateAccount))
      .map(|row| row.label().to_string())
      .unwrap()
  };
  assert_eq!(create_label(&app), tr(pt, "control_center.create"));
  app.admin.passwords[1] = "segredo".into();
  app.admin.passwords[2] = "segredo".into();
  assert_eq!(
    create_label(&app),
    tr(pt, "control_center.create"),
    "the label does not change"
  );
  app.admin_activate(Item::CreateAccount);
  assert!(
    app.admin.pending.is_some(),
    "valid passwords must reach the confirm flow"
  );
  let payload = app.admin.pending.as_ref().unwrap().1.clone();
  assert_eq!(payload["password"].as_str(), Some("segredo"));
  app.admin.pending = None;

  app.admin.passwords[1] = "a".into();
  app.admin.passwords[2] = "b".into();
  app.admin_activate(Item::CreateAccount);
  assert!(
    app.error_modal.is_some(),
    "mismatched passwords must be rejected"
  );
  assert!(app.admin.pending.is_none());
  app.error_modal = None;

  app.admin.passwords[1].clear();
  app.admin.passwords[2].clear();
  app.admin_activate(Item::CreateAccount);
  assert!(
    app.admin.pending.is_some(),
    "empty passwords keep the locked-account confirm flow"
  );
}

#[test]
/// The user page follows the approved layout: titled sections, read-only
/// identity as Info, the draft fields and their Save row, the password and
/// avatar actions, and the deletions in the Danger zone at the end.
fn user_page_layout_has_info_block_and_actions_block() {
  let mut app = app(Page::User);
  app.admin.user["name"] = json!("Alice Liddell");
  let rows = app.rows();
  let sections: Vec<&str> = rows
    .iter()
    .filter(|row| row.is_section())
    .map(|row| row.label())
    .collect();
  assert_eq!(sections, vec![tr(app.lang, "control_center.actions")]);
  assert!(rows.iter().all(|row| !row.label().starts_with("--")));
  let infos: Vec<&str> = rows
    .iter()
    .filter(|row| row.kind() == RowKind::Info)
    .map(|row| row.label())
    .collect();
  assert_eq!(
    infos,
    vec![
      "Username",
      tr(app.lang, "control_center.uid_gid"),
      tr(app.lang, "control_center.home_directory"),
    ]
  );
  let selectable: Vec<Item> = rows
    .iter()
    .filter(|row| row.is_selectable())
    .filter_map(|row| row.id().copied())
    .collect();
  assert_eq!(
    selectable,
    vec![
      Item::AvatarImage,
      Item::FullName,
      Item::Shell,
      Item::PrimaryGroup,
      Item::SupplementaryGroups,
      Item::UserAdmin,
      Item::ToggleAutoLogin,
      Item::ChangePassword,
      Item::SaveUser,
      Item::CancelUser,
      Item::DeleteUserMenu,
    ]
  );
  let kind = |item: Item| {
    rows
      .iter()
      .find(|row| row.id() == Some(&item))
      .map(|row| row.kind())
      .unwrap()
  };
  assert_eq!(kind(Item::AvatarImage), RowKind::Submenu);
  assert_eq!(kind(Item::FullName), RowKind::Submenu);
  assert_eq!(kind(Item::Shell), RowKind::Submenu);
  assert_eq!(kind(Item::CancelUser), RowKind::Action);
  assert_eq!(kind(Item::DeleteUserMenu), RowKind::Destructive);
  let save = rows
    .iter()
    .position(|row| row.id() == Some(&Item::SaveUser))
    .unwrap();
  assert_eq!(rows[save - 1].kind(), RowKind::Separator);
  assert!(rows[save - 2].is_section(), "Save starts the Actions block");
}

#[test]
fn readonly_user_and_group_rows_are_not_selectable() {
  let mut user_app = app(Page::User);
  assert_eq!(user_app.selected_item(), Some(Item::AvatarImage));
  press(&mut user_app, KeyCode::Up);
  assert_eq!(
    user_app.selected_item(),
    Some(Item::AvatarImage),
    "Info rows above are skipped"
  );

  let group_app = app(Page::Group);
  let rows = group_app.rows();
  assert!(!rows[0].is_selectable(), "section title");
  assert_eq!(rows[1].id(), Some(&Item::GroupName));
  assert_eq!(rows[2].kind(), RowKind::Info, "GID");
}

#[test]
fn group_members_picker_toggles_users_in_list() {
  let mut app = app(Page::GroupMembers);
  app.admin.accounts = json!({
    "actor_is_admin": true,
    "users": [
      {"user":"alice","uid":1000,"shell":"/bin/bash","groups":["users"],"password":"x","locked":false},
      {"user":"bob","uid":1001,"shell":"/bin/bash","groups":["users"],"password":"x","locked":false}
    ],
    "groups": [],
    "group_details": []
  });
  app.admin.group = json!({"name":"dev","gid":1000,"members":["alice"]});
  assert_eq!(strings(&app.admin.group["members"]), vec!["alice"]);
  assert_eq!(app.rows()[0].kind(), RowKind::Toggle { on: true });
  select(&mut app, Item::Member(1));
  press(&mut app, KeyCode::Enter);
  assert_eq!(strings(&app.admin.group["members"]), vec!["alice", "bob"]);
  press(&mut app, KeyCode::Enter);
  assert_eq!(strings(&app.admin.group["members"]), vec!["alice"]);
}

#[test]
fn toggle_autologin_confirms_the_opposite_of_the_current_state() {
  let mut app = app(Page::User);
  assert!(!app.admin.is_autologin_user("alice"));
  app.admin_activate(Item::ToggleAutoLogin);
  assert_eq!(
    app.admin.pending,
    Some((
      false,
      json!({"action":"auto-login", "user":"alice", "enabled":true})
    ))
  );
  app.cancel_modal();

  app.admin.accounts["autologin_user"] = json!("alice");
  assert!(app.admin.is_autologin_user("alice"));
  app.admin_activate(Item::ToggleAutoLogin);
  assert_eq!(
    app.admin.pending,
    Some((
      false,
      json!({"action":"auto-login", "user":"alice", "enabled":false})
    ))
  );
}

#[test]
fn administrator_toggle_uses_wheel_when_the_system_has_it() {
  let mut app = app(Page::User);
  // alice is in wheel, so she is an administrator; turning it off leaves wheel.
  app.admin_activate(Item::UserAdmin);
  assert!(strings(&app.admin.user["groups"]).is_empty());
  app.admin_activate(Item::UserAdmin);
  assert_eq!(strings(&app.admin.user["groups"]), vec!["wheel"]);
}

#[test]
fn administrator_toggle_falls_back_to_sudo_without_wheel() {
  let mut app = app(Page::User);
  app.admin.accounts["groups"] = json!(["users", "sudo"]);
  app.admin.user["groups"] = json!([]);
  app.admin_activate(Item::UserAdmin);
  assert_eq!(strings(&app.admin.user["groups"]), vec!["sudo"]);
}

#[test]
fn up_down_walk_the_info_list_and_left_right_walk_the_buttons() {
  let mut app = app(Page::User);
  assert_eq!(app.selected_item(), Some(Item::AvatarImage));
  for expected in [
    Item::FullName,
    Item::Shell,
    Item::PrimaryGroup,
    Item::SupplementaryGroups,
    Item::UserAdmin,
    Item::ToggleAutoLogin,
    Item::ChangePassword,
  ] {
    press(&mut app, KeyCode::Down);
    assert_eq!(app.selected_item(), Some(expected));
  }
  press(&mut app, KeyCode::Down);
  assert_eq!(
    app.selected_item(),
    Some(Item::ChangePassword),
    "the list ends at the info block"
  );
  assert_eq!(app.user_button_cursor(), None);

  // The bar is Save, Cancel, Delete: the first Right focuses Save.
  press(&mut app, KeyCode::Right);
  assert_eq!(app.user_button_cursor(), Some(0));
  press(&mut app, KeyCode::Right);
  assert_eq!(app.user_button_cursor(), Some(1), "Cancel");
  press(&mut app, KeyCode::Right);
  assert_eq!(app.user_button_cursor(), Some(2), "Delete user");
  press(&mut app, KeyCode::Right);
  assert_eq!(
    app.user_button_cursor(),
    Some(0),
    "Right wraps to the first button"
  );
  press(&mut app, KeyCode::Left);
  assert_eq!(app.user_button_cursor(), Some(2), "Left wraps back");

  // Up gives the focus back to the list and moves it one option up.
  press(&mut app, KeyCode::Up);
  assert_eq!(
    app.user_button_cursor(),
    None,
    "Up gives the focus back to the list"
  );
  assert_eq!(app.selected_item(), Some(Item::ToggleAutoLogin));
  press(&mut app, KeyCode::Tab);
  assert_eq!(
    app.user_button_cursor(),
    Some(0),
    "Tab moves to the buttons"
  );
  press(&mut app, KeyCode::Tab);
  assert_eq!(
    app.user_button_cursor(),
    None,
    "Tab again returns to the list"
  );
}

#[test]
/// Lock and Unlock are exclusive drafts and Expire is a checkbox; none run
/// until Save, and Esc on the page asks before dropping them.
fn password_page_security_options_are_drafts() {
  let mut app = app(Page::UserPassword);
  select(&mut app, Item::LockPassword);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.admin.lock_draft, Some(true));
  select(&mut app, Item::UnlockPassword);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.admin.lock_draft, Some(false), "Unlock replaces Lock");
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.admin.lock_draft, None, "choosing it again clears it");
  select(&mut app, Item::ExpirePassword);
  press(&mut app, KeyCode::Enter);
  assert!(app.admin.expire_draft);
  assert!(
    app.confirm.is_none(),
    "nothing is confirmed or saved while editing"
  );
  assert!(
    app.admin.any_user_draft_changed(),
    "the footer warns about it"
  );
  press(&mut app, KeyCode::Esc);
  assert!(
    app.confirm.is_some(),
    "Esc asks before dropping the options"
  );
}

#[test]
/// Covers the subpages of the user page: Full name and Password open their
/// own page, and Ok of the password page goes back, keeping the change for Save.
fn user_subpages_open_their_pages_and_ok_keeps_the_password_draft() {
  let mut app = app(Page::User);
  select(&mut app, Item::FullName);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::UserFullName);
  press(&mut app, KeyCode::Esc);
  assert_eq!(app.page(), Page::User, "Esc cancels the Full name page");

  select(&mut app, Item::ChangePassword);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::UserPassword);
  app.admin.passwords = ["old".into(), "new".into(), "new".into()];
  select(&mut app, Item::OkPassword);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::User, "Ok returns to the user page");
  assert!(app.admin.passwords_changed(), "the password waits for Save");
}

#[test]
fn save_changes_is_always_selectable_and_notes_the_draft() {
  let mut app = app(Page::User);
  let save = |app: &App| {
    app
      .rows()
      .into_iter()
      .find(|row| row.id() == Some(&Item::SaveUser))
      .unwrap()
  };
  assert!(save(&app).is_selectable(), "Save is always reachable");
  assert_eq!(save(&app).detail_text(), None, "no draft note while clean");
  app.admin.user["groups"] = json!(["wheel"]);
  assert!(!app.admin.user_draft_changed(), "same groups");
  app.admin.user["name"] = json!("Alice Liddell");
  assert!(save(&app).is_selectable());
  assert_eq!(
    save(&app).detail_text(),
    Some(tr(app.lang, "control_center.draft_changed"))
  );
}

#[test]
fn esc_with_a_changed_user_draft_asks_and_discard_restores_it() {
  let mut app = app(Page::UserList);
  app.admin_activate(Item::UserEntry(0));
  assert_eq!(app.page(), Page::User);
  press(&mut app, KeyCode::Esc);
  assert_eq!(
    app.page(),
    Page::UserList,
    "clean draft leaves without asking"
  );

  app.admin_activate(Item::UserEntry(0));
  app.admin.user["name"] = json!("Changed");
  press(&mut app, KeyCode::Esc);
  assert!(matches!(app.confirm, Some(PendingAction::DiscardDraft)));
  assert!(
    !app.confirm_focus.is_confirm_focused(),
    "focus starts on Cancel"
  );
  press(&mut app, KeyCode::Enter);
  assert_eq!(
    app.page(),
    Page::User,
    "Cancel keeps the draft and the page"
  );
  assert_eq!(text(&app.admin.user, "name"), "Changed");

  press(&mut app, KeyCode::Esc);
  press(&mut app, KeyCode::Char('y'));
  assert_eq!(app.page(), Page::UserList);
  assert_eq!(text(&app.admin.user, "name"), "Alice");
}

#[test]
fn subpages_of_the_same_draft_do_not_ask() {
  let mut app = app(Page::User);
  app.admin_activate(Item::SupplementaryGroups);
  app.admin_activate(Item::GroupOption(2));
  assert!(app.admin.user_draft_changed());
  press(&mut app, KeyCode::Esc);
  assert_eq!(app.page(), Page::User);
  assert!(app.confirm.is_none());
  app.admin_activate(Item::Shell);
  app.admin_activate(Item::ShellOption(1));
  assert_eq!(app.page(), Page::User, "choosing a shell goes back");
  assert_eq!(text(&app.admin.user, "shell"), "/bin/zsh");
}

#[test]
fn reload_keeps_a_changed_user_draft() {
  let mut app = app(Page::User);
  app.admin.user["name"] = json!("Changed");
  let (sender, receiver) = mpsc::channel();
  app.admin.operation = "snapshot".into();
  app.admin.worker = Some(receiver);
  sender
    .send(Ok((false, app.admin.accounts.clone())))
    .unwrap();
  assert!(app.admin.poll().is_some_and(|result| result.is_ok()));
  assert_eq!(text(&app.admin.user, "name"), "Changed");

  app.admin.user["name"] = json!("Alice");
  let mut accounts = app.admin.accounts.clone();
  accounts["users"][1]["shell"] = json!("/bin/zsh");
  let (sender, receiver) = mpsc::channel();
  app.admin.worker = Some(receiver);
  sender.send(Ok((false, accounts))).unwrap();
  app.admin.poll();
  assert_eq!(
    text(&app.admin.user, "shell"),
    "/bin/zsh",
    "a clean draft follows the reload"
  );
}

#[test]
fn pages_and_password_editor_render_at_minimum_and_large_sizes() {
  use ratatui::{Terminal, backend::TestBackend};
  for (width, height) in [(60, 15), (120, 40)] {
    for page in [
      Page::Users,
      Page::UserList,
      Page::SystemUsers,
      Page::User,
      Page::CreateUser,
      Page::UserGroups,
      Page::UserShell,
      Page::UserPrimaryGroup,
      Page::UserPassword,
      Page::UserAvatar,
      Page::UserFullName,
      Page::UserDelete,
      Page::Groups,
      Page::GroupList,
      Page::SystemGroups,
      Page::Group,
      Page::CreateGroup,
    ] {
      let mut app = app(page);
      let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
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
      assert!(rendered.chars().any(|cell| cell != ' '));
      // The user page draws its action buttons, so only the other pages
      // must not show bracketed button labels.
      if !matches!(page, Page::User | Page::CreateUser) {
        assert!(
          !rendered.replace("[ ]", "").contains("[ "),
          "{page:?} has no button labels"
        );
      }
      if page == Page::User && height == 40 {
        assert!(rendered.contains("Delete"));
        assert!(rendered.contains("Actions"));
        assert!(!rendered.contains("--"));
      }
      app.admin.editor = Some(Editor::new(
        "Password".into(),
        "private-password".into(),
        EditTarget::Password(1),
        false,
      ));
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
      assert!(!rendered.contains("private-password"));
      assert!(rendered.contains("****************"));
    }
  }
}
