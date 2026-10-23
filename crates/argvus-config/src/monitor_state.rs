//! Monitor settings changed by the display tools.
//!
//! `config.json` under `/displays` is the authority for the monitor layout. The
//! display script and the nwg-displays import hand their changes here, and the
//! Hyprland monitor file is projected from that state. Nothing else writes it.
//!
//! The legacy state file keeps `monitor.key=value` lines for older consumers.
//! Its keys are translated to the canonical fields below: `rotation` in degrees
//! becomes the Hyprland `transform` index, and `resolution` and `refresh` are
//! joined into one `mode` as `WxH@R`.

use crate::ConfigStore;
use crate::{ConfigDocument, ConfigError, ConfigResult};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// One canonical field of a monitor and its value.
#[derive(Debug, PartialEq)]
struct Change {
  field: &'static str,
  value: Value,
}

/// Applies one `key=value` change to a monitor and projects the layout. An
/// empty value removes a canonical field, so the monitor falls back to the
/// defaults of Hyprland for that field.
pub fn set(store: &ConfigStore, monitor: &str, key: &str, value: &str) -> ConfigResult<()> {
  store.modify(|document| apply_change(document, monitor, key, value))?;
  crate::project::project(store, false)?;
  Ok(())
}

/// Sets the primary monitor. An empty name clears it.
pub fn primary(store: &ConfigStore, monitor: &str) -> ConfigResult<()> {
  store.modify(|document| {
    if monitor.is_empty() {
      document.unset("/displays/primary_monitor")
    } else {
      document.set("/displays/primary_monitor", Value::from(monitor))
    }
  })?;
  crate::project::project(store, false)?;
  Ok(())
}

/// Sets the workspaces of a monitor from a comma separated list of ids. An
/// empty list removes the monitor's workspaces.
pub fn workspaces(store: &ConfigStore, monitor: &str, ids: &str) -> ConfigResult<()> {
  let ids = parse_workspace_ids(ids)?;
  let pointer = format!("/displays/workspaces/{}", escape_pointer(monitor));
  store.modify(|document| {
    if ids.is_empty() {
      document.unset(&pointer)
    } else {
      document.set(&pointer, Value::from(ids))
    }
  })?;
  crate::project::project(store, false)?;
  Ok(())
}

/// Removes a monitor that is no longer part of the layout, with its workspaces.
pub fn remove(store: &ConfigStore, monitor: &str) -> ConfigResult<()> {
  if monitor.is_empty() {
    return Err(ConfigError::Invalid("monitor name is empty".into()));
  }
  store.modify(|document| {
    document.unset(&format!("/displays/monitors/{}", escape_pointer(monitor)))?;
    document.unset(&format!("/displays/workspaces/{}", escape_pointer(monitor)))
  })?;
  crate::project::project(store, false)?;
  Ok(())
}

fn parse_workspace_ids(text: &str) -> ConfigResult<Vec<u32>> {
  text
    .split(',')
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .map(|part| {
      part
        .parse::<u32>()
        .ok()
        .filter(|id| *id >= 1)
        .ok_or_else(|| invalid("workspace", part))
    })
    .collect()
}

/// Imports every `monitor.key=value` line of a legacy state file in one change,
/// then projects the layout. The file is the nwg-displays import, so its lines
/// are always applied.
pub fn import(store: &ConfigStore, state_file: &Path) -> ConfigResult<()> {
  import_lines(store, &read_lines(state_file)?)
}

/// Applies `monitor.key=value` lines in order, in one change, then projects.
pub fn import_lines(store: &ConfigStore, lines: &[String]) -> ConfigResult<()> {
  store.modify(|document| {
    for line in lines {
      let (monitor, key, value) = parse_line(line)?;
      apply_change(document, monitor, key, value)?;
    }
    Ok(())
  })?;
  crate::project::project(store, false)?;
  Ok(())
}

/// Checks one legacy key and value without a monitor, so the display command
/// refuses a bad value before it touches the runtime.
pub fn validate(key: &str, value: &str) -> ConfigResult<()> {
  changes(Some("1x1@1"), key, value).map(|_| ())
}

