//! The theme build of the active theme, kept in step with the switch.
//!
//! The switch and the accent command render the active theme with
//! `argvus-theme-core` and point `generated/theme/current` at the build. Every
//! themed file of the desktop is read through `current` or copied from it, so
//! the build is required: a theme that cannot be rendered is not applied.

use crate::session_paths::SessionPaths;
use argvus_theme_core::build;
use argvus_theme_core::color::Color;
use argvus_theme_core::paths::ThemePaths;
use argvus_theme_core::render;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Builds kept besides the active one, so a quick switch back reuses a build.
const KEPT_BUILDS: usize = 2;

/// The theme roots of these session paths.
pub fn theme_paths(paths: &SessionPaths) -> ThemePaths {
  let data_home = env::var_os("XDG_DATA_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .or_else(|| env::var_os("HOME").map(|home| Path::new(&home).join(".local/share")))
    .unwrap_or_else(|| paths.config_home.join("../.local/share"));
  ThemePaths {
    system_root: paths.system_config.clone(),
    user_themes: data_home.join("argvus/themes"),
    generated: paths.data_home.join("generated/theme"),
  }
}

/// Renders `theme` (`<id>` or `<id>-float`) with `accent` (`#RRGGBB`, or the
/// theme's own when `None`) and makes it the current build. A failed render
/// changes nothing: the previous build stays current.
pub fn refresh(paths: &SessionPaths, theme: &str, accent: Option<&str>) -> Result<PathBuf, String> {
  let theme_paths = theme_paths(paths);
  let accent = accent
    .map(|text| Color::parse(text.trim()).ok_or_else(|| format!("accent `{text}` is not a colour")))
    .transpose()?;
  let rendered = render::render_installed(&theme_paths, theme, accent, None)?;
  let directory = rendered
    .write_build(&theme_paths.builds())
    .map_err(|error| error.to_string())?;
  build::commit(&theme_paths.current(), &directory).map_err(|error| error.to_string())?;
  let _ = build::collect_garbage(&theme_paths.builds(), &theme_paths.current(), KEPT_BUILDS);
  Ok(directory)
}

/// `refresh`, reporting a failure on stderr. Returns whether the build is current.
pub fn refresh_or_report(paths: &SessionPaths, theme: &str, accent: Option<&str>) -> bool {
  match refresh(paths, theme, accent) {
    Ok(_) => true,
    Err(error) => {
      eprintln!("argvus-config: theme build: {error}");
      false
    }
  }
}

/// The build `current` points at, to put back when a switch fails after its
/// render.
pub fn active_build(paths: &SessionPaths) -> Option<PathBuf> {
  build::active_build(&theme_paths(paths).current())
}

/// Points `current` back at `previous`, or removes it when there was none.
pub fn restore(paths: &SessionPaths, previous: Option<&Path>) {
  let current = theme_paths(paths).current();
  match previous {
    Some(previous) => {
      let _ = build::commit(&current, previous);
    }
    None => {
      let _ = fs::remove_file(current);
    }
  }
}

/// The theme the desktop shows: the theme of the current build, or else the
/// active theme marker. A layout switch rewrites the marker before the theme
/// switch runs, so the build is what tells the previous layout apart.
pub fn displayed_theme(paths: &SessionPaths) -> String {
  build::current_theme(&theme_paths(paths).current()).unwrap_or_else(|| {
    fs::read_to_string(paths.data_home.join(".active-theme"))
      .map(|text| text.trim().to_owned())
      .unwrap_or_default()
  })
}

/// A file of the current build, only when the build was rendered for `theme`
/// (`<id>` or `<id>-float`).
pub fn current_file_for(paths: &SessionPaths, theme: &str, relative: &str) -> Option<PathBuf> {
  build::current_file_for(&theme_paths(paths).current(), theme, relative)
}

/// A file of the current build (`relative` to the build), when it exists.
pub fn current_file(paths: &SessionPaths, relative: &str) -> Option<PathBuf> {
  let path = theme_paths(paths).current().join(relative);
  path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn scratch(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!("argvus-theme-build-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
  }

  fn session(root: &Path) -> SessionPaths {
    SessionPaths {
      config_home: root.join("config"),
      data_home: root.join("config/argvus/data"),
      system_config: root.join("system"),
    }
  }

  fn install_theme(system: &Path) {
    let theme = system.join("themes/sunset");
    fs::create_dir_all(&theme).unwrap();
    fs::write(
      theme.join("theme.toml"),
      "schema = 2\nid = \"sunset\"\nname = \"Sunset\"\ncategory = \"dark\"\n\
       [palette]\nbg = \"#0F0F0F\"\nfg = \"#EADCCC\"\naccent = \"#E2BE8A\"\n",
    )
    .unwrap();
    fs::create_dir_all(system.join("taskbar")).unwrap();
    fs::write(
      system.join("taskbar/theme.css"),
      "accent {{ @accent }} round {{ layout.rounded }}\n",
    )
    .unwrap();
    fs::create_dir_all(system.join("theme/consumers.d")).unwrap();
    fs::write(
      system.join("theme/consumers.d/taskbar.toml"),
      "id = \"taskbar\"\ntemplate = \"taskbar/theme.css\"\noutput = \"taskbar/theme.css\"\n",
    )
    .unwrap();
    fs::create_dir_all(system.join("theme/layouts")).unwrap();
    fs::write(system.join("theme/layouts/sticky.toml"), "rounded = 0\n").unwrap();
    fs::write(system.join("theme/layouts/float.toml"), "rounded = 1\n").unwrap();
  }

  #[test]
  fn refresh_points_current_at_the_rendered_theme() {
    let root = scratch("refresh");
    let paths = session(&root);
    install_theme(&paths.system_config);

    refresh(&paths, "sunset-float", Some("#3590BD")).unwrap();
    let file = current_file(&paths, "taskbar/theme.css").unwrap();
    assert_eq!(
      fs::read_to_string(file).unwrap(),
      "accent #3590BD round 1\n"
    );

    refresh(&paths, "sunset", None).unwrap();
    let file = current_file(&paths, "taskbar/theme.css").unwrap();
    assert_eq!(
      fs::read_to_string(file).unwrap(),
      "accent #E2BE8A round 0\n"
    );
    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn a_theme_without_a_manifest_keeps_the_current_build() {
    let root = scratch("missing");
    let paths = session(&root);
    install_theme(&paths.system_config);
    refresh(&paths, "sunset", None).unwrap();
    let before = active_build(&paths);

    assert!(refresh(&paths, "not-installed", None).is_err());
    assert_eq!(active_build(&paths), before);
    assert!(current_file(&paths, "taskbar/theme.css").is_some());
    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn restore_puts_the_previous_build_back() {
    let root = scratch("restore");
    let paths = session(&root);
    install_theme(&paths.system_config);
    refresh(&paths, "sunset", None).unwrap();
    let previous = active_build(&paths);
    refresh(&paths, "sunset-float", None).unwrap();
    assert_ne!(active_build(&paths), previous);

    restore(&paths, previous.as_deref());
    assert_eq!(active_build(&paths), previous);
    restore(&paths, None);
    assert!(!theme_paths(&paths).current().exists());
    fs::remove_dir_all(root).unwrap();
  }
}
