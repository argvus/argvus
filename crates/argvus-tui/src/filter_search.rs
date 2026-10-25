//! Filter line, search box and titled block of the filtered lists.
//!
//! A filtered page reads top to bottom as: a `Filter:` line with the value
//! on the right, a bordered `Search:` box that shows the typed text, and a
//! bordered block titled with the list it holds. The pages only supply the
//! text and the state; the layout and the focus colors live here.

use argvus_theme::Theme;
use ratatui::{
  Frame,
  layout::{Constraint, Layout, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Paragraph},
};

use crate::text::display_width;

/// Draws `label:` on the left and `value` right-aligned on the same row.
pub fn filter_line(frame: &mut Frame, area: Rect, theme: &Theme, label: &str, value: &str) {
  let value_width = u16::try_from(display_width(value)).unwrap_or(u16::MAX);
  let [label_area, value_area] =
    Layout::horizontal([Constraint::Min(1), Constraint::Length(value_width)]).areas(area);
  frame.render_widget(
    Paragraph::new(Line::from(Span::styled(
      format!("{label}:"),
      Style::new()
        .fg(theme.foreground)
        .add_modifier(Modifier::BOLD),
    )))
    .style(Style::new().bg(theme.background)),
    label_area,
  );
  frame.render_widget(
    Paragraph::new(Line::from(Span::styled(
      value.to_string(),
      Style::new().fg(theme.accent),
    )))
    .style(Style::new().bg(theme.background)),
    value_area,
  );
}

/// Draws a bordered `label: query` box. While `focused`, the border is
/// highlighted and a cursor follows the text, because typed letters go to it.
pub fn search_box(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  label: &str,
  query: &str,
  focused: bool,
) {
  let (border, cursor) = if focused {
    (theme.border_active, "▌")
  } else {
    (theme.border, "")
  };
  let line = Line::from(vec![
    Span::styled(
      format!("{label}: "),
      Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    ),
    Span::styled(
      format!("{query}{cursor}"),
      Style::new().fg(theme.foreground),
    ),
  ]);
  frame.render_widget(
    Paragraph::new(line)
      .style(Style::new().bg(theme.background))
      .block(Block::bordered().border_style(Style::new().fg(border))),
    area,
  );
}

/// Draws a bordered block titled `title` and returns its inner area, where
/// the list goes.
pub fn titled_block(frame: &mut Frame, area: Rect, theme: &Theme, title: &str) -> Rect {
  let block = Block::bordered()
    .border_style(Style::new().fg(theme.border_active))
    .title(format!(" {title} "));
  let inner = block.inner(area);
  frame.render_widget(block, area);
  inner
}
