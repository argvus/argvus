//! Runtime half of a configuration reload: the file synchronization that keeps
//! Rofi, btop and yazi aligned with the active theme, then the compositor
//! reload.
//!
//! This replaces the reload that the session scripts used to run. The scripts it still
//! depends on (lock, spacing, borders, display) run unchanged, through the
//! same resolved paths the shell used.

use crate::hyprland_ipc;
use crate::session_paths::{SessionPaths, font_default_value};
use crate::session_timing::timed;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Theme used when no active theme is recorded (`ACTIVE_THEME` in the shell).
const DEFAULT_THEME: &str = "argvus-dark";

/// Inputs that vary per run.
pub struct ReloadContext<'a> {
  pub paths: &'a SessionPaths,
  /// Directory of the private ARGVUS commands (`ARGVUS_COMMANDS_DIR`).
  pub commands_dir: &'a Path,
}

/// The active theme. The canonical configuration is the source of truth; the
/// `.active-theme` marker is a projection of it. The marker is used only when the
/// canonical document has no theme, and then the default when it is absent too.
pub fn active_theme(paths: &SessionPaths) -> String {
  if let Some(theme) = canonical_theme(&paths.config_home) {
    return theme;
  }
  let file = paths.config_home.join("argvus/data/.active-theme");
  if file.is_file() {
    return fs::read_to_string(file)
      .ok()
      .and_then(|text| text.lines().next().map(str::to_string))
      .unwrap_or_default();
  }
  DEFAULT_THEME.to_string()
}

/// `/appearance/theme` of the canonical configuration under `config_home`, when set.
fn canonical_theme(config_home: &Path) -> Option<String> {
  let document = crate::ConfigStore::at(config_home).load().ok()?;
  document
    .get("/appearance/theme")
    .and_then(serde_json::Value::as_str)
    .filter(|theme| !theme.is_empty())
    .map(str::to_string)
}

/// Runs the reload and returns the exit status the session script expects.
///
/// The first failing step that the shell treated as fatal stops the run.
/// Synchronization steps never fail the reload, as before.
pub fn reload_config(context: &ReloadContext<'_>) -> i32 {
  let paths = context.paths;
  let lock_script = paths.read_config("lock/sh/hyprlock-theme.sh");
  timed("reload.lock-invalidate", || {
    let _ = Command::new("sh")
      .arg(&lock_script)
      .arg("--invalidate")
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
  });

  let mut environment: Vec<(String, String)> = Vec::new();
  if let Some(config) = timed("reload.user-files", || sync_user_files(context)) {
    environment.push(("YAZI_CONFIG_HOME".into(), config.display().to_string()));
  }

  let data = paths.config_home.join("argvus/data");
  let applies_spacing = data.join(".active-theme").is_file() || data.join(".spaces").is_file();
  if applies_spacing {
    let status = timed("reload.spaces-switch", || {
      run_script(paths, "hyprland/sh/spaces-switch.sh", &environment)
    });
    if status != 0 {
      return status;
    }
  }
  let applies_borders = data.join(".active-theme").is_file() || data.join(".borders").is_file();
  if applies_borders {
    let status = timed("reload.borders-switch", || {
      run_script(paths, "hyprland/sh/borders-switch.sh", &environment)
    });
    if status != 0 {
      return status;
    }
  }

  let compositor_reload = timed("reload.compositor", hyprland_ipc::reload);
  match compositor_reload {
    Ok(Some(warning)) => eprintln!("argvus-config: {warning}"),
    Ok(None) => {}
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return 1;
    }
  }

  timed("reload.display", || apply_display(&environment));
  0
}

/// Login variant of [`reload_config`].
///
/// Session preparation has just synchronized the user files, spacing, borders
/// and displays, and Hyprland has just loaded the generated configuration. The
/// only work left is a compositor reload, and only when the projection that ran
/// at this login changed something Hyprland reads.
pub fn reload_at_login() -> i32 {
  if !crate::project::last_projection_affects_compositor() {
    return 0;
  }
  match timed("reload.compositor", hyprland_ipc::reload) {
    Ok(Some(warning)) => {
      eprintln!("argvus-config: {warning}");
      0
    }
    Ok(None) => 0,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      1
    }
  }
}

/// Brings the Rofi, btop and yazi files in line with the active theme. Used by
/// the reload and by session preparation. Returns the yazi directory that
/// later children must read, when there is one.
pub fn sync_user_files(context: &ReloadContext<'_>) -> Option<PathBuf> {
  let paths = context.paths;
  let _ = sync_rofi(paths);
  let theme = active_theme(paths);
  sync_btop(&theme);
  sync_yazi(paths, &theme)
}

