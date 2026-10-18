//! Implements scroll state management in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget};

use crate::app::App;
use crate::pages::Doc;

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(frame: &mut Frame, area: Rect, app: &App, doc: &Doc<'_>) {
  if app.viewport == 0 || doc.height() <= app.viewport {
    return;
  }
  let track = Rect {
    x: area.right().saturating_sub(1),
    y: area.y,
    width: 1,
    height: area.height,
  };
  if track.x < area.x || track.width == 0 {
    return;
  }
  let mut state = ScrollbarState::new(doc.height()).position(app.scroll);
  Scrollbar::new(ScrollbarOrientation::VerticalRight)
    .begin_symbol(None)
    .end_symbol(None)
    .track_symbol(Some("│"))
    .thumb_style(Style::new().fg(app.theme.accent))
    .track_style(Style::new().fg(app.theme.border_active))
    .render(track, frame.buffer_mut(), &mut state);
}
