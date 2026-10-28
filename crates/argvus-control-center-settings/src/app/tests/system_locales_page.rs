//! System locales page: marks, Apply and leaving with pending edits.

use super::*;

#[test]
fn system_locales_enter_toggles_the_mark_until_apply() {
  let mut app = system_locales_app();
  press(&mut app, KeyCode::Down);
  assert_eq!(app.selected_item(), Some(Item::LocaleGen(1)));
  press(&mut app, KeyCode::Enter);
  assert!(app.selected_locales.contains("pt_BR.UTF-8 UTF-8"));
  assert!(app.confirm.is_none(), "Enter toggles instead of applying");
}

#[test]
fn system_locales_apply_lives_in_the_action_bar_not_in_the_list() {
  let app = system_locales_app();
  assert!(app.has_action_bar());
  assert!(app.search_on_top());
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::ApplyLocales)),
    "the list cursor never walks into Apply"
  );
  assert_eq!(app.action_rows().len(), 1);
  assert_eq!(app.action_rows()[0].id(), Some(&Item::ApplyLocales));
}

#[test]
fn system_locales_apply_is_dimmed_until_a_change() {
  let mut app = system_locales_app();
  let apply = |app: &App| app.action_rows()[0].clone();
  assert!(
    !apply(&app).is_selectable(),
    "Apply is dimmed without changes"
  );
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Enter);
  assert!(apply(&app).is_selectable());
}

#[test]
fn system_locales_tab_focuses_apply_only_when_it_is_enabled() {
  let mut app = system_locales_app();
  press(&mut app, KeyCode::Tab);
  assert_eq!(
    app.action_cursor(),
    None,
    "Apply is dimmed: nothing to focus"
  );
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Enter);
  press(&mut app, KeyCode::Tab);
  assert_eq!(app.action_cursor(), Some(0), "Tab focuses Apply");
}

#[test]
fn system_locales_apply_asks_for_confirmation_before_running() {
  let mut app = system_locales_app();
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Enter);
  press(&mut app, KeyCode::Tab);
  press(&mut app, KeyCode::Enter);
  assert!(
    matches!(app.confirm, Some(PendingAction::ApplySystemLocales)),
    "running Apply opens the confirmation first"
  );
}

#[test]
fn system_locales_slash_focuses_the_top_search_and_filters_the_list() {
  let mut app = system_locales_app();
  press(&mut app, KeyCode::Char('/'));
  assert!(app.searching, "/ focuses the search box");
  for character in "pt_".chars() {
    press(&mut app, KeyCode::Char(character));
  }
  let ids: Vec<Item> = app
    .rows()
    .iter()
    .filter_map(|row| row.id().copied())
    .collect();
  assert_eq!(
    ids,
    [Item::LocaleGen(1)],
    "only the matching locale is listed"
  );
}

#[test]
fn system_locales_draft_asks_before_leaving_with_pending_edits() {
  let mut app = system_locales_app();
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Enter);
  press(&mut app, KeyCode::Esc);
  assert!(matches!(app.confirm, Some(PendingAction::DiscardDraft)));
  assert_eq!(app.page(), Page::SystemLocales);
}

#[test]
fn system_locales_discard_restores_the_system_marks() {
  let mut app = system_locales_app();
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Enter);
  app.discard_and_leave();
  assert_eq!(
    app.selected_locales,
    BTreeSet::from(["en_US.UTF-8 UTF-8".to_string()])
  );
  assert!(!app.locales_changed());
}