/// Imports a legacy state file only when the canonical layout has no monitor
/// yet, so an installation that predates `/displays` keeps its layout. Later
/// changes live in `config.json` and are never overwritten by the file.
pub fn sync_legacy(store: &ConfigStore, state_file: &Path) -> ConfigResult<()> {
  if !import_legacy_if_needed(store, state_file)? {
    crate::project::project(store, false)?;
  }
  Ok(())
}

/// The import half of [`sync_legacy`], without the projection that follows it.
/// Returns whether the legacy file was imported (the import projects itself).
///
/// Session preparation uses it alone: `argvus-config.service` projected the
/// store just before, so another projection would only repeat that work.
pub fn import_legacy_if_needed(store: &ConfigStore, state_file: &Path) -> ConfigResult<bool> {
  let document = store.load()?;
  let has_monitors = document
    .get("/displays/monitors")
    .and_then(Value::as_object)
    .is_some_and(|monitors| !monitors.is_empty());
  if !has_monitors && state_file.is_file() {
    import(store, state_file)?;
    return Ok(true);
  }
  Ok(false)
}

fn apply_change(
  document: &mut ConfigDocument,
  monitor: &str,
  key: &str,
  value: &str,
) -> ConfigResult<()> {
  if monitor.is_empty() {
    return Err(ConfigError::Invalid("monitor name is empty".into()));
  }
  if value.is_empty() {
    let Some(field) = canonical_field(key) else {
      return Err(invalid(key, value));
    };
    return document.unset(&monitor_pointer(monitor, field));
  }
  let current_mode = document
    .get(&monitor_pointer(monitor, "mode"))
    .and_then(Value::as_str)
    .map(str::to_string);
  for change in changes(current_mode.as_deref(), key, value)? {
    document.set(&monitor_pointer(monitor, change.field), change.value)?;
  }
  Ok(())
}

/// The canonical field of a key that can be cleared with an empty value.
fn canonical_field(key: &str) -> Option<&'static str> {
  Some(match key {
    "mode" => "mode",
    "dpms" => "dpms",
    "position" => "position",
    "scale" => "scale",
    "transform" => "transform",
    "vrr" => "vrr",
    "hdr" => "hdr",
    "mirror" => "mirror",
    "bitdepth" => "bitdepth",
    "disabled" => "disabled",
    "sdr_brightness" => "sdr_brightness",
    "sdr_saturation" => "sdr_saturation",
    _ => return None,
  })
}

