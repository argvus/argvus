//! Theme projections for the surfaces around the desktop: the removable-devices
//! panel, the taskbar calendar, and the pre-login greeter.
//!
//! The desktop files are written by the theme switch itself. These surfaces
//! read the same theme id and copy or publish one small file each. Built-in
//! themes map to a stylesheet by name; drop-in themes name theirs in the
//! manifest (`removable_devices_css` and `calendar_css` under `[appearance]`).

use crate::session_paths::SessionPaths;
use crate::theme_resolve;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The built-in theme families both surfaces know by name.
const BUILTIN_FAMILIES: [&str; 22] = [
  "one-dark",
  "dracula",
  "argvus-dark",
  "silver-dark",
  "slate-dark",
  "argvus-light",
  "github-light",
  "solarized-light",
  "one-light",
  "everforest-light",
  "catppuccin-latte",
  "gruvbox-light",
  "universe",
  "gruvbox-high-dark",
  "gruvbox-dark",
  "rose-pine",
  "tokyo-night",
  "solitude",
  "sunset",
  "matrix",
  "monokai-dark",
  "nord-light",
];

/// Themes the greeter may show before login, besides the ones with a manifest.
const GREETER_BUILTIN: [&str; 42] = [
  "dracula",
  "dracula-float",
  "argvus-dark",
  "argvus-dark-float",
  "silver-dark",
  "silver-dark-float",
  "slate-dark",
  "slate-dark-float",
  "universe",
  "universe-float",
  "argvus-light",
  "argvus-light-float",
  "github-light",
  "github-light-float",
  "solarized-light",
  "solarized-light-float",
  "one-light",
  "one-light-float",
  "everforest-light",
  "everforest-light-float",
  "catppuccin-latte",
  "catppuccin-latte-float",
  "gruvbox-light",
  "gruvbox-light-float",
  "gruvbox-high-dark",
  "gruvbox-high-dark-float",
  "gruvbox-dark",
  "gruvbox-dark-float",
  "rose-pine",
  "rose-pine-float",
  "tokyo-night",
  "tokyo-night-float",
  "solitude",
  "solitude-float",
  "sunset",
  "sunset-float",
  "matrix",
  "matrix-float",
  "monokai-dark",
  "monokai-dark-float",
  "nord-light",
  "nord-light-float",
];

/// The Float variant of a theme is served by its family.
fn family_of(theme: &str) -> &str {
  theme.strip_suffix("-float").unwrap_or(theme)
}

fn is_builtin(theme: &str) -> bool {
  BUILTIN_FAMILIES.contains(&family_of(theme))
}

/// The stylesheet of the removable-devices panel for `theme`.
pub fn storage_stylesheet(theme: &str, themes_dir: &Path) -> Option<String> {
  if is_builtin(theme) {
    // one-light shares the light stylesheet of the argvus family.
    let family = family_of(theme);
    let name = if family == "one-light" {
      "argvus-light"
    } else {
      family
    };
    return Some(format!("{name}.css"));
  }
  let name = theme_resolve::table_value(themes_dir, theme, "appearance", "removable_devices_css")?;
  Some(format!("{name}.css"))
}

/// The fallback stylesheet used when a theme's own one is missing: light
/// families keep the light palette, everything else the dark one.
fn storage_fallback(theme: &str) -> &'static str {
  const LIGHT_PREFIXES: [&str; 7] = [
    "argvus-light",
    "catppuccin-latte",
    "github-light",
    "gruvbox-light",
    "solarized-light",
    "one-light",
    "everforest-light",
  ];
  if LIGHT_PREFIXES
    .iter()
    .any(|prefix| theme.starts_with(prefix))
  {
    "argvus-light.css"
  } else {
    "argvus-dark.css"
  }
}

