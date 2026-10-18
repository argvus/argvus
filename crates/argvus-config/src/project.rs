//! Typed canonical-configuration projections.
//!
//! This module owns the structured part of projection.  Component-specific
//! theme/effect adapters remain external processes until their public payload
//! contracts can be moved without duplicating asset ownership.

use crate::{ConfigDocument, ConfigError, ConfigResult, ConfigStore};
use serde_json::{Map, Value};
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const MANIFEST_SCHEMA_VERSION: u32 = 3;
const SECTIONS: [&str; 14] = [
  "appearance",
  "layout",
  "effects",
  "fonts",
  "control_panel",
  "default_apps",
  "keyboard_shortcuts",
  "hyprland",
  "displays",
  "power",
  "session",
  "calendar",
  "removable_devices",
  "audio",
];

#[derive(Debug, Clone)]
pub struct ArgvusPaths {
  pub root: PathBuf,
  pub data: PathBuf,
  pub generated: PathBuf,
  pub internal: PathBuf,
  pub backups: PathBuf,
  pub projection_manifest: PathBuf,
}

impl ArgvusPaths {
  pub fn from_store(store: &ConfigStore) -> ConfigResult<Self> {
    let root = store
      .path
      .parent()
      .ok_or(ConfigError::MissingConfigDirectory)?
      .to_path_buf();
    let state_home = env::var_os("XDG_STATE_HOME")
      .map(PathBuf::from)
      .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
      .ok_or(ConfigError::MissingConfigDirectory)?;
    let state_root = state_home.join("argvus");
    Ok(Self {
      data: root.join("data"),
      generated: root.join("data/generated"),
      internal: root.join("data/internal"),
      backups: root.join("data/backups"),
      projection_manifest: state_root.join("config-projection.json"),
      root,
    })
  }
}

