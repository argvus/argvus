//! Implements module declarations for the `ui` subsystem in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod footer;
pub mod header;
pub mod layout;
pub mod scroll;
pub mod tabs;

use ratatui::Frame;
use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::app::{App, Tab};
use crate::pages::Doc;

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(app: &mut App, frame: &mut Frame) {
  let area = frame.area();
  if app.too_small() {
    draw_too_small(app, frame, area);
    return;
  }

  let areas = layout::areas(area);
  app.set_viewport(areas.body.height as usize);

  header::draw(frame, areas.header, app);
  tabs::draw(frame, areas.tabs, app);

  let doc = app.current_doc();
  match app.active_tab {
    Tab::System => draw_system_page(frame, areas.body, app, &doc),
    _ => {
      draw_doc(frame, content_rect(areas.body, app, &doc), app, &doc);
      scroll::draw(frame, areas.body, app, &doc);
    }
  }

  footer::draw(frame, areas.footer, app);
}

/// Renders `draw_system_page` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn draw_system_page(frame: &mut Frame, body: Rect, app: &mut App, doc: &Doc<'_>) {
  let logo_visible = app.logo.as_ref().is_some_and(|logo| logo.is_visible());
  if !logo_visible {
    draw_doc(frame, content_rect(body, app, doc), app, doc);
    scroll::draw(frame, body, app, doc);
    return;
  }

  let wide = body.width >= 96;
  let (logo_area, doc_area) = layout::system_body(body, wide);

  if let Some(logo) = app.logo.as_mut() {
    logo.sync_target(ratatui::layout::Size::new(
      logo_area.width,
      logo_area.height,
    ));
    if logo.is_graphics() {
      logo.render_graphics(frame, logo_area, &app.theme);
    } else {
      let lines = logo.lines(
        logo_area.width as usize,
        logo_area.height as usize,
        &app.theme,
      );
      if !lines.is_empty() {
        frame.render_widget(Paragraph::new(lines), logo_area);
      }
    }
  }

  draw_doc(frame, content_rect(doc_area, app, doc), app, doc);
  scroll::draw(frame, doc_area, app, doc);

  if wide {
    let separator = Rect {
      x: logo_area.right().saturating_sub(1),
      y: body.y,
      width: 1,
      height: body.height,
    };
    Block::default()
      .bg(app.theme.border)
      .render(separator, frame.buffer_mut());
  }
}

/// Renders `draw_doc` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn draw_doc(frame: &mut Frame, area: Rect, app: &App, doc: &Doc<'_>) {
  let paragraph = Paragraph::new(doc.lines.clone())
    .style(Style::new().bg(app.theme.background))
    .scroll((app.scroll as u16, 0));
  frame.render_widget(paragraph, area);
}

/// Executes the `content_rect` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn content_rect(area: Rect, app: &App, doc: &Doc<'_>) -> Rect {
  if app.viewport > 0 && doc.height() > app.viewport {
    Rect {
      x: area.x,
      y: area.y,
      width: area.width.saturating_sub(1),
      height: area.height,
    }
  } else {
    area
  }
}

/// Renders `draw_too_small` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn draw_too_small(app: &App, frame: &mut Frame, area: Rect) {
  Block::new()
    .bg(app.theme.background)
    .render(area, frame.buffer_mut());

  let message = "Terminal window is too small.";
  let width = 32.min(area.width);
  let height = 3.min(area.height);
  let x = area.x + (area.width.saturating_sub(width)) / 2;
  let y = area.y + (area.height.saturating_sub(height)) / 2;
  let cell = Rect::new(x, y, width, height);

  frame.render_widget(Clear, cell);
  frame.render_widget(
    Block::new()
      .borders(Borders::ALL)
      .border_style(Style::new().fg(app.theme.error))
      .bg(app.theme.background),
    cell,
  );
  let inner = cell.inner(Margin {
    horizontal: 1,
    vertical: 0,
  });
  frame.render_widget(
    Paragraph::new(Line::from(message))
      .alignment(Alignment::Center)
      .style(Style::new().fg(app.theme.foreground)),
    inner,
  );
}
