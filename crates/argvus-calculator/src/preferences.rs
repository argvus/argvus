//! Persisted profile and per-profile options; legacy scientific flags remain readable.
use crate::{conversion::Category, financial::Operation, programming::Options};
use serde::{Deserialize, Serialize};
use std::{
  io,
  path::{Path, PathBuf},
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
  #[default]
  Basic,
  Scientific,
  Programming,
  Financial,
  Conversion,
}
impl Profile {
  pub const ALL: [Self; 5] = [
    Self::Basic,
    Self::Scientific,
    Self::Programming,
    Self::Financial,
    Self::Conversion,
  ];
  pub fn key(self) -> &'static str {
    match self {
      Self::Basic => "mode.basic",
      Self::Scientific => "mode.scientific",
      Self::Programming => "mode.programming",
      Self::Financial => "mode.financial",
      Self::Conversion => "mode.conversion",
    }
  }
  pub fn size(self) -> (i32, i32) {
    match self {
      Self::Basic => (370, 420),
      Self::Scientific | Self::Financial => (440, 640),
      Self::Programming => (520, 640),
      Self::Conversion => (440, 520),
    }
  }
  pub fn index(self) -> usize {
    Self::ALL
      .iter()
      .position(|profile| *profile == self)
      .unwrap_or(0)
  }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
  pub profile: Profile,
  pub radians: bool,
  pub programming: Options,
  pub financial: Operation,
  pub category: Category,
  pub from: usize,
  pub to: usize,
}
impl Default for Preferences {
  fn default() -> Self {
    Self {
      profile: Profile::Basic,
      radians: false,
      programming: Options::default(),
      financial: Operation::default(),
      category: Category::default(),
      from: 0,
      to: 1,
    }
  }
}
impl Preferences {
  pub fn validate(&mut self) {
    if self.programming.validate().is_err() {
      self.programming = Options::default();
    }
    let count = self.category.units().len();
    if self.from >= count {
      self.from = 0;
    }
    if self.to >= count {
      self.to = 1;
    }
  }
}
pub fn path() -> io::Result<PathBuf> {
  let root = std::env::var_os("XDG_CONFIG_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    .ok_or_else(|| io::Error::other("user config directory unavailable"))?;
  Ok(root.join("argvus/calculator/settings.json"))
}
pub fn load(path: &Path) -> io::Result<Preferences> {
  let bytes = match std::fs::read(path) {
    Ok(bytes) => bytes,
    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Preferences::default()),
    Err(error) => return Err(error),
  };
  let value: serde_json::Value = serde_json::from_slice(&bytes)?;
  let legacy = value.get("profile").is_none()
    && value.get("scientific").and_then(serde_json::Value::as_bool) == Some(true);
  let mut preferences: Preferences = serde_json::from_value(value)?;
  if legacy {
    preferences.profile = Profile::Scientific;
  }
  preferences.validate();
  Ok(preferences)
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn validates_units_and_preserves_profile_options() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("settings.json");
    let preferences = Preferences {
      profile: Profile::Conversion,
      radians: true,
      category: Category::Data,
      from: 8,
      to: 3,
      programming: Options {
        bits: 16,
        signed: false,
        base: crate::programming::Base::Hexadecimal,
      },
      financial: Operation::Future,
    };
    crate::history::save_json(&path, &preferences).unwrap();
    let loaded = load(&path).unwrap();
    assert_eq!(loaded.from, 8);
    assert_eq!(loaded.to, 3);
    assert!(loaded.radians);
    assert_eq!(loaded.programming, preferences.programming);
    assert_eq!(loaded.financial, Operation::Future);
    std::fs::write(
      &path,
      r#"{"profile":"conversion","from":999,"to":999,"programming":{"bits":3}}"#,
    )
    .unwrap();
    let loaded = load(&path).unwrap();
    assert_eq!(loaded.from, 0);
    assert_eq!(loaded.to, 1);
    assert_eq!(loaded.programming, Options::default());
  }
  #[test]
  fn migrates_legacy_and_restores_profiles() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("settings.json");
    assert_eq!(load(&path).unwrap().profile, Profile::Basic);
    for (flag, profile) in [(true, Profile::Scientific), (false, Profile::Basic)] {
      std::fs::write(&path, format!("{{\"scientific\":{flag}}}")).unwrap();
      assert_eq!(load(&path).unwrap().profile, profile);
    }
    for profile in Profile::ALL {
      let preferences = Preferences {
        profile,
        ..Preferences::default()
      };
      crate::history::save_json(&path, &preferences).unwrap();
      assert_eq!(load(&path).unwrap().profile, profile);
    }
  }
}
