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
mod tests;
