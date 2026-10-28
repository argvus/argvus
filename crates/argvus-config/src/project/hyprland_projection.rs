//! Hyprland projections: layout, input, window rules, keyboard, shortcuts, monitors, hypridle and the shared Lua state.

use super::file_writes::{atomic_write, atomic_write_lua};
use super::*;

/// Projects the canonical layout values into the compatibility inputs consumed
/// by the official Hyprland spacing and border adapters. These files remain
/// derived state; the modular configuration files are the only persistent source of truth.
/// Compute the effective rounding value.
/// When `rounded` is false (disabled), rounding must be 0 regardless of the configured value.
/// When `rounded` is true, clamp rounding to the valid range [2..10].
pub(super) fn effective_rounding(rounded: bool, rounding: u64) -> u64 {
  if !rounded { 0 } else { rounding.clamp(2, 10) }
}

pub(super) fn project_layout(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(layout) = document.get("/layout").and_then(Value::as_object) else {
    return Ok(());
  };

  if let Some(window) = layout.get("window").and_then(Value::as_object) {
    let mut lines = Vec::new();
    for key in [
      "gaps_in",
      "gaps_out_top",
      "gaps_out_left",
      "gaps_out_right",
      "gaps_out_bottom",
    ] {
      if let Some(value) = window.get(key).and_then(Value::as_u64) {
        lines.push(format!("{key}={value}"));
      }
    }
    if let Some(value) = window.get("gaps_out").and_then(Value::as_u64) {
      lines.push(format!("gaps_out={value}"));
    }
    if !lines.is_empty() {
      atomic_write(
        &paths.data.join(".spaces"),
        &format!("{}\n", lines.join("\n")),
      )?;
    }

    let mut border_lines = Vec::new();
    let rounded = window
      .get("rounded")
      .and_then(Value::as_bool)
      .unwrap_or(false);
    if let Some(value) = window.get("rounded").and_then(Value::as_bool) {
      border_lines.push(format!("rounded={}", u8::from(value)));
    }
    // Use effective rounding: when rounded is false, force rounding to 0
    let rounding = window.get("rounding").and_then(Value::as_u64).unwrap_or(0);
    let effective_rounding_value = effective_rounding(rounded, rounding);
    border_lines.push(format!("rounding={effective_rounding_value}"));

    // border_size (mapped to thickness) is independent of rounded state
    if let Some(value) = window.get("border_size").and_then(Value::as_u64) {
      border_lines.push(format!("thickness={value}"));
    }
    if !border_lines.is_empty() {
      atomic_write(
        &paths.data.join(".borders"),
        &format!("{}\n", border_lines.join("\n")),
      )?;
    }
  }

  if let Some(taskbar) = layout.get("taskbar").and_then(Value::as_object) {
    let mut lines = Vec::new();
    if let Some(value) = taskbar.get("position").and_then(Value::as_str) {
      lines.push(format!("waybar_pos={value}"));
    }
    for (config_key, runtime_key) in [
      ("margin_top", "waybar_top"),
      ("margin_left", "waybar_left"),
      ("margin_right", "waybar_right"),
      ("margin_bottom", "waybar_bottom"),
    ] {
      if let Some(value) = taskbar.get(config_key).and_then(Value::as_u64) {
        lines.push(format!("{runtime_key}={value}"));
      }
    }
    if !lines.is_empty() {
      let path = paths.data.join(".spaces");
      let existing = fs::read_to_string(&path).unwrap_or_default();
      let prefix = existing.trim_end();
      let content = if prefix.is_empty() {
        lines.join("\n")
      } else {
        format!("{prefix}\n{}", lines.join("\n"))
      };
      atomic_write(&path, &format!("{content}\n"))?;
    }
  }
  Ok(())
}