pub fn project(store: &ConfigStore, force: bool) -> ConfigResult<String> {
  let document = store.ensure()?;
  let paths = ArgvusPaths::from_store(store)?;
  fs::create_dir_all(&paths.data)?;
  fs::create_dir_all(&paths.generated)?;
  fs::create_dir_all(&paths.internal)?;

  let previous_manifest = if !force {
    read_manifest(&paths.projection_manifest)
  } else {
    None
  };
  let mut section_hashes = serde_json::Map::new();
  let mut changed_sections = Vec::new();
  for section in SECTIONS {
    let hash = section_hash(&document, section)?;
    if previous_manifest
      .as_ref()
      .and_then(|manifest| manifest.get("sections"))
      .and_then(|sections| sections.get(section))
      .and_then(Value::as_str)
      != Some(hash.as_str())
    {
      changed_sections.push(section);
    }
    section_hashes.insert(section.to_owned(), Value::String(hash));
  }

  let theme_hash = value_hash(document.get("/appearance/theme"))?;
  let accent_hash = value_hash(document.get("/appearance/accent"))?;
  let accent_custom_hash = value_hash(document.get("/appearance/accent_custom"))?;
  let gtk_mode_hash = value_hash(document.get("/appearance/gtk_mode"))?;
  let wallpaper_hash = value_hash(document.get("/appearance/wallpaper"))?;
  let appearance_runtime_changed = previous_manifest
    .as_ref()
    .and_then(|manifest| manifest.get("appearance_runtime"))
    .map(|runtime| {
      runtime.get("theme").and_then(Value::as_str) != Some(theme_hash.as_str())
        || runtime.get("accent").and_then(Value::as_str) != Some(accent_hash.as_str())
    })
    .unwrap_or(true);
  let appearance_visual_changed = changed_sections.contains(&"appearance")
    && (manifest_field_changed(previous_manifest.as_ref(), "theme", &theme_hash)
      || manifest_field_changed(previous_manifest.as_ref(), "accent", &accent_hash)
      || manifest_field_changed(
        previous_manifest.as_ref(),
        "accent_custom",
        &accent_custom_hash,
      )
      || manifest_field_changed(previous_manifest.as_ref(), "gtk_mode", &gtk_mode_hash));
  let wallpaper_changed = changed_sections.contains(&"appearance")
    && manifest_field_changed(previous_manifest.as_ref(), "wallpaper", &wallpaper_hash);

  let mut affected_consumers = Vec::new();
  if changed_sections.contains(&"appearance") {
    affected_consumers.extend(["theme", "accent", "wallpaper", "surfaces"]);
    if appearance_runtime_changed {
      affected_consumers.push("hyprland");
    }
  }
  if changed_sections.contains(&"effects") {
    affected_consumers.extend(["surfaces", "hyprland"]);
  }
  if changed_sections
    .iter()
    .any(|section| ["layout", "hyprland", "displays"].contains(section))
  {
    affected_consumers.push("hyprland");
  }
  if changed_sections.contains(&"fonts") {
    affected_consumers.extend(["terminal", "surfaces"]);
  }
  if changed_sections.contains(&"control_panel") {
    affected_consumers.push("control-panel");
  }
  if changed_sections.contains(&"power") {
    affected_consumers.push("hypridle");
  }
  if changed_sections.contains(&"audio") {
    affected_consumers.push("audio");
  }
  if changed_sections.iter().any(|section| {
    [
      "default_apps",
      "keyboard_shortcuts",
      "session",
      "removable_devices",
    ]
    .contains(section)
  }) {
    affected_consumers.push("session");
  }
  affected_consumers.sort_unstable();
  affected_consumers.dedup();

  let reload_targets = reload_targets(
    &changed_sections,
    appearance_visual_changed,
    wallpaper_changed,
  );

  let reload_plan = if changed_sections
    .iter()
    .any(|section| ["appearance", "effects", "layout"].contains(section))
  {
    "full"
  } else if changed_sections
    .iter()
    .any(|section| ["keyboard_shortcuts", "default_apps", "hyprland"].contains(section))
  {
    "hyprland"
  } else if changed_sections.contains(&"power") {
    "hypridle"
  } else {
    "none"
  };

  project_hypridle(&document, &paths)?;
  project_audio(&document, &changed_sections)?;
  project_wallpaper(&document, &paths)?;
  project_json_section(
    &document,
    "/default_apps",
    &paths.data.join("control-center/defaults.json"),
  )?;
  project_json_section(
    &document,
    "/removable_devices",
    &paths.data.join("removable-devices/config.json"),
  )?;
  project_language(&document, &paths)?;
  project_control_panel(&document, &paths)?;
  project_hypr_state(&document, &paths)?;
  project_scalar(
    &document,
    "/fonts/legacy",
    &paths.generated.join("fonts.conf"),
  )?;
  project_bool_marker(
    &document,
    "/power/keep_awake",
    &paths.data.join("power/keep-awake"),
  )?;
  project_input(&document, &paths)?;
  project_window_rules(&document, &paths)?;
  project_keyboard(&document, &paths)?;
  project_shortcuts(&document, &paths)?;
  project_displays(&document, &paths)?;
  project_calendar(&document, &paths)?;
  let adapters_ran = run_component_adapters(
    &document,
    &paths,
    &changed_sections,
    appearance_visual_changed,
  )?;
  // Layout runs after the adapters: the theme adapter resets the derived
  // `.spaces` and `.borders` files before it reapplies them, so the canonical
  // layout must be written last to be the value that survives the projection.
  project_layout(&document, &paths)?;
  if adapters_ran {
    // The reset also cleared the waybar margins written from `.spaces`.
    // Reapply them now that the layout is restored. `--apply-static` writes
    // only taskbar and geometry files; ARGVUS_NO_RUNTIME keeps it off the
    // compositor, as for every adapter.
    run_adapter(
      &system_config_root().join("hyprland/sh/spaces-switch.sh"),
      &["--apply-static"],
      &paths,
    )?;
  }
  remove_legacy_markers(&paths.root)?;

  let manifest = serde_json::json!({
      "schema_version": MANIFEST_SCHEMA_VERSION,
      "sections": section_hashes,
      "appearance_runtime": {"theme": theme_hash, "accent": accent_hash},
      "appearance_values": {
          "theme": theme_hash,
          "accent": accent_hash,
          "accent_custom": accent_custom_hash,
          "gtk_mode": gtk_mode_hash,
          "wallpaper": wallpaper_hash,
      },
      "changed_sections": changed_sections,
      "affected_consumers": affected_consumers,
      "reload_required": !affected_consumers.is_empty(),
      "reload_plan": reload_plan,
      "reload_targets": reload_targets,
  });
  atomic_write(
    &paths.projection_manifest,
    &format!(
      "{}\n",
      serde_json::to_string_pretty(&crate::sorted_value(&manifest))?
    ),
  )?;
  Ok(format!(
    "argvus-config: projected canonical configuration into {}",
    paths.root.display()
  ))
}

/// Projects only the shortcut table. Interactive settings changes use this
/// narrow path so an unrelated theme/effects adapter cannot prevent a
/// generated Hyprland keybinding from being applied.
pub fn project_shortcuts_only(store: &ConfigStore) -> ConfigResult<String> {
  let document = store.ensure()?;
  let paths = ArgvusPaths::from_store(store)?;
  fs::create_dir_all(&paths.generated)?;
  fs::create_dir_all(&paths.internal)?;
  project_shortcuts(&document, &paths)?;
  Ok(format!(
    "argvus-config: projected keyboard shortcuts into {}",
    paths.generated.display()
  ))
}

fn read_manifest(path: &Path) -> Option<Value> {
  let value: Value = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?;
  (value.get("schema_version").and_then(Value::as_u64) == Some(MANIFEST_SCHEMA_VERSION as u64))
    .then_some(value)
}

