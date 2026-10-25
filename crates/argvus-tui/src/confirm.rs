//! Single confirmation component for destructive and privileged actions.
//!
//! Layout: title, message, then two vertical rows (`Confirm` / `Cancel`).
//! Enter runs the focused row, `y` confirms, `n` and Esc cancel, and the
//! arrows, `j`/`k` and Tab move the focus between the two rows.
//!
//! [`draw_choice`] is the same popup with several options above `Cancel`,
//! for an action that can run in more than one way (for example removing a
//! package alone or together with the packages that need it).
//!
//! The focus starts on `Cancel`. That is the behavior of every existing
//! `components::ConfirmationState` use in the Control Center (all of them
//! start from `ConfirmationState::default()`), so plain confirmations keep
//! it, and dangerous ones require it.

use std::time::Duration;

use argvus_theme::Theme;
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::text::display_width;

/// Result of a key press while the confirmation is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmOutcome {
  /// Still open; the focus may have moved.
  Pending,
  Confirmed,
  Cancelled,
}

/// Focus of the confirmation component.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfirmState {
  confirm_focused: bool,
}

impl ConfirmState {
  /// A fresh confirmation with the focus on `Cancel`.
  pub fn new() -> Self {
    Self::default()
  }

  pub fn is_confirm_focused(&self) -> bool {
    self.confirm_focused
  }

  pub fn handle(&mut self, key: KeyCode) -> ConfirmOutcome {
    match key {
      KeyCode::Char('y' | 'Y') => ConfirmOutcome::Confirmed,
      KeyCode::Char('n' | 'N') | KeyCode::Esc => ConfirmOutcome::Cancelled,
      KeyCode::Enter if self.confirm_focused => ConfirmOutcome::Confirmed,
      KeyCode::Enter => ConfirmOutcome::Cancelled,
      // `Confirm` is the upper row, `Cancel` the lower one.
      KeyCode::Up | KeyCode::Char('k') | KeyCode::Left => {
        self.confirm_focused = true;
        ConfirmOutcome::Pending
      }
      KeyCode::Down | KeyCode::Char('j') | KeyCode::Right => {
        self.confirm_focused = false;
        ConfirmOutcome::Pending
      }
      KeyCode::Tab | KeyCode::BackTab => {
        self.confirm_focused = !self.confirm_focused;
        ConfirmOutcome::Pending
      }
      _ => ConfirmOutcome::Pending,
    }
  }
}

/// Texts and options of one confirmation. Labels come translated from the
/// caller.
#[derive(Debug, Clone, Copy)]
pub struct ConfirmDialog<'a> {
  pub title: &'a str,
  pub message: &'a str,
  pub confirm: &'a str,
  pub cancel: &'a str,
  /// Draws the border and the `Confirm` row with the theme's error color.
  pub danger: bool,
  /// Time left before the page applies its default outcome (for example the
  /// display revert countdown). Shown under the message.
  pub deadline: Option<Duration>,
}

/// Draws the confirmation as a centered popup over `area`.
pub fn draw_confirm(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  dialog: ConfirmDialog<'_>,
  state: &ConfirmState,
) {
  let mut lines = message_lines(dialog.message, dialog.deadline, theme);
  let accent = if dialog.danger {
    theme.error
  } else {
    theme.accent
  };
  let label_width = display_width(dialog.confirm).max(display_width(dialog.cancel));
  lines.push(choice_line(
    dialog.confirm,
    label_width,
    state.confirm_focused,
    accent,
    theme,
  ));
  lines.push(choice_line(
    dialog.cancel,
    label_width,
    !state.confirm_focused,
    theme.foreground,
    theme,
  ));
  draw_popup(frame, area, theme, dialog.title, dialog.danger, lines);
}

/// Result of a key press while the choice dialog is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceOutcome {
  /// Still open; the focus may have moved.
  Pending,
  /// The option at this index was chosen.
  Chosen(usize),
  Cancelled,
}

/// Focus of the choice dialog: an option index, or `Cancel` after the last
/// option. Like [`ConfirmState`], it starts on `Cancel`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChoiceState {
  /// Focused option; `None` is `Cancel`.
  focused: Option<usize>,
}

