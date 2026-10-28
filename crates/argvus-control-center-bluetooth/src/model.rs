//! Domain state and models consumed by the UI of the Bluetooth page.
use crossterm::event::KeyCode;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothPage {
  Home,
  State,
  Devices,
  Pair,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BluetoothSnapshot {
  pub available: bool,
  pub adapter: Option<Adapter>,
  pub devices: Vec<Device>,
  pub discovering: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Adapter {
  pub address: String,
  pub name: String,
  pub powered: bool,
  pub discoverable: bool,
  pub pairable: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Device {
  pub address: String,
  pub name: String,
  pub paired: bool,
  pub trusted: bool,
  pub connected: bool,
  pub battery: Option<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
  RequestPinCode(String),
  RequestPasskey(String),
  DisplayPasskey(String, u32),
  RequestConfirmation(String, u32),
  RequestAuthorization(String),
  AuthorizeService(String, String),
  Cancelled,
  Released,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentReply {
  Pin(String),
  Passkey(u32),
  Accept,
  Reject,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentPrompt {
  Pin { device: String, input: String },
  Passkey { device: String, input: String },
  DisplayPasskey { device: String, passkey: u32 },
  Confirm { device: String, passkey: u32 },
  Authorize { device: String },
  Service { device: String, uuid: String },
}
impl AgentPrompt {
  pub fn device(&self) -> &str {
    match self {
      Self::Pin { device, .. }
      | Self::Passkey { device, .. }
      | Self::DisplayPasskey { device, .. }
      | Self::Confirm { device, .. }
      | Self::Authorize { device }
      | Self::Service { device, .. } => device,
    }
  }
  pub fn from_event(event: AgentEvent) -> Option<Self> {
    match event {
      AgentEvent::RequestPinCode(device) => Some(Self::Pin {
        device,
        input: String::new(),
      }),
      AgentEvent::RequestPasskey(device) => Some(Self::Passkey {
        device,
        input: String::new(),
      }),
      AgentEvent::DisplayPasskey(device, passkey) => Some(Self::DisplayPasskey { device, passkey }),
      AgentEvent::RequestConfirmation(device, passkey) => Some(Self::Confirm { device, passkey }),
      AgentEvent::RequestAuthorization(device) => Some(Self::Authorize { device }),
      AgentEvent::AuthorizeService(device, uuid) => Some(Self::Service { device, uuid }),
      AgentEvent::Cancelled | AgentEvent::Released => None,
    }
  }
  pub fn is_valid(&self) -> bool {
    match self {
      Self::Pin { input, .. } => valid_pin(input),
      Self::Passkey { input, .. } => valid_passkey(input),
      Self::DisplayPasskey { .. }
      | Self::Confirm { .. }
      | Self::Authorize { .. }
      | Self::Service { .. } => true,
    }
  }
  pub fn handle_key(&mut self, key: KeyCode, confirm: &mut bool) -> PromptStep {
    match self {
      Self::Pin { input, .. } => Self::text_step(input, key, false),
      Self::Passkey { input, .. } => Self::text_step(input, key, true),
      Self::DisplayPasskey { .. }
      | Self::Confirm { .. }
      | Self::Authorize { .. }
      | Self::Service { .. } => match key {
        KeyCode::Tab | KeyCode::BackTab => {
          *confirm = !*confirm;
          PromptStep::Updated
        }
        KeyCode::Left => {
          *confirm = true;
          PromptStep::Updated
        }
        KeyCode::Right => {
          *confirm = false;
          PromptStep::Updated
        }
        KeyCode::Enter if *confirm => PromptStep::Accepted(AgentReply::Accept),
        KeyCode::Enter => PromptStep::Dismissed,
        KeyCode::Esc => PromptStep::Dismissed,
        _ => PromptStep::Idle,
      },
    }
  }
  fn text_step(input: &mut String, key: KeyCode, digits_only: bool) -> PromptStep {
    match key {
      KeyCode::Char(c) => {
        let allowed = if digits_only {
          c.is_ascii_digit()
        } else {
          c.is_ascii_alphanumeric()
        };
        if allowed && input.chars().count() < 16 {
          input.push(c);
          PromptStep::Updated
        } else {
          PromptStep::Idle
        }
      }
      KeyCode::Backspace => {
        let removed = input.pop().is_some();
        if removed {
          PromptStep::Updated
        } else {
          PromptStep::Idle
        }
      }
      KeyCode::Enter
        if if digits_only {
          valid_passkey(input)
        } else {
          valid_pin(input)
        } =>
      {
        let reply = if digits_only {
          AgentReply::Passkey(input.parse().unwrap_or(0))
        } else {
          AgentReply::Pin(input.clone())
        };
        PromptStep::Accepted(reply)
      }
      KeyCode::Esc => PromptStep::Dismissed,
      _ => PromptStep::Idle,
    }
  }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptStep {
  Idle,
  Updated,
  Accepted(AgentReply),
  Dismissed,
}
pub fn sanitize_pin(input: &str) -> String {
  input
    .chars()
    .filter(|c| c.is_ascii_alphanumeric())
    .take(16)
    .collect()
}
pub fn valid_pin(input: &str) -> bool {
  !input.is_empty()
    && input.chars().count() <= 16
    && input.chars().all(|c| c.is_ascii_alphanumeric())
}
pub fn valid_passkey(input: &str) -> bool {
  !input.is_empty()
    && input.chars().count() <= 6
    && input.chars().all(|c| c.is_ascii_digit())
    && input.parse::<u32>().is_ok_and(|value| value <= 999_999)
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn pin_accepts_alphanumeric_within_sixteen_chars() {
    assert!(valid_pin("1234"));
    assert!(valid_pin("CODE9"));
    assert!(!valid_pin(""));
    assert!(!valid_pin("hello world"));
    assert!(!valid_pin("CODE-9"));
    assert!(!valid_pin("12345678901234567"));
  }
  #[test]
  fn passkey_is_at_most_six_digits_and_bounded() {
    assert!(valid_passkey("123456"));
    assert!(valid_passkey("0"));
    assert!(!valid_passkey(""));
    assert!(!valid_passkey("12a4"));
    assert!(!valid_passkey("1234567"));
  }
  #[test]
  fn prompt_maps_from_events_and_preserves_device() {
    let prompt = AgentPrompt::from_event(AgentEvent::RequestConfirmation(
      "AA:BB:CC:DD:EE:FF".into(),
      123456,
    ));
    assert_eq!(
      prompt,
      Some(AgentPrompt::Confirm {
        device: "AA:BB:CC:DD:EE:FF".into(),
        passkey: 123456,
      })
    );
    assert_eq!(AgentPrompt::from_event(AgentEvent::Cancelled), None);
  }
  #[test]
  fn text_prompt_types_and_confirms_pin() {
    let mut prompt = AgentPrompt::Pin {
      device: "AA:BB:CC:DD:EE:FF".into(),
      input: String::new(),
    };
    let mut confirm = false;
    assert_eq!(
      prompt.handle_key(KeyCode::Char('1'), &mut confirm),
      PromptStep::Updated
    );
    assert_eq!(
      prompt.handle_key(KeyCode::Char('2'), &mut confirm),
      PromptStep::Updated
    );
    assert_eq!(
      prompt.handle_key(KeyCode::Char(' '), &mut confirm),
      PromptStep::Idle
    );
    assert_eq!(
      prompt.handle_key(KeyCode::Enter, &mut confirm),
      PromptStep::Accepted(AgentReply::Pin("12".into()))
    );
  }
  #[test]
  fn decision_prompt_refuses_by_default_and_can_accept() {
    let mut prompt = AgentPrompt::Confirm {
      device: "AA:BB:CC:DD:EE:FF".into(),
      passkey: 123456,
    };
    let mut confirm = false;
    assert_eq!(
      prompt.handle_key(KeyCode::Enter, &mut confirm),
      PromptStep::Dismissed
    );
    assert_eq!(
      prompt.handle_key(KeyCode::Tab, &mut confirm),
      PromptStep::Updated
    );
    assert!(confirm);
    assert_eq!(
      prompt.handle_key(KeyCode::Enter, &mut confirm),
      PromptStep::Accepted(AgentReply::Accept)
    );
  }
}