pub(super) fn project_input(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(input) = document
    .get(json_pointers::HYPRLAND_INPUT)
    .and_then(Value::as_object)
  else {
    return Ok(());
  };
  let mouse = input.get("mouse").and_then(Value::as_object);
  let touchpad = input.get("touchpad").and_then(Value::as_object);
  let toml = format!(
    "[mouse]\nsensitivity = {}\naccel_profile = \"{}\"\nnatural_scroll = {}\nscroll_factor = {}\nleft_handed = {}\n\n[touchpad]\nnatural_scroll = {}\ntap_to_click = {}\ntap_and_drag = {}\ntwo_finger_right_click = {}\ndisable_while_typing = {}\n",
    object_value(mouse, "sensitivity").unwrap_or(&Value::from(0)),
    object_value(mouse, "accel_profile")
      .and_then(Value::as_str)
      .unwrap_or(""),
    object_value(mouse, "natural_scroll").unwrap_or(&Value::Bool(false)),
    object_value(mouse, "scroll_factor").unwrap_or(&Value::from(1)),
    object_value(mouse, "left_handed").unwrap_or(&Value::Bool(false)),
    object_value(touchpad, "natural_scroll").unwrap_or(&Value::Bool(false)),
    object_value(touchpad, "tap_to_click").unwrap_or(&Value::Bool(true)),
    object_value(touchpad, "tap_and_drag").unwrap_or(&Value::Bool(true)),
    object_value(touchpad, "two_finger_right_click").unwrap_or(&Value::Bool(true)),
    object_value(touchpad, "disable_while_typing").unwrap_or(&Value::Bool(true))
  );
  atomic_write(&paths.data.join("hypr/input.toml"), &toml)?;
  let lua = format!(
    "-- Generated by argvus-config. Do not edit.\nreturn {{\n  sensitivity = {},\n  accel_profile = {},\n  natural_scroll = {},\n  scroll_factor = {},\n  left_handed = {},\n  touchpad = {{\n    natural_scroll = {},\n    tap_to_click = {},\n    tap_and_drag = {},\n    clickfinger_behavior = {},\n    disable_while_typing = {},\n  }},\n}}\n",
    object_value(mouse, "sensitivity").unwrap_or(&Value::from(0)),
    serde_json::to_string(
      object_value(mouse, "accel_profile")
        .and_then(Value::as_str)
        .unwrap_or("flat")
    )?,
    object_value(mouse, "natural_scroll").unwrap_or(&Value::Bool(false)),
    object_value(mouse, "scroll_factor").unwrap_or(&Value::from(1)),
    object_value(mouse, "left_handed").unwrap_or(&Value::Bool(false)),
    object_value(touchpad, "natural_scroll").unwrap_or(&Value::Bool(false)),
    object_value(touchpad, "tap_to_click").unwrap_or(&Value::Bool(true)),
    object_value(touchpad, "tap_and_drag").unwrap_or(&Value::Bool(true)),
    object_value(touchpad, "two_finger_right_click").unwrap_or(&Value::Bool(false)),
    object_value(touchpad, "disable_while_typing").unwrap_or(&Value::Bool(true)),
  );
  atomic_write_lua(&paths.generated.join("hypr/input-settings.lua"), &lua)
}

pub(super) fn project_window_rules(
  document: &ConfigDocument,
  paths: &ArgvusPaths,
) -> ConfigResult<()> {
  let Some(rules) = document
    .get("/hyprland/window_rules")
    .and_then(Value::as_object)
  else {
    return Ok(());
  };
  let lua = render_window_rules(rules)?;
  atomic_write_lua(&paths.generated.join("hypr/window-rules.lua"), &lua)
}

pub(super) fn render_window_rules(rules: &serde_json::Map<String, Value>) -> ConfigResult<String> {
  let mut lua = String::from("-- Generated by argvus-config. Do not edit.\nreturn {\n");
  for (name, rule) in rules {
    let workspace = rule
      .get("workspace")
      .and_then(Value::as_u64)
      .filter(|workspace| (1..=10).contains(workspace))
      .ok_or_else(|| {
        ConfigError::Invalid(format!(
          "hyprland.window_rules.{name}.workspace must be an integer from 1 to 10"
        ))
      })?;
    let classes = rule
      .get("classes")
      .and_then(Value::as_array)
      .ok_or_else(|| {
        ConfigError::Invalid(format!(
          "hyprland.window_rules.{name}.classes must be a list"
        ))
      })?;
    lua.push_str(&format!("  {{ workspace = {workspace}, classes = {{"));
    for class in classes {
      let class = class
        .as_str()
        .filter(|class| is_class_pattern(class))
        .ok_or_else(|| {
          ConfigError::Invalid(format!(
            "hyprland.window_rules.{name}.classes contains an invalid class pattern"
          ))
        })?;
      lua.push_str(&format!(" \"{class}\","));
    }
    lua.push_str(" } },\n");
  }
  lua.push_str("}\n");
  Ok(lua)
}

