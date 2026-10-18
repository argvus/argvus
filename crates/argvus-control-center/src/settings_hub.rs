//! Settings of the ARGVUS Control Center itself: its language and its style.
//!
//! The hub only chooses where to go. Language opens the settings page of the
//! interface language; Style opens the configuration screen with the icons and
//! transparency controls.

use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::page::{list, shell};
use crossterm::event::KeyCode;
use ratatui::Frame;

/// What the hub asks the application to do after a key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HubAction {
  None,
  Back,
  Language,
  Style,
}

/// Cursor of the Settings hub. It has no other state, so it is rebuilt cheaply.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SettingsHub {
  selected: usize,
}

impl SettingsHub {
  /// Number of rows listed by the hub.
  const ROWS: usize = 2;

  /// Applies one key press and reports the navigation it asks for.
  pub fn handle(&mut self, key: KeyCode) -> HubAction {
    match key {
      KeyCode::Esc => HubAction::Back,
      KeyCode::Up | KeyCode::Char('k') => {
        self.selected = self.selected.saturating_sub(1);
        HubAction::None
      }
      KeyCode::Down | KeyCode::Char('j') => {
        self.selected = (self.selected + 1).min(Self::ROWS - 1);
        HubAction::None
      }
      KeyCode::Enter | KeyCode::Right => match self.selected {
        0 => HubAction::Language,
        _ => HubAction::Style,
      },
      _ => HubAction::None,
    }
  }

  /// Draws the hub inside the standard shell of the Control Center.
  pub fn draw(&self, frame: &mut Frame, lang: Lang, theme: &Theme) {
    let body = shell(
      frame,
      frame.area(),
      theme,
      lang,
      tr(lang, "control_center.settings"),
      tr(lang, "control_center.navigate_enter_open_esc_back_help"),
    );
    let rows = [
      format!("  {}", tr(lang, "control_center.language")),
      format!("  {} ›", tr(lang, "control_center.style")),
    ];
    list(frame, body, theme, &rows, self.selected);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Esc leaves the hub, and Enter reaches each row in order.
  fn keys_move_the_cursor_and_choose_a_row() {
    let mut hub = SettingsHub::default();
    assert_eq!(hub.handle(KeyCode::Enter), HubAction::Language);
    assert_eq!(hub.handle(KeyCode::Down), HubAction::None);
    assert_eq!(hub.handle(KeyCode::Enter), HubAction::Style);
    assert_eq!(hub.handle(KeyCode::Down), HubAction::None);
    assert_eq!(hub.handle(KeyCode::Enter), HubAction::Style);
    assert_eq!(hub.handle(KeyCode::Esc), HubAction::Back);
  }
}
