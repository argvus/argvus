//! The display command, `argvus config display`.
//!
//! This replaces the display script. The layout is the canonical `/displays`
//! state, changed through `monitor_state`, so the Hyprland file is projected
//! from one place. The runtime (`hyprctl` queries, DPMS, disabled monitors and
//! the reload) goes through the `argvus config hyprland` routes. nwg-displays
//! stays an external program: its `monitors.conf` is read here.

use crate::ConfigStore;
use crate::i18n::{self, Stream};
use crate::json_pointers;
use crate::monitor_state;
use crate::session_paths::SessionPaths;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The `display` domain of argvus-i18n.
const DOMAIN: &str = "display";

/// Runs `argvus config display SUBCOMMAND ...`. The exit code is 0 on success,
/// 1 when the change is refused, and 2 for a usage error.
pub fn run(arguments: &[String]) -> i32 {
  let store = match ConfigStore::from_environment() {
    Ok(store) => store,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return 1;
    }
  };
  let Some(subcommand) = arguments.first() else {
    return usage();
  };
  let rest = &arguments[1..];
  match (subcommand.as_str(), rest) {
    ("status", []) => status(),
    ("night-light", arguments) => crate::night_light::run(&store, arguments),
    ("list-modes", [monitor]) => list_modes(monitor),
    ("set", [monitor, key, value]) => set(&store, monitor, key, value),
    ("apply" | "session-reload", []) => apply(&store, ApplyMode::Full),
    ("session-prepare", []) => apply(&store, ApplyMode::SessionPrepare),
    ("apply-nwg", []) => apply_nwg(),
    ("migrate-nwg", []) => migrate_nwg(&store),
    ("nwg-import", [file]) => nwg_import(&store, Path::new(file)),
    ("nwg-apply", [file]) => nwg_runtime(Path::new(file)),
    ("list-modes" | "set" | "nwg-import" | "nwg-apply", _) => {
      i18n::print_in(
        DOMAIN,
        "error.missing_monitor_key_value",
        &[],
        Stream::Stderr,
      );
      2
    }
    _ => usage(),
  }
}

fn usage() -> i32 {
  i18n::print_in(DOMAIN, "cli.usage", &[], Stream::Stderr);
  i18n::print_in(DOMAIN, "cli.keys", &[], Stream::Stderr);
  2
}

/// The monitors reported by Hyprland, or an error when `hyprctl` is missing.
/// A reply that is not JSON reads as no monitors, as the script did.
fn hyprctl_monitors() -> Result<Vec<Value>, i32> {
  let output = match Command::new("hyprctl")
    .args(["monitors", "-j"])
    .stderr(Stdio::null())
    .output()
  {
    Ok(output) => output,
    Err(_) => {
      i18n::print_in(DOMAIN, "error.hyprctl_missing", &[], Stream::Stderr);
      return Err(1);
    }
  };
  Ok(
    serde_json::from_slice::<Value>(&output.stdout)
      .ok()
      .and_then(|value| value.as_array().cloned())
      .unwrap_or_default(),
  )
}

/// One block per monitor: name, resolution and refresh, position, scale, DPI
/// and power, one `key=value` line each.
fn status() -> i32 {
  let monitors = match hyprctl_monitors() {
    Ok(monitors) => monitors,
    Err(code) => return code,
  };
  for monitor in &monitors {
    let number =
      |key: &str, default: f64| monitor.get(key).and_then(Value::as_f64).unwrap_or(default);
    let text = |key: &str| {
      monitor
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
    };
    let powered = monitor
      .get("dpmsStatus")
      .and_then(Value::as_bool)
      .unwrap_or(false);
    println!("name={}", text("name"));
    println!(
      "res={}x{}@{}",
      number("width", 0.0) as i64,
      number("height", 0.0) as i64,
      number("refreshRate", 0.0) as i64
    );
    println!(
      "pos={},{}",
      number("x", 0.0) as i64,
      number("y", 0.0) as i64
    );
    println!(
      "scale={}",
      monitor
        .get("scale")
        .map_or("1".to_string(), |value| value.to_string())
    );
    println!("dpi={}", number("dpi", 96.0) as i64);
    println!("power={}", if powered { "on" } else { "off" });
  }
  0
}

/// The modes of one monitor as `WxH@R`, one per line.
fn list_modes(monitor: &str) -> i32 {
  let monitors = match hyprctl_monitors() {
    Ok(monitors) => monitors,
    Err(code) => return code,
  };
  let Some(found) = monitors
    .iter()
    .find(|item| item.get("name").and_then(Value::as_str) == Some(monitor))
  else {
    return 0;
  };
  for mode in found
    .get("modes")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
  {
    // Hyprland reports the refresh rate as a float; the script printed it
    // as an integer, so it is truncated the same way.
    let width = mode.get("width").and_then(Value::as_f64).unwrap_or(0.0) as i64;
    let height = mode.get("height").and_then(Value::as_f64).unwrap_or(0.0) as i64;
    let refresh = mode
      .get("refreshRate")
      .and_then(Value::as_f64)
      .unwrap_or(0.0) as i64;
    if width > 0 && height > 0 && refresh > 0 {
      println!("{width}x{height}@{refresh}");
    }
  }
  0
}

