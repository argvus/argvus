//! Implements isolated integration with system tools and APIs in crate `argvus control center displays`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::{
  DisplayState, Mode, Monitor, MonitorInfo, MonitorProfile, PersistedConfig, PersistedMonitor,
};
use argvus_control_center_core::{
  config::reload_argvus_config_service,
  paths::argvus_data_home,
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Executes the `hyprctl_version` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn hyprctl_version() -> (u32, u32) {
  match SystemProcessRunner.run(&ProcessRequest::new("hyprctl").arg("version")) {
    Ok(out) => {
      let first = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
      parse_version(&first)
    }
    Err(_) => (0, 0),
  }
}

/// Converts input data into `parse_version` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_version(text: &str) -> (u32, u32) {
  let mut digits = String::new();
  for character in text.chars() {
    if character.is_ascii_digit() || character == '.' {
      digits.push(character);
    } else if !digits.is_empty() {
      break;
    }
  }
  let mut parts = digits.split('.');
  let major = parts
    .next()
    .and_then(|value| value.parse::<u32>().ok())
    .unwrap_or(0);
  let minor = parts
    .next()
    .and_then(|value| value.parse::<u32>().ok())
    .unwrap_or(0);
  (major, minor)
}

/// Lists all monitors Hyprland knows about (active and disabled). Tries
/// `hyprctl -j monitors all` first and falls back to plain `monitors` on
/// versions that do not accept the `all` flag.
pub fn list_monitors() -> Result<Vec<Monitor>, String> {
  let out = SystemProcessRunner
    .run(
      &ProcessRequest::new("hyprctl")
        .arg("-j")
        .arg("monitors")
        .arg("all"),
    )
    .or_else(|_| SystemProcessRunner.run(&ProcessRequest::new("hyprctl").arg("-j").arg("monitors")))
    .map_err(|error| error.to_string())?;
  if out.status.is_none_or(|status| status != 0) {
    return Err(
      terminal_text(&String::from_utf8_lossy(&out.stderr))
        .trim()
        .into(),
    );
  }
  let value: Value = serde_json::from_slice(&out.stdout).map_err(|error| error.to_string())?;
  let rows = value
    .as_array()
    .ok_or("hyprctl monitors JSON was not an array")?;
  rows
    .iter()
    .map(monitor_from)
    .collect::<Result<Vec<_>, String>>()
}

/// Executes the `monitor_from` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn monitor_from(row: &Value) -> Result<Monitor, String> {
  let name = row
    .get("name")
    .and_then(Value::as_str)
    .ok_or("monitor without name")?;
  let modes = row
    .get("availableModes")
    .and_then(Value::as_array)
    .map(|modes| modes.iter().filter_map(mode_from).collect::<Vec<Mode>>())
    .unwrap_or_default();
  let mirror_of = row
    .get("mirrorOf")
    .and_then(Value::as_str)
    .map(|value| value.to_string())
    .filter(|value| value != "none" && !value.is_empty());
  let active_workspace = row
    .get("activeWorkspace")
    .and_then(Value::as_object)
    .and_then(|object| object.get("id"))
    .and_then(Value::as_i64)
    .map(|value| value as i32);
  let vrr = match row.get("vrr") {
    Some(value) => value.as_i64().map(|v| v as i32),
    None => None,
  }
  .or_else(|| {
    row
      .get("vrr")
      .and_then(Value::as_bool)
      .map(|on| if on { 1_i32 } else { 0 })
  })
  .unwrap_or(0);
  Ok(Monitor {
    id: row
      .get("id")
      .and_then(Value::as_i64)
      .map(|v| v as i32)
      .unwrap_or(-1),
    name: name.to_string(),
    x: row
      .get("x")
      .and_then(Value::as_i64)
      .map(|v| v as i32)
      .unwrap_or(0),
    y: row
      .get("y")
      .and_then(Value::as_i64)
      .map(|v| v as i32)
      .unwrap_or(0),
    width: row
      .get("width")
      .and_then(Value::as_u64)
      .map(|v| v as u32)
      .unwrap_or(0),
    height: row
      .get("height")
      .and_then(Value::as_u64)
      .map(|v| v as u32)
      .unwrap_or(0),
    refresh_rate: row
      .get("refreshRate")
      .and_then(Value::as_f64)
      .unwrap_or(0.0),
    scale: row.get("scale").and_then(Value::as_f64).unwrap_or(1.0),
    transform: row
      .get("transform")
      .and_then(Value::as_i64)
      .map(|v| v as i32)
      .unwrap_or(0),
    focused: row.get("focused").and_then(Value::as_bool).unwrap_or(false),
    vrr,
    dpms_status: row
      .get("dpmsStatus")
      .and_then(Value::as_bool)
      .map(|on| if on { "on" } else { "off" })
      .unwrap_or("")
      .to_string(),
    disabled: row
      .get("disabled")
      .and_then(Value::as_bool)
      .unwrap_or(false),
    modes,
    connected: true,
    info: MonitorInfo {
      make: row
        .get("make")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string(),
      model: row
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string(),
      serial: row
        .get("serial")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string(),
      description: row
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string(),
      physical_width: row
        .get("physicalWidth")
        .and_then(Value::as_u64)
        .map(|v| v as u32),
      physical_height: row
        .get("physicalHeight")
        .and_then(Value::as_u64)
        .map(|v| v as u32),
      current_format: row
        .get("currentFormat")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string(),
      mirror_of,
      sdr_brightness: row.get("sdrBrightness").and_then(Value::as_f64),
      sdr_saturation: row.get("sdrSaturation").and_then(Value::as_f64),
    },
    active_workspace,
  })
}

