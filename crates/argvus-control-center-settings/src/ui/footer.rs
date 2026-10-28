//! Footer rendering of the Settings pages.
use argvus_tui::chrome::draw_footer;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  draw_footer(frame, area, &app.theme, None, &app.footer());
}
