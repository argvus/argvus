//! Header rendering of the Settings pages.
use argvus_tui::chrome::draw_control_center_header;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  let title = app.breadcrumb();
  draw_control_center_header(frame, area, &app.theme, &title, None);
}