/// Writes the Rofi font into the user's Rofi config, copying the packaged one
/// first when the user has none.
fn sync_rofi(paths: &SessionPaths) -> io::Result<()> {
  let relative = "launcher/config/config.rasi";
  let config = paths.read_config(relative);
  let packaged = paths.system_config(relative);
  if !config.is_file() && packaged.is_file() {
    if let Some(parent) = config.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::copy(&packaged, &config)?;
  }
  if !config.is_file() {
    return Ok(());
  }
  let font_line = format!("    font: \"{}\";", font_default_value(paths));
  let text = fs::read_to_string(&config)?;
  write_atomically(&config, &apply_rofi_font(&text, &font_line))
}

/// Replaces the font line of an existing Rofi config, or inserts one after
/// each `configuration {` line when there is none. Every line is kept.
pub fn apply_rofi_font(text: &str, font_line: &str) -> String {
  let has_font = text
    .lines()
    .any(|line| line.trim_start().starts_with("font:"));
  let mut output = String::new();
  for line in text.lines() {
    if has_font {
      if line.trim_start().starts_with("font:") {
        output.push_str(font_line);
      } else {
        output.push_str(line);
      }
      output.push('\n');
      continue;
    }
    output.push_str(line);
    output.push('\n');
    if is_configuration_open(line) {
      output.push_str(font_line);
      output.push('\n');
    }
  }
  output
}

fn is_configuration_open(line: &str) -> bool {
  line
    .trim_start()
    .strip_prefix("configuration")
    .is_some_and(|rest| rest.trim_start().starts_with('{'))
}

fn write_atomically(path: &Path, contents: &str) -> io::Result<()> {
  let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
  fs::write(&temporary, contents)?;
  fs::rename(&temporary, path)
}

