//! Implements isolated integration with system tools and APIs in crate `argvus control center appearance`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::animations::AnimationValues;
use crate::blur::BlurValues;
use crate::model::{
  AppearanceState, ControlPanelCard, ControlPanelCards, EffectSurface, TASKBAR_BORDER_THICKNESSES,
  TASKBAR_HEIGHTS, TaskbarDateFormat, TaskbarPosition, TaskbarTimeFormat, TaskbarUtilityGroupMode,
  TaskbarUtilityWidget, TaskbarUtilityWidgets, WallpaperCollection, WallpaperEntry, WallpaperMode,
  WidgetTelemetryBlock, WidgetTelemetryBlocks, canonical_theme_id, normalize_hex_color,
};
use argvus_control_center_core::{
  config::reload_argvus_config_service,
  paths::{argvus_data_home, cache_home, system_config_root},
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use crate::profile::{
  active_custom_theme, apply_custom_theme, custom_themes, delete_custom_theme,
  export_named as export_theme_profile, import_archive as import_theme_profile,
  inspect_archive as inspect_theme_profile, list_import_archives,
  preview_export_path as preview_theme_profile_path,
};

/// Defines the constant `WALLPAPERS_DIR`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const WALLPAPERS_DIR: &str = "/usr/share/backgrounds/argvus";
/// Defines the constant `DEFAULT_THEME`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const DEFAULT_THEME: &str = "argvus-dark";
/// Defines the constant `DEFAULT_ACCENT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const DEFAULT_ACCENT: &str = "#3590bd";

/// Executes the `script` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn script(name: &str) -> PathBuf {
  let project = match name {
    "hypr-wallpaper-pick.sh"
    | "taskbar-right-2-mode.sh"
    | "taskbar-widgets-mode.sh"
    | "brightness-switch.sh"
    | "layout-mode-switch.sh"
    | "icon-switch.sh" => "appearance",
    "hyprlock-theme.sh" => "lock",
    "spaces-switch.sh" | "borders-switch.sh" => "hyprland",
    _ => "session",
  };
  system_config_root().join(project).join("sh").join(name)
}

/// Returns the Control Panel-owned preference helper without duplicating its
/// persistence rules in the Control Center.
fn control_panel_cards_script() -> PathBuf {
  system_config_root()
    .join("control-panel")
    .join("sh")
    .join("cards-config.sh")
}

/// Executes the `run_script` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_script(script_path: &Path, args: &[&str]) -> Result<(), String> {
  run_script_with_env(script_path, args, &[])
}

fn run_script_with_env(
  script_path: &Path,
  args: &[&str],
  environment: &[(&str, &str)],
) -> Result<(), String> {
  if !script_path.is_file() {
    return Err(format!("script não encontrado: {}", script_path.display()));
  }
  let request = ProcessRequest::new("sh").arg(script_path.to_string_lossy().to_string());
  run_process(
    request,
    &script_path.display().to_string(),
    args,
    environment,
  )
}

/// Runs an `argvus <route>` command: the same entry point the keybindings and
/// the menus use, so the route alone decides the implementation.
pub(crate) fn run_argvus_route(
  route: &str,
  args: &[&str],
  environment: &[(&str, &str)],
) -> Result<(), String> {
  let request = ProcessRequest::new("argvus").arg(route);
  run_process(request, &format!("argvus {route}"), args, environment)
}

/// Runs `argvus effects-toggle` with the arguments: the visual effects route.
fn run_effects(arguments: &[&str]) -> Result<(), String> {
  run_argvus_route("effects-toggle", arguments, &[])
}

/// Reads from `argvus effects-toggle`, falling back to `None` when it fails.
fn run_effects_output(arguments: &[&str]) -> Option<String> {
  run_argvus_route_output("effects-toggle", arguments)
}

/// Validates the arguments, applies the environment, runs the command and
/// reports its stderr when it fails.
fn run_process(
  mut request: ProcessRequest,
  label: &str,
  args: &[&str],
  environment: &[(&str, &str)],
) -> Result<(), String> {
  for argument in args {
    if argument.is_empty()
      || argument
        .chars()
        .any(|character| character.is_control() || character == '\n')
    {
      return Err("invalid script argument".into());
    }
    request = request.arg(*argument);
  }
  for (name, value) in environment {
    request = request.env(*name, *value);
  }
  let output = SystemProcessRunner
    .run(&request)
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr))
      .trim()
      .to_string();
    return Err(if stderr.is_empty() {
      format!("{label} falhou")
    } else {
      stderr
    });
  }
  Ok(())
}

pub(crate) fn run_profile_script_with_env(
  name: &str,
  args: &[&str],
  environment: &[(&str, &str)],
) -> Result<(), String> {
  if name == "argvus-widget-telemetry-toggle" {
    let mut request = ProcessRequest::new("argvus").arg("widget-telemetry-toggle");
    for argument in args {
      request = request.arg(*argument);
    }
    for (name, value) in environment {
      request = request.env(*name, *value);
    }
    let output = SystemProcessRunner
      .run(&request)
      .map_err(|error| error.to_string())?;
    return (output.status == Some(0))
      .then_some(())
      .ok_or_else(|| "telemetry apply failed".into());
  }
  let path = script(name);
  run_script_with_env(&path, args, environment)
}

/// Executes the `run_script_output` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_script_output(script_path: &Path, args: &[&str]) -> Option<String> {
  if !script_path.is_file() {
    return None;
  }
  run_output(
    ProcessRequest::new("sh").arg(script_path.to_string_lossy().to_string()),
    args,
  )
}

/// Runs an `argvus <route>` command and returns its stdout, like `run_script_output`.
fn run_argvus_route_output(route: &str, args: &[&str]) -> Option<String> {
  run_output(ProcessRequest::new("argvus").arg(route), args)
}

/// Runs the prepared request with validated arguments. A failed run or an
/// invalid argument gives `None`, so readers fall back to their defaults.
fn run_output(mut request: ProcessRequest, args: &[&str]) -> Option<String> {
  for argument in args {
    if argument.is_empty()
      || argument
        .chars()
        .any(|character| character.is_control() || character == '\n')
    {
      return None;
    }
    request = request.arg(*argument);
  }
  let output = SystemProcessRunner
    .run(&request.timeout(Duration::from_secs(5)))
    .ok()?;
  if output.status.is_none_or(|status| status != 0) {
    return None;
  }
  Some(terminal_text(&String::from_utf8_lossy(&output.stdout)))
}

/// Retrieves data for `read_first` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_first(path: &Path, fallback: &str) -> String {
  fs::read_to_string(path)
    .ok()
    .and_then(|content| content.lines().next().map(str::to_string))
    .filter(|line| !line.trim().is_empty())
    .unwrap_or_else(|| fallback.to_string())
}

/// Retrieves data for `read_first_or_default` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_first_or_default(path: &Path) -> Option<String> {
  fs::read_to_string(path)
    .ok()
    .and_then(|content| content.lines().next().map(str::to_string))
    .map(|line| line.trim().to_string())
}

/// Executes the `active_theme_file` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn active_theme_file() -> PathBuf {
  argvus_data_home().join(".active-theme")
}

/// Executes the `accent_file` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn accent_file() -> PathBuf {
  argvus_data_home().join(".accent-color")
}

/// Reads the default accent declared by an installed theme manifest. Drop-in
/// theme packages are discovered at runtime, so this needs no per-theme entry.
fn discovered_default_accent(theme: &str) -> Option<String> {
  discovered_default_accent_in(&system_config_root(), theme)
}

fn discovered_default_accent_in(system_config: &Path, theme: &str) -> Option<String> {
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  argvus_theme::discovery::discover_themes(system_config)
    .themes
    .into_iter()
    .find(|entry| entry.id == family)
    .and_then(|entry| normalize_hex_color(&entry.accent))
}