pub(super) fn is_class_pattern(value: &str) -> bool {
  !value.is_empty()
    && value.len() <= 128
    && value
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || "._*+?-[]()|^$".contains(c))
}

pub(super) fn object_value<'a>(
  object: Option<&'a serde_json::Map<String, Value>>,
  key: &str,
) -> Option<&'a Value> {
  object.and_then(|value| value.get(key))
}

pub(super) fn project_keyboard(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(keyboard) = document
    .get(json_pointers::HYPRLAND_KEYBOARD)
    .and_then(Value::as_object)
  else {
    return Ok(());
  };
  let quote =
    |key: &str| serde_json::to_string(keyboard.get(key).and_then(Value::as_str).unwrap_or(""));
  atomic_write_lua(
    &paths.generated.join("hypr/input.lua"),
    &format!(
      "-- Generated by argvus-config. Do not edit.\nreturn {{ kb_layout = {}, kb_variant = {}, kb_options = {}, }}\n",
      quote("layout")?,
      quote("variant")?,
      quote("options")?
    ),
  )
}

pub(super) fn project_shortcuts(
  document: &ConfigDocument,
  paths: &ArgvusPaths,
) -> ConfigResult<()> {
  let Some(shortcuts) = document
    .get(json_pointers::KEYBOARD_SHORTCUTS)
    .and_then(Value::as_object)
  else {
    return Ok(());
  };
  let mut lua = String::from("-- Generated by argvus-config. Do not edit.\nreturn {\n");
  let mut toml = String::new();
  let binding_ids = shortcut_binding_ids();
  for (key, shortcut) in shortcuts {
    let (keys, enabled) = match shortcut {
      Value::String(value) => (Some(value.as_str()), true),
      Value::Null => (None, false),
      _ => {
        return Err(ConfigError::Invalid(format!(
          "keyboard_shortcuts.{key} must be a string or null"
        )));
      }
    };
    let binding_id = binding_ids.get(key).map(String::as_str).unwrap_or(key);
    lua.push_str(&format!("  [{}] = {{", serde_json::to_string(binding_id)?));
    if let Some(keys) = keys {
      lua.push_str(&format!(" keys = {},", serde_json::to_string(keys)?));
    }
    lua.push_str(&format!(" enabled = {enabled} }},\n"));
    toml.push_str(&format!("[keybindings.{}]\n", key.replace(['.', '-'], "_")));
    toml.push_str(&format!("enabled = {enabled}\n"));
    if let Some(keys) = keys {
      toml.push_str(&format!("keys = {}\n", serde_json::to_string(keys)?));
    }
    toml.push('\n');
  }
  lua.push_str("}\n");
  atomic_write_lua(&paths.generated.join("hypr/keybindings.lua"), &lua)?;
  atomic_write(&paths.data.join("hypr/keybindings.toml"), &toml)
}

pub(super) fn shortcut_binding_ids() -> std::collections::BTreeMap<String, String> {
  let manifest_path = env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(session_paths::PACKAGED_SYSTEM_CONFIG))
    .join("hyprland/keybindings.json");
  let Ok(contents) = fs::read_to_string(manifest_path) else {
    return std::collections::BTreeMap::new();
  };
  let Ok(manifest) = serde_json::from_str::<Value>(&contents) else {
    return std::collections::BTreeMap::new();
  };
  manifest
    .get("bindings")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
    .filter_map(|binding| {
      let binding_id = binding.get("id")?.as_str()?;
      let config_key = binding
        .get("config_key")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| crate::config_key_for_binding(binding_id));
      Some((config_key, binding_id.to_owned()))
    })
    .collect()
}

