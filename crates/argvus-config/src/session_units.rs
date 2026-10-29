//! Session components and the systemd user units that serve them.
//!
//! `argvus-sessionctl` used to keep this table in a shell `case`. Here the
//! table is data: each component name maps to the units that restart it, and
//! the waybar telemetry state decides whether the optional telemetry unit is
//! part of the taskbar restart.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::ConfigStore;

const TASKBAR_UNIT: &str = "argvus-taskbar.service";
const TELEMETRY_UNIT: &str = "argvus-widget-telemetry.service";

/// A component name that no unit serves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownComponent(pub String);

/// Resolves component names to units, in the order they were given.
///
/// Every name is checked before any unit is returned, so one unknown name
/// means nothing is restarted. `telemetry_enabled` is consulted only when the
/// `waybar` component is present.
///
/// # Errors
///
/// Returns the first unknown component name.
pub fn units_for(
  components: &[&str],
  telemetry_enabled: impl Fn() -> bool,
) -> Result<Vec<&'static str>, UnknownComponent> {
  let mut units = Vec::new();
  for component in components {
    match *component {
      "waybar" | "waybars" => {
        units.push(TASKBAR_UNIT);
        if telemetry_enabled() {
          units.push(TELEMETRY_UNIT);
        }
      }
      "waybar-taskbar" | "taskbar" => units.push(TASKBAR_UNIT),
      "widget-telemetry" | "desktop-telemetry" | "telemetry" | "waybar-sysinfo" | "sysinfo" => {
        units.push(TELEMETRY_UNIT)
      }
      "wallpaper" => units.push("argvus-wallpaper.service"),
      "hypridle" | "idle" => units.push("argvus-hypridle.service"),
      "dunst" | "notifications" => units.push("argvus-dunst.service"),
      "control-panel" | "shell" | "quickshell" => {
        units.push("argvus-control-panel.service");
      }
      "clipboard" => {
        units.push("argvus-clipboard-text.service");
        units.push("argvus-clipboard-image.service");
      }
      "keyboard" | "keyboard-layout" => units.push("argvus-keyboard-layout.service"),
      "bluetooth" | "blueman" | "blueman-applet" => {
        units.push("argvus-blueman-applet.service");
      }
      "polkit" | "policykit" => units.push("argvus-polkit.service"),
      "secret-service" | "keyring" | "gnome-keyring" => {
        units.push("argvus-secret-service.service");
      }
      unknown => return Err(UnknownComponent(unknown.to_string())),
    }
  }
  Ok(units)
}

/// Whether the waybar widget telemetry is enabled.
///
/// The current state file wins. Without it, the legacy desktop state file is
/// read, then the legacy sysinfo state file. Only a first line of `enabled`
/// counts as enabled. When no state file exists, `configured_default` decides:
/// a new account has no state file, and the systemd unit must follow the same
/// master switch that the widget toggle reads.
pub fn telemetry_enabled(cache_home: &Path, configured_default: impl FnOnce() -> bool) -> bool {
  let waybar = cache_home.join("waybar");
  let state_file = [
    waybar.join("widget-telemetry-state"),
    waybar.join("desktop-telemetry-state"),
    waybar.join("sysinfo-state"),
  ]
  .into_iter()
  .find(|path| path.is_file());
  match state_file {
    Some(path) => first_line(&path).as_deref() == Some("enabled"),
    None => configured_default(),
  }
}

/// The master switch `effects.widget_telemetry_enabled` from the stored
/// configuration. The packaged default is on, so an unreadable configuration
/// keeps the widget on.
pub fn configured_telemetry_default() -> bool {
  ConfigStore::from_environment()
    .and_then(|store| store.load_effective())
    .ok()
    .and_then(|document| {
      document
        .get("/effects/widget_telemetry_enabled")
        .and_then(Value::as_bool)
    })
    .unwrap_or(true)
}

/// Reads the first line of a state file, or `None` when it cannot be read.
fn first_line(path: &PathBuf) -> Option<String> {
  fs::read_to_string(path)
    .ok()
    .map(|text| text.lines().next().unwrap_or("").to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;

  fn unique_cache(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!("argvus-units-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("waybar")).expect("cache directory");
    root
  }

  #[test]
  fn waybar_adds_telemetry_only_when_enabled() {
    assert_eq!(
      units_for(&["waybar"], || true).expect("known component"),
      vec![TASKBAR_UNIT, TELEMETRY_UNIT]
    );
    assert_eq!(
      units_for(&["waybar"], || false).expect("known component"),
      vec![TASKBAR_UNIT]
    );
  }

  #[test]
  fn telemetry_is_not_consulted_without_waybar() {
    let units =
      units_for(&["wallpaper"], || panic!("must not be called")).expect("known component");
    assert_eq!(units, vec!["argvus-wallpaper.service"]);
  }

  #[test]
  fn aliases_resolve_to_the_same_units() {
    assert_eq!(
      units_for(&["taskbar"], || false).expect("alias"),
      units_for(&["waybar-taskbar"], || false).expect("alias")
    );
    assert_eq!(
      units_for(&["clipboard"], || false).expect("clipboard"),
      vec![
        "argvus-clipboard-text.service",
        "argvus-clipboard-image.service"
      ]
    );
  }

  #[test]
  fn one_unknown_component_rejects_the_whole_request() {
    let error = units_for(&["wallpaper", "nope"], || false).expect_err("unknown component");
    assert_eq!(error, UnknownComponent("nope".into()));
  }

  #[test]
  fn telemetry_state_follows_the_legacy_precedence() {
    let cache = unique_cache("precedence");
    let waybar = cache.join("waybar");

    fs::write(waybar.join("sysinfo-state"), "enabled\n").expect("legacy state");
    assert!(telemetry_enabled(&cache, || false));

    fs::write(waybar.join("desktop-telemetry-state"), "disabled\n").expect("desktop state");
    assert!(
      !telemetry_enabled(&cache, || true),
      "desktop legacy overrides sysinfo"
    );

    fs::write(waybar.join("widget-telemetry-state"), "enabled\n").expect("current state");
    assert!(
      telemetry_enabled(&cache, || false),
      "current state overrides legacy"
    );
    let _ = fs::remove_dir_all(cache);
  }

  #[test]
  fn configured_default_applies_only_without_a_state_file() {
    let cache = unique_cache("default");
    let waybar = cache.join("waybar");

    assert!(
      telemetry_enabled(&cache, || true),
      "new account follows the default on"
    );
    assert!(!telemetry_enabled(&cache, || false), "and the default off");

    fs::write(waybar.join("widget-telemetry-state"), "disabled\n").expect("current state");
    assert!(
      !telemetry_enabled(&cache, || true),
      "an explicit disabled state beats the default"
    );
    let _ = fs::remove_dir_all(cache);
  }

  #[test]
  fn state_file_skips_reading_the_configured_default() {
    let cache = unique_cache("skip-default");
    fs::write(cache.join("waybar/widget-telemetry-state"), "enabled\n").expect("current state");
    assert!(telemetry_enabled(&cache, || panic!("must not be read")));
    let _ = fs::remove_dir_all(cache);
  }
}