/// Executes the `mode_from` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn mode_from(mode: &Value) -> Option<Mode> {
  if let Some(s) = mode.as_str() {
    return parse_mode_string(s);
  }
  Some(Mode {
    id: mode.get("id").and_then(Value::as_i64).unwrap_or(0),
    width: mode.get("width").and_then(Value::as_u64)? as u32,
    height: mode.get("height").and_then(Value::as_u64)? as u32,
    refresh_rate: mode
      .get("refreshRate")
      .or_else(|| mode.get("refresh_rate"))
      .and_then(Value::as_f64)?,
    bit_depth: mode
      .get("bitDepth")
      .or_else(|| mode.get("bit_depth"))
      .and_then(Value::as_u64)
      .unwrap_or(8) as u32,
  })
}

/// Converts input data into `parse_mode_string` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_mode_string(s: &str) -> Option<Mode> {
  let s = s.trim_end_matches("Hz");
  let (res, rate_str) = s.split_once('@')?;
  let (w_str, h_str) = res.split_once('x')?;
  let width = w_str.parse::<u32>().ok()?;
  let height = h_str.parse::<u32>().ok()?;
  let refresh_rate = rate_str.parse::<f64>().ok()?;
  Some(Mode {
    id: 0,
    width,
    height,
    refresh_rate,
    bit_depth: 8,
  })
}

/// Executes the `run_hyprctl` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_hyprctl(arguments: &[&str]) -> Result<(), String> {
  if arguments.iter().any(|argument| {
    argument.is_empty()
      || argument
        .chars()
        .any(|character| character.is_control() || character == '\n')
  }) {
    return Err("invalid hyprctl arguments".into());
  }
  let mut request = ProcessRequest::new("hyprctl");
  for argument in arguments {
    request = request.arg(*argument);
  }
  let out = SystemProcessRunner
    .run(&request)
    .map_err(|error| error.to_string())?;
  if out.status.is_none_or(|status| status != 0) {
    let stderr = terminal_text(&String::from_utf8_lossy(&out.stderr))
      .trim()
      .to_string();
    return Err(if stderr.is_empty() {
      format!("hyprctl falhou ao executar '{}'", arguments.join(" "))
    } else {
      stderr
    });
  }
  Ok(())
}

/// Applies an immediate Hyprland monitor change:
/// `hyprctl keyword monitor <comma-separated arguments>`.
pub fn apply_keyword(arguments: &str) -> Result<(), String> {
  if arguments.is_empty()
    || arguments
      .chars()
      .any(|character| character.is_control() || character == '\n')
  {
    return Err("invalid hyprctl keyword arguments".into());
  }
  run_hyprctl(&["keyword", "monitor", arguments])
}

/// Applies the `apply_workspace` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn apply_workspace(workspace: u32, monitor: Option<&str>) -> Result<(), String> {
  match monitor {
    Some(monitor) => {
      let argument = format!("{workspace},monitor:{monitor}");
      run_hyprctl(&["keyword", "workspace", &argument])
    }
    None => {
      let argument = format!("{workspace},monitor:");
      run_hyprctl(&["keyword", "workspace", &argument])
    }
  }
}

/// Applies the `set_dpms` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set_dpms(name: &str, on: bool) -> Result<(), String> {
  run_hyprctl(&["dispatch", "dpms", if on { "on" } else { "off" }, name])
}

/// Applies the full persisted configuration: writes the generated Lua and
/// triggers a Hyprland config reload so all monitor rules take effect.
pub fn apply_all(config: &PersistedConfig) -> Result<(), String> {
  save_config(config)?;
  run_hyprctl(&["reload"])
}

/// Executes the `config_path` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn config_path() -> PathBuf {
  argvus_data_home()
    .join("generated")
    .join("hypr")
    .join("monitors.lua")
}

/// Executes the `state_path` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn state_path() -> PathBuf {
  argvus_data_home().join("display").join("config.toml")
}

/// Reads the monitors generated for Hyprland. Both the executable
/// `hl.monitor({...})` / `hl.workspace_rule({...})` format written by this
/// module and the legacy `return {...}` format written by older releases are
/// parsed so existing user configs migrate in place.
pub fn load_config() -> PersistedConfig {
  if let Some(config) = load_canonical_config() {
    return config;
  }
  let content = match fs::read_to_string(config_path()) {
    Ok(content) => content,
    Err(_) => return PersistedConfig::default(),
  };
  if content.contains("hl.monitor") || content.contains("hl.workspace_rule") {
    parse_generated(&content)
  } else {
    parse_legacy(&content)
  }
}

