//! Text field opened in a popup over a page: one line (an address, a name)
//! or several (a configuration file, a public key).
//!
//! Keys: the arrows, Home/End, Backspace and Delete edit; Enter saves a
//! single line or breaks a line; `Ctrl+S` saves; Esc cancels. Pasted text
//! (bracketed paste) is inserted at the cursor.

use argvus_theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
  Frame,
  layout::Rect,
  style::Style,
  text::Line,
  widgets::{Block, Clear, Paragraph},
};

/// Longest text accepted, in characters, as the largest privileged argument.
const MAX_CHARS: usize = 256 * 1024;

/// Result of a key press while the editor is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorOutcome {
  /// Still open; the text or the cursor may have changed.
  Pending,
  Saved(String),
  Cancelled,
}

/// The text being edited and its cursor, counted in characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEditor {
  title: String,
  value: Vec<char>,
  cursor: usize,
  multiline: bool,
}

impl TextEditor {
  /// An editor for one line, with the cursor after `value`.
  pub fn single_line(title: impl Into<String>, value: &str) -> Self {
    Self::new(title.into(), value, false)
  }

  /// An editor for several lines, with the cursor after `value`.
  pub fn multi_line(title: impl Into<String>, value: &str) -> Self {
    Self::new(title.into(), value, true)
  }

  fn new(title: String, value: &str, multiline: bool) -> Self {
    let value: Vec<char> = value.chars().collect();
    Self {
      title,
      cursor: value.len(),
      value,
      multiline,
    }
  }

  pub fn is_multiline(&self) -> bool {
    self.multiline
  }

  /// The text typed so far.
  pub fn text(&self) -> String {
    self.value.iter().collect()
  }

