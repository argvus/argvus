//! Reads the active monitors of the running Hyprland session.
use argvus_control_center_core::process;
use argvus_control_center_core::sanitize::terminal_text;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
/// One active monitor, as Hyprland drives it.
pub struct Display {
  /// Connector name, such as `DP-1`, `HDMI-A-1` or `eDP-1`.
  pub name: String,
  pub width: u64,
  pub height: u64,
  pub refresh_hz: f64,
  pub scale: f64,
}

impl Display {
  /// Formats `DP-1  2560x1440 @ 143.91 Hz  ×1.25`; the scale is left out at 1.
  pub fn label(&self) -> String {
    let mut label = format!(
      "{}  {}x{} @ {} Hz",
      self.name,
      self.width,
      self.height,
      trimmed_decimal(self.refresh_hz)
    );
    if (self.scale - 1.0).abs() > f64::EPSILON {
      label.push_str(&format!("  ×{}", trimmed_decimal(self.scale)));
    }
    label
  }
}

/// Lists the monitors through `hyprctl monitors -j`; `None` outside a Hyprland session.
pub fn displays() -> Option<Vec<Display>> {
  let output = process::command("hyprctl")
    .args(["monitors", "-j"])
    .output()
    .ok()
    .filter(|output| output.status.success())?;
  parse_monitors(&String::from_utf8_lossy(&output.stdout))
}

/// Parses the JSON array printed by `hyprctl monitors -j`, skipping incomplete entries.
pub fn parse_monitors(json: &str) -> Option<Vec<Display>> {
  let monitors = serde_json::from_str::<Value>(json).ok()?;
  let displays = monitors
    .as_array()?
    .iter()
    .filter_map(|monitor| {
      Some(Display {
        name: terminal_text(monitor.get("name")?.as_str()?),
        width: monitor.get("width")?.as_u64()?,
        height: monitor.get("height")?.as_u64()?,
        refresh_hz: monitor.get("refreshRate")?.as_f64()?,
        scale: monitor.get("scale").and_then(Value::as_f64).unwrap_or(1.0),
      })
    })
    .collect::<Vec<_>>();
  (!displays.is_empty()).then_some(displays)
}

/// Prints a number with up to two decimals and no trailing zeros (`60`, `143.91`, `1.5`).
fn trimmed_decimal(value: f64) -> String {
  let text = format!("{value:.2}");
  text.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  const TWO_MONITORS: &str = r#"[
    {"id": 0, "name": "eDP-1", "width": 1920, "height": 1200, "refreshRate": 60.00100, "scale": 1.00},
    {"id": 1, "name": "DP-2", "width": 2560, "height": 1440, "refreshRate": 143.91200, "scale": 1.25}
  ]"#;

  #[test]
  fn parses_every_active_monitor() {
    let displays = parse_monitors(TWO_MONITORS).expect("monitors");
    let labels = displays.iter().map(Display::label).collect::<Vec<_>>();
    assert_eq!(
      labels,
      [
        "eDP-1  1920x1200 @ 60 Hz",
        "DP-2  2560x1440 @ 143.91 Hz  ×1.25"
      ]
    );
  }

  #[test]
  fn returns_none_without_monitors_or_valid_json() {
    assert_eq!(parse_monitors("[]"), None);
    assert_eq!(parse_monitors("Couldn't connect to socket"), None);
  }

  #[test]
  fn skips_incomplete_monitor_entries() {
    let json = r#"[{"name": "DP-1"}, {"name": "HDMI-A-1", "width": 1920, "height": 1080, "refreshRate": 75.0}]"#;
    let displays = parse_monitors(json).expect("monitors");
    assert_eq!(displays.len(), 1);
    assert_eq!(displays[0].label(), "HDMI-A-1  1920x1080 @ 75 Hz");
  }
}
