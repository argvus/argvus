//! The highlight color of the desktop.
//!
//! The color is rendered into the theme build with the theme, so every themed
//! file carries it. This module stores the choice, publishes it for the
//! greeter, points qt6ct at the build and, for an accent change on its own,
//! rewrites the files copied from the build and reloads the running
//! consumers. The callers reach it through `argvus config accent` (see
//! `accent_switch`), which adds the picker, the icon theme and the messages.

use crate::session_paths::SessionPaths;
use crate::text_edit;
use crate::theme_files::{self, Fonts};
use crate::user_copy;
use argvus_theme_core::paths::ThemePaths;
use regex::Regex;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const DEFAULT_ACCENT: &str = "#798186";
pub const DEFAULT_THEME: &str = "argvus-dark";

/// A validated color, with the text color that reads on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accent {
  /// `#RRGGBB`, uppercase.
  pub color: String,
  /// Black on light colors, white on dark ones.
  pub text: &'static str,
}

/// Validates a color: an optional `#` and six hex digits, in either case.
pub fn normalize(requested: &str) -> Option<Accent> {
  let upper = requested.to_ascii_uppercase();
  let digits = upper.strip_prefix('#').unwrap_or(&upper);
  if digits.len() != 6
    || !digits
      .bytes()
      .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
  {
    return None;
  }
  let red = u8::from_str_radix(&digits[0..2], 16).ok()?;
  let green = u8::from_str_radix(&digits[2..4], 16).ok()?;
  let blue = u8::from_str_radix(&digits[4..6], 16).ok()?;
  let luminance = (299 * u32::from(red) + 587 * u32::from(green) + 114 * u32::from(blue)) / 1000;
  Some(Accent {
    color: format!("#{digits}"),
    text: if luminance >= 128 {
      "#000000"
    } else {
      "#FFFFFF"
    },
  })
}

/// The theme ids the switch keeps as the Catppuccin Latte family.
pub fn canonical_theme_id(theme: &str) -> &str {
  // One alias table for the whole crate: the theme resolver owns it.
  crate::theme_resolve::canonical_theme_id(theme)
}

/// The accent a theme owns when the user has not chosen one: `palette.accent`
/// of its manifest.
pub fn theme_default_accent(paths: &ThemePaths, theme: &str) -> Option<String> {
  crate::theme_resolve::theme_accent(paths, theme)
}

/// The first line of a state file, trimmed, or `fallback` when the file is
/// missing or empty.
pub fn read_state(path: &Path, fallback: &str) -> String {
  match fs::read_to_string(path) {
    Ok(text) if !text.is_empty() => text
      .split('\n')
      .next()
      .unwrap_or("")
      .trim_end_matches('\r')
      .trim()
      .to_string(),
    _ => fallback.to_string(),
  }
}

/// The canonical highlight color, when the user owns it. The canonical document
/// is read through the same store as `argvus config get`.
pub fn canonical_accent(config_store: Option<&crate::ConfigStore>) -> Option<String> {
  let store = config_store?;
  let document = store.load().ok()?;
  let owned = match document.get("/appearance/accent_custom") {
    Some(serde_json::Value::Bool(value)) => *value,
    Some(serde_json::Value::String(value)) => value == "true",
    _ => false,
  };
  if !owned {
    return None;
  }
  Some(match document.get("/appearance/accent") {
    Some(serde_json::Value::String(value)) => value.trim().to_string(),
    Some(other) => other.to_string(),
    None => "null".to_string(),
  })
}

/// The accent the state files say should be applied: the saved one, unless the
/// canonical document owns a color, or the saved one is not a color.
pub fn state_accent(paths: &ThemePaths, state_dir: &Path, canonical: Option<String>) -> String {
  let state_theme = read_state(&state_dir.join(".active-theme"), DEFAULT_THEME);
  let state_default =
    theme_default_accent(paths, &state_theme).unwrap_or_else(|| DEFAULT_ACCENT.to_string());
  let mut accent = read_state(&state_dir.join(".accent-color"), &state_default);
  if let Some(canonical) = canonical.filter(|value| !value.is_empty()) {
    accent = canonical;
  }
  let accent = match accent.as_str() {
    "blue" => "#3590BD".to_string(),
    "slate-blue" => "#7391A5".to_string(),
    "brown" => "#996548".to_string(),
    "green" => "#17D174".to_string(),
    "magenta" => "#CB17D1".to_string(),
    "red" => "#D1174F".to_string(),
    "yellow" => "#D1CE17".to_string(),
    "purple" => "#9617D1".to_string(),
    "silver" => "#595959".to_string(),
    _ => accent,
  };
  let is_color = (accent.len() == 7 && accent.starts_with('#')) || accent.chars().count() == 6;
  if is_color { accent } else { state_default }
}

