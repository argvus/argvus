//! Isolated integration with system tools and APIs of the Displays page.
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

fn parse_mode_string(mode: &str) -> Option<Mode> {
  let mode = mode.trim_end_matches("Hz");
  let (resolution, rate_str) = mode.split_once('@')?;
  let (w_str, h_str) = resolution.split_once('x')?;
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

pub fn set_dpms(name: &str, on: bool) -> Result<(), String> {
  run_hyprctl(&["dispatch", "dpms", if on { "on" } else { "off" }, name])
}

/// Applies the full persisted configuration: writes the generated Lua and
/// triggers a Hyprland config reload so all monitor rules take effect.
pub fn apply_all(config: &PersistedConfig) -> Result<(), String> {
  save_config(config)?;
  run_hyprctl(&["reload"])
}

fn config_path() -> PathBuf {
  argvus_data_home()
    .join("generated")
    .join("hypr")
    .join("monitors.lua")
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredState {
  #[serde(skip_serializing_if = "Option::is_none")]
  primary_monitor: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  active_profile: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  profiles: Vec<StoredProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
struct StoredWorkspace {
  monitor: String,
  #[serde(default)]
  ids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
mod tests;