/// Executes the `theme_default_accent` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn theme_default_accent(theme: &str) -> Option<&'static str> {
  match theme {
    "dracula" | "dracula-float" => Some("#BD93F9"),
    "argvus-dark" | "argvus-dark-float" => Some("#3590bd"),
    "silver-dark" | "silver-dark-float" => Some("#595959"),
    "argvus-light" | "argvus-light-float" => Some("#181818"),
    "github-light" | "github-light-float" => Some("#0969DA"),
    "one-light" | "one-light-float" => Some("#4078F2"),
    "everforest-light" | "everforest-light-float" => Some("#3A94C5"),
    "solarized-light" | "solarized-light-float" => Some("#268BD2"),
    "rose-pine" | "rose-pine-float" => Some("#C4A7E7"),
    "catppuccin-latte" | "catppuccin-latte-float" => Some("#1E66F5"),
    "gruvbox-light" | "gruvbox-light-float" => Some("#458588"),
    "slate-dark" | "slate-dark-float" => Some("#7391a5"),
    "universe" | "universe-float" => Some("#eeeeee"),
    "gruvbox-high-dark" | "gruvbox-high-dark-float" => Some("#D79921"),
    "gruvbox-dark" | "gruvbox-dark-float" => Some("#D4BE98"),
    "tokyo-night" | "tokyo-night-float" => Some("#7AA2F7"),
    "solitude" | "solitude-float" => Some("#798186"),
    "sunset" | "sunset-float" => Some("#E2BE8A"),
    "matrix" | "matrix-float" => Some("#82FB9C"),
    "monokai-dark" | "monokai-dark-float" => Some("#78DCE8"),
    _ => None,
  }
}

/// Converts input data into `parse_pairs` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_pairs(output: &str) -> impl Iterator<Item = (&str, &str)> {
  output.lines().filter_map(|line| {
    line
      .split_once('=')
      .map(|(key, value)| (key.trim(), value.trim()))
  })
}

/// Converts input data into `parse_spacing_status` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_spacing_status(output: &str, state: &mut AppearanceState) {
  for (key, value) in parse_pairs(output) {
    match key {
      "waybar_pos" => state.waybar_pos = TaskbarPosition::from_value(value),
      "waybar_top" => state.waybar_top = value.parse().unwrap_or(state.waybar_top),
      "waybar_left" => state.waybar_left = value.parse().unwrap_or(state.waybar_left),
      "waybar_right" => state.waybar_right = value.parse().unwrap_or(state.waybar_right),
      "waybar_bottom" => state.waybar_bottom = value.parse().unwrap_or(state.waybar_bottom),
      "gaps_in" => state.gaps_in = value.parse().unwrap_or(state.gaps_in),
      "gaps_out_top" => state.gaps_out_top = value.parse().unwrap_or(state.gaps_out_top),
      "gaps_out_left" => state.gaps_out_left = value.parse().unwrap_or(state.gaps_out_left),
      "gaps_out_right" => state.gaps_out_right = value.parse().unwrap_or(state.gaps_out_right),
      "gaps_out_bottom" => state.gaps_out_bottom = value.parse().unwrap_or(state.gaps_out_bottom),
      _ => {}
    }
  }
}

/// Converts input data into `parse_borders_status` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_borders_status(output: &str, state: &mut AppearanceState) {
  for (key, value) in parse_pairs(output) {
    match key {
      "rounded" => state.rounded = value == "1",
      "rounding" => state.rounding = value.parse().unwrap_or(state.rounding),
      "thickness" => state.thickness = value.parse().unwrap_or(state.thickness),
      _ => {}
    }
  }
}

/// Reads the `height=` and `border=` lines of `argvus config taskbar-shape`.
/// A border of `0` is off and keeps the last thickness, so turning it back on
/// restores it. Values outside the offered ones keep the state as it was.
fn parse_taskbar_shape(output: &str, state: &mut AppearanceState) {
  for (key, value) in parse_pairs(output) {
    match key {
      "height" => {
        if let Some(height) = value
          .parse()
          .ok()
          .filter(|height| TASKBAR_HEIGHTS.contains(height))
        {
          state.taskbar_height = height;
        }
      }
      "border" => match value.parse::<u32>() {
        Ok(0) => state.taskbar_border_enabled = false,
        Ok(width) if TASKBAR_BORDER_THICKNESSES.contains(&width) => {
          state.taskbar_border_enabled = true;
          state.taskbar_border_thickness = width;
        }
        _ => {}
      },
      _ => {}
    }
  }
}

pub(crate) fn canonical_layout() -> Option<Value> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", "/layout", "--effective"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  serde_json::from_slice(&output.stdout).ok()
}

fn apply_canonical_layout(state: &mut AppearanceState) {
  let Some(layout) = canonical_layout() else {
    return;
  };
  let window = layout.get("window").and_then(Value::as_object);
  let taskbar = layout.get("taskbar").and_then(Value::as_object);
  let integer = |object: Option<&serde_json::Map<String, Value>>, key: &str| {
    object
      .and_then(|values| values.get(key))
      .and_then(Value::as_i64)
  };
  if let Some(value) = integer(window, "gaps_in") {
    state.gaps_in = value as i32;
  }
  for (key, target) in [
    ("gaps_out_top", &mut state.gaps_out_top),
    ("gaps_out_left", &mut state.gaps_out_left),
    ("gaps_out_right", &mut state.gaps_out_right),
    ("gaps_out_bottom", &mut state.gaps_out_bottom),
  ] {
    if let Some(value) = integer(window, key) {
      *target = value as i32;
    }
  }
  if let Some(value) = window
    .and_then(|values| values.get("rounded"))
    .and_then(Value::as_bool)
  {
    state.rounded = value;
  }
  if let Some(value) = integer(window, "rounding") {
    state.rounding = value as i32;
  }
  if let Some(value) = integer(window, "border_size") {
    state.thickness = value as i32;
  }
  if let Some(value) = taskbar
    .and_then(|values| values.get("position"))
    .and_then(Value::as_str)
  {
    state.waybar_pos = TaskbarPosition::from_value(value);
  }
  for (key, target) in [
    ("margin_top", &mut state.waybar_top),
    ("margin_left", &mut state.waybar_left),
    ("margin_right", &mut state.waybar_right),
    ("margin_bottom", &mut state.waybar_bottom),
  ] {
    if let Some(value) = integer(taskbar, key) {
      *target = value as i32;
    }
  }
}

pub(crate) fn canonical_taskbar() -> Option<Value> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", "/taskbar", "--effective"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  serde_json::from_slice(&output.stdout).ok()
}

fn apply_canonical_taskbar(state: &mut AppearanceState) {
  let Some(taskbar) = canonical_taskbar() else {
    return;
  };
  let icons = taskbar.get("icons").and_then(Value::as_object);
  let date = taskbar.get("date").and_then(Value::as_object);
  let time = taskbar.get("time").and_then(Value::as_object);
  let boolean = |object: Option<&serde_json::Map<String, Value>>, key: &str| {
    object
      .and_then(|values| values.get(key))
      .and_then(Value::as_bool)
  };
  if let Some(value) = boolean(icons, "audio_player_enabled") {
    state.taskbar_audio_player_enabled = value;
  }
  if let Some(value) = boolean(icons, "launcher_enabled") {
    state.taskbar_launcher_enabled = value;
  }
  state.taskbar_launcher_custom_icon_path = icons
    .and_then(|values| values.get("launcher_custom_icon_path"))
    .and_then(Value::as_str)
    .map(str::to_owned);
  for widget in TaskbarUtilityWidget::ALL {
    if let Some(value) = boolean(icons, &format!("{}_enabled", widget.key())) {
      state.taskbar_utility_widgets.set(widget, value);
    }
  }
  if let Some(value) = date
    .and_then(|values| values.get("format"))
    .and_then(Value::as_str)
  {
    state.taskbar_date_format = TaskbarDateFormat::from_value(value);
  }
  if let Some(value) = boolean(time, "seconds_enabled") {
    state.taskbar_time_seconds_enabled = value;
  }
  if let Some(value) = time
    .and_then(|values| values.get("format"))
    .and_then(Value::as_str)
  {
    state.taskbar_time_format = TaskbarTimeFormat::from_value(value);
  }
}

/// Reads one independent visual state from the shared session contract.
fn effect_state(component: &str) -> bool {
  let path = argvus_data_home().join("state").join(component);
  match read_first_or_default(&path).as_deref() {
    Some("enabled") => true,
    Some("disabled") => false,
    _ => matches!(
      run_effects_output(&[component, "status"])
        .as_deref()
        .map(str::trim),
      Some("enabled")
    ),
  }
}

fn effect_value(kind: &str, surface: EffectSurface) -> i32 {
  run_effects_output(&[&format!("{kind}-value"), surface.key(), "get"])
    .and_then(|value| value.trim().parse::<i32>().ok())
    .filter(|value| (0..=100).contains(value))
    .unwrap_or(50)
}

/// Reads the Hyprland animation values from the session script. A missing
/// script or config falls back to the defaults.
fn animation_values() -> AnimationValues {
  run_effects_output(&["animations-settings", "get"])
    .map(|output| AnimationValues::from_script_output(&output))
    .unwrap_or_default()
}