fn section_hash(document: &ConfigDocument, section: &str) -> ConfigResult<String> {
  value_hash(document.sections.get(section))
}

fn value_hash(value: Option<&Value>) -> ConfigResult<String> {
  let bytes = serde_json::to_vec(value.unwrap_or(&Value::Null))?;
  let mut hasher = DefaultHasher::new();
  bytes.hash(&mut hasher);
  Ok(format!("{:016x}", hasher.finish()))
}

fn manifest_field_changed(previous: Option<&Value>, field: &str, current_hash: &str) -> bool {
  previous
    .and_then(|manifest| manifest.get("appearance_values"))
    .and_then(Value::as_object)
    .and_then(|values| values.get(field))
    .and_then(Value::as_str)
    != Some(current_hash)
}

fn reload_targets(
  changed_sections: &[&str],
  appearance_visual_changed: bool,
  wallpaper_changed: bool,
) -> serde_json::Map<String, Value> {
  let mut targets = serde_json::Map::new();
  let mut enable = |name: &str| {
    targets.insert(name.to_owned(), Value::Bool(true));
  };

  if appearance_visual_changed {
    enable("hyprland");
    enable("taskbar");
    enable("widget_telemetry");
    enable("control_panel");
    enable("notifications");
    enable("polkit");
    enable("wallpaper");
  } else if wallpaper_changed {
    enable("wallpaper");
  }

  if changed_sections.contains(&"effects") {
    enable("hyprland");
    enable("taskbar");
    enable("widget_telemetry");
    enable("control_panel");
    enable("notifications");
  }
  if changed_sections.contains(&"layout") {
    enable("hyprland");
    enable("taskbar");
    enable("widget_telemetry");
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

fn atomic_write(path: &Path, contents: &str) -> ConfigResult<()> {
  let temporary = write_temporary(path, contents)?;
  commit_temporary(&temporary, path)
}

/// Writes a generated Lua file only after `luac -p` accepts the new contents.
///
/// Hyprland loads these files on every reload, so a syntax error here would
/// break the whole configuration. A rejected candidate is removed and the
/// previous file is left in place.
fn atomic_write_lua(path: &Path, contents: &str) -> ConfigResult<()> {
  let temporary = write_temporary(path, contents)?;
  if let Err(error) = validate_lua(&temporary, path) {
    let _ = fs::remove_file(&temporary);
    return Err(error);
  }
  commit_temporary(&temporary, path)
}

/// Writes `contents` to a sibling temporary file with owner-only permissions.
///
/// The temporary file is removed again when any write step fails, so a
/// failed projection never leaves a partial candidate behind.
fn write_temporary(path: &Path, contents: &str) -> ConfigResult<PathBuf> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
  let written = (|| -> std::io::Result<()> {
    let mut file = OpenOptions::new()
      .create_new(true)
      .write(true)
      .open(&temporary)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
  })();
  if let Err(error) = written {
    let _ = fs::remove_file(&temporary);
    return Err(error.into());
  }
  Ok(temporary)
}

fn commit_temporary(temporary: &Path, path: &Path) -> ConfigResult<()> {
  fs::rename(temporary, path)?;
  Ok(())
}

/// Runs `luac -p` on a candidate Lua file.
///
/// `luac` is optional at runtime: without it the projection still writes the
/// file and reports a warning, so a missing Lua package does not block
/// unrelated settings from being projected.
fn validate_lua(candidate: &Path, target: &Path) -> ConfigResult<()> {
  match Command::new("luac").arg("-p").arg(candidate).output() {
    Ok(output) if output.status.success() => Ok(()),
    Ok(output) => Err(ConfigError::Invalid(format!(
      "generated Lua for {} is invalid: {}",
      target.display(),
      String::from_utf8_lossy(&output.stderr).trim()
    ))),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
      eprintln!(
        "argvus-config: luac not found; {} was written without a Lua syntax check",
        target.display()
      );
      Ok(())
    }
    Err(error) => Err(error.into()),
  }
}

fn project_scalar(document: &ConfigDocument, pointer: &str, path: &Path) -> ConfigResult<()> {
  match document.get(pointer) {
    Some(Value::Null) | None => {
      if path.exists() {
        fs::remove_file(path)?;
      }
    }
    Some(value) => atomic_write(
      path,
      &format!("{}\n", value.as_str().unwrap_or(&value.to_string())),
    )?,
  }
  Ok(())
}

fn project_json_section(document: &ConfigDocument, pointer: &str, path: &Path) -> ConfigResult<()> {
  match document.get(pointer) {
    Some(Value::Null) | None => Ok(()),
    Some(value) => atomic_write(
      path,
      &format!(
        "{}\n",
        serde_json::to_string_pretty(&crate::sorted_value(value))?
      ),
    ),
  }
}

