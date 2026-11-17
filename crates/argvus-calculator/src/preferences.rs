//! User-owned calculator preferences, independent of window geometry.
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
  pub scientific: bool,
}

pub fn path() -> io::Result<PathBuf> {
  let config = std::env::var_os("XDG_CONFIG_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    .ok_or_else(|| io::Error::other("user config directory unavailable"))?;
  Ok(config.join("argvus/calculator/settings.json"))
}
pub fn load(path: &std::path::Path) -> io::Result<Preferences> {
  match std::fs::read(path) {
    Ok(data) => Ok(serde_json::from_slice(&data)?),
    Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Preferences::default()),
    Err(error) => Err(error),
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn restores_scientific_and_basic_modes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("settings.json");
    assert!(!load(&path).unwrap().scientific);
    crate::history::save_json(&path, &Preferences { scientific: true }).unwrap();
    assert!(load(&path).unwrap().scientific);
    crate::history::save_json(&path, &Preferences { scientific: false }).unwrap();
    assert!(!load(&path).unwrap().scientific);
  }
}