/// Sets one value: the canonical layout is changed and projected, and the
/// running session follows where it can. The message names the change.
fn set(store: &ConfigStore, monitor: &str, key: &str, value: &str) -> i32 {
  if monitor_state::validate(key, value).is_err() {
    let message = match key {
      "scale" => "error.invalid_scale",
      "resolution" => "error.invalid_resolution",
      "refresh" => "error.invalid_refresh",
      "position" => "error.invalid_position",
      "rotation" => "error.invalid_rotation",
      "power" => "error.invalid_power",
      "enabled" => "error.invalid_enabled",
      _ => "error.invalid_key",
    };
    i18n::print_in(
      DOMAIN,
      message,
      &[format!("value={value}"), format!("key={key}")],
      Stream::Stderr,
    );
    return 1;
  }
  let monitors = match hyprctl_monitors() {
    Ok(monitors) => monitors,
    Err(code) => return code,
  };
  let exists = monitors
    .iter()
    .any(|item| item.get("name").and_then(Value::as_str) == Some(monitor));
  if !exists {
    i18n::print_in(
      DOMAIN,
      "error.monitor_not_found",
      &[format!("monitor={monitor}")],
      Stream::Stderr,
    );
    return 1;
  }
  // The runtime follows the change first, as the script did for the scale
  // and the power state.
  if key == "scale" {
    run_runtime(&[
      "hyprland",
      "monitor",
      &format!("{monitor},preferred,auto,{value}"),
    ]);
  }
  if key == "power" {
    run_runtime(&["hyprland", "dpms", monitor, value]);
  }
  if let Err(error) = monitor_state::set(store, monitor, key, value) {
    eprintln!("argvus-config: {error}");
    return 1;
  }
  reload_service();
  i18n::print_in(
    DOMAIN,
    "status.applied",
    &[
      format!("key={key}"),
      format!("value={value}"),
      format!("monitor={monitor}"),
    ],
    Stream::Stdout,
  );
  0
}

/// Applies the canonical layout to the running session: the legacy state is
/// migrated when the layout has no monitor yet, disabled monitors and powered
/// down monitors are applied, and Hyprland reloads its configuration.
/// How much of the display state an apply touches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApplyMode {
  /// A user or reload request: project the store, apply the runtime state and
  /// reload the compositor.
  Full,
  /// Session preparation: `argvus-config.service` has just projected the store
  /// and Hyprland has just loaded the generated monitors, so only the state that
  /// lives outside the configuration (DPMS, disabled outputs) is applied.
  SessionPrepare,
}

fn apply(store: &ConfigStore, mode: ApplyMode) -> i32 {
  let legacy = legacy_state_file();
  let synchronized = match mode {
    ApplyMode::Full => monitor_state::sync_legacy(store, &legacy),
    ApplyMode::SessionPrepare => monitor_state::import_legacy_if_needed(store, &legacy).map(|_| ()),
  };
  if let Err(error) = synchronized {
    eprintln!("argvus-config: {error}");
    return 1;
  }
  let Ok(document) = store.load() else {
    return 1;
  };
  let monitors = document
    .get(json_pointers::DISPLAYS_MONITORS)
    .and_then(Value::as_object)
    .cloned()
    .unwrap_or_default();
  if monitors.is_empty() {
    return 0;
  }
  {
    for (name, monitor) in &monitors {
      if monitor.get("dpms").and_then(Value::as_bool) == Some(false) {
        run_runtime(&["hyprland", "dpms", name, "off"]);
      }
      if monitor.get("disabled").and_then(Value::as_bool) == Some(true) {
        run_runtime(&["hyprland", "monitor", &format!("{name},disabled")]);
      }
    }
  }
  if mode == ApplyMode::Full {
    run_runtime(&["hyprland", "reload"]);
  }
  0
}

/// The legacy monitor state file, read only to migrate an installation that
/// predates the canonical layout.
fn legacy_state_file() -> PathBuf {
  env::var_os("ARGVUS_DISPLAY_STATE_FILE")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      SessionPaths::from_environment()
        .map(|paths| paths.config_home.join("argvus/data/.monitors"))
        .unwrap_or_default()
    })
}

