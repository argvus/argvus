//! Shared button rendering.
use argvus_theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
  Primary,
  Secondary,
  Danger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
  pub label: String,
  pub kind: ButtonKind,
}

impl Button {
  pub fn new(label: impl Into<String>, kind: ButtonKind) -> Self {
    Self {
      label: label.into(),
      kind,
    }
  }
}

pub fn height(buttons: &[Button], width: u16) -> u16 {
  if buttons.is_empty() {
    return 0;
  }
  let mut lines = 1usize;
  let mut line_width = 0usize;
  for button in buttons {
    let item = item_width(button);
    if line_width == 0 {
      line_width = item;
    } else if line_width + 1 + item > width as usize {
      lines += 1;
      line_width = item;
    } else {
      line_width += 1 + item;
    }
  }
  lines as u16
}

pub fn draw(frame: &mut Frame, area: Rect, buttons: &[Button], selected: usize, theme: &Theme) {
  if buttons.is_empty() || area.width == 0 || area.height == 0 {
    return;
  }
  let mut lines: Vec<Line<'static>> = Vec::new();
  let mut current: Vec<Span<'static>> = Vec::new();
  let mut line_width = 0usize;
  for (index, button) in buttons.iter().enumerate() {
    let item = item_width(button);
    if line_width > 0 && line_width + 1 + item > area.width as usize {
      lines.push(Line::from(std::mem::take(&mut current)));
      line_width = 0;
    }
    if line_width > 0 {
      current.push(Span::raw(" "));
      line_width += 1;
    }
    current.push(Span::styled(
      format!("[ {} ]", button.label),
      style_for(button, index == selected, theme),
    ));
    line_width += item;
  }
  if !current.is_empty() {
    lines.push(Line::from(current));
  }
  while lines.len() < area.height as usize {
    lines.push(Line::from(""));
  }
  frame.render_widget(
    Paragraph::new(lines).style(Style::new().bg(theme.surface)),
    area,
  );
}

fn item_width(button: &Button) -> usize {
  crate::text::display_width(&button.label) + 4
}

fn style_for(button: &Button, focused: bool, theme: &Theme) -> Style {
  if focused {
    return Style::new()
      .bg(theme.selected_background)
      .fg(theme.selected_foreground)
      .add_modifier(Modifier::BOLD);
  }
  match button.kind {
    ButtonKind::Primary => Style::new().fg(theme.accent),
    ButtonKind::Danger => Style::new().fg(theme.error),
    ButtonKind::Secondary => Style::new().fg(theme.muted),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::load()
  }

  #[test]
  fn height_accounts_for_wrapping() {
    let buttons = vec![
      Button::new("Salvar alterações", ButtonKind::Primary),
      Button::new("Excluir usuário", ButtonKind::Danger),
      Button::new("Cancelar", ButtonKind::Secondary),
    ];
    assert_eq!(height(&buttons, 200), 1);
    assert_eq!(height(&buttons, 30), 3);
    assert_eq!(height(&[], 30), 0);
  }

  #[test]
  fn button_kinds_style_differs() {
    let theme = theme();
    let primary = style_for(&Button::new("a", ButtonKind::Primary), false, &theme);
    let danger = style_for(&Button::new("a", ButtonKind::Danger), false, &theme);
    let focused = style_for(&Button::new("a", ButtonKind::Primary), true, &theme);
    assert_ne!(primary, danger);
    assert!(focused.add_modifier.contains(Modifier::BOLD));
  }

  #[test]
  fn button_bar_uses_the_same_background_as_the_footer() {
    let theme = theme();
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(90, 4)).unwrap();
    terminal
      .draw(|frame| {
        draw(
          frame,
          ratatui::layout::Rect::new(0, 2, 90, 2),
          &[Button::new("Iniciar", ButtonKind::Primary)],
          usize::MAX,
          &theme,
        );
      })
      .unwrap();
    let cell = terminal.backend().buffer().cell((0, 3)).unwrap();
    assert_eq!(cell.bg, theme.surface);
  }

  #[test]
  fn button_labels_are_wrapped_once_by_the_renderer() {
    let theme = theme();
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(32, 2)).unwrap();
    terminal
      .draw(|frame| {
        draw(
          frame,
          ratatui::layout::Rect::new(0, 0, 32, 1),
          &[Button::new("Apply", ButtonKind::Primary)],
          0,
          &theme,
        );
      })
      .unwrap();
    let rendered = (0..9)
      .map(|column| {
        terminal
          .backend()
          .buffer()
          .cell((column, 0))
          .unwrap()
          .symbol()
      })
      .collect::<String>();
    assert_eq!(rendered, "[ Apply ]");
    assert!(!rendered.contains("[ ["));
  }
}