/// The calendar stylesheet for `theme`. Built-in themes use their own name,
/// `tokyo-night` excepted; drop-ins name theirs in the manifest.
pub fn calendar_stylesheet(theme: &str, themes_dir: &Path) -> Option<String> {
  if is_builtin(theme) {
    return Some(if family_of(theme) == "tokyo-night" {
      "tokyo-night.css".to_string()
    } else {
      format!("{theme}.css")
    });
  }
  let name = theme_resolve::table_value(themes_dir, theme, "appearance", "calendar_css")?;
  Some(format!("{name}.css"))
}

/// Where the removable-devices theme is read from and written to. The caller
/// resolves the user directory and the destination, because the shell's
/// mutable config lookup may create them.
pub struct StorageLocations<'a> {
  pub user_dir: &'a Path,
  pub system_dir: &'a Path,
  pub destination: &'a Path,
  pub workspace_root: Option<&'a Path>,
}

/// Copies the removable-devices stylesheet of `theme` to the panel's theme file.
/// Returns `false` when the theme has no stylesheet to copy, which leaves the
/// current file as it is.
pub fn apply_storage_theme(
  locations: &StorageLocations,
  themes_dir: &Path,
  theme: &str,
) -> io::Result<bool> {
  let Some(name) = storage_stylesheet(theme, themes_dir) else {
    return Ok(false);
  };
  let search = |name: &str| -> Option<PathBuf> {
    let mut candidates = vec![
      locations.user_dir.join(name),
      locations.system_dir.join(name),
      PathBuf::from("/etc/argvus/taskbar/storage/themes").join(name),
    ];
    if let Some(root) = locations.workspace_root {
      candidates.push(
        root
          .join("argvus-removable-devices/src/usr/share/argvus/removable-devices/config/themes")
          .join(name),
      );
    }
    candidates.into_iter().find(|candidate| candidate.is_file())
  };
  let source = match search(&name) {
    Some(source) => source,
    None => {
      // Some consumers ship a smaller palette: keep them in step with the
      // light or dark mode rather than keeping the previous theme.
      let fallback = storage_fallback(theme);
      match search(fallback) {
        Some(source) => source,
        None => return Ok(false),
      }
    }
  };
  if let Some(parent) = locations.destination.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::copy(&source, locations.destination)?;
  Ok(true)
}

/// Writes the calendar stylesheet of `theme` to the calendar cache. A Float
/// variant is its family's stylesheet followed by the variant's own rules,
/// without the imports the variant adds. Returns `false` when there is nothing
/// to write. The calendar is reloaded after a write.
pub fn apply_calendar_theme(
  paths: &SessionPaths,
  themes_dir: &Path,
  cache_file: &Path,
  theme: &str,
  workspace_root: Option<&Path>,
) -> io::Result<bool> {
  let Some(name) = calendar_stylesheet(theme, themes_dir) else {
    return Ok(false);
  };
  let mut candidates = vec![
    paths.read_config(&format!("taskbar/calendar/themes/{name}")),
    PathBuf::from("/etc/argvus/taskbar/calendar/themes").join(&name),
  ];
  if let Some(root) = workspace_root {
    candidates.push(
      root
        .join("argvus-taskbar-calendar/resources/themes")
        .join(&name),
    );
  }
  let Some(source) = candidates.into_iter().find(|candidate| candidate.is_file()) else {
    return Ok(false);
  };

  let mut text = format!("/* argvus-theme: {theme} */\n").into_bytes();
  match name.strip_suffix("-float.css") {
    Some(family) => {
      let base = source.with_file_name(format!("{family}.css"));
      if !base.is_file() {
        return Ok(false);
      }
      text.extend(fs::read(&base)?);
      text.extend(without_imports(&fs::read_to_string(&source)?).into_bytes());
    }
    None => text.extend(fs::read(&source)?),
  }

  if let Some(parent) = cache_file.parent() {
    fs::create_dir_all(parent)?;
  }
  // Write beside the cache file and rename, so the calendar never reads a
  // half-written stylesheet.
  let temporary = cache_file.with_extension(format!("css.{}", std::process::id()));
  let written = fs::write(&temporary, &text).and_then(|()| fs::rename(&temporary, cache_file));
  if let Err(error) = written {
    let _ = fs::remove_file(&temporary);
    return Err(error);
  }
  let _ = Command::new("argvus")
    .args(["taskbar-calendar", "reload"])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  Ok(true)
}

