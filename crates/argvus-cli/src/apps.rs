//! `spf` and `yazi`: the file manager and the file browser. Both start from the
//! bootstrap environment and choose their configuration with the ARGVUS path
//! contract, as the shell routes did.

use crate::copy::copy_tree;
use crate::exec::{Failure, Outcome, env_nonempty, io_failure, replace_process};
use crate::paths::{Layout, exists, first_line};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SUPERFILE_CONFIG: &str = "app-profiles/config/superfile";
const YAZI_CONFIG: &str = "app-profiles/config/yazi";
const YAZI_PACKAGE_FILES: [&str; 2] = ["package.toml", "keymap.toml"];
const ACTIVE_THEME_FILE: &str = "argvus/data/.active-theme";
const DEFAULT_THEME: &str = "argvus-dark";
const ARGVUS_FLAVOR_PREFIX: &str = "argvus-";
const PACKAGED_YAZI_CONFIG: &str = "/usr/share/argvus/yazi";

/// Runs `spf`. Configuration flags are passed through untouched, so the user's
/// own file selection wins over the ARGVUS directory.
pub fn run_spf(arguments: &[OsString]) -> Outcome {
  let layout = Layout::from_environment()?;
  let mut command = bootstrap_command("spf", &layout);
  if has_config_flag(arguments) {
    return Err(replace_process(command.args(arguments)));
  }
  // A native `superfile` directory under the config home is used in place.
  // Otherwise the directory chosen by the ARGVUS path contract is used, and its
  // `/superfile` suffix is removed, because `spf` appends it itself.
  if exists(&layout.config_home().join("superfile")) {
    command.env("XDG_CONFIG_HOME", layout.config_home());
  } else {
    let superfile_directory = layout
      .config(SUPERFILE_CONFIG)
      .to_string_lossy()
      .into_owned();
    let config_home = superfile_directory
      .strip_suffix("/superfile")
      .unwrap_or(&superfile_directory);
    command.env("XDG_CONFIG_HOME", config_home);
  }
  Err(replace_process(command.args(arguments)))
}

/// Runs `yazi`. A requested configuration directory is honored unless it is one of
/// the ARGVUS defaults, which are resolved below.
pub fn run_yazi(arguments: &[OsString]) -> Outcome {
  let layout = Layout::from_environment()?;
  let mut command = bootstrap_command("yazi", &layout);
  if let Some(requested) = env_nonempty("YAZI_CONFIG_HOME") {
    let is_argvus_default = is_argvus_default_yazi(&requested, &layout);
    if Path::new(&requested).is_dir() && !is_argvus_default {
      return Err(replace_process(command.args(arguments)));
    }
  }
  // A native `yazi` directory is used unless its theme is an ARGVUS flavor that
  // the native directory does not ship. That case gets the materialized copy.
  let native_dir = layout.config_home().join("yazi");
  if exists(&native_dir) {
    let native_flavor = read_dark_flavor(&native_dir.join("theme.toml")).unwrap_or_default();
    let flavor_file = native_dir
      .join("flavors")
      .join(format!("{native_flavor}.yazi"))
      .join("flavor.toml");
    let needs_materialized =
      native_flavor.starts_with(ARGVUS_FLAVOR_PREFIX) && !flavor_file.is_file();
    if !needs_materialized {
      command.env("YAZI_CONFIG_HOME", &native_dir);
      return Err(replace_process(command.args(arguments)));
    }
  }
  let yazi_config = materialize_yazi_config(&layout)?;
  command.env("YAZI_CONFIG_HOME", yazi_config);
  Err(replace_process(command.args(arguments)))
}

/// A command for a configured program, with the bootstrap exports applied.
fn bootstrap_command(program: &str, layout: &Layout) -> Command {
  let mut command = Command::new(program);
  if let Some(config_dirs) = layout.xdg_config_dirs_override() {
    command.env("XDG_CONFIG_DIRS", config_dirs);
  }
  command
}

fn has_config_flag(arguments: &[OsString]) -> bool {
  arguments
    .iter()
    .filter_map(|argument| argument.to_str())
    .any(|argument| {
      matches!(argument, "-c" | "--config-file" | "--hf" | "--hotkey-file")
        || argument.starts_with("--config-file=")
        || argument.starts_with("--hotkey-file=")
    })
}

fn is_argvus_default_yazi(requested: &OsString, layout: &Layout) -> bool {
  let requested = requested.to_string_lossy();
  let managed = layout.config_home().join("argvus/data/yazi");
  requested == managed.to_string_lossy() || requested == PACKAGED_YAZI_CONFIG
}

/// Reads the `dark` flavor from a yazi `theme.toml`. The first line that matches
/// `dark = "<flavor>"` wins, as the shell `sed | head -n1` pipeline did.
fn read_dark_flavor(theme_file: &Path) -> Option<String> {
  let content = fs::read_to_string(theme_file).ok()?;
  parse_dark_flavor(&content)
}