/// The theme a session or a projection applies for the configured one: the
/// theme itself when it is installed, else the ARGVUS Dark or Light theme of its
/// mode. A removed theme must never stop the desktop from starting or from
/// projecting the configuration.
pub fn effective_theme(configured: &str) -> String {
  applied_theme(&crate::theme_resolve::theme_paths(), configured)
}

/// The theme id the accent applies to: the active theme when it is installed,
/// else the ARGVUS theme of its mode.
pub fn applied_theme(paths: &ThemePaths, active_theme: &str) -> String {
  let theme = canonical_theme_id(active_theme).to_string();
  if crate::theme_resolve::manifest(paths, &theme).is_some() {
    return theme;
  }
  if theme.ends_with("-dark-float") {
    "argvus-dark-float".to_string()
  } else if theme.ends_with("-light-float") {
    "argvus-light-float".to_string()
  } else if theme.ends_with("-light") {
    "argvus-light".to_string()
  } else {
    DEFAULT_THEME.to_string()
  }
}

/// One line edit, the Rust form of a `sed -e 's|…|…|'` expression: the first
/// match on a line is replaced. `$1`-style references are kept only where the
/// template asks for them.
pub struct Rule {
  pattern: Regex,
  template: String,
  literal: bool,
}

impl Rule {
  pub fn literal(pattern: &str, replacement: String) -> Self {
    Self {
      pattern: Regex::new(pattern).expect("accent rule pattern is valid"),
      template: replacement,
      literal: true,
    }
  }

  /// A rule whose template refers to capture groups, as `\1` did in `sed`.
  pub fn groups(pattern: &str, template: String) -> Self {
    Self {
      pattern: Regex::new(pattern).expect("accent rule pattern is valid"),
      template,
      literal: false,
    }
  }

  fn apply(&self, line: &str) -> String {
    if self.literal {
      self
        .pattern
        .replace(line, regex::NoExpand(&self.template))
        .into_owned()
    } else {
      self
        .pattern
        .replace(line, self.template.as_str())
        .into_owned()
    }
  }
}

/// Applies the rules to each line in order, keeping line endings.
pub fn apply_rules(text: &str, rules: &[Rule]) -> String {
  text
    .split_inclusive('\n')
    .map(|line| {
      let (content, ending) = match line.strip_suffix('\n') {
        Some(content) => (content, "\n"),
        None => (line, ""),
      };
      let edited = rules
        .iter()
        .fold(content.to_string(), |current, rule| rule.apply(&current));
      format!("{edited}{ending}")
    })
    .collect()
}

/// Edits a file in place when it exists. A missing file is not an error: the
/// shell skipped it the same way.
pub fn edit_file(path: Option<&Path>, edit: impl Fn(&str) -> String) -> io::Result<()> {
  let Some(path) = path.filter(|path| path.is_file()) else {
    return Ok(());
  };
  let text = fs::read_to_string(path)?;
  let edited = edit(&text);
  if edited != text {
    text_edit::replace_file(path, &edited)?;
  }
  Ok(())
}

/// `replace_setting`: `NAME = VALUE`, replacing the first line that sets NAME.
fn replace_setting(path: Option<&Path>, name: &str, value: &str) -> io::Result<()> {
  let pattern = format!(r"^{}[[:space:]]*=.*", regex::escape(name));
  let rules = [Rule::literal(&pattern, format!("{name} = {value}"))];
  edit_file(path, |text| apply_rules(text, &rules))
}

/// The `[flavor]` table of the yazi theme file that selects the theme.
pub fn yazi_theme_file(theme: &str) -> String {
  format!("[flavor]\ndark = \"{theme}\"\n")
}

/// Everything the apply step needs from its caller.
pub struct Context<'a> {
  pub paths: &'a SessionPaths,
  pub theme: &'a str,
  /// Effective GTK and Qt icon theme, from the icon helpers.
  pub icon_theme: &'a str,
  pub icon_default_dark: &'a str,
  pub icon_default_light: &'a str,
  pub runtime: bool,
  /// A theme switch writes the build copies and reloads the consumers itself.
  pub theme_switch: bool,
  pub xdg_config_home: &'a Path,
}

