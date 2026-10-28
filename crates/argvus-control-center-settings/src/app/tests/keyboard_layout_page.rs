//! Keyboard layout page: the draft of enabled layouts and its Apply action.

use super::*;

#[test]
fn keyboard_layout_footer_lists_enter_and_space() {
  let mut app = app(Page::KeyboardLayout);
  app.keyboard_layouts = vec![keyboard::Layout {
    code: "us".into(),
    description: "English (US)".into(),
  }];
  app.navigation.current_mut().menu = argvus_tui::menu::MenuState::default();
  let footer = app.footer();
  assert!(footer.contains("Enter "), "{footer}");
  assert!(footer.contains("Space "), "{footer}");
  assert!(!footer.contains("Enter/Space"), "{footer}");
}

#[test]
fn keyboard_marker_follows_the_layout_in_use_without_dirtying_apply() {
  let mut app = keyboard_layout_app();
  assert!(
    app.apply_live_layout("us"),
    "a switch elsewhere moves the marker"
  );
  let rows = app.rows();
  assert!(!row_of(&rows, Item::Layout(0)).is_marked());
  assert!(row_of(&rows, Item::Layout(1)).is_marked());
  assert!(
    !app.keyboard_layout_changed(),
    "the taskbar switch is not a pending change to Apply"
  );
  assert!(
    !app.apply_live_layout("us"),
    "an unchanged layout reports nothing"
  );
}

#[test]
fn keyboard_default_is_named_in_the_detail_not_the_marker() {
  let mut app = keyboard_layout_app();
  app.apply_live_layout("us");
  let default_label = crate::i18n::tr(app.lang, "control_center.default");
  let rows = app.rows();
  let detail_of = |item: Item| row_of(&rows, item).detail_text().unwrap_or_default();
  assert!(detail_of(Item::Layout(0)).contains(default_label));
  assert!(!detail_of(Item::Layout(1)).contains(default_label));
}

#[test]
fn keyboard_space_and_enter_only_edit_the_draft_until_apply() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Down);
  assert_eq!(app.selected_item(), Some(Item::Layout(1)));
  press(&mut app, KeyCode::Char(' '));
  assert!(
    app.keyboard_draft.is_enabled("us"),
    "Space enables the layout"
  );
  press(&mut app, KeyCode::Enter);
  assert!(
    app.keyboard_draft.is_default("us"),
    "Enter makes it the default"
  );
  assert_eq!(
    app.keyboard_info.hypr_layout, "br",
    "the system state is untouched before Apply"
  );
  assert!(app.keyboard_layout_changed());
}

#[test]
fn keyboard_default_is_marked_and_apply_is_dimmed_until_a_change() {
  let mut app = keyboard_layout_app();
  let rows = app.rows();
  assert!(row_of(&rows, Item::Layout(0)).is_marked());
  assert!(!row_of(&rows, Item::Layout(1)).is_marked());
  let apply = |app: &App| app.action_rows()[0].clone();
  assert_eq!(apply(&app).id(), Some(&Item::ApplyKeyboardLayout));
  assert!(!apply(&app).is_selectable());
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Char(' '));
  assert!(apply(&app).is_selectable());
}

#[test]
fn keyboard_apply_lives_in_the_action_bar_not_in_the_list() {
  let app = keyboard_layout_app();
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::ApplyKeyboardLayout)),
    "the list cursor never walks into Apply"
  );
  assert_eq!(app.action_rows().len(), 1);
}

#[test]
fn keyboard_tab_focuses_the_action_bar_only_when_apply_is_enabled() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Tab);
  assert_eq!(
    app.action_cursor(),
    None,
    "Apply is dimmed: nothing to focus"
  );
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Char(' '));
  press(&mut app, KeyCode::Tab);
  assert_eq!(app.action_cursor(), Some(0), "Tab focuses Apply");
  press(&mut app, KeyCode::Up);
  assert_eq!(
    app.action_cursor(),
    None,
    "Up gives the focus back to the list"
  );
  assert_eq!(
    app.selected_item(),
    Some(Item::Layout(0)),
    "and moves the list"
  );
}

#[test]
fn keyboard_arrows_on_the_action_bar_do_not_move_the_list() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Char(' '));
  press(&mut app, KeyCode::Tab);
  let before = app.selected_item();
  press(&mut app, KeyCode::Right);
  assert_eq!(app.action_cursor(), Some(0), "one button: it stays focused");
  assert_eq!(app.selected_item(), before, "the list did not move");
}

#[test]
fn keyboard_arrows_do_not_toggle_layouts_on_the_list() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Down);
  let before = app.keyboard_draft.clone();
  press(&mut app, KeyCode::Right);
  press(&mut app, KeyCode::Left);
  assert_eq!(app.keyboard_draft, before, "arrows never change a layout");
  assert_eq!(app.action_cursor(), None, "the list keeps the focus");
}

#[test]
fn keyboard_slash_focuses_the_top_search_and_filters_the_list() {
  let mut app = keyboard_layout_app();
  assert!(app.search_on_top());
  press(&mut app, KeyCode::Char('/'));
  assert!(app.searching, "/ focuses the search box");
  press(&mut app, KeyCode::Char('u'));
  press(&mut app, KeyCode::Char('s'));
  assert_eq!(app.search, "us");
  let ids: Vec<Item> = app
    .rows()
    .iter()
    .filter_map(|row| row.id().copied())
    .collect();
  assert_eq!(ids, [Item::Layout(1)], "only the matching layout is listed");
  press(&mut app, KeyCode::Esc);
  assert!(!app.searching);
  assert!(app.search.is_empty(), "Esc clears the query");
}

#[test]
fn keyboard_last_enabled_layout_cannot_be_switched_off_in_the_draft() {
  let mut app = keyboard_layout_app();
  // `br` is the only enabled layout, so Space on it is refused.
  press(&mut app, KeyCode::Char(' '));
  assert!(app.keyboard_draft.is_enabled("br"));
  assert!(!app.keyboard_layout_changed());
}

#[test]
fn keyboard_draft_asks_before_leaving_with_pending_edits() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Char(' '));
  press(&mut app, KeyCode::Esc);
  assert!(matches!(app.confirm, Some(PendingAction::DiscardDraft)));
  assert_eq!(app.page(), Page::KeyboardLayout);
}
