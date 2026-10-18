//! Implements tab navigation in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::app::{App, Tab};

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  Block::new()
    .bg(app.theme.surface)
    .render(area, frame.buffer_mut());

  let mut spans = Vec::with_capacity(Tab::ALL.len() * 2 - 1);
  for (index, tab) in Tab::ALL.iter().enumerate() {
    let active = *tab == app.active_tab;
    let style = if active {
      Style::new()
        .fg(app.theme.tab_active)
        .bg(app.theme.accent_alpha)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::new().fg(app.theme.tab_inactive)
    };
    spans.push(Span::styled(format!(" {} ", tab.label(app.lang)), style));
    if index + 1 < Tab::ALL.len() {
      spans.push(Span::raw(" "));
    }
  }

  let inner = area.inner(Margin {
    horizontal: 1,
    vertical: 0,
  });
  frame.render_widget(Paragraph::new(Line::from(spans)), inner);
}