/// Applies the accent of the current build. Each step is independent: a
/// failure is reported on stderr and the next step still runs.
pub fn apply(context: &Context) {
  let qt6ct = user_copy::config_path(context.paths, "appearance/config/qt6ct/qt6ct.conf");
  let colors = qt6ct_colors(context);
  report(replace_setting(
    qt6ct.as_deref(),
    "color_scheme_path",
    &colors.display().to_string(),
  ));
  report(sync_native_qt6ct(context, qt6ct.as_deref()));
  if !context.theme_switch {
    let fonts = Fonts::read(context.paths);
    if let Err(key) =
      theme_files::write_build_copies(context.paths, context.theme, &fonts, context.icon_theme)
    {
      eprintln!("argvus-config: accent: theme.{key}");
    }
  }
  if context.runtime && !context.theme_switch {
    crate::theme_runtime::reload_consumers(context.paths, None);
  }
}

fn report(result: io::Result<()>) {
  if let Err(error) = result {
    eprintln!("argvus-config: accent: {error}");
  }
}

/// The qt6ct colour scheme of the build, which carries the accent.
fn qt6ct_colors(context: &Context) -> PathBuf {
  crate::theme_build::theme_paths(context.paths)
    .current()
    .join("qt6ct/colors.conf")
}

/// Keeps the user's own qt6ct file in step. The file is taken over only when it
/// is absent or was generated by ARGVUS; any other icon theme is a user choice.
fn sync_native_qt6ct(context: &Context, managed: Option<&Path>) -> io::Result<()> {
  let Some(managed) = managed else {
    return Ok(());
  };
  let native = context.xdg_config_home.join("qt6ct/qt6ct.conf");
  if native == managed {
    return Ok(());
  }
  if !native.exists() {
    if let Some(parent) = native.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::copy(managed, &native)?;
    return Ok(());
  }
  let text = fs::read_to_string(&native)?;
  let lines: Vec<&str> = text.lines().collect();
  let owns_icon = lines.contains(&format!("icon_theme={}", context.icon_default_dark).as_str())
    || lines.contains(&format!("icon_theme={}", context.icon_default_light).as_str())
    || lines.contains(&format!("icon_theme={}", context.icon_theme).as_str());
  let owns_scheme = lines.iter().any(|line| {
    line
      .strip_prefix("color_scheme_path")
      .map(|rest| {
        rest.trim_start().starts_with('=')
          && (rest.contains("argvus/colors/argvus-")
            || rest.contains("appearance/config/qt6ct/colors/")
            || rest.contains("generated/theme/current/qt6ct/"))
      })
      .unwrap_or(false)
  });
  if !(owns_icon || owns_scheme) {
    return Ok(());
  }
  let colors = qt6ct_colors(context);
  replace_setting(
    Some(&native),
    "color_scheme_path",
    &colors.display().to_string(),
  )?;
  replace_setting(Some(&native), "custom_palette", "true")?;
  let icon = context.icon_theme.to_string();
  let rules = [Rule::literal(
    r"^[[:space:]]*icon_theme[[:space:]]*=.*",
    format!("icon_theme={icon}"),
  )];
  edit_file(Some(&native), |text| apply_rules(text, &rules))
}

/// Writes the accent for the greeter, in the file named by the user id.
pub fn publish_greeter_accent(greeter_state_dir: &Path, accent: &Accent) -> io::Result<()> {
  if !greeter_state_dir.is_dir() {
    return Ok(());
  }
  let user_id = crate::theme_surfaces::current_user_id();
  let target = greeter_state_dir.join(format!("{user_id}.accent"));
  let temporary = greeter_state_dir.join(format!("{user_id}.accent.tmp.{}", std::process::id()));
  let written = fs::write(&temporary, format!("{}\n", accent.color)).and_then(|()| {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(&temporary, fs::Permissions::from_mode(0o644));
    fs::rename(&temporary, &target)
  });
  if written.is_err() {
    let _ = fs::remove_file(&temporary);
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn colors_are_normalized_and_the_text_color_contrasts() {
    let accent = normalize("#3590bd").expect("valid");
    assert_eq!(accent.color, "#3590BD");
    assert_eq!(accent.text, "#FFFFFF");
    assert_eq!(normalize("ffffff").expect("valid").text, "#000000");
    assert!(normalize("#3590B").is_none(), "five digits");
    assert!(normalize("#3590BG").is_none(), "not hex");
    assert!(normalize("##3590BD").is_none(), "two hashes");
  }

  #[test]
  fn uninstalled_theme_ids_map_to_the_argvus_theme_of_their_mode() {
    let themes = ThemePaths {
      system_root: PathBuf::from("/nonexistent"),
      user_themes: PathBuf::from("/nonexistent"),
      generated: PathBuf::from("/nonexistent"),
    };
    let themes = &themes;
    assert_eq!(
      applied_theme(themes, "custom-dark-float"),
      "argvus-dark-float"
    );
    assert_eq!(applied_theme(themes, "custom-light"), "argvus-light");
    assert_eq!(applied_theme(themes, "unknown"), DEFAULT_THEME);
  }
}