/// Reads the Hyprland blur parameters from the session script. A missing
/// script or config falls back to the defaults.
fn blur_values() -> BlurValues {
  run_effects_output(&["blur-settings", "get"])
    .map(|output| BlurValues::from_script_output(&output))
    .unwrap_or_default()
}

fn effect_enabled(kind: &str, surface: EffectSurface) -> bool {
  run_effects_output(&["effect-enabled", surface.key(), kind])
    .is_some_and(|value| value.trim() == "enabled")
}

fn control_panel_enabled() -> bool {
  run_script_output(&control_panel_cards_script(), &["master", "status"])
    .is_none_or(|value| value.trim() != "disabled")
}

/// Executes the `telemetry_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn telemetry_state() -> bool {
  if let Ok(output) = SystemProcessRunner.run(
    &ProcessRequest::new("env")
      .arg("ARGVUS_MACHINE_OUTPUT=1")
      .arg("argvus")
      .arg("widget-telemetry-toggle")
      .arg("status")
      .timeout(Duration::from_secs(3)),
  ) && output.status.is_none_or(|status| status == 0)
  {
    match terminal_text(&String::from_utf8_lossy(&output.stdout)).trim() {
      "enabled" => return true,
      "disabled" => return false,
      _ => {}
    }
  }
  let cache = cache_home().join("argvus").join("waybar");
  let candidates = [
    cache.join("widget-telemetry-state"),
    cache.join("desktop-telemetry-state"),
    cache.join("sysinfo-state"),
  ];
  for candidate in candidates {
    match read_first_or_default(&candidate).as_deref() {
      Some("enabled") => return true,
      Some("disabled") => return false,
      _ => {}
    }
  }
  false
}

/// Maps the widget package's stable command-line identifier back to its
/// enum variant, shared by the block-state and display-order readers.
fn telemetry_block_from_key(key: &str) -> Option<WidgetTelemetryBlock> {
  Some(match key {
    "system" => WidgetTelemetryBlock::System,
    "cpu_gpu" => WidgetTelemetryBlock::CpuGpu,
    "memory" => WidgetTelemetryBlock::Memory,
    "storage" => WidgetTelemetryBlock::Storage,
    "processes" => WidgetTelemetryBlock::Processes,
    "network" => WidgetTelemetryBlock::Network,
    "keys" => WidgetTelemetryBlock::Shortcuts,
    "dev_dashboard" => WidgetTelemetryBlock::DevDashboard,
    _ => return None,
  })
}

/// Runs `argvus-widget-telemetry-toggle blocks status` once, whose output is
/// shared by [`telemetry_blocks`] and [`telemetry_order`].
fn telemetry_blocks_status() -> Option<String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("env")
        .arg("ARGVUS_MACHINE_OUTPUT=1")
        .arg("argvus")
        .arg("widget-telemetry-toggle")
        .arg("blocks")
        .arg("status")
        .timeout(Duration::from_secs(3)),
    )
    .ok()?;
  if output.status.is_some_and(|status| status != 0) {
    return None;
  }
  Some(terminal_text(&String::from_utf8_lossy(&output.stdout)))
}

/// Reads the widget-owned machine format while retaining enabled defaults when
/// an older package does not yet expose block preferences.
fn telemetry_blocks(status: &str) -> WidgetTelemetryBlocks {
  let mut blocks = WidgetTelemetryBlocks::default();
  for line in status.lines() {
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    let Some(block) = telemetry_block_from_key(key.trim()) else {
      continue;
    };
    match value.trim() {
      "enabled" => blocks.set(block, true),
      "disabled" => blocks.set(block, false),
      _ => {}
    }
  }
  blocks
}

/// Reads the persisted display order, appending any block missing from it
/// (an older package install, before this preference existed) at the end.
fn telemetry_order(status: &str) -> Vec<WidgetTelemetryBlock> {
  let mut order = Vec::new();
  if let Some(value) = status
    .lines()
    .find_map(|line| line.strip_prefix("__order__="))
  {
    for key in value.split(',') {
      if let Some(block) = telemetry_block_from_key(key.trim())
        && !order.contains(&block)
      {
        order.push(block);
      }
    }
  }
  for block in WidgetTelemetryBlock::ALL {
    if !order.contains(&block) {
      order.push(block);
    }
  }
  order
}

/// Reads the Control Panel helper status: the enabled flags and the order,
/// which come from the same call. The hardware availability is not probed
/// here: it is carried over from `previous` and refreshed separately by
/// [`control_panel_availability`], so the page never waits for a hardware probe.
fn control_panel_cards(previous: &ControlPanelCards) -> (ControlPanelCards, Vec<ControlPanelCard>) {
  let status = run_script_output(&control_panel_cards_script(), &["status"]);
  control_panel_cards_from_status(status.as_deref(), previous)
}

/// Decodes the helper status and keeps package defaults when an older
/// installation does not provide the helper or returns malformed data. The
/// availability of every card is the one `previous` already knew.
fn control_panel_cards_from_status(
  status: Option<&str>,
  previous: &ControlPanelCards,
) -> (ControlPanelCards, Vec<ControlPanelCard>) {
  let mut cards = status.map(parse_control_panel_cards).unwrap_or_default();
  for card in ControlPanelCard::ALL {
    cards.set_available(card, previous.available(card));
  }
  let order = status
    .map(parse_control_panel_order)
    .unwrap_or_else(|| ControlPanelCard::ALL.to_vec());
  (cards, order)
}

/// Probes the hardware the cards depend on, with the same checks the cards
/// run themselves. A disabled card stays disabled when its hardware is
/// temporarily disconnected and becomes available again later.
///
/// The probes invoke external tools with timeouts (bluetoothctl may wait up to
/// seconds for BlueZ), so they run concurrently, and the caller runs this in a
/// background job of its own instead of the page load.
pub fn control_panel_availability() -> Vec<(ControlPanelCard, bool)> {
  let (bluetooth_available, brightness_available) = std::thread::scope(|scope| {
    let bluetooth = scope.spawn(|| {
      run_argvus_route_output("bluetoothctl", &["status"])
        .is_some_and(|status| status.lines().any(|line| line.trim() == "available=yes"))
    });
    let brightness = scope.spawn(|| {
      run_script_output(&script("brightness-switch.sh"), &["--status"])
        .is_some_and(|backend| matches!(backend.trim(), "brightnessctl" | "ddcutil"))
    });
    (
      bluetooth.join().unwrap_or(false),
      brightness.join().unwrap_or(false),
    )
  });
  vec![
    (ControlPanelCard::Bluetooth, bluetooth_available),
    (ControlPanelCard::Brightness, brightness_available),
  ]
}

/// Maps the Control Panel helper's stable card ID back to its enum variant,
/// shared by the enabled-state and display-order readers.
fn control_panel_card_from_key(key: &str) -> Option<ControlPanelCard> {
  Some(match key {
    "user" => ControlPanelCard::User,
    "notifications" => ControlPanelCard::Notifications,
    "calendar" => ControlPanelCard::Calendar,
    "weather" => ControlPanelCard::Weather,
    "volume" => ControlPanelCard::Volume,
    "brightness" => ControlPanelCard::Brightness,
    "network" => ControlPanelCard::Network,
    "bluetooth" => ControlPanelCard::Bluetooth,
    "system" => ControlPanelCard::System,
    "appearance" => ControlPanelCard::Appearance,
    "session" => ControlPanelCard::Session,
    "display" => ControlPanelCard::Display,
    "power" => ControlPanelCard::Power,
    _ => return None,
  })
}

/// Parses helper JSON independently from process execution for deterministic
/// coverage of compatibility and malformed-status fallbacks.
fn parse_control_panel_cards(output: &str) -> ControlPanelCards {
  let mut cards = ControlPanelCards::default();
  let Ok(value) = serde_json::from_str::<Value>(output) else {
    return cards;
  };
  let Some(entries) = value.get("cards").and_then(Value::as_array) else {
    return cards;
  };
  for entry in entries {
    let Some(key) = entry.get("id").and_then(Value::as_str) else {
      continue;
    };
    let Some(enabled) = entry.get("enabled").and_then(Value::as_bool) else {
      continue;
    };
    let Some(card) = control_panel_card_from_key(key) else {
      continue;
    };
    cards.set(card, enabled);
  }
  cards
}

