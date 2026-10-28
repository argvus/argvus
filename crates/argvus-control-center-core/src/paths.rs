//! Shared filesystem path resolution.
use std::env;
use std::path::PathBuf;

pub fn home() -> PathBuf {
  env::var_os("HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/tmp"))
}

pub fn config_home() -> PathBuf {
  env::var_os("XDG_CONFIG_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| home().join(".config"))
}

pub fn argvus_config_home() -> PathBuf {
  env::var_os("ARGVUS_CONFIG_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(config_home)
    .join("argvus")
}

/// Returns the mutable runtime tree inside the ARGVUS configuration root.
///
/// The root holds exactly two entries: `config/` (the canonical modular
/// document) and `data/` (every managed component tree, generated file and
/// state marker). Prefer this over joining onto [`argvus_config_home`]
/// directly so a new consumer cannot reintroduce a root-level path.
pub fn argvus_data_home() -> PathBuf {
  argvus_config_home().join("data")
}

pub fn legacy_argvus_state_home() -> PathBuf {
  let base = env::var_os("XDG_STATE_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| home().join(".local").join("state"));
  base.join("argvus")
}

pub fn active_argvus_theme_file() -> PathBuf {
  argvus_data_home().join(".active-theme")
}

pub fn cache_home() -> PathBuf {
  env::var_os("XDG_CACHE_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| home().join(".cache"))
}

/// Returns the user's XDG data directory for durable, non-configuration data.
pub fn data_home() -> PathBuf {
  env::var_os("XDG_DATA_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| home().join(".local").join("share"))
}

pub fn defaults_file() -> PathBuf {
  argvus_data_home().join("control-center/defaults.json")
}

pub fn fonts_file() -> PathBuf {
  argvus_data_home().join("generated/fonts.conf")
}

pub fn system_config_root() -> PathBuf {
  env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
}

pub fn legacy_defaults_file() -> PathBuf {
  legacy_argvus_state_home().join("defaults.json")
}

pub fn mimeapps_list() -> PathBuf {
  config_home().join("mimeapps.list")
}

pub fn user_applications_dir() -> PathBuf {
  env::var_os("XDG_DATA_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| home().join(".local").join("share"))
    .join("applications")
}

pub fn system_applications_dirs() -> Vec<PathBuf> {
  let mut dirs = Vec::new();
  if let Some(list) = env::var_os("XDG_DATA_DIRS") {
    for d in env::split_paths(&list) {
      if !d.as_os_str().is_empty() {
        dirs.push(d);
      }
    }
  }
  if dirs.is_empty() {
    dirs.push(PathBuf::from("/usr/local/share"));
    dirs.push(PathBuf::from("/usr/share"));
  }
  dirs.into_iter().map(|d| d.join("applications")).collect()
}
