//! Import of settings that older ARGVUS versions kept in separate files: fonts, default apps, shortcuts, language, power, removable devices, Hyprland input and displays.

use super::legacy_formats::{
  lua_call_bodies, lua_fields, parse_input_toml, parse_keybindings_toml, parse_keyboard_lua,
  strip_json_comments,
};
use super::legacy_migration::resolved;
use super::*;

pub(super) fn import_fonts(
  document: &mut ConfigDocument,
  path: Option<PathBuf>,
) -> ConfigResult<bool> {
  let Some(path) = path else {
    return Ok(false);
  };
  if !path.is_file() {
    return Ok(false);
  }
  let legacy = fs::read_to_string(&path)?;
  let state = legacy
    .lines()
    .filter_map(|line| line.trim().split_once('='))
    .map(|(key, value)| (key.trim(), value.trim()))
    .collect::<BTreeMap<_, _>>();
  let mut changed = false;

  if document.get(json_pointers::FONTS_LEGACY).is_none() {
    document.set(json_pointers::FONTS_LEGACY, Value::String(legacy.clone()))?;
    changed = true;
  }

  if document.get("/fonts/targets").is_none() {
    let mut targets = Map::new();
    for target in [
      "taskbar",
      "sysinfo",
      "control_panel",
      "system",
      "apps",
      "terminal",
      "browser",
    ] {
      let mut selection = Map::new();
      let family_key = format!("{target}_family");
      if let Some(value) = state.get(family_key.as_str()) {
        selection.insert("family".into(), Value::String((*value).into()));
      }
      let style_key = format!("{target}_style");
      if let Some(value) = state.get(style_key.as_str()) {
        selection.insert("style".into(), Value::String((*value).into()));
      }
      let size_key = format!("{target}_size");
      if let Some(value) = state.get(size_key.as_str())
        && let Ok(value) = (*value).parse::<u16>()
      {
        selection.insert("size".into(), value.into());
      }
      if !selection.is_empty() {
        targets.insert(target.into(), Value::Object(selection));
      }
    }
    if !targets.is_empty() {
      document.set("/fonts/targets", Value::Object(targets))?;
      changed = true;
    }
  }

  if document.get("/fonts/rendering").is_none() {
    let mut rendering = Map::new();
    if let Some(value) = state.get("antialias") {
      rendering.insert("antialias".into(), Value::Bool(*value == "true"));
    }
    for key in ["hinting", "subpixel"] {
      if let Some(value) = state.get(key) {
        rendering.insert(key.into(), Value::String((*value).into()));
      }
    }
    if let Some(value) = state.get("custom_dpi") {
      rendering.insert("custom_dpi".into(), Value::Bool(*value == "true"));
    }
    if let Some(value) = state.get("dpi")
      && let Ok(value) = (*value).parse::<u16>()
    {
      rendering.insert("dpi".into(), value.into());
    }
    if !rendering.is_empty() {
      document.set("/fonts/rendering", Value::Object(rendering))?;
      changed = true;
    }
  }

  Ok(changed)
}

pub(super) fn import_defaults(
  document: &mut ConfigDocument,
  path: Option<PathBuf>,
) -> ConfigResult<bool> {
  if document.get(json_pointers::DEFAULT_APPS).is_some() {
    return Ok(false);
  }
  let Some(path) = path else {
    return Ok(false);
  };
  let Ok(contents) = fs::read_to_string(&path) else {
    return Ok(false);
  };
  let value: Value = serde_json::from_str(&contents).map_err(ConfigError::InvalidJson)?;
  let Some(object) = value.as_object() else {
    return Ok(false);
  };
  const CATEGORIES: [&str; 11] = [
    "terminal",
    "file_manager",
    "text_editor",
    "terminal_editor",
    "browser",
    "image_viewer",
    "pdf_viewer",
    "video_player",
    "audio_player",
    "archive",
    "launcher",
  ];
  let migrated = object
    .iter()
    .filter(|(key, value)| CATEGORIES.contains(&key.as_str()) && value.is_string())
    .map(|(key, value)| (key.clone(), value.clone()))
    .collect();
  document.set(json_pointers::DEFAULT_APPS, Value::Object(migrated))?;
  Ok(true)
}

