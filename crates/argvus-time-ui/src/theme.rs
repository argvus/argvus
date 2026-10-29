//! ARGVUS theme colours for the time applications.
//!
//! Each theme build renders the `time` consumer
//! (`/usr/share/argvus/theme/consumers.d/time.toml`) into
//! `<generated>/theme/current/time/theme.css`: a list of `@define-color`
//! rules. The applications load that file, or, when the build predates the
//! consumer (right after the package is installed), colours derived from the
//! build's `build.toml` (background, accent and light/dark category). Each
//! application then adds its own stylesheet, which only uses those colour
//! names and only matches its own style classes.
//!
//! A theme switch replaces the `current` build, so the style polls a small
//! fingerprint once per second while the window is visible and reloads when
//! it changes, like the taskbar calendar.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use argvus_theme_core::paths::ThemePaths;
use gtk::prelude::*;
use gtk::{gdk, glib};
use serde::Deserialize;

/// Theme consumer shared by ARGVUS Focus and ARGVUS Clock.
pub const CONSUMER: &str = "time";
const CONSUMER_OUTPUT: &str = "time/theme.css";
const BUILD_MANIFEST: &str = "build.toml";
const POLL_INTERVAL: Duration = Duration::from_secs(1);
/// Extra icon directory, set by `cargo run` to the package payload.
pub const ICONS_DIR_VARIABLE: &str = "ARGVUS_ICONS_DIR";

/// ARGVUS Dark, used when no theme build exists at all.
const DEFAULT_BACKGROUND: &str = "#111316";
const DEFAULT_ACCENT: &str = "#3791BE";
const DARK_FOREGROUND: &str = "#DFE5EA";
const LIGHT_FOREGROUND: &str = "#1F2328";

/// Colour names every application stylesheet may use.
const SHARED_CSS: &str = include_str!("shared.css");

