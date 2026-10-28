//! Power operations: lid and power-key actions in the logind drop-in, and the CPU governor.

use super::boot::backup_file;
use super::streaming::locate_command;
use super::*;

/// Power actions are routed to systemd-logind. Lid/Power-key behaviors are
/// applied through an ARGVUS drop-in under `/etc/systemd/logind.conf.d` (the
/// upstream config file is never overwritten) and the running logind daemon is
/// asked to re-read its configuration with a SIGHUP.
pub(super) fn run_power(action: &str, values: &[String]) -> Result<(), String> {
  match action {
    "set-lid" => {
      let (context, value) = split_lid(values)?;
      let key = lid_key(context)?;
      update_logind_drop_in(&[(key, value.to_string())])?;
      reload_logind()
    }
    "set-power-button" => {
      let [value] = values else {
        return Err("power button needs one value".into());
      };
      if !POWER_KEY_VALUES.contains(&value.as_str()) {
        return Err("unsupported HandlePowerKey value".into());
      }
      update_logind_drop_in(&[("HandlePowerKey", value.to_owned())])?;
      reload_logind()
    }
    "suspend-now" => run_command("systemctl", &["suspend"]),
    "hibernate-now" => run_command("systemctl", &["hibernate"]),
    _ => Err("unsupported power operation".into()),
  }
}

pub(super) fn validate_power(action: &str, values: &[String]) -> Result<(), String> {
  match action {
    "set-lid" => {
      let (_, value) = split_lid(values)?;
      if valid_lid_value(value) {
        Ok(())
      } else {
        Err(format!("unsupported HandleLidSwitch value: {value}"))
      }
    }
    "set-power-button" => {
      if values.len() != 1 || !POWER_KEY_VALUES.contains(&values[0].as_str()) {
        Err("unsupported HandlePowerKey value".into())
      } else {
        Ok(())
      }
    }
    "suspend-now" | "hibernate-now" => {
      if values.is_empty() {
        Ok(())
      } else {
        Err("suspend-now and hibernate-now take no arguments".into())
      }
    }
    _ => Err("unsupported power operation".into()),
  }
}

pub(super) fn valid_lid_value(value: &str) -> bool {
  matches!(
    value,
    "ignore"
      | "poweroff"
      | "reboot"
      | "halt"
      | "kexec"
      | "suspend"
      | "hibernate"
      | "suspend-then-hibernate"
      | "lock"
      | "handled-by-system"
  )
}

pub(super) const POWER_KEY_VALUES: [&str; 7] = [
  "ignore", "poweroff", "reboot", "halt", "kexec", "suspend", "lock",
];

pub(super) fn split_lid(values: &[String]) -> Result<(&str, &str), String> {
  match values {
    [value] => Ok(("default", value)),
    [context, value] => {
      if !["default", "battery", "ac", "docked"].contains(&context.as_str()) {
        return Err("lid context must be default, battery, ac or docked".into());
      }
      Ok((context, value))
    }
    _ => Err("set-lid expects [value] or [context, value]".into()),
  }
}

pub(super) fn lid_key(context: &str) -> Result<&'static str, String> {
  match context {
    "default" => Ok("HandleLidSwitch"),
    "battery" => Ok("HandleLidSwitch"),
    "ac" => Ok("HandleLidSwitchExternalPower"),
    "docked" => Ok("HandleLidSwitchDocked"),
    _ => Err("unsupported lid context".into()),
  }
}

pub(super) const LOGIND_DROP_IN: &str = "/etc/systemd/logind.conf.d/argvus.conf";

/// Updates (or creates) the ARGVUS logind drop-in, replacing only the keys in
/// `entries` while preserving every other line and key already present.
pub(super) fn update_logind_drop_in(entries: &[(&str, String)]) -> Result<(), String> {
  update_logind_drop_in_at(Path::new(LOGIND_DROP_IN), entries, true)
}

