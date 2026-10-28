//! Input-device configuration of the Settings pages.
use std::fs;
use std::path::PathBuf;

use super::ratbag;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct InputSettings {
  pub mouse: MouseSettings,
  pub touchpad: TouchpadSettings,
  #[serde(skip)]
  pub devices: Devices,
  #[serde(skip)]
  pub ratbag: Vec<ratbag::Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MouseSettings {
  pub sensitivity: f64,
  pub accel_profile: String,
  pub natural_scroll: bool,
  pub scroll_factor: f64,
  pub left_handed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TouchpadSettings {
  pub natural_scroll: bool,
  pub tap_to_click: bool,
  pub tap_and_drag: bool,
  pub two_finger_right_click: bool,
  pub disable_while_typing: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Devices {
  pub mouse: bool,
  pub touchpad: bool,
}

impl Default for MouseSettings {
  fn default() -> Self {
    Self {
      sensitivity: 0.0,
      accel_profile: "adaptive".into(),
      natural_scroll: false,
      scroll_factor: 1.0,
      left_handed: false,
    }
  }
}
impl Default for TouchpadSettings {
  fn default() -> Self {
    Self {
      natural_scroll: true,
      tap_to_click: true,
      tap_and_drag: true,
      two_finger_right_click: true,
      disable_while_typing: true,
    }
  }
}
pub fn path() -> PathBuf {
  argvus_control_center_core::paths::argvus_data_home().join("hypr/input.toml")
}

/// Runs `argvus config set` for `pointer` with `value`. Failures are tolerated
/// so the compositor-side apply already performed by the caller still stands.
/// Reads a raw effective modular value for `pointer`. Returns `None` when
/// the tool is unavailable or the value is missing/empty, which forces the
/// caller to fall back to the local replica.
fn read_config(pointer: &str) -> Option<String> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", pointer, "--effective", "--raw"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
  if value.is_empty() || value == "null" {
    None
  } else {
    Some(value)
  }
}

/// Loads the input settings from the modular canonical store. When the
/// tool is unavailable no value is returned and the caller uses the replica.
fn input_from_config() -> Option<InputSettings> {
  let mut settings = InputSettings::default();
  let mut observed = 0usize;

  if let Some(value) = read_config("/hyprland/input/mouse/sensitivity")
    && let Ok(parsed) = value.parse::<f64>()
  {
    settings.mouse.sensitivity = parsed;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/mouse/accel_profile") {
    settings.mouse.accel_profile = value;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/mouse/natural_scroll")
    && let Ok(parsed) = value.parse::<bool>()
  {
    settings.mouse.natural_scroll = parsed;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/mouse/scroll_factor")
    && let Ok(parsed) = value.parse::<f64>()
  {
    settings.mouse.scroll_factor = parsed;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/mouse/left_handed")
    && let Ok(parsed) = value.parse::<bool>()
  {
    settings.mouse.left_handed = parsed;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/touchpad/natural_scroll")
    && let Ok(parsed) = value.parse::<bool>()
  {
    settings.touchpad.natural_scroll = parsed;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/touchpad/tap_to_click")
    && let Ok(parsed) = value.parse::<bool>()
  {
    settings.touchpad.tap_to_click = parsed;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/touchpad/tap_and_drag")
    && let Ok(parsed) = value.parse::<bool>()
  {
    settings.touchpad.tap_and_drag = parsed;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/touchpad/two_finger_right_click")
    && let Ok(parsed) = value.parse::<bool>()
  {
    settings.touchpad.two_finger_right_click = parsed;
    observed += 1;
  }
  if let Some(value) = read_config("/hyprland/input/touchpad/disable_while_typing")
    && let Ok(parsed) = value.parse::<bool>()
  {
    settings.touchpad.disable_while_typing = parsed;
    observed += 1;
  }

  (observed > 0).then_some(settings)
}

/// Persists the current input settings into the modular canonical store
/// before the derived replicas are updated. A missing argvus-config tool is
/// tolerated; the live `hyprctl` apply performed by the caller still stands.
fn sync_to_config(settings: &InputSettings) -> Result<(), String> {
  let patch = serde_json::json!({
    "/hyprland/input/mouse": settings.mouse,
    "/hyprland/input/touchpad": settings.touchpad,
  });
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist input settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the input settings".into());
  }
  argvus_control_center_core::config::reload_argvus_config_service()
}

pub fn load() -> InputSettings {
  let mut settings = input_from_config()
    .or_else(load_from_replica)
    .unwrap_or_default();
  settings.devices = detect_devices();
  settings.ratbag = ratbag::devices();
  settings
}

/// Reads the persisted `input.toml` replica, falling back to live Hyprland
/// values when the replica has not been materialized yet.
fn load_from_replica() -> Option<InputSettings> {
  let persisted = fs::read_to_string(path());
  let mut settings: InputSettings = persisted
    .as_deref()
    .ok()
    .and_then(|v| toml::from_str(v).ok())
    .unwrap_or_default();
  if persisted.is_err() {
    settings.load_runtime_values();
  }
  Some(settings)
}

impl InputSettings {
  fn load_runtime_values(&mut self) {
    self.mouse.sensitivity = query_float("input:sensitivity").unwrap_or(self.mouse.sensitivity);
    self.mouse.scroll_factor =
      query_float("input:scroll_factor").unwrap_or(self.mouse.scroll_factor);
    self.mouse.natural_scroll =
      query_bool("input:natural_scroll").unwrap_or(self.mouse.natural_scroll);
    self.mouse.left_handed = query_bool("input:left_handed").unwrap_or(self.mouse.left_handed);
    self.mouse.accel_profile = query_string("input:accel_profile")
      .filter(|v| matches!(v.as_str(), "adaptive" | "flat"))
      .unwrap_or(self.mouse.accel_profile.clone());
    self.touchpad.natural_scroll =
      query_bool("input:touchpad:natural_scroll").unwrap_or(self.touchpad.natural_scroll);
    self.touchpad.tap_to_click =
      query_bool("input:touchpad:tap_to_click").unwrap_or(self.touchpad.tap_to_click);
    self.touchpad.tap_and_drag =
      query_bool("input:touchpad:tap_and_drag").unwrap_or(self.touchpad.tap_and_drag);
    self.touchpad.two_finger_right_click = query_bool("input:touchpad:clickfinger_behavior")
      .unwrap_or(self.touchpad.two_finger_right_click);
    self.touchpad.disable_while_typing = query_bool("input:touchpad:disable_while_typing")
      .unwrap_or(self.touchpad.disable_while_typing);
  }
}

fn save(settings: &InputSettings) -> Result<(), String> {
  sync_to_config(settings)
}

impl InputSettings {
  pub fn toggle(&mut self, index: usize) -> Result<(), String> {
    let (key, value) = match index {
      3 => ("input:natural_scroll", !self.mouse.natural_scroll),
      5 => ("input:left_handed", !self.mouse.left_handed),
      7 => (
        "input:touchpad:natural_scroll",
        !self.touchpad.natural_scroll,
      ),
      8 => ("input:touchpad:tap_to_click", !self.touchpad.tap_to_click),
      9 => ("input:touchpad:tap_and_drag", !self.touchpad.tap_and_drag),
      10 => (
        "input:touchpad:clickfinger_behavior",
        !self.touchpad.two_finger_right_click,
      ),
      11 => (
        "input:touchpad:disable_while_typing",
        !self.touchpad.disable_while_typing,
      ),
      _ => return Ok(()),
    };
    self.set_bool(key, value)?;
    match index {
      3 => self.mouse.natural_scroll = value,
      5 => self.mouse.left_handed = value,
      7 => self.touchpad.natural_scroll = value,
      8 => self.touchpad.tap_to_click = value,
      9 => self.touchpad.tap_and_drag = value,
      10 => self.touchpad.two_finger_right_click = value,
      11 => self.touchpad.disable_while_typing = value,
      _ => {}
    }
    save(self)
  }

  pub fn cycle(&mut self, index: usize, direction: i8) -> Result<(), String> {
    self.cycle_by(index, direction)
  }

  /// Apply an accumulated slider movement in one compositor/config write.
  /// This keeps keyboard auto-repeat and slider dragging from spawning one
  /// `hyprctl` process per intermediate step.
  pub fn cycle_by(&mut self, index: usize, direction: i8) -> Result<(), String> {
    match index {
      1 => {
        self.mouse.sensitivity =
          validate_sensitivity(self.mouse.sensitivity + f64::from(direction) * 0.1)?;
        apply("input:sensitivity", &self.mouse.sensitivity.to_string())?;
      }
      2 => {
        self.mouse.accel_profile = if self.mouse.accel_profile == "adaptive" {
          "flat"
        } else {
          "adaptive"
        }
        .into();
        apply("input:accel_profile", &self.mouse.accel_profile)?;
      }
      4 => {
        self.mouse.scroll_factor =
          validate_scroll_factor(self.mouse.scroll_factor + f64::from(direction) * 0.1)?;
        apply("input:scroll_factor", &self.mouse.scroll_factor.to_string())?;
      }
      _ => return Ok(()),
    }
    save(self)
  }

  fn set_bool(&self, key: &str, value: bool) -> Result<(), String> {
    apply(key, if value { "true" } else { "false" })
  }
}

pub fn validate_sensitivity(value: f64) -> Result<f64, String> {
  if (-1.0..=1.0).contains(&value) {
    Ok((value * 10.0).round() / 10.0)
  } else {
    Err("sensitivity must be between -1.0 and 1.0".into())
  }
}
pub fn validate_scroll_factor(value: f64) -> Result<f64, String> {
  if (0.1..=10.0).contains(&value) {
    Ok((value * 10.0).round() / 10.0)
  } else {
    Err("scroll factor must be between 0.1 and 10.0".into())
  }
}
pub fn speed_display(value: f64) -> String {
  format!("{value:+.1}  [{}]", slider_bar(value, -1.0, 1.0))
}
pub fn factor_display(value: f64) -> String {
  format!("{value:.1}  [{}]", slider_bar(value, 0.1, 10.0))
}
fn slider_bar(value: f64, minimum: f64, maximum: f64) -> String {
  const SLOTS: usize = 15;
  let position = (((value - minimum) / (maximum - minimum)) * (SLOTS - 1) as f64)
    .round()
    .clamp(0.0, (SLOTS - 1) as f64) as usize;
  (0..SLOTS)
    .map(|index| if index == position { '●' } else { '─' })
    .collect()
}
pub fn accel_label(value: &str) -> &str {
  match value {
    "flat" => "control_center.input_acceleration_flat",
    _ => "control_center.input_acceleration_adaptive",
  }
}

fn apply(key: &str, value: &str) -> Result<(), String> {
  let output = argvus_control_center_core::process::command("hyprctl")
    .args(["keyword", key, value])
    .output()
    .map_err(|e| e.to_string())?;
  if output.status.success() {
    Ok(())
  } else {
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
  }
}

pub fn detect_devices() -> Devices {
  let output = argvus_control_center_core::process::command("hyprctl")
    .args(["devices", "-j"])
    .output()
    .ok();
  let Some(output) = output else {
    return Devices::default();
  };
  let Ok(root) = serde_json::from_slice::<Value>(&output.stdout) else {
    return Devices::default();
  };
  devices_from_json(&root)
}

fn devices_from_json(root: &Value) -> Devices {
  let mice = root
    .get("mice")
    .and_then(Value::as_array)
    .is_some_and(|v| !v.is_empty());
  let touchpad = root
    .get("touch")
    .or_else(|| root.get("touchpads"))
    .and_then(Value::as_array)
    .is_some_and(|v| !v.is_empty());
  Devices {
    mouse: mice,
    touchpad,
  }
}

fn query_value(key: &str) -> Option<Value> {
  let output = argvus_control_center_core::process::command("hyprctl")
    .args(["getoption", key, "-j"])
    .output()
    .ok()?;
  serde_json::from_slice(&output.stdout).ok()
}
fn query_float(key: &str) -> Option<f64> {
  query_value(key)?.get("float").and_then(Value::as_f64)
}
fn query_bool(key: &str) -> Option<bool> {
  query_value(key)?
    .get("int")
    .and_then(Value::as_i64)
    .map(|v| v != 0)
    .or_else(|| query_value(key)?.get("bool").and_then(Value::as_bool))
}
fn query_string(key: &str) -> Option<String> {
  query_value(key)?
    .get("str")
    .and_then(Value::as_str)
    .map(str::to_string)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn validates_ranges() {
    assert!(validate_sensitivity(-1.0).is_ok());
    assert!(validate_sensitivity(1.1).is_err());
    assert!(validate_scroll_factor(0.1).is_ok());
    assert!(validate_scroll_factor(10.1).is_err());
  }
  #[test]
  fn normalizes_profile() {
    assert_eq!(
      accel_label("flat"),
      "control_center.input_acceleration_flat"
    );
    assert_eq!(
      accel_label("custom"),
      "control_center.input_acceleration_adaptive"
    );
  }

  #[test]
  fn detects_current_hyprland_touch_device_key() {
    let devices = devices_from_json(&serde_json::json!({
      "mice": [],
      "touch": [{"name": "test-touchpad"}]
    }));
    assert!(!devices.mouse);
    assert!(devices.touchpad);
  }

  #[test]
  fn accepts_legacy_touchpads_device_key() {
    let devices = devices_from_json(&serde_json::json!({
      "mice": [{"name": "test-mouse"}],
      "touchpads": [{"name": "test-touchpad"}]
    }));
    assert!(devices.mouse);
    assert!(devices.touchpad);
  }

  #[test]
  fn slider_display_tracks_normalized_value() {
    assert!(speed_display(-1.0).contains("●──────────────"));
    assert!(speed_display(1.0).contains("──────────────●"));
    assert!(factor_display(0.1).contains("●──────────────"));
    assert!(factor_display(10.0).contains("──────────────●"));
  }
}
