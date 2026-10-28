//! Wallpaper collections, the active wallpaper and switching it.

use super::commands::script;
use super::*;

pub(super) fn classify_wallpaper(relative: &Path) -> Option<(WallpaperCollection, WallpaperMode)> {
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

pub(super) fn hyprpaper_config_path() -> PathBuf {
  argvus_data_home().join("hypr").join("hyprpaper.conf")
}

pub(super) fn persist_wallpaper_canonical(wallpaper: &Path) -> Result<(), String> {
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
