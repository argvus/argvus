//! Implements shared action buttons bar rendering in crate `argvus tui`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! Action buttons are a horizontal bar of buttons with icon, label, and keyboard shortcut.
//! They are ideal for action menus like user management or confirmation dialogs.
use argvus_theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `ActionButtonKind`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum ActionButtonKind {
  Primary,
  Secondary,
  Danger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `ActionButton`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct ActionButton {
  pub icon: String,
  pub label: String,
  pub shortcut: String,
  pub kind: ActionButtonKind,
}

impl ActionButton {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(
    icon: impl Into<String>,
    label: impl Into<String>,
    shortcut: impl Into<String>,
    kind: ActionButtonKind,
  ) -> Self {
    Self {
      icon: icon.into(),
      label: label.into(),
      shortcut: shortcut.into(),
      kind,
    }
  }

  /// Sets kind to Primary.
  pub fn primary(
    icon: impl Into<String>,
    label: impl Into<String>,
    shortcut: impl Into<String>,
  ) -> Self {
    Self::new(icon, label, shortcut, ActionButtonKind::Primary)
  }

  /// Sets kind to Secondary.
  pub fn secondary(
    icon: impl Into<String>,
    label: impl Into<String>,
    shortcut: impl Into<String>,
  ) -> Self {
    Self::new(icon, label, shortcut, ActionButtonKind::Secondary)
  }

  /// Sets kind to Danger.
  pub fn danger(
    icon: impl Into<String>,
    label: impl Into<String>,
    shortcut: impl Into<String>,
  ) -> Self {
    Self::new(icon, label, shortcut, ActionButtonKind::Danger)
  }
}

/// Keyboard focus of an action bar placed under a list. The list has the
/// focus by default; `Tab` moves it to the bar, where `←/→` step through the
/// enabled buttons and the cursor holds the index of the focused one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActionFocus {
  cursor: Option<usize>,
}

impl ActionFocus {
  /// Index of the focused button, or `None` while the list has the focus.
  pub fn cursor(self) -> Option<usize> {
    self.cursor
  }

  /// Gives the focus back to the list.
  pub fn focus_list(&mut self) {
    self.cursor = None;
  }

  /// `←/→`: the first press only moves the focus to the first enabled
  /// button; later presses step through `enabled`, wrapping around.
  /// `enabled` lists the indexes of the selectable buttons, in order.
  pub fn move_by(&mut self, backwards: bool, enabled: &[usize]) {
    let Some(&first) = enabled.first() else {
      return;
    };
    let Some(current) = self.cursor else {
      self.cursor = Some(first);
      return;
    };
    let next = if backwards {
      enabled
        .iter()
        .rev()
        .find(|&&index| index < current)
        .or(enabled.last())
    } else {
      enabled
        .iter()
        .find(|&&index| index > current)
        .or(enabled.first())
    };
    if let Some(&index) = next {
      self.cursor = Some(index);
    }
  }

  /// `Tab`: moves the focus between the list and the bar.
  pub fn toggle(&mut self, enabled: &[usize]) {
    if self.cursor.is_some() {
      self.focus_list();
    } else {
      self.move_by(false, enabled);
    }
  }
}

/// Executes the `height` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn height(buttons: &[ActionButton], width: u16) -> u16 {
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

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(
  frame: &mut Frame,
  area: Rect,
  buttons: &[ActionButton],
  selected: usize,
  theme: &Theme,
) {
  draw_aligned(frame, area, buttons, selected, theme, Alignment::Left);
}

/// Like `draw`, with the rows of buttons aligned as `align` (for example centered).
pub fn draw_aligned(
  frame: &mut Frame,
  area: Rect,
  buttons: &[ActionButton],
  selected: usize,
  theme: &Theme,
  align: Alignment,
) {
  draw_bar(
    frame,
    area,
    buttons,
    selected,
    theme,
    align,
    Some(theme.surface),
  );
}

/// Like `draw_aligned`, but painted without a background: the bar shows
/// whatever the page drew underneath it.
pub fn draw_aligned_transparent(
  frame: &mut Frame,
  area: Rect,
  buttons: &[ActionButton],
  selected: usize,
  theme: &Theme,
  align: Alignment,
) {
  draw_bar(frame, area, buttons, selected, theme, align, None);
}