fn load_canonical_config() -> Option<PersistedConfig> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", "/displays", "--effective"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let value: Value = serde_json::from_slice(&output.stdout).ok()?;
  let mut config = PersistedConfig {
    primary_monitor: value
      .get("primary_monitor")
      .and_then(Value::as_str)
      .map(str::to_owned),
    ..PersistedConfig::default()
  };
  for (name, value) in value.get("monitors").and_then(Value::as_object)? {
    let mut monitor: StoredMonitor = serde_json::from_value(value.clone()).ok()?;
    monitor.name = name.clone();
    config
      .monitors
      .push((name.clone(), stored_to_persisted(&monitor)));
  }
  if let Some(workspaces) = value.get("workspaces").and_then(Value::as_object) {
    for (monitor, ids) in workspaces {
      config.workspaces.push((
        monitor.clone(),
        ids
          .as_array()?
          .iter()
          .map(|id| id.as_u64().and_then(|id| u32::try_from(id).ok()))
          .collect::<Option<Vec<_>>>()?,
      ));
    }
  }
  config.monitors.sort_by(|left, right| left.0.cmp(&right.0));
  Some(config)
}

/// Converts input data into `parse_generated` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_generated(content: &str) -> PersistedConfig {
  let mut config = PersistedConfig::default();
  for block in lua_call_bodies(content, "hl.monitor") {
    if let Some((name, persisted)) = parse_monitor_block(&block) {
      config.monitors.push((name, persisted));
    }
  }
  for block in lua_call_bodies(content, "hl.workspace_rule") {
    let fields = entry_fields(&block);
    let workspace = fields
      .iter()
      .find(|(key, _)| *key == "workspace")
      .and_then(|(_, value)| value.trim_matches('"').parse::<u32>().ok());
    let monitor = fields
      .iter()
      .find(|(key, _)| *key == "monitor")
      .map(|(_, value)| value.trim_matches('"').to_string());
    if let (Some(workspace), Some(monitor)) = (workspace, monitor) {
      config.set_workspaces(&monitor, {
        let mut ids = config.workspaces_of(&monitor);
        if !ids.contains(&workspace) {
          ids.push(workspace);
        }
        ids.sort_unstable();
        ids
      });
      let default = fields
        .iter()
        .any(|(key, value)| *key == "default" && value.trim() == "true");
      if default && workspace == 1 {
        config.primary_monitor = Some(monitor);
      }
    }
  }
  config.monitors.sort_by(|left, right| left.0.cmp(&right.0));
  config
}

/// Converts input data into `parse_monitor_block` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_monitor_block(block: &str) -> Option<(String, PersistedMonitor)> {
  let fields = entry_fields(block);
  let name = fields
    .iter()
    .find(|(key, _)| *key == "output")?
    .1
    .to_string();
  if name.is_empty() {
    return None;
  }
  let mut persisted = PersistedMonitor::default();
  if let Some(value) = fields.iter().find(|(key, _)| *key == "mode") {
    persisted.mode = Some(value.1.to_string());
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "position") {
    persisted.position = Some(value.1.to_string());
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "scale") {
    persisted.scale = value.1.parse::<f64>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "transform") {
    persisted.transform = value.1.parse::<i32>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "vrr") {
    persisted.vrr = value.1.parse::<i32>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "mirror") {
    persisted.mirror = Some(value.1.to_string());
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "bitdepth") {
    persisted.bitdepth = value.1.parse::<i32>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "disabled") {
    persisted.disabled = Some(value.1.trim() == "true");
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "supports_hdr") {
    persisted.hdr = value.1.parse::<i32>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "hdr") {
    persisted.hdr = value.1.parse::<i32>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "sdrbrightness") {
    persisted.sdr_brightness = value.1.parse::<f64>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "sdrsaturation") {
    persisted.sdr_saturation = value.1.parse::<f64>().ok();
  }
  Some((name, persisted))
}

/// Converts input data into `parse_legacy` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_legacy(content: &str) -> PersistedConfig {
  let primary_monitor = content.lines().find_map(|line| {
    line
      .trim_start()
      .starts_with("primary_monitor")
      .then(|| quoted_value(line, "primary_monitor"))
      .flatten()
  });
  let monitors = table_body(content, "monitors")
    .map(|body| {
      entry_bodies(body)
        .into_iter()
        .filter_map(parse_monitor_entry)
        .collect()
    })
    .unwrap_or_default();
  PersistedConfig {
    primary_monitor,
    monitors,
    workspaces: Vec::new(),
  }
}

/// Converts input data into `parse_monitor_entry` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_monitor_entry(block: &str) -> Option<(String, PersistedMonitor)> {
  let fields = entry_fields(block);
  let name = fields.iter().find(|(key, _)| *key == "name")?.1.to_string();
  let mut persisted = PersistedMonitor::default();
  if let Some(value) = fields.iter().find(|(key, _)| *key == "mode") {
    persisted.mode = Some(value.1.to_string());
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "pos") {
    persisted.position = Some(value.1.to_string());
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "scale") {
    persisted.scale = value.1.parse::<f64>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "transform") {
    persisted.transform = value.1.parse::<i32>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "vrr") {
    persisted.vrr = value.1.parse::<i32>().ok();
  }
  if let Some(value) = fields.iter().find(|(key, _)| *key == "hdr") {
    persisted.hdr = value.1.parse::<i32>().ok();
  }
  Some((name, persisted))
}

