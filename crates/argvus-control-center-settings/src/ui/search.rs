//! Implements search interaction in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::{App, StatusKind};
use crate::i18n::tr;
use crate::navigation::Page;

/// The search box of a page with a top search (`App::search_on_top`): a
/// bordered `Search:` line that is highlighted while the search is typed in.
pub fn draw_box(frame: &mut Frame, area: Rect, app: &App) {
  let (border, cursor) = if app.searching {
    (app.theme.border_active, "▌")
  } else {
    (app.theme.border, "")
  };
  let line = Line::from(vec![
    Span::styled(
      // The label key of the home search: the bare `control_center.search`
      // value carries a `/` shortcut hint that does not belong in a label.
      format!("{}: ", tr(app.lang, "control_center.search_2c43ee")),
      Style::new()
        .fg(app.theme.accent)
        .add_modifier(Modifier::BOLD),
    ),
    Span::styled(
      format!("{}{cursor}", app.search),
      Style::new().fg(app.theme.foreground),
    ),
  ]);
  frame.render_widget(
    Paragraph::new(line)
      .style(Style::new().bg(app.theme.background))
      .block(Block::bordered().border_style(Style::new().fg(border))),
    area,
  );
}

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  // On a page with a top search the box shows the query, so the bottom line
  // only carries status messages.
  let line = if !app.search_on_top() && (app.searching || !app.search.is_empty()) {
    Line::from(vec![
      Span::styled(
        format!("{}: ", tr(app.lang, "control_center.search")),
        Style::new().fg(app.theme.accent),
      ),
      Span::styled(
        format!("{}_", app.search),
        Style::new().fg(app.theme.foreground),
      ),
    ])
  } else if let Some(status) = &app.status {
    let color = match status.kind {
      StatusKind::Success => app.theme.success,
      StatusKind::Error => app.theme.error,
    };
    Line::from(Span::styled(&status.text, Style::new().fg(color)))
  } else if let Page::FontSelector(_) = app.page() {
    Line::from(Span::styled(
      format!(
        "{}: {}",
        tr(app.lang, "control_center.size"),
        app.pending_size
      ),
      Style::new().fg(app.theme.muted),
    ))
  } else {
    Line::default()
  };
  frame.render_widget(
    Paragraph::new(line).style(Style::new().bg(app.theme.surface)),
    area,
  );
}