/// The flavor named by the first `dark = "<flavor>"` line, if any.
fn parse_dark_flavor(content: &str) -> Option<String> {
  content.lines().find_map(|line| {
    let rest = line
      .trim_start()
      .strip_prefix("dark")?
      .trim_start()
      .strip_prefix('=')?
      .trim_start()
      .strip_prefix('"')?;
    rest.find('"').map(|end| rest[..end].to_owned())
  })
}

/// Creates the managed yazi configuration from the packaged one and returns the
/// directory to use. The active theme's flavor is copied and selected in
/// `theme.toml`. Without that flavor the resolved configuration is used instead.
fn materialize_yazi_config(layout: &Layout) -> Result<PathBuf, Failure> {
  let theme = read_active_theme(layout);
  let destination = layout.user_config(YAZI_CONFIG);
  let source = layout.system_config(YAZI_CONFIG);

  if !destination.is_dir() && source.is_dir() {
    copy_tree(&source, &destination).map_err(|error| io_failure(&destination, error))?;
  } else if source.is_dir() {
    fs::create_dir_all(&destination).map_err(|error| io_failure(&destination, error))?;
    // Package files are created only when missing, so user edits survive updates.
    for item in YAZI_PACKAGE_FILES {
      let item_source = source.join(item);
      let item_destination = destination.join(item);
      if item_source.is_file() && !exists(&item_destination) {
        fs::copy(&item_source, &item_destination)
          .map_err(|error| io_failure(&item_destination, error))?;
      }
    }
    let source_flavors = source.join("flavors");
    if source_flavors.is_dir() {
      let destination_flavors = destination.join("flavors");
      fs::create_dir_all(&destination_flavors)
        .map_err(|error| io_failure(&destination_flavors, error))?;
      for flavor_source in flavor_directories(&source_flavors)? {
        let Some(flavor_name) = flavor_source.file_name() else {
          continue;
        };
        let flavor_destination = destination_flavors.join(flavor_name);
        if !exists(&flavor_destination) {
          copy_tree(&flavor_source, &flavor_destination)
            .map_err(|error| io_failure(&flavor_destination, error))?;
        }
      }
    }
    // The active theme's flavor always replaces the copy, so theme switches apply.
    let theme_source = source_flavors.join(format!("{theme}.yazi"));
    if theme_source.is_dir() {
      let theme_destination = destination.join("flavors").join(format!("{theme}.yazi"));
      copy_tree(&theme_source, &theme_destination)
        .map_err(|error| io_failure(&theme_destination, error))?;
    }
  }

  let theme_flavor_file = destination
    .join("flavors")
    .join(format!("{theme}.yazi"))
    .join("flavor.toml");
  if theme_flavor_file.is_file() {
    let theme_file = destination.join("theme.toml");
    fs::write(&theme_file, format!("[flavor]\ndark = \"{theme}\"\n"))
      .map_err(|error| io_failure(&theme_file, error))?;
    Ok(destination)
  } else {
    Ok(resolve_yazi_config(layout))
  }
}

/// The yazi directories to search, in order: the user copy, the generated copy
/// and finally the packaged configuration.
fn resolve_yazi_config(layout: &Layout) -> PathBuf {
  [
    layout.user_config(YAZI_CONFIG),
    layout.generated_config(YAZI_CONFIG),
  ]
  .into_iter()
  .find(|candidate| exists(candidate))
  .unwrap_or_else(|| layout.system_config(YAZI_CONFIG))
}

/// The active theme name from the ARGVUS data root. It is the first line of
/// `.active-theme`, and `argvus-dark` when the file is absent.
fn read_active_theme(layout: &Layout) -> String {
  first_line(&layout.config_home().join(ACTIVE_THEME_FILE))
    .unwrap_or_else(|| DEFAULT_THEME.to_owned())
}

/// The `*.yazi` flavor directories of `directory`, sorted by name. Hidden entries
/// are skipped, as the shell glob skips them.
fn flavor_directories(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
  let entries = fs::read_dir(directory).map_err(|error| io_failure(directory, error))?;
  let mut flavors: Vec<PathBuf> = entries
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .filter(|path| {
      let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
      name.is_some_and(|name| !name.starts_with('.') && name.ends_with(".yazi")) && path.is_dir()
    })
    .collect();
  flavors.sort();
  Ok(flavors)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_the_dark_flavor_with_loose_spacing() {
    let content = "[flavor]\n  dark   =   \"argvus-dark\"\n";
    assert_eq!(parse_dark_flavor(content).as_deref(), Some("argvus-dark"));
  }

  #[test]
  fn ignores_keys_that_only_start_with_dark() {
    let content = "darkness = \"not-a-flavor\"\n";
    assert_eq!(parse_dark_flavor(content), None);
  }

  #[test]
  fn detects_the_spf_config_flags() {
    let arguments = [OsString::from("--config-file=/tmp/x.toml")];
    assert!(has_config_flag(&arguments));
    let plain = [OsString::from("--help")];
    assert!(!has_config_flag(&plain));
  }
}
