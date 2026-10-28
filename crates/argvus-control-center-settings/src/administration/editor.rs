//! Inline text editor used for names, passwords, firewall values and rules.

use super::*;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum EditTarget {
  Config(String),
  User(String),
  Password(usize),
  Rules,
  Group,
  Avatar,
  DateTime,
  /// Size used when a font is applied on the font selector (8–32).
  FontSize,
  /// Draft name of a saved rule, keyed by its loaded name.
  WindowRuleName(String),
  WindowRuleClasses(String),
  /// The name typed for the next rule.
  NewWindowRuleName,
  /// A new project path, or a new root when true.
  ProjectPath(bool),
  /// The name and content typed for the next snippet.
  NewSnippetName,
  NewSnippetContent,
  /// Replacement content of a saved snippet, keyed by its name.
  SnippetContent(String),
  /// The name typed for the next SSH key pair.
  NewSshKeyName,
  /// The name, e-mail and comment typed for the next GPG key.
  GpgFullName,
  GpgEmail,
  GpgComment,
  /// The expiry date typed for the next GPG key.
  GpgExpiry,
  /// The passphrase and its confirmation for the next GPG key. Both are masked.
  GpgPassphrase,
  GpgPassphraseConfirm,
  /// Export path of one key, keyed by its fingerprint.
  GpgExportPath(String),
  /// File to import public keys from.
  GpgImportPath,
  /// File to write the private key of one fingerprint to.
  GpgExportPrivatePath(String),
  /// File to import a private key from.
  GpgImportPrivatePath,
  /// The expiry date typed on the edit page of a key.
  GpgEditExpiry,
}

pub struct Editor {
  pub(super) title: String,
  pub(super) value: Vec<char>,
  pub(super) cursor: usize,
  pub(super) target: EditTarget,
  pub(super) multiline: bool,
}

impl Editor {
  pub(crate) fn new(title: String, value: String, target: EditTarget, multiline: bool) -> Self {
    let value: Vec<char> = value.chars().collect();
    Self {
      title,
      cursor: value.len(),
      value,
      target,
      multiline,
    }
  }

  /// The text currently typed in the editor.
  #[cfg(test)]
  pub(crate) fn text(&self) -> String {
    self.value.iter().collect()
  }

  pub(super) fn input(&mut self, key: KeyEvent) {
    match key.code {
      KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
      KeyCode::Right => self.cursor = (self.cursor + 1).min(self.value.len()),
      KeyCode::Home => {
        self.cursor = self.value[..self.cursor]
          .iter()
          .rposition(|c| *c == '\n')
          .map(|i| i + 1)
          .unwrap_or(0)
      }
      KeyCode::End => {
        self.cursor += self.value[self.cursor..]
          .iter()
          .position(|c| *c == '\n')
          .unwrap_or(self.value.len() - self.cursor)
      }
      KeyCode::Up | KeyCode::Down => {
        let start = self.value[..self.cursor]
          .iter()
          .rposition(|c| *c == '\n')
          .map(|i| i + 1)
          .unwrap_or(0);
        let column = self.cursor - start;
        if key.code == KeyCode::Up && start > 0 {
          let previous = self.value[..start - 1]
            .iter()
            .rposition(|c| *c == '\n')
            .map(|i| i + 1)
            .unwrap_or(0);
          self.cursor = (previous + column).min(start - 1);
        } else if key.code == KeyCode::Down
          && let Some(end) = self.value[self.cursor..].iter().position(|c| *c == '\n')
        {
          let next = self.cursor + end + 1;
          let length = self.value[next..]
            .iter()
            .position(|c| *c == '\n')
            .unwrap_or(self.value.len() - next);
          self.cursor = next + column.min(length);
        }
      }
      KeyCode::Backspace if self.cursor > 0 => {
        self.cursor -= 1;
        self.value.remove(self.cursor);
      }
      KeyCode::Delete if self.cursor < self.value.len() => {
        self.value.remove(self.cursor);
      }
      KeyCode::Enter if self.multiline => {
        self.value.insert(self.cursor, '\n');
        self.cursor += 1;
      }
      KeyCode::Char(character)
        if !key
          .modifiers
          .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
          && !character.is_control()
          && self.value.len() < 262144 =>
      {
        self.value.insert(self.cursor, character);
        self.cursor += 1;
      }
      _ => {}
    }
  }

  pub fn draw(&self, frame: &mut Frame, area: Rect, app: &App) {
    let area = crate::ui::layout::centered(
      area,
      area.width.saturating_sub(6),
      if self.multiline {
        area.height.saturating_sub(4)
      } else {
        5
      },
    );
    frame.render_widget(Clear, area);
    let block = Block::bordered()
      .title(self.title.as_str())
      .title_bottom(tr(app.lang, "control_center.enter_ctrl_s_save_esc_cancel"))
      .style(
        Style::new()
          .fg(app.theme.foreground)
          .bg(app.theme.background),
      );
    let inner = block.inner(area);
    let secret = matches!(
      self.target,
      EditTarget::Password(_) | EditTarget::GpgPassphrase | EditTarget::GpgPassphraseConfirm
    );
    let text: String = self
      .value
      .iter()
      .map(|c| if secret { '*' } else { *c })
      .collect();
    let before: String = self.value[..self.cursor].iter().collect();
    let line = before.chars().filter(|c| *c == '\n').count();
    let current_line = before.rsplit('\n').next().unwrap_or("");
    let column = if secret {
      current_line.chars().count()
    } else {
      ratatui::text::Line::from(current_line).width()
    };
    let vertical = line.saturating_sub(inner.height.saturating_sub(1) as usize);
    let horizontal = column.saturating_sub(inner.width.saturating_sub(1) as usize);
    frame.render_widget(block, area);
    frame.render_widget(
      Paragraph::new(text).scroll((vertical as u16, horizontal as u16)),
      inner,
    );
    frame.set_cursor_position((
      inner.x + (column - horizontal) as u16,
      inner.y + (line - vertical) as u16,
    ));
  }
}