/// Parses the card order from the same helper JSON (the `cards` array is
/// already emitted in persisted order), appending any card missing from it
/// — an older helper, before this preference existed — at the end.
fn parse_control_panel_order(output: &str) -> Vec<ControlPanelCard> {
  let mut order = Vec::new();
  if let Ok(value) = serde_json::from_str::<Value>(output)
    && let Some(entries) = value.get("cards").and_then(Value::as_array)
  {
    for entry in entries {
      let Some(key) = entry.get("id").and_then(Value::as_str) else {
        continue;
      };
      if let Some(card) = control_panel_card_from_key(key)
        && !order.contains(&card)
      {
        order.push(card);
      }
    }
  }
  for card in ControlPanelCard::ALL {
    if !order.contains(&card) {
      order.push(card);
    }
  }
  order
}

/// Executes the `list_wallpapers` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn classify_wallpaper(relative: &Path) -> Option<(WallpaperCollection, WallpaperMode)> {
  let components = relative
    .components()
    .filter_map(|component| component.as_os_str().to_str())
    .collect::<Vec<_>>();
  match components.as_slice() {
    ["argvus-dark.jxl"] => Some((WallpaperCollection::Abstract, WallpaperMode::Dark)),
    ["argvus-light.jxl"] => Some((WallpaperCollection::Abstract, WallpaperMode::Light)),
    [collection, mode, filename] if filename.ends_with(".jxl") => {
      let collection = match *collection {
        "abstract" => WallpaperCollection::Abstract,
        "landscape" => WallpaperCollection::Landscape,
        _ => return None,
      };
      let mode = match *mode {
        "dark" => WallpaperMode::Dark,
        "light" => WallpaperMode::Light,
        _ => return None,
      };
      Some((collection, mode))
    }
    _ => None,
  }
}

pub fn list_wallpapers() -> Vec<WallpaperEntry> {
  fn collect(root: &Path, current: &Path, entries: &mut Vec<WallpaperEntry>) {
    let Ok(directory_entries) = fs::read_dir(current) else {
      return;
    };
    for entry in directory_entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        collect(root, &path, entries);
      } else if path.extension().is_some_and(|extension| extension == "jxl")
        && let Ok(relative) = path.strip_prefix(root)
        && let Some((collection, mode)) = classify_wallpaper(relative)
      {
        entries.push(WallpaperEntry {
          path: relative.to_string_lossy().into_owned(),
          collection,
          mode,
        });
      }
    }
  }

  let mut entries = Vec::new();
  collect(
    Path::new(WALLPAPERS_DIR),
    Path::new(WALLPAPERS_DIR),
    &mut entries,
  );
  entries.sort_by(|left, right| left.path.cmp(&right.path));
  entries
}

/// Executes the `hyprpaper_config_path` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn hyprpaper_config_path() -> PathBuf {
  argvus_data_home().join("hypr").join("hyprpaper.conf")
}

fn persist_wallpaper_canonical(wallpaper: &Path) -> Result<(), String> {
  let patch = serde_json::json!({
    "/appearance/wallpaper": wallpaper.to_string_lossy(),
    "/appearance/wallpaper_custom": true,
  });
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist wallpaper settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the wallpaper settings".into());
  }
  reload_argvus_config_service()
}

/// Reads `/appearance/icon_theme` through `argvus config get`. A failed read
/// resolves to the ARGVUS default, so the Icons page still opens.
pub fn read_configured_icon_theme(default: &str) -> String {
  let output = argvus_control_center_core::process::command("argvus")
    .args(["config", "get", "/appearance/icon_theme", "--raw"])
    .output()
    .ok()
    .filter(|output| output.status.success())
    .map(|output| String::from_utf8_lossy(&output.stdout).into_owned());
  crate::icons::configured_icon_theme(output.as_deref(), default)
}

/// Loads the configured icon theme and the installed icon themes for the
/// Icons page.
pub(crate) fn load_icon_themes(state: &mut AppearanceState) {
  // The empty-configuration default depends on the mode of the active theme,
  // so the official themes (which declare their category) must be loaded.
  refresh_official_themes(state);
  let default = crate::icons::default_icon_theme_for(&state.theme, &state.official_themes);
  state.icon_theme = read_configured_icon_theme(default);
  state.icon_themes = crate::icons::installed_icon_themes();
}

/// Writes the icon theme to `/appearance/icon_theme` and reloads the config
/// service so consumers read the new value.
fn persist_icon_theme_canonical(name: &str) -> Result<(), String> {
  let patch = serde_json::json!({
    "/appearance/icon_theme": name,
  });
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist icon theme settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the icon theme settings".into());
  }
  reload_argvus_config_service()
}

/// Selects an installed icon theme and applies it to GNOME, GTK and Qt.
/// The name is checked before anything is written, so an unknown or
/// unsafe name leaves the configuration untouched.
pub fn set_icon_theme(name: &str) -> Result<(), String> {
  crate::icons::validate_icon_theme_name(name)?;
  let installed = crate::icons::installed_icon_themes()
    .iter()
    .any(|entry| entry.directory_name == name);
  if !installed {
    return Err(format!("icon theme is not installed: {name}"));
  }
  persist_icon_theme_canonical(name)?;
  run_script(&script("icon-switch.sh"), &["apply"])
}

/// Executes the `active_wallpaper` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn active_wallpaper() -> Option<String> {
  let content = fs::read_to_string(hyprpaper_config_path()).ok()?;
  let path = content.lines().find_map(|line| {
    let (key, value) = line.split_once('=')?;
    if key.trim() != "path" {
      return None;
    }
    let value = value.trim();
    let expanded = value.strip_prefix('~').map(|rest| {
      let home = argvus_control_center_core::paths::home();
      home.join(rest).to_string_lossy().to_string()
    });
    Some(expanded.unwrap_or_else(|| value.to_string()))
  })?;
  path
    .strip_prefix(WALLPAPERS_DIR)
    .map(|name| name.trim_start_matches('/').to_string())
}
/// Discovers and loads official themes (built-in + drop-in packages).
/// Errors during discovery are collected as warnings and do not fail the operation.
fn refresh_official_themes(state: &mut AppearanceState) {
  let report = argvus_theme::discovery::discover_themes(&system_config_root());
  state.official_themes = report.themes;
  state.discovery_warnings = report.warnings;
}

/// Surfaces whose transparency (and, for some, blur) `load_page` reads.
const SURFACES_WITH_EFFECTS: [EffectSurface; 5] = [
  EffectSurface::Taskbar,
  EffectSurface::ControlPanel,
  EffectSurface::WidgetTelemetry,
  EffectSurface::Terminal,
  EffectSurface::Launchers,
];

/// What the concurrent readers of [`load_page`] collected. `None` means the
/// page does not show that data, so the previous values are kept.
struct PageReadings {
  global_effects: Option<GlobalEffects>,
  surface_effects: Vec<(EffectSurface, SurfaceEffects)>,
  telemetry: Option<TelemetryReading>,
  control_panel: Option<ControlPanelReading>,
  layout: Option<LayoutStatus>,
}

/// The global effect switches and the Hyprland blur parameters.
struct GlobalEffects {
  animations: bool,
  transparency: bool,
  blur: bool,
  blur_values: BlurValues,
}

/// One surface's transparency, and its blur when the surface has one.
struct SurfaceEffects {
  transparency: i32,
  transparency_enabled: bool,
  blur: Option<(i32, bool)>,
}

/// The widget switch and the raw `blocks status` output.
struct TelemetryReading {
  enabled: bool,
  blocks_status: String,
}

/// The Control Panel master switch and its cards.
struct ControlPanelReading {
  cards: ControlPanelCards,
  order: Vec<ControlPanelCard>,
  enabled: bool,
}

/// The raw status of the layout scripts, parsed after the join.
struct LayoutStatus {
  utility_group: Option<String>,
  spacing: Option<String>,
  borders: Option<String>,
  taskbar_shape: Option<String>,
}

/// The result of a reader that ran, or `None` when it did not run or panicked
/// (the page then keeps its previous values, as when a reader fails).
fn joined<T>(handle: Option<std::thread::ScopedJoinHandle<'_, T>>) -> Option<T> {
  handle.and_then(|handle| handle.join().ok())
}

fn read_global_effects() -> GlobalEffects {
  std::thread::scope(|scope| {
    let blur_values = scope.spawn(blur_values);
    let animations = scope.spawn(|| effect_state("animations"));
    let transparency = scope.spawn(|| effect_state("transparency"));
    let blur = effect_state("blur");
    GlobalEffects {
      animations: animations.join().unwrap_or_default(),
      transparency: transparency.join().unwrap_or_default(),
      blur,
      blur_values: blur_values.join().unwrap_or_default(),
    }
  })
}