/// The monitors of the user's own Hyprland file, applied at runtime. This is
/// the layout of a session that still has the old file.
fn apply_nwg() -> i32 {
  let Some(home) = env::var_os("HOME") else {
    return 0;
  };
  let legacy = PathBuf::from(home).join(".config/hypr/monitors.conf");
  if !legacy.is_file() {
    return 0;
  }
  let code = nwg_runtime(&legacy);
  notify("notification.legacy_applied");
  code
}

/// Moves the user's own Hyprland monitor file into the canonical layout when
/// the layout has no monitor yet.
fn migrate_nwg(store: &ConfigStore) -> i32 {
  let Some(home) = env::var_os("HOME") else {
    return 0;
  };
  let legacy = PathBuf::from(home).join(".config/hypr/monitors.conf");
  if !legacy.is_file() {
    return 0;
  }
  let has_layout = store
    .load()
    .ok()
    .and_then(|document| document.get(json_pointers::DISPLAYS_MONITORS).cloned())
    .and_then(|monitors| monitors.as_object().map(|object| !object.is_empty()))
    .unwrap_or(false);
  if has_layout {
    return 0;
  }
  match nwg_import(store, &legacy) {
    0 => {
      notify("notification.migrated");
      0
    }
    code => code,
  }
}

/// Imports an nwg-displays `monitors.conf` into the canonical layout, then
/// projects it.
fn nwg_import(store: &ConfigStore, file: &Path) -> i32 {
  let text = match fs::read_to_string(file) {
    Ok(text) => text,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return 1;
    }
  };
  let lines = nwg_state_lines(&text);
  if lines.is_empty() {
    return 0;
  }
  match monitor_state::import_lines(store, &lines) {
    Ok(()) => {
      reload_service();
      0
    }
    Err(error) => {
      eprintln!("argvus-config: {error}");
      1
    }
  }
}

/// Applies the `monitor=` lines of an nwg-displays file to the running session.
fn nwg_runtime(file: &Path) -> i32 {
  let Ok(text) = fs::read_to_string(file) else {
    return 0;
  };
  for argument in nwg_monitor_arguments(&text) {
    run_runtime(&["hyprland", "monitor", &argument]);
  }
  0
}

/// The arguments of each `monitor=` line, without comments or spacing.
fn nwg_monitor_arguments(text: &str) -> Vec<String> {
  text
    .lines()
    .filter_map(|line| {
      let line = strip_comment(line).trim();
      let argument = line
        .strip_prefix("monitor=")
        .or_else(|| line.strip_prefix("monitor = "))?;
      let argument = argument.trim();
      (!argument.is_empty()).then(|| argument.to_string())
    })
    .collect()
}

fn strip_comment(line: &str) -> &str {
  line.split('#').next().unwrap_or("")
}

/// Translates an nwg-displays `monitors.conf` into `monitor.key=value` lines.
/// This keeps the rules of the former parser, including its output order, so
/// the same file gives the same layout.
pub fn nwg_state_lines(text: &str) -> Vec<String> {
  let mut lines = Vec::new();
  for raw in text.lines() {
    let line = raw.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let Some(argument) = line
      .strip_prefix("monitor=")
      .or_else(|| line.strip_prefix("monitor = "))
    else {
      continue;
    };
    let (output, args) = match argument.split_once(',') {
      Some((output, args)) => (output.trim(), args),
      None => (argument.trim(), argument),
    };
    if output.is_empty() {
      continue;
    }
    if matches!(args.trim(), "disable" | "disabled") {
      lines.push(format!("{output}.enabled=false"));
      continue;
    }
    let fields: Vec<&str> = args.split(',').map(str::trim).collect();
    if args.starts_with("transform,") {
      if let Some(degrees) = transform_degrees(fields.get(1).copied().unwrap_or("")) {
        lines.push(format!("{output}.rotation={degrees}"));
      }
      continue;
    }
    if args.starts_with("mirror,") {
      let mirror = fields.get(1).copied().unwrap_or("");
      if !mirror.is_empty() {
        lines.push(format!("{output}.mirror={mirror}"));
      }
      continue;
    }
    if args.trim() == "bitdepth,10" {
      lines.push(format!("{output}.bitdepth=10"));
      continue;
    }
    let mode = fields.first().copied().unwrap_or("");
    let position = fields.get(1).copied().unwrap_or("");
    let scale = fields.get(2).copied().unwrap_or("");
    let inline_transform = fields.get(3).copied().unwrap_or("");
    match mode {
      "disable" | "disabled" => lines.push(format!("{output}.enabled=false")),
      "preferred" | "auto" => {}
      _ => {
        let (width, rest) = mode.split_once('x').unwrap_or((mode, ""));
        let (height, refresh) = rest.split_once('@').unwrap_or((rest, ""));
        let refresh = refresh.trim_end_matches("Hz");
        if !width.is_empty() && !height.is_empty() {
          lines.push(format!("{output}.resolution={width}x{height}"));
        }
        if !refresh.is_empty() {
          lines.push(format!("{output}.refresh={refresh}"));
        }
      }
    }
    if !position.is_empty() && position != "auto" && position != "0x0" {
      lines.push(format!("{output}.position={position}"));
    }
    if !scale.is_empty() {
      lines.push(format!("{output}.scale={scale}"));
    }
    // An inline transform of 0 or normal is no rotation at all.
    if let Some(degrees) = transform_degrees(inline_transform).filter(|degrees| *degrees != "0") {
      lines.push(format!("{output}.rotation={degrees}"));
    }
  }
  lines
}

