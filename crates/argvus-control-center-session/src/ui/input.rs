//! Keys and text input on the Session page.

use super::*;

impl SessionApp {
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.pending.is_none() && self.action.is_none() && self.job.is_none() && self.on_buttons {
      match key {
        KeyCode::Tab | KeyCode::BackTab => {
          self.toggle_buttons(key == KeyCode::BackTab);
          return false;
        }
        KeyCode::Left | KeyCode::Char('h') => {
          self.move_button(-1);
          return false;
        }
        KeyCode::Right | KeyCode::Char('l') => {
          self.move_button(1);
          return false;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
          self.activate_button();
          return false;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') => return false,
        _ => {}
      }
    }
    if matches!(key, KeyCode::Esc | KeyCode::Left) && self.pending.is_none() {
      if self.page == SessionPage::Home {
        return true;
      }
      self.page = if matches!(self.page, SessionPage::LogDetail(_)) {
        SessionPage::Logs
      } else {
        SessionPage::Home
      };
      self.selected = 0;
      self.on_buttons = false;
      self.button_from = None;
      if self.page == SessionPage::Home {
        self.refresh();
      }
      return false;
    }
    if self.job.is_some() || self.action.is_some() {
      return false;
    }
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          self.confirmation = ConfirmationState::default();
          let pending = self.pending.take().unwrap();
          self.status = Some(StatusMessage {
            kind: StatusKind::Info,
            text: tr(self.lang, "control_center.applying").into(),
          });
          let lang = self.lang;
          self.action = Some(self.manager.spawn(move |_| match pending {
            Pending::Restart(id) => {
              backend::restart_component(&id)?;
              Ok(JobData::Action(format!(
                "{} '{id}'",
                tr(lang, "control_center.restarted")
              )))
            }
            Pending::ToggleAutostart(id, enabled) => {
              backend::set_autostart(&id, enabled)?;
              Ok(JobData::Action(format!(
                "{} '{id}'",
                if enabled {
                  tr(lang, "control_center.enabled")
                } else {
                  tr(lang, "control_center.disabled")
                }
              )))
            }
          }));
        }
        ConfirmationOutcome::Cancelled => {
          self.pending = None;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    match key {
      KeyCode::Tab | KeyCode::BackTab => self.toggle_buttons(key == KeyCode::BackTab),
      KeyCode::Char('r') => self.refresh(),
      KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
      KeyCode::Down | KeyCode::Char('j') => self.selected = self.selected.saturating_add(1),
      KeyCode::Home => self.selected = 0,
      KeyCode::End => self.selected = self.selection_len().saturating_sub(1),
      KeyCode::PageUp => self.selected = self.selected.saturating_sub(8),
      KeyCode::PageDown => {
        self.selected = self
          .selected
          .saturating_add(8)
          .min(self.selection_len().saturating_sub(1))
      }
      KeyCode::Enter | KeyCode::Right => self.open(),
      _ => {}
    }
    self.selected = self.selected.min(self.selection_len().saturating_sub(1));
    false
  }

  pub(super) fn selection_len(&self) -> usize {
    match self.page {
      SessionPage::Home => 4,
      SessionPage::Components => self.components.len(),
      SessionPage::Autostart => self.autostart.len(),
      SessionPage::Diagnostics => self.diagnostics.len(),
      SessionPage::Logs => self.logs.len(),
      SessionPage::LogDetail(_) => 1,
    }
  }

  pub(super) fn move_button(&mut self, delta: isize) {
    let count = self.buttons().len();
    if count == 0 {
      return;
    }
    self.button_selected =
      (self.button_selected as isize + delta).rem_euclid(count as isize) as usize;
  }
}