/// Reads one surface. Terminal and launchers have no blur of their own.
fn read_surface_effects(surface: EffectSurface) -> SurfaceEffects {
  let has_blur = !matches!(surface, EffectSurface::Terminal | EffectSurface::Launchers);
  SurfaceEffects {
    transparency: effect_value("transparency", surface),
    transparency_enabled: effect_enabled("transparency", surface),
    blur: has_blur.then(|| {
      (
        effect_value("blur", surface),
        effect_enabled("blur", surface),
      )
    }),
  }
}

fn store_surface_effects(
  state: &mut AppearanceState,
  surface: EffectSurface,
  effects: SurfaceEffects,
) {
  let (blur, blur_enabled) = effects.blur.unwrap_or((0, true));
  match surface {
    EffectSurface::Taskbar => {
      state.taskbar_transparency = effects.transparency;
      state.taskbar_transparency_enabled = effects.transparency_enabled;
      state.taskbar_blur = blur;
      state.taskbar_blur_enabled = blur_enabled;
    }
    EffectSurface::ControlPanel => {
      state.control_panel_transparency = effects.transparency;
      state.control_panel_transparency_enabled = effects.transparency_enabled;
      state.control_panel_blur = blur;
      state.control_panel_blur_enabled = blur_enabled;
    }
    EffectSurface::WidgetTelemetry => {
      state.widget_telemetry_transparency = effects.transparency;
      state.widget_telemetry_transparency_enabled = effects.transparency_enabled;
      state.widget_telemetry_blur = blur;
      state.widget_telemetry_blur_enabled = blur_enabled;
    }
    EffectSurface::Terminal => {
      state.terminal_transparency = effects.transparency;
      state.terminal_transparency_enabled = effects.transparency_enabled;
    }
    EffectSurface::Launchers => {
      state.launcher_transparency = effects.transparency;
      state.launcher_transparency_enabled = effects.transparency_enabled;
    }
  }
}

fn read_telemetry() -> TelemetryReading {
  std::thread::scope(|scope| {
    let blocks_status = scope.spawn(telemetry_blocks_status);
    let enabled = telemetry_state();
    TelemetryReading {
      enabled,
      blocks_status: blocks_status.join().ok().flatten().unwrap_or_default(),
    }
  })
}

fn read_control_panel(previous_cards: &ControlPanelCards) -> ControlPanelReading {
  std::thread::scope(|scope| {
    let enabled = scope.spawn(control_panel_enabled);
    let (cards, order) = control_panel_cards(previous_cards);
    ControlPanelReading {
      cards,
      order,
      enabled: enabled.join().unwrap_or(true),
    }
  })
}

fn read_layout_status() -> LayoutStatus {
  std::thread::scope(|scope| {
    let utility_group =
      scope.spawn(|| run_script_output(&script("taskbar-right-2-mode.sh"), &["status"]));
    let spacing = scope.spawn(|| run_script_output(&script("spaces-switch.sh"), &["--status"]));
    let borders = run_script_output(&script("borders-switch.sh"), &["--status"]);
    let taskbar_shape = scope.spawn(|| run_argvus_route_output("config", &["taskbar-shape"]));
    LayoutStatus {
      utility_group: utility_group.join().ok().flatten(),
      spacing: spacing.join().ok().flatten(),
      borders,
      taskbar_shape: taskbar_shape.join().ok().flatten(),
    }
  })
}

/// Retrieves data for `load_state` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
/// Collect only data used by the requested page. Existing values survive unrelated refreshes.
pub fn load_page(
  page: crate::model::AppearancePage,
  mut state: AppearanceState,
) -> AppearanceState {
  use crate::model::AppearancePage;
  let theme = canonical_theme_id(&read_first(&active_theme_file(), DEFAULT_THEME));
  state.theme = theme.clone();
  let default_accent = discovered_default_accent(&theme)
    .or_else(|| theme_default_accent(&theme).map(str::to_string))
    .unwrap_or_else(|| DEFAULT_ACCENT.to_string());
  state.accent = normalize_hex_color(&read_first(&accent_file(), &default_accent))
    .unwrap_or_else(|| default_accent.clone());
  // The Desktop page shows the active wallpaper name, so it loads the active
  // value too. The file list is only needed by the Wallpapers pages.
  if matches!(page, AppearancePage::Desktop | AppearancePage::Wallpapers) {
    state.wallpaper_active = active_wallpaper();
  }
  if page == AppearancePage::Wallpapers {
    state.wallpapers = list_wallpapers();
  }
  if page == AppearancePage::Icons {
    load_icon_themes(&mut state);
  }
  let wants_global_effects = matches!(
    page,
    AppearancePage::Blur
      | AppearancePage::Animations
      | AppearancePage::Taskbar
      | AppearancePage::WidgetTelemetry
      | AppearancePage::ControlPanel
      | AppearancePage::SurfaceSection { .. }
  );
  let wants_surface_effects = matches!(
    page,
    AppearancePage::Taskbar
      | AppearancePage::WidgetTelemetry
      | AppearancePage::ControlPanel
      | AppearancePage::Terminal
      | AppearancePage::Launchers
      | AppearancePage::TerminalTransparency
      | AppearancePage::SurfaceSection { .. }
  );
  let wants_telemetry = matches!(
    page,
    AppearancePage::WidgetTelemetry
      | AppearancePage::SurfaceSection {
        surface: EffectSurface::WidgetTelemetry,
        ..
      }
  );
  let wants_control_panel = matches!(
    page,
    AppearancePage::ControlPanel
      | AppearancePage::SurfaceSection {
        surface: EffectSurface::ControlPanel,
        ..
      }
  );
  let wants_layout = matches!(
    page,
    AppearancePage::Borders
      | AppearancePage::Taskbar
      | AppearancePage::TaskbarPosition
      | AppearancePage::TaskbarSpaces
      | AppearancePage::TaskbarHeight
      | AppearancePage::TaskbarBorder
      | AppearancePage::TaskbarBorderThickness
      | AppearancePage::WindowSpaces
      | AppearancePage::WindowSpacesInner
      | AppearancePage::WindowSpacesOuter
      | AppearancePage::GeneralBorders
      | AppearancePage::EdgeThickness
      | AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        ..
      }
  );
  // Every reader below is an independent external process (about twenty for
  // the surface pages), so they run concurrently instead of one after the
  // other. Nothing is written to `state` until all of them are joined.
  let previous_cards = state.control_panel_cards.clone();
  let readings = std::thread::scope(|scope| {
    let global_effects = wants_global_effects.then(|| scope.spawn(read_global_effects));
    let surface_effects: Vec<_> = if wants_surface_effects {
      SURFACES_WITH_EFFECTS
        .into_iter()
        .map(|surface| (surface, scope.spawn(move || read_surface_effects(surface))))
        .collect()
    } else {
      Vec::new()
    };
    let telemetry = wants_telemetry.then(|| scope.spawn(read_telemetry));
    let control_panel =
      wants_control_panel.then(|| scope.spawn(|| read_control_panel(&previous_cards)));
    let layout = wants_layout.then(|| scope.spawn(read_layout_status));
    PageReadings {
      global_effects: joined(global_effects),
      surface_effects: surface_effects
        .into_iter()
        .filter_map(|(surface, handle)| handle.join().ok().map(|effects| (surface, effects)))
        .collect(),
      telemetry: joined(telemetry),
      control_panel: joined(control_panel),
      layout: joined(layout),
    }
  });
  if let Some(global_effects) = readings.global_effects {
    state.animations = global_effects.animations;
    state.transparency = global_effects.transparency;
    state.blur = global_effects.blur;
    state.blur_values = global_effects.blur_values;
  }
  if page.is_animations() {
    state.animations = effect_state("animations");
    state.animation_values = animation_values();
  }
  for (surface, effects) in readings.surface_effects {
    store_surface_effects(&mut state, surface, effects);
  }
  if let Some(telemetry) = readings.telemetry {
    state.widget_telemetry = telemetry.enabled;
    state.widget_telemetry_blocks = telemetry_blocks(&telemetry.blocks_status);
    state.widget_telemetry_order = telemetry_order(&telemetry.blocks_status);
  }
  if let Some(control_panel) = readings.control_panel {
    state.control_panel_cards = control_panel.cards;
    state.control_panel_order = control_panel.order;
    state.control_panel_enabled = control_panel.enabled;
  }
  if matches!(
    page,
    AppearancePage::Themes
      | AppearancePage::ThemeFamilies { .. }
      | AppearancePage::CustomThemes
      | AppearancePage::OfficialThemes
  ) {
    state.custom_themes = custom_themes();
    state.active_custom_theme = active_custom_theme();
    // Discover and load official themes (built-in + drop-in packages)
    refresh_official_themes(&mut state);
  }
  if let Some(layout) = readings.layout {
    if let Some(output) = layout.utility_group {
      state.taskbar_utility_group = TaskbarUtilityGroupMode::from_value(output.trim());
    }
    if let Some(output) = layout.spacing {
      parse_spacing_status(&output, &mut state);
    }
    if let Some(output) = layout.borders {
      parse_borders_status(&output, &mut state);
    }
    if let Some(output) = layout.taskbar_shape {
      parse_taskbar_shape(&output, &mut state);
    }
    apply_canonical_layout(&mut state);
  }
  if matches!(
    page,
    AppearancePage::Taskbar
      | AppearancePage::TaskbarIcons
      | AppearancePage::TaskbarDate
      | AppearancePage::TaskbarDateFormat
      | AppearancePage::TaskbarTime
      | AppearancePage::TaskbarTimeFormat
      | AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        ..
      }
  ) {
    apply_canonical_taskbar(&mut state);
  }
  state
}