  /// Handles one key press.
  pub fn handle(&mut self, key: KeyEvent) -> EditorOutcome {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
      KeyCode::Esc => return EditorOutcome::Cancelled,
      KeyCode::Char('s' | 'S') if control => return EditorOutcome::Saved(self.text()),
      KeyCode::Enter if !self.multiline => return EditorOutcome::Saved(self.text()),
      KeyCode::Enter => self.insert('\n'),
      KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
      KeyCode::Right => self.cursor = (self.cursor + 1).min(self.value.len()),
      KeyCode::Home => self.cursor = self.line_start(self.cursor),
      KeyCode::End => self.cursor = self.line_end(self.cursor),
      KeyCode::Up => self.move_vertically(false),
      KeyCode::Down => self.move_vertically(true),
      KeyCode::Backspace if self.cursor > 0 => {
        self.cursor -= 1;
        self.value.remove(self.cursor);
      }
      KeyCode::Delete if self.cursor < self.value.len() => {
        self.value.remove(self.cursor);
      }
      KeyCode::Tab if self.multiline => self.insert('\t'),
      KeyCode::Char(character)
        if !key
          .modifiers
          .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
      {
        self.insert(character)
      }
      _ => {}
    }
    EditorOutcome::Pending
  }

  /// Inserts pasted `text` at the cursor. Carriage returns are dropped; a
  /// single-line editor also drops line breaks, so a copied address with a
  /// trailing newline still fits.
  pub fn paste(&mut self, text: &str) {
    for character in text.chars() {
      match character {
        '\r' => {}
        '\n' | '\t' if !self.multiline => {}
        _ => self.insert(character),
      }
    }
  }

  fn insert(&mut self, character: char) {
    let allowed = !character.is_control() || (self.multiline && matches!(character, '\n' | '\t'));
    if allowed && self.value.len() < MAX_CHARS {
      self.value.insert(self.cursor, character);
      self.cursor += 1;
    }
  }

  fn line_start(&self, position: usize) -> usize {
    self.value[..position]
      .iter()
      .rposition(|character| *character == '\n')
      .map_or(0, |index| index + 1)
  }

  fn line_end(&self, position: usize) -> usize {
    position
      + self.value[position..]
        .iter()
        .position(|character| *character == '\n')
        .unwrap_or(self.value.len() - position)
  }

  /// Moves to the line above or below, keeping the column when it fits.
  fn move_vertically(&mut self, down: bool) {
    let start = self.line_start(self.cursor);
    let column = self.cursor - start;
    if down {
      let end = self.line_end(self.cursor);
      if end < self.value.len() {
        let next = end + 1;
        self.cursor = next + column.min(self.line_end(next) - next);
      }
    } else if start > 0 {
      let previous = self.line_start(start - 1);
      self.cursor = previous + column.min(start - 1 - previous);
    }
  }

  /// Draws the editor in a popup over `area`, with `hint` (the translated
  /// keys) on the bottom border, and places the terminal cursor.
  pub fn draw(&self, frame: &mut Frame, area: Rect, theme: &Theme, hint: &str) {
    let height = if self.multiline {
      area.height.saturating_sub(4)
    } else {
      3
    };
    let popup = crate::chrome::centered(area, area.width.saturating_sub(6), height);
    frame.render_widget(Clear, popup);
    let block = Block::bordered()
      .title(format!(" {} ", self.title))
      .title_bottom(Line::from(hint.to_owned()))
      .border_style(Style::new().fg(theme.border_active))
      .style(Style::new().fg(theme.foreground).bg(theme.background));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // Scroll so the cursor stays visible in both directions.
    let before: String = self.value[..self.cursor].iter().collect();
    let line = before.matches('\n').count();
    let column = Line::from(before.rsplit('\n').next().unwrap_or_default()).width();
    let vertical = line.saturating_sub(usize::from(inner.height.saturating_sub(1)));
    let horizontal = column.saturating_sub(usize::from(inner.width.saturating_sub(1)));
    let scroll = (
      u16::try_from(vertical).unwrap_or(u16::MAX),
      u16::try_from(horizontal).unwrap_or(u16::MAX),
    );
    frame.render_widget(Paragraph::new(self.text()).scroll(scroll), inner);
    let cursor_x = u16::try_from(column - horizontal).unwrap_or(u16::MAX);
    let cursor_y = u16::try_from(line - vertical).unwrap_or(u16::MAX);
    frame.set_cursor_position((
      inner.x.saturating_add(cursor_x),
      inner.y.saturating_add(cursor_y),
    ));
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ratatui::{Terminal, backend::TestBackend};

  fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
  }

  fn type_text(editor: &mut TextEditor, text: &str) {
    for character in text.chars() {
      editor.handle(key(KeyCode::Char(character)));
    }
  }

  #[test]
  fn single_line_saves_with_enter_and_ignores_line_breaks() {
    let mut editor = TextEditor::single_line("URL", "https://");
    type_text(&mut editor, "a.org");
    editor.paste("/x.conf\r\n");
    assert_eq!(
      editor.handle(key(KeyCode::Enter)),
      EditorOutcome::Saved("https://a.org/x.conf".into())
    );
    assert_eq!(editor.handle(key(KeyCode::Esc)), EditorOutcome::Cancelled);
  }

  #[test]
  fn multi_line_breaks_lines_with_enter_and_saves_with_ctrl_s() {
    let mut editor = TextEditor::multi_line("Configuration", "");
    type_text(&mut editor, "[x]");
    assert_eq!(editor.handle(key(KeyCode::Enter)), EditorOutcome::Pending);
    editor.paste("Server = https://a\r\n");
    assert_eq!(
      editor.handle(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
      EditorOutcome::Saved("[x]\nServer = https://a\n".into())
    );
  }

  #[test]
  fn arrows_move_between_lines_and_keep_the_column() {
    let mut editor = TextEditor::multi_line("Key", "abcd\nef\nghij");
    editor.handle(key(KeyCode::Up));
    assert_eq!(editor.cursor, 7, "end of the shorter line above");
    editor.handle(key(KeyCode::Up));
    assert_eq!(editor.cursor, 2);
    editor.handle(key(KeyCode::Home));
    assert_eq!(editor.cursor, 0);
    editor.handle(key(KeyCode::End));
    editor.handle(key(KeyCode::Down));
    editor.handle(key(KeyCode::Down));
    assert_eq!(editor.cursor, 10, "column 2, kept from the short line");
    editor.handle(key(KeyCode::Backspace));
    editor.handle(key(KeyCode::Left));
    editor.handle(key(KeyCode::Delete));
    assert_eq!(editor.text(), "abcd\nef\nij");
  }

  #[test]
  fn draws_the_title_hint_and_text() {
    let editor = TextEditor::multi_line("Configuration", "[argvus]\nServer = https://a");
    let mut terminal = Terminal::new(TestBackend::new(60, 16)).unwrap();
    terminal
      .draw(|frame| editor.draw(frame, frame.area(), &Theme::load(), "Ctrl+S save"))
      .unwrap();
    let screen: String = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect();
    for expected in [
      "Configuration",
      "Ctrl+S save",
      "[argvus]",
      "Server = https://a",
    ] {
      assert!(screen.contains(expected), "{expected}");
    }
  }
}