fn draw_bar(
  frame: &mut Frame,
  area: Rect,
  buttons: &[ActionButton],
  selected: usize,
  theme: &Theme,
  align: Alignment,
  background: Option<Color>,
) {
  if buttons.is_empty() || area.width == 0 || area.height == 0 {
    return;
  }
  let mut lines: Vec<Line<'static>> = Vec::new();
  let mut current: Vec<Span<'static>> = Vec::new();
  let mut line_width = 0usize;

  for (index, button) in buttons.iter().enumerate() {
    let button_display = format!("[ {} {} ]", button.icon, button.label);
    let item_total = item_width(button);

    if line_width > 0 && line_width + 1 + item_total > area.width as usize {
      lines.push(Line::from(std::mem::take(&mut current)));
      line_width = 0;
    }

    if line_width > 0 {
      current.push(Span::raw(" "));
      line_width += 1;
    }

    let button_style = style_for(button, index == selected, theme);

    current.push(Span::styled(button_display, button_style));
    if !button.shortcut.is_empty() {
      let shortcut_display = format!("[{}]", button.shortcut);
      current.push(Span::raw(" "));
      current.push(Span::styled(shortcut_display, Style::new().fg(theme.muted)));
    }

    line_width += item_total;
  }

  if !current.is_empty() {
    lines.push(Line::from(current));
  }

  while lines.len() < area.height as usize {
    lines.push(Line::from(""));
  }

  frame.render_widget(
    Paragraph::new(lines)
      .alignment(align)
      .style(background.map_or_else(Style::new, |color| Style::new().bg(color))),
    area,
  );
}

/// Width of one button with its shortcut; a button without a shortcut is only its label.
fn item_width(button: &ActionButton) -> usize {
  let button_part = crate::text::display_width(&format!("[ {} {} ]", button.icon, button.label));
  if button.shortcut.is_empty() {
    return button_part;
  }
  let shortcut_part = crate::text::display_width(&format!("[{}]", button.shortcut));
  button_part + 1 + shortcut_part
}

/// Executes the `style_for` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn style_for(button: &ActionButton, focused: bool, theme: &Theme) -> Style {
  if focused {
    return Style::new()
      .bg(theme.selected_background)
      .fg(theme.selected_foreground)
      .add_modifier(Modifier::BOLD);
  }
  match button.kind {
    ActionButtonKind::Primary => Style::new().fg(theme.accent),
    ActionButtonKind::Danger => Style::new().fg(theme.error),
    ActionButtonKind::Secondary => Style::new().fg(theme.muted),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Executes the `theme` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn theme() -> Theme {
    Theme::load()
  }

  #[test]
  /// Executes the `height_accounts_for_wrapping` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn height_accounts_for_wrapping() {
    let buttons = vec![
      ActionButton::primary("💾", "Salvar alterações", "Ctrl+S"),
      ActionButton::secondary("🔐", "Alterar senha", "Ctrl+P"),
      ActionButton::danger("✕", "Excluir usuário", "Del"),
    ];
    assert_eq!(height(&buttons, 200), 1);
    assert!(height(&buttons, 40) >= 2);
    assert_eq!(height(&[], 30), 0);
  }

  #[test]
  /// Executes the `button_kinds_style_differs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn button_kinds_style_differs() {
    let theme = theme();
    let primary = style_for(
      &ActionButton::primary("🔧", "Primary", "C-p"),
      false,
      &theme,
    );
    let danger = style_for(&ActionButton::danger("✕", "Danger", "C-d"), false, &theme);
    let focused = style_for(&ActionButton::primary("🔧", "Primary", "C-p"), true, &theme);
    assert_ne!(primary, danger);
    assert!(focused.add_modifier.contains(Modifier::BOLD));
  }

  #[test]
  fn tab_moves_focus_between_the_list_and_the_bar() {
    let mut focus = ActionFocus::default();
    focus.toggle(&[0]);
    assert_eq!(
      focus.cursor(),
      Some(0),
      "Tab focuses the first enabled button"
    );
    focus.toggle(&[0]);
    assert_eq!(focus.cursor(), None, "Tab again gives the list the focus");
  }

  #[test]
  fn arrows_step_through_enabled_buttons_and_wrap() {
    let enabled = [0, 2];
    let mut focus = ActionFocus::default();
    focus.move_by(false, &enabled);
    assert_eq!(focus.cursor(), Some(0), "first press only focuses the bar");
    focus.move_by(false, &enabled);
    assert_eq!(focus.cursor(), Some(2));
    focus.move_by(false, &enabled);
    assert_eq!(focus.cursor(), Some(0), "Right wraps to the first");
    focus.move_by(true, &enabled);
    assert_eq!(focus.cursor(), Some(2), "Left wraps to the last");
  }

  #[test]
  fn no_enabled_button_means_no_focus() {
    let mut focus = ActionFocus::default();
    focus.toggle(&[]);
    focus.move_by(false, &[]);
    assert_eq!(focus.cursor(), None);
  }
}
