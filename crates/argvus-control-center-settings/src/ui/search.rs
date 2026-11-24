//! Search interaction of the Settings pages.
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, StatusKind};
use crate::i18n::tr;
use crate::navigation::Page;

/// Draws the page filter directly above its list.
pub fn draw_box(frame: &mut Frame, area: Rect, app: &App) {
  argvus_tui::filter_search::search_box(
    frame,
    area,
    &app.theme,
    tr(app.lang, "control_center.filter"),
    &app.search,
    app.searching,
  );
}

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
        "−/+  {}: {}  (8–32)",
        tr(app.lang, "control_center.size"),
        app.pending_size
      ),
      Style::new().fg(app.theme.muted),
    ))
  } else {
    Line::default()
  };
  frame.render_widget(
    Paragraph::new(line).style(Style::new().bg(app.theme.background)),
    area,
  );
}
