//! Reads the Hyprland compositor version of the running session. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_control_center_core::process;

/// Returns the version of the Hyprland compositor answering `hyprctl`, or `na` when no session is reachable.
///
/// `hyprctl` talks to the compositor through `HYPRLAND_INSTANCE_SIGNATURE`, so the value reflects the
/// session that is running the Control Center, not an installed package.
pub fn hyprland_version(na: &str) -> String {
  process::command("hyprctl")
    .arg("version")
    .output()
    .ok()
    .filter(|output| output.status.success())
    .and_then(|output| parse_hyprland_version(&String::from_utf8_lossy(&output.stdout)))
    .unwrap_or_else(|| na.to_string())
}

/// Extracts the version token from the first line of `hyprctl version`, such as `Hyprland 0.56.2 built from ...`.
fn parse_hyprland_version(output: &str) -> Option<String> {
  let first_line = output.lines().next()?.trim();
  let mut words = first_line.split_whitespace();
  (words.next()? == "Hyprland").then(|| words.next().map(str::to_string))?
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Extracts the version from the banner printed by `hyprctl version`.
  fn parses_version_from_hyprctl_banner() {
    let banner = "Hyprland 0.56.2 built from branch v0.56.2 at commit efb5099\nDate: Wed Aug 5\n";
    assert_eq!(parse_hyprland_version(banner), Some("0.56.2".into()));
  }

  #[test]
  /// Rejects output that does not start with the Hyprland banner.
  fn rejects_unrelated_output() {
    assert_eq!(parse_hyprland_version("Couldn't connect to socket\n"), None);
    assert_eq!(parse_hyprland_version(""), None);
  }
}
