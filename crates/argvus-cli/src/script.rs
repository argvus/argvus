//! Script routes: Hyprland script commands that run a packaged shell script, or
//! the user's copy of it when one exists.

use crate::commands::ScriptRoute;
use crate::exec::{Outcome, env_nonempty, replace_process};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

/// Packaged scripts are read from here unless `ARGVUS_SYSTEM_CONFIG` overrides it.
const DEFAULT_SYSTEM_CONFIG: &str = "/usr/share/argvus";

/// Replaces this process with `sh <script>`. The interpreter is named by absolute
/// path, so the route does not depend on PATH.
pub fn run(route: ScriptRoute, arguments: &[OsString]) -> Outcome {
  let script = resolve_script_path(route.relative_path());
  Err(replace_process(
    Command::new("/bin/sh").arg(script).args(arguments),
  ))
}

/// Chooses the script to run. The precedence matches `_config_path` in
/// `hyprland.lua`: the user's config copy, then the ARGVUS data copy, then the
/// state copy, then the generated copy, and only then the packaged file. Keep
/// both lists in sync until the Lua configuration stops resolving scripts.
fn resolve_script_path(relative: &str) -> PathBuf {
  // Older layouts stored scripts flat under `scripts/argvus/`, so both names are
  // checked at every level.
  let legacy_relative = format!("scripts/argvus/{}", basename(relative));
  let config_home = configuration_home();
  let state_home = config_home.join("argvus");
  let data_home = state_home.join("data");
  let generated_home = data_home.join("generated");

  let candidates = [config_home, data_home, state_home, generated_home]
    .into_iter()
    .flat_map(|base| [base.join(&legacy_relative), base.join(relative)]);
  for candidate in candidates {
    // `symlink_metadata` also accepts a dangling link, as `-e || -L` does.
    if candidate.symlink_metadata().is_ok() {
      return candidate;
    }
  }
  system_config().join(relative)
}

/// The root of the user configuration: `ARGVUS_CONFIG_HOME`, then
/// `XDG_CONFIG_HOME`, then `$HOME/.config`.
fn configuration_home() -> PathBuf {
  if let Some(home) = env_nonempty("ARGVUS_CONFIG_HOME").or_else(|| env_nonempty("XDG_CONFIG_HOME"))
  {
    return PathBuf::from(home);
  }
  // An empty HOME yields `/.config`, as the shell expansion `${HOME:-}/.config` does.
  let mut default_home = env_nonempty("HOME").unwrap_or_default();
  default_home.push("/.config");
  PathBuf::from(default_home)
}

fn system_config() -> PathBuf {
  env_nonempty("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(DEFAULT_SYSTEM_CONFIG))
}

fn basename(relative: &str) -> &str {
  relative.rsplit('/').next().unwrap_or(relative)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn basename_keeps_the_last_component() {
    assert_eq!(basename("hyprland/sh/scratchpad.sh"), "scratchpad.sh");
  }
}
