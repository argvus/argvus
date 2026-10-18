//! Cursor of the single vertical menu list.
//!
//! The cursor only ever rests on selectable rows (see
//! [`Row::is_selectable`]): Info rows, separators and disabled rows are
//! skipped by every navigation key. A page without selectable rows has no
//! cursor at all and the navigation keys scroll it instead. Like every list
//! in the Control Center before this component, the cursor stops at both
//! ends instead of wrapping around.

use crossterm::event::KeyCode;

use super::row::{Row, RowKind};

/// What the page should do after a key press. The menu only interprets
/// navigation; business rules stay in the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent<Id> {
  /// The key does not belong to the menu; the page may handle it.
  None,
  /// The cursor (or the scroll position) moved.
  Moved,
  /// Enter/`→` on an Action, Submenu, Choice or Value row.
  Activate(Id),
  /// Enter/Space on a Toggle row.
  Toggle(Id),
  /// `←/→` on a Value row with a step; carries the signed step.
  Adjust(Id, i32),
  /// Enter/`→` on a Destructive row: the page must open the confirmation.
  Confirm(Id),
  /// Esc, or `←` on a row that does not adjust.
  Back,
}

/// Selection and scroll state of one menu list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuState {
  selected: Option<usize>,
  offset: usize,
  /// Visible rows during the last draw; used to bound scrolling.
  viewport: usize,
}

impl MenuState {
  /// Index of the selected row, or `None` when the page is scroll-only.
  pub fn selected_index(&self) -> Option<usize> {
    self.selected
  }

  /// First visible row.
  pub fn offset(&self) -> usize {
    self.offset
  }

  /// Whether the page has no selectable row and only scrolls.
  pub fn is_scroll_only(&self) -> bool {
    self.selected.is_none()
  }

  /// Keeps the cursor on a selectable row after the rows changed.
  ///
  /// The cursor stays where it is when still valid. When its row became
  /// unselectable (disabled, turned into Info, or removed) it moves to the
  /// next selectable row, or the previous one at the end of the list. With
  /// no cursor yet, it lands on the first selectable row.
  pub fn normalize<Id>(&mut self, rows: &[Row<Id>]) {
    self.selected = match self.selected {
      Some(index) if rows.get(index).is_some_and(Row::is_selectable) => Some(index),
      Some(index) => {
        let start = index.min(rows.len());
        next_selectable(rows, start).or_else(|| previous_selectable(rows, start))
      }
      None => next_selectable(rows, 0),
    };
    self.offset = self.offset.min(self.max_offset(rows.len()));
  }

  /// Moves the cursor to the row with `id`, when it is selectable.
  pub fn select<Id: PartialEq>(&mut self, rows: &[Row<Id>], id: &Id) -> bool {
    let found = rows
      .iter()
      .position(|row| row.is_selectable() && row.id() == Some(id));
    if found.is_some() {
      self.selected = found;
    }
    found.is_some()
  }

  /// Identifier of the selected row.
  pub fn selected_id<Id: Copy>(&self, rows: &[Row<Id>]) -> Option<Id> {
    self.selected_row(rows).and_then(|row| row.id().copied())
  }

  /// Kind of the selected row, used to derive the contextual footer.
  pub fn selected_kind<Id>(&self, rows: &[Row<Id>]) -> Option<RowKind> {
    self.selected_row(rows).map(Row::kind)
  }

