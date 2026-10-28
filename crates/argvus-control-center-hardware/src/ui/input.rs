//! Keys and text input on the Hardware Overview page.

use super::*;

impl HardwareApp {
  pub(super) fn selection_len(&self) -> usize {
    match self.page {
      HardwarePage::Home => 6,
      HardwarePage::Cpu => self.snapshot.cpu.governors.len(),
      HardwarePage::Gpu => self.snapshot.gpus.len(),
      HardwarePage::Power => self.snapshot.energy.profiles.len(),
      HardwarePage::Devices => self.snapshot.devices.len(),
      _ => 1,
    }
  }

  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          if let Some(Pending::Action(act)) = self.pending.take() {
            self.execute_action(act);
          }
        }
        ConfirmationOutcome::Cancelled => {
          self.pending = None;
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }

    if matches!(key, KeyCode::Esc | KeyCode::Left) && !self.on_buttons {
      if self.page == HardwarePage::Home {
        return true;
      }
      self.navigate_back();
      return false;
    }

    if self.busy() {
      return false;
    }

    let buttons = self.buttons();

    if key == KeyCode::Tab && !buttons.is_empty() {
      if self.on_buttons {
        self.on_buttons = false;
        if let Some(old) = self.button_from {
          self.selected.index = old;
        }
      } else {
        self.on_buttons = true;
        self.button_from = Some(self.selected.index);
        self.button_selected = 0;
      }
      return false;
    }

    if self.on_buttons {
      match key {
        KeyCode::Left | KeyCode::Char('h') => {
          self.button_selected = self.button_selected.saturating_sub(1);
        }
        KeyCode::Right | KeyCode::Char('l') => {
          self.button_selected += 1;
          if !buttons.is_empty() {
            self.button_selected = self.button_selected.min(buttons.len() - 1);
          }
        }
        KeyCode::Enter => {
          if let Some((action, _)) = buttons.get(self.button_selected) {
            self.trigger_button(*action);
          }
        }
        KeyCode::Esc => {
          self.on_buttons = false;
          if let Some(old) = self.button_from {
            self.selected.index = old;
          }
        }
        _ => {}
      }
      return false;
    }

    if self.selected.handle(key, self.selection_len(), 8) {
      return false;
    }

    match key {
      KeyCode::Char('r') => self.refresh(),

      KeyCode::Char('g') if self.page == HardwarePage::Cpu => {
        if let Some(gov) = self.snapshot.cpu.governors.get(self.selected.index) {
          self.pending = Some(Pending::Action(HardwareAction::SetGovernor(gov.clone())));
          self.confirmation = ConfirmationState::default();
        }
      }
      KeyCode::Char('e') if self.page == HardwarePage::Power => {
        if let Some(prof) = self.snapshot.energy.profiles.get(self.selected.index) {
          self.pending = Some(Pending::Action(HardwareAction::SetProfile(prof.clone())));
          self.confirmation = ConfirmationState::default();
        }
      }

      KeyCode::Enter | KeyCode::Right => match self.page {
        HardwarePage::Home => {
          self.page = [
            HardwarePage::Summary,
            HardwarePage::Cpu,
            HardwarePage::Gpu,
            HardwarePage::Memory,
            HardwarePage::Power,
            HardwarePage::Devices,
          ][self.selected.index];
          self.selected.index = 0;
        }
        HardwarePage::Devices if self.snapshot.devices.get(self.selected.index).is_some() => {
          self.page = HardwarePage::DeviceDetail(self.selected.index);
        }
        HardwarePage::Gpu if self.snapshot.gpus.get(self.selected.index).is_some() => {
          self.page = HardwarePage::GpuDetail(self.selected.index);
          self.selected.index = 0;
        }
        HardwarePage::Cpu => {
          if let Some(gov) = self.snapshot.cpu.governors.get(self.selected.index) {
            self.pending = Some(Pending::Action(HardwareAction::SetGovernor(gov.clone())));
            self.confirmation = ConfirmationState::default();
          }
        }
        HardwarePage::Power => {
          if let Some(prof) = self.snapshot.energy.profiles.get(self.selected.index) {
            self.pending = Some(Pending::Action(HardwareAction::SetProfile(prof.clone())));
            self.confirmation = ConfirmationState::default();
          }
        }
        _ => {}
      },
      _ => {}
    }
    false
  }
}
