//! Preferences and block scrolling for the ARGVUS telemetry surface.

use argvus_runtime::messages;
use argvus_runtime::process::{program_exists, read_text, run_quiet};
use argvus_widget_telemetry_core::blocks::{self, BLOCK_NAMES, Blocks};
use argvus_widget_telemetry_core::scroll::{self, Layout};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

/// The name the usage message shows for this command.
const PROGRAM: &str = "argvus-widget-telemetry-toggle";
/// The order of the block states in `apply-state`, as the Control Panel sends them.
const APPLY_ORDER: [&str; 8] = [
  "system",
  "cpu_gpu",
  "memory",
  "storage",
  "processes",
  "network",
  "keys",
  "dev_dashboard",
];
/// Runs one command of this crate. `arguments` are the words after the command
/// name. The `argvus` dispatcher and the binary both call this function.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  let Some(command) = arguments.first().map(String::as_str) else {
    return usage();
  };
  let rest = &arguments[1..];
  match command {
    "apply-state" => apply_state(rest),
    "scroll" => scroll_command(rest),
    "visible" => visible(),
    "layout" => layout_command(),
    "on" => on(),
    "off" => off(),
    "status" => status(),
    "migrate-master" => migrate_master(),
    "master-status" => {
      print_state(&master_state_value());
      ExitCode::SUCCESS
    }
    "toggle" => toggle(),
    "blocks" => blocks_command(rest),
    _ => usage(),
  }
}

fn usage() -> ExitCode {
  let text = messages::text(
    "cli.usage",
    &[
      format!("command={PROGRAM}"),
      "options={on|off|toggle|status|blocks|scroll}".to_string(),
    ],
  );
  eprintln!("{text}");
  ExitCode::from(1)
}

// -- Paths ---------------------------------------------------------------------

fn config_home() -> PathBuf {
  if let Some(home) = env::var_os("ARGVUS_CONFIG_HOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(home);
  }
  env::var_os("XDG_CONFIG_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"))
}

fn cache_home() -> PathBuf {
  if let Some(home) = env::var_os("ARGVUS_CACHE_HOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(home);
  }
  let base = env::var_os("XDG_CACHE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"));
  base.join("argvus")
}

fn state_home() -> PathBuf {
  if let Some(home) = env::var_os("ARGVUS_STATE_HOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(home);
  }
  if let Some(data) = env::var_os("ARGVUS_DATA_HOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(data).join("state");
  }
  config_home().join("argvus/data/state")
}

/// The state file of the widget master switch.
fn state_file() -> PathBuf {
  cache_home().join("taskbar/widget-telemetry-state")
}

/// The state files of the earlier widget names, read when the new one is missing.
fn legacy_state_files() -> [PathBuf; 2] {
  [
    cache_home().join("taskbar/desktop-telemetry-state"),
    cache_home().join("taskbar/sysinfo-state"),
  ]
}

fn blocks_file() -> PathBuf {
  state_home().join("widget-telemetry-blocks")
}

fn scroll_file() -> PathBuf {
  cache_home().join("taskbar/widget-telemetry-scroll")
}

// -- Files and programs --------------------------------------------------------

/// Writes a file through a temporary file in the same directory, so the file
/// is either the old or the new content.
fn write_atomic(path: &Path, content: &str) -> bool {
  let Some(parent) = path.parent() else {
    return false;
  };
  if fs::create_dir_all(parent).is_err() {
    return false;
  }
  let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
  if fs::write(&temporary, content).is_err() {
    return false;
  }
  fs::rename(&temporary, path).is_ok()
}

/// Runs `argvus config get POINTER` and returns its text, or empty on failure.
fn config_get(pointer: &str, effective: bool) -> String {
  let mut command = Command::new("argvus");
  command.args(["config", "get", pointer, "--raw"]);
  if effective {
    command.arg("--effective");
  }
  command
    .stderr(Stdio::null())
    .output()
    .ok()
    .filter(|output| output.status.success())
    .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
    .unwrap_or_default()
}

/// The master switch in argvus-config, when `argvus` is installed and the value
/// is one the switch knows.
fn canonical_master_state(effective: bool) -> Option<&'static str> {
  if !program_exists("argvus") {
    return None;
  }
  match config_get("/effects/widget_telemetry_enabled", effective).as_str() {
    "true" | "enabled" => Some("enabled"),
    "false" | "disabled" => Some("disabled"),
    _ => None,
  }
}

/// Stores the master switch in argvus-config and reloads it. Returns false when
/// the value was not stored or the reload failed.
fn set_canonical_master(enabled: bool) -> bool {
  if !program_exists("argvus") {
    return true;
  }
  let value = if enabled { "true" } else { "false" };
  if !run_quiet(
    "argvus",
    &["config", "set", "/effects/widget_telemetry_enabled", value],
  ) {
    return false;
  }
  if env::var("ARGVUS_CONFIG_SERVICE").as_deref() != Ok("1")
    && env::var("ARGVUS_PROJECTING").as_deref() != Ok("1")
  {
    return run_quiet("systemctl", &["--user", "reload", "argvus-config.service"]);
  }
  true
}