  fn selected_row<'rows, Id>(&self, rows: &'rows [Row<Id>]) -> Option<&'rows Row<Id>> {
    self
      .selected
      .and_then(|index| rows.get(index))
      .filter(|row| row.is_selectable())
  }

  /// Interprets a key press. `page` is the PgUp/PgDn distance in rows.
  pub fn handle<Id: Copy>(&mut self, key: KeyCode, rows: &[Row<Id>], page: usize) -> MenuEvent<Id> {
    self.normalize(rows);
    let page = page.max(1);
    let Some(current) = self.selected else {
      return self.handle_scroll_only(key, rows.len(), page);
    };

    match key {
      KeyCode::Up | KeyCode::Char('k') => self.move_to(
        current
          .checked_sub(1)
          .and_then(|start| previous_selectable(rows, start)),
      ),
      KeyCode::Down | KeyCode::Char('j') => self.move_to(next_selectable(rows, current + 1)),
      KeyCode::Home => self.move_to(next_selectable(rows, 0)),
      KeyCode::End => self.move_to(
        rows
          .len()
          .checked_sub(1)
          .and_then(|last| previous_selectable(rows, last)),
      ),
      KeyCode::PageUp => {
        // Jump a full page, then settle on the nearest selectable row at or
        // above the target; fall back to one below it (still above the cursor).
        let target = current.saturating_sub(page);
        let landing = previous_selectable(rows, target)
          .or_else(|| next_selectable(rows, target).filter(|&index| index < current));
        self.move_to(landing)
      }
      KeyCode::PageDown => {
        let target = (current + page).min(rows.len().saturating_sub(1));
        let landing = next_selectable(rows, target)
          .or_else(|| previous_selectable(rows, target).filter(|&index| index > current));
        self.move_to(landing)
      }
      _ => activation_event(key, &rows[current]),
    }
  }

  /// Navigation keys scroll a page that has no selectable row.
  fn handle_scroll_only<Id>(&mut self, key: KeyCode, len: usize, page: usize) -> MenuEvent<Id> {
    let max_offset = self.max_offset(len);
    let target = match key {
      KeyCode::Up | KeyCode::Char('k') => self.offset.saturating_sub(1),
      KeyCode::Down | KeyCode::Char('j') => self.offset + 1,
      KeyCode::PageUp => self.offset.saturating_sub(page),
      KeyCode::PageDown => self.offset + page,
      KeyCode::Home => 0,
      KeyCode::End => max_offset,
      KeyCode::Esc | KeyCode::Left => return MenuEvent::Back,
      _ => return MenuEvent::None,
    }
    .min(max_offset);
    if target == self.offset {
      return MenuEvent::None;
    }
    self.offset = target;
    MenuEvent::Moved
  }

  fn move_to<Id>(&mut self, landing: Option<usize>) -> MenuEvent<Id> {
    match landing {
      Some(index) if Some(index) != self.selected => {
        self.selected = Some(index);
        MenuEvent::Moved
      }
      _ => MenuEvent::None,
    }
  }

  /// Largest scroll offset that still fills the viewport. Before the first
  /// draw the viewport is unknown, so scrolling is allowed up to the last row.
  fn max_offset(&self, len: usize) -> usize {
    len.saturating_sub(self.viewport.max(1))
  }

  /// Records the visible height and scrolls so the cursor stays visible.
  ///
  /// Reaching the first (or last) selectable row also reveals the Info rows
  /// before (or after) it, so section headers are not left hidden.
  pub(super) fn fit_viewport<Id>(&mut self, rows: &[Row<Id>], viewport: usize) {
    self.viewport = viewport.max(1);
    self.normalize(rows);
    let Some(selected) = self.selected else {
      return;
    };
    if next_selectable(rows, 0) == Some(selected) {
      self.offset = 0;
    } else if rows
      .len()
      .checked_sub(1)
      .and_then(|last| previous_selectable(rows, last))
      == Some(selected)
    {
      self.offset = self.max_offset(rows.len());
    }
    if selected < self.offset {
      self.offset = selected;
    } else if selected >= self.offset + self.viewport {
      self.offset = selected + 1 - self.viewport;
    }
  }
}

/// Maps an activation key to the event of the selected row's kind.
fn activation_event<Id: Copy>(key: KeyCode, row: &Row<Id>) -> MenuEvent<Id> {
  let Some(&id) = row.id() else {
    return MenuEvent::None;
  };
  match (row.kind(), key) {
    (_, KeyCode::Esc) => MenuEvent::Back,
    (RowKind::Value { step: Some(step) }, KeyCode::Left) => MenuEvent::Adjust(id, -step),
    (RowKind::Value { step: Some(step) }, KeyCode::Right) => MenuEvent::Adjust(id, step),
    (_, KeyCode::Left) => MenuEvent::Back,
    (RowKind::Toggle { .. }, KeyCode::Enter | KeyCode::Right | KeyCode::Char(' ')) => {
      MenuEvent::Toggle(id)
    }
    (RowKind::Destructive, KeyCode::Enter | KeyCode::Right) => MenuEvent::Confirm(id),
    (RowKind::CollapsibleSection { .. }, KeyCode::Enter | KeyCode::Right | KeyCode::Char(' ')) => {
      MenuEvent::Toggle(id)
    }
    (
      RowKind::Action | RowKind::Submenu | RowKind::Choice { .. } | RowKind::Value { .. },
      KeyCode::Enter | KeyCode::Right,
    ) => MenuEvent::Activate(id),
    _ => MenuEvent::None,
  }
}

/// First selectable row at or after `start`.
fn next_selectable<Id>(rows: &[Row<Id>], start: usize) -> Option<usize> {
  rows
    .iter()
    .enumerate()
    .skip(start)
    .find(|(_, row)| row.is_selectable())
    .map(|(index, _)| index)
}

/// Last selectable row at or before `start`.
fn previous_selectable<Id>(rows: &[Row<Id>], start: usize) -> Option<usize> {
  let end = start.saturating_add(1).min(rows.len());
  rows[..end].iter().rposition(Row::is_selectable)
}

#[cfg(test)]
mod tests {
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
}
