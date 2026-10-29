//! Window and taskbar spacing, borders and the layout mode, read from and written to the canonical configuration.

use super::commands::{run_argvus_route_output, run_script, run_script_output, script};
use super::*;

pub(super) fn parse_spacing_status(output: &str, state: &mut AppearanceState) {
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

pub(super) fn parse_borders_status(output: &str, state: &mut AppearanceState) {
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
pub(super) fn parse_taskbar_shape(output: &str, state: &mut AppearanceState) {
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

pub(super) fn apply_canonical_layout(state: &mut AppearanceState) {
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

pub(super) fn apply_canonical_taskbar(state: &mut AppearanceState) {
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
  if let Some(value) = boolean(icons, "focus_enabled") {
    state.taskbar_focus_enabled = value;
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

/// The raw status of the layout scripts, parsed after the join.
pub(super) struct LayoutStatus {
  pub(super) utility_group: Option<String>,
  pub(super) spacing: Option<String>,
  pub(super) borders: Option<String>,
  pub(super) taskbar_shape: Option<String>,
}

pub(super) fn read_layout_status() -> LayoutStatus {
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

pub fn set_waybar_position(position: &str) -> Result<(), String> {
  if position != "top" && position != "bottom" {
    return Err("invalid waybar position".into());
  }
  set_spacing("waybar_pos", position)
}

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

/// Turns rounded window corners on or off in one write. Turning them on also
/// sets the rounding to `2`, the value the toggle has always started from.
pub fn set_rounded(enabled: bool) -> Result<(), String> {
  let mut patch = serde_json::Map::new();
  patch.insert("/layout/window/rounded".into(), Value::Bool(enabled));
  if enabled {
    patch.insert("/layout/window/rounding".into(), Value::from(2_u64));
  }
  persist_layout_patch(patch)
}

pub(super) fn persist_layout_patch(patch: serde_json::Map<String, Value>) -> Result<(), String> {
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