pub(super) fn project_displays(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(displays) = document
    .get(json_pointers::DISPLAYS)
    .and_then(Value::as_object)
  else {
    return Ok(());
  };
  let mut output = String::from("-- Generated by argvus-config. Do not edit.\n");
  if let Some(monitors) = displays.get("monitors").and_then(Value::as_object) {
    for (output_name, monitor) in monitors {
      output.push_str(&format!(
        "hl.monitor({{ output = {}",
        serde_json::to_string(output_name)?
      ));
      if monitor.get("disabled").and_then(Value::as_bool) == Some(true) {
        output.push_str(", disabled = true");
      }
      for key in [
        "mode",
        "position",
        "scale",
        "transform",
        "mirror",
        "bitdepth",
        "vrr",
        "hdr",
        "sdr_brightness",
        "sdr_saturation",
      ] {
        if let Some(value) = monitor.get(key) {
          let runtime_key = match key {
            "hdr" => "supports_hdr",
            "sdr_brightness" => "sdrbrightness",
            "sdr_saturation" => "sdrsaturation",
            _ => key,
          };
          output.push_str(&format!(", {runtime_key} = {}", lua_value(value)?));
        }
      }
      output.push_str(" })\n");
    }
  }
  if let Some(workspaces) = displays.get("workspaces").and_then(Value::as_object) {
    let primary_monitor = displays.get("primary_monitor").and_then(Value::as_str);
    let mut emitted_workspace_one = false;
    for (monitor, workspace_ids) in workspaces {
      let Some(workspace_ids) = workspace_ids.as_array() else {
        continue;
      };
      for (index, workspace_id) in workspace_ids.iter().filter_map(Value::as_u64).enumerate() {
        let is_default = primary_monitor == Some(monitor.as_str()) && index == 0;
        emitted_workspace_one |= workspace_id == 1;
        output.push_str(&format!(
          "hl.workspace_rule({{ workspace = {}, monitor = {}{} }})\n",
          serde_json::to_string(&workspace_id.to_string())?,
          serde_json::to_string(monitor)?,
          if is_default { ", default = true" } else { "" }
        ));
      }
    }
    if !emitted_workspace_one && let Some(primary_monitor) = primary_monitor {
      output.push_str(&format!(
        "hl.workspace_rule({{ workspace = \"1\", monitor = {}, default = true }})\n",
        serde_json::to_string(primary_monitor)?
      ));
    }
  }
  atomic_write_lua(&paths.generated.join("hypr/monitors.lua"), &output)
}

pub(super) fn scalar(
  object: &serde_json::Map<String, Value>,
  key: &str,
  default: impl Into<Value>,
) -> Value {
  object.get(key).cloned().unwrap_or_else(|| default.into())
}

pub(super) fn lua_value(value: &Value) -> ConfigResult<String> {
  Ok(match value {
    Value::String(value) => serde_json::to_string(value)?,
    Value::Bool(value) => value.to_string(),
    Value::Number(value) => value.to_string(),
    _ => serde_json::to_string(&crate::sorted_value(value))?,
  })
}

/// Writes `hypr/state.lua`, the values `hyprland.lua` needs when it loads.
///
/// The Lua configuration reads this derived file instead of spawning
/// `argvus config get` or `systemd-detect-virt` while Hyprland starts. The
/// file is rewritten on every projection, so it always matches the canonical
/// document that the reload targets were computed from.
pub(super) fn project_hypr_state(
  document: &ConfigDocument,
  paths: &ArgvusPaths,
) -> ConfigResult<()> {
  let layout_is_float = document
    .get(json_pointers::LAYOUT_VARIANT)
    .and_then(Value::as_str)
    == Some("float");
  let accent_custom = document
    .get(json_pointers::APPEARANCE_ACCENT_CUSTOM)
    .and_then(Value::as_bool)
    .unwrap_or(false);
  let effects = document
    .get("/effects")
    .cloned()
    .unwrap_or_else(|| Value::Object(Map::new()));
  let published_virtualization = env::var("ARGVUS_VIRTUALIZATION").ok();
  let detected_virtualization = detect_virtualization();
  let virtual_machine = virtual_machine_state(
    published_virtualization.as_deref(),
    detected_virtualization.as_deref(),
  );

  let background_color = splash_background_color(&crate::accent::effective_theme(
    document
      .get(json_pointers::APPEARANCE_THEME)
      .and_then(Value::as_str)
      .unwrap_or_default(),
  ));

  let lua = format!(
    "-- Generated by argvus-config. Do not edit.\nreturn {{\n  layout_variant = {},\n  accent_custom = {accent_custom},\n  virtual_machine = {virtual_machine},\n  background_color = \"{background_color}\",\n  effects = {},\n}}\n",
    if layout_is_float {
      "\"float\""
    } else {
      "\"sticky\""
    },
    lua_table(&effects)
  );
  atomic_write_lua(&paths.generated.join("hypr/state.lua"), &lua)
}

