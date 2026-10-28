//! System time and timezone configuration of the Settings pages.
use std::collections::BTreeSet;
use std::fs;
use std::process::Stdio;

use crate::error::SettingsError;

#[derive(Debug, Clone, Default)]
pub struct DateTimeInfo {
  pub local_time: String,
  pub time_zone: String,
  pub ntp: Option<bool>,
  pub rtc_local: Option<bool>,
}

pub fn current_timezone() -> String {
  command_stdout("timedatectl", &["show", "-p", "Timezone", "--value"])
    .filter(|value| !value.trim().is_empty())
    .unwrap_or_else(|| {
      fs::read_link("/etc/localtime")
        .ok()
        .and_then(|path| {
          path
            .to_string_lossy()
            .split("/zoneinfo/")
            .nth(1)
            .map(str::to_string)
        })
        .unwrap_or_default()
    })
    .trim()
    .to_string()
}

pub fn list_timezones() -> Vec<String> {
  command_stdout("timedatectl", &["list-timezones"])
    .unwrap_or_default()
    .lines()
    .map(str::trim)
    .filter(|line| is_valid_timezone_name(line))
    .map(str::to_string)
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect()
}

pub fn datetime_info() -> DateTimeInfo {
  let mut info = DateTimeInfo {
    time_zone: current_timezone(),
    ..Default::default()
  };
  if let Some(output) = command_stdout("timedatectl", &["show"]) {
    for line in output.lines() {
      let Some((key, value)) = line.split_once('=') else {
        continue;
      };
      match key {
        "Timezone" => info.time_zone = value.to_string(),
        "LocalRTC" => info.rtc_local = Some(value == "yes"),
        "NTP" => info.ntp = Some(value == "yes"),
        _ => {}
      }
    }
  }
  info.local_time = command_stdout("date", &["+%F %T"])
    .unwrap_or_default()
    .trim()
    .to_string();
  info
}

pub fn set_timezone(zone: &str, valid: &[String]) -> Result<(), SettingsError> {
  if !valid.iter().any(|candidate| candidate == zone) {
    return Err(SettingsError::System(format!("invalid time zone: {zone}")));
  }
  super::privileged::run(&["timezone", "set", zone]).map(|_| ())
}

pub fn set_ntp(enabled: bool) -> Result<(), SettingsError> {
  super::privileged::run(&["datetime", "ntp", if enabled { "true" } else { "false" }]).map(|_| ())
}

pub fn set_local_time(value: &str) -> Result<(), SettingsError> {
  if !is_valid_datetime(value) {
    return Err(SettingsError::System("invalid date/time".into()));
  }
  super::privileged::run(&["datetime", "set", value]).map(|_| ())
}

pub fn is_valid_datetime(value: &str) -> bool {
  value.len() == 19
    && value.as_bytes()[4] == b'-'
    && value.as_bytes()[7] == b'-'
    && value.as_bytes()[10] == b' '
    && value.as_bytes()[13] == b':'
    && value.as_bytes()[16] == b':'
    && value
      .bytes()
      .enumerate()
      .all(|(index, byte)| matches!(index, 4 | 7 | 10 | 13 | 16) || byte.is_ascii_digit())
}

pub fn is_valid_timezone_name(value: &str) -> bool {
  !value.is_empty()
    && !value.starts_with('/')
    && !value.contains("..")
    && value
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'_' | b'-' | b'+'))
}

fn command_stdout(command: &str, args: &[&str]) -> Option<String> {
  let output = argvus_control_center_core::process::command(command)
    .args(args)
    .stdin(Stdio::null())
    .output()
    .ok()?;
  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validates_timezone_names() {
    assert!(is_valid_timezone_name("America/Sao_Paulo"));
    assert!(is_valid_timezone_name("Etc/GMT+3"));
    assert!(!is_valid_timezone_name("../UTC"));
    assert!(!is_valid_timezone_name("/UTC"));
    assert!(!is_valid_timezone_name("UTC;reboot"));
  }
}
