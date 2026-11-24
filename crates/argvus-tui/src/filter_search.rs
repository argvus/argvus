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

thread_local! {
  static CURSOR_VISIBLE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Sets the cursor blink phase for local search fields.
pub fn set_cursor_visible(visible: bool) {
  CURSOR_VISIBLE.with(|cursor| cursor.set(visible));
}

/// Returns the cursor glyph for the current blink phase.
pub fn cursor() -> &'static str {
  CURSOR_VISIBLE.with(|visible| if visible.get() { "▌" } else { " " })
}

/// Label shared by local list filters, including the editable package query.
pub fn filter_label(label: &str) -> String {
  format!("{} {label} [Ctrl+F]: ", crate::icons::FILTER)
}

/// Draws a one-line local filter with a focus indicator and keyboard shortcut.
pub fn search_box(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  label: &str,
  query: &str,
  focused: bool,
) {
  let border = if focused {
    theme.border_active
  } else {
    theme.border
  };
  let line = Line::from(vec![
    Span::styled(
      filter_label(label),
      Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    ),
    Span::styled(
      format!("{query}{}", if focused { cursor() } else { "" }),
      Style::new().fg(theme.foreground),
    ),
  ]);
  frame.render_widget(
    Paragraph::new(line)
      .style(Style::new().bg(theme.background))
      .block(
        Block::default()
          .borders(ratatui::widgets::Borders::LEFT)
          .border_style(Style::new().fg(border)),
      ),
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
