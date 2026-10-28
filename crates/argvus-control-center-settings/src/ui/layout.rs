//! Responsive layout calculation of the Settings pages.
use ratatui::layout::{Constraint, Layout, Rect};

pub struct Areas {
  pub header: Rect,
  pub body: Rect,
  pub message: Rect,
  pub footer: Rect,
}

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

pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
  Rect::new(
    area.x + area.width.saturating_sub(width) / 2,
    area.y + area.height.saturating_sub(height) / 2,
    width.min(area.width),
    height.min(area.height),
  )
}