fn legacy_master_state() -> Option<&'static str> {
  let mut candidates = vec![state_file()];
  candidates.extend(legacy_state_files());
  candidates.into_iter().find_map(|path| {
    let contents = fs::read_to_string(path).ok()?;
    match contents.lines().next() {
      Some("enabled") => Some("enabled"),
      Some("disabled") => Some("disabled"),
      _ => None,
    }
  })
}

fn master_state_value() -> String {
  canonical_master_state(false)
    .or_else(legacy_master_state)
    .or_else(|| canonical_master_state(true))
    .unwrap_or("enabled")
    .to_string()
}

fn migrate_master() -> ExitCode {
  if canonical_master_state(false).is_some() {
    return ExitCode::SUCCESS;
  }
  let Some(state) = legacy_master_state() else {
    return ExitCode::SUCCESS;
  };
  let value = if state == "enabled" { "true" } else { "false" };
  if run_quiet(
    "argvus",
    &["config", "set", "/effects/widget_telemetry_enabled", value],
  ) {
    ExitCode::SUCCESS
  } else {
    ExitCode::FAILURE
  }
}

fn any_block_enabled(blocks: &Blocks) -> bool {
  BLOCK_NAMES.iter().any(|block| blocks.enabled(block))
}

fn effective_state(blocks: &Blocks) -> &'static str {
  if master_state_value() == "enabled" && any_block_enabled(blocks) {
    "enabled"
  } else {
    "disabled"
  }
}

fn sync_widget_service(blocks: &Blocks) -> bool {
  let verb = if effective_state(blocks) == "enabled" {
    "restart"
  } else {
    "stop"
  };
  run_quiet(
    "systemctl",
    &["--user", verb, "argvus-widget-telemetry.service"],
  )
}

fn refresh_widget() {
  let _ = run_quiet(
    "argvus",
    &["sessionctl", "bars-ipc", "telemetry", "refresh"],
  );
}

fn load_blocks() -> Blocks {
  Blocks::parse(&read_text(&blocks_file()).unwrap_or_default())
}

// -- Commands ------------------------------------------------------------------

fn on() -> ExitCode {
  set_master(true)
}

fn off() -> ExitCode {
  set_master(false)
}

fn set_master(enabled: bool) -> ExitCode {
  if !set_canonical_master(enabled) {
    return ExitCode::FAILURE;
  }
  if enabled {
    reset_scroll();
  }
  let blocks = load_blocks();
  if !sync_widget_service(&blocks) {
    return ExitCode::FAILURE;
  }
  print_state(effective_state(&blocks));
  ExitCode::SUCCESS
}

/// The state word of `on`, `status` and the toggle. The machine output is the
/// bare word; the others use the translated name.
fn print_state(state: &str) {
  if env::var("ARGVUS_MACHINE_OUTPUT").as_deref() == Ok("1") {
    println!("{state}");
  } else {
    let key = if state == "enabled" {
      "state.enabled"
    } else {
      "state.disabled"
    };
    println!("{}", messages::text(key, &[]));
  }
}

fn status() -> ExitCode {
  let blocks = load_blocks();
  print_state(effective_state(&blocks));
  ExitCode::SUCCESS
}

fn toggle() -> ExitCode {
  let blocks = load_blocks();
  if effective_state(&blocks) == "enabled" {
    off()
  } else {
    on()
  }
}

/// `blocks status|apply|set BLOCK enabled|disabled`.
fn blocks_command(arguments: &[String]) -> ExitCode {
  match arguments.first().map(String::as_str).unwrap_or("status") {
    "status" => {
      blocks_status();
      ExitCode::SUCCESS
    }
    "apply" => {
      refresh_widget();
      ExitCode::SUCCESS
    }
    "set" => set_block(
      arguments.get(1).map(String::as_str).unwrap_or(""),
      arguments.get(2).map(String::as_str).unwrap_or(""),
    ),
    _ => {
      eprintln!("usage: {PROGRAM} blocks {{status|apply|set <block> <enabled|disabled>}}");
      ExitCode::from(2)
    }
  }
}

fn blocks_status() {
  let blocks = load_blocks();
  println!("__order__={}", blocks.order().join(","));
  for block in BLOCK_NAMES {
    println!("{block}={}", blocks::state_name(blocks.enabled(block)));
  }
}

