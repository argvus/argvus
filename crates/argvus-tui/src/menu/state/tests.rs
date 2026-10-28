use super::*;

/// Header, two options, a disabled option, a separator and an action:
/// indexes 1, 2 and 5 are the only selectable rows.
fn mixed_rows() -> Vec<Row<u8>> {
  vec![
    Row::info("Section", ""),
    Row::action(1, "First"),
    Row::toggle(2, "Second", true),
    Row::action(3, "Disabled").enabled(false),
    Row::separator(),
    Row::action(5, "Last"),
  ]
}

fn state_at(rows: &[Row<u8>], index: usize) -> MenuState {
  let mut state = MenuState {
    selected: Some(index),
    ..MenuState::default()
  };
  state.normalize(rows);
  state
}

#[test]
fn initial_selection_lands_on_first_selectable_row() {
  let rows = mixed_rows();
  let mut state = MenuState::default();
  state.normalize(&rows);
  assert_eq!(state.selected_index(), Some(1));
  assert_eq!(state.selected_id(&rows), Some(1));
}

#[test]
fn down_and_up_skip_info_separator_and_disabled_rows() {
  let rows = mixed_rows();
  let mut state = state_at(&rows, 1);
  assert_eq!(state.handle(KeyCode::Down, &rows, 3), MenuEvent::Moved);
  assert_eq!(state.selected_index(), Some(2));
  assert_eq!(state.handle(KeyCode::Char('j'), &rows, 3), MenuEvent::Moved);
  assert_eq!(
    state.selected_index(),
    Some(5),
    "skips disabled row and separator"
  );
  assert_eq!(state.handle(KeyCode::Char('k'), &rows, 3), MenuEvent::Moved);
  assert_eq!(state.selected_index(), Some(2));
  assert_eq!(state.handle(KeyCode::Up, &rows, 3), MenuEvent::Moved);
  assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn cursor_stops_at_both_ends_without_wrapping() {
  let rows = mixed_rows();
  let mut first = state_at(&rows, 1);
  assert_eq!(first.handle(KeyCode::Up, &rows, 3), MenuEvent::None);
  assert_eq!(
    first.selected_index(),
    Some(1),
    "header above is not selectable"
  );
  let mut last = state_at(&rows, 5);
  assert_eq!(last.handle(KeyCode::Down, &rows, 3), MenuEvent::None);
  assert_eq!(last.selected_index(), Some(5));
}

#[test]
fn home_and_end_go_to_first_and_last_selectable_rows() {
  let mut rows = mixed_rows();
  rows.push(Row::info("Footer", "value"));
  let mut state = state_at(&rows, 2);
  state.handle(KeyCode::End, &rows, 3);
  assert_eq!(state.selected_index(), Some(5));
  state.handle(KeyCode::Home, &rows, 3);
  assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn page_keys_land_on_selectable_rows() {
  let mut rows: Vec<Row<u8>> = Vec::new();
  for id in 0..12u8 {
    // Every third row is an Info header.
    if id % 3 == 0 {
      rows.push(Row::info(format!("Header {id}"), ""));
    } else {
      rows.push(Row::action(id, format!("Item {id}")));
    }
  }
  let mut state = state_at(&rows, 1);
  state.handle(KeyCode::PageDown, &rows, 5);
  assert_eq!(
    state.selected_index(),
    Some(7),
    "target 6 is Info, next is 7"
  );
  state.handle(KeyCode::PageDown, &rows, 5);
  assert_eq!(state.selected_index(), Some(11), "clamped to the last row");
  state.handle(KeyCode::PageUp, &rows, 5);
  assert_eq!(
    state.selected_index(),
    Some(5),
    "target 6 is Info, previous is 5"
  );
  state.handle(KeyCode::PageUp, &rows, 5);
  assert_eq!(
    state.selected_index(),
    Some(1),
    "target 0 is Info, next above cursor is 1"
  );
  assert_eq!(state.handle(KeyCode::PageUp, &rows, 5), MenuEvent::None);
}

#[test]
fn empty_list_has_no_cursor_and_ignores_navigation() {
  let rows: Vec<Row<u8>> = Vec::new();
  let mut state = MenuState::default();
  state.normalize(&rows);
  assert!(state.is_scroll_only());
  for key in [
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Home,
    KeyCode::End,
    KeyCode::PageUp,
    KeyCode::PageDown,
    KeyCode::Enter,
  ] {
    assert_eq!(state.handle(key, &rows, 5), MenuEvent::None, "{key:?}");
  }
  assert_eq!(state.selected_id(&rows), None);
}

#[test]
fn info_only_page_scrolls_instead_of_selecting() {
  let rows: Vec<Row<u8>> = (0..10)
    .map(|line| Row::info(format!("Field {line}"), "value"))
    .collect();
  let mut state = MenuState::default();
  state.fit_viewport(&rows, 4);
  assert!(state.is_scroll_only());
  assert_eq!(state.handle(KeyCode::Down, &rows, 4), MenuEvent::Moved);
  assert_eq!(state.offset(), 1);
  state.handle(KeyCode::End, &rows, 4);
  assert_eq!(state.offset(), 6, "last page fills the viewport");
  assert_eq!(state.handle(KeyCode::PageDown, &rows, 4), MenuEvent::None);
  state.handle(KeyCode::Home, &rows, 4);
  assert_eq!(state.offset(), 0);
  assert_eq!(state.handle(KeyCode::Enter, &rows, 4), MenuEvent::None);
  assert_eq!(state.handle(KeyCode::Esc, &rows, 4), MenuEvent::Back);
}

#[test]
fn normalize_moves_off_a_row_that_became_disabled() {
  let mut rows = mixed_rows();
  let mut state = state_at(&rows, 2);
  rows[2] = Row::toggle(2, "Second", true).enabled(false);
  state.normalize(&rows);
  assert_eq!(state.selected_index(), Some(5), "next selectable row");
  rows[5] = Row::action(5, "Last").enabled(false);
  state.normalize(&rows);
  assert_eq!(
    state.selected_index(),
    Some(1),
    "previous selectable row at the end"
  );
}

#[test]
fn normalize_recovers_when_rows_shrink() {
  let rows = mixed_rows();
  let mut state = state_at(&rows, 5);
  let shorter = &rows[..3];
  state.normalize(shorter);
  assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn select_by_id_ignores_unselectable_rows() {
  let rows = mixed_rows();
  let mut state = state_at(&rows, 1);
  assert!(!state.select(&rows, &3), "disabled row");
  assert_eq!(state.selected_index(), Some(1));
  assert!(state.select(&rows, &5));
  assert_eq!(state.selected_index(), Some(5));
}

#[test]
fn collapsible_sections_take_focus_and_toggle_on_enter_or_space() {
  let rows: Vec<Row<u8>> = vec![
    Row::collapsible_section(0, "Generate key", true),
    Row::action(1, "Generate"),
  ];
  assert!(rows[0].is_selectable());
  let event = |index: usize, key: KeyCode| state_at(&rows, index).handle(key, &rows, 3);
  assert_eq!(event(0, KeyCode::Enter), MenuEvent::Toggle(0));
  assert_eq!(event(0, KeyCode::Char(' ')), MenuEvent::Toggle(0));
}

#[test]
fn activation_events_follow_the_row_kind() {
  let rows: Vec<Row<u8>> = vec![
    Row::action(0, "Refresh"),
    Row::submenu(1, "Themes"),
    Row::toggle(2, "Blur", false),
    Row::choice(3, "Top", true),
    Row::value(4, "Opacity", "80%", Some(5)),
    Row::value(5, "Hostname", "argvus", None),
    Row::destructive(6, "Delete"),
  ];
  let event = |index: usize, key: KeyCode| state_at(&rows, index).handle(key, &rows, 3);

  assert_eq!(event(0, KeyCode::Enter), MenuEvent::Activate(0));
  assert_eq!(event(0, KeyCode::Right), MenuEvent::Activate(0));
  assert_eq!(event(0, KeyCode::Char(' ')), MenuEvent::None);
  assert_eq!(event(1, KeyCode::Enter), MenuEvent::Activate(1));
  assert_eq!(event(2, KeyCode::Enter), MenuEvent::Toggle(2));
  assert_eq!(event(2, KeyCode::Char(' ')), MenuEvent::Toggle(2));
  assert_eq!(event(3, KeyCode::Enter), MenuEvent::Activate(3));
  assert_eq!(event(4, KeyCode::Enter), MenuEvent::Activate(4));
  assert_eq!(event(6, KeyCode::Enter), MenuEvent::Confirm(6));
  assert_eq!(
    event(0, KeyCode::Char('r')),
    MenuEvent::None,
    "page shortcuts pass through"
  );
}

#[test]
fn arrows_adjust_only_values_with_a_step() {
  let rows: Vec<Row<u8>> = vec![
    Row::value(4, "Opacity", "80%", Some(5)),
    Row::value(5, "Hostname", "argvus", None),
    Row::submenu(6, "Themes"),
  ];
  let event = |index: usize, key: KeyCode| state_at(&rows, index).handle(key, &rows, 3);

  assert_eq!(event(0, KeyCode::Right), MenuEvent::Adjust(4, 5));
  assert_eq!(event(0, KeyCode::Left), MenuEvent::Adjust(4, -5));
  assert_eq!(
    event(0, KeyCode::Esc),
    MenuEvent::Back,
    "Esc still goes back on a Value row"
  );
  assert_eq!(event(1, KeyCode::Right), MenuEvent::Activate(5));
  assert_eq!(event(1, KeyCode::Left), MenuEvent::Back);
  assert_eq!(event(2, KeyCode::Left), MenuEvent::Back);
}

#[test]
fn home_and_end_navigate_even_on_value_rows() {
  let rows: Vec<Row<u8>> = vec![
    Row::action(0, "First"),
    Row::value(1, "Opacity", "80%", Some(5)),
    Row::action(2, "Last"),
  ];
  let mut state = state_at(&rows, 1);
  assert_eq!(state.handle(KeyCode::End, &rows, 3), MenuEvent::Moved);
  assert_eq!(state.selected_index(), Some(2));
  let mut state = state_at(&rows, 1);
  assert_eq!(state.handle(KeyCode::Home, &rows, 3), MenuEvent::Moved);
  assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn viewport_keeps_cursor_visible_and_reveals_leading_headers() {
  let mut rows: Vec<Row<u8>> = vec![Row::info("Header", "")];
  rows.extend((1..20u8).map(|id| Row::action(id, format!("Item {id}"))));
  let mut state = state_at(&rows, 1);
  state.fit_viewport(&rows, 5);
  assert_eq!(
    state.offset(),
    0,
    "header above the first item stays visible"
  );
  state.handle(KeyCode::End, &rows, 5);
  state.fit_viewport(&rows, 5);
  assert_eq!(state.offset(), 15);
  state.handle(KeyCode::Up, &rows, 5);
  state.handle(KeyCode::Up, &rows, 5);
  state.fit_viewport(&rows, 5);
  assert_eq!(state.offset(), 15, "cursor still inside the viewport");
  state.handle(KeyCode::Home, &rows, 5);
  state.fit_viewport(&rows, 5);
  assert_eq!(state.offset(), 0);
}
