//! The body of a theme switch, run under the switch lock.
//!
//! The theme is rendered into a build first: a theme that is not installed or
//! does not render changes nothing. Then the choice is committed, the consumers
//! that read the build reload, and the files the other applications read are
//! copied from the build. The helpers that belong to other packages (the icon
//! and notification libraries, the taskbar mode scripts, spaces, borders and
//! the lock screen) still run as processes, so each package keeps its own code.

use crate::ConfigStore;
use crate::appearance_runtime::{self, GtkFiles, GtkRuntime};
use crate::icons;
use crate::session_paths::SessionPaths;
use crate::text_edit::{self, Target};
use crate::theme_build;
use crate::theme_files::{self, Fonts, on_path, run_quiet};
use crate::theme_finish::{self, Finish};
use crate::theme_resolve;
use crate::theme_surfaces;
use crate::user_copy;
use crate::waybar;
use argvus_theme_core::paths::ThemePaths;
use argvus_theme_core::registry::ReloadStage;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Runs the body and returns the exit status.
pub fn run(paths: &SessionPaths, theme: &str, runtime: bool, live: bool) -> i32 {
  let theme_paths = theme_build::theme_paths(paths);
  if theme_resolve::manifest(&theme_paths, theme).is_none() {
    message("theme.not_installed", &[format!("theme={theme}")]);
    return 1;
  }
  let active_file = paths.data_home.join(".active-theme");
  if let Some(parent) = active_file.parent() {
    let _ = fs::create_dir_all(parent);
  }
  // The layout of the theme being replaced: a layout change restarts the bars,
  // so only then are their managed files rebuilt from the packaged ones.
  let layout_changed =
    theme_build::displayed_theme(paths).ends_with("-float") != theme.ends_with("-float");
  let icon = icons::resolve(paths, theme, Stdio::inherit());
  let fonts = Fonts::read(paths);
  let wallpaper = find_theme_wallpaper(&theme_paths, theme).unwrap_or_default();

  // An official selection resets the accent with the canonical commit, so the
  // build takes the theme's own accent then.
  let accent = if env::var("ARGVUS_ACCENT_OFFICIAL").as_deref() == Ok("1") {
    None
  } else {
    crate::accent::canonical_accent(Some(&ConfigStore::at(&paths.config_home)))
  };
  let previous_build = theme_build::active_build(paths);
  if let Err(error) = theme_build::refresh(paths, theme, accent.as_deref()) {
    message(
      "theme.build_failed",
      &[format!("theme={theme}"), format!("error={error}")],
    );
    return 1;
  }
  if !apply_canonical_theme(theme, &wallpaper) {
    theme_build::restore(paths, previous_build.as_deref());
    return 1;
  }

  let mut status = 0;
  let _ = text_edit::replace_file(&active_file, theme);
  // Rofi takes its background from its own theme file, which the effects step
  // at the end of the switch used to write seconds after the build changed.
  // Writing it now, once the theme is committed, keeps a launcher opened during
  // the rest of the switch from showing the previous theme's background.
  crate::effects::apply_launcher();
  // The consumers that read the build directly change now; the files the
  // others read are written below and reloaded when the switch ends.
  if live {
    crate::theme_runtime::reload_consumers(paths, Some(ReloadStage::Build));
    if !wallpaper.is_empty() {
      crate::theme_runtime::show_wallpaper(paths, &wallpaper);
    }
  }
  let _ = theme_surfaces::publish_greeter_theme(
    &theme_paths.system_themes(),
    &greeter_dir(),
    theme,
    theme_surfaces::current_user_id(),
  );

  apply_waybar(paths, theme, !runtime || layout_changed, &fonts);
  apply_qt6ct(paths);
  if let Err(key) = theme_files::write_build_copies(paths, theme, &fonts, &icon.current) {
    message(&format!("theme.{key}"), &[format!("theme={theme}")]);
    status = 1;
  }
  if let Err(error) = theme_files::reset_mode_stylesheet(paths) {
    eprintln!("argvus-config: theme files: {error}");
  }
  if runtime {
    restart_polkit_agent();
  }
  apply_gtk(
    paths,
    &theme_paths,
    theme,
    &icon.current,
    &fonts.apps,
    runtime,
  );

  let finish = Finish {
    wallpaper: &wallpaper,
    runtime,
  };
  if let Err(key) = theme_finish::run(paths, &finish) {
    message(&format!("theme.{key}"), &[format!("theme={theme}")]);
    status = 1;
  }
  finish_transition(paths, status, runtime)
}