/// Hyprland's background color for `theme`: the background of the loading
/// splash, from the table both splash surfaces use.
///
/// Hyprland paints it between the login handoff surface and the session
/// overlay; with the same color the output never flashes another shade.
pub(super) fn splash_background_color(theme: &str) -> String {
  let background = argvus_loading_theme_core::theme_colors(theme)
    .map(|(background, _, _)| background)
    .unwrap_or(argvus_loading_theme_core::DEFAULT_BACKGROUND);
  let rgb = argvus_loading_theme_core::hex_to_rgb(background)
    .or_else(|| {
      argvus_loading_theme_core::hex_to_rgb(argvus_loading_theme_core::DEFAULT_BACKGROUND)
    })
    .unwrap_or_default();
  format!("rgb({rgb:06x})")
}

/// Decides whether the session is a virtual machine.
///
/// `ARGVUS_VIRTUALIZATION`, published by argvus-start, is authoritative when
/// set. Otherwise the detector output decides. Both report `none` on physical
/// hardware, and that must never count as a virtual machine.
pub(super) fn virtual_machine_state(published: Option<&str>, detected: Option<&str>) -> bool {
  match published.filter(|value| !value.is_empty()) {
    Some(published) => published != "none",
    None => detected.is_some_and(|value| !value.is_empty() && value != "none"),
  }
}

/// Runs `systemd-detect-virt --vm`. Returns `None` when the detector is missing.
pub(super) fn detect_virtualization() -> Option<String> {
  let output = Command::new("systemd-detect-virt")
    .arg("--vm")
    .output()
    .ok()?;
  Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Renders a JSON value as a Lua table literal, keeping object keys as strings.
pub(super) fn lua_table(value: &Value) -> String {
  match value {
    Value::Object(entries) => {
      let fields: Vec<String> = entries
        .iter()
        .map(|(key, field)| format!("[{}] = {}", lua_string(key), lua_table(field)))
        .collect();
      format!("{{ {} }}", fields.join(", "))
    }
    Value::Array(items) => {
      let elements: Vec<String> = items.iter().map(lua_table).collect();
      format!("{{ {} }}", elements.join(", "))
    }
    Value::String(text) => lua_string(text),
    Value::Bool(flag) => flag.to_string(),
    Value::Number(number) => number.to_string(),
    Value::Null => "nil".to_owned(),
  }
}

/// Quotes text as a Lua string literal. Control characters use Lua's
/// three-digit decimal escapes, so no input can end the literal early.
pub(super) fn lua_string(text: &str) -> String {
  let mut quoted = String::from("\"");
  for character in text.chars() {
    match character {
      '\\' => quoted.push_str("\\\\"),
      '"' => quoted.push_str("\\\""),
      '\n' => quoted.push_str("\\n"),
      '\r' => quoted.push_str("\\r"),
      control if control.is_control() => {
        quoted.push_str(&format!("\\{:03}", control as u32));
      }
      plain => quoted.push(plain),
    }
  }
  quoted.push('"');
  quoted
}

pub(super) fn project_hypridle(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let lock_minutes = document
    .get("/power/lock_minutes")
    .and_then(Value::as_u64)
    .unwrap_or(30);
  let screen_minutes = document
    .get("/power/screen_off_minutes")
    .and_then(Value::as_u64)
    .unwrap_or(0);
  let mut text = String::from(
    "# ARGVUS hypridle configuration\n# Generated from modular ARGVUS configuration by argvus-config project. Do not edit.\n\ngeneral {\n  lock_cmd = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock\n  before_sleep_cmd = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock\n  after_sleep_cmd = argvus config hyprland dpms on\n}\n",
  );
  if lock_minutes > 0 {
    text.push_str(&format!("\nlistener {{\n  timeout = {}\n  on-timeout = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock\n}}\n", lock_minutes * 60));
  }
  if screen_minutes > 0 {
    text.push_str(&format!("\nlistener {{\n  timeout = {}\n  on-timeout = argvus config hyprland dpms off\n  on-resume = argvus config hyprland dpms on\n}}\n", screen_minutes * 60));
  }
  // hypridle is an application-owned runtime configuration. Keep the
  // generated file at the same path resolved by paths_config() and by the
  // Control Center; leaving a second copy makes the running service consume
  // stale values.
  let runtime_path = paths.data.join("hypr/hypridle.conf");
  atomic_write(&runtime_path, &text)?;
  // Retire the pre-`data/` runtime file so the idle service cannot fall
  // back to a stale copy at the configuration root.
  let obsolete_path = paths.root.join("hypr/hypridle.conf");
  if obsolete_path != runtime_path && obsolete_path.exists() {
    fs::remove_file(obsolete_path)?;
  }
  Ok(())
}