/// Applies the `set_theme` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set_theme(name: &str) -> Result<(), String> {
  let canonical_name = canonical_theme_id(name);
  run_argvus_route(
    "theme-switch",
    &[canonical_name.as_str()],
    &[("ARGVUS_ACCENT_OFFICIAL", "1")],
  )?;
  let _ = fs::remove_file(argvus_data_home().join("state").join("custom-theme"));
  Ok(())
}

/// Switches the Sticky/Float layout mode without changing the active theme.
///
/// Delegates to `layout-mode-switch.sh`, the same entry point the
/// `SUPER + Shift + M` Rofi picker uses, so the Control Center and the Rofi
/// menu always apply the mode through a single implementation.
///
/// # Errors
///
/// Returns an error when `layout-mode-switch.sh` cannot be found or exits
/// with a non-zero status.
pub fn set_layout_mode(variant: &str) -> Result<(), String> {
  run_script(&script("layout-mode-switch.sh"), &[variant])
}

pub(crate) fn set_theme_static(name: &str) -> Result<(), String> {
  let canonical_name = canonical_theme_id(name);
  run_argvus_route(
    "theme-switch",
    &[canonical_name.as_str()],
    &[("ARGVUS_NO_RUNTIME", "1"), ("ARGVUS_THEME_SWITCH", "1")],
  )?;
  let _ = fs::remove_file(argvus_data_home().join("state").join("custom-theme"));
  Ok(())
}

/// Applies the `set_accent` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
///
/// A user-selected highlight color is a canonical decision, not a runtime
/// detail, so `/appearance/accent_custom` is persisted before the appearance
/// adapters run. Without that key the compositor keeps the per-theme border
/// color (`hyprland.lua` only honors `.accent-color` when the canonical flag is
/// set), and the next `argvus-config project` would otherwise re-derive the
/// accent from the theme default.
pub fn set_accent(color: &str) -> Result<(), String> {
  if color == "--theme-default" {
    return reset_accent_to_theme_default();
  }
  // `argvus config accent` writes the canonical custom accent and applies it in
  // one command; there is no separate patch step that could leave them apart.
  run_argvus_route("config", &["accent", color], &[])
}

/// Restores the active theme's default highlight color and clears the custom
/// flag so a later theme switch keeps the theme owning the accent again.
fn reset_accent_to_theme_default() -> Result<(), String> {
  run_argvus_route("config", &["accent", "--theme-default"], &[])
}

/// Persists the accent decision in the canonical document before the runtime
/// adapters run, so canonical state and generated state cannot disagree.
pub(crate) fn persist_accent_canonical(accent: &str, is_custom: bool) -> Result<(), String> {
  // An empty accent is meaningful only for the "back to theme default" action:
  // it must not overwrite the stored color, it just drops the custom flag.
  let mut patch = serde_json::Map::new();
  if !accent.is_empty() {
    patch.insert(
      "/appearance/accent".to_string(),
      Value::String(accent.to_string()),
    );
  }
  patch.insert(
    "/appearance/accent_custom".to_string(),
    Value::Bool(is_custom),
  );
  let patch = serde_json::to_string(&Value::Object(patch)).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist accent settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the accent settings".into());
  }
  Ok(())
}

/// Applies the `set_animations` operation through the shared session helper.
pub fn set_animations(enabled: bool) -> Result<(), String> {
  run_effects(&["animations", if enabled { "enable" } else { "disable" }])
}

/// Writes every animation value except the global switch in one call, so
/// `Apply` triggers a single compositor reload. The script validates each
/// value again.
pub fn set_animation_values(values: AnimationValues) -> Result<(), String> {
  let arguments = values.script_arguments();
  let mut request = vec!["animations-settings", "set"];
  request.extend(arguments.iter().map(String::as_str));
  run_effects(&request)
}

pub fn set_blur(enabled: bool) -> Result<(), String> {
  run_effects(&["blur", if enabled { "enable" } else { "disable" }])
}

pub fn set_effect_value(kind: &str, surface: EffectSurface, value: i32) -> Result<(), String> {
  if !matches!(kind, "transparency" | "blur") || !(0..=100).contains(&value) {
    return Err("invalid effect value".into());
  }
  if kind == "transparency" && surface == EffectSurface::Terminal {
    return apply_surface_effects(surface, true, value, true, 0);
  }
  let value = value.to_string();
  run_effects(&[&format!("{kind}-value"), surface.key(), "set", &value])
}

/// Writes every Blur parameter in one call, so `Apply` triggers a single
/// compositor reload. The script validates each value again.
pub fn set_blur_values(values: BlurValues) -> Result<(), String> {
  let arguments = values.script_arguments();
  let mut request = vec!["blur-settings", "set"];
  request.extend(arguments.iter().map(String::as_str));
  run_effects(&request)
}

pub fn apply_surface_effects(
  surface: EffectSurface,
  transparency_enabled: bool,
  transparency: i32,
  blur_enabled: bool,
  blur: i32,
) -> Result<(), String> {
  if !(0..=100).contains(&transparency) || !(0..=100).contains(&blur) {
    return Err("invalid effect value".into());
  }
  let args = [
    "surface-apply".to_string(),
    surface.key().to_string(),
    if transparency_enabled {
      "enabled"
    } else {
      "disabled"
    }
    .into(),
    transparency.to_string(),
    if blur_enabled { "enabled" } else { "disabled" }.into(),
    blur.to_string(),
  ];
  let references = args.iter().map(String::as_str).collect::<Vec<_>>();
  run_effects(&references)
}

pub fn set_control_panel_enabled(enabled: bool) -> Result<(), String> {
  run_script(
    &control_panel_cards_script(),
    &[
      "master",
      "set",
      if enabled { "enabled" } else { "disabled" },
    ],
  )
}

pub fn apply_widget_telemetry(
  enabled: bool,
  blocks: &WidgetTelemetryBlocks,
  order: &[WidgetTelemetryBlock],
) -> Result<(), String> {
  let values = WidgetTelemetryBlock::ALL.map(|block| {
    if blocks.enabled(block) {
      "enabled"
    } else {
      "disabled"
    }
  });
  let order = order
    .iter()
    .map(|block| block.key())
    .collect::<Vec<_>>()
    .join(",");
  let mut args = vec!["apply-state", if enabled { "enabled" } else { "disabled" }];
  args.extend(values);
  args.push(&order);
  let mut request = ProcessRequest::new("argvus").arg("widget-telemetry-toggle");
  for argument in args {
    request = request.arg(argument);
  }
  let output = SystemProcessRunner
    .run(&request)
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    return Err(terminal_text(&String::from_utf8_lossy(&output.stderr)).to_string());
  }
  Ok(())
}

/// Applies the `set_wallpaper` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set_wallpaper(name: &str) -> Result<(), String> {
  if name.is_empty()
    || name
      .chars()
      .any(|character| character.is_control() || character == '\n')
  {
    return Err("invalid wallpaper name".into());
  }
  let relative = Path::new(name);
  if relative.is_absolute()
    || relative
      .components()
      .any(|component| matches!(component, std::path::Component::ParentDir))
  {
    return Err("invalid wallpaper path".into());
  }
  let path = PathBuf::from(WALLPAPERS_DIR).join(relative);
  if !path.is_file() || path.extension().is_none_or(|extension| extension != "jxl") {
    return Err(format!("papel de parede não encontrado: {name}"));
  }
  persist_wallpaper_canonical(&path)?;
  if let Ok(output) = SystemProcessRunner.run(
    &ProcessRequest::new("sh")
      .arg(script("hyprlock-theme.sh").to_string_lossy().to_string())
      .arg("--invalidate"),
  ) {
    let _ = output;
  }
  Ok(())
}

