//! Control panel cards: which ones are available and enabled, and their order.

use super::commands::{
  control_panel_cards_script, run_argvus_route_output, run_script, run_script_output, script,
};
use super::*;

pub(super) fn control_panel_enabled() -> bool {
  run_script_output(&control_panel_cards_script(), &["master", "status"])
    .is_none_or(|value| value.trim() != "disabled")
}

/// Reads the Control Panel helper status: the enabled flags and the order,
/// which come from the same call. The hardware availability is not probed
/// here: it is carried over from `previous` and refreshed separately by
/// [`control_panel_availability`], so the page never waits for a hardware probe.
pub(super) fn control_panel_cards(
  previous: &ControlPanelCards,
) -> (ControlPanelCards, Vec<ControlPanelCard>) {
  let status = run_script_output(&control_panel_cards_script(), &["status"]);
  control_panel_cards_from_status(status.as_deref(), previous)
}

/// Decodes the helper status and keeps package defaults when an older
/// installation does not provide the helper or returns malformed data. The
/// availability of every card is the one `previous` already knew.
pub(super) fn control_panel_cards_from_status(
  status: Option<&str>,
  previous: &ControlPanelCards,
) -> (ControlPanelCards, Vec<ControlPanelCard>) {
  let mut cards = status.map(parse_control_panel_cards).unwrap_or_default();
  for card in ControlPanelCard::ALL {
    cards.set_available(card, previous.available(card));
  }
  let order = status
    .map(parse_control_panel_order)
    .unwrap_or_else(|| ControlPanelCard::ALL.to_vec());
  (cards, order)
}

/// Probes the hardware the cards depend on, with the same checks the cards
/// run themselves. A disabled card stays disabled when its hardware is
/// temporarily disconnected and becomes available again later.
///
/// The probes invoke external tools with timeouts (bluetoothctl may wait up to
/// seconds for BlueZ), so they run concurrently, and the caller runs this in a
/// background job of its own instead of the page load.
pub fn control_panel_availability() -> Vec<(ControlPanelCard, bool)> {
  let (bluetooth_available, brightness_available) = std::thread::scope(|scope| {
    let bluetooth = scope.spawn(|| {
      run_argvus_route_output("bluetoothctl", &["status"])
        .is_some_and(|status| status.lines().any(|line| line.trim() == "available=yes"))
    });
    let brightness = scope.spawn(|| {
      run_script_output(&script("brightness-switch.sh"), &["--status"])
        .is_some_and(|backend| matches!(backend.trim(), "brightnessctl" | "ddcutil"))
    });
    (
      bluetooth.join().unwrap_or(false),
      brightness.join().unwrap_or(false),
    )
  });
  vec![
    (ControlPanelCard::Bluetooth, bluetooth_available),
    (ControlPanelCard::Brightness, brightness_available),
  ]
}

/// Maps the Control Panel helper's stable card ID back to its enum variant,
/// shared by the enabled-state and display-order readers.
pub(super) fn control_panel_card_from_key(key: &str) -> Option<ControlPanelCard> {
  Some(match key {
    "user" => ControlPanelCard::User,
    "notifications" => ControlPanelCard::Notifications,
    "calendar" => ControlPanelCard::Calendar,
    "weather" => ControlPanelCard::Weather,
    "volume" => ControlPanelCard::Volume,
    "brightness" => ControlPanelCard::Brightness,
    "network" => ControlPanelCard::Network,
    "bluetooth" => ControlPanelCard::Bluetooth,
    "system" => ControlPanelCard::System,
    "appearance" => ControlPanelCard::Appearance,
    "session" => ControlPanelCard::Session,
    "display" => ControlPanelCard::Display,
    "power" => ControlPanelCard::Power,
    _ => return None,
  })
}

/// Parses helper JSON independently from process execution for deterministic
/// coverage of compatibility and malformed-status fallbacks.
pub(super) fn parse_control_panel_cards(output: &str) -> ControlPanelCards {
  let mut cards = ControlPanelCards::default();
  let Ok(value) = serde_json::from_str::<Value>(output) else {
    return cards;
  };
  let Some(entries) = value.get("cards").and_then(Value::as_array) else {
    return cards;
  };
  for entry in entries {
    let Some(key) = entry.get("id").and_then(Value::as_str) else {
      continue;
    };
    let Some(enabled) = entry.get("enabled").and_then(Value::as_bool) else {
      continue;
    };
    let Some(card) = control_panel_card_from_key(key) else {
      continue;
    };
    cards.set(card, enabled);
  }
  cards
}

/// Parses the card order from the same helper JSON (the `cards` array is
/// already emitted in persisted order), appending any card missing from it
/// — an older helper, before this preference existed — at the end.
pub(super) fn parse_control_panel_order(output: &str) -> Vec<ControlPanelCard> {
  let mut order = Vec::new();
  if let Ok(value) = serde_json::from_str::<Value>(output)
    && let Some(entries) = value.get("cards").and_then(Value::as_array)
  {
    for entry in entries {
      let Some(key) = entry.get("id").and_then(Value::as_str) else {
        continue;
      };
      if let Some(card) = control_panel_card_from_key(key)
        && !order.contains(&card)
      {
        order.push(card);
      }
    }
  }
  for card in ControlPanelCard::ALL {
    if !order.contains(&card) {
      order.push(card);
    }
  }
  order
}

/// The Control Panel master switch and its cards.
pub(super) struct ControlPanelReading {
  pub(super) cards: ControlPanelCards,
  pub(super) order: Vec<ControlPanelCard>,
  pub(super) enabled: bool,
}

pub(super) fn read_control_panel(previous_cards: &ControlPanelCards) -> ControlPanelReading {
  std::thread::scope(|scope| {
    let enabled = scope.spawn(control_panel_enabled);
    let (cards, order) = control_panel_cards(previous_cards);
    ControlPanelReading {
      cards,
      order,
      enabled: enabled.join().unwrap_or(true),
    }
  })
}

pub fn set_control_panel_enabled(enabled: bool) -> Result<(), String> {
  run_script(
    &control_panel_cards_script(),
    &[
      "master",
      "set",
      if enabled { "enabled" } else { "disabled" },
    ],
  )
}

/// Persists a complete batch of Control Panel card switches. It does not
/// restart the panel: the `Apply` restarts it once after every write.
pub fn set_control_panel_cards(changes: &[(ControlPanelCard, bool)]) -> Result<(), String> {
  for &(card, enabled) in changes {
    run_script(
      &control_panel_cards_script(),
      &[
        "set",
        card.key(),
        if enabled { "enabled" } else { "disabled" },
      ],
    )?;
  }
  Ok(())
}

/// Moves a Control Panel card to an absolute position in its package-owned
/// order (0-based, the same index space as `CARD_IDS` in `cards-config.sh`),
/// the same primitive the real panel's drag-and-drop already calls.
pub fn move_control_panel_card(card: ControlPanelCard, index: usize) -> Result<(), String> {
  let index = index.to_string();
  run_script(&control_panel_cards_script(), &["move", card.key(), &index])?;
  reload_control_panel()
}

/// Restarts only the Control Panel consumer instead of reapplying every
/// mutable session component. This keeps rapid card toggles responsive while
/// preserving the required immediate panel reload.
pub fn reload_control_panel() -> Result<(), String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("argvus")
        .arg("sessionctl")
        .arg("restart")
        .arg("control-panel"),
    )
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr))
      .trim()
      .to_string();
    return Err(if stderr.is_empty() {
      "argvus-sessionctl restart control-panel falhou".into()
    } else {
      stderr
    });
  }
  Ok(())
}
