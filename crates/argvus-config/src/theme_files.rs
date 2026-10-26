//! The files that applications read from their own places, copied from the
//! current theme build: the terminal and system monitor profiles, the
//! notifications, the Hyprland toolkit and application style, yazi, superfile,
//! the removable-devices panel, the calendar and the lock screen.
//!
//! A theme switch writes them after the build is current, and so does an
//! accent change, which renders a new build with the same theme.

use crate::session_paths::{self, SessionPaths};
use crate::text_edit::{self, Layout};
use crate::theme_build;
use crate::theme_surfaces;
use crate::user_copy;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The font values of the themed files, read from the generated fonts file
/// with the fallbacks the switch has always used.
pub struct Fonts {
  pub apps: String,
  pub taskbar_family: String,
  pub taskbar_size: String,
  pub sysinfo_family: String,
  pub sysinfo_size: String,
  pub system_family: String,
  pub system_size: String,
}

impl Fonts {
  pub fn read(paths: &SessionPaths) -> Self {
    let file = paths.generated_fonts();
    let value = |key: &str, fallback: &str| session_paths::font_state_value(&file, key, fallback);
    let default_name = value("default_name", "IBM Plex Mono");
    let default_size = value("default_size", "12");
    let default_family = value("default_family", "IBM Plex Mono");
    let monospace_family = value("monospace_family", "IBM Plex Mono");
    let monospace_size = value("monospace_size", "14");
    let apps_name = value("apps_name", &default_name);
    let apps_size = value("apps_size", &default_size);
    Self {
      apps: format!("{apps_name} {apps_size}"),
      taskbar_family: value("taskbar_family", &value("default_family", "IBM Plex Mono")),
      taskbar_size: value("taskbar_size", &value("default_size", "13")),
      sysinfo_family: value("sysinfo_family", &monospace_family),
      sysinfo_size: value("sysinfo_size", &monospace_size),
      system_family: value("system_family", &default_family),
      system_size: value("system_size", &value("default_size", "13")),
    }
  }
}

/// Writes every file copied from the current build for `theme`. Each step runs
/// even when an earlier one failed; the error is the message key (without its
/// `theme.` prefix) of the first step that must fail the caller.
pub fn write_build_copies(
  paths: &SessionPaths,
  theme: &str,
  fonts: &Fonts,
  icon_theme: &str,
) -> Result<(), &'static str> {
  let mut failure = None;
  if on_path("argvus") && !run_quiet("argvus", &["terminal", "--apply", theme]) {
    failure = Some("terminal_apply_failed");
  }
  apply_dunst(paths, theme);
  report(apply_toolkit(paths, fonts));
  report(apply_qt6ct_icons(paths, icon_theme));
  if on_path("argvus") {
    let _ = run_quiet("argvus", &["system-monitor", "--apply", theme]);
  }
  report(apply_yazi(paths, theme));
  report(apply_superfile(paths, theme));
  report(apply_storage(paths));
  report(apply_calendar(paths, theme));
  if !run_lock_theme(paths) {
    failure.get_or_insert("hyprlock_failed");
  }
  failure.map_or(Ok(()), Err)
}

fn report(result: io::Result<()>) {
  if let Err(error) = result {
    eprintln!("argvus-config: theme files: {error}");
  }
}

/// A build file the theme must have: a missing one is an error, so a broken
/// build is reported instead of leaving another theme's copy in place.
fn build_file(paths: &SessionPaths, relative: &str) -> io::Result<PathBuf> {
  theme_build::current_file(paths, relative).ok_or_else(|| {
    io::Error::new(
      io::ErrorKind::NotFound,
      format!("the theme build has no {relative}"),
    )
  })
}

/// Copies `source` over `destination`, through a file beside it and a rename,
/// so a reader never sees half a file.
fn copy_atomic(source: &Path, destination: &Path) -> io::Result<()> {
  if let Some(parent) = destination.parent() {
    fs::create_dir_all(parent)?;
  }
  let temporary = destination.with_extension(format!("argvus.{}", std::process::id()));
  let copied = fs::copy(source, &temporary).and_then(|_| fs::rename(&temporary, destination));
  if copied.is_err() {
    let _ = fs::remove_file(&temporary);
  }
  copied
}

/// `argvus config setting` for a spaced setting: replaces the key or appends it,
/// and does nothing when the file is missing.
pub fn set_setting(path: &Path, key: &str, value: &str) -> io::Result<()> {
  if !path.is_file() {
    return Ok(());
  }
  let text = fs::read_to_string(path)?;
  let edited = text_edit::set_setting(&text, key, value, Layout::Spaced);
  if edited != text {
    text_edit::replace_file(path, &edited)?;
  }
  Ok(())
}

