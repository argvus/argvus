//! Where themes, templates and builds live.
//!
//! | Path | Owner |
//! |---|---|
//! | `<system>/themes/<id>/theme.toml` | official theme packages |
//! | `<user data>/argvus/themes/<id>/theme.toml` | themes the user installed |
//! | `<system>/theme/consumers.d/*.toml` | the consumer registry of each component |
//! | `<system>/theme/layouts/{sticky,float}.toml` | layout geometry |
//! | `<argvus data>/generated/theme/{builds,current}` | rendered builds, derived state |

use std::env;
use std::path::{Component, Path, PathBuf};

const DEFAULT_SYSTEM_CONFIG: &str = "/usr/share/argvus";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePaths {
  /// `ARGVUS_SYSTEM_CONFIG`, the packaged files (`/usr/share/argvus`).
  pub system_root: PathBuf,
  /// `$XDG_DATA_HOME/argvus/themes`.
  pub user_themes: PathBuf,
  /// `<argvus data>/generated/theme`, which holds `builds/` and `current`.
  pub generated: PathBuf,
}

impl ThemePaths {
  /// Resolves the roots like `argvus config` does: `ARGVUS_CONFIG_HOME`, then
  /// `XDG_CONFIG_HOME`, then `~/.config` hold `argvus/data`, unless
  /// `ARGVUS_DATA_HOME` names it directly.
  pub fn from_environment() -> Option<Self> {
    let home = non_empty_var("HOME");
    let config_home = non_empty_var("ARGVUS_CONFIG_HOME")
      .or_else(|| non_empty_var("XDG_CONFIG_HOME"))
      .or_else(|| home.as_ref().map(|home| home.join(".config")))?;
    let argvus_data =
      non_empty_var("ARGVUS_DATA_HOME").unwrap_or_else(|| config_home.join("argvus/data"));
    let data_home = non_empty_var("XDG_DATA_HOME")
      .or_else(|| home.as_ref().map(|home| home.join(".local/share")))?;
    Some(Self {
      system_root: non_empty_var("ARGVUS_SYSTEM_CONFIG")
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SYSTEM_CONFIG)),
      user_themes: data_home.join("argvus/themes"),
      generated: argvus_data.join("generated/theme"),
    })
  }

  pub fn system_themes(&self) -> PathBuf {
    self.system_root.join("themes")
  }

  pub fn registry(&self) -> PathBuf {
    self.system_root.join("theme/consumers.d")
  }

  pub fn layouts(&self) -> PathBuf {
    self.system_root.join("theme/layouts")
  }

  pub fn builds(&self) -> PathBuf {
    self.generated.join("builds")
  }

  /// The single pointer the consumers read: a symlink to the active build.
  pub fn current(&self) -> PathBuf {
    self.generated.join("current")
  }
}

fn non_empty_var(name: &str) -> Option<PathBuf> {
  env::var_os(name)
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
}

/// A relative path made only of normal components, so joining it to a
/// directory cannot leave that directory.
pub fn relative_inside(value: &str) -> Result<PathBuf, String> {
  let path = Path::new(value);
  if value.is_empty() {
    return Err("path must not be empty".into());
  }
  if !path
    .components()
    .all(|component| matches!(component, Component::Normal(_)))
  {
    return Err(format!("`{value}` must be a relative path without `..`"));
  }
  Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn relative_paths_cannot_escape() {
    assert_eq!(
      relative_inside("taskbar/theme.css").unwrap(),
      Path::new("taskbar/theme.css")
    );
    for value in ["", "/etc/passwd", "../x", "a/../../x", "./a"] {
      assert!(relative_inside(value).is_err(), "{value}");
    }
  }

  #[test]
  fn derived_locations_hang_off_the_roots() {
    let paths = ThemePaths {
      system_root: PathBuf::from("/usr/share/argvus"),
      user_themes: PathBuf::from("/home/u/.local/share/argvus/themes"),
      generated: PathBuf::from("/home/u/.config/argvus/data/generated/theme"),
    };
    assert_eq!(paths.system_themes(), Path::new("/usr/share/argvus/themes"));
    assert_eq!(
      paths.registry(),
      Path::new("/usr/share/argvus/theme/consumers.d")
    );
    assert_eq!(
      paths.current(),
      Path::new("/home/u/.config/argvus/data/generated/theme/current")
    );
  }
}
