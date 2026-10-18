//! Implements shared TUI components in crate `argvus tui`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_theme::Theme;
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::text::{display_width, ellipsize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `StatusKind`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum StatusKind {
  Info,
  Success,
  Warning,
  Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `StatusMessage`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct StatusMessage {
  pub kind: StatusKind,
  pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Defines `OperationResult`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum OperationResult {
  Success(String),
  Warning(String),
  Error(String),
}

impl OperationResult {
  /// Executes the `status` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn status(&self) -> StatusMessage {
    let (kind, text) = match self {
      Self::Success(text) => (StatusKind::Success, text),
      Self::Warning(text) => (StatusKind::Warning, text),
      Self::Error(text) => (StatusKind::Error, text),
    };
    StatusMessage {
      kind,
      text: text.clone(),
    }
  }
}

/// Renders `draw_status` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw_status(frame: &mut Frame, area: Rect, theme: &Theme, status: &StatusMessage) {
  let symbol = status_symbol(status.kind);
  let color = status_color(status.kind, theme);
  let prefix = format!("{symbol} ");
  let prefix_len = display_width(&prefix);
  let available = usize::from(area.width).saturating_sub(prefix_len).max(1);
  let wrapped = wrap_text(&status.text, available);
  let max_lines = usize::from(area.height).max(1);
  let (lines, truncated) = if wrapped.len() > max_lines {
    (wrapped[..max_lines].to_vec(), true)
  } else {
    (wrapped, false)
  };
  let mut rendered = Vec::new();
  for (index, mut text) in lines.into_iter().enumerate() {
    if truncated && index + 1 == max_lines {
      text = ellipsize(&text, available);
    }
    if index == 0 {
      rendered.push(Line::from(vec![
        Span::styled(symbol, Style::new().fg(color).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(text, Style::new().fg(theme.foreground)),
      ]));
    } else {
      rendered.push(Line::from(Span::styled(
        text,
        Style::new().fg(theme.foreground),
      )));
    }
  }
  frame.render_widget(
    Paragraph::new(rendered).style(Style::new().bg(theme.surface)),
    area,
  );
}

/// Renders a centered loading overlay while a page-owned background job is pending.
pub fn draw_loading_splash(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  title: &str,
  message: &str,
) {
  if area.width < 12 || area.height < 3 {
    return;
  }
  let popup = crate::chrome::centered(area, area.width.saturating_sub(8).min(60), 5);
  frame.render_widget(Clear, popup);
  frame.render_widget(
    Paragraph::new(vec![
      Line::from(Span::styled(
        "◌",
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
      )),
      Line::from(message),
    ])
    .alignment(Alignment::Center)
    .block(
      Block::bordered()
        .title(title)
        .border_style(Style::new().fg(theme.border_active))
        .style(Style::new().bg(theme.surface).fg(theme.foreground)),
    ),
    popup,
  );
}

/// Executes the `status_line_count` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn status_line_count(message: &StatusMessage, area_width: u16) -> usize {
  let symbol = status_symbol(message.kind);
  let available = usize::from(area_width)
    .saturating_sub(display_width(symbol) + 1)
    .max(1);
  wrap_text(&message.text, available).len()
}

/// Executes the `status_symbol` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn status_symbol(kind: StatusKind) -> &'static str {
  match kind {
    StatusKind::Info => "[i]",
    StatusKind::Success => "[OK]",
    StatusKind::Warning => "[WARN]",
    StatusKind::Error => "[ERROR]",
  }
}

/// Executes the `status_color` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn status_color(kind: StatusKind, theme: &Theme) -> ratatui::style::Color {
  match kind {
    StatusKind::Info => theme.muted,
    StatusKind::Success => theme.success,
    StatusKind::Warning => theme.warning,
    StatusKind::Error => theme.error,
  }
}

/// Executes the `wrap_text` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn wrap_text(value: &str, width: usize) -> Vec<String> {
  let width = width.max(1);
  let mut lines = Vec::new();
  let mut current = String::new();
  for character in value.chars() {
    if display_width(&current) >= width {
      lines.push(std::mem::take(&mut current));
    }
    current.push(character);
  }
  if !current.is_empty() || lines.is_empty() {
    lines.push(current);
  }
  lines
}