/// Asks `argvus system-monitor` to apply the theme. Its errors are ignored.
fn sync_btop(theme: &str) {
  let _ = Command::new("argvus")
    .args(["system-monitor", "--apply", theme])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

/// Makes the yazi configuration match the theme and returns the directory yazi
/// must read, or `None` when there is no yazi configuration at all.
fn sync_yazi(paths: &SessionPaths, theme: &str) -> Option<PathBuf> {
  let yazi_dir = paths.user_config("yazi");
  let system_yazi = paths.system_config("app-profiles/config/yazi");
  let flavor_name = format!("{theme}.yazi");

  if !yazi_dir.is_dir() && system_yazi.is_dir() {
    fs::create_dir_all(&yazi_dir).ok()?;
    copy_dir_contents(&system_yazi, &yazi_dir).ok()?;
  } else if system_yazi.is_dir() {
    fs::create_dir_all(&yazi_dir).ok()?;
    for item in ["package.toml", "keymap.toml"] {
      let source = system_yazi.join(item);
      let target = yazi_dir.join(item);
      if source.is_file() && !target.exists() {
        let _ = fs::copy(&source, &target);
      }
    }
    if system_yazi.join("flavors").is_dir() {
      let flavors = yazi_dir.join("flavors");
      fs::create_dir_all(&flavors).ok()?;
      for entry in fs::read_dir(system_yazi.join("flavors")).ok()?.flatten() {
        let source = entry.path();
        let name = entry.file_name();
        let is_flavor = name.to_string_lossy().ends_with(".yazi");
        if is_flavor && source.is_dir() && !flavors.join(&name).exists() {
          let _ = copy_dir_contents(&source, &flavors.join(&name));
        }
      }
    }
    let theme_source = system_yazi.join("flavors").join(&flavor_name);
    if theme_source.is_dir() {
      let theme_target = yazi_dir.join("flavors").join(&flavor_name);
      fs::create_dir_all(&theme_target).ok()?;
      copy_dir_contents(&theme_source, &theme_target).ok()?;
    }
  }

  let config = if yazi_dir
    .join("flavors")
    .join(&flavor_name)
    .join("flavor.toml")
    .is_file()
  {
    fs::write(yazi_dir.join("theme.toml"), flavor_toml(theme)).ok()?;
    yazi_dir.clone()
  } else if system_yazi.is_dir() {
    system_yazi.clone()
  } else {
    return None;
  };

  // Same propagation as before: systemd and the compositor both learn the
  // directory, so later launches of yazi read it. The compositor call is a
  // setenv, not a config change, and stays as it was.
  let _ = Command::new("systemctl")
    .args(["--user", "set-environment"])
    .arg(format!("YAZI_CONFIG_HOME={}", config.display()))
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  let _ = Command::new("hyprctl")
    .arg("setenv")
    .arg(format!("YAZI_CONFIG_HOME,{}", config.display()))
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();

  mirror_native_yazi(paths, &yazi_dir, theme, &flavor_name);
  Some(config)
}

/// Copies the active flavor into the native yazi directory when that copy
/// already belongs to ARGVUS (its theme file names an `argvus-` theme).
fn mirror_native_yazi(paths: &SessionPaths, yazi_dir: &Path, theme: &str, flavor_name: &str) {
  let native_dir = paths.config_home.join("yazi");
  let native_theme = native_dir.join("theme.toml");
  let owned_by_argvus = fs::read_to_string(&native_theme)
    .map(|text| text.contains("argvus-"))
    .unwrap_or(false);
  let flavor = yazi_dir.join("flavors").join(flavor_name);
  if !owned_by_argvus || !flavor.is_dir() {
    return;
  }
  let target = native_dir.join("flavors").join(flavor_name);
  if fs::create_dir_all(&target).is_ok() && copy_dir_contents(&flavor, &target).is_ok() {
    let _ = fs::write(native_theme, flavor_toml(theme));
  }
}

fn flavor_toml(theme: &str) -> String {
  format!("[flavor]\ndark = \"{theme}\"\n")
}

/// Copies every entry of `source` into `target`, overwriting files. Like
/// `cp -R src/. dst/`.
fn copy_dir_contents(source: &Path, target: &Path) -> io::Result<()> {
  fs::create_dir_all(target)?;
  for entry in fs::read_dir(source)? {
    let entry = entry?;
    let destination = target.join(entry.file_name());
    if entry.file_type()?.is_dir() {
      copy_dir_contents(&entry.path(), &destination)?;
    } else {
      fs::copy(entry.path(), destination)?;
    }
  }
  Ok(())
}

/// Runs a packaged script with `--apply-static` and returns its status. A
/// missing script reports 127, as `sh` does.
pub fn run_script(paths: &SessionPaths, relative: &str, environment: &[(String, String)]) -> i32 {
  let script = paths.read_config(relative);
  let mut command = Command::new("sh");
  command
    .arg(&script)
    .arg("--apply-static")
    .env("ARGVUS_CONFIG_HOME", &paths.config_home)
    .env("ARGVUS_SYSTEM_CONFIG", &paths.system_config);
  for (name, value) in environment {
    command.env(name, value);
  }
  match command.status() {
    Ok(status) => status.code().unwrap_or(1),
    Err(_) => 127,
  }
}

/// Re-applies the monitor layout written by nwg-displays. Errors are ignored,
/// as the shell ignored them.
fn apply_display(environment: &[(String, String)]) {
  crate::display::run_step("session-reload", environment);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rofi_font_replaces_the_existing_line() {
    let text = "configuration {\n    font: \"Old 9\";\n}\n";
    assert_eq!(
      apply_rofi_font(text, "    font: \"New 11\";"),
      "configuration {\n    font: \"New 11\";\n}\n"
    );
  }

  #[test]
  fn rofi_font_is_inserted_after_configuration_when_missing() {
    let text = "configuration {\n    show-icons: true;\n}\n";
    assert_eq!(
      apply_rofi_font(text, "    font: \"New 11\";"),
      "configuration {\n    font: \"New 11\";\n    show-icons: true;\n}\n"
    );
  }

  #[test]
  fn active_theme_defaults_when_no_file_exists() {
    let paths = SessionPaths {
      config_home: std::env::temp_dir().join("argvus-reload-no-theme"),
      data_home: std::env::temp_dir().join("argvus-reload-no-theme/argvus/data"),
      system_config: std::env::temp_dir().join("argvus-reload-no-theme/system"),
    };
    assert_eq!(active_theme(&paths), DEFAULT_THEME);
  }

  #[test]
  fn active_theme_reads_the_first_line_of_the_file() {
    let root = std::env::temp_dir().join(format!("argvus-reload-theme-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let data = root.join("config/argvus/data");
    fs::create_dir_all(&data).expect("data dir");
    fs::write(data.join(".active-theme"), "tokyo-night\nignored\n").expect("theme file");
    let paths = SessionPaths {
      config_home: root.join("config"),
      data_home: data,
      system_config: root.join("system"),
    };
    assert_eq!(active_theme(&paths), "tokyo-night");
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn active_theme_prefers_the_canonical_theme_over_the_marker() {
    let root = std::env::temp_dir().join(format!("argvus-reload-canonical-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let config_home = root.join("config");
    let data = config_home.join("argvus/data");
    fs::create_dir_all(&data).expect("data dir");
    fs::write(data.join(".active-theme"), "tokyo-night\n").expect("marker");
    crate::ConfigStore::at(&config_home)
      .modify(|document| {
        document.set(
          "/appearance/theme",
          serde_json::Value::String("nord-light".into()),
        )
      })
      .expect("canonical theme");
    let paths = SessionPaths {
      config_home,
      data_home: data,
      system_config: root.join("system"),
    };
    assert_eq!(active_theme(&paths), "nord-light");
    let _ = fs::remove_dir_all(root);
  }
}
