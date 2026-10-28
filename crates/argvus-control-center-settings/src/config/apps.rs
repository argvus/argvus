//! Default applications backend: desktop files, installed applications per category and the saved choices.
use std::collections::HashMap;

use crate::error::SettingsError;
use argvus_control_center_apps::{
  apply,
  catalog::{Category, find_app},
  detect::{self, DesktopFile, InstalledApp},
  state::AppState,
};

pub struct AppsBackend {
  desktops: Vec<DesktopFile>,
  installed: HashMap<Category, Vec<InstalledApp>>,
  state: AppState,
  current: HashMap<Category, String>,
}

impl AppsBackend {
  /// Builds a placeholder with no desktop entries, used while `load` runs on a background job so the page renders instantly.
  pub fn empty() -> Self {
    Self {
      desktops: Vec::new(),
      installed: HashMap::new(),
      state: AppState::load(),
      current: HashMap::new(),
    }
  }

  pub fn load() -> Self {
    let desktops = detect::scan_desktop_files();
    let installed: HashMap<Category, Vec<InstalledApp>> = Category::ORDER
      .into_iter()
      .map(|category| (category, detect::installed_apps(category, &desktops)))
      .collect();
    let state = AppState::load();
    let mimeapps = std::fs::read_to_string(argvus_control_center_core::paths::mimeapps_list())
      .unwrap_or_default();
    let current = Category::ORDER
      .into_iter()
      .map(|category| {
        let configured = state.effective(category);
        let value = if configured.is_empty() && category.uses_xdg() {
          default_from_mimeapps(
            category,
            &mimeapps,
            installed.get(&category).map_or(&[], Vec::as_slice),
          )
          .unwrap_or_default()
        } else {
          configured
        };
        (category, value)
      })
      .collect();
    Self {
      desktops,
      installed,
      state,
      current,
    }
  }

  pub fn installed(&self, category: Category) -> &[InstalledApp] {
    self.installed.get(&category).map_or(&[], Vec::as_slice)
  }

  pub fn current(&self, category: Category) -> String {
    self.current.get(&category).cloned().unwrap_or_default()
  }

  pub fn is_default(&self, category: Category) -> bool {
    self.state.get(category).is_none()
  }

  pub fn set_default(&mut self, category: Category, binary: &str) -> Result<(), SettingsError> {
    let canonical = find_app(category, binary)
      .map(|app| app.binary)
      .unwrap_or(binary);
    if !detect::binary_available(canonical, &self.desktops) {
      return Err(SettingsError::DefaultApps(format!(
        "'{canonical}' is no longer installed"
      )));
    }

    let previous = self.state.get(category);
    self.state.set(category, canonical);
    if let Err(error) = self.state.save() {
      self.state.set(category, previous.unwrap_or_default());
      return Err(SettingsError::DefaultApps(error.to_string()));
    }
    if let Err(error) = apply::apply(category, canonical, &self.desktops) {
      self.state.set(category, previous.unwrap_or_default());
      let _ = self.state.save();
      return Err(SettingsError::DefaultApps(error));
    }
    self.current.insert(category, canonical.to_string());
    Ok(())
  }

  pub fn reset_default(&mut self, category: Category) -> Result<(), SettingsError> {
    let previous = self.state.get(category);
    self.state.reset(category);
    if let Err(error) = self.state.save() {
      self.state.set(category, previous.unwrap_or_default());
      return Err(SettingsError::DefaultApps(error.to_string()));
    }
    let value = self.state.effective(category);
    if !value.is_empty() {
      if let Err(error) = apply::apply(category, &value, &self.desktops) {
        self.state.set(category, previous.unwrap_or_default());
        let _ = self.state.save();
        return Err(SettingsError::DefaultApps(error));
      }
    } else {
      apply::refresh_argvus();
    }
    *self = Self::load();
    Ok(())
  }

  pub fn reset_all(&mut self) -> Result<(), SettingsError> {
    let previous = self.state.clone();
    self.state.reset_all();
    if let Err(error) = self.state.save() {
      self.state = previous;
      return Err(SettingsError::DefaultApps(error.to_string()));
    }
    for category in Category::ORDER {
      let value = self.state.effective(category);
      if !value.is_empty()
        && let Err(error) = apply::apply(category, &value, &self.desktops)
      {
        self.state = previous;
        let _ = self.state.save();
        return Err(SettingsError::DefaultApps(error));
      }
    }
    apply::refresh_argvus();
    *self = Self::load();
    Ok(())
  }
}

fn default_from_mimeapps(
  category: Category,
  contents: &str,
  installed: &[InstalledApp],
) -> Option<String> {
  let mut in_defaults = false;
  for line in contents.lines() {
    let line = line.trim();
    if line.starts_with('[') && line.ends_with(']') {
      in_defaults = line == "[Default Applications]";
      continue;
    }
    if !in_defaults {
      continue;
    }
    let Some((mime, desktop_ids)) = line.split_once('=') else {
      continue;
    };
    if !category.mimes().contains(&mime.trim()) {
      continue;
    }
    for desktop_id in desktop_ids.split(';').filter(|value| !value.is_empty()) {
      if let Some(app) = installed.iter().find(|app| {
        app
          .desktop_id
          .as_deref()
          .is_some_and(|installed_id| installed_id.eq_ignore_ascii_case(desktop_id.trim()))
      }) {
        return Some(app.binary.clone());
      }
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolves_xdg_default_from_default_applications_section() {
    let installed = vec![InstalledApp {
      binary: "firefox".into(),
      display: "Firefox".into(),
      desktop_id: Some("firefox.desktop".into()),
      tui: false,
    }];
    let contents = "[Added Associations]\ntext/html=other.desktop;\n\
                    [Default Applications]\ntext/html=firefox.desktop;\n";
    assert_eq!(
      default_from_mimeapps(Category::Browser, contents, &installed).as_deref(),
      Some("firefox")
    );
  }

  #[test]
  fn internal_categories_do_not_resolve_from_mime() {
    assert_eq!(default_from_mimeapps(Category::Launcher, "", &[]), None);
  }
}