pub(super) fn update_logind_drop_in_at(
  path: &Path,
  entries: &[(&str, String)],
  backup: bool,
) -> Result<(), String> {
  if fs::symlink_metadata(path)
    .map(|metadata| metadata.file_type().is_symlink())
    .unwrap_or(false)
  {
    return Err("refusing to update a symlinked logind drop-in".into());
  }
  let existing = fs::read_to_string(path).unwrap_or_default();
  let mut updated = String::new();
  let mut found: Vec<bool> = vec![false; entries.len()];
  for line in existing.lines() {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
      let mut replaced = false;
      for (index, (key, _)) in entries.iter().enumerate() {
        if let Some(rest) = trimmed.strip_prefix(key)
          && rest.trim_start().starts_with('=')
        {
          updated.push_str(key);
          updated.push('=');
          updated.push_str(&entries[index].1);
          found[index] = true;
          replaced = true;
          break;
        }
      }
      if replaced {
        updated.push('\n');
        continue;
      }
      if trimmed.is_empty() && updated.ends_with('\n') {
        continue;
      }
    }
    updated.push_str(line);
    updated.push('\n');
  }
  for (index, (key, value)) in entries.iter().enumerate() {
    if !found[index] {
      if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
      }
      updated.push_str(key);
      updated.push('=');
      updated.push_str(value);
      updated.push('\n');
    }
  }
  let original = existing.trim_end();
  let written = updated.trim_end();
  if original != written {
    if backup && path.exists() {
      let _ = backup_file(path);
    }
    let metadata = fs::metadata(path)
      .or_else(|_| fs::metadata(Path::new("/etc/systemd/logind.conf")))
      .map_err(|error| error.to_string())?;
    let tmp = path.with_extension("tmp");
    {
      let mut file = fs::File::create(&tmp).map_err(|error| error.to_string())?;
      file
        .set_permissions(metadata.permissions())
        .map_err(|error| error.to_string())?;
      file
        .write_all((String::from(written) + "\n").as_bytes())
        .map_err(|error| error.to_string())?;
      file.sync_all().map_err(|error| error.to_string())?;
    }
    fs::rename(&tmp, path).map_err(|error| error.to_string())?;
  }
  Ok(())
}

pub(super) fn reload_logind() -> Result<(), String> {
  let _ = run_command("systemctl", &["kill", "-s", "HUP", "systemd-logind"]);
  Ok(())
}

pub(super) fn set_governor(value: &str) -> Result<(), String> {
  if locate_command(&["powerprofilesctl"]).is_some() {
    let profile = match value {
      "performance" => Some("performance"),
      "powersave" | "power-saver" => Some("power-saver"),
      "balanced" => Some("balanced"),
      _ => None,
    };
    if let Some(profile) = profile
      && run_command("powerprofilesctl", &["set", profile]).is_ok()
    {
      return Ok(());
    }
  }
  let mut changed = 0;
  if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu/cpufreq") {
    for entry in entries.flatten() {
      let path = entry.path();
      let available =
        fs::read_to_string(path.join("scaling_available_governors")).unwrap_or_default();
      if available
        .split_whitespace()
        .any(|candidate| candidate == value)
        && fs::write(path.join("scaling_governor"), format!("{value}\n")).is_ok()
      {
        changed += 1;
      }
    }
  }
  if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
    for entry in entries.flatten() {
      if !entry.file_name().to_string_lossy().starts_with("cpu") {
        continue;
      }
      let path = entry.path().join("cpufreq");
      let available =
        fs::read_to_string(path.join("scaling_available_governors")).unwrap_or_default();
      if available
        .split_whitespace()
        .any(|candidate| candidate == value)
        && fs::write(path.join("scaling_governor"), format!("{value}\n")).is_ok()
      {
        changed += 1;
      }
    }
  }
  (changed > 0)
    .then_some(())
    .ok_or_else(|| "no compatible cpufreq policy found".into())
}
