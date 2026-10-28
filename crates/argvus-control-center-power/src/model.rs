//! Domain state and models consumed by the UI of the Power page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerPage {
  Home,
}

/// The logind "HandleLidSwitch" family of behaviors. `HandledBySystem` maps to
/// the newer `handled-by-system` value and is only used when another component
/// owns the lid policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerBehavior {
  Ignore,
  Poweroff,
  Reboot,
  Halt,
  Kexec,
  Suspend,
  Hibernate,
  SuspendThenHibernate,
  Lock,
  HandledBySystem,
}

impl PowerBehavior {
  pub const ALL: [Self; 9] = [
    Self::Ignore,
    Self::Suspend,
    Self::Hibernate,
    Self::SuspendThenHibernate,
    Self::Lock,
    Self::Poweroff,
    Self::Reboot,
    Self::Halt,
    Self::HandledBySystem,
  ];

  pub fn parse(value: &str) -> Option<Self> {
    Some(match value.trim().to_ascii_lowercase().as_str() {
      "ignore" => Self::Ignore,
      "poweroff" | "off" => Self::Poweroff,
      "reboot" => Self::Reboot,
      "halt" => Self::Halt,
      "kexec" => Self::Kexec,
      "suspend" => Self::Suspend,
      "hibernate" => Self::Hibernate,
      "suspend-then-hibernate" => Self::SuspendThenHibernate,
      "lock" => Self::Lock,
      "handled-by-system" => Self::HandledBySystem,
      _ => return None,
    })
  }

  pub fn value(self) -> &'static str {
    match self {
      Self::Ignore => "ignore",
      Self::Poweroff => "poweroff",
      Self::Reboot => "reboot",
      Self::Halt => "halt",
      Self::Kexec => "kexec",
      Self::Suspend => "suspend",
      Self::Hibernate => "hibernate",
      Self::SuspendThenHibernate => "suspend-then-hibernate",
      Self::Lock => "lock",
      Self::HandledBySystem => "handled-by-system",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerButtonBehavior {
  Ignore,
  Poweroff,
  Reboot,
  Halt,
  Suspend,
  Lock,
}

impl PowerButtonBehavior {
  pub const ALL: [Self; 6] = [
    Self::Poweroff,
    Self::Suspend,
    Self::Lock,
    Self::Ignore,
    Self::Reboot,
    Self::Halt,
  ];

  pub fn parse(value: &str) -> Option<Self> {
    Some(match value.trim().to_ascii_lowercase().as_str() {
      "ignore" => Self::Ignore,
      "poweroff" | "off" => Self::Poweroff,
      "reboot" => Self::Reboot,
      "halt" => Self::Halt,
      "suspend" => Self::Suspend,
      "lock" => Self::Lock,
      _ => return None,
    })
  }

  pub fn value(self) -> &'static str {
    match self {
      Self::Ignore => "ignore",
      Self::Poweroff => "poweroff",
      Self::Reboot => "reboot",
      Self::Halt => "halt",
      Self::Suspend => "suspend",
      Self::Lock => "lock",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LidContext {
  Battery,
  Ac,
}

impl LidContext {
  pub const ALL: [Self; 2] = [Self::Battery, Self::Ac];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerState {
  /// HandleLidSwitch (battery) and HandleLidSwitchExternalPower (AC).
  pub lid: [PowerBehavior; 2],
  pub power_button: PowerButtonBehavior,
  /// Screen-off idle timeout in minutes from the canonical power module,
  /// with the derived hypridle file as a compatibility fallback.
  pub screen_off_minutes: Option<u32>,
  /// Screen-lock idle timeout in minutes from the canonical power module,
  /// with the derived hypridle file as a compatibility fallback.
  pub lock_minutes: Option<u32>,
  pub keep_awake: bool,
  pub can_suspend: bool,
  pub can_hibernate: bool,
  pub screen_off_supported: bool,
  pub lock_supported: bool,
  pub is_laptop: bool,
}

impl PowerState {
  pub fn lid_for(&self, context: LidContext) -> PowerBehavior {
    match context {
      LidContext::Battery => self.lid[0],
      LidContext::Ac => self.lid[1],
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn behaviors_parse_and_serialize_like_logind() {
    for behavior in PowerBehavior::ALL {
      assert_eq!(PowerBehavior::parse(behavior.value()), Some(behavior));
    }
    assert_eq!(PowerBehavior::parse("off"), Some(PowerBehavior::Poweroff));
    assert!(PowerBehavior::parse("explode").is_none());
    assert!(PowerButtonBehavior::parse("").is_none());
    for behavior in PowerButtonBehavior::ALL {
      assert_eq!(PowerButtonBehavior::parse(behavior.value()), Some(behavior));
    }
  }

  #[test]
  fn state_reads_per_context_lid() {
    let state = PowerState {
      lid: [PowerBehavior::Lock, PowerBehavior::Ignore],
      power_button: PowerButtonBehavior::Poweroff,
      screen_off_minutes: Some(15),
      lock_minutes: None,
      can_suspend: true,
      can_hibernate: false,
      screen_off_supported: true,
      lock_supported: true,
      keep_awake: false,
      is_laptop: true,
    };
    assert_eq!(state.lid_for(LidContext::Battery), PowerBehavior::Lock);
    assert_eq!(state.lid_for(LidContext::Ac), PowerBehavior::Ignore);
  }
}