pub(super) fn import_keyboard_shortcuts(
  document: &mut ConfigDocument,
  root: &Path,
) -> ConfigResult<bool> {
  if document.get(json_pointers::KEYBOARD_SHORTCUTS).is_some() {
    return Ok(false);
  }
  let old_value = if let Some(value) = document.get("/hyprland/keybindings") {
    Some(value.clone())
  } else if let Some(path) = resolved(root, "keybindings.toml") {
    parse_keybindings_toml(&path)?
  } else {
    None
  };
  let Some(value) = old_value else {
    return Ok(false);
  };
  let shortcuts = legacy_keybindings_to_shortcuts(&value);
  document.set(json_pointers::KEYBOARD_SHORTCUTS, shortcuts)?;
  document.unset("/hyprland/keybindings")?;
  Ok(true)
}

pub(super) fn legacy_keybindings_to_shortcuts(value: &Value) -> Value {
  let mut shortcuts = Map::new();
  if let Some(entries) = value.as_object() {
    for (id, binding) in entries {
      let config_key = config_key_for_binding(id);
      let shortcut = binding
        .get("enabled")
        .and_then(Value::as_bool)
        .filter(|enabled| !enabled)
        .map(|_| Value::Null)
        .or_else(|| binding.get("keys").cloned())
        .unwrap_or(Value::Null);
      shortcuts.insert(config_key, shortcut);
    }
  }
  Value::Object(shortcuts)
}

pub(crate) fn config_key_for_binding(id: &str) -> String {
  match id {
    "window.close" => "close_window",
    "window.drag_mouse" => "drag_window__floating_window_only",
    "window.maximize" => "maximize_window__toggle",
    "app.browser" => "open_browser",
    "system.about" => "open_about",
    "appearance.theme" => "open_theme_selector",
    _ => return id.replace(['.', '-'], "_"),
  }
  .into()
}

pub(super) fn manifest_shortcut_defaults() -> BTreeMap<String, Value> {
  let path = env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(session_paths::PACKAGED_SYSTEM_CONFIG))
    .join("hyprland/keybindings.json");
  let Ok(contents) = fs::read_to_string(path) else {
    return BTreeMap::new();
  };
  let Ok(manifest) = serde_json::from_str::<Value>(&contents) else {
    return BTreeMap::new();
  };
  let mut defaults = BTreeMap::new();
  let Some(bindings) = manifest.get("bindings").and_then(Value::as_array) else {
    return defaults;
  };
  for binding in bindings {
    let Some(id) = binding.get("id").and_then(Value::as_str) else {
      continue;
    };
    let key = binding
      .get("config_key")
      .and_then(Value::as_str)
      .map(str::to_owned)
      .unwrap_or_else(|| config_key_for_binding(id));
    if let Some(shortcut) = binding.get("keys") {
      defaults.insert(key, shortcut.clone());
    }
  }
  defaults
}

pub(super) fn import_language(
  document: &mut ConfigDocument,
  path: Option<PathBuf>,
) -> ConfigResult<bool> {
  if document.get(json_pointers::SESSION_LANGUAGE).is_some() {
    return Ok(false);
  }
  let Some(path) = path else {
    return Ok(false);
  };
  let Ok(value) = fs::read_to_string(&path) else {
    return Ok(false);
  };
  let language = value.trim();
  if language.is_empty() {
    return Ok(false);
  }
  document.set(
    json_pointers::SESSION_LANGUAGE,
    Value::String(language.replace('_', "-")),
  )?;
  Ok(true)
}

pub(super) fn import_power(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  let mut changed = false;
  if document.get(json_pointers::POWER_KEEP_AWAKE).is_none()
    && let Some(path) = resolved(root, ".keep-awake")
    && let Ok(value) = fs::read_to_string(&path)
    && value.trim() == "enabled"
  {
    document.set(json_pointers::POWER_KEEP_AWAKE, Value::Bool(true))?;
    changed = true;
  }
  let Some(path) = resolved(root, "hypr/hypridle.conf") else {
    return Ok(changed);
  };
  let Ok(contents) = fs::read_to_string(&path) else {
    return Ok(changed);
  };
  let mut timeout = None;
  let mut action = String::new();
  for line in contents.lines() {
    let line = line.trim();
    if let Some(value) = line.strip_prefix("timeout =") {
      timeout = value.trim().parse::<u64>().ok();
    } else if let Some(value) = line.strip_prefix("on-timeout =") {
      action = value.trim().to_ascii_lowercase();
    } else if line == "}" {
      if let Some(seconds) = timeout.take() {
        let key = if action.contains("dpms") && action.contains("off") {
          "screen_off_minutes"
        } else if action.contains("lock") {
          "lock_minutes"
        } else {
          ""
        };
        if !key.is_empty() {
          let pointer = format!("/power/{key}");
          if document.get(&pointer).is_none() {
            document.set(&pointer, Value::from(seconds / 60))?;
            changed = true;
          }
        }
      }
      action.clear();
    }
  }
  Ok(changed)
}

