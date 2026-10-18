//! Implements responsive layout calculation in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use ratatui::layout::{Constraint, Layout, Rect};

/// Represents `Areas`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Areas {
  pub header: Rect,
  pub body: Rect,
  pub message: Rect,
  pub footer: Rect,
}

/// Executes the `areas` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn areas(area: Rect) -> Areas {
  let inner = area.inner(ratatui::layout::Margin {
    horizontal: 1,
    vertical: 1,
  });
  let rows = Layout::vertical([
    Constraint::Length(2),
    Constraint::Min(1),
    Constraint::Length(1),
    Constraint::Length(1),
  ])
  .split(inner);
  Areas {
    header: rows[0],
    body: rows[1],
    message: rows[2],
    footer: rows[3],
  }
}

/// Executes the `centered` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
  Rect::new(
    area.x + area.width.saturating_sub(width) / 2,
    area.y + area.height.saturating_sub(height) / 2,
    width.min(area.width),
    height.min(area.height),
  )
}
