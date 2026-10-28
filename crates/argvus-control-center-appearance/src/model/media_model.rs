//! Wallpapers and icon themes offered by the appearance pages.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperCollection {
  Abstract,
  Landscape,
}

impl WallpaperCollection {
  pub const ALL: [Self; 2] = [Self::Abstract, Self::Landscape];

  pub fn label_key(self) -> &'static str {
    match self {
      Self::Abstract => "control_center.wallpaper_category_abstract",
      Self::Landscape => "control_center.wallpaper_category_landscape",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperMode {
  Dark,
  Light,
}

impl WallpaperMode {
  pub const ALL: [Self; 2] = [Self::Dark, Self::Light];

  pub fn label_key(self) -> &'static str {
    match self {
      Self::Dark => "control_center.wallpaper_category_dark",
      Self::Light => "control_center.wallpaper_category_light",
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperEntry {
  pub path: String,
  pub collection: WallpaperCollection,
  pub mode: WallpaperMode,
}

/// Where an installed icon theme was found. Only system themes can be
/// official, so the Icons page can tell the two apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconThemeSource {
  /// Installed by a package under `/usr/share/icons`.
  System,
  /// Found in `~/.icons` or `$XDG_DATA_HOME/icons`.
  User,
}

/// The variant an official ARGVUS icon theme declares in its manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconThemeVariant {
  Default,
  Dark,
  Light,
}

/// One icon theme the user can select. `directory_name` is the value stored
/// in `/appearance/icon_theme`, because the shell helper looks the theme up
/// by that directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconThemeEntry {
  pub directory_name: String,
  /// `Name=` from `index.theme`, falling back to the directory name.
  pub display_name: String,
  pub comment: Option<String>,
  pub source: IconThemeSource,
  /// Set only for system themes that have an official ARGVUS manifest.
  pub official: Option<IconThemeVariant>,
}