pub(super) fn import_removable_devices(
  document: &mut ConfigDocument,
  root: &Path,
) -> ConfigResult<bool> {
  if document.get(json_pointers::REMOVABLE_DEVICES).is_some() {
    return Ok(false);
  }
  let Some(path) = resolved(root, "removable-devices/config.json") else {
    return Ok(false);
  };
  let Ok(contents) = fs::read_to_string(&path) else {
    return Ok(false);
  };
  let value: Value =
    serde_json::from_str(&strip_json_comments(&contents)).map_err(ConfigError::InvalidJson)?;
  if !value.is_object() {
    return Ok(false);
  }
  document.set(json_pointers::REMOVABLE_DEVICES, value)?;
  Ok(true)
}

pub(super) fn import_hyprland(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  let mut changed = false;
  if document.get(json_pointers::HYPRLAND_INPUT).is_none()
    && let Some(path) = resolved(root, "input.toml")
    && let Some(value) = parse_input_toml(&path)?
  {
    document.set(json_pointers::HYPRLAND_INPUT, value)?;
    changed = true;
  }
  if document.get(json_pointers::HYPRLAND_KEYBOARD).is_none()
    && let Some(path) = resolved(root, "generated/hypr/input.lua")
    && let Some(value) = parse_keyboard_lua(&path)?
  {
    document.set(json_pointers::HYPRLAND_KEYBOARD, value)?;
    changed = true;
  }
  Ok(changed)
}

pub(super) fn import_displays(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  if document.get(json_pointers::DISPLAYS).is_some() {
    return Ok(false);
  }
  let Some(path) = resolved(root, "generated/hypr/monitors.lua") else {
    return Ok(false);
  };
  let Ok(contents) = fs::read_to_string(&path) else {
    return Ok(false);
  };
  let mut monitors = Map::new();
  let mut workspaces: Map<String, Value> = Map::new();
  let mut primary_monitor = None;

  for body in lua_call_bodies(&contents, "hl.monitor") {
    let fields = lua_fields(&body);
    let Some(name) = fields.get("output").and_then(Value::as_str) else {
      continue;
    };
    let mut monitor = Map::new();
    for key in [
      "mode",
      "position",
      "scale",
      "transform",
      "vrr",
      "mirror",
      "bitdepth",
      "disabled",
      "supports_hdr",
      "hdr",
      "sdrbrightness",
      "sdrsaturation",
    ] {
      if let Some(value) = fields.get(key) {
        let canonical = match key {
          "supports_hdr" => "hdr",
          "sdrbrightness" => "sdr_brightness",
          "sdrsaturation" => "sdr_saturation",
          _ => key,
        };
        monitor.insert(canonical.into(), value.clone());
      }
    }
    monitors.insert(name.to_string(), Value::Object(monitor));
  }

  for body in lua_call_bodies(&contents, "hl.workspace_rule") {
    let fields = lua_fields(&body);
    let (Some(workspace), Some(monitor)) = (
      fields.get("workspace").and_then(Value::as_str),
      fields.get("monitor").and_then(Value::as_str),
    ) else {
      continue;
    };
    let Ok(workspace) = workspace.parse::<u64>() else {
      continue;
    };
    let ids = workspaces
      .entry(monitor.to_string())
      .or_insert_with(|| Value::Array(Vec::new()));
    if let Some(ids) = ids.as_array_mut()
      && !ids.iter().any(|value| value.as_u64() == Some(workspace))
    {
      ids.push(Value::Number(workspace.into()));
    }
    if workspace == 1 && fields.get("default") == Some(&Value::Bool(true)) {
      primary_monitor = Some(Value::String(monitor.to_string()));
    }
  }

  if monitors.is_empty() && workspaces.is_empty() {
    return Ok(false);
  }
  for ids in workspaces.values_mut() {
    if let Some(ids) = ids.as_array_mut() {
      ids.sort_by_key(Value::as_u64);
    }
  }
  let mut displays = Map::new();
  displays.insert(
    "primary_monitor".into(),
    primary_monitor.unwrap_or(Value::Null),
  );
  displays.insert("monitors".into(), Value::Object(monitors));
  displays.insert("workspaces".into(), Value::Object(workspaces));
  document.set(json_pointers::DISPLAYS, Value::Object(displays))?;
  Ok(true)
}