/// Splits a single-line Lua field list like
/// `output = "eDP-1", mode = "1920x1080@60", scale = 1.25` (or a multi-line
/// equivalent wrapped in `{ ... }`) into key/value pairs. Values keep their
/// raw text so numbers and booleans survive.
fn entry_fields(block: &str) -> Vec<(&str, &str)> {
  let mut fields = Vec::new();
  for token in block.split(',') {
    let token = token.trim().trim_start_matches('{').trim_end_matches('}');
    let Some((key, value)) = token.trim().split_once('=') else {
      continue;
    };
    fields.push((key.trim(), value.trim().trim_matches('"')));
  }
  fields
}

/// Extracts the argument body of every `name(`...`)` call, matching nested
/// balanced delimiters so multi-line calls parse too.
fn lua_call_bodies(content: &str, name: &str) -> Vec<String> {
  let mut bodies = Vec::new();
  let mut rest = content;
  while let Some(index) = rest.find(name) {
    let after = &rest[index + name.len()..];
    let Some(open) = after.find('(') else {
      break;
    };
    let (inside, tail) = delim_tail(&after[open..], '(', ')');
    if !inside.is_empty() {
      bodies.push(inside.to_string());
    }
    rest = tail;
  }
  bodies
}

/// Returns the text inside a `open ... close` pair (the first character of
/// `text` must be `open`), skipping nested occurrences of the same delimiter.
fn delim_tail(text: &str, open: char, close: char) -> (&str, &str) {
  if !text.starts_with(open) {
    return ("", "");
  }
  let mut depth = 0i32;
  for (index, character) in text.char_indices() {
    if character == open {
      depth += 1;
    } else if character == close {
      depth -= 1;
      if depth == 0 {
        return (&text[1..index], &text[index + 1..]);
      }
    }
  }
  ("", "")
}

/// Returns the text inside `name = { ... }`, skipping nested braces correctly.
fn table_body<'a>(content: &'a str, name: &str) -> Option<&'a str> {
  let index = content.find(&format!("{name} = {{"))?;
  let rest = &content[index..];
  let open = rest.find('{')?;
  let (inside, _) = delim_tail(&rest[open..], '{', '}');
  Some(inside)
}

/// Splits a brace block into its top-level `{ ... }` entries.
fn entry_bodies(block: &str) -> Vec<&str> {
  let mut entries = Vec::new();
  let mut rest = block;
  while let Some(open) = rest.find('{') {
    let (inside, after) = delim_tail(&rest[open..], '{', '}');
    if inside.trim().is_empty() {
      break;
    }
    entries.push(inside);
    rest = after;
  }
  entries
}

/// Executes the `quoted_value` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn quoted_value(line: &str, key: &str) -> Option<String> {
  let (_, value) = line.split_once(&format!("{key} ="))?;
  let value = value.trim();
  let value = value.strip_prefix('"')?;
  let value = value.strip_suffix(',').unwrap_or(value);
  let value = value.strip_suffix('"')?;
  (!value.contains('"')).then(|| value.to_string())
}

/// Writes the generated Hyprland Lua that `hyprland.lua` executes at startup.
/// Each persisted monitor becomes an `hl.monitor({...})` statement (the same
/// format `argvus-display` produces) and each workspace binding becomes an
/// `hl.workspace_rule({...})` statement, so configuration actually survives a
/// restart — the previous `return {...}` table was discarded by `hyprland.lua`.
pub fn save_config(config: &PersistedConfig) -> Result<(), String> {
  let previous = load_canonical_config().unwrap_or_default();
  persist_layout(&previous, config)?;
  reload_argvus_config_service()
}

/// Writes the layout through `argvus config monitor-*`, which validates each
/// field and projects the Hyprland file from the canonical `/displays`. Only the
/// differences from the previous canonical layout are written, so an unchanged
/// field is never rewritten.
fn persist_layout(previous: &PersistedConfig, config: &PersistedConfig) -> Result<(), String> {
  for (name, _) in &previous.monitors {
    if !config.monitors.iter().any(|(current, _)| current == name) {
      run_monitor_command(&["monitor-remove", name])?;
    }
  }
  for (name, monitor) in &config.monitors {
    let before = previous
      .monitors
      .iter()
      .find(|(previous_name, _)| previous_name == name)
      .map(|(_, monitor)| monitor.clone())
      .unwrap_or_default();
    for (key, now, then) in monitor_fields(monitor, &before) {
      if now != then {
        run_monitor_command(&["monitor-set", name, key, &now])?;
      }
    }
  }
  if config.primary_monitor != previous.primary_monitor {
    run_monitor_command(&[
      "monitor-primary",
      config.primary_monitor.as_deref().unwrap_or(""),
    ])?;
  }
  for (name, _) in &previous.workspaces {
    if !config.workspaces.iter().any(|(current, _)| current == name) {
      run_monitor_command(&["monitor-workspaces", name, ""])?;
    }
  }
  for (name, ids) in &config.workspaces {
    let before = previous
      .workspaces
      .iter()
      .find(|(previous_name, _)| previous_name == name)
      .map(|(_, ids)| ids.clone());
    if before.as_ref() != Some(ids) {
      let list = ids.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
      run_monitor_command(&["monitor-workspaces", name, &list])?;
    }
  }
  Ok(())
}