fn project_bool_marker(document: &ConfigDocument, pointer: &str, path: &Path) -> ConfigResult<()> {
  match document.get(pointer).and_then(Value::as_bool) {
    Some(true) => atomic_write(path, "enabled\n"),
    Some(false) => atomic_write(path, "disabled\n"),
    None => {
      if path.exists() {
        fs::remove_file(path)?;
      }
      Ok(())
    }
  }
}

fn project_language(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  if let Some(language) = document.get("/session/language").and_then(Value::as_str) {
    atomic_write(
      &paths.internal.join("language"),
      &format!("{}\n", language.replace('-', "_")),
    )
  } else if paths.internal.join("language").exists() {
    fs::remove_file(paths.internal.join("language"))?;
    Ok(())
  } else {
    Ok(())
  }
}

fn project_control_panel(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  if let Some(cards) = document
    .get("/control_panel/cards")
    .and_then(Value::as_array)
  {
    let disabled = cards
      .iter()
      .filter_map(|card| {
        (card.get("enabled").and_then(Value::as_bool) == Some(false))
          .then(|| card.get("id").cloned())
          .flatten()
      })
      .collect::<Vec<_>>();
    let order = cards
      .iter()
      .filter_map(|card| card.get("id").cloned())
      .collect::<Vec<_>>();
    atomic_write(
      &paths.data.join("control-panel/cards.json"),
      &format!(
        "{}\n",
        serde_json::json!({"disabled": disabled, "order": order})
      ),
    )?;
  }
  if let Some(enabled) = document
    .get("/control_panel/enabled")
    .and_then(Value::as_bool)
  {
    atomic_write(
      &paths.data.join("control-panel/state"),
      if enabled { "enabled\n" } else { "disabled\n" },
    )?;
  }
  Ok(())
}

/// Projects the canonical layout values into the compatibility inputs consumed
/// by the official Hyprland spacing and border adapters. These files remain
/// derived state; the modular configuration files are the only persistent source of truth.
/// Compute the effective rounding value.
/// When `rounded` is false (disabled), rounding must be 0 regardless of the configured value.
/// When `rounded` is true, clamp rounding to the valid range [2..10].
fn effective_rounding(rounded: bool, rounding: u64) -> u64 {
  if !rounded { 0 } else { rounding.clamp(2, 10) }
}