/// The polkit agent reads the toolkit colours at start. It shows nothing while
/// idle, so a restart is not visible.
fn restart_polkit_agent() {
  let _ = Command::new("systemctl")
    .args([
      "--user",
      "set-environment",
      "QT_QPA_PLATFORM=wayland",
      "QT_QPA_PLATFORMTHEME=qt6ct",
      "QT_QUICK_CONTROLS_STYLE=org.hyprland.style",
    ])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  let _ = Command::new("systemctl")
    .args(["--user", "restart", "hyprpolkitagent"])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

/// The end of the switch: with the runtime, the spacing and borders get their
/// final geometry before the status is returned. Waybar is left to the
/// transaction, which restarts the bars only when the layout changed.
fn finish_transition(paths: &SessionPaths, mut status: i32, runtime: bool) -> i32 {
  if !runtime {
    return status;
  }
  for helper in [
    "hyprland/sh/spaces-switch.sh",
    "hyprland/sh/borders-switch.sh",
  ] {
    if let Some(script) = user_copy::config_path(paths, helper).filter(|path| path.is_file())
      && !apply_geometry_helper(&script)
      && status == 0
    {
      status = 1;
    }
  }
  status
}

/// Runs a geometry helper with `--apply`, leaving Waybar to the transaction.
fn apply_geometry_helper(script: &Path) -> bool {
  Command::new("sh")
    .arg(script)
    .arg("--apply")
    .env("ARGVUS_SKIP_WAYBAR_RESTART", "1")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .is_ok_and(|status| status.success())
}

/// Applies the theme through the canonical configuration, unless a projection
/// is running (that projection already did it).
/// Returns `false` when the canonical commit failed, which the caller reports.
fn apply_canonical_theme(theme: &str, wallpaper: &str) -> bool {
  // A projection or a mode switch already holds the canonical choice: commit nothing,
  // so a custom wallpaper or accent is not reset. The runtime still runs.
  let skip_commit = env::var("ARGVUS_PROJECTING").as_deref() == Ok("1")
    || env::var("ARGVUS_SKIP_CANONICAL").as_deref() == Ok("1");
  if skip_commit || !on_path("argvus") {
    return true;
  }
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  let mut arguments = vec![
    "config".to_string(),
    "apply-theme".to_string(),
    family.to_string(),
  ];
  if env::var("ARGVUS_ACCENT_OFFICIAL").as_deref() == Ok("1") {
    arguments.push("--reset-accent".to_string());
  }
  if !wallpaper.is_empty() {
    arguments.push("--wallpaper".to_string());
    arguments.push(wallpaper.to_string());
    arguments.push("--reset-wallpaper".to_string());
  }
  let status = Command::new("argvus")
    .args(&arguments)
    .stdout(Stdio::null())
    .status();
  match status {
    Ok(result) if result.success() => true,
    Ok(result) => {
      eprintln!("argvus-config: the canonical theme commit failed ({result})");
      false
    }
    Err(error) => {
      eprintln!("argvus-config: cannot run argvus for the canonical commit: {error}");
      false
    }
  }
}

/// The Waybar stylesheets and the rofi theme of the switch: each imports its
/// colours from the build.
fn apply_waybar(paths: &SessionPaths, theme: &str, refresh_bars: bool, fonts: &Fonts) {
  // A running bar reloads its stylesheet on every write. Restoring the packaged
  // copy first would show the packaged theme for a moment, so a running switch
  // rebuilds the files only when the bars restart anyway or a file is missing.
  for relative in [
    "taskbar/config/argvus-taskbar.jsonc",
    "taskbar/config/argvus-taskbar.css",
    "widget-telemetry/config/argvus-widget-telemetry.jsonc",
    "widget-telemetry/config/argvus-widget-telemetry.css",
  ] {
    if refresh_bars || !paths.user_config(relative).is_file() {
      let _ = waybar::refresh_managed(paths, relative);
    }
  }
  for script in [
    "appearance/sh/taskbar-right-2-mode.sh",
    "appearance/sh/taskbar-widgets-mode.sh",
  ] {
    if let Some(path) = user_copy::config_path(paths, script).filter(|path| path.is_file()) {
      let _ = Command::new(path)
        .arg("apply")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
  }
  if on_path("argvus") {
    let _ = run_quiet("argvus", &["widget-telemetry-toggle", "blocks", "apply"]);
  }

  let taskbar_css = path_or_empty(paths, "taskbar/config/argvus-taskbar.css");
  let telemetry_css = path_or_empty(paths, "widget-telemetry/config/argvus-widget-telemetry.css");
  let taskbar_json = path_or_empty(paths, "taskbar/config/argvus-taskbar.jsonc");
  for css in [&taskbar_css, &telemetry_css] {
    set_target_in_file(css, Target::WaybarModeImport, "./mode.css");
  }

  let _ = waybar::apply_layout(paths, theme);
  apply_chosen_taskbar_shape(paths);
  let build = theme_build::theme_paths(paths).current();
  set_target_in_file(
    &taskbar_css,
    Target::WaybarThemeImport,
    &build.join("taskbar/theme.css").display().to_string(),
  );
  set_target_in_file(
    &telemetry_css,
    Target::WaybarTelemetryThemeImport,
    &build
      .join("widget-telemetry/theme.css")
      .display()
      .to_string(),
  );

  let taskbar_block = format!(
    "* {{\n  font-family: \"{}\", \"Symbols Nerd Font Mono\", monospace;\n  font-size: {}px;\n}}\n\n\
#custom-icon-window,\n#network,\n#bluetooth,\n#custom-bluetooth,\n#custom-expand-icon,\n\
#custom-removable-devices,\n#custom-recording,\n#custom-search,\n#pulseaudio,\n\
#power-profiles-daemon,\n#custom-settings,\n#custom-power {{\n  font-family: \"Symbols Nerd Font Mono\";\n}}",
    fonts.taskbar_family, fonts.taskbar_size
  );
  let telemetry_block = format!(
    "* {{\n  font-family: \"{}\", \"Symbols Nerd Font Mono\", monospace;\n  font-size: {}px;\n}}",
    fonts.sysinfo_family, fonts.sysinfo_size
  );
  let _ = waybar::write_font_block(&taskbar_css, &taskbar_block);
  let _ = waybar::write_font_block(&telemetry_css, &telemetry_block);

  let rofi_config = path_or_empty(paths, "launcher/config/config.rasi");
  let rofi_theme_file = path_or_empty(paths, "launcher/config/theme.rasi");
  let rofi_mode = path_or_empty(paths, "launcher/config/mode.rasi");
  set_target_in_file(
    &taskbar_json,
    Target::WaybarRofiDrun,
    &rofi_config.display().to_string(),
  );
  set_target_in_file(
    &rofi_config,
    Target::RofiTheme,
    &rofi_theme_file.display().to_string(),
  );
  set_target_in_file(&rofi_config, Target::RofiFont, &fonts.apps);
  set_target_in_file(
    &rofi_theme_file,
    Target::RofiImportTheme,
    &build.join("launcher/theme.rasi").display().to_string(),
  );
  set_target_in_file(
    &rofi_theme_file,
    Target::RofiImportMode,
    &rofi_mode.display().to_string(),
  );
}

/// Writes the taskbar height and border chosen in the Control Center back into
/// the managed files, which the switch has just refreshed from the packaged
/// copies. Unset values keep what the theme class wrote.
fn apply_chosen_taskbar_shape(paths: &SessionPaths) {
  let result = ConfigStore::at(&paths.config_home)
    .load_effective()
    .map(|document| waybar::TaskbarShape::chosen(&document))
    .map_err(|error| error.to_string())
    .and_then(|shape| waybar::apply_taskbar_shape(paths, shape).map_err(|error| error.to_string()));
  if let Err(error) = result {
    eprintln!("argvus-config: taskbar shape: {error}");
  }
}

/// Writes one line of a file through the same rule the `theme-file` command uses.
fn set_target_in_file(path: &Path, target: Target, value: &str) {
  if path.as_os_str().is_empty() || !path.is_file() {
    return;
  }
  if let Ok(text) = fs::read_to_string(path) {
    let edited = text_edit::set_target(&text, target, value);
    if edited != text {
      let _ = text_edit::replace_file(path, &edited);
    }
  }
}

/// The qt6ct colour scheme: the build's, which follows every switch.
fn apply_qt6ct(paths: &SessionPaths) {
  let qt6ct = path_or_empty(paths, "appearance/config/qt6ct/qt6ct.conf");
  let colors = theme_build::theme_paths(paths)
    .current()
    .join("qt6ct/colors.conf");
  set_target_in_file(
    &qt6ct,
    Target::Qt6ctColorScheme,
    &colors.display().to_string(),
  );
  set_target_in_file(&qt6ct, Target::Qt6ctCustomPalette, "true");
}

/// The GTK mode of the theme: the settings and, when running, the live GTK
/// settings.
fn apply_gtk(
  paths: &SessionPaths,
  theme_paths: &ThemePaths,
  theme: &str,
  icon: &str,
  font: &str,
  runtime: bool,
) {
  let light = theme_resolve::gtk_mode(theme_paths, theme) == "light";
  let name = appearance_runtime::gtk_theme_name(light).to_string();
  let (file_mode, prefer, scheme, fallback) = if light {
    ("light", "0", "prefer-light", "Adwaita-dark")
  } else {
    ("dark", "1", "prefer-dark", "Adwaita")
  };
  appearance_runtime::apply_gtk_theme_files(
    paths,
    &GtkFiles {
      mode: file_mode.to_string(),
      gtk_theme_name: name.clone(),
      prefer_dark: prefer.to_string(),
      icon_theme: icon.to_string(),
      font: font.to_string(),
    },
  );
  if runtime {
    appearance_runtime::apply_gtk_runtime(&GtkRuntime {
      scheme: scheme.to_string(),
      theme_name: name,
      fallback_theme: fallback.to_string(),
      font: font.to_string(),
      icon_theme: icon.to_string(),
    });
  }
}

/// The wallpaper of the theme: from its manifest or its built-in name. `None`
/// when the theme has none.
fn find_theme_wallpaper(theme_paths: &ThemePaths, theme: &str) -> Option<String> {
  let root = env::var_os("WALLPAPER_ROOT")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      env::var_os("ARGVUS_BACKGROUNDS_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/share/backgrounds"))
        .join("argvus")
    });
  theme_resolve::theme_wallpaper(theme_paths, &root, theme).map(|path| path.display().to_string())
}

/// The mutable copy of a file, or empty when it cannot be made.
fn path_or_empty(paths: &SessionPaths, relative: &str) -> PathBuf {
  user_copy::config_path(paths, relative).unwrap_or_default()
}

/// The greeter's state directory.
fn greeter_dir() -> PathBuf {
  env::var_os("ARGVUS_GREETER_THEME_STATE_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/var/lib/argvus/greeter/themes"))
}

/// Prints a translated message on stderr, as `argvus_tr` did.
fn message(key: &str, arguments: &[String]) {
  if on_path("argvus") {
    let _ = Command::new("argvus")
      .args(["i18n", "get", "appearance", key])
      .args(arguments)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .status();
  } else {
    eprintln!("{key}");
  }
}