/// The canonical fields of one legacy change. `current_mode` is the mode stored
/// for the monitor, so a resolution keeps its refresh rate and a refresh rate
/// keeps its resolution. A power change is runtime state and is not stored.
fn changes(current_mode: Option<&str>, key: &str, value: &str) -> ConfigResult<Vec<Change>> {
  match key {
    "scale" => {
      let scale: f64 = value.parse().map_err(|_| invalid(key, value))?;
      if !(0.1..=10.0).contains(&scale) {
        return Err(invalid(key, value));
      }
      Ok(vec![field("scale", number_text(value)?)])
    }
    "resolution" => {
      if !is_size(value) {
        return Err(invalid(key, value));
      }
      let mode = match current_refresh(current_mode) {
        Some(refresh) => format!("{value}@{refresh}"),
        None => value.to_string(),
      };
      Ok(vec![field("mode", Value::from(mode))])
    }
    "refresh" => {
      let refresh: f64 = value.parse().map_err(|_| invalid(key, value))?;
      if !(1.0..=2000.0).contains(&refresh) {
        return Err(invalid(key, value));
      }
      // The refresh rate is part of the mode, so it needs a resolution.
      let Some(resolution) = current_mode.and_then(|mode| mode.split('@').next()) else {
        return Err(ConfigError::Invalid(
          "set the resolution before the refresh rate".into(),
        ));
      };
      Ok(vec![field(
        "mode",
        Value::from(format!("{resolution}@{value}")),
      )])
    }
    "position" => {
      if !is_size(value) {
        return Err(invalid(key, value));
      }
      Ok(vec![field("position", Value::from(value))])
    }
    "rotation" => {
      let transform = match value {
        "0" => 0,
        "90" => 1,
        "180" => 2,
        "270" => 3,
        _ => return Err(invalid(key, value)),
      };
      Ok(vec![field("transform", Value::from(transform))])
    }
    "enabled" => match value {
      "true" => Ok(vec![field("disabled", Value::from(false))]),
      "false" => Ok(vec![field("disabled", Value::from(true))]),
      _ => Err(invalid(key, value)),
    },
    "power" => match value {
      "on" => Ok(vec![field("dpms", Value::from(true))]),
      "off" => Ok(vec![field("dpms", Value::from(false))]),
      _ => Err(invalid(key, value)),
    },
    "mirror" => {
      if value.is_empty() {
        return Err(invalid(key, value));
      }
      Ok(vec![field("mirror", Value::from(value))])
    }
    "bitdepth" => Ok(vec![field("bitdepth", number_text(value)?)]),
    // The canonical fields the Control Center edits directly.
    "mode" => {
      if !is_mode(value) {
        return Err(invalid(key, value));
      }
      Ok(vec![field("mode", Value::from(value))])
    }
    "disabled" => match value {
      "true" | "false" => Ok(vec![field("disabled", Value::from(value == "true"))]),
      _ => Err(invalid(key, value)),
    },
    "transform" => {
      let transform: u8 = value.parse().map_err(|_| invalid(key, value))?;
      if transform > 7 {
        return Err(invalid(key, value));
      }
      Ok(vec![field("transform", Value::from(transform))])
    }
    "vrr" => {
      let vrr: u8 = value.parse().map_err(|_| invalid(key, value))?;
      if vrr > 2 {
        return Err(invalid(key, value));
      }
      Ok(vec![field("vrr", Value::from(vrr))])
    }
    "hdr" => match value {
      "-1" | "0" | "1" => Ok(vec![field("hdr", number_text(value)?)]),
      _ => Err(invalid(key, value)),
    },
    "sdr_brightness" | "sdr_saturation" => {
      let amount: f64 = value.parse().map_err(|_| invalid(key, value))?;
      if !(0.0..=2.0).contains(&amount) {
        return Err(invalid(key, value));
      }
      let field_name = if key == "sdr_brightness" {
        "sdr_brightness"
      } else {
        "sdr_saturation"
      };
      Ok(vec![field(field_name, number_text(value)?)])
    }
    _ => Err(ConfigError::Invalid(format!("unknown monitor key: {key}"))),
  }
}

fn field(name: &'static str, value: Value) -> Change {
  Change { field: name, value }
}

fn invalid(key: &str, value: &str) -> ConfigError {
  ConfigError::Invalid(format!("invalid value for {key}: {value}"))
}

/// A number written as typed, so `1.5` stays `1.5` in the file.
fn number_text(value: &str) -> ConfigResult<Value> {
  serde_json::from_str::<Value>(value)
    .ok()
    .filter(Value::is_number)
    .ok_or_else(|| invalid("number", value))
}

/// A Hyprland mode: `WxH`, `WxH@R`, or one of the automatic words.
fn is_mode(value: &str) -> bool {
  if matches!(value, "preferred" | "auto" | "highres") {
    return true;
  }
  let (size, refresh) = match value.split_once('@') {
    Some((size, refresh)) => (size, Some(refresh)),
    None => (value, None),
  };
  is_size(size) && refresh.is_none_or(|refresh| refresh.parse::<f64>().is_ok_and(|rate| rate > 0.0))
}

/// `WxH` with digits on both sides, used for resolutions and positions.
fn is_size(value: &str) -> bool {
  let Some((width, height)) = value.split_once('x') else {
    return false;
  };
  let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
  digits(width) && digits(height)
}

/// The refresh part of `WxH@R`, when the mode has one.
fn current_refresh(mode: Option<&str>) -> Option<&str> {
  mode?.split_once('@').map(|(_, refresh)| refresh)
}

/// The JSON pointer of one field of a monitor.
fn monitor_pointer(monitor: &str, field: &str) -> String {
  format!("/displays/monitors/{}/{field}", escape_pointer(monitor))
}