#[derive(Debug, Deserialize, Default)]
struct BuildDescription {
  #[serde(default)]
  category: String,
  #[serde(default)]
  accent: String,
  #[serde(default)]
  background: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Fingerprint {
  build: Option<PathBuf>,
  consumer_modified: Option<SystemTime>,
}

/// The theme and application stylesheets of one display.
pub struct ThemeStyle {
  colors: gtk::CssProvider,
  current: Option<PathBuf>,
  fingerprint: RefCell<Option<Fingerprint>>,
}

impl ThemeStyle {
  /// Installs the theme colours, the shared rules and `app_css` on the
  /// default display.
  pub fn install(app_css: &str) -> Rc<Self> {
    let style = Rc::new(Self {
      colors: gtk::CssProvider::new(),
      current: ThemePaths::from_environment().map(|paths| paths.current()),
      fingerprint: RefCell::new(None),
    });
    let Some(display) = gdk::Display::default() else {
      log::warn!("no display: theme styles not installed");
      return style;
    };

    // Running from the source tree: find the application icons of the
    // package payload. Installed, they are in /usr/share/icons/hicolor.
    if let Some(icons) = std::env::var_os(ICONS_DIR_VARIABLE).filter(|path| !path.is_empty()) {
      gtk::IconTheme::for_display(&display).add_search_path(icons);
    }

    let rules = gtk::CssProvider::new();
    rules.load_from_string(&format!("{SHARED_CSS}\n{app_css}"));
    gtk::style_context_add_provider_for_display(
      &display,
      &style.colors,
      gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    gtk::style_context_add_provider_for_display(
      &display,
      &rules,
      gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
    );
    style.reload_if_changed();
    style
  }

  /// Polls for theme switches while `window` is visible, until it is
  /// destroyed.
  pub fn watch(self: &Rc<Self>, window: &impl IsA<gtk::Widget>) {
    let style = Rc::downgrade(self);
    let window = window.downgrade();
    glib::timeout_add_local(POLL_INTERVAL, move || {
      let (Some(style), Some(window)) = (style.upgrade(), window.upgrade()) else {
        return glib::ControlFlow::Break;
      };
      if window.is_visible() {
        style.reload_if_changed();
      }
      glib::ControlFlow::Continue
    });
  }

  fn reload_if_changed(&self) {
    let fingerprint = self.fingerprint();
    if self.fingerprint.borrow().as_ref() == Some(&fingerprint) {
      return;
    }
    self.colors.load_from_string(&self.load_colors());
    *self.fingerprint.borrow_mut() = Some(fingerprint);
  }

  fn fingerprint(&self) -> Fingerprint {
    let build = self
      .current
      .as_deref()
      .and_then(|current| fs::canonicalize(current).ok());
    let consumer_modified = build
      .as_deref()
      .and_then(|build| fs::metadata(build.join(CONSUMER_OUTPUT)).ok())
      .and_then(|metadata| metadata.modified().ok());
    Fingerprint {
      build,
      consumer_modified,
    }
  }

  /// The rendered colour rules, or rules derived from the build. GTK's own
  /// widgets follow the system colour scheme, which the ARGVUS theme sets.
  fn load_colors(&self) -> String {
    let rendered = self
      .current
      .as_deref()
      .and_then(|current| fs::read_to_string(current.join(CONSUMER_OUTPUT)).ok());
    rendered.unwrap_or_else(|| {
      let description = self
        .current
        .as_deref()
        .map(read_build_description)
        .unwrap_or_default();
      fallback_colors(&description)
    })
  }
}

fn read_build_description(current: &Path) -> BuildDescription {
  fs::read_to_string(current.join(BUILD_MANIFEST))
    .ok()
    .and_then(|text| toml::from_str(&text).ok())
    .unwrap_or_default()
}

/// The same colour names the `time` consumer renders, derived with GTK CSS
/// colour functions from the build's background and accent.
fn fallback_colors(description: &BuildDescription) -> String {
  let background = valid_hex(&description.background).unwrap_or(DEFAULT_BACKGROUND);
  let accent = valid_hex(&description.accent).unwrap_or(DEFAULT_ACCENT);
  let foreground = if description.category == "light" {
    LIGHT_FOREGROUND
  } else {
    DARK_FOREGROUND
  };
  let on_accent = readable_on(accent);
  format!(
    "@define-color time_bg {background};\n\
     @define-color time_fg {foreground};\n\
     @define-color time_accent {accent};\n\
     @define-color time_on_accent {on_accent};\n\
     @define-color time_surface mix({background}, {accent}, 0.07);\n\
     @define-color time_surface_strong mix({background}, {accent}, 0.14);\n\
     @define-color time_muted mix({foreground}, {background}, 0.35);\n\
     @define-color time_border alpha({accent}, 0.28);\n\
     @define-color time_focus alpha({accent}, 0.75);\n\
     @define-color time_track alpha({foreground}, 0.12);\n\
     @define-color time_break #5FB38A;\n\
     @define-color time_long_break #9A86D6;\n\
     @define-color time_danger #E06C75;\n"
  )
}

/// `#RRGGBB` or `#RRGGBBAA`, as the build writes them.
fn valid_hex(value: &str) -> Option<&str> {
  let digits = value.strip_prefix('#')?;
  (matches!(digits.len(), 6 | 8) && digits.chars().all(|digit| digit.is_ascii_hexdigit()))
    .then_some(value)
}

/// Black or white text, whichever reads better on `color` (same rule as the
/// taskbar calendar).
fn readable_on(color: &str) -> &'static str {
  let channel = |range: std::ops::Range<usize>| {
    color
      .get(range)
      .and_then(|digits| u32::from_str_radix(digits, 16).ok())
      .unwrap_or(0)
  };
  let brightness = (299 * channel(1..3) + 587 * channel(3..5) + 114 * channel(5..7)) / 1000;
  if brightness >= 128 {
    "#000000"
  } else {
    "#FFFFFF"
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fallback_uses_build_colors_and_category() {
    let description = BuildDescription {
      category: "light".to_owned(),
      accent: "#3590BD".to_owned(),
      background: "#FAFAFA".to_owned(),
    };
    let css = fallback_colors(&description);
    assert!(css.contains("@define-color time_bg #FAFAFA;"));
    assert!(css.contains("@define-color time_accent #3590BD;"));
    assert!(css.contains(&format!("@define-color time_fg {LIGHT_FOREGROUND};")));
  }

  #[test]
  fn invalid_build_colors_fall_back_to_argvus_dark() {
    let description = BuildDescription {
      accent: "blue; } * { color: red".to_owned(),
      ..BuildDescription::default()
    };
    let css = fallback_colors(&description);
    assert!(css.contains(&format!("@define-color time_accent {DEFAULT_ACCENT};")));
    assert!(css.contains(&format!("@define-color time_bg {DEFAULT_BACKGROUND};")));
  }

  #[test]
  fn text_on_accent_is_readable() {
    assert_eq!(readable_on("#3791BE"), "#FFFFFF");
    assert_eq!(readable_on("#F2D45C"), "#000000");
  }

  #[test]
  fn build_description_is_read_from_the_current_build() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
      directory.path().join(BUILD_MANIFEST),
      "theme = \"argvus-light\"\ncategory = \"light\"\naccent = \"#123456\"\nbackground = \"#FFFFFF\"\n",
    )
    .unwrap();
    let description = read_build_description(directory.path());
    assert_eq!(description.category, "light");
    assert_eq!(description.accent, "#123456");
  }

  #[test]
  fn consumer_template_defines_every_fallback_color() {
    let template = include_str!(
      "../../../packaging/archlinux/src/usr/share/argvus/time/theme-template/theme.css"
    );
    let fallback = fallback_colors(&BuildDescription::default());
    for line in fallback.lines() {
      let name = line.split_whitespace().nth(1).expect("color name");
      assert!(
        template.contains(&format!("@define-color {name} ")),
        "the template does not define {name}"
      );
    }
  }
}
