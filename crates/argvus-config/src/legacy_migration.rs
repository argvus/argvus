//! One-time migration of the pre-JSON configuration: legacy key/value files, the old layout tree and theme names with a layout suffix.

use super::legacy_import::{
  import_defaults, import_displays, import_fonts, import_hyprland, import_keyboard_shortcuts,
  import_language, import_power, import_removable_devices, legacy_keybindings_to_shortcuts,
};
use super::*;

pub(super) fn import_legacy_values(
  document: &mut ConfigDocument,
  root: &Path,
) -> ConfigResult<bool> {
  let mut changed = false;
  changed |= import_line(
    document,
    json_pointers::APPEARANCE_THEME,
    resolved(root, ".active-theme"),
  )?;
  // Legacy installs encoded Sticky/Float as a `-float` suffix on the theme
  // name itself (e.g. `.active-theme` containing `argvus-dark-float`).
  // Strip it into the now-independent `/layout/variant` so the user's
  // existing mode survives the migration instead of being silently lost.
  changed |= migrate_theme_suffix_to_variant(document)?;
  let imported_accent = import_line(
    document,
    json_pointers::APPEARANCE_ACCENT,
    resolved(root, ".accent-color"),
  )?;
  changed |= imported_accent;
  if imported_accent && resolved(root, ".accent-custom").is_some_and(|path| path.is_file()) {
    document.set(json_pointers::APPEARANCE_ACCENT_CUSTOM, Value::Bool(true))?;
    changed = true;
  }
  changed |= import_line(
    document,
    json_pointers::APPEARANCE_GTK_MODE,
    resolved(root, ".gtk-mode"),
  )?;
  changed |= import_line(
    document,
    json_pointers::APPEARANCE_WALLPAPER,
    resolved(root, ".wallpaper-custom"),
  )?;
  changed |= import_layout_state(document, root)?;
  changed |= import_fonts(document, resolved(root, "fonts.conf"))?;
  changed |= import_hyprland(document, root)?;
  changed |= import_displays(document, root)?;
  changed |= import_defaults(document, resolved(root, "defaults.json"))?;
  changed |= import_keyboard_shortcuts(document, root)?;
  changed |= import_language(document, resolved(root, "language"))?;
  changed |= import_power(document, root)?;
  changed |= import_removable_devices(document, root)?;
  Ok(changed)
}

pub(super) fn migrate_theme_suffix_to_variant(document: &mut ConfigDocument) -> ConfigResult<bool> {
  let Some(theme) = document
    .get(json_pointers::APPEARANCE_THEME)
    .and_then(Value::as_str)
  else {
    return Ok(false);
  };
  let Some(family) = theme.strip_suffix("-float") else {
    return Ok(false);
  };
  document.set(
    json_pointers::APPEARANCE_THEME,
    Value::String(family.to_owned()),
  )?;
  if document.get(json_pointers::LAYOUT_VARIANT).is_none() {
    document.set(json_pointers::LAYOUT_VARIANT, Value::String("float".into()))?;
  }
  Ok(true)
}

