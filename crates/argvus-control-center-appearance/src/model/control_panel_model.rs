//! Control panel cards: the known cards and which ones are available and enabled.

/// Identifies a configurable Control Panel card using its stable helper ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlPanelCard {
  User,
  Notifications,
  Calendar,
  Weather,
  Volume,
  Brightness,
  Network,
  Bluetooth,
  System,
  Appearance,
  Session,
  Display,
  Power,
}

impl ControlPanelCard {
  pub const ALL: [Self; 13] = [
    Self::User,
    Self::Notifications,
    Self::Calendar,
    Self::Weather,
    Self::Volume,
    Self::Brightness,
    Self::Network,
    Self::Bluetooth,
    Self::System,
    Self::Appearance,
    Self::Session,
    Self::Display,
    Self::Power,
  ];

  /// Returns the machine-readable ID owned by the Control Panel helper.
  pub fn key(self) -> &'static str {
    match self {
      Self::User => "user",
      Self::Notifications => "notifications",
      Self::Calendar => "calendar",
      Self::Weather => "weather",
      Self::Volume => "volume",
      Self::Brightness => "brightness",
      Self::Network => "network",
      Self::Bluetooth => "bluetooth",
      Self::System => "system",
      Self::Appearance => "appearance",
      Self::Session => "session",
      Self::Display => "display",
      Self::Power => "power",
    }
  }

  /// Returns the shared catalog key for the Control Center row.
  pub fn label_key(self) -> &'static str {
    match self {
      Self::User => "control_center.control_panel_card_user",
      Self::Notifications => "control_center.control_panel_card_notifications",
      Self::Calendar => "control_center.control_panel_card_calendar",
      Self::Weather => "control_center.control_panel_card_weather",
      Self::Volume => "control_center.control_panel_card_volume",
      Self::Brightness => "control_center.control_panel_card_brightness",
      Self::Network => "control_center.control_panel_card_network",
      Self::Bluetooth => "control_center.control_panel_card_bluetooth",
      Self::System => "control_center.control_panel_card_system",
      Self::Appearance => "control_center.control_panel_card_appearance",
      Self::Session => "control_center.control_panel_card_session",
      Self::Display => "control_center.control_panel_card_display",
      Self::Power => "control_center.control_panel_card_power",
    }
  }
}

/// Stores effective Control Panel visibility while its package owns ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPanelCards {
  pub(super) enabled: [bool; 14],
  pub(super) available: [bool; 14],
}

impl Default for ControlPanelCards {
  fn default() -> Self {
    Self {
      enabled: [true; 14],
      available: [true; 14],
    }
  }
}

impl ControlPanelCards {
  pub fn enabled(&self, card: ControlPanelCard) -> bool {
    self.enabled[ControlPanelCard::ALL
      .iter()
      .position(|candidate| *candidate == card)
      .expect("all Control Panel cards have a stable index")]
  }

  pub fn set(&mut self, card: ControlPanelCard, enabled: bool) {
    let index = ControlPanelCard::ALL
      .iter()
      .position(|candidate| *candidate == card)
      .expect("all Control Panel cards have a stable index");
    self.enabled[index] = enabled;
  }

  /// Reports whether the card's required hardware is present on this host.
  pub fn available(&self, card: ControlPanelCard) -> bool {
    self.available[ControlPanelCard::ALL
      .iter()
      .position(|candidate| *candidate == card)
      .expect("all Control Panel cards have a stable index")]
  }

  /// Updates hardware availability while preserving the user's enabled state.
  pub fn set_available(&mut self, card: ControlPanelCard, available: bool) {
    let index = ControlPanelCard::ALL
      .iter()
      .position(|candidate| *candidate == card)
      .expect("all Control Panel cards have a stable index");
    self.available[index] = available;
  }
}
