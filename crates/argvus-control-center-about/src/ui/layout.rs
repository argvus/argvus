//! Implements responsive layout calculation in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use ratatui::layout::{Constraint, Layout, Rect};

/// Represents `Areas`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Areas {
  pub header: Rect,
  pub tabs: Rect,
  pub body: Rect,
  pub footer: Rect,
}

/// Executes the `areas` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn areas(area: Rect) -> Areas {
  let rows = Layout::vertical([
    Constraint::Length(1),
    Constraint::Length(1),
    Constraint::Min(0),
    Constraint::Length(2),
  ])
  .split(area);
  Areas {
    header: rows[0],
    tabs: rows[1],
    body: rows[2],
    footer: rows[3],
  }
}

/// Executes the `system_body` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn system_body(body: Rect, wide: bool) -> (Rect, Rect) {
  if wide {
    let columns =
      Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)]).split(body);
    (columns[0], columns[1])
  } else {
    let rows =
      Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)]).split(body);
    (rows[0], rows[1])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `areas_cover_full_height` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn areas_cover_full_height() {
    let areas = areas(Rect::new(0, 0, 120, 30));
    assert_eq!(areas.header.height, 1);
    assert_eq!(areas.tabs.height, 1);
    assert_eq!(areas.footer.height, 2);
    assert_eq!(areas.body.y, 2);
    assert_eq!(areas.footer.y, 28);
  }

  #[test]
  /// Executes the `system_body_splits_wide_and_narrow` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn system_body_splits_wide_and_narrow() {
    let body = Rect::new(0, 0, 120, 20);
    let (logo, doc) = system_body(body, true);
    assert_eq!(logo.x, 0);
    assert!(doc.x > logo.x);

    let narrow = Rect::new(0, 0, 80, 20);
    let (logo, doc) = system_body(narrow, false);
    assert_eq!(logo.x, 0);
    assert_eq!(doc.x, 0);
    assert!(doc.y > logo.y);
  }
}