pub(super) fn import_layout_state(
  document: &mut ConfigDocument,
  root: &Path,
) -> ConfigResult<bool> {
  let mut changed = false;
  let spaces = resolved(root, ".spaces")
    .map(|path| parse_legacy_pairs(&path))
    .unwrap_or_default();
  let borders = resolved(root, ".borders")
    .map(|path| parse_legacy_pairs(&path))
    .unwrap_or_default();
  for (key, value) in spaces {
    let pointer = match key.as_str() {
      "gaps_in" => "/layout/window/gaps_in",
      "gaps_out" => "/layout/window/gaps_out",
      "gaps_out_top" => "/layout/window/gaps_out_top",
      "gaps_out_left" => "/layout/window/gaps_out_left",
      "gaps_out_right" => "/layout/window/gaps_out_right",
      "gaps_out_bottom" => "/layout/window/gaps_out_bottom",
      "waybar_pos" => "/layout/taskbar/position",
      "waybar_top" => "/layout/taskbar/margin_top",
      "waybar_left" => "/layout/taskbar/margin_left",
      "waybar_right" => "/layout/taskbar/margin_right",
      "waybar_bottom" => "/layout/taskbar/margin_bottom",
      _ => continue,
    };
    let parsed = if pointer.ends_with("position") {
      Value::String(value)
    } else {
      Value::Number(
        value
          .parse::<u64>()
          .map_err(|_| ConfigError::Invalid(format!("invalid legacy layout value for {key}")))?
          .into(),
      )
    };
    if document.get(pointer).is_none() {
      document.set(pointer, parsed)?;
      changed = true;
    }
  }
  for (key, value) in borders {
    let pointer = match key.as_str() {
      "rounded" => "/layout/window/rounded",
      "rounding" => "/layout/window/rounding",
      "thickness" => "/layout/window/border_size",
      _ => continue,
    };
    let parsed = if pointer.ends_with("rounded") {
      Value::Bool(value == "1" || value.eq_ignore_ascii_case("true"))
    } else {
      Value::Number(
        value
          .parse::<u64>()
          .map_err(|_| ConfigError::Invalid(format!("invalid legacy border value for {key}")))?
          .into(),
      )
    };
    if document.get(pointer).is_none() {
      document.set(pointer, parsed)?;
      changed = true;
    }
  }
  Ok(changed)
}

pub(super) fn parse_legacy_pairs(path: &Path) -> Vec<(String, String)> {
  fs::read_to_string(path)
    .unwrap_or_default()
    .lines()
    .filter_map(|line| {
      let (key, value) = line.split_once('=')?;
      Some((key.trim().to_owned(), value.trim().to_owned()))
    })
    .collect()
}

pub(super) fn import_line(
  document: &mut ConfigDocument,
  pointer: &str,
  path: Option<PathBuf>,
) -> ConfigResult<bool> {
  let Some(path) = path else {
    return Ok(false);
  };
  if document.get(pointer).is_some() || !path.is_file() {
    return Ok(false);
  }
  let value = fs::read_to_string(&path)?
    .lines()
    .next()
    .unwrap_or_default()
    .trim()
    .to_owned();
  if value.is_empty() {
    return Ok(false);
  }
  document.set(pointer, Value::String(value))?;
  Ok(true)
}

pub(super) fn migrate_legacy_sections(document: &mut ConfigDocument) -> ConfigResult<()> {
  if document.get(json_pointers::DEFAULT_APPS).is_none()
    && let Some(value) = document.sections.remove("defaults")
  {
    document.set(json_pointers::DEFAULT_APPS, value)?;
  } else {
    document.sections.remove("defaults");
  }
  if document.get(json_pointers::KEYBOARD_SHORTCUTS).is_none()
    && let Some(value) = document
      .sections
      .get_mut("hyprland")
      .and_then(Value::as_object_mut)
      .and_then(|object| object.remove("keybindings"))
  {
    document.set(
      json_pointers::KEYBOARD_SHORTCUTS,
      legacy_keybindings_to_shortcuts(&value),
    )?;
  }
  if let Some(hyprland) = document.sections.get_mut("hyprland") {
    hyprland
      .as_object_mut()
      .expect("hyprland is object")
      .remove("keybindings");
  }
  Ok(())
}

/// Destination of a pre-`data/` root entry, relative to the configuration root.
///
/// Every entry other than `config/` and `data/` itself belongs under `data/`.
/// A few names already had a semantic home from an earlier migration and keep
/// it; everything else follows the uniform `data/<name>` rule so a new
/// component directory never has to be registered here.
pub(super) fn data_destination(name: &str) -> PathBuf {
  match name {
    ".config.lock" => PathBuf::from("data/internal/config.lock"),
    "config.json.bak" => PathBuf::from("data/backups/config.json.bak"),
    "defaults.json" => PathBuf::from("data/control-center/defaults.json"),
    "keybindings.toml" => PathBuf::from("data/hypr/keybindings.toml"),
    "input.toml" => PathBuf::from("data/hypr/input.toml"),
    ".idle-timeout" | ".keep-awake" => PathBuf::from(format!("data/power/{name}")),
    "fonts.conf" => PathBuf::from("data/generated/fonts.conf"),
    "generated" => PathBuf::from("data/generated"),
    "control-center" => PathBuf::from("data/control-center"),
    // The legacy root `terminal/` directory already holds one subdirectory
    // per terminal, so it maps onto `data/terminal/` itself. A standalone
    // root `kitty/` directory belongs beside the others.
    "terminal" => PathBuf::from("data/terminal"),
    "kitty" => PathBuf::from("data/terminal/kitty"),
    _ => PathBuf::from(format!("data/{name}")),
  }
}