/// The notification colours, through the notifications library, which keeps
/// every other setting of the user's dunstrc.
fn apply_dunst(paths: &SessionPaths, theme: &str) {
  let helper = paths.system_config("notifications/sh/theme.sh");
  let Some(dunstrc) = user_copy::config_path(paths, "notifications/config/dunstrc") else {
    return;
  };
  if !helper.is_file() {
    return;
  }
  let _ = Command::new("sh")
    .arg("-c")
    .arg(". \"$1\"; argvus_notifications_apply_theme \"$2\" \"$3\"")
    .arg("sh")
    .arg(&helper)
    .arg(theme)
    .arg(&dunstrc)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

/// The Hyprland toolkit (with the system font) and the application style.
fn apply_toolkit(paths: &SessionPaths, fonts: &Fonts) -> io::Result<()> {
  if let Some(toolkit) = user_copy::config_path(paths, "appearance/config/hypr/hyprtoolkit.conf") {
    copy_atomic(&build_file(paths, "hypr/hyprtoolkit.conf")?, &toolkit)?;
    set_setting(
      &toolkit,
      "font_family",
      &format!("\"{}\"", fonts.system_family),
    )?;
    set_setting(&toolkit, "font_size", &fonts.system_size)?;
  }
  if let Some(style) =
    user_copy::config_path(paths, "appearance/config/hypr/application-style.conf")
  {
    copy_atomic(&build_file(paths, "hypr/application-style.conf")?, &style)?;
  }
  Ok(())
}

/// The icon theme line of qt6ct, where the file already has one.
fn apply_qt6ct_icons(paths: &SessionPaths, icon_theme: &str) -> io::Result<()> {
  let Some(qt6ct) = user_copy::config_path(paths, "appearance/config/qt6ct/qt6ct.conf")
    .filter(|path| path.is_file())
  else {
    return Ok(());
  };
  let text = fs::read_to_string(&qt6ct)?;
  let rules = [crate::accent::Rule::literal(
    r"^[[:space:]]*icon_theme[[:space:]]*=.*",
    format!("icon_theme={icon_theme}"),
  )];
  let edited = crate::accent::apply_rules(&text, &rules);
  if edited != text {
    text_edit::replace_file(&qt6ct, &edited)?;
  }
  Ok(())
}

/// The yazi flavor of the build, selected under the theme's name.
fn apply_yazi(paths: &SessionPaths, theme: &str) -> io::Result<()> {
  let Some(yazi_root) = user_copy::config_path(paths, "app-profiles/config/yazi") else {
    return Ok(());
  };
  let flavor = yazi_root.join("flavors").join(format!("{theme}.yazi"));
  for name in ["flavor.toml", "tmtheme.xml"] {
    copy_atomic(
      &build_file(paths, &format!("yazi/{name}"))?,
      &flavor.join(name),
    )?;
  }
  fs::write(
    yazi_root.join("theme.toml"),
    crate::accent::yazi_theme_file(theme),
  )
}

/// The superfile theme of the build, selected in its config, and in the native
/// superfile directory when the user has one.
fn apply_superfile(paths: &SessionPaths, theme: &str) -> io::Result<()> {
  let Some(config_root) = user_copy::config_path(paths, "app-profiles/config/superfile") else {
    return Ok(());
  };
  let theme_file = config_root.join("theme").join(format!("{theme}.toml"));
  copy_atomic(&build_file(paths, "superfile/theme.toml")?, &theme_file)?;
  let config = config_root.join("config.toml");
  if !config.is_file() {
    return Ok(());
  }
  set_setting(&config, "theme", &format!("\"{theme}\""))?;
  let native = paths.config_home.join("superfile");
  if native.is_dir() {
    copy_atomic(
      &theme_file,
      &native.join("theme").join(format!("{theme}.toml")),
    )?;
    let native_config = native.join("config.toml");
    if !native_config.exists() {
      fs::copy(&config, &native_config)?;
    }
    set_setting(&native_config, "theme", &format!("\"{theme}\""))?;
  }
  Ok(())
}

/// The stylesheet of the removable-devices panel.
fn apply_storage(paths: &SessionPaths) -> io::Result<()> {
  let Some(destination) = user_copy::config_path(paths, "removable-devices/config/theme.css")
  else {
    return Ok(());
  };
  copy_atomic(
    &build_file(paths, "removable-devices/theme.css")?,
    &destination,
  )
}

/// The calendar's stylesheet cache, which the running calendar is told to reload.
fn apply_calendar(paths: &SessionPaths, theme: &str) -> io::Result<()> {
  let cache_home = env::var_os("XDG_CACHE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"));
  theme_surfaces::apply_calendar_theme(
    &build_file(paths, "taskbar-calendar/theme.css")?,
    &cache_home.join("argvus-taskbar-calendar/theme.css"),
    theme,
  )
}

/// The lock screen, rebuilt from the build when the lock package has its helper.
fn run_lock_theme(paths: &SessionPaths) -> bool {
  let Some(script) =
    user_copy::config_path(paths, "lock/sh/hyprlock-theme.sh").filter(|path| path.is_file())
  else {
    return true;
  };
  Command::new("sh")
    .arg(script)
    .arg("--invalidate")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .is_ok_and(|status| status.success())
}

/// The Waybar mode stylesheet, emptied by a theme switch.
pub fn reset_mode_stylesheet(paths: &SessionPaths) -> io::Result<()> {
  let Some(mode_css) = user_copy::config_path(paths, "appearance/config/waybar/mode.css") else {
    return Ok(());
  };
  if let Some(parent) = mode_css.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::write(&mode_css, "/* mode.css — reset on theme switch */\n")
}

pub(crate) fn on_path(program: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
    .unwrap_or(false)
}

/// Runs a command and reports whether it succeeded. Its output is discarded.
pub(crate) fn run_quiet(program: &str, arguments: &[&str]) -> bool {
  Command::new(program)
    .args(arguments)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}
