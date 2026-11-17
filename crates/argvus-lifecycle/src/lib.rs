//! Selection and confirmation state for the lifecycle interface.

pub use argvus_session_actions::Action;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
  List,
  Confirm { action: Action, confirmed: bool },
  Running(Action),
}

#[derive(Debug)]
pub struct State {
  pub selected: Action,
  pub screen: Screen,
}

impl Default for State {
  fn default() -> Self {
    Self {
      selected: Action::Lock,
      screen: Screen::List,
    }
  }
}

impl State {
  pub fn select(&mut self, action: Action) {
    if self.screen == Screen::List {
      self.selected = action;
    }
  }

  pub fn move_selection(&mut self, offset: i32) {
    if self.screen != Screen::List {
      return;
    }
    let index = Action::ALL
      .iter()
      .position(|action| *action == self.selected)
      .unwrap_or(0);
    let next = (index as i32 + offset).rem_euclid(Action::ALL.len() as i32) as usize;
    self.selected = Action::ALL[next];
  }

  pub fn activate(&mut self) -> Option<Action> {
    match self.screen {
      Screen::List if self.selected.requires_confirmation() => {
        self.screen = Screen::Confirm {
          action: self.selected,
          confirmed: false,
        };
        None
      }
      Screen::List => {
        self.screen = Screen::Running(self.selected);
        Some(self.selected)
      }
      Screen::Confirm {
        action,
        confirmed: true,
      } => {
        self.screen = Screen::Running(action);
        Some(action)
      }
      Screen::Confirm {
        confirmed: false, ..
      } => {
        self.screen = Screen::List;
        None
      }
      Screen::Running(_) => None,
    }
  }

  pub fn set_confirmation(&mut self, confirmed: bool) {
    if let Screen::Confirm { action, .. } = self.screen {
      self.screen = Screen::Confirm { action, confirmed };
    }
  }

  pub fn cancel(&mut self) -> bool {
    match self.screen {
      Screen::List => true,
      Screen::Confirm { .. } => {
        self.screen = Screen::List;
        false
      }
      Screen::Running(_) => false,
    }
  }

  pub fn execution_failed(&mut self) {
    if matches!(self.screen, Screen::Running(_)) {
      self.screen = Screen::List;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn selection_wraps_and_confirmation_defaults_to_cancel() {
    let mut state = State::default();
    state.move_selection(-1);
    assert_eq!(state.selected, Action::Shutdown);
    assert_eq!(state.activate(), None);
    assert_eq!(state.activate(), None);
    assert_eq!(state.screen, Screen::List);
    state.activate();
    state.set_confirmation(true);
    assert_eq!(state.activate(), Some(Action::Shutdown));
    assert_eq!(state.activate(), None);
    state.execution_failed();
    assert_eq!(state.screen, Screen::List);
  }

  #[test]
  fn destructive_actions_use_graceful_shutdown() {
    assert_eq!(Action::LogOut.command(), ("hyprshutdown", &[][..]));
    assert_eq!(
      Action::Reboot.command(),
      ("hyprshutdown", &["--post-cmd", "systemctl reboot"][..])
    );
    assert_eq!(
      Action::Shutdown.command(),
      ("hyprshutdown", &["--post-cmd", "systemctl poweroff"][..])
    );
  }
}
