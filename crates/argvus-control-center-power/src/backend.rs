//! Isolated integration with system tools and APIs of the Power page.
use crate::{
  hypridle,
  model::{LidContext, PowerBehavior, PowerButtonBehavior, PowerState},
};
use argvus_control_center_core::{
  paths::{argvus_config_home, system_config_root},
  privileged::{PrivilegedOperation, PrivilegedRequest, SystemSettingsOperation},
  process::{ProcessOutput, ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use serde_json::Value;
use std::fs;

const LOGIND_MAIN: &str = "/etc/systemd/logind.conf";
const LOGIND_DROP_IN: &str = "/etc/systemd/logind.conf.d/argvus.conf";

pub fn load() -> PowerState {
  let lid_battery = canonical_value("/power/lid_battery")
    .or_else(|| logind_value("HandleLidSwitch"))
    .as_deref()
    .and_then(PowerBehavior::parse)
    .unwrap_or(PowerBehavior::Suspend);
  let lid_ac = canonical_value("/power/lid_ac")
    .or_else(|| logind_value("HandleLidSwitchExternalPower"))
    .as_deref()
    .and_then(PowerBehavior::parse)
    .unwrap_or(lid_battery);
  let power_button = canonical_value("/power/power_button")
    .or_else(|| logind_value("HandlePowerKey"))
    .as_deref()
    .and_then(PowerButtonBehavior::parse)
    .unwrap_or(PowerButtonBehavior::Poweroff);
  let screen_off_minutes = canonical_value("/power/screen_off_minutes")
    .and_then(|value| value.parse().ok())
    .or_else(hypridle::screen_off_minutes);
  // The modular power file is the source of truth.  A valid canonical value
  // (including zero, which means "never") is enough to render the setting;
  // the derived hypridle path must not hide it behind the compatibility text.
  let screen_off_supported = screen_off_minutes.is_some();
  let lock_minutes = canonical_value("/power/lock_minutes")
    .and_then(|value| value.parse().ok())
    .or_else(hypridle::lock_minutes);
  let lock_supported = lock_minutes.is_some();
  let keep_awake = canonical_value("/power/keep_awake")
    .and_then(|value| value.parse().ok())
    .unwrap_or_else(keep_awake_status);
  let can_suspend = systemctl_can("can-suspend");
  let can_hibernate = systemctl_can("can-hibernate");
  let is_laptop = detect_is_laptop();
  PowerState {
    lid: [lid_battery, lid_ac],
    power_button,
    screen_off_minutes,
    lock_minutes,
    can_suspend,
    can_hibernate,
    screen_off_supported,
    lock_supported,
    keep_awake,
    is_laptop,
  }
}

fn keep_awake_status() -> bool {
  SystemProcessRunner
    .run(&ProcessRequest::new("/usr/share/argvus/power/sh/keep-awake.sh").arg("status"))
    .ok()
    .is_some_and(|output| String::from_utf8_lossy(&output.stdout).trim() == "enabled")
}

fn detect_is_laptop() -> bool {
  if let Ok(chassis) = fs::read_to_string("/sys/class/dmi/id/chassis_type")
    && let Ok(n) = chassis.trim().parse::<u8>()
    && matches!(n, 8..=14 | 30..=32)
  {
    return true;
  }
  fs::read_dir("/sys/class/power_supply")
    .ok()
    .and_then(|dir| {
      dir
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with("BAT"))
    })
    .is_some()
}

/// Reads a logind `Key = value`, honoring the ARGVUS drop-in first, then the
/// main config file, then the documented systemd defaults.
fn logind_value(key: &str) -> Option<String> {
  let drop_in = fs::read_to_string(LOGIND_DROP_IN).ok()?;
  if let Some(value) = parse_logind_key(&drop_in, key) {
    return Some(value);
  }
  let main = fs::read_to_string(LOGIND_MAIN).ok()?;
  parse_logind_key(&main, key)
}

fn parse_logind_key(content: &str, key: &str) -> Option<String> {
  content.lines().find_map(|line| {
    let line = line.trim();
    if line.starts_with('#') {
      return None;
    }
    let (name, value) = line.split_once('=')?;
    (name.trim() == key).then(|| value.trim().to_owned())
  })
}

fn systemctl_can(command: &str) -> bool {
  let output = SystemProcessRunner.run(&ProcessRequest::new("systemctl").arg(command));
  output
    .ok()
    .is_some_and(|out| matches!(String::from_utf8_lossy(&out.stdout).trim(), "yes"))
}

pub fn set_lid(context: LidContext, behavior: PowerBehavior) -> Result<(), String> {
  let (context, key, pointer) = match context {
    LidContext::Battery => ("battery", "HandleLidSwitch", "/power/lid_battery"),
    LidContext::Ac => ("ac", "HandleLidSwitchExternalPower", "/power/lid_ac"),
  };
  persist_canonical(pointer, behavior.value())?;
  privileged_power(
    "set-lid",
    vec![context.into(), behavior.value().into()],
    &format!("{key}={}", behavior.value()),
  )
}

pub fn set_power_button(behavior: PowerButtonBehavior) -> Result<(), String> {
  persist_canonical("/power/power_button", behavior.value())?;
  privileged_power(
    "set-power-button",
    vec![behavior.value().into()],
    &format!("HandlePowerKey={}", behavior.value()),
  )
}

pub fn apply_idle(minutes: u32) -> Result<(), String> {
  hypridle::apply_screen_off_minutes(minutes).map(|_| ())
}

pub fn apply_lock(minutes: u32) -> Result<(), String> {
  hypridle::apply_lock_minutes(minutes).map(|_| ())
}

pub fn set_keep_awake(enabled: bool) -> Result<(), String> {
  persist_canonical("/power/keep_awake", if enabled { "true" } else { "false" })?;
  let value = if enabled { "on" } else { "off" };
  let output = SystemProcessRunner
    .run(&ProcessRequest::new("/usr/share/argvus/power/sh/keep-awake.sh").arg(value))
    .map_err(|error| error.to_string())?;
  (output.status == Some(0))
    .then_some(())
    .ok_or_else(|| terminal_text(String::from_utf8_lossy(&output.stderr).trim()).to_string())
}

fn canonical_value(pointer: &str) -> Option<String> {
  let module_key = pointer.strip_prefix("/power/")?;
  modular_power_value(module_key).or_else(|| argvus_config_value(pointer))
}

/// Reads the modular power database directly. The Control Center must not
/// replace a valid value with the compatibility label merely because a
/// subprocess lookup is unavailable in a desktop launcher environment.
fn modular_power_value(key: &str) -> Option<String> {
  let paths = [
    argvus_config_home().join("config/power.json"),
    system_config_root().join("config/power.json"),
  ];
  paths.into_iter().find_map(|path| {
    let content = fs::read_to_string(path).ok()?;
    parse_modular_power_value(&content, key)
  })
}

fn parse_modular_power_value(content: &str, key: &str) -> Option<String> {
  let root = serde_json::from_str::<Value>(content).ok()?;
  let value = root.get(key)?;
  match value {
    Value::String(value) => Some(value.clone()),
    Value::Bool(value) => Some(value.to_string()),
    Value::Number(value) => Some(value.to_string()),
    _ => None,
  }
}

fn argvus_config_value(pointer: &str) -> Option<String> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", pointer, "--effective", "--raw"])
    .output()
    .ok()?;
  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    .filter(|value| value != "null" && !value.is_empty())
}

