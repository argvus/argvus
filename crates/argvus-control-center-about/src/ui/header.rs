//! Implements header rendering in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_tui::chrome::{Header, draw_header};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;
use crate::i18n::tr;

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  let title = format!(
    "{} > {}",
    tr(app.lang, "control_center.argvus_control_center"),
    app.active_tab.label(app.lang),
  );
  draw_header(
    frame,
    area,
    &app.theme,
    Header {
      title: &title,
      version: Some(&app.argvus_version),
      version_label: tr(app.lang, "control_center.version"),
    },
  );
}
