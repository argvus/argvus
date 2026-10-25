//! Structured edits of `key = value` settings in the theme configuration files.
//!
//! Two layouts are used by ARGVUS: the spaced `key = value` form of TOML-like
//! and Hyprland-like files, and the compact `key=value` form of the GTK
//! `[Settings]` INI files. Both replace every existing line for the key, or
//! append one when the key is absent, as the shell helpers did.

use regex::Regex;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// How a setting line is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
  /// `key = value`, with the separator spaced.
  Spaced,
  /// `key=value`, as GTK `[Settings]` files use.
  Compact,
}

impl Layout {
  fn render(self, key: &str, value: &str) -> String {
    match self {
      Self::Spaced => format!("{key} = {value}"),
      Self::Compact => format!("{key}={value}"),
    }
  }
}

/// Sets `key` to `value` in a file whose text is `text`.
///
/// Every line that starts with the key (after optional blanks) and an `=` is
/// rewritten. When there is none, the setting is appended after a blank line
/// (spaced layout), or on its own line (compact layout).
pub fn set_setting(text: &str, key: &str, value: &str, layout: Layout) -> String {
  let pattern = Regex::new(&format!(
    r"^[[:space:]]*{}[[:space:]]*=",
    regex::escape(key)
  ))
  .expect("escaped key is a valid pattern");
  let rendered = layout.render(key, value);
  if text.lines().any(|line| pattern.is_match(line)) {
    return text
      .split_inclusive('\n')
      .map(|line| {
        let (content, ending) = split_ending(line);
        if pattern.is_match(content) {
          format!("{rendered}{ending}")
        } else {
          line.to_string()
        }
      })
      .collect();
  }
  match layout {
    Layout::Spaced => format!("{text}\n{rendered}\n"),
    Layout::Compact => format!("{text}{rendered}\n"),
  }
}