/// The canonical field name, the value to write now and the value before. An
/// absent field is written as an empty value, which removes it. `dpms` is
/// runtime-only and is not part of the layout.
fn monitor_fields(
  monitor: &PersistedMonitor,
  before: &PersistedMonitor,
) -> Vec<(&'static str, String, String)> {
  fn text<T: ToString>(value: &Option<T>) -> String {
    value.as_ref().map(ToString::to_string).unwrap_or_default()
  }
  vec![
    ("mode", text(&monitor.mode), text(&before.mode)),
    ("position", text(&monitor.position), text(&before.position)),
    ("scale", text(&monitor.scale), text(&before.scale)),
    (
      "transform",
      text(&monitor.transform),
      text(&before.transform),
    ),
    ("vrr", text(&monitor.vrr), text(&before.vrr)),
    ("hdr", text(&monitor.hdr), text(&before.hdr)),
    ("mirror", text(&monitor.mirror), text(&before.mirror)),
    ("bitdepth", text(&monitor.bitdepth), text(&before.bitdepth)),
    ("disabled", text(&monitor.disabled), text(&before.disabled)),
    (
      "sdr_brightness",
      text(&monitor.sdr_brightness),
      text(&before.sdr_brightness),
    ),
    (
      "sdr_saturation",
      text(&monitor.sdr_saturation),
      text(&before.sdr_saturation),
    ),
  ]
}

/// Runs one `argvus config monitor-*` command and reports why it was refused.
fn run_monitor_command(arguments: &[&str]) -> Result<(), String> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(arguments)
    .output()
    .map_err(|error| format!("failed to persist display configuration: {error}"))?;
  if output.status.success() {
    return Ok(());
  }
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
  Err(if stderr.is_empty() {
    format!("argvus config rejected {}", arguments.join(" "))
  } else {
    stderr
  })
}

/// Executes the `append_workspace_rules` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
#[cfg(test)]
fn append_workspace_rules(text: &mut String, config: &PersistedConfig) {
  let mut emitted = Vec::new();
  for (monitor, ids) in &config.workspaces {
    for (index, workspace) in ids.iter().enumerate() {
      let default = config.primary_monitor.as_deref() == Some(monitor.as_str()) && index == 0;
      text.push_str(&format!(
        "hl.workspace_rule({{ workspace = \"{workspace}\", monitor = {monitor:?}{} }})\n",
        if default { ", default = true" } else { "" }
      ));
      emitted.push(*workspace);
    }
  }
  if let Some(primary) = &config.primary_monitor
    && !emitted.contains(&1)
  {
    text.push_str(&format!(
      "hl.workspace_rule({{ workspace = \"1\", monitor = {primary:?}, default = true }})\n"
    ));
  }
}

/// Loads the ARGVUS display state (primary monitor, profiles). Missing or
/// unparsable files collapse to the default empty state so a foreign file is
/// never overwritten.
pub fn load_state() -> DisplayState {
  let mut state = match fs::read_to_string(state_path()) {
    Ok(content) => match toml::from_str::<StoredState>(&content) {
      Ok(stored) => stored.into_model(),
      Err(_) => DisplayState::default(),
    },
    Err(_) => DisplayState::default(),
  };
  // The primary monitor is part of the canonical layout, not of this catalog.
  state.primary_monitor = load_canonical_config().and_then(|config| config.primary_monitor);
  state
}

/// Applies the `save_state` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn save_state(state: &DisplayState) -> Result<(), String> {
  let path = state_path();
  // The primary monitor is written to the canonical layout by `save_config`.
  let mut stored = StoredState::from_model(state);
  stored.primary_monitor = None;
  let content = toml::to_string_pretty(&stored).map_err(|error| error.to_string())?;
  if let Some(parent) = path.parent()
    && !parent.exists()
  {
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let tmp = path.with_extension("tmp");
  fs::write(&tmp, content).map_err(|error| error.to_string())?;
  fs::rename(&tmp, &path).map_err(|error| error.to_string())?;
  Ok(())
}