impl ChoiceState {
  /// A fresh choice with the focus on `Cancel`.
  pub fn new() -> Self {
    Self::default()
  }

  /// The focused option, or `None` on `Cancel`.
  pub fn focused(&self) -> Option<usize> {
    self.focused
  }

  /// Handles `key` for a dialog with `options` options above `Cancel`.
  /// Enter runs the focused row, `n` and Esc cancel, and the arrows, `j`/`k`
  /// and Tab move the focus, wrapping around. There is no `y`: with several
  /// options, the choice must be explicit.
  pub fn handle(&mut self, key: KeyCode, options: usize) -> ChoiceOutcome {
    // Rows top to bottom: the options, then `Cancel` at index `options`.
    let rows = options + 1;
    let current = self.focused.unwrap_or(options);
    let next = match key {
      KeyCode::Char('n' | 'N') | KeyCode::Esc => return ChoiceOutcome::Cancelled,
      KeyCode::Enter => {
        return self
          .focused
          .map_or(ChoiceOutcome::Cancelled, ChoiceOutcome::Chosen);
      }
      KeyCode::Up | KeyCode::Char('k') | KeyCode::Left | KeyCode::BackTab => {
        (current + rows - 1) % rows
      }
      KeyCode::Down | KeyCode::Char('j') | KeyCode::Right | KeyCode::Tab => (current + 1) % rows,
      _ => return ChoiceOutcome::Pending,
    };
    self.focused = (next < options).then_some(next);
    ChoiceOutcome::Pending
  }
}

/// Texts of one choice dialog. Labels come translated from the caller.
#[derive(Debug, Clone, Copy)]
pub struct ChoiceDialog<'a> {
  pub title: &'a str,
  pub message: &'a str,
  /// Options, top to bottom, drawn above `cancel`.
  pub options: &'a [&'a str],
  pub cancel: &'a str,
  /// Draws the border and the options with the theme's error color.
  pub danger: bool,
}

/// Draws the choice dialog as a centered popup over `area`.
pub fn draw_choice(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  dialog: ChoiceDialog<'_>,
  state: &ChoiceState,
) {
  let mut lines = message_lines(dialog.message, None, theme);
  let accent = if dialog.danger {
    theme.error
  } else {
    theme.accent
  };
  let label_width = dialog
    .options
    .iter()
    .chain([&dialog.cancel])
    .map(|label| display_width(label))
    .max()
    .unwrap_or(0);
  for (index, option) in dialog.options.iter().enumerate() {
    lines.push(choice_line(
      option,
      label_width,
      state.focused == Some(index),
      accent,
      theme,
    ));
  }
  lines.push(choice_line(
    dialog.cancel,
    label_width,
    state.focused.is_none(),
    theme.foreground,
    theme,
  ));
  draw_popup(frame, area, theme, dialog.title, dialog.danger, lines);
}

/// The message block of a dialog: a blank line, the message, the remaining
/// time when there is a deadline, and a blank line before the options.
fn message_lines(message: &str, deadline: Option<Duration>, theme: &Theme) -> Vec<Line<'static>> {
  let mut lines = vec![Line::from("")];
  lines.extend(message.lines().map(|text| Line::from(text.to_owned())));
  if let Some(remaining) = deadline {
    lines.push(Line::from(Span::styled(
      format!("{} s", remaining.as_secs()),
      Style::new().fg(theme.warning).add_modifier(Modifier::BOLD),
    )));
  }
  lines.push(Line::from(""));
  lines
}

/// Draws `lines` in a centered, bordered popup sized to its wrapped content.
fn draw_popup(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  title: &str,
  danger: bool,
  lines: Vec<Line<'static>>,
) {
  let popup_width = area.width.saturating_sub(8).clamp(36, 72).min(area.width);
  let content_width = usize::from(popup_width.saturating_sub(2)).max(1);

  // Wrapped message rows decide the height; borders add two more.
  let content_rows = lines
    .iter()
    .map(|line| line.width().max(1).div_ceil(content_width))
    .sum::<usize>();
  let height = u16::try_from(content_rows + 2)
    .unwrap_or(u16::MAX)
    .min(area.height);
  let popup = crate::chrome::centered(area, popup_width, height);
  frame.render_widget(Clear, popup);
  let border = if danger {
    theme.error
  } else {
    theme.border_active
  };
  frame.render_widget(
    Paragraph::new(lines)
      .alignment(Alignment::Center)
      .wrap(Wrap { trim: true })
      .block(
        Block::bordered()
          .title(format!(" {title} "))
          .border_style(Style::new().fg(border))
          .style(Style::new().bg(theme.background).fg(theme.foreground)),
      ),
    popup,
  );
}

