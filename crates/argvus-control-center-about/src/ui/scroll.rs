//! Implements scroll state management in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_tui::scroll::thumb_position;
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
  let mut state = ScrollbarState::new(doc.height())
    .viewport_content_length(app.viewport)
    .position(thumb_position(app.scroll, doc.height(), app.viewport));
  Scrollbar::new(ScrollbarOrientation::VerticalRight)
    .begin_symbol(None)
    .end_symbol(None)
    .track_symbol(Some("│"))
    .thumb_style(Style::new().fg(app.theme.accent))
    .track_style(Style::new().fg(app.theme.border_active))
    .render(track, frame.buffer_mut(), &mut state);
}

#[cfg(test)]
mod tests {
  use ratatui::Terminal;
  use ratatui::backend::TestBackend;

  use crate::app::{App, Tab};
  use crate::ui;

  /// Symbol ratatui draws for the thumb of a vertical scrollbar.
  const THUMB: &str = "█";

  /// Draws the page and returns the scrollbar column, top to bottom, over the body rows.
  fn scrollbar_column(app: &mut App, terminal: &mut Terminal<TestBackend>) -> Vec<String> {
    terminal.draw(|frame| ui::draw(app, frame)).expect("draw");
    let buffer = terminal.backend().buffer();
    let area = buffer.area;
    let body = ui::layout::areas(area).body;
    (body.top()..body.bottom())
      .map(|y| buffer[(area.right() - 1, y)].symbol().to_string())
      .collect()
  }

  #[test]
  fn thumb_reaches_the_bottom_at_the_end_of_every_long_tab() {
    for tab in [Tab::System, Tab::About, Tab::Copyright] {
      let mut app = App::test();
      app.goto_tab(tab);
      let mut terminal = Terminal::new(TestBackend::new(80, 20)).expect("terminal");

      let top = scrollbar_column(&mut app, &mut terminal);
      assert_eq!(top.first().map(String::as_str), Some(THUMB), "{tab:?} top");
      assert_ne!(top.last().map(String::as_str), Some(THUMB), "{tab:?} top");

      app.end();
      let bottom = scrollbar_column(&mut app, &mut terminal);
      assert_eq!(
        bottom.last().map(String::as_str),
        Some(THUMB),
        "{tab:?} end"
      );
      assert_ne!(
        bottom.first().map(String::as_str),
        Some(THUMB),
        "{tab:?} end"
      );
    }
  }
}
