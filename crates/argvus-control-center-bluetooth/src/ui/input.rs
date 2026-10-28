//! Keys and text input on the Bluetooth page.

use super::*;

impl BluetoothApp {
  pub(super) fn selected_device(&self) -> Option<&Device> {
    self.snapshot.devices.get(self.selected.index)
  }

  pub fn handle(&mut self, key: KeyCode) -> bool {
    if let Some(mut prompt) = self.prompt.take() {
      let step = prompt.handle_key(key, &mut self.prompt_confirm);
      match step {
        PromptStep::Accepted(reply) => {
          if let Some(agent) = &self.agent {
            agent.reply(reply);
          }
          self.prompt_confirm = false;
        }
        PromptStep::Dismissed => {
          if let Some(agent) = &self.agent {
            agent.reply(AgentReply::Reject);
          }
          self.prompt_confirm = false;
        }
        PromptStep::Idle | PromptStep::Updated => {
          self.prompt = Some(prompt);
        }
      }
      return false;
    }
    if self.confirm_remove {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          self.confirm_remove = false;
          self.confirmation = ConfirmationState::default();
          self.act_device("remove");
        }
        ConfirmationOutcome::Cancelled => {
          self.confirm_remove = false;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    if self.on_buttons && !self.buttons().is_empty() {
      match key {
        KeyCode::Tab => {
          self.toggle_buttons(false);
          return false;
        }
        KeyCode::BackTab => {
          self.toggle_buttons(true);
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
    if matches!(key, KeyCode::Esc | KeyCode::Left) {
      if self.page == BluetoothPage::Home {
        return true;
      }
      self.page = BluetoothPage::Home;
      self.selected.index = 0;
      self.on_buttons = false;
      self.button_from = None;
      self.stop_agent();
      return false;
    }
    if key == KeyCode::Char('r') {
      if self.page == BluetoothPage::Pair {
        self.start_scan();
      } else {
        self.reload();
      }
      return false;
    }
    if self.action.is_some() {
      return false;
    }
    if self.page == BluetoothPage::Home && matches!(key, KeyCode::Enter | KeyCode::Right) {
      self.page = match self.selected.index {
        0 => BluetoothPage::State,
        1 => BluetoothPage::Devices,
        _ => BluetoothPage::Pair,
      };
      self.selected.index = 0;
      self.on_buttons = false;
      self.button_selected = 0;
      self.button_from = None;
      self.status = None;
      if self.page == BluetoothPage::Pair {
        self.open_agent();
      }
      self.reload();
      return false;
    }
    if self.selected.handle(key, self.row_count(), 8) {
      return false;
    }
    match (self.page, key) {
      (_, KeyCode::Tab | KeyCode::BackTab) => self.toggle_buttons(key == KeyCode::BackTab),
      (BluetoothPage::Pair, KeyCode::Enter) => self.act_pair(),
      (BluetoothPage::Devices, KeyCode::Enter) => {
        if self.selected_device().map(|d| d.connected).unwrap_or(false) {
          self.act_device("disconnect");
        } else if self.selected_device().map(|d| d.paired).unwrap_or(false) {
          self.act_device("connect");
        } else {
          self.act_device("pair");
        }
      }
      _ => {}
    }
    false
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
