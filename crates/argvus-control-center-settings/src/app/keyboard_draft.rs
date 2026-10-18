//! Draft of the Keyboard > Layout page: the enabled layouts and the default
//! one, edited in memory until `Apply` hands them to the keyboard backend.
//!
//! Space and Enter only change this draft, so the Hyprland configuration is
//! written once, by `Apply`.

use crate::system::keyboard::KeyboardInfo;

/// Message shown when the last enabled layout is switched off.
pub(super) const LAST_LAYOUT: &str = "at least one keyboard layout must remain selected";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct LayoutDraft {
  /// Enabled layout codes, in the order Hyprland lists them.
  pub(super) selected: Vec<String>,
  /// The layout Hyprland uses first; always one of `selected`.
  pub(super) default: String,
}

impl LayoutDraft {
  /// The configured state: the Hyprland layout list, whose first entry is
  /// the default.
  pub(super) fn loaded(info: &KeyboardInfo) -> Self {
    let selected: Vec<String> = info
      .hypr_layout
      .split(',')
      .map(str::trim)
      .filter(|layout| !layout.is_empty())
      .map(str::to_string)
      .collect();
    let default = selected.first().cloned().unwrap_or_default();
    Self { selected, default }
  }

  pub(super) fn is_enabled(&self, code: &str) -> bool {
    self.selected.iter().any(|layout| layout == code)
  }

  pub(super) fn is_default(&self, code: &str) -> bool {
    self.default == code
  }

  /// Enables or disables `code`. The last enabled layout cannot be removed;
  /// removing the default hands it to the first remaining layout.
  pub(super) fn toggle(&mut self, code: &str) -> Result<(), &'static str> {
    if let Some(position) = self.selected.iter().position(|layout| layout == code) {
      if self.selected.len() == 1 {
        return Err(LAST_LAYOUT);
      }
      self.selected.remove(position);
      if self.default == code {
        self.default = self.selected.first().cloned().unwrap_or_default();
      }
    } else {
      self.selected.push(code.to_string());
    }
    Ok(())
  }

  /// Makes `code` the default, enabling it first when it is off.
  pub(super) fn set_default(&mut self, code: &str) {
    if !self.is_enabled(code) {
      self.selected.push(code.to_string());
    }
    self.default = code.to_string();
  }

  /// Whether the draft differs from `loaded`, ignoring the order of the
  /// enabled layouts.
  pub(super) fn changed_from(&self, loaded: &Self) -> bool {
    self.default != loaded.default
      || self.selected.len() != loaded.selected.len()
      || self
        .selected
        .iter()
        .any(|layout| !loaded.is_enabled(layout))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn draft(layouts: &str) -> LayoutDraft {
    LayoutDraft::loaded(&KeyboardInfo {
      hypr_layout: layouts.to_string(),
      ..KeyboardInfo::default()
    })
  }

  #[test]
  fn loaded_default_is_the_first_hyprland_layout() {
    let loaded = draft("br,us");
    assert_eq!(loaded.default, "br");
    assert_eq!(loaded.selected, ["br", "us"]);
    assert!(loaded.is_default("br"));
    assert!(!loaded.is_default("us"));
  }

  #[test]
  fn toggle_adds_and_removes_without_touching_the_default() {
    let mut layouts = draft("br,us");
    layouts.toggle("de").unwrap();
    assert!(layouts.is_enabled("de"));
    layouts.toggle("us").unwrap();
    assert!(!layouts.is_enabled("us"));
    assert_eq!(layouts.default, "br");
  }

  #[test]
  fn removing_the_default_hands_it_to_the_first_remaining_layout() {
    let mut layouts = draft("br,us,de");
    layouts.toggle("br").unwrap();
    assert_eq!(layouts.default, "us");
    assert_eq!(layouts.selected, ["us", "de"]);
  }

  #[test]
  fn the_last_enabled_layout_cannot_be_removed() {
    let mut layouts = draft("br");
    assert_eq!(layouts.toggle("br"), Err(LAST_LAYOUT));
    assert!(layouts.is_enabled("br"));
  }

  #[test]
  fn set_default_enables_the_layout_first() {
    let mut layouts = draft("br");
    layouts.set_default("us");
    assert!(layouts.is_enabled("us"));
    assert!(layouts.is_default("us"));
    assert_eq!(layouts.selected, ["br", "us"]);
  }

  #[test]
  fn changed_from_detects_default_and_membership_changes() {
    let loaded = draft("br,us");
    assert!(!draft("br,us").changed_from(&loaded), "untouched draft");
    let mut moved = draft("br,us");
    moved.set_default("us");
    assert!(moved.changed_from(&loaded), "default moved");
    let mut added = draft("br,us");
    added.toggle("de").unwrap();
    assert!(added.changed_from(&loaded), "membership changed");
  }
}