/// One option row; the focused one carries the cursor and the selection
/// colors, so focus never depends on color alone.
///
/// Every row is padded to `label_width` so they line up when centered.
fn choice_line(
  label: &str,
  label_width: usize,
  is_focused: bool,
  color: ratatui::style::Color,
  theme: &Theme,
) -> Line<'static> {
  let fill = " ".repeat(label_width.saturating_sub(display_width(label)));
  let text = format!(" {} {label}{fill} ", if is_focused { ">" } else { " " });
  let style = if is_focused {
    Style::new()
      .fg(theme.selected_foreground)
      .bg(theme.selected_background)
      .add_modifier(Modifier::BOLD)
  } else {
    Style::new().fg(color)
  };
  Line::from(Span::styled(text, style))
}

#[cfg(test)]
mod tests {
  use super::*;
  use ratatui::{Terminal, backend::TestBackend};

  #[test]
  fn focus_starts_on_cancel_and_enter_cancels() {
    let mut state = ConfirmState::new();
    assert!(!state.is_confirm_focused());
    assert_eq!(state.handle(KeyCode::Enter), ConfirmOutcome::Cancelled);
  }

  #[test]
  fn y_confirms_and_n_or_esc_cancel_regardless_of_focus() {
    for (key, expected) in [
      (KeyCode::Char('y'), ConfirmOutcome::Confirmed),
      (KeyCode::Char('Y'), ConfirmOutcome::Confirmed),
      (KeyCode::Char('n'), ConfirmOutcome::Cancelled),
      (KeyCode::Char('N'), ConfirmOutcome::Cancelled),
      (KeyCode::Esc, ConfirmOutcome::Cancelled),
    ] {
      for confirm_focused in [false, true] {
        let mut state = ConfirmState { confirm_focused };
        assert_eq!(
          state.handle(key),
          expected,
          "{key:?} focus={confirm_focused}"
        );
      }
    }
  }

  #[test]
  fn navigation_keys_move_focus_between_rows() {
    let mut state = ConfirmState::new();
    assert_eq!(state.handle(KeyCode::Up), ConfirmOutcome::Pending);
    assert!(state.is_confirm_focused());
    assert_eq!(state.handle(KeyCode::Enter), ConfirmOutcome::Confirmed);
    state.handle(KeyCode::Char('j'));
    assert!(!state.is_confirm_focused());
    state.handle(KeyCode::Char('k'));
    assert!(state.is_confirm_focused());
    state.handle(KeyCode::Right);
    assert!(!state.is_confirm_focused());
    state.handle(KeyCode::Left);
    assert!(state.is_confirm_focused());
    state.handle(KeyCode::Tab);
    assert!(!state.is_confirm_focused());
    state.handle(KeyCode::BackTab);
    assert!(state.is_confirm_focused());
  }

  #[test]
  fn unrelated_keys_keep_the_dialog_open() {
    let mut state = ConfirmState::new();
    assert_eq!(state.handle(KeyCode::Char('q')), ConfirmOutcome::Pending);
    assert_eq!(state.handle(KeyCode::Char(' ')), ConfirmOutcome::Pending);
    assert!(!state.is_confirm_focused());
  }

  fn render(dialog: ConfirmDialog<'_>, state: &ConfirmState) -> Vec<String> {
    let (width, height) = (60u16, 16u16);
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let theme = Theme::load();
    terminal
      .draw(|frame| draw_confirm(frame, Rect::new(0, 0, width, height), &theme, dialog, state))
      .unwrap();
    terminal
      .backend()
      .buffer()
      .content
      .chunks(usize::from(width))
      .map(|line| line.iter().map(|cell| cell.symbol()).collect())
      .collect()
  }

