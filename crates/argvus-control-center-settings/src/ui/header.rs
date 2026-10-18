//! Implements header rendering in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_tui::chrome::{Header, draw_header};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  let title = app.breadcrumb();
  draw_header(
    frame,
    area,
    &app.theme,
    Header {
      title: &title,
      version: None,
      version_label: "",
    },
  );
}
