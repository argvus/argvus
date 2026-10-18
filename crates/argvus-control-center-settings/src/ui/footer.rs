//! Implements footer rendering in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_tui::chrome::draw_footer;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  draw_footer(frame, area, &app.theme, None, &app.footer());
}