  fn dialog(deadline: Option<Duration>) -> ConfirmDialog<'static> {
    ConfirmDialog {
      title: "Remove package",
      message: "Remove: chromium\nDownload bytes: 0",
      confirm: "Confirm",
      cancel: "Cancel",
      danger: true,
      deadline,
    }
  }

  #[test]
  fn renders_message_lines_and_vertical_options_with_cursor_on_focus() {
    let screen = render(dialog(None), &ConfirmState::new());
    let row_of = |needle: &str| {
      screen
        .iter()
        .position(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("{needle} missing: {screen:#?}"))
    };
    assert!(row_of("Remove: chromium") < row_of("Download bytes: 0"));
    let confirm = row_of("Confirm");
    let cancel = row_of("Cancel");
    assert_eq!(cancel, confirm + 1, "options are stacked vertically");
    assert!(screen[cancel].contains("> Cancel"));
    assert!(!screen[confirm].contains("> Confirm"));
    assert!(
      !screen.iter().any(|line| line.contains("[ ")),
      "no button brackets"
    );
  }

  #[test]
  fn choice_focus_starts_on_cancel_and_moves_through_every_row() {
    let mut state = ChoiceState::new();
    assert_eq!(state.focused(), None);
    assert_eq!(state.handle(KeyCode::Enter, 2), ChoiceOutcome::Cancelled);
    assert_eq!(state.handle(KeyCode::Up, 2), ChoiceOutcome::Pending);
    assert_eq!(state.focused(), Some(1));
    state.handle(KeyCode::Char('k'), 2);
    assert_eq!(state.focused(), Some(0));
    assert_eq!(state.handle(KeyCode::Enter, 2), ChoiceOutcome::Chosen(0));
    state.handle(KeyCode::Up, 2);
    assert_eq!(
      state.focused(),
      None,
      "wraps from the first option to Cancel"
    );
    state.handle(KeyCode::Tab, 2);
    assert_eq!(state.focused(), Some(0));
    state.handle(KeyCode::Down, 2);
    assert_eq!(state.handle(KeyCode::Enter, 2), ChoiceOutcome::Chosen(1));
  }

  #[test]
  fn choice_has_no_y_and_n_or_esc_cancel() {
    let mut state = ChoiceState::new();
    state.handle(KeyCode::Up, 2);
    assert_eq!(state.handle(KeyCode::Char('y'), 2), ChoiceOutcome::Pending);
    assert_eq!(
      state.handle(KeyCode::Char('n'), 2),
      ChoiceOutcome::Cancelled
    );
    assert_eq!(state.handle(KeyCode::Esc, 2), ChoiceOutcome::Cancelled);
  }

  #[test]
  fn renders_every_choice_above_cancel_with_the_cursor_on_cancel() {
    let (width, height) = (60u16, 16u16);
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let theme = Theme::load();
    let dialog = ChoiceDialog {
      title: "Remove package",
      message: "7zip is required by julia.",
      options: &["Remove only 7zip", "Remove with dependents"],
      cancel: "Cancel",
      danger: true,
    };
    terminal
      .draw(|frame| {
        draw_choice(
          frame,
          Rect::new(0, 0, width, height),
          &theme,
          dialog,
          &ChoiceState::new(),
        )
      })
      .unwrap();
    let screen: Vec<String> = terminal
      .backend()
      .buffer()
      .content
      .chunks(usize::from(width))
      .map(|line| line.iter().map(|cell| cell.symbol()).collect())
      .collect();
    let row_of = |needle: &str| {
      screen
        .iter()
        .position(|line| line.contains(needle))
        .unwrap()
    };
    let only = row_of("Remove only 7zip");
    assert_eq!(row_of("Remove with dependents"), only + 1);
    assert_eq!(row_of("Cancel"), only + 2);
    assert!(screen[only + 2].contains("> Cancel"), "{screen:#?}");
  }

  #[test]
  fn renders_remaining_time_when_a_deadline_is_set() {
    let screen = render(dialog(Some(Duration::from_secs(15))), &ConfirmState::new());
    assert!(
      screen.iter().any(|line| line.contains("15 s")),
      "{screen:#?}"
    );
  }
}