/// Resolve a migrated entry for reading, accepting the pre-`data/` root layout.
///
/// Migration runs after the legacy import, so a profile that has not been
/// migrated yet still has its markers at the root. Prefer the managed location
/// and fall back to the root so a single import pass works on both layouts.
pub(super) fn resolved(root: &Path, name: &str) -> Option<PathBuf> {
  let managed = root.join(data_destination(name));
  if managed.exists() {
    return Some(managed);
  }
  let legacy = root.join(name);
  if legacy.exists() {
    return Some(legacy);
  }
  None
}

pub(super) fn migrate_legacy_layout(root: &Path) -> ConfigResult<bool> {
  let data = root.join("data");
  fs::create_dir_all(data.join("internal"))?;
  fs::create_dir_all(data.join("backups"))?;
  let mut moved = false;
  let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, io::Error>>()?;
  // `generated` and `terminal` are parents of other migrated components, so
  // they must be moved before their children; otherwise the child would find
  // its destination already present and be diverted to `data/legacy`.
  entries.sort_by_key(|entry| match entry.file_name().to_string_lossy().as_ref() {
    "generated" => 0,
    "terminal" => 1,
    "kitty" => 2,
    _ => 3,
  });
  for entry in entries {
    let source = entry.path();
    let name = entry.file_name();
    if name == "config.json" || name == MODULE_DIRECTORY || name == "data" {
      continue;
    }
    let name = name.to_string_lossy();
    let relative = data_destination(name.as_ref());
    let destination = root.join(relative);
    if matches!(
      name.as_ref(),
      "generated" | "control-center" | "terminal" | "kitty"
    ) && source.is_dir()
    {
      merge_legacy_tree(&source, &destination, &data.join("legacy"))?;
    } else if destination.exists() {
      let fallback = root.join("data/legacy").join(name.as_ref());
      if fallback.exists() {
        continue;
      }
      fs::create_dir_all(fallback.parent().expect("legacy parent"))?;
      fs::rename(&source, fallback)?;
    } else {
      fs::create_dir_all(destination.parent().expect("migration parent"))?;
      fs::rename(&source, destination)?;
    }
    moved = true;
  }
  Ok(moved)
}

/// Merge an old component directory into its new data-owned location.
///
/// New files win on collision because they are the current layout. Conflicting
/// legacy files are retained under data/legacy instead of being discarded.
pub(super) fn merge_legacy_tree(
  source: &Path,
  destination: &Path,
  legacy_root: &Path,
) -> ConfigResult<bool> {
  fs::create_dir_all(destination)?;
  let component_name = source
    .file_name()
    .ok_or_else(|| ConfigError::Invalid("legacy component has no name".into()))?;
  let component_legacy_root = legacy_root.join(component_name);
  let mut changed = false;

  for entry in fs::read_dir(source)?.collect::<Result<Vec<_>, io::Error>>()? {
    let source_entry = entry.path();
    let entry_name = entry.file_name();
    let destination_entry = destination.join(&entry_name);

    if destination_entry.is_dir() && source_entry.is_dir() {
      changed |= merge_legacy_tree(&source_entry, &destination_entry, &component_legacy_root)?;
      continue;
    }

    if destination_entry.exists() {
      fs::create_dir_all(&component_legacy_root)?;
      let mut legacy_entry = component_legacy_root.join(&entry_name);
      let mut suffix = 1u32;
      while legacy_entry.exists() {
        legacy_entry =
          component_legacy_root.join(format!("{}.legacy-{suffix}", entry_name.to_string_lossy()));
        suffix += 1;
      }
      fs::rename(&source_entry, legacy_entry)?;
    } else {
      fs::rename(&source_entry, destination_entry)?;
    }
    changed = true;
  }

  if fs::read_dir(source)?.next().is_none() {
    fs::remove_dir(source)?;
  }
  Ok(changed)
}