/// Sets a waybar `"margin-<edge>"` value in a JSONC text. Comments and other
/// keys are left as they are. The first match on each line is replaced, as the
/// `sed` edit did. `signed` allows a leading minus in the value being replaced.
pub fn set_jsonc_margin(text: &str, edge: &str, value: u32, signed: bool) -> String {
  let sign = if signed { "-?" } else { "" };
  let pattern = Regex::new(&format!(
    r#""margin-{}": {sign}[0-9]*"#,
    regex::escape(edge)
  ))
  .expect("escaped edge is a valid pattern");
  let replacement = format!(r#""margin-{edge}": {value}"#);
  text
    .split_inclusive('\n')
    .map(|line| {
      let (content, ending) = split_ending(line);
      format!(
        "{}{ending}",
        pattern.replace(content, regex::NoExpand(&replacement))
      )
    })
    .collect()
}

/// Sets the waybar `"height"` of a JSONC text. Only the first match is replaced,
/// as the taskbar file has a single height. Comments and other keys are kept.
pub fn set_jsonc_height(text: &str, value: u32) -> String {
  let pattern = Regex::new(r#""height": [0-9]+"#).expect("valid pattern");
  pattern
    .replace(text, format!(r#""height": {value}"#).as_str())
    .into_owned()
}

/// The waybar `"height"` of a JSONC text, if it has one.
pub fn jsonc_height(text: &str) -> Option<u32> {
  let pattern = Regex::new(r#""height": ([0-9]+)"#).expect("valid pattern");
  pattern.captures(text)?[1].parse().ok()
}

/// A named line in one of the appearance configuration files. Each target has
/// one pattern and one line template; the first match on each line is replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
  /// `@theme "…theme.rasi"` in the Rofi config.
  RofiTheme,
  /// `font: "…";` in the Rofi config (only the value is replaced).
  RofiFont,
  /// `@import "…/themes/…/theme.rasi"` in the Rofi theme.
  RofiImportTheme,
  /// `@import "…mode.rasi"` in the Rofi theme.
  RofiImportMode,
  /// The `monitor` line of hyprpaper.conf.
  HyprpaperMonitor,
  /// The `path` line of hyprpaper.conf.
  HyprpaperPath,
  /// `color_scheme_path=` in qt6ct.conf.
  Qt6ctColorScheme,
  /// `custom_palette=` in qt6ct.conf.
  Qt6ctCustomPalette,
  /// `name = …` in the native GTK settings file.
  NativeName,
  /// `@import url("…/waybar/mode.css")` in the taskbar and telemetry styles.
  WaybarModeImport,
  /// `@import url("./themes/…/theme.css")` in the taskbar style.
  WaybarThemeImport,
  /// `@import url("./themes/…/widget-telemetry-theme.css")` in the telemetry style.
  WaybarTelemetryThemeImport,
  /// `rofi -config … -show drun` in the Waybar configuration.
  WaybarRofiDrun,
}

impl Target {
  /// Parses the name used on the command line.
  pub fn parse(name: &str) -> Option<Self> {
    match name {
      "rofi-theme" => Some(Self::RofiTheme),
      "rofi-font" => Some(Self::RofiFont),
      "rofi-import-theme" => Some(Self::RofiImportTheme),
      "rofi-import-mode" => Some(Self::RofiImportMode),
      "hyprpaper-monitor" => Some(Self::HyprpaperMonitor),
      "hyprpaper-path" => Some(Self::HyprpaperPath),
      "qt6ct-color-scheme" => Some(Self::Qt6ctColorScheme),
      "qt6ct-custom-palette" => Some(Self::Qt6ctCustomPalette),
      "native-name" => Some(Self::NativeName),
      "waybar-mode-import" => Some(Self::WaybarModeImport),
      "waybar-theme-import" => Some(Self::WaybarThemeImport),
      "waybar-telemetry-theme-import" => Some(Self::WaybarTelemetryThemeImport),
      "waybar-rofi-drun" => Some(Self::WaybarRofiDrun),
      _ => None,
    }
  }

  fn pattern(self) -> &'static str {
    match self {
      Self::RofiTheme => r#"^@theme ".*theme.rasi"$"#,
      Self::RofiFont => r#"font: ".*";"#,
      Self::RofiImportTheme => r#"^@import ".*themes/.*/theme.rasi"$"#,
      Self::RofiImportMode => r#"^@import ".*mode.rasi"$"#,
      Self::HyprpaperMonitor => r"^[[:space:]]*monitor[[:space:]]*=.*$",
      Self::HyprpaperPath => r"^[[:space:]]*path[[:space:]]*=.*$",
      Self::Qt6ctColorScheme => r"^color_scheme_path=.*",
      Self::Qt6ctCustomPalette => r"^custom_palette=.*",
      Self::NativeName => r"^[[:space:]]*name[[:space:]]*=.*",
      Self::WaybarModeImport => {
        r#"@import url\("/usr/share/argvus/appearance/config/waybar/mode\.css"\);"#
      }
      Self::WaybarThemeImport => r#"@import url\("\./themes/.*/theme\.css"\);"#,
      Self::WaybarTelemetryThemeImport => {
        r#"@import url\("\./themes/.*/widget-telemetry-theme\.css"\);"#
      }
      Self::WaybarRofiDrun => r"rofi -config [^ ]* -show drun",
    }
  }

  fn template(self, value: &str) -> String {
    match self {
      Self::RofiTheme | Self::RofiImportTheme | Self::RofiImportMode => {
        format!("@{} \"{value}\"", self.keyword())
      }
      Self::RofiFont => format!("font: \"{value}\";"),
      Self::HyprpaperMonitor => format!("  monitor = {value}"),
      Self::HyprpaperPath => format!("  path =  {value}"),
      Self::Qt6ctColorScheme => format!("color_scheme_path={value}"),
      Self::Qt6ctCustomPalette => format!("custom_palette={value}"),
      Self::NativeName => format!("name = {value}"),
      Self::WaybarModeImport => format!(r#"@import url("{value}");"#),
      Self::WaybarThemeImport => format!(r#"@import url("./themes/{value}/theme.css");"#),
      Self::WaybarTelemetryThemeImport => {
        format!(r#"@import url("./themes/{value}/widget-telemetry-theme.css");"#)
      }
      Self::WaybarRofiDrun => format!("rofi -config {value} -show drun"),
    }
  }

  fn keyword(self) -> &'static str {
    match self {
      Self::RofiTheme => "theme",
      _ => "import",
    }
  }
}

/// Applies a target edit to a file text. Lines that do not match are kept, and
/// no line is added when the target is absent: the shell `sed` did the same.
pub fn set_target(text: &str, target: Target, value: &str) -> String {
  let pattern = Regex::new(target.pattern()).expect("target pattern is valid");
  let template = target.template(value);
  text
    .split_inclusive('\n')
    .map(|line| {
      let (content, ending) = split_ending(line);
      let edited = pattern.replace(content, regex::NoExpand(&template));
      format!("{edited}{ending}")
    })
    .collect()
}

/// Writes `contents` over `path` atomically, keeping its permissions. An
/// identical file is left untouched.
pub fn replace_file(path: &Path, contents: &str) -> std::io::Result<()> {
  // A dotfile manager often links the file: replace the file the link points to, so
  // the link keeps working and is not turned into a regular file.
  let target = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
  if fs::read_to_string(&target).ok().as_deref() == Some(contents) {
    return Ok(());
  }
  // The temporary name is unique to this file and this write, so two edits of
  // sibling files (for example `argvus-taskbar.css` and `.jsonc`) never share it.
  let name = target
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or("file");
  let stamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|elapsed| elapsed.as_nanos())
    .unwrap_or_default();
  let temporary =
    target.with_file_name(format!(".{name}.argvus-tmp-{}-{stamp}", std::process::id()));
  let mut file = fs::OpenOptions::new()
    .create_new(true)
    .write(true)
    .open(&temporary)?;
  let written = file
    .write_all(contents.as_bytes())
    .and_then(|()| file.sync_all());
  drop(file);
  if let Err(error) = written {
    let _ = fs::remove_file(&temporary);
    return Err(error);
  }
  if let Ok(metadata) = fs::metadata(&target) {
    fs::set_permissions(&temporary, metadata.permissions())?;
  }
  fs::rename(&temporary, &target)?;
  if let Some(parent) = target.parent() {
    fs::File::open(parent)?.sync_all()?;
  }
  Ok(())
}

/// Sets one `key=value` in a GTK `settings.ini`, creating the file and its
/// `[Settings]` header when they are missing. The file is replaced atomically.
pub fn set_ini_setting(path: &Path, key: &str, value: &str) -> io::Result<()> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  if !path.is_file() {
    fs::write(path, "[Settings]\n")?;
  }
  let text = ensure_settings_header(&fs::read_to_string(path)?);
  let edited = set_setting(&text, key, value, Layout::Compact);
  replace_file(path, &edited)
}

/// Makes sure a GTK settings text starts a `[Settings]` section. A text that
/// already has one is returned unchanged.
pub fn ensure_settings_header(text: &str) -> String {
  if text.lines().any(|line| line == "[Settings]") {
    return text.to_string();
  }
  format!("[Settings]\n{text}")
}

fn split_ending(line: &str) -> (&str, &str) {
  match line.strip_suffix('\n') {
    Some(content) => (content, "\n"),
    None => (line, ""),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn spaced_setting_replaces_every_matching_line() {
    let text = "font_size = 10\n  font_size=12\nother = 1\n";
    assert_eq!(
      set_setting(text, "font_size", "13", Layout::Spaced),
      "font_size = 13\nfont_size = 13\nother = 1\n"
    );
  }

  #[test]
  fn spaced_setting_is_appended_after_a_blank_line_when_absent() {
    assert_eq!(
      set_setting("other = 1\n", "theme", "\"x\"", Layout::Spaced),
      "other = 1\n\ntheme = \"x\"\n"
    );
  }

  #[test]
  fn compact_setting_is_appended_without_a_blank_line() {
    assert_eq!(
      set_setting("[Settings]\n", "gtk-font-name", "Inter 11", Layout::Compact),
      "[Settings]\ngtk-font-name=Inter 11\n"
    );
  }

  #[test]
  fn key_is_matched_as_a_whole_key_only() {
    // `font` must not match the line for `font_size`.
    assert_eq!(
      set_setting("font_size = 10\n", "font", "x", Layout::Spaced),
      "font_size = 10\n\nfont = x\n"
    );
  }

  #[test]
  fn settings_header_is_added_once() {
    assert_eq!(ensure_settings_header("a=1\n"), "[Settings]\na=1\n");
    assert_eq!(
      ensure_settings_header("[Settings]\na=1\n"),
      "[Settings]\na=1\n"
    );
  }

  #[test]
  fn jsonc_margin_replaces_the_first_value_per_line_and_keeps_comments() {
    let text =
      "{\n  // margin-top: 9 in a comment\n  \"margin-top\": 1,\n  \"margin-bottom\": -4,\n}\n";
    let unsigned = set_jsonc_margin(text, "bottom", 0, false);
    assert!(
      unsigned.contains("\"margin-bottom\": 0-4,"),
      "unsigned keeps the sign: {unsigned}"
    );
    let signed = set_jsonc_margin(text, "bottom", 0, true);
    assert!(signed.contains("\"margin-bottom\": 0,"));
    assert!(set_jsonc_margin(text, "top", 16, false).contains("\"margin-top\": 16,"));
    assert!(set_jsonc_margin(text, "top", 16, false).contains("// margin-top: 9"));
  }

  #[test]
  fn targets_replace_only_their_lines() {
    let rofi = "@import \"x\"\n@theme \"old/theme.rasi\"\nfont: \"Old 9\";\n";
    assert_eq!(
      set_target(rofi, Target::RofiTheme, "/new/theme.rasi"),
      "@import \"x\"\n@theme \"/new/theme.rasi\"\nfont: \"Old 9\";\n"
    );
    assert_eq!(
      set_target(rofi, Target::RofiFont, "Inter 11"),
      "@import \"x\"\n@theme \"old/theme.rasi\"\nfont: \"Inter 11\";\n"
    );
    let hypr = "wallpaper {\n  monitor = DP-1\n  path = /old.jxl\n}\n";
    assert_eq!(
      set_target(hypr, Target::HyprpaperPath, "~/new.jxl"),
      "wallpaper {\n  monitor = DP-1\n  path =  ~/new.jxl\n}\n"
    );
  }

  #[test]
  fn waybar_and_native_targets_keep_the_rest_of_the_line_shape() {
    assert_eq!(
      set_target("name = old.ini\n", Target::NativeName, "argvus-dark.ini"),
      "name = argvus-dark.ini\n"
    );
    assert_eq!(
      set_target(
        "@import url(\"/usr/share/argvus/appearance/config/waybar/mode.css\");\n",
        Target::WaybarModeImport,
        "./mode.css"
      ),
      "@import url(\"./mode.css\");\n"
    );
    assert_eq!(
      set_target(
        "@import url(\"./themes/old/widget-telemetry-theme.css\");\n@import url(\"./themes/old/theme.css\");\n",
        Target::WaybarTelemetryThemeImport,
        "github-light"
      ),
      "@import url(\"./themes/github-light/widget-telemetry-theme.css\");\n@import url(\"./themes/old/theme.css\");\n"
    );
    assert_eq!(
      set_target(
        "\"exec\": \"rofi -config /old/config.rasi -show drun\"\n",
        Target::WaybarRofiDrun,
        "/new/config.rasi"
      ),
      "\"exec\": \"rofi -config /new/config.rasi -show drun\"\n"
    );
  }

  #[test]
  fn qt6ct_target_does_not_add_a_missing_key() {
    assert_eq!(
      set_target("[Appearance]\n", Target::Qt6ctCustomPalette, "true"),
      "[Appearance]\n"
    );
  }

  #[test]
  fn replacement_text_is_literal() {
    // A `&` or `$` in a value is written as-is, never expanded.
    assert_eq!(
      set_setting("k = 1\n", "k", "a&b$1", Layout::Spaced),
      "k = a&b$1\n"
    );
  }

  #[test]
  fn jsonc_height_is_replaced_and_read_back() {
    let text = "{\n  \"margin-top\": 1,\n  \"height\": 28,\n  // keep me\n}\n";
    let edited = set_jsonc_height(text, 36);
    assert!(edited.contains("  \"height\": 36,\n  // keep me\n"));
    assert_eq!(jsonc_height(&edited), Some(36));
    assert_eq!(jsonc_height("{}"), None);
  }
}