/// Opens the configured File Manager in a window focused on top of the Control
/// Center. TUI managers are launched inside a dedicated terminal (the
/// `argvus-wallpaper-picker` class); GUI managers are raised via their WM class
/// or matched by PID to guarantee they appear in front.
pub fn choose_wallpaper() -> Result<(), String> {
  let Some(selected) = argvus_control_center_core::picker::pick_file(
    "argvus-wallpaper-picker",
    "wallpaper-selection",
  )?
  else {
    return Ok(());
  };
  persist_wallpaper_canonical(&selected)
}

/// Persists the taskbar launcher's custom icon path to the canonical
/// document and re-derives the generated Waybar config from it, restarting
/// the taskbar so the new icon appears immediately. Unlike the Enable
/// toggle (see `apply_taskbar_icons_and_format`), picking a custom icon is
/// an immediate action, not part of the Taskbar > Icons draft/Apply cycle —
/// the same way `persist_wallpaper_canonical` applies outside the draft.
fn persist_launcher_icon_canonical(path: &Path) -> Result<(), String> {
  let patch = serde_json::json!({
    "/taskbar/icons/launcher_custom_icon_path": path.to_string_lossy(),
  });
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist launcher icon settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the launcher icon settings".into());
  }
  run_script(&script("taskbar-widgets-mode.sh"), &["apply"])?;
  reload_taskbar()
}

/// Opens the configured File Manager so the user can pick a custom launcher
/// icon (PNG/SVG) for the taskbar, in a window focused on top of the Control
/// Center. Mirrors `choose_wallpaper`'s GUI/TUI flow.
pub fn choose_launcher_icon() -> Result<(), String> {
  let Some(selected) = argvus_control_center_core::picker::pick_file(
    "argvus-launcher-icon-picker",
    "launcher-icon-selection",
  )?
  else {
    return Ok(());
  };
  persist_launcher_icon_canonical(&selected)
}

/// Applies the `set_spacing` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set_spacing(key: &str, value: &str) -> Result<(), String> {
  if key.is_empty() || value.is_empty() {
    return Err("invalid spaces key/value".into());
  }
  let number = value.parse::<u64>().ok();
  let mut patch = serde_json::Map::new();
  let pointer = match key {
    "gaps_in" => "/layout/window/gaps_in",
    "gaps_out_top" => "/layout/window/gaps_out_top",
    "gaps_out_left" => "/layout/window/gaps_out_left",
    "gaps_out_right" => "/layout/window/gaps_out_right",
    "gaps_out_bottom" => "/layout/window/gaps_out_bottom",
    "gaps_out" => {
      let number = number.ok_or_else(|| "invalid spaces value".to_string())?;
      for edge in ["top", "left", "right", "bottom"] {
        patch.insert(
          format!("/layout/window/gaps_out_{edge}"),
          Value::from(number),
        );
      }
      ""
    }
    "waybar_top" => "/layout/taskbar/margin_top",
    "waybar_left" => "/layout/taskbar/margin_left",
    "waybar_right" => "/layout/taskbar/margin_right",
    "waybar_bottom" => "/layout/taskbar/margin_bottom",
    "waybar" => {
      let number = number.ok_or_else(|| "invalid spaces value".to_string())?;
      for edge in ["top", "left", "right", "bottom"] {
        patch.insert(
          format!("/layout/taskbar/margin_{edge}"),
          Value::from(number),
        );
      }
      ""
    }
    "waybar_pos" => {
      if value != "top" && value != "bottom" {
        return Err("invalid waybar position".into());
      }
      patch.insert(
        "/layout/taskbar/position".into(),
        Value::String(value.to_owned()),
      );
      ""
    }
    _ => return Err("invalid spaces key".into()),
  };
  if !pointer.is_empty() {
    patch.insert(
      pointer.into(),
      Value::from(number.ok_or_else(|| "invalid spaces value".to_string())?),
    );
  }
  persist_layout_patch(patch)?;
  Ok(())
}

/// Applies the `set_waybar_position` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set_waybar_position(position: &str) -> Result<(), String> {
  if position != "top" && position != "bottom" {
    return Err("invalid waybar position".into());
  }
  set_spacing("waybar_pos", position)
}

/// Persists the taskbar utility-group mode (the `group/right-2` drawer's
/// expanded/collapsed presentation) and regenerates its managed Waybar
/// configuration. Reload is left to the caller, which also applies the rest
/// of the taskbar icon/date/time preferences in the same action and restarts
/// the taskbar exactly once.
pub fn set_taskbar_utility_group(mode: TaskbarUtilityGroupMode) -> Result<(), String> {
  run_script(&script("taskbar-right-2-mode.sh"), &["set", mode.value()])
}

/// Writes the taskbar height and border to the live taskbar files. A border that
/// is off is written as width `0`. The taskbar is restarted by the icon apply
/// that always follows this in `Apply`, so the new shape is shown.
pub fn set_taskbar_shape(
  height: u32,
  border_enabled: bool,
  border_thickness: u32,
) -> Result<(), String> {
  let border_width = if border_enabled { border_thickness } else { 0 };
  run_taskbar_config(&["taskbar-height", &height.to_string()])?;
  run_taskbar_config(&["taskbar-border", &border_width.to_string()])
}

fn run_taskbar_config(arguments: &[&str]) -> Result<(), String> {
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(arguments)
    .status()
    .map_err(|error| format!("failed to set the taskbar shape: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the taskbar shape".into());
  }
  Ok(())
}

/// Persists the taskbar icon/date/time preferences to the canonical document,
/// re-derives the generated Waybar config from them, then restarts the
/// taskbar so it picks up the regenerated file.
///
/// A plain `argvus-sessionctl reload` only reloads `argvus-config.service`
/// and never touches `argvus-taskbar.service`, so module visibility and
/// clock/date format changes would silently never apply — the taskbar must
/// be restarted explicitly (see `reload_taskbar()`).
pub fn apply_taskbar_icons_and_format(
  audio_player_enabled: bool,
  launcher_enabled: bool,
  utility_widgets: &TaskbarUtilityWidgets,
  date_format: TaskbarDateFormat,
  time_seconds_enabled: bool,
  time_format: TaskbarTimeFormat,
) -> Result<(), String> {
  let mut patch = serde_json::Map::new();
  patch.insert(
    "/taskbar/icons/audio_player_enabled".into(),
    Value::Bool(audio_player_enabled),
  );
  patch.insert(
    "/taskbar/icons/launcher_enabled".into(),
    Value::Bool(launcher_enabled),
  );
  for widget in TaskbarUtilityWidget::ALL {
    patch.insert(
      format!("/taskbar/icons/{}_enabled", widget.key()),
      Value::Bool(utility_widgets.enabled(widget)),
    );
  }
  patch.insert(
    "/taskbar/date/format".into(),
    Value::String(date_format.value().into()),
  );
  patch.insert(
    "/taskbar/time/seconds_enabled".into(),
    Value::Bool(time_seconds_enabled),
  );
  patch.insert(
    "/taskbar/time/format".into(),
    Value::String(time_format.value().into()),
  );
  persist_taskbar_patch(patch)?;
  run_script(&script("taskbar-widgets-mode.sh"), &["apply"])?;
  reload_taskbar()
}

/// Restarts the taskbar (Waybar) process so it re-parses its generated
/// JSONC from scratch. `argvus-sessionctl reload` is not sufficient here: it
/// only reload-or-restarts `argvus-config.service` and never touches
/// `argvus-taskbar.service` (confirmed in `de/argvus-session`'s
/// `argvus-sessionctl`/`test-session-reload.py`). A full restart is also the
/// only way Waybar re-reads anything beyond CSS — `reload_style_on_change`
/// only watches the stylesheet, and the SIGUSR1 binding in Hyprland merely
/// toggles bar visibility, not a config reload.
fn reload_taskbar() -> Result<(), String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("argvus")
        .arg("sessionctl")
        .arg("restart")
        .arg("taskbar"),
    )
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr))
      .trim()
      .to_string();
    return Err(if stderr.is_empty() {
      "argvus-sessionctl restart taskbar falhou".into()
    } else {
      stderr
    });
  }
  Ok(())
}

fn persist_taskbar_patch(patch: serde_json::Map<String, Value>) -> Result<(), String> {
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist taskbar settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the taskbar settings".into());
  }
  reload_argvus_config_service()
}

