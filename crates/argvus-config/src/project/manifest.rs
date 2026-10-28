//! Projection manifest: section hashes of the last run and the reload targets a change implies.

use super::*;

/// Whether the last projection changed something the compositor reads.
///
/// A missing or unreadable manifest answers `true`, so a caller that skips the
/// compositor reload on `false` never skips one that may be needed.
pub fn last_projection_affects_compositor() -> bool {
  let Some(manifest) = argvus_state_root()
    .ok()
    .and_then(|state_root| read_manifest(&state_root.join(PROJECTION_MANIFEST_FILE)))
  else {
    return true;
  };
  manifest_affects_compositor(&manifest)
}

pub(super) fn manifest_affects_compositor(manifest: &Value) -> bool {
  manifest
    .get("affected_consumers")
    .and_then(Value::as_array)
    .is_none_or(|consumers| {
      consumers
        .iter()
        .any(|consumer| consumer.as_str() == Some(HYPRLAND_CONSUMER))
    })
}

pub(super) fn read_manifest(path: &Path) -> Option<Value> {
  let value: Value = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?;
  (value.get("schema_version").and_then(Value::as_u64) == Some(MANIFEST_SCHEMA_VERSION as u64))
    .then_some(value)
}

pub(super) fn section_hash(document: &ConfigDocument, section: &str) -> ConfigResult<String> {
  value_hash(document.sections.get(section))
}

pub(super) fn value_hash(value: Option<&Value>) -> ConfigResult<String> {
  let bytes = serde_json::to_vec(value.unwrap_or(&Value::Null))?;
  let mut hasher = DefaultHasher::new();
  bytes.hash(&mut hasher);
  Ok(format!("{:016x}", hasher.finish()))
}

pub(super) fn manifest_field_changed(
  previous: Option<&Value>,
  field: &str,
  current_hash: &str,
) -> bool {
  previous
    .and_then(|manifest| manifest.get("appearance_values"))
    .and_then(Value::as_object)
    .and_then(|values| values.get(field))
    .and_then(Value::as_str)
    != Some(current_hash)
}

/// What the runtime half of a reload does for the changed sections. Only
/// `taskbar`, `widget_telemetry` and `control_panel` recreate a surface (and
/// show the splash); `consumers` reloads the themed consumers in place (the
/// Waybar stylesheets, the Control Panel, Hyprland colours, Dunst, btop and
/// kitty) and `wallpaper` goes through hyprpaper's IPC.
pub(super) fn reload_targets(
  changed_sections: &[&str],
  appearance_visual_changed: bool,
  wallpaper_changed: bool,
  taskbar_layout_changed: bool,
) -> serde_json::Map<String, Value> {
  let mut targets = serde_json::Map::new();
  let mut enable = |name: &str| {
    targets.insert(name.to_owned(), Value::Bool(true));
  };

  if appearance_visual_changed {
    enable("hyprland");
    enable("consumers");
    enable("polkit");
    enable("wallpaper");
  } else if wallpaper_changed {
    enable("wallpaper");
  }
  if changed_sections.contains(&"effects") {
    enable("hyprland");
    enable("consumers");
  }
  if changed_sections.contains(&"layout") {
    enable("hyprland");
    if taskbar_layout_changed {
      enable("taskbar");
      enable("widget_telemetry");
    }
  }
  if changed_sections.contains(&"fonts") {
    enable("consumers");
  }
  if changed_sections
    .iter()
    .any(|section| ["default_apps", "keyboard_shortcuts", "hyprland", "displays"].contains(section))
  {
    enable("hyprland");
  }
  if changed_sections.contains(&"control_panel") {
    enable("control_panel");
  }
  if changed_sections.contains(&"power") {
    enable("hypridle");
  }

  targets
}
