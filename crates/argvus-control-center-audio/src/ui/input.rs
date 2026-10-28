//! Keys and text input on the Audio page.

use super::*;

impl AudioApp {
  pub(super) fn selected_device(&self) -> Option<&AudioDevice> {
    self
      .devices()
      .get(self.selected.index)
      .copied()
      .filter(|device| device.id > 0)
  }

  /// Whether typed characters currently go to the volume value field, so
  /// `q`/`?` must not act as the global quit/help keys. The confirmation
  /// sits above the field in [`Self::handle`] and does not take text.
  pub fn captures_text(&self) -> bool {
    !self.confirm_discard && self.volume_input.is_some()
  }

  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.confirm_discard {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          self.confirm_discard = false;
          self.confirmation = ConfirmationState::default();
          self.leave_page();
        }
        ConfirmationOutcome::Cancelled => {
          self.confirm_discard = false;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    if self.volume_input.is_some() {
      return self.handle_volume_input(key);
    }
    if self.has_device_actions() {
      match key {
        KeyCode::Left | KeyCode::Char('-') => {
          self.adjust_volume(-5);
          return false;
        }
        KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') => {
          self.adjust_volume(5);
          return false;
        }
        KeyCode::Char(' ') => {
          self.toggle_mute();
          return false;
        }
        KeyCode::Char('v') => {
          self.volume_input = Some(String::new());
          return false;
        }
        KeyCode::Char('s') => {
          self.save();
          return false;
        }
        _ => {}
      }
    }
    if matches!(key, KeyCode::Esc | KeyCode::Left) {
      if self.page == AudioPage::Home {
        return true;
      }
      if self.has_changes() {
        self.confirm_discard = true;
        self.confirmation = ConfirmationState::default();
        return false;
      }
      self.leave_page();
      return false;
    }
    match key {
      KeyCode::Char('r') => self.reload(),
      KeyCode::Up
      | KeyCode::Char('k')
      | KeyCode::Down
      | KeyCode::Char('j')
      | KeyCode::Home
      | KeyCode::End
      | KeyCode::PageUp
      | KeyCode::PageDown => {
        self.selected.handle(key, self.row_count(), 8);
      }
      KeyCode::Enter | KeyCode::Right => self.open_selected(),
      _ => {}
    }
    false
  }

  pub(super) fn handle_volume_input(&mut self, key: KeyCode) -> bool {
    let input = self.volume_input.as_mut().unwrap();
    match key {
      KeyCode::Esc => self.volume_input = None,
      KeyCode::Enter => {
        let value = input
          .parse::<i16>()
          .ok()
          .filter(|value| (0..=100).contains(value))
          .map(clamp_volume);
        self.volume_input = None;
        let id = self.selected_device().map(|d| d.id);
        match (value, id) {
          (Some(v), Some(id)) => {
            self.draft.volumes.insert(id, v);
          }
          _ => self.error(tr(self.lang, "control_center.invalid_volume_0_100")),
        }
      }
      KeyCode::Backspace => {
        input.pop();
      }
      KeyCode::Char(c) if c.is_ascii_digit() && input.len() < 3 => input.push(c),
      _ => {}
    }
    false
  }
}
