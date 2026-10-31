//! System time and timezone configuration of the Settings pages.
use std::collections::BTreeSet;
use std::fs;
use std::process::Stdio;

use crate::error::SettingsError;

#[derive(Debug, Clone, Default)]
pub struct DateTimeInfo {
  pub local_time: String,
  pub system_time_zone: String,
  pub time_zone: String,
  pub ntp: Option<bool>,
  pub rtc_local: Option<bool>,
}

fn timezone_path() -> Option<std::path::PathBuf> {
  argvus_i18n::config_path().and_then(|path| {
    path
      .parent()
      .map(|parent| parent.join("environment.d/20-timezone.conf"))
  })
}

fn parse_timezone(contents: &str) -> Option<String> {
  contents
    .lines()
    .filter_map(|line| line.trim().strip_prefix("TZ="))
    .next_back()
    .map(|value| value.trim_matches(['\"', '\'']).to_string())
    .filter(|value| is_valid_timezone_name(value))
}

pub fn current_timezone() -> String {
  if let Some(zone) = timezone_path()
    .and_then(|path| fs::read_to_string(path).ok())
    .and_then(|contents| parse_timezone(&contents))
  {
    return zone;
  }
  if let Ok(zone) = std::env::var("TZ")
    && is_valid_timezone_name(&zone)
  {
    return zone;
  }

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
        "Timezone" => info.system_time_zone = value.to_string(),
        "LocalRTC" => info.rtc_local = Some(value == "yes"),
        "NTP" => info.ntp = Some(value == "yes"),
        _ => {}
      }
    }
  }
  info.local_time = argvus_control_center_core::process::command("date")
    .arg("+%F %T")
    .env_remove("TZ")
    .output()
    .ok()
    .filter(|output| output.status.success())
    .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
    .unwrap_or_default()
    .trim()
    .to_string();
  info
}

pub fn set_timezone(zone: &str, valid: &[String]) -> Result<(), SettingsError> {
  if !is_valid_timezone_name(zone) || !valid.iter().any(|candidate| candidate == zone) {
    return Err(SettingsError::System(format!("invalid time zone: {zone}")));
  }
  let path = timezone_path()
    .ok_or_else(|| SettingsError::System("cannot locate user timezone configuration".into()))?;
  save_timezone(&path, zone).map_err(|error| SettingsError::System(error.to_string()))
}

fn save_timezone(path: &std::path::Path, zone: &str) -> std::io::Result<()> {
  use std::io::Write as _;
  use std::os::unix::fs::OpenOptionsExt as _;
  let previous = match fs::read_to_string(path) {
    Ok(contents) => contents,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
    Err(error) => return Err(error),
  };
  let parent = path
    .parent()
    .ok_or_else(|| std::io::Error::other("missing timezone parent"))?;
  fs::create_dir_all(parent)?;
  let temporary = parent.join(format!(".timezone.{}.tmp", std::process::id()));
  let mut file = fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .mode(0o600)
    .open(&temporary)?;
  let result = (|| {
    for line in previous
      .lines()
      .filter(|line| !line.trim_start().starts_with("TZ="))
    {
      writeln!(file, "{line}")?;
    }
    writeln!(file, "TZ={zone}")?;
    file.sync_all()?;
    fs::rename(&temporary, path)
  })();
  if result.is_err() {
    let _ = fs::remove_file(&temporary);
  }
  result
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
  fn personal_timezone_preserves_other_entries_and_profiles() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first/20-timezone.conf");
    let second = directory.path().join("second/20-timezone.conf");
    save_timezone(&first, "UTC").unwrap();
    save_timezone(&second, "UTC").unwrap();
    fs::write(&first, "# personal\nTZ=UTC\nOTHER=keep\n").unwrap();
    save_timezone(&first, "America/Sao_Paulo").unwrap();
    let contents = fs::read_to_string(&first).unwrap();
    assert_eq!(
      parse_timezone(&contents).as_deref(),
      Some("America/Sao_Paulo")
    );
    assert!(contents.contains("OTHER=keep"));
    assert_eq!(fs::read_to_string(second).unwrap(), "TZ=UTC\n");
    assert!(parse_timezone("TZ=../etc/passwd").is_none());
  }

  #[test]
  fn validates_timezone_names() {
    assert!(is_valid_timezone_name("America/Sao_Paulo"));
    assert!(is_valid_timezone_name("Etc/GMT+3"));
    assert!(!is_valid_timezone_name("../UTC"));
    assert!(!is_valid_timezone_name("/UTC"));
    assert!(!is_valid_timezone_name("UTC;reboot"));
  }
}
