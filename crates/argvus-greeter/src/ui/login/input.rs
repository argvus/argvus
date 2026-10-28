//! Keyboard handling of the login screen: focus movement, session choice, cancel and the power menu.

use super::*;

impl LoginApp {
  /// Handles one terminal key event and updates only local UI state.
  ///
  /// Authentication and power actions are sent to their respective owners;
  /// this method never performs blocking I/O directly on the TUI thread.
  pub fn handle_key(&mut self, key: KeyEvent) {
    if self.handoff_active {
      return;
    }
    if key.kind != KeyEventKind::Press {
      return;
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
      // Ctrl+C is deliberately consumed. Exiting the login process leaves the
      // greetd compositor without a surface and produces a blank screen.
      return;
    }
    if self.power_open {
      // The modal owns all keys while open so normal form controls cannot be
      // activated behind it.
      self.handle_power_key(key.code);
      return;
    }
    match key.code {
      KeyCode::Tab => self.focus = self.focus.next(),
      KeyCode::BackTab => self.focus = self.focus.previous(),
      KeyCode::Up | KeyCode::Char('k') => self.move_vertical(-1),
      KeyCode::Down | KeyCode::Char('j') => self.move_vertical(1),
      KeyCode::Left | KeyCode::Char('h') if self.focus == Focus::Session => self.move_session(-1),
      KeyCode::Right | KeyCode::Char('l') if self.focus == Focus::Session => self.move_session(1),
      KeyCode::Char(character) if self.focus == Focus::Password => self.password.push(character),
      KeyCode::Backspace if self.focus == Focus::Password => {
        self.password.pop();
      }
      KeyCode::Enter => self.activate_focus(),
      KeyCode::Esc => self.cancel_input(),
      _ => {}
    }
    // Avatar refresh is cheap when the selected user is unchanged and ensures
    // selection changes become visible in the same event cycle.
    self.refresh_avatar();
    self.refresh_theme();
  }

  /// Moves the active indexed control vertically, with wrap-around behavior.
  pub(super) fn move_vertical(&mut self, delta: isize) {
    match self.focus {
      Focus::User => self.selected_user = move_index(self.selected_user, self.users.len(), delta),
      Focus::Session => {
        self.selected_session = move_index(self.selected_session, self.sessions.len(), delta)
      }
      Focus::Power => self.power_selected = move_index(self.power_selected, 3, delta),
      _ => {}
    }
  }

  /// Changes the selected session when focus is on the session control.
  pub(super) fn move_session(&mut self, delta: isize) {
    self.selected_session = move_index(self.selected_session, self.sessions.len(), delta);
  }

  /// Executes the action associated with the currently focused control.
  pub(super) fn activate_focus(&mut self) {
    match self.focus {
      Focus::User => self.focus = Focus::Password,
      Focus::Password => self.submit(),
      Focus::Session => self.focus = Focus::Login,
      Focus::Login => self.submit(),
      Focus::Power => self.power_open = true,
    }
  }

  /// Cancels the current password entry or returns focus to account selection.
  pub(super) fn cancel_input(&mut self) {
    if !self.password.is_empty() {
      self.password.clear();
      self.status = None;
    } else {
      self.focus = Focus::User;
    }
  }

  /// Handles navigation and activation inside the power modal.
  pub(super) fn handle_power_key(&mut self, key: KeyCode) {
    match key {
      KeyCode::Esc => self.power_open = false,
      KeyCode::Up | KeyCode::Char('k') => {
        self.power_selected = move_index(self.power_selected, 3, -1)
      }
      KeyCode::Down | KeyCode::Char('j') => {
        self.power_selected = move_index(self.power_selected, 3, 1)
      }
      KeyCode::Enter => {
        // The selected index is bounded by move_index, making this fixed array
        // safe while keeping the menu order explicit.
        let action = [
          PowerAction::Shutdown,
          PowerAction::Restart,
          PowerAction::Suspend,
        ][self.power_selected];
        match power::request(action) {
          Ok(()) => self.status = Some((self.i18n.tr("power_requested"), StatusKind::Success)),
          Err(error) => {
            self.status = Some((
              format!("{}: {error}", self.i18n.tr("power_failed")),
              StatusKind::Error,
            ))
          }
        }
        self.power_open = false;
      }
      _ => {}
    }
  }
}

/// Moves an index by one step and wraps at either end of a non-empty list.
pub(super) fn move_index(index: usize, length: usize, delta: isize) -> usize {
  // Empty lists are valid during discovery failures; returning zero avoids
  // underflow and lets the UI show its empty-state message.
  if length == 0 {
    return 0;
  }
  if delta < 0 {
    if index == 0 { length - 1 } else { index - 1 }
  } else if index + 1 >= length {
    0
  } else {
    index + 1
  }
}