fn set_block(block: &str, value: &str) -> ExitCode {
  if !blocks::is_block(block) {
    eprintln!(
      "invalid telemetry block: {}",
      if block.is_empty() { "empty" } else { block }
    );
    return ExitCode::from(2);
  }
  let enabled = match blocks::parse_state(value) {
    Some(enabled) => enabled,
    None => {
      eprintln!(
        "invalid telemetry block state: {}",
        if value.is_empty() { "empty" } else { value }
      );
      return ExitCode::from(2);
    }
  };
  let blocks = load_blocks();
  let order = blocks.order().join(",");
  let states: Vec<(&str, bool)> = BLOCK_NAMES
    .iter()
    .map(|name| {
      (
        *name,
        if *name == block {
          enabled
        } else {
          blocks.enabled(name)
        },
      )
    })
    .collect();
  if !write_atomic(&blocks_file(), &blocks::render(&order, &states)) {
    return ExitCode::FAILURE;
  }
  apply_block_change()
}

/// `apply-state MASTER SYSTEM CPU_GPU MEMORY STORAGE PROCESSES NETWORK KEYS
/// DEV_DASHBOARD ORDER`: writes the whole state at once, as the Control Panel
/// does when it applies its preferences.
fn apply_state(arguments: &[String]) -> ExitCode {
  let Some((master, rest)) = arguments.split_first() else {
    return ExitCode::from(2);
  };
  if master != "enabled" && master != "disabled" {
    return ExitCode::from(2);
  }
  let [states @ .., order] = rest else {
    return ExitCode::from(2);
  };
  if states.len() != BLOCK_NAMES.len() {
    return ExitCode::from(2);
  }
  let mut parsed = Vec::new();
  for value in states {
    match blocks::parse_state(value) {
      Some(enabled) => parsed.push(enabled),
      None => return ExitCode::from(2),
    }
  }
  if !blocks::valid_order(order) {
    return ExitCode::from(2);
  }
  if master_state_value() != *master && !set_canonical_master(master == "enabled") {
    return ExitCode::FAILURE;
  }
  let block_states: Vec<(&str, bool)> = APPLY_ORDER.iter().copied().zip(parsed).collect();
  if !write_atomic(
    &blocks_file(),
    &blocks::render_in_order(order, &block_states),
  ) {
    return ExitCode::FAILURE;
  }
  apply_block_change()
}

fn apply_block_change() -> ExitCode {
  reset_scroll();
  if sync_widget_service(&load_blocks()) {
    ExitCode::SUCCESS
  } else {
    ExitCode::FAILURE
  }
}

// -- Scroll window -------------------------------------------------------------

/// The focused monitor's usable height in pixels, or `None` when Hyprland cannot
/// answer. The height is the monitor height divided by its scale, minus the
/// reserved space at the top and at the bottom.
fn monitor_budget() -> Option<u64> {
  let output = Command::new("hyprctl")
    .args(["monitors", "-j"])
    .stderr(Stdio::null())
    .output()
    .ok()?;
  let monitors: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
  let focused = monitors
    .as_array()?
    .iter()
    .find(|monitor| monitor.get("focused").and_then(serde_json::Value::as_bool) == Some(true))?;
  let height = focused.get("height")?.as_f64()?;
  let scale = focused.get("scale")?.as_f64()?;
  let reserved = focused.get("reserved")?.as_array()?;
  let top = reserved.first()?.as_f64()?;
  let bottom = reserved.get(2)?.as_f64()?;
  let budget = (height / scale - top - bottom).floor();
  scroll::parse_budget(Some(&format!("{budget:.0}")))
}

fn reset_scroll() {
  write_scroll_offset(0);
}

fn write_scroll_offset(offset: usize) {
  let _ = write_atomic(&scroll_file(), &format!("{offset}\n"));
}

/// The window of the enabled blocks, in display order, with the stored offset.
fn current_layout() -> Layout {
  let blocks = load_blocks();
  let enabled: Vec<String> = blocks
    .order()
    .into_iter()
    .filter(|block| blocks.enabled(block))
    .collect();
  let stored = scroll::parse_offset(read_text(&scroll_file()).as_deref());
  scroll::layout(&enabled, monitor_budget(), stored)
}

fn visible() -> ExitCode {
  println!("{}", current_layout().visible.join(" "));
  ExitCode::SUCCESS
}

fn layout_command() -> ExitCode {
  let layout = current_layout();
  println!("visible={}", layout.visible.join(" "));
  println!("first={}", layout.first.unwrap_or_default());
  println!("last={}", layout.last.unwrap_or_default());
  println!("offset={}", layout.offset);
  println!("max={}", layout.max);
  println!("above={}", u8::from(layout.above));
  println!("below={}", u8::from(layout.below));
  ExitCode::SUCCESS
}

/// `scroll up|down`: moves the window by one block, within its limits.
fn scroll_command(arguments: &[String]) -> ExitCode {
  let layout = current_layout();
  let offset = layout.offset as i64;
  let target = match arguments.first().map(String::as_str) {
    Some("up") => offset - 1,
    Some("down") => offset + 1,
    _ => {
      eprintln!("usage: {PROGRAM} scroll {{up|down}}");
      return ExitCode::from(2);
    }
  };
  let target = target.clamp(0, layout.max as i64);
  write_scroll_offset(target as usize);
  refresh_widget();
  ExitCode::SUCCESS
}
