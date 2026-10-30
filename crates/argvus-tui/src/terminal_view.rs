//! A small VT screen model for embedding a terminal child inside a Ratatui view.

use argvus_theme::Theme;
use ratatui::{
  Frame,
  layout::Rect,
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::Paragraph,
};
use std::num::NonZeroU16;

/// A terminal screen that consumes ANSI output from a child process.
pub struct TerminalView {
  parser: vt100::Parser,
  rows: u16,
  columns: u16,
}

impl TerminalView {
  /// Creates an empty screen, clamping each dimension to a valid PTY size.
  pub fn new(rows: u16, columns: u16) -> Self {
    Self {
      parser: vt100::Parser::new(nonzero(rows), nonzero(columns), 0),
      rows: rows.max(1),
      columns: columns.max(1),
    }
  }

  /// Feeds terminal output into the screen model.
  pub fn process(&mut self, output: &[u8]) {
    self.parser.process(output);
  }

  /// Resizes the emulated screen after the containing TUI changes size.
  pub fn resize(&mut self, rows: u16, columns: u16) {
    self.rows = rows.max(1);
    self.columns = columns.max(1);
    self
      .parser
      .set_size(nonzero(self.rows), nonzero(self.columns));
  }

  /// Whether the current terminal screen contains the given visible text.
  pub fn contains_text(&self, text: &str) -> bool {
    let screen = self.parser.screen();
    (0..self.rows).any(|row| {
      let mut line = String::new();
      for column in 0..self.columns {
        let Some(cell) = screen.cell(row, column) else {
          continue;
        };
        if cell.has_contents() && !cell.is_wide_continuation() {
          line.push_str(cell.contents());
        } else {
          line.push(' ');
        }
      }
      line.contains(text)
    })
  }

  /// Whether the child currently requests terminal mouse events.
  pub fn mouse_enabled(&self) -> bool {
    self.parser.screen().mouse_protocol_mode() != vt100::MouseProtocolMode::None
  }

  /// Draws the visible terminal cells, preserving ANSI colors and attributes.
  pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
    let screen = self.parser.screen();
    let lines = (0..area.height)
      .map(|row| {
        let mut spans: Vec<Span<'static>> = Vec::new();
        for column in 0..area.width {
          let Some(cell) = screen.cell(row, column) else {
            continue;
          };
          let contents = if cell.has_contents() {
            cell.contents()
          } else {
            " "
          };
          if cell.is_wide_continuation() {
            continue;
          }
          let mut foreground = map_color(cell.fgcolor(), theme.foreground);
          let mut background = map_color(cell.bgcolor(), theme.background);
          if cell.inverse() {
            std::mem::swap(&mut foreground, &mut background);
          }
          let mut style = Style::new().fg(foreground).bg(background);
          if cell.bold() {
            style = style.add_modifier(Modifier::BOLD);
          }
          if cell.dim() {
            style = style.add_modifier(Modifier::DIM);
          }
          if cell.italic() {
            style = style.add_modifier(Modifier::ITALIC);
          }
          if cell.underline() {
            style = style.add_modifier(Modifier::UNDERLINED);
          }
          if let Some(previous) = spans.last_mut()
            && previous.style == style
          {
            previous.content.to_mut().push_str(contents);
          } else {
            spans.push(Span::styled(contents.to_owned(), style));
          }
        }
        Line::from(spans)
      })
      .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines), area);
  }
}

fn nonzero(value: u16) -> NonZeroU16 {
  NonZeroU16::new(value.max(1)).unwrap_or(NonZeroU16::MIN)
}

fn map_color(color: vt100::Color, fallback: Color) -> Color {
  match color {
    vt100::Color::Default => fallback,
    vt100::Color::Idx(index) => Color::Indexed(index),
    vt100::Color::Rgb(red, green, blue) => Color::Rgb(red, green, blue),
  }
}

#[cfg(test)]
mod tests {
  use super::TerminalView;
  use ratatui::{Terminal, backend::TestBackend};

  #[test]
  fn parses_ansi_output_and_keeps_content_after_resize() {
    let theme = argvus_theme::Theme::load();
    let mut view = TerminalView::new(4, 20);
    view.process(b"\x1b[31mCPU 42%\x1b[0m");
    view.resize(6, 24);
    let mut terminal = Terminal::new(TestBackend::new(24, 6)).unwrap();
    terminal
      .draw(|frame| view.render(frame, frame.area(), &theme))
      .unwrap();
    let content = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(content.contains("CPU 42%"), "{content}");
    assert!(view.contains_text("CPU 42%"));
    assert!(!view.contains_text("gpu0"));
  }

  #[test]
  fn detects_a_gpu_box_by_its_btop_slot_name() {
    let mut view = TerminalView::new(2, 20);
    view.process("╭─┐⁵gpu0┌────╮".as_bytes());
    assert!(view.contains_text("gpu0"));
  }
}