/// Public accessor other ARGVUS components use to discover the persisted
/// primary monitor without depending on this crate's UI.
#[allow(dead_code)]
pub fn primary_monitor() -> Option<String> {
  load_state().primary_monitor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents `StoredState`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
struct StoredState {
  #[serde(skip_serializing_if = "Option::is_none")]
  primary_monitor: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  active_profile: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  profiles: Vec<StoredProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents `StoredProfile`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
struct StoredProfile {
  name: String,
  #[serde(default)]
  apply_wallpapers: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  primary_monitor: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  monitors: Vec<StoredMonitor>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  workspaces: Vec<StoredWorkspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents `StoredWorkspace`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
struct StoredWorkspace {
  monitor: String,
  #[serde(default)]
  ids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents `StoredMonitor`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
struct StoredMonitor {
  #[serde(default)]
  name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  mode: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  position: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  scale: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  transform: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  vrr: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  hdr: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  mirror: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  bitdepth: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  disabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  sdr_brightness: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  sdr_saturation: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  dpms: Option<bool>,
}

/// Executes the `persisted_to_stored` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn persisted_to_stored(persisted: &PersistedMonitor) -> StoredMonitor {
  StoredMonitor {
    name: String::new(),
    mode: persisted.mode.clone(),
    position: persisted.position.clone(),
    scale: persisted.scale,
    transform: persisted.transform,
    vrr: persisted.vrr,
    hdr: persisted.hdr,
    mirror: persisted.mirror.clone(),
    bitdepth: persisted.bitdepth,
    disabled: persisted.disabled,
    sdr_brightness: persisted.sdr_brightness,
    sdr_saturation: persisted.sdr_saturation,
    dpms: persisted.dpms,
  }
}

/// Executes the `stored_to_persisted` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn stored_to_persisted(stored: &StoredMonitor) -> PersistedMonitor {
  PersistedMonitor {
    mode: stored.mode.clone(),
    position: stored.position.clone(),
    scale: stored.scale,
    transform: stored.transform,
    vrr: stored.vrr,
    hdr: stored.hdr,
    mirror: stored.mirror.clone(),
    bitdepth: stored.bitdepth,
    disabled: stored.disabled,
    sdr_brightness: stored.sdr_brightness,
    sdr_saturation: stored.sdr_saturation,
    dpms: stored.dpms,
  }
}

impl StoredState {
  /// Converts input data into `from_model` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn from_model(state: &DisplayState) -> StoredState {
    StoredState {
      primary_monitor: state.primary_monitor.clone(),
      active_profile: state.active_profile.clone(),
      profiles: state
        .profiles
        .iter()
        .map(StoredProfile::from_model)
        .collect(),
    }
  }

  /// Executes the `into_model` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn into_model(self) -> DisplayState {
    DisplayState {
      primary_monitor: self.primary_monitor,
      active_profile: self.active_profile,
      profiles: self
        .profiles
        .into_iter()
        .map(StoredProfile::into_model)
        .collect(),
    }
  }
}