/// Escapes one pointer component, so a name with a `/` or a `~` stays one
/// component.
fn escape_pointer(name: &str) -> String {
  name.replace('~', "~0").replace('/', "~1")
}

pub fn read_lines(state_file: &Path) -> ConfigResult<Vec<String>> {
  Ok(
    fs::read_to_string(state_file)?
      .lines()
      .map(str::trim)
      .filter(|line| !line.is_empty())
      .map(str::to_string)
      .collect(),
  )
}

/// Splits `monitor.key=value`. The monitor is everything before the first dot,
/// as the legacy state file has always been read.
fn parse_line(line: &str) -> ConfigResult<(&str, &str, &str)> {
  let (monitor, rest) = line
    .split_once('.')
    .ok_or_else(|| ConfigError::Invalid(format!("invalid state line: {line}")))?;
  let (key, value) = rest
    .split_once('=')
    .ok_or_else(|| ConfigError::Invalid(format!("invalid state line: {line}")))?;
  Ok((monitor, key, value))
}

#[cfg(test)]
mod tests {
  use super::{Change, changes, monitor_pointer, parse_line};
  use serde_json::Value;

  fn one(current_mode: Option<&str>, key: &str, value: &str) -> Change {
    let mut all = changes(current_mode, key, value).expect("valid change");
    assert_eq!(all.len(), 1, "one canonical field for {key}");
    all.remove(0)
  }

  #[test]
  fn rotation_in_degrees_becomes_the_transform_index() {
    assert_eq!(one(None, "rotation", "90").value, Value::from(1));
    assert_eq!(one(None, "rotation", "180").value, Value::from(2));
    assert_eq!(one(None, "rotation", "270").value, Value::from(3));
    assert_eq!(one(None, "rotation", "0").value, Value::from(0));
    assert!(changes(None, "rotation", "45").is_err());
  }

  #[test]
  fn resolution_keeps_the_refresh_rate_of_the_mode() {
    let mode = one(Some("1920x1080@60"), "resolution", "2560x1440");
    assert_eq!(mode.field, "mode");
    assert_eq!(mode.value, Value::from("2560x1440@60"));
    assert_eq!(
      one(None, "resolution", "1920x1080").value,
      Value::from("1920x1080")
    );
  }

  #[test]
  fn refresh_rate_keeps_the_resolution_of_the_mode() {
    assert_eq!(
      one(Some("1920x1080@60"), "refresh", "144").value,
      Value::from("1920x1080@144")
    );
    assert_eq!(
      one(Some("1920x1080@60"), "refresh", "59.94").value,
      Value::from("1920x1080@59.94")
    );
    assert!(changes(None, "refresh", "144").is_err());
  }

  #[test]
  fn scale_is_kept_as_typed_within_the_range() {
    assert_eq!(one(None, "scale", "1.5").value, serde_json::json!(1.5));
    assert!(changes(None, "scale", "0.05").is_err());
    assert!(changes(None, "scale", "11").is_err());
  }

  #[test]
  fn enabled_maps_to_disabled_and_power_is_kept_as_dpms() {
    assert_eq!(one(None, "enabled", "false").value, Value::from(true));
    assert_eq!(one(None, "enabled", "true").value, Value::from(false));
    assert_eq!(one(None, "power", "off").value, Value::from(false));
    assert_eq!(one(None, "power", "on").field, "dpms");
    assert!(changes(None, "power", "sleep").is_err());
  }

  #[test]
  fn unknown_keys_and_malformed_sizes_are_rejected() {
    assert!(changes(None, "gamma", "1").is_err());
    assert!(changes(None, "position", "0,0").is_err());
    assert!(changes(None, "resolution", "x1080").is_err());
  }

  #[test]
  fn legacy_lines_split_at_the_first_dot() {
    assert_eq!(
      parse_line("DP-1.position=0x0").expect("line"),
      ("DP-1", "position", "0x0")
    );
    assert!(parse_line("DP-1 position").is_err());
  }

  #[test]
  fn monitor_names_are_escaped_in_the_pointer() {
    assert_eq!(
      monitor_pointer("a/b", "scale"),
      "/displays/monitors/a~1b/scale"
    );
  }
}
