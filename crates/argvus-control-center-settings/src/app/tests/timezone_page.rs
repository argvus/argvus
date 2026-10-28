//! Time zone page: the draft zone and its Apply action.

use super::*;

#[test]
fn timezone_enter_only_edits_the_draft_until_apply() {
  let mut app = timezone_app();
  assert_eq!(
    app.selected_item(),
    Some(Item::Zone(1)),
    "the cursor starts on the system zone"
  );
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.timezone_draft, "America/Sao_Paulo");
  assert_eq!(
    app.datetime.time_zone, "Europe/Lisbon",
    "the system zone is untouched before Apply"
  );
  assert!(app.timezone_changed());
}

#[test]
fn timezone_apply_is_dimmed_until_a_change() {
  let mut app = timezone_app();
  let apply = |app: &App| app.action_rows()[0].clone();
  assert_eq!(apply(&app).id(), Some(&Item::ApplyTimezone));
  assert!(!apply(&app).is_selectable());
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  assert!(apply(&app).is_selectable());
}

#[test]
fn timezone_apply_lives_in_the_action_bar_not_in_the_list() {
  let app = timezone_app();
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::ApplyTimezone)),
    "the list cursor never walks into Apply"
  );
  assert_eq!(app.action_rows().len(), 1);
}

#[test]
fn timezone_tab_focuses_apply_only_when_it_is_enabled() {
  let mut app = timezone_app();
  press(&mut app, KeyCode::Tab);
  assert_eq!(
    app.action_cursor(),
    None,
    "Apply is dimmed: nothing to focus"
  );
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  press(&mut app, KeyCode::Tab);
  assert_eq!(app.action_cursor(), Some(0), "Tab focuses Apply");
}

#[test]
fn timezone_slash_focuses_the_top_search_and_filters_the_list() {
  let mut app = timezone_app();
  assert!(app.search_on_top());
  press(&mut app, KeyCode::Char('/'));
  assert!(app.searching, "/ focuses the search box");
  for character in "sao".chars() {
    press(&mut app, KeyCode::Char(character));
  }
  let ids: Vec<Item> = app
    .rows()
    .iter()
    .filter_map(|row| row.id().copied())
    .collect();
  assert_eq!(ids, [Item::Zone(0)], "only the matching zone is listed");
}

#[test]
fn timezone_draft_asks_before_leaving_with_pending_edits() {
  let mut app = timezone_app();
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  press(&mut app, KeyCode::Esc);
  assert!(matches!(app.confirm, Some(PendingAction::DiscardDraft)));
  assert_eq!(app.page(), Page::TimeZone);
}

#[test]
fn timezone_discard_restores_the_system_zone_in_the_draft() {
  let mut app = timezone_app();
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  app.discard_and_leave();
  assert_eq!(app.timezone_draft, "Europe/Lisbon");
  assert!(!app.timezone_changed());
}

#[test]
fn timezone_arrows_on_the_list_move_the_focus_to_apply() {
  let mut app = timezone_app();
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  let before = app.selected_item();
  press(&mut app, KeyCode::Right);
  assert_eq!(app.action_cursor(), Some(0), "→ focuses Apply, as Tab does");
  assert_eq!(app.selected_item(), before, "the list did not move");
  press(&mut app, KeyCode::Up);
  assert_eq!(app.action_cursor(), None, "Up gives the focus back");
  press(&mut app, KeyCode::Left);
  assert_eq!(app.action_cursor(), Some(0), "← focuses Apply too");
}

#[test]
fn timezone_arrows_do_nothing_while_apply_is_dimmed() {
  let mut app = timezone_app();
  let before = app.selected_item();
  press(&mut app, KeyCode::Left);
  press(&mut app, KeyCode::Right);
  assert_eq!(app.action_cursor(), None, "nothing to focus");
  assert_eq!(app.selected_item(), before, "the list did not move");
}
