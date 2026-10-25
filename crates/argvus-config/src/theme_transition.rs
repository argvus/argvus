//! Service and overlay steps of a theme switch.
//!
//! A theme switch rewrites the files of several session components, so it
//! stops those services first and restarts the ones that were running. It also
//! shows the loading overlay with the colours of the theme. These steps do not
//! depend on the theme, so they live here. The transaction itself (the lock and
//! the order of the writes) lives in `theme_apply` and `theme_body`.

use crate::session_paths::SessionPaths;
use crate::session_splash::{
  self, Owner, SplashColors, SplashEnvironment, SplashRequest, StartOutcome,
};
use std::fs;
use std::process::{Command, Stdio};

/// The services a theme transition stops and restarts. Restart requests are
/// checked against this table before any `systemctl` call.
pub const TRANSITION_UNITS: [&str; 5] = [
  "argvus-taskbar.service",
  "argvus-widget-telemetry.service",
  "argvus-control-panel.service",
  "argvus-dunst.service",
  "argvus-wallpaper.service",
];

/// Colours used when the theme does not define one.
const FALLBACK_FOREGROUND: &str = "#f4f4f4";
const FALLBACK_BACKGROUND: &str = "#101218";
const FALLBACK_ACCENT: &str = "#7aa2f7";

/// A restart request named a unit outside `TRANSITION_UNITS`.
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownUnit(pub String);

/// The transition units that are active now, in table order.
pub fn active_units() -> Vec<&'static str> {
  TRANSITION_UNITS
    .iter()
    .copied()
    .filter(|unit| systemctl_user(&["is-active", "--quiet", unit]))
    .collect()
}

/// Stops every transition unit. A unit that is not running is not an error.
pub fn stop_units() {
  for unit in TRANSITION_UNITS {
    systemctl_user(&["stop", unit]);
  }
}

/// Starts the units named in `list`, separated by blanks, as the script
/// recorded them. Every name must be in `TRANSITION_UNITS`; a failed start is
/// ignored, as it always was, so one unit cannot block the others.
pub fn restart_units(list: &str) -> Result<(), UnknownUnit> {
  let requested: Vec<&str> = list.split_whitespace().collect();
  if let Some(unknown) = requested
    .iter()
    .find(|unit| !TRANSITION_UNITS.contains(unit))
  {
    return Err(UnknownUnit((*unknown).to_string()));
  }
  for unit in requested {
    systemctl_user(&["start", unit]);
  }
  Ok(())
}

/// Runs `systemctl --user` with the given arguments. Returns whether it
/// succeeded; output is discarded, as the script discarded it.
fn systemctl_user(arguments: &[&str]) -> bool {
  Command::new("systemctl")
    .arg("--user")
    .args(arguments)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

/// Starts the overlay for `theme` with the theme's colours. The outcome tells
/// the caller whether this call started it.
pub fn start_overlay(theme: &str) -> StartOutcome {
  start_overlay_with(theme, false)
}

/// `start_overlay`, optionally as the opaque spinner a session reload shows.
/// The Control Center uses the spinner while it applies a surface, because the
/// theme overlay's "applying the theme" text does not describe that change.
pub fn start_overlay_with(theme: &str, spinner_only: bool) -> StartOutcome {
  let paths = SessionPaths::from_environment();
  let colors = paths
    .map(|paths| overlay_colors(&paths, theme))
    .unwrap_or_else(default_colors);
  let request = SplashRequest {
    theme,
    colors: Some(colors),
    await_compositor: false,
    spinner_only,
    detached: false,
    owner: Owner::Transition,
  };
  session_splash::start_with(
    &SplashEnvironment::resolve(),
    &crate::session_loading::splash_binary(),
    &request,
  )
}

/// Stops the overlay that a theme switch started. The session splash is not
/// touched, because it records its pid in a different file.
pub fn stop_overlay() {
  session_splash::stop(&SplashEnvironment::resolve(), Owner::Transition);
}

/// The message printed when the overlay could not be shown. `None` when the
/// overlay runs, or when it was already running.
pub fn overlay_warning(outcome: StartOutcome) -> Option<&'static str> {
  match outcome {
    StartOutcome::Started | StartOutcome::AlreadyRunning => None,
    StartOutcome::Unavailable => {
      Some("argvus-appearance: argvus-loading-theme is unavailable; continuing without overlay")
    }
    StartOutcome::NoCompositor | StartOutcome::NotReady => Some(
      "argvus-appearance: argvus-loading-theme did not become ready; continuing without overlay",
    ),
  }
}