/// Removes the `@import url(...)` lines of a stylesheet, as `sed` did.
fn without_imports(text: &str) -> String {
  text
    .split_inclusive('\n')
    .filter(|line| {
      let trimmed = line.trim_start();
      !(trimmed.starts_with("@import")
        && trimmed["@import".len()..].trim_start().starts_with("url("))
    })
    .collect()
}

/// Publishes the theme id for the greeter. Only themes the greeter can draw are
/// published, and the state directory must already exist. The file is named by
/// the user id and replaced atomically.
pub fn publish_greeter_theme(
  themes_dir: &Path,
  state_dir: &Path,
  theme: &str,
  user_id: u32,
) -> io::Result<bool> {
  let has_manifest = themes_dir.join(theme).join("theme.toml").is_file();
  if !has_manifest && !GREETER_BUILTIN.contains(&theme) {
    return Ok(false);
  }
  if !state_dir.is_dir() {
    return Ok(false);
  }
  let target = state_dir.join(user_id.to_string());
  let temporary = state_dir.join(format!("{user_id}.tmp.{}", std::process::id()));
  let written = (|| -> io::Result<()> {
    let mut file = fs::File::create(&temporary)?;
    writeln!(file, "{theme}")?;
    drop(file);
    // The greeter reads the file as another user; the mode is not secret.
    let _ = fs::set_permissions(&temporary, fs::Permissions::from_mode(0o644));
    fs::rename(&temporary, &target)
  })();
  if let Err(error) = written {
    let _ = fs::remove_file(&temporary);
    return Err(error);
  }
  Ok(true)
}

/// The user id of the calling process.
pub fn current_user_id() -> u32 {
  // SAFETY: getuid cannot fail and has no preconditions.
  unsafe { libc::getuid() }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builtin_storage_names_follow_the_family() {
    let themes = Path::new("/nonexistent");
    assert_eq!(
      storage_stylesheet("github-light-float", themes).as_deref(),
      Some("github-light.css")
    );
    assert_eq!(
      storage_stylesheet("one-light", themes).as_deref(),
      Some("argvus-light.css")
    );
    assert_eq!(storage_stylesheet("nope", themes), None);
  }

  #[test]
  fn calendar_names_keep_the_float_suffix_except_tokyo_night() {
    let themes = Path::new("/nonexistent");
    assert_eq!(
      calendar_stylesheet("github-light-float", themes).as_deref(),
      Some("github-light-float.css")
    );
    assert_eq!(
      calendar_stylesheet("tokyo-night-float", themes).as_deref(),
      Some("tokyo-night.css")
    );
  }

  #[test]
  fn fallback_keeps_light_families_light() {
    assert_eq!(storage_fallback("github-light-float"), "argvus-light.css");
    assert_eq!(storage_fallback("rose-pine"), "argvus-dark.css");
  }

  #[test]
  fn imports_are_removed_and_other_lines_kept() {
    let text = "@import url(\"x.css\");\n  @import url(\"y.css\");\nwindow { color: red; }\n";
    assert_eq!(without_imports(text), "window { color: red; }\n");
    assert_eq!(
      without_imports("@import \"plain.css\";\n"),
      "@import \"plain.css\";\n"
    );
  }

  #[test]
  fn greeter_refuses_themes_it_cannot_draw() {
    let root = std::env::temp_dir().join(format!("argvus-greeter-refuse-{}", std::process::id()));
    fs::create_dir_all(root.join("state")).expect("state");
    assert!(
      !publish_greeter_theme(&root, &root.join("state"), "one-dark-unknown", 1000)
        .expect("publish")
    );
    assert!(!root.join("state/1000").exists());
    let _ = fs::remove_dir_all(root);
  }
}
