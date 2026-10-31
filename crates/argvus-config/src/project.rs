//! Typed canonical-configuration projections.
//!
//! This module owns the structured part of projection.  Component-specific
//! theme/effect adapters remain external processes until their public payload
//! contracts can be moved without duplicating asset ownership.

use crate::json_pointers;
use crate::session_paths;
use crate::session_timing::timed;
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

mod component_adapters;
mod file_writes;
mod hyprland_projection;
mod manifest;

use component_adapters::{
  remove_legacy_markers, run_adapter, run_component_adapters, system_config_root,
};
use file_writes::atomic_write;
use hyprland_projection::{
  project_displays, project_hypr_state, project_hypridle, project_input, project_keyboard,
  project_layout, project_shortcuts, project_window_rules, scalar,
};
pub use manifest::last_projection_affects_compositor;
use manifest::{manifest_field_changed, read_manifest, reload_targets, section_hash, value_hash};

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

/// The ARGVUS state directory: `$XDG_STATE_HOME/argvus`, or `~/.local/state/argvus`.
pub fn argvus_state_root() -> ConfigResult<PathBuf> {
  let state_home = env::var_os("XDG_STATE_HOME")
    .map(PathBuf::from)
    .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
    .ok_or(ConfigError::MissingConfigDirectory)?;
  Ok(state_home.join("argvus"))
}

/// File name of the projection manifest under the state directory.
const PROJECTION_MANIFEST_FILE: &str = "config-projection.json";

/// Consumer name the manifest uses for the compositor.
const HYPRLAND_CONSUMER: &str = "hyprland";