/// The rotation in degrees of a transform code, as nwg-displays writes it.
fn transform_degrees(code: &str) -> Option<&'static str> {
  match code {
    "0" | "normal" => Some("0"),
    "1" | "90" => Some("90"),
    "2" | "180" => Some("180"),
    "3" | "270" => Some("270"),
    _ => None,
  }
}

/// Runs one `argvus config` route of the runtime. A failure is not an error of
/// the change itself, as the script ignored it.
fn run_runtime(arguments: &[&str]) {
  let _ = Command::new("argvus")
    .arg("config")
    .args(arguments)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

/// Reloads argvus-config so the canonical change reaches the running session,
/// unless a projection is already running.
fn reload_service() {
  if env::var("ARGVUS_CONFIG_SERVICE").as_deref() == Ok("1")
    || env::var("ARGVUS_PROJECTING").as_deref() == Ok("1")
  {
    return;
  }
  let _ = Command::new("systemctl")
    .args(["--user", "reload", "argvus-config.service"])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

fn notify(key: &str) {
  let title = i18n::text_in(DOMAIN, "notification.title", &[]);
  let message = i18n::text_in(DOMAIN, key, &[]);
  let _ = Command::new("notify-send")
    .arg(title)
    .arg(message)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

/// Runs one `argvus config display` subcommand as a session step. The session
/// steps ignore its output and its failure, as the shell steps did.
pub fn run_step(subcommand: &str, environment: &[(String, String)]) {
  let mut command = std::process::Command::new("argvus");
  command.args(["config", "display", subcommand]);
  for (name, value) in environment {
    command.env(name, value);
  }
  let _ = command
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status();
}

#[cfg(test)]
mod tests {
  use super::{nwg_monitor_arguments, nwg_state_lines};

  #[test]
  fn a_full_nwg_line_becomes_the_state_fields() {
    let lines = nwg_state_lines("monitor=DP-1,1920x1080@60,1920x0,1.5\n");
    assert_eq!(
      lines,
      vec![
        "DP-1.resolution=1920x1080",
        "DP-1.refresh=60",
        "DP-1.position=1920x0",
        "DP-1.scale=1.5",
      ]
    );
  }

  #[test]
  fn disabled_and_transform_lines_keep_their_meaning() {
    assert_eq!(
      nwg_state_lines("monitor=HDMI-A-1,disable\n"),
      vec!["HDMI-A-1.enabled=false"]
    );
    assert_eq!(
      nwg_state_lines("monitor=DP-1,transform,1\n"),
      vec!["DP-1.rotation=90"]
    );
    assert_eq!(
      nwg_state_lines("monitor=DP-1,1920x1080@59.94Hz,0x0,1,1\n"),
      vec![
        "DP-1.resolution=1920x1080",
        "DP-1.refresh=59.94",
        "DP-1.scale=1",
        "DP-1.rotation=90",
      ]
    );
  }

  #[test]
  fn mirror_bitdepth_and_automatic_modes_are_read() {
    assert_eq!(
      nwg_state_lines("monitor=DP-1,mirror,DP-2\nmonitor=DP-2,bitdepth,10\n"),
      vec!["DP-1.mirror=DP-2", "DP-2.bitdepth=10"]
    );
    assert_eq!(
      nwg_state_lines("monitor=DP-2,bitdepth,10\n"),
      vec!["DP-2.bitdepth=10"]
    );
    assert!(
      nwg_state_lines("monitor=DP-3,preferred,auto,1\n").contains(&"DP-3.scale=1".to_string())
    );
  }

  #[test]
  fn comments_and_other_lines_are_ignored() {
    assert!(nwg_state_lines("# monitor=DP-1\nsomething=else\n").is_empty());
  }

  #[test]
  fn runtime_arguments_skip_comments_and_spacing() {
    assert_eq!(
      nwg_monitor_arguments("monitor = DP-1, 1920x1080@60, 0x0, 1 # main\n"),
      vec!["DP-1, 1920x1080@60, 0x0, 1"]
    );
  }
}