impl StoredProfile {
  /// Converts input data into `from_model` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn from_model(profile: &MonitorProfile) -> StoredProfile {
    StoredProfile {
      name: profile.name.clone(),
      apply_wallpapers: profile.apply_wallpapers,
      primary_monitor: profile.config.primary_monitor.clone(),
      monitors: profile
        .config
        .monitors
        .iter()
        .map(|(name, persisted)| {
          let mut stored = persisted_to_stored(persisted);
          stored.name = name.clone();
          stored
        })
        .collect(),
      workspaces: profile
        .config
        .workspaces
        .iter()
        .map(|(monitor, ids)| StoredWorkspace {
          monitor: monitor.clone(),
          ids: ids.clone(),
        })
        .collect(),
    }
  }

  /// Executes the `into_model` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn into_model(self) -> MonitorProfile {
    let mut config = PersistedConfig {
      primary_monitor: self.primary_monitor,
      monitors: Vec::new(),
      workspaces: Vec::new(),
    };
    for monitor in self.monitors {
      config
        .monitors
        .push((monitor.name.clone(), stored_to_persisted(&monitor)));
    }
    config.monitors.sort_by(|left, right| left.0.cmp(&right.0));
    for workspace in self.workspaces {
      config.workspaces.push((workspace.monitor, workspace.ids));
    }
    MonitorProfile {
      name: self.name,
      apply_wallpapers: self.apply_wallpapers,
      config,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Defines the constant `SAMPLE`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const SAMPLE: &str = r#"[
  {
    "id": 0,
    "name": "eDP-1",
    "x": 0, "y": 0, "width": 1920, "height": 1080,
    "refreshRate": 60.001,
    "scale": 1.25, "transform": 0, "focused": true,
    "vrr": true, "dpmsStatus": true, "disabled": false,
    "description": "AUO 16.0\"", "make": "AUO", "model": "B160QAN02",
    "serial": "1234", "currentFormat": "XRGB2101010",
    "physicalWidth": 344, "physicalHeight": 194,
    "activeWorkspace": { "id": 1 },
    "availableModes": [
      { "width": 1920, "height": 1080, "refreshRate": 60.0, "bitDepth": 8, "id": 1 },
      { "width": 1920, "height": 1080, "refreshRate": 59.94, "bitDepth": 10, "id": 2 }
    ]
  },
  {
    "id": 1,
    "name": "DP-1", "x": 1920, "y": 0, "width": 2560, "height": 1440,
    "refreshRate": 165.0, "scale": 1.0, "transform": 0, "focused": false,
    "vrr": 0, "dpmsStatus": true, "disabled": true,
    "mirrorOf": "eDP-1", "currentFormat": "XRGB8888",
    "availableModes": []
  }
]"#;

  #[test]
  /// Converts input data into `parses_hyprctl_monitor_json` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_hyprctl_monitor_json() {
    let value: Value = serde_json::from_str(SAMPLE).unwrap();
    let monitor = monitor_from(&value[0]).unwrap();
    assert_eq!(monitor.name, "eDP-1");
    assert_eq!(monitor.refresh_rate, 60.001);
    assert_eq!(monitor.modes.len(), 2);
    assert!(monitor.focused);
    assert_eq!(monitor.info.make, "AUO");
    assert_eq!(monitor.info.mirror_of, None);
    assert_eq!(monitor.active_workspace, Some(1));
    let disabled = monitor_from(&value[1]).unwrap();
    assert!(disabled.disabled);
    assert_eq!(disabled.info.mirror_of.as_deref(), Some("eDP-1"));
  }

  #[test]
  /// Executes the `vrr_field_accepts_boolean_and_integer` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn vrr_field_accepts_boolean_and_integer() {
    let value: Value = serde_json::from_str(SAMPLE).unwrap();
    assert_eq!(monitor_from(&value[0]).unwrap().vrr, 1);
    assert_eq!(monitor_from(&value[1]).unwrap().vrr, 0);
  }

  #[test]
  /// Executes the `versions_are_extracted_from_text` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn versions_are_extracted_from_text() {
    assert_eq!(parse_version("Hyprland 0.42.0, commit..."), (0, 42));
    assert_eq!(parse_version("0.44.1"), (0, 44));
    assert_eq!(parse_version("garbage"), (0, 0));
  }

  #[test]
  /// Executes the `generated_lua_round_trips` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn generated_lua_round_trips() {
    let config = PersistedConfig {
      primary_monitor: Some("eDP-1".into()),
      monitors: vec![
        (
          "eDP-1".into(),
          PersistedMonitor {
            mode: Some("1920x1080@60".into()),
            position: Some("0x0".into()),
            scale: Some(1.25),
            transform: Some(0),
            vrr: Some(1),
            hdr: Some(1),
            bitdepth: Some(10),
            ..Default::default()
          },
        ),
        (
          "DP-1".into(),
          PersistedMonitor {
            disabled: Some(true),
            ..Default::default()
          },
        ),
      ],
      workspaces: vec![("eDP-1".into(), vec![1, 2])],
    };
    let text = serialize_config(&config);
    assert!(text.contains("hl.monitor({ output = \"eDP-1\""), "{text}");
    assert!(text.contains("supports_hdr = 1"), "{text}");
    assert!(text.contains("disabled = true"), "{text}");
    assert!(
      text
        .contains("hl.workspace_rule({ workspace = \"1\", monitor = \"eDP-1\", default = true })"),
      "{text}"
    );
    let parsed = parse_generated(&text);
    assert_eq!(parsed.primary_monitor.as_deref(), Some("eDP-1"));
    assert_eq!(parsed.monitors.len(), 2);
    assert_eq!(parsed.monitors[0].0, "DP-1");
    assert!(parsed.monitors[0].1.disabled == Some(true));
    let (name, monitor) = &parsed.monitors[1];
    assert_eq!(name, "eDP-1");
    assert_eq!(monitor.scale, Some(1.25));
    assert_eq!(monitor.mode.as_deref(), Some("1920x1080@60"));
    assert_eq!(monitor.hdr, Some(1));
    assert_eq!(monitor.bitdepth, Some(10));
    assert_eq!(monitor.vrr, Some(1));
    assert_eq!(parsed.workspaces_of("eDP-1"), vec![1, 2]);
  }

  #[test]
  /// Executes the `multiline_lua_calls_parse` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn multiline_lua_calls_parse() {
    let text = "\
hl.monitor({
  output = \"eDP-1\",
  mode = \"1920x1080@144\",
  scale = 1.25,
  transform = 1,
})
-- trailing comment
hl.workspace_rule({
  workspace = \"7\",
  monitor = \"DP-2\",
})
";
    let parsed = parse_generated(text);
    assert_eq!(parsed.monitors.len(), 1);
    let (name, monitor) = &parsed.monitors[0];
    assert_eq!(name, "eDP-1");
    assert_eq!(monitor.mode.as_deref(), Some("1920x1080@144"));
    assert_eq!(monitor.scale, Some(1.25));
    assert_eq!(monitor.transform, Some(1));
    assert_eq!(parsed.workspaces_of("DP-2"), vec![7]);
  }

  #[test]
  /// Executes the `legacy_configs_still_parse` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn legacy_configs_still_parse() {
    let config = parse_legacy(
      "-- old\nreturn {\n  primary_monitor = \"eDP-1\"\n  monitors = { { name = \"eDP-1\", mode = \"1920x1080@60\", pos = \"0x0\", scale = 1.25, hdr = 1 } }\n}\n",
    );
    assert_eq!(config.primary_monitor.as_deref(), Some("eDP-1"));
    assert_eq!(config.monitors.len(), 1);
    assert_eq!(config.monitors[0].1.hdr, Some(1));
  }

  #[test]
  /// Executes the `foreign_configs_parse_to_defaults` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn foreign_configs_parse_to_defaults() {
    let config = parse_generated("-- not ours\nreturn { }\n");
    assert_eq!(config.primary_monitor, None);
    assert!(config.monitors.is_empty());
  }

  #[test]
  /// Executes the `workspace_rules_add_primary_default_rule_when_unbound` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn workspace_rules_add_primary_default_rule_when_unbound() {
    let config = PersistedConfig {
      primary_monitor: Some("DP-1".into()),
      ..Default::default()
    };
    let mut text = String::new();
    append_workspace_rules(&mut text, &config);
    assert!(
      text.contains("hl.workspace_rule({ workspace = \"1\", monitor = \"DP-1\", default = true })"),
      "{text}"
    );
  }

  use std::sync::Mutex;

  /// Maintains the static state `ENV_LOCK`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  static ENV_LOCK: Mutex<()> = Mutex::new(());

  #[test]
  /// Executes the `state_round_trips_profiles_through_toml` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn state_round_trips_profiles_through_toml() {
    let state = DisplayState {
      primary_monitor: Some("eDP-1".into()),
      active_profile: Some("Trabalho".into()),
      profiles: vec![MonitorProfile {
        name: "Trabalho".into(),
        apply_wallpapers: true,
        config: PersistedConfig {
          primary_monitor: Some("eDP-1".into()),
          monitors: vec![(
            "eDP-1".into(),
            PersistedMonitor {
              scale: Some(1.5),
              vrr: Some(2),
              mirror: Some("DP-1".into()),
              ..Default::default()
            },
          )],
          workspaces: vec![("eDP-1".into(), vec![1, 2, 3])],
        },
      }],
    };
    let dir = std::env::temp_dir().join(format!("argvus-displays-{}", std::process::id()));
    let _guard = ENV_LOCK.lock().unwrap();
    let original = argvus_control_center_core::paths::argvus_config_home();
    // Safety: single-threaded test, no other thread reads ARGVUS_CONFIG_HOME.
    unsafe { std::env::set_var("ARGVUS_CONFIG_HOME", &dir) };
    save_state(&state).unwrap();
    let loaded = load_state();
    unsafe {
      std::env::set_var(
        "ARGVUS_CONFIG_HOME",
        original.to_str().unwrap_or("/tmp/nonexistent"),
      )
    };
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(loaded.active_profile.as_deref(), Some("Trabalho"));
    let profile = loaded.active_profile().unwrap();
    assert!(profile.apply_wallpapers);
    assert_eq!(profile.config.workspaces_of("eDP-1"), vec![1, 2, 3]);
    assert_eq!(profile.config.persisted("eDP-1").vrr, Some(2));
    assert_eq!(
      profile.config.persisted("eDP-1").mirror.as_deref(),
      Some("DP-1")
    );
  }

  #[test]
  /// Only the fields that differ from the canonical layout are written, and a
  /// field that was removed is written as an empty value so it is cleared.
  fn monitor_fields_report_only_changes() {
    let before = PersistedMonitor {
      scale: Some(1.5),
      mode: Some("1920x1080@60".into()),
      ..Default::default()
    };
    let now = PersistedMonitor {
      scale: Some(2.0),
      vrr: Some(1),
      ..Default::default()
    };
    let changed: Vec<_> = monitor_fields(&now, &before)
      .into_iter()
      .filter(|(_, now, then)| now != then)
      .map(|(key, now, _)| (key, now))
      .collect();
    assert_eq!(
      changed,
      vec![
        ("mode", String::new()),
        ("scale", "2".to_string()),
        ("vrr", "1".to_string()),
      ]
    );
  }

  #[test]
  /// The primary monitor belongs to the canonical layout, so the state file
  /// never stores it.
  fn state_file_does_not_store_the_primary_monitor() {
    let dir = std::env::temp_dir().join(format!("argvus-displays-primary-{}", std::process::id()));
    let _guard = ENV_LOCK.lock().unwrap();
    let original = argvus_control_center_core::paths::argvus_config_home();
    // Safety: single-threaded test, no other thread reads ARGVUS_CONFIG_HOME.
    unsafe { std::env::set_var("ARGVUS_CONFIG_HOME", &dir) };
    let state = DisplayState {
      primary_monitor: Some("DP-3".into()),
      ..Default::default()
    };
    save_state(&state).unwrap();
    let stored = fs::read_to_string(state_path()).unwrap();
    assert!(!stored.contains("primary_monitor"), "{stored}");
    unsafe {
      std::env::set_var(
        "ARGVUS_CONFIG_HOME",
        original.to_str().unwrap_or("/tmp/nonexistent"),
      )
    };
    let _ = fs::remove_dir_all(&dir);
  }

  /// Executes the `serialize_config` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn serialize_config(config: &PersistedConfig) -> String {
    let mut text = String::from("-- Generated by ARGVUS Control Center. Do not edit.\n");
    for (name, monitor) in &config.monitors {
      text.push_str(&format!("hl.monitor({{ output = {name:?}"));
      if monitor.disabled == Some(true) {
        text.push_str(", disabled = true");
      } else {
        if let Some(mode) = &monitor.mode {
          text.push_str(&format!(", mode = {mode:?}"));
        }
        if let Some(position) = &monitor.position {
          text.push_str(&format!(", position = {position:?}"));
        }
        if let Some(scale) = monitor.scale {
          text.push_str(&format!(", scale = {scale}"));
        }
        if let Some(transform) = monitor.transform {
          text.push_str(&format!(", transform = {transform}"));
        }
        if let Some(bitdepth) = monitor.bitdepth {
          text.push_str(&format!(", bitdepth = {bitdepth}"));
        }
        if let Some(vrr) = monitor.vrr {
          text.push_str(&format!(", vrr = {vrr}"));
        }
        if let Some(hdr) = monitor.hdr {
          text.push_str(&format!(", supports_hdr = {hdr}"));
        }
      }
      text.push_str(" })\n");
    }
    append_workspace_rules(&mut text, config);
    text
  }
}