impl ArgvusPaths {
  pub fn from_store(store: &ConfigStore) -> ConfigResult<Self> {
    let root = store
      .path
      .parent()
      .ok_or(ConfigError::MissingConfigDirectory)?
      .to_path_buf();
    let state_root = argvus_state_root()?;
    Ok(Self {
      data: root.join("data"),
      generated: root.join("data/generated"),
      internal: root.join("data/internal"),
      backups: root.join("data/backups"),
      projection_manifest: state_root.join(PROJECTION_MANIFEST_FILE),
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
  let (section_hashes, changed_sections) =
    detect_changed_sections(&document, previous_manifest.as_ref())?;

  let theme_hash = value_hash(document.get(json_pointers::APPEARANCE_THEME))?;
  let accent_hash = value_hash(document.get(json_pointers::APPEARANCE_ACCENT))?;
  let accent_custom_hash = value_hash(document.get(json_pointers::APPEARANCE_ACCENT_CUSTOM))?;
  let gtk_mode_hash = value_hash(document.get(json_pointers::APPEARANCE_GTK_MODE))?;
  let wallpaper_hash = value_hash(document.get(json_pointers::APPEARANCE_WALLPAPER))?;
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
  // The taskbar margins and position are layer-shell settings of the bars: only
  // they recreate the bars. Window gaps and borders reload in place.
  let taskbar_layout_hash = value_hash(document.get("/layout/taskbar"))?;
  let taskbar_layout_changed = changed_sections.contains(&"layout")
    && previous_manifest
      .as_ref()
      .and_then(|manifest| manifest.pointer("/layout_values/taskbar"))
      .and_then(Value::as_str)
      != Some(taskbar_layout_hash.as_str());

  let affected_consumers = affected_consumers(&changed_sections, appearance_runtime_changed);

  let reload_targets = reload_targets(
    &changed_sections,
    appearance_visual_changed,
    wallpaper_changed,
    taskbar_layout_changed,
  );

  let reload_plan = reload_plan(&changed_sections);

  write_projections(
    &document,
    &paths,
    &changed_sections,
    appearance_visual_changed,
  )?;

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
      "layout_values": {"taskbar": taskbar_layout_hash},
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

/// Hashes every projected section and lists the ones whose hash differs from
/// the previous manifest (all of them when there is none).
fn detect_changed_sections(
  document: &ConfigDocument,
  previous_manifest: Option<&Value>,
) -> ConfigResult<(serde_json::Map<String, Value>, Vec<&'static str>)> {
  let mut section_hashes = serde_json::Map::new();
  let mut changed_sections = Vec::new();
  for section in SECTIONS {
    let hash = section_hash(document, section)?;
    if previous_manifest
      .and_then(|manifest| manifest.get("sections"))
      .and_then(|sections| sections.get(section))
      .and_then(Value::as_str)
      != Some(hash.as_str())
    {
      changed_sections.push(section);
    }
    section_hashes.insert(section.to_owned(), Value::String(hash));
  }
  Ok((section_hashes, changed_sections))
}

/// Consumers that must be refreshed for the changed sections, sorted and
/// without duplicates. Hyprland is affected by the theme only when the
/// runtime part of the appearance (theme or accent) changed.
fn affected_consumers(
  changed_sections: &[&'static str],
  appearance_runtime_changed: bool,
) -> Vec<&'static str> {
  let mut affected_consumers = Vec::new();
  if changed_sections.contains(&"appearance") {
    affected_consumers.extend(["theme", "accent", "wallpaper", "surfaces"]);
    if appearance_runtime_changed {
      affected_consumers.push(HYPRLAND_CONSUMER);
    }
  }
  if changed_sections.contains(&"effects") {
    affected_consumers.extend(["surfaces", HYPRLAND_CONSUMER]);
  }
  if changed_sections
    .iter()
    .any(|section| ["layout", "hyprland", "displays"].contains(section))
  {
    affected_consumers.push(HYPRLAND_CONSUMER);
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
  affected_consumers
}

/// The session reload the changed sections need, from the widest to none.
fn reload_plan(changed_sections: &[&'static str]) -> &'static str {
  if changed_sections
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
  }
}

/// Writes every projected file. The order matters only for the layout, which
/// must come after the component adapters (see below).
fn write_projections(
  document: &ConfigDocument,
  paths: &ArgvusPaths,
  changed_sections: &[&'static str],
  appearance_visual_changed: bool,
) -> ConfigResult<()> {
  project_hypridle(document, paths)?;
  project_audio(document, changed_sections)?;
  project_wallpaper(document, paths)?;
  project_json_section(
    document,
    json_pointers::DEFAULT_APPS,
    &paths.data.join("control-center/defaults.json"),
  )?;
  project_json_section(
    document,
    json_pointers::REMOVABLE_DEVICES,
    &paths.data.join("removable-devices/config.json"),
  )?;
  project_control_panel(document, paths)?;
  project_hypr_state(document, paths)?;
  project_scalar(
    document,
    json_pointers::FONTS_LEGACY,
    &paths.generated.join("fonts.conf"),
  )?;
  project_bool_marker(
    document,
    json_pointers::POWER_KEEP_AWAKE,
    &paths.data.join("power/.keep-awake"),
  )?;
  project_input(document, paths)?;
  project_window_rules(document, paths)?;
  project_keyboard(document, paths)?;
  project_shortcuts(document, paths)?;
  project_displays(document, paths)?;
  project_calendar(document, paths)?;
  let adapters_ran =
    run_component_adapters(document, paths, changed_sections, appearance_visual_changed)?;
  // Layout runs after the adapters: the theme adapter resets the derived
  // `.spaces` and `.borders` files before it reapplies them, so the canonical
  // layout must be written last to be the value that survives the projection.
  project_layout(document, paths)?;
  if adapters_ran {
    // The reset also cleared the waybar margins written from `.spaces`.
    // Reapply them now that the layout is restored. `--apply-static` writes
    // only taskbar and geometry files; ARGVUS_NO_RUNTIME keeps it off the
    // compositor, as for every adapter.
    run_adapter(
      &system_config_root().join("hyprland/sh/spaces-switch.sh"),
      &["--apply-static"],
      paths,
    )?;
  }
  remove_legacy_markers(&paths.root)?;
  Ok(())
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
    .get(json_pointers::APPEARANCE_WALLPAPER)
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

#[cfg(test)]
mod tests;