fn default_colors() -> SplashColors {
  SplashColors {
    foreground: FALLBACK_FOREGROUND.to_string(),
    background: FALLBACK_BACKGROUND.to_string(),
    accent: FALLBACK_ACCENT.to_string(),
  }
}

/// The colours the theme's `hyprtoolkit.conf` defines for `text`, `background`
/// and `accent`, with the fallbacks for any it does not define.
fn overlay_colors(paths: &SessionPaths, theme: &str) -> SplashColors {
  // The shell read this file with the mutable `paths_config`, which also
  // creates the theme's user directory. Keep that side effect as it was.
  let file = crate::user_copy::config_path(
    paths,
    &format!("appearance/config/hypr/themes/{theme}/hyprtoolkit.conf"),
  )
  .unwrap_or_default();
  let text = fs::read_to_string(file).unwrap_or_default();
  SplashColors {
    foreground: colour_value(&text, "text").unwrap_or_else(|| FALLBACK_FOREGROUND.to_string()),
    background: colour_value(&text, "background")
      .unwrap_or_else(|| FALLBACK_BACKGROUND.to_string()),
    accent: colour_value(&text, "accent").unwrap_or_else(|| FALLBACK_ACCENT.to_string()),
  }
}

/// The first `key = 0xFFrrggbb` line, as `#rrggbb`. The key must be followed by
/// blanks and `=`, so `text` does not match `text_size`.
pub fn colour_value(text: &str, key: &str) -> Option<String> {
  text.lines().find_map(|line| {
    let rest = line.trim_start().strip_prefix(key)?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let rest = rest.strip_prefix("0xFF")?;
    let hex = rest.get(..6)?;
    let trailing = &rest[6..];
    (hex.chars().all(|digit| digit.is_ascii_hexdigit()) && trailing.trim().is_empty())
      .then(|| format!("#{hex}"))
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn colour_is_read_from_the_matching_key_only() {
    let text = "text_size = 0xFF112233\ntext = 0xFFaabbcc\n";
    assert_eq!(colour_value(text, "text").as_deref(), Some("#aabbcc"));
    assert_eq!(colour_value(text, "accent"), None);
  }

  #[test]
  fn colour_needs_exactly_six_hex_digits_after_the_prefix() {
    assert_eq!(
      colour_value("accent = 0xFF1234\n", "accent"),
      None,
      "five digits"
    );
    assert_eq!(
      colour_value("accent = 0xFF12345g\n", "accent"),
      None,
      "not hex"
    );
    assert_eq!(
      colour_value("accent = 0xFF123456  \n", "accent").as_deref(),
      Some("#123456"),
      "trailing blanks are allowed"
    );
    assert_eq!(
      colour_value("accent=0xFF123456\n", "accent").as_deref(),
      Some("#123456")
    );
  }

  #[test]
  fn first_matching_line_wins() {
    let text = "accent = 0xFF000001\naccent = 0xFF000002\n";
    assert_eq!(colour_value(text, "accent").as_deref(), Some("#000001"));
  }

  #[test]
  fn restart_rejects_units_outside_the_table_before_starting_any() {
    assert_eq!(
      restart_units("argvus-taskbar.service evil.service"),
      Err(UnknownUnit("evil.service".to_string()))
    );
  }

  #[test]
  fn warnings_match_the_outcome() {
    assert_eq!(overlay_warning(StartOutcome::Started), None);
    assert_eq!(overlay_warning(StartOutcome::AlreadyRunning), None);
    assert!(
      overlay_warning(StartOutcome::Unavailable)
        .is_some_and(|message| message.contains("is unavailable"))
    );
    assert!(
      overlay_warning(StartOutcome::NotReady)
        .is_some_and(|message| message.contains("did not become ready"))
    );
  }
}