/// Persists a complete batch of Control Panel preferences and performs one
/// targeted restart after all writes have succeeded.
pub fn set_control_panel_cards(changes: Vec<(ControlPanelCard, bool)>) -> Result<(), String> {
  for &(card, enabled) in &changes {
    run_script(
      &control_panel_cards_script(),
      &[
        "set",
        card.key(),
        if enabled { "enabled" } else { "disabled" },
      ],
    )?;
  }
  if changes.is_empty() {
    return Ok(());
  }
  reload_control_panel()
}

/// Moves a Control Panel card to an absolute position in its package-owned
/// order (0-based, the same index space as `CARD_IDS` in `cards-config.sh`),
/// the same primitive the real panel's drag-and-drop already calls.
pub fn move_control_panel_card(card: ControlPanelCard, index: usize) -> Result<(), String> {
  let index = index.to_string();
  run_script(&control_panel_cards_script(), &["move", card.key(), &index])?;
  reload_control_panel()
}

/// Restarts only the Control Panel consumer instead of reapplying every
/// mutable session component. This keeps rapid card toggles responsive while
/// preserving the required immediate panel reload.
fn reload_control_panel() -> Result<(), String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("argvus")
        .arg("sessionctl")
        .arg("restart")
        .arg("control-panel"),
    )
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr))
      .trim()
      .to_string();
    return Err(if stderr.is_empty() {
      "argvus-sessionctl restart control-panel falhou".into()
    } else {
      stderr
    });
  }
  Ok(())
}

fn reload_session() -> Result<(), String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("argvus")
        .arg("sessionctl")
        .arg("reload"),
    )
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr))
      .trim()
      .to_string();
    return Err(if stderr.is_empty() {
      "argvus-sessionctl reload falhou".into()
    } else {
      stderr
    });
  }
  Ok(())
}

pub(crate) fn reload_session_for_profile() -> Result<(), String> {
  reload_session()
}

/// Applies the `set_border` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set_border(key: &str, value: &str) -> Result<(), String> {
  if key.is_empty() || value.is_empty() {
    return Err("invalid border key/value".into());
  }
  let mut patch = serde_json::Map::new();
  match key {
    "rounded" => {
      let rounded = match value {
        "0" => false,
        "1" => true,
        _ => return Err("invalid rounded value".into()),
      };
      patch.insert("/layout/window/rounded".into(), Value::Bool(rounded));
    }
    "rounding" => {
      let rounding = value
        .parse::<u64>()
        .map_err(|_| "invalid rounding value".to_string())?;
      patch.insert("/layout/window/rounding".into(), Value::from(rounding));
    }
    "thickness" => {
      let thickness = value
        .parse::<u64>()
        .map_err(|_| "invalid thickness value".to_string())?;
      patch.insert("/layout/window/border_size".into(), Value::from(thickness));
    }
    _ => return Err("invalid border key".into()),
  }
  persist_layout_patch(patch)?;
  Ok(())
}

fn persist_layout_patch(patch: serde_json::Map<String, Value>) -> Result<(), String> {
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist layout settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the layout settings".into());
  }
  reload_argvus_config_service()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn wallpaper_paths_are_grouped_by_collection_and_mode() {
    assert_eq!(
      classify_wallpaper(Path::new("argvus-dark.jxl")),
      Some((WallpaperCollection::Abstract, WallpaperMode::Dark))
    );
    assert_eq!(
      classify_wallpaper(Path::new("argvus-light.jxl")),
      Some((WallpaperCollection::Abstract, WallpaperMode::Light))
    );
    assert_eq!(
      classify_wallpaper(Path::new("abstract/dark/gruvbox-abstract-dark.jxl")),
      Some((WallpaperCollection::Abstract, WallpaperMode::Dark))
    );
    assert_eq!(
      classify_wallpaper(Path::new("abstract/light/everforest-abstract-light.jxl")),
      Some((WallpaperCollection::Abstract, WallpaperMode::Light))
    );
    assert_eq!(
      classify_wallpaper(Path::new("landscape/light/gruvbox-landscape-light.jxl")),
      Some((WallpaperCollection::Landscape, WallpaperMode::Light))
    );
    assert_eq!(classify_wallpaper(Path::new("dark/wallpaper.jxl")), None);
    assert_eq!(
      classify_wallpaper(Path::new("abstract/dark/readme.png")),
      None
    );
  }

  #[test]
  fn drop_in_theme_accent_comes_from_its_manifest() {
    let system_config = tempfile::tempdir().unwrap();
    let family_dir = system_config.path().join("appearance/themes.d/nord-light");
    std::fs::create_dir_all(&family_dir).unwrap();
    std::fs::write(
      family_dir.join("theme.toml"),
      "id = \"nord-light\"\nname = \"Nord Light\"\ncategory = \"light\"\naccent = \"#5E81AC\"\n",
    )
    .unwrap();

    assert_eq!(
      discovered_default_accent_in(system_config.path(), "nord-light").as_deref(),
      Some("#5E81AC")
    );
    assert_eq!(
      discovered_default_accent_in(system_config.path(), "nord-light-float").as_deref(),
      Some("#5E81AC")
    );
    assert_eq!(
      discovered_default_accent_in(system_config.path(), "not-installed"),
      None
    );
  }

  #[test]
  /// Executes the `theme_default_accents_exist` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn theme_default_accents_exist() {
    assert_eq!(theme_default_accent("slate-dark-float"), Some("#7391a5"));
    assert_eq!(theme_default_accent("unknown"), None);
  }

  #[test]
  /// Executes the `status_parsers_ignore_unknown_keys` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn status_parsers_ignore_unknown_keys() {
    let mut state = AppearanceState::default();
    parse_spacing_status(
      "waybar_pos=top\nwaybar_top=16\nwaybar_left=14\nwaybar_right=14\nwaybar_bottom=0\ngaps_in=8\ngaps_out_top=8\ngaps_out_left=10\ngaps_out_right=10\ngaps_out_bottom=8\nfuture_property=123\n",
      &mut state,
    );
    parse_borders_status(
      "rounded=1\nrounding=4\nthickness=1\nfuture_property=123\n",
      &mut state,
    );
    assert_eq!(state.waybar_top, 16);
    assert_eq!(state.gaps_out_left, 10);
    assert!(state.rounded);
    assert_eq!(state.rounding, 4);
  }

  #[test]
  fn control_panel_status_keeps_defaults_and_applies_known_cards() {
    let cards = parse_control_panel_cards(
      r#"{"cards":[
        {"id":"weather","enabled":false},
        {"id":"future-card","enabled":false},
        {"id":"power","enabled":true}
      ]}"#,
    );
    assert!(!cards.enabled(ControlPanelCard::Weather));
    assert!(cards.enabled(ControlPanelCard::Power));
    assert!(cards.enabled(ControlPanelCard::User));
    assert!(parse_control_panel_cards("invalid").enabled(ControlPanelCard::User));
  }

  #[test]
  fn control_panel_status_keeps_the_known_availability() {
    let mut previous = ControlPanelCards::default();
    previous.set_available(ControlPanelCard::Bluetooth, false);
    let (cards, order) = control_panel_cards_from_status(
      Some(r#"{"cards":[{"id":"bluetooth","enabled":false}]}"#),
      &previous,
    );
    assert!(!cards.available(ControlPanelCard::Bluetooth));
    assert!(!cards.enabled(ControlPanelCard::Bluetooth));
    assert!(cards.available(ControlPanelCard::Brightness));
    assert_eq!(order.first(), Some(&ControlPanelCard::Bluetooth));

    let (cards, order) = control_panel_cards_from_status(None, &previous);
    assert!(!cards.available(ControlPanelCard::Bluetooth));
    assert!(cards.enabled(ControlPanelCard::Bluetooth));
    assert_eq!(order, ControlPanelCard::ALL.to_vec());
  }

  #[test]
  fn taskbar_shape_status_keeps_offered_values_only() {
    let mut state = AppearanceState::default();
    parse_taskbar_shape("height=36\nborder=2\n", &mut state);
    assert_eq!(state.taskbar_height, 36);
    assert!(state.taskbar_border_enabled);
    assert_eq!(state.taskbar_border_thickness, 2);

    // A border of 0 is off and keeps the last thickness for the next enable.
    parse_taskbar_shape("border=0\n", &mut state);
    assert!(!state.taskbar_border_enabled);
    assert_eq!(state.taskbar_border_thickness, 2);

    parse_taskbar_shape("height=30\nborder=9\n", &mut state);
    assert_eq!(state.taskbar_height, 36, "30 is not offered");
    assert!(!state.taskbar_border_enabled, "9 is not offered");
  }
}