/// Represents `ConfirmationDialog`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct ConfirmationDialog<'a> {
  pub title: &'a str,
  pub message: &'a str,
  pub confirm_label: &'a str,
  pub cancel_label: &'a str,
  pub confirm_selected: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Represents `ConfirmationState`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct ConfirmationState {
  pub confirm_selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `ConfirmationOutcome`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum ConfirmationOutcome {
  Pending,
  Confirmed,
  Cancelled,
}

impl ConfirmationState {
  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> ConfirmationOutcome {
    match key {
      KeyCode::Tab | KeyCode::BackTab => {
        self.confirm_selected = !self.confirm_selected;
        ConfirmationOutcome::Pending
      }
      KeyCode::Left => {
        self.confirm_selected = true;
        ConfirmationOutcome::Pending
      }
      KeyCode::Right => {
        self.confirm_selected = false;
        ConfirmationOutcome::Pending
      }
      KeyCode::Enter if self.confirm_selected => ConfirmationOutcome::Confirmed,
      KeyCode::Enter | KeyCode::Esc => ConfirmationOutcome::Cancelled,
      _ => ConfirmationOutcome::Pending,
    }
  }
}

/// Renders `draw_confirmation` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw_confirmation(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  dialog: ConfirmationDialog<'_>,
) {
  let popup_width = area.width.saturating_sub(8).clamp(36, 72);
  let content_width = popup_width.saturating_sub(2).max(1) as usize;
  let message_lines: Vec<Line> = dialog.message.lines().map(Line::from).collect();
  let message_rows = message_lines
    .iter()
    .map(|line| {
      let width = line.width();
      if width == 0 {
        1
      } else {
        width.div_ceil(content_width)
      }
    })
    .sum::<usize>();
  let popup = crate::chrome::centered(
    area,
    popup_width,
    (message_rows.saturating_add(5).max(9) as u16).min(area.height),
  );
  frame.render_widget(Clear, popup);
  let selected = |label: &str, active: bool| {
    if active {
      Span::styled(
        format!("[ {label} ]"),
        Style::new()
          .fg(theme.selected_foreground)
          .bg(theme.selected_background)
          .add_modifier(Modifier::BOLD),
      )
    } else {
      Span::styled(format!("[ {label} ]"), Style::new().fg(theme.accent))
    }
  };
  let mut lines = vec![Line::from("")];
  lines.extend(message_lines);
  lines.push(Line::from(""));
  lines.push(Line::from(vec![
    selected(dialog.confirm_label, dialog.confirm_selected),
    Span::raw("  "),
    selected(dialog.cancel_label, !dialog.confirm_selected),
  ]));
  frame.render_widget(
    Paragraph::new(lines)
      .alignment(Alignment::Center)
      .wrap(Wrap { trim: true })
      .block(
        Block::bordered()
          .title(format!(" {} ", dialog.title))
          .border_style(Style::new().fg(theme.border_active))
          .style(Style::new().bg(theme.background)),
      ),
    popup,
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  /// Executes the `result_maps_to_visible_status_without_colour_only_semantics` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn result_maps_to_visible_status_without_colour_only_semantics() {
    let status = OperationResult::Warning("backend unavailable".into()).status();
    assert_eq!(status.kind, StatusKind::Warning);
    assert!(status.text.contains("backend"));
  }

  #[test]
  /// Executes the `confirmation_navigation_and_safe_default_work` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn confirmation_navigation_and_safe_default_work() {
    let mut state = ConfirmationState::default();
    assert!(!state.confirm_selected);
    assert_eq!(state.handle(KeyCode::Enter), ConfirmationOutcome::Cancelled);
    assert_eq!(state.handle(KeyCode::Tab), ConfirmationOutcome::Pending);
    assert!(state.confirm_selected);
    assert_eq!(state.handle(KeyCode::Enter), ConfirmationOutcome::Confirmed);
    assert_eq!(state.handle(KeyCode::Right), ConfirmationOutcome::Pending);
    assert!(!state.confirm_selected);
    assert_eq!(state.handle(KeyCode::BackTab), ConfirmationOutcome::Pending);
    assert!(state.confirm_selected);
    assert_eq!(state.handle(KeyCode::Esc), ConfirmationOutcome::Cancelled);
  }
  #[test]
  /// Executes the `confirmation_renders_multiline_message_on_separate_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn confirmation_renders_multiline_message_on_separate_rows() {
    use ratatui::{Terminal, backend::TestBackend};
    let theme = argvus_theme::Theme::load();
    let mut terminal = Terminal::new(TestBackend::new(50, 12)).unwrap();
    terminal
      .draw(|frame| {
        draw_confirmation(
          frame,
          Rect::new(0, 0, 50, 12),
          &theme,
          ConfirmationDialog {
            title: "Confirmar transação",
            message: "Remover: chromium\nInstalar: 0\nRemover: 1\nDownload bytes: 0",
            confirm_label: "Aplicar",
            cancel_label: "Cancelar",
            confirm_selected: true,
          },
        );
      })
      .unwrap();
    let buffer = terminal.backend().buffer();
    let rows: Vec<String> = buffer
      .content
      .chunks(50)
      .map(|row| row.iter().map(|cell| cell.symbol()).collect())
      .collect();
    let first = rows
      .iter()
      .position(|row| row.contains("Remover: chromium"))
      .expect("first message line should render");
    let last = rows
      .iter()
      .position(|row| row.contains("Download bytes: 0"))
      .expect("last message line should render");
    assert_ne!(
      first, last,
      "message lines must not be glued together in a single row"
    );
  }

  #[test]
  /// Executes the `long_status_is_ellipsized_on_character_boundaries` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn long_status_is_ellipsized_on_character_boundaries() {
    assert_eq!(ellipsize("erro muito longo", 6), "erro …");
    assert_eq!(ellipsize("áudio", 3), "áu…");
  }

  #[test]
  /// Executes the `status_line_count_grows_when_the_width_shrinks` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn status_line_count_grows_when_the_width_shrinks() {
    let status = StatusMessage {
      kind: StatusKind::Warning,
      text: "abcdefghijklmn".into(),
    };
    assert_eq!(status_line_count(&status, 21), 1);
    assert_eq!(status_line_count(&status, 14), 2);
    assert_eq!(status_line_count(&status, 10), 5);
  }

  #[test]
  /// Executes the `status_wraps_long_text_into_multiple_lines_without_losing_content` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn status_wraps_long_text_into_multiple_lines_without_losing_content() {
    use ratatui::{Terminal, backend::TestBackend};
    let theme = argvus_theme::Theme::load();
    let text = "Aplicado nesta sessão, mas não foi possível gravar a configuração: argvus-control-center: usage text";
    let mut terminal = Terminal::new(TestBackend::new(40, 4)).unwrap();
    terminal
      .draw(|frame| {
        draw_status(
          frame,
          Rect::new(0, 0, 40, 4),
          &theme,
          &StatusMessage {
            kind: StatusKind::Warning,
            text: text.into(),
          },
        );
      })
      .unwrap();
    let buffer = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(!buffer.contains('…'), "must not be ellipsized");
    assert!(buffer.contains("possível"));
    assert!(buffer.contains("argvus-control-center:"));
    assert!(buffer.contains("usage"));
  }
}