fn persist_canonical(pointer: &str, value: &str) -> Result<(), String> {
  let value = if matches!(value, "true" | "false") {
    value.to_owned()
  } else {
    serde_json::to_string(value).map_err(|error| error.to_string())?
  };
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["set", pointer, &value])
    .status()
    .map_err(|error| format!("failed to persist power setting: {error}"))?;
  status
    .success()
    .then_some(())
    .ok_or_else(|| String::from("argvus config rejected the power setting"))?;
  argvus_control_center_core::config::reload_argvus_config_service()
}

pub fn suspend_now() -> Result<(), String> {
  privileged_power("suspend-now", Vec::new(), "A operação de suspensão falhou.")
}

pub fn hibernate_now() -> Result<(), String> {
  privileged_power(
    "hibernate-now",
    Vec::new(),
    "A operação de hibernação falhou.",
  )
}

fn privileged_power(action: &str, arguments: Vec<String>, message: &str) -> Result<(), String> {
  let executable = std::env::current_exe()
    .map_err(|error| error.to_string())?
    .to_string_lossy()
    .into_owned();
  let operation = SystemSettingsOperation::new(SystemProcessRunner, executable);
  let request = PrivilegedRequest::new("power", action, arguments)?;
  let output = operation.execute(&request)?;
  if output.status == Some(0) {
    Ok(())
  } else {
    Err(privileged_failure(&output, message))
  }
}

fn privileged_failure(output: &ProcessOutput, message: &str) -> String {
  let stderr = terminal_text(String::from_utf8_lossy(&output.stderr).trim());
  match output.status {
    Some(126) => "Autorização cancelada.".into(),
    Some(127) => "Autorização negada.".into(),
    _ if stderr.contains("pkexec was not found") => "pkexec não encontrado; instale polkit.".into(),
    _ if stderr.is_empty() => message.into(),
    _ => stderr,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_logind_keys_ignoring_comments() {
    let content = "\
# HandleLidSwitch=poweroff
HandleLidSwitch=lock
HandlePowerKey=poweroff
";
    assert_eq!(
      parse_logind_key(content, "HandleLidSwitch"),
      Some("lock".into())
    );
    assert_eq!(
      parse_logind_key(content, "HandlePowerKey"),
      Some("poweroff".into())
    );
    assert_eq!(parse_logind_key(content, "HandleLidSwitchDocked"), None);
  }

  #[test]
  fn drop_in_precedence_over_main_config() {
    let field = "HandleLidSwitchExternalPower";
    let drop_in = format!("{field}=ignore");
    let main = format!("{field}=suspend");
    assert_eq!(parse_logind_key(&drop_in, field), Some("ignore".into()));
    let _ = main;
    assert!(parse_logind_key("", field).is_none());
  }

  #[test]
  fn modular_power_values_include_zero_and_numbers() {
    let content = r#"{
      "keep_awake": false,
      "lock_minutes": 30,
      "screen_off_minutes": 0
    }"#;
    assert_eq!(
      parse_modular_power_value(content, "screen_off_minutes"),
      Some("0".into())
    );
    assert_eq!(
      parse_modular_power_value(content, "lock_minutes"),
      Some("30".into())
    );
    assert_eq!(
      parse_modular_power_value(content, "keep_awake"),
      Some("false".into())
    );
    assert_eq!(parse_modular_power_value(content, "missing"), None);
  }
}