fn project_layout(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
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

fn project_input(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(input) = document.get("/hyprland/input").and_then(Value::as_object) else {
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

fn project_window_rules(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(rules) = document
    .get("/hyprland/window_rules")
    .and_then(Value::as_object)
  else {
    return Ok(());
  };
  let lua = render_window_rules(rules)?;
  atomic_write_lua(&paths.generated.join("hypr/window-rules.lua"), &lua)
}

fn render_window_rules(rules: &serde_json::Map<String, Value>) -> ConfigResult<String> {
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

fn is_class_pattern(value: &str) -> bool {
  !value.is_empty()
    && value.len() <= 128
    && value
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || "._*+?-[]()|^$".contains(c))
}

fn object_value<'a>(
  object: Option<&'a serde_json::Map<String, Value>>,
  key: &str,
) -> Option<&'a Value> {
  object.and_then(|value| value.get(key))
}

fn project_keyboard(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(keyboard) = document
    .get("/hyprland/keyboard")
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

fn project_shortcuts(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(shortcuts) = document
    .get("/keyboard_shortcuts")
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

fn shortcut_binding_ids() -> std::collections::BTreeMap<String, String> {
  let manifest_path = env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
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

fn project_displays(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(displays) = document.get("/displays").and_then(Value::as_object) else {
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

fn project_calendar(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(calendar) = document.get("/calendar").and_then(Value::as_object) else {
    return Ok(());
  };
  let config_home = paths
    .root
    .parent()
    .ok_or(ConfigError::MissingConfigDirectory)?;
  let text = format!(
    "[calendar]\nweek_start = {}\nshow_events = {}\ndefault_event_duration_minutes = {}\ndefault_reminder_minutes = {}\nsync_interval_minutes = {}\n",
    scalar(calendar, "week_start", 1),
    scalar(calendar, "show_events", true),
    scalar(calendar, "default_event_duration_minutes", 60),
    scalar(calendar, "default_reminder_minutes", 10),
    scalar(calendar, "sync_interval_minutes", 15)
  );
  atomic_write(
    &config_home.join("argvus-taskbar-calendar/config.toml"),
    &text,
  )
}

/// Projects the canonical wallpaper path into the user-facing hyprpaper
/// configuration consumed by the appearance/session helpers. The wallpaper
/// service remains responsible for selecting the runtime backend and monitor
/// layout; this file only carries the durable selected path.
fn project_wallpaper(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let Some(wallpaper) = document
    .get("/appearance/wallpaper")
    .and_then(Value::as_str)
    .filter(|value| !value.is_empty())
  else {
    return Ok(());
  };
  let path = paths.data.join("hypr/hyprpaper.conf");
  let home = env::var_os("HOME").map(PathBuf::from);
  let display_path = home
    .as_ref()
    .and_then(|home| {
      let home_prefix = format!("{}/", home.display());
      wallpaper.strip_prefix(home_prefix.as_str())
    })
    .map(|relative| format!("~/{relative}"))
    .unwrap_or_else(|| wallpaper.to_owned());
  let existing = fs::read_to_string(&path).unwrap_or_default();
  let mut output = String::new();
  let mut replaced = false;
  for line in existing.lines() {
    if line.trim_start().starts_with("path =") {
      let indent = &line[..line.len() - line.trim_start().len()];
      output.push_str(&format!("{indent}path = {display_path}\n"));
      replaced = true;
    } else {
      output.push_str(line);
      output.push('\n');
    }
  }
  if !replaced {
    if !output.trim().is_empty() {
      output.push('\n');
    }
    output.push_str(&format!(
            "wallpaper {{\n  monitor =\n  path = {display_path}\n  fit_mode = cover\n}}\n\nsplash = false\n"
        ));
  }
  atomic_write(&path, &output)
}

fn scalar(object: &serde_json::Map<String, Value>, key: &str, default: impl Into<Value>) -> Value {
  object.get(key).cloned().unwrap_or_else(|| default.into())
}
fn lua_value(value: &Value) -> ConfigResult<String> {
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
fn project_hypr_state(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
  let layout_is_float = document.get("/layout/variant").and_then(Value::as_str) == Some("float");
  let accent_custom = document
    .get("/appearance/accent_custom")
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

  let lua = format!(
    "-- Generated by argvus-config. Do not edit.\nreturn {{\n  layout_variant = {},\n  accent_custom = {accent_custom},\n  virtual_machine = {virtual_machine},\n  effects = {},\n}}\n",
    if layout_is_float {
      "\"float\""
    } else {
      "\"sticky\""
    },
    lua_table(&effects)
  );
  atomic_write_lua(&paths.generated.join("hypr/state.lua"), &lua)
}

/// Decides whether the session is a virtual machine.
///
/// `ARGVUS_VIRTUALIZATION`, published by argvus-start, is authoritative when
/// set. Otherwise the detector output decides. Both report `none` on physical
/// hardware, and that must never count as a virtual machine.
fn virtual_machine_state(published: Option<&str>, detected: Option<&str>) -> bool {
  match published.filter(|value| !value.is_empty()) {
    Some(published) => published != "none",
    None => detected.is_some_and(|value| !value.is_empty() && value != "none"),
  }
}

/// Runs `systemd-detect-virt --vm`. Returns `None` when the detector is missing.
fn detect_virtualization() -> Option<String> {
  let output = Command::new("systemd-detect-virt")
    .arg("--vm")
    .output()
    .ok()?;
  Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Renders a JSON value as a Lua table literal, keeping object keys as strings.
fn lua_table(value: &Value) -> String {
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
fn lua_string(text: &str) -> String {
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

fn project_hypridle(document: &ConfigDocument, paths: &ArgvusPaths) -> ConfigResult<()> {
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

/// Applies audio values from the canonical module to the default PipeWire
/// sink. Audio has no durable application config file, so wpctl is the
/// projection adapter and the modular JSON remains the persistent source.
fn project_audio(document: &ConfigDocument, changed_sections: &[&str]) -> ConfigResult<()> {
  if !changed_sections.contains(&"audio") {
    return Ok(());
  }
  let volume = document
    .get("/audio/output_volume")
    .and_then(Value::as_u64)
    .unwrap_or(50)
    .min(100);
  let muted = document
    .get("/audio/output_muted")
    .and_then(Value::as_bool)
    .unwrap_or(false);

  let volume_value = format!("{volume}%");
  let volume_status = Command::new("wpctl")
    .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &volume_value])
    .status();
  report_audio_projection("volume", volume_status);

  let mute_status = Command::new("wpctl")
    .args([
      "set-mute",
      "@DEFAULT_AUDIO_SINK@",
      if muted { "1" } else { "0" },
    ])
    .status();
  report_audio_projection("mute", mute_status);
  Ok(())
}

fn report_audio_projection(action: &str, result: std::io::Result<std::process::ExitStatus>) {
  match result {
    Ok(status) if status.success() => {}
    Ok(status) => eprintln!("argvus-config: wpctl {action} projection exited with {status}"),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
    Err(error) => eprintln!("argvus-config: wpctl {action} projection failed: {error}"),
  }
}

/// Packaged defaults root. `ARGVUS_SYSTEM_CONFIG` lets tests and DESTDIR
/// checks point the projection at another tree.
fn system_config_root() -> PathBuf {
  env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
}

/// Runs the component adapters and reports whether any of them ran.
fn run_component_adapters(
  document: &ConfigDocument,
  paths: &ArgvusPaths,
  changed_sections: &[&str],
  appearance_visual_changed: bool,
) -> ConfigResult<bool> {
  let theme = document
    .get("/appearance/theme")
    .and_then(Value::as_str)
    .unwrap_or("argvus-dark");
  let theme_output = paths.generated.join("theme-effective.conf");
  let theme_qml = paths.generated.join(format!(
    "quickshell/argvus-control-panel/themes/{theme}/Theme.qml"
  ));
  let theme_transaction = env::var("ARGVUS_THEME_SWITCH").ok().as_deref() == Some("1");
  let theme_missing = !theme_output.is_file() || !theme_qml.is_file();
  let theme_applied = !theme_transaction
    && (appearance_visual_changed || (changed_sections.is_empty() && theme_missing));
  if theme_applied {
    run_theme_switch(theme, paths)?;
  }
  let effects_output = paths.generated.join(format!("effects/{theme}.conf"));
  let effects_missing = !effects_output.is_file();
  let effects_applied = !theme_transaction
    && (changed_sections.contains(&"effects") || (changed_sections.is_empty() && effects_missing));
  if effects_applied {
    run_adapter_command(
      Command::new("argvus").arg("effects-toggle").arg("apply"),
      Path::new("argvus effects-toggle"),
      paths,
    )?;
  }
  Ok(theme_applied || effects_applied)
}

fn run_adapter(path: &Path, arguments: &[&str], paths: &ArgvusPaths) -> ConfigResult<()> {
  if !path.is_file() {
    return Ok(());
  }
  run_adapter_command(Command::new("sh").arg(path).args(arguments), path, paths)
}

/// Runs the theme switch through the `argvus theme-switch` route, which is
/// argvus-config's `theme-apply`.
fn run_theme_switch(theme: &str, paths: &ArgvusPaths) -> ConfigResult<()> {
  run_adapter_command(
    Command::new("argvus").arg("theme-switch").arg(theme),
    Path::new("argvus theme-switch"),
    paths,
  )
}

fn run_adapter_command(
  command: &mut Command,
  path: &Path,
  paths: &ArgvusPaths,
) -> ConfigResult<()> {
  let status = command
    .env("ARGVUS_NO_RUNTIME", "1")
    .env("ARGVUS_PROJECTING", "1")
    .env(
      "ARGVUS_CONFIG_HOME",
      paths.root.parent().unwrap_or(Path::new("")),
    )
    .status()?;
  if status.success() {
    Ok(())
  } else {
    Err(ConfigError::Invalid(format!(
      "projection adapter {} exited with {status}",
      path.display()
    )))
  }
}

fn remove_legacy_markers(root: &Path) -> ConfigResult<()> {
  // These files are still consumed by the Lua compositor and shell
  // adapters. They are derived compatibility projections, not competing
  // sources of truth, so do not delete them until all consumers read data/.
  let path = root.join("data/.accent-custom");
  if path.exists() {
    fs::remove_file(path)?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hypridle_projection_uses_the_runtime_consumer_path() {
    let root = env::temp_dir().join(format!("argvus-project-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = ArgvusPaths {
      data: root.join("data"),
      generated: root.join("data/generated"),
      internal: root.join("data/internal"),
      backups: root.join("data/backups"),
      projection_manifest: root.join("state/config-projection.json"),
      root: root.clone(),
    };
    let mut document = ConfigDocument::default();
    document
      .set("/power/lock_minutes", Value::from(15))
      .expect("valid power value");
    document
      .set("/power/screen_off_minutes", Value::from(10))
      .expect("valid power value");

    project_hypridle(&document, &paths).expect("hypridle projection succeeds");
    let runtime = fs::read_to_string(root.join("data/hypr/hypridle.conf"))
      .expect("runtime hypridle file exists");
    assert!(runtime.contains("timeout = 900"));
    // Idle actions are runtime commands; they must not go back through hyprctl.
    assert!(!runtime.contains("hyprctl"));
    assert!(runtime.contains("argvus config hyprland dpms off"));
    assert!(runtime.contains("timeout = 600"));
    assert!(!root.join("hypr/hypridle.conf").exists());
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn reload_targets_keep_power_changes_scoped_to_hypridle() {
    let targets = reload_targets(&["power"], false, false);
    assert_eq!(targets.get("hypridle"), Some(&Value::Bool(true)));
    assert_eq!(targets.len(), 1);
  }

  #[test]
  fn reload_targets_keep_wallpaper_changes_scoped_to_wallpaper() {
    let targets = reload_targets(&["appearance"], false, true);
    assert_eq!(targets.get("wallpaper"), Some(&Value::Bool(true)));
    assert_eq!(targets.len(), 1);
  }

  #[test]
  fn reload_targets_expand_theme_changes_to_visual_consumers() {
    let targets = reload_targets(&["appearance"], true, true);
    for target in [
      "hyprland",
      "taskbar",
      "widget_telemetry",
      "control_panel",
      "notifications",
      "wallpaper",
    ] {
      assert_eq!(targets.get(target), Some(&Value::Bool(true)), "{target}");
    }
  }

  #[test]
  fn wallpaper_projection_updates_the_native_hyprpaper_file() {
    let root = env::temp_dir().join(format!("argvus-wallpaper-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = ArgvusPaths {
      data: root.join("data"),
      generated: root.join("data/generated"),
      internal: root.join("data/internal"),
      backups: root.join("data/backups"),
      projection_manifest: root.join("state/config-projection.json"),
      root: root.clone(),
    };
    let mut document = ConfigDocument::default();
    document
      .set(
        "/appearance/wallpaper",
        Value::from("/usr/share/backgrounds/argvus/argvus-dark.jxl"),
      )
      .expect("valid wallpaper value");

    project_wallpaper(&document, &paths).expect("wallpaper projection succeeds");
    let native = fs::read_to_string(root.join("data/hypr/hyprpaper.conf"))
      .expect("native wallpaper file exists");
    assert!(native.contains("path = /usr/share/backgrounds/argvus/argvus-dark.jxl"));
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn effective_rounding_is_zero_when_rounded_is_false() {
    assert_eq!(effective_rounding(false, 0), 0);
    assert_eq!(effective_rounding(false, 4), 0);
    assert_eq!(effective_rounding(false, 8), 0);
    assert_eq!(effective_rounding(false, 10), 0);
  }

  #[test]
  fn effective_rounding_clamps_to_valid_range_when_rounded_is_true() {
    // Below minimum (2)
    assert_eq!(effective_rounding(true, 0), 2);
    assert_eq!(effective_rounding(true, 1), 2);
    // Within range
    assert_eq!(effective_rounding(true, 2), 2);
    assert_eq!(effective_rounding(true, 5), 5);
    assert_eq!(effective_rounding(true, 10), 10);
    // Above maximum (10)
    assert_eq!(effective_rounding(true, 11), 10);
    assert_eq!(effective_rounding(true, 100), 10);
  }

  #[test]
  fn layout_projection_ignores_rounding_when_rounded_is_false() {
    let root = env::temp_dir().join(format!("argvus-borders-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = ArgvusPaths {
      data: root.join("data"),
      generated: root.join("data/generated"),
      internal: root.join("data/internal"),
      backups: root.join("data/backups"),
      projection_manifest: root.join("state/config-projection.json"),
      root: root.clone(),
    };
    let mut document = ConfigDocument::default();
    document
      .set("/layout/window/rounded", Value::Bool(false))
      .expect("valid rounded value");
    document
      .set("/layout/window/rounding", Value::from(8))
      .expect("valid rounding value");
    document
      .set("/layout/window/border_size", Value::from(1))
      .expect("valid border_size value");

    project_layout(&document, &paths).expect("layout projection succeeds");
    let borders = fs::read_to_string(root.join("data/.borders")).expect(".borders file exists");
    assert!(borders.contains("rounded=0"), "rounded should be 0");
    assert!(
      borders.contains("rounding=0"),
      "rounding should be forced to 0 when rounded=false"
    );
    assert!(
      borders.contains("thickness=1"),
      "thickness should be preserved"
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn layout_projection_respects_rounding_when_rounded_is_true() {
    let root = env::temp_dir().join(format!("argvus-borders-true-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = ArgvusPaths {
      data: root.join("data"),
      generated: root.join("data/generated"),
      internal: root.join("data/internal"),
      backups: root.join("data/backups"),
      projection_manifest: root.join("state/config-projection.json"),
      root: root.clone(),
    };
    let mut document = ConfigDocument::default();
    document
      .set("/layout/window/rounded", Value::Bool(true))
      .expect("valid rounded value");
    document
      .set("/layout/window/rounding", Value::from(5))
      .expect("valid rounding value");
    document
      .set("/layout/window/border_size", Value::from(1))
      .expect("valid border_size value");

    project_layout(&document, &paths).expect("layout projection succeeds");
    let borders = fs::read_to_string(root.join("data/.borders")).expect(".borders file exists");
    assert!(borders.contains("rounded=1"), "rounded should be 1");
    assert!(
      borders.contains("rounding=5"),
      "rounding should be preserved when rounded=true"
    );
    assert!(
      borders.contains("thickness=1"),
      "thickness should be preserved"
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn renders_window_rules_and_rejects_unsafe_patterns() {
    let rules = serde_json::json!({
        "browser": { "workspace": 2, "classes": ["firefox.*", "zen-browser.*"] },
        "ide": { "workspace": 1, "classes": ["[Cc]ode.*"] }
    });
    let lua = render_window_rules(rules.as_object().unwrap()).unwrap();
    assert!(lua.contains("{ workspace = 2, classes = { \"firefox.*\", \"zen-browser.*\", } },"));
    assert!(lua.contains("{ workspace = 1, classes = { \"[Cc]ode.*\", } },"));

    for invalid in [
      serde_json::json!({ "x": { "workspace": 11, "classes": ["a"] } }),
      serde_json::json!({ "x": { "workspace": 1, "classes": ["a\"b"] } }),
      serde_json::json!({ "x": { "workspace": 1, "classes": "code" } }),
    ] {
      assert!(render_window_rules(invalid.as_object().unwrap()).is_err());
    }
  }

  #[test]
  fn virtual_machine_state_treats_none_as_physical_hardware() {
    assert!(!virtual_machine_state(Some("none"), Some("kvm")));
    assert!(virtual_machine_state(Some("kvm"), None));
    assert!(!virtual_machine_state(None, Some("none")));
    assert!(!virtual_machine_state(None, Some("")));
    assert!(!virtual_machine_state(None, None));
    // An empty published value is not an answer, so detection decides.
    assert!(virtual_machine_state(Some(""), Some("kvm")));
  }

  #[test]
  fn lua_table_escapes_strings_and_keeps_nested_types() {
    let value = serde_json::json!({
        "quote\"and\\slash": "line\nbreak",
        "blur_size": 6,
        "animations": true,
    });
    let lua = lua_table(&value);
    assert!(lua.contains("[\"quote\\\"and\\\\slash\"] = \"line\\nbreak\""));
    assert!(lua.contains("[\"blur_size\"] = 6"));
    assert!(lua.contains("[\"animations\"] = true"));
  }

  #[test]
  fn hypr_state_projection_writes_values_the_lua_config_reads() {
    if !luac_available() {
      return;
    }
    let root = env::temp_dir().join(format!("argvus-state-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = ArgvusPaths {
      data: root.join("data"),
      generated: root.join("data/generated"),
      internal: root.join("data/internal"),
      backups: root.join("data/backups"),
      projection_manifest: root.join("state/config-projection.json"),
      root: root.clone(),
    };
    let mut document = ConfigDocument::default();
    document
      .set("/layout/variant", Value::from("float"))
      .expect("valid layout variant");
    document
      .set("/appearance/accent_custom", Value::Bool(true))
      .expect("valid accent flag");
    document
      .set("/effects/blur_size", Value::from(6))
      .expect("valid blur size");

    project_hypr_state(&document, &paths).expect("state projection succeeds");
    let state =
      fs::read_to_string(root.join("data/generated/hypr/state.lua")).expect("state file exists");
    assert!(state.contains("layout_variant = \"float\""));
    assert!(state.contains("accent_custom = true"));
    assert!(state.contains("[\"blur_size\"] = 6"));
    let _ = fs::remove_dir_all(root);
  }

  /// Lua checks need `luac`; the tests skip (with a note) when it is absent,
  /// because the projection itself only warns in that case.
  fn luac_available() -> bool {
    let available = Command::new("luac").arg("-v").output().is_ok();
    if !available {
      eprintln!("skipping Lua projection test: luac is not installed");
    }
    available
  }

  #[test]
  fn lua_projection_keeps_previous_file_when_candidate_is_invalid() {
    if !luac_available() {
      return;
    }
    let root = env::temp_dir().join(format!("argvus-lua-invalid-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let target = root.join("hypr/monitors.lua");
    atomic_write_lua(&target, "return { ok = true }\n").expect("valid Lua is written");
    let previous_contents = fs::read_to_string(&target).expect("previous file exists");

    let error =
      atomic_write_lua(&target, "return { broken = \n").expect_err("invalid Lua must be rejected");
    assert!(matches!(error, ConfigError::Invalid(_)));
    assert_eq!(
      fs::read_to_string(&target).expect("previous file survives"),
      previous_contents
    );
    // The rejected candidate must not be left next to the live file.
    let remaining_entries = fs::read_dir(root.join("hypr"))
      .expect("generated directory exists")
      .count();
    assert_eq!(remaining_entries, 1);
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn lua_projection_replaces_file_when_candidate_is_valid() {
    if !luac_available() {
      return;
    }
    let root = env::temp_dir().join(format!("argvus-lua-valid-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let target = root.join("hypr/keybindings.lua");
    atomic_write_lua(&target, "return { ok = true }\n").expect("first Lua is written");
    atomic_write_lua(&target, "return { ok = false }\n").expect("second Lua is written");
    assert_eq!(
      fs::read_to_string(&target).expect("generated file exists"),
      "return { ok = false }\n"
    );
    let _ = fs::remove_dir_all(root);
  }
}
