//! The body of a theme switch, run under the switch lock.
//!
//! It rewrites the theme files of every component, in the order the switch
//! always used. The helpers that belong to other packages (the icon and
//! notification libraries, the taskbar mode scripts, spaces, borders and the
//! lock screen) still run as processes, so each package keeps its own code.
//! The body of a theme switch: the theme files, in the order the switch has
//! always applied them.

use crate::appearance_runtime::{self, GtkFiles, GtkRuntime};
use crate::icons;
use crate::session_paths::{self, SessionPaths};
use crate::text_edit::{self, Target};
use crate::theme_dirs;
use crate::theme_files;
use crate::theme_finish::{self, Finish};
use crate::theme_resolve;
use crate::theme_surfaces;
use crate::theme_transition;
use crate::user_copy;
use crate::waybar;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Runs the body and returns the exit status. The cleanup of the transition
/// runs when the switch has started it, on success and on failure.
pub fn run(paths: &SessionPaths, theme: &str, runtime: bool) -> i32 {
  let themes_dir = paths.system_config("appearance/themes.d");
  let active_file = paths.config_home.join("argvus/data/.active-theme");
  if let Some(parent) = active_file.parent() {
    let _ = fs::create_dir_all(parent);
  }
  let icon = icons::resolve(paths, theme, Stdio::inherit());
  let workspace_root = workspace_root(paths);

  // Directories and paths of the theme. A missing hypr theme is reported and
  // the switch goes on with an empty directory, as the subshell did.
  let hypr_themes = match theme_dirs::theme_parent(paths, "appearance/config/hypr/themes", theme) {
    Some(parent) => parent,
    None => {
      message(
        paths,
        "theme.directory_missing",
        &[format!(
          "path={}",
          paths
            .system_config(&format!("appearance/config/hypr/themes/{theme}"))
            .display()
        )],
      );
      // The shell's empty variable made these paths absolute.
      PathBuf::from("/")
    }
  };
  let rofi_themes = optional_parent(paths, "launcher/config/themes", theme);
  optional_parent(paths, "taskbar/config/themes", theme);
  optional_parent(paths, "widget-telemetry/config/themes", theme);
  optional_parent(
    paths,
    "control-panel/config/quickshell/argvus-control-panel/themes",
    theme,
  );
  let rofi_config = path_or_empty(paths, "launcher/config/config.rasi");
  let rofi_theme_file = path_or_empty(paths, "launcher/config/theme.rasi");
  let rofi_mode = path_or_empty(paths, "launcher/config/mode.rasi");
  let _ = user_copy::config_path(paths, "app-profiles/config/superfile");
  let superfile_theme = user_copy::config_path(
    paths,
    &format!("app-profiles/config/superfile/theme/{theme}.toml"),
  )
  .unwrap_or_default();
  let qt6ct_colors = path_or_empty(paths, "appearance/config/qt6ct/colors");
  let _ = user_copy::config_path(paths, "appearance/config/hypr/hyprpaper.conf");
  let fonts = Fonts::read(paths);

  if !superfile_theme.is_file() {
    message(
      paths,
      "theme.superfile_missing",
      &[format!("path={}", superfile_theme.display())],
    );
  }
  let colors_file = qt6ct_colors.join(format!("{theme}.conf"));
  if !colors_file.is_file() {
    message(
      paths,
      "theme.qt6ct_missing",
      &[format!("path={}", colors_file.display())],
    );
  }

  // The shell lookup always succeeded, even without a wallpaper, so the
  // switch goes on with an empty one.
  let wallpaper = find_theme_wallpaper(paths, &themes_dir, theme).unwrap_or_default();

  if !apply_canonical_theme(theme, &wallpaper) {
    return 1;
  }

  let mut status = 0;
  if runtime {
    theme_transition::stop_units();
  }
  let _ = crate::text_edit::replace_file(&active_file, theme);
  let uid_dir = greeter_dir();
  let _ = theme_surfaces::publish_greeter_theme(
    &themes_dir,
    &uid_dir,
    theme,
    theme_surfaces::current_user_id(),
  );

  apply_waybar(
    paths,
    theme,
    &fonts,
    &rofi_config,
    &rofi_theme_file,
    &rofi_mode,
    &rofi_themes,
  );

  if on_path("argvus") && !run_quiet("argvus", &["terminal", "--apply", theme]) {
    message(paths, "theme.terminal_apply_failed", &[]);
    status = 1;
  }

  apply_dunst(paths, theme);

  let _ = theme_files::apply_hypr_stage(
    paths,
    &hypr_themes,
    theme,
    &fonts.system_family,
    &fonts.system_size,
    &icon.current,
  );

  apply_qt6ct(paths, theme, &qt6ct_colors);

  if runtime
    && (hypr_themes.join(theme).join("hyprtoolkit.conf").is_file()
      || hypr_themes
        .join(theme)
        .join("application-style.conf")
        .is_file())
  {
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

  if on_path("argvus") {
    let _ = run_quiet("argvus", &["system-monitor", "--apply", theme]);
  }

  if let Ok(Some(missing)) = theme_files::apply_tail_stage(paths, theme) {
    message(
      paths,
      "theme.yazi_missing",
      &[format!("path={}", missing.display())],
    );
  }

  apply_gtk(
    paths,
    &themes_dir,
    theme,
    &icon.current,
    &fonts.apps,
    runtime,
  );

  let finish = Finish {
    theme,
    wallpaper: &wallpaper,
    workspace_root: &workspace_root,
    runtime,
  };
  if let Err(key) = theme_finish::run(paths, &finish) {
    message(paths, &format!("theme.{key}"), &[format!("theme={theme}")]);
    status = 1;
    return finish_transition(paths, status, runtime);
  }

  finish_transition(paths, status, runtime)
}

/// The end of the switch: with the transition started, the spacing and borders
/// get their final geometry before the status is returned.
fn finish_transition(paths: &SessionPaths, mut status: i32, runtime: bool) -> i32 {
  if !runtime {
    return status;
  }
  for helper in [
    "hyprland/sh/spaces-switch.sh",
    "hyprland/sh/borders-switch.sh",
  ] {
    if let Some(script) = user_copy::config_path(paths, helper).filter(|path| path.is_file())
      && !run_quiet("sh", &[script.to_string_lossy().as_ref(), "--apply"])
      && status == 0
    {
      status = 1;
    }
  }
  status
}

/// Applies the theme through the canonical configuration, unless a projection
/// is running (that projection already did it).
/// Returns `false` when the canonical commit failed. The switch then stops before
/// any service or file is touched, and the failure is reported to the caller.
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

/// The Waybar stylesheets and the rofi theme of the switch.
#[allow(clippy::too_many_arguments)]
fn apply_waybar(
  paths: &SessionPaths,
  theme: &str,
  fonts: &Fonts,
  rofi_config: &Path,
  rofi_theme_file: &Path,
  rofi_mode: &Path,
  rofi_themes: &Path,
) {
  for relative in [
    "taskbar/config/argvus-taskbar.jsonc",
    "taskbar/config/argvus-taskbar.css",
    "widget-telemetry/config/argvus-widget-telemetry.jsonc",
    "widget-telemetry/config/argvus-widget-telemetry.css",
  ] {
    let _ = waybar::refresh_managed(paths, relative);
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
  set_target_in_file(&taskbar_css, Target::WaybarThemeImport, theme);
  set_target_in_file(&telemetry_css, Target::WaybarTelemetryThemeImport, theme);

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

  set_target_in_file(
    &taskbar_json,
    Target::WaybarRofiDrun,
    &rofi_config.display().to_string(),
  );
  let rofi_theme_value = rofi_theme_file.display().to_string();
  set_target_in_file(rofi_config, Target::RofiTheme, &rofi_theme_value);
  set_target_in_file(rofi_config, Target::RofiFont, &fonts.apps);
  set_target_in_file(
    rofi_theme_file,
    Target::RofiImportTheme,
    &format!("{}/{theme}/theme.rasi", rofi_themes.display()),
  );
  set_target_in_file(
    rofi_theme_file,
    Target::RofiImportMode,
    &rofi_mode.display().to_string(),
  );
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

/// The notification theme of the switch, through the notifications library.
fn apply_dunst(paths: &SessionPaths, theme: &str) {
  let helper = paths.system_config("notifications/sh/theme.sh");
  if !helper.is_file() {
    return;
  }
  let dunstrc = path_or_empty(paths, "notifications/config/dunstrc");
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

/// The qt6ct color scheme and palette of the theme, when the theme has colors.
fn apply_qt6ct(paths: &SessionPaths, theme: &str, colors_dir: &Path) {
  let qt6ct = path_or_empty(paths, "appearance/config/qt6ct/qt6ct.conf");
  if !qt6ct.is_file() || !colors_dir.join(format!("{theme}.conf")).is_file() {
    return;
  }
  let colors = colors_dir.join(format!("{theme}.conf"));
  set_target_in_file(
    &qt6ct,
    Target::Qt6ctColorScheme,
    &colors.display().to_string(),
  );
  set_target_in_file(&qt6ct, Target::Qt6ctCustomPalette, "true");
}

/// The GTK mode of the theme: the stylesheets, the settings and, when running,
/// the live GTK settings.
fn apply_gtk(
  paths: &SessionPaths,
  themes_dir: &Path,
  theme: &str,
  icon: &str,
  font: &str,
  runtime: bool,
) {
  let mode = theme_resolve::gtk_mode(themes_dir, theme);
  let name = appearance_runtime::gtk_theme_name(themes_dir, theme)
    .unwrap_or("")
    .to_string();
  let light = mode == "light";
  let (file_mode, prefer, scheme, fallback) = if light {
    ("light", "0", "prefer-light", "Adwaita-dark")
  } else {
    ("dark", "1", "prefer-dark", "Adwaita")
  };
  appearance_runtime::apply_gtk_theme_files(
    paths,
    &GtkFiles {
      theme: theme.to_string(),
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

/// The font values of the switch, read from the generated fonts file with the
/// same fallbacks the shell used.
struct Fonts {
  apps: String,
  taskbar_family: String,
  taskbar_size: String,
  sysinfo_family: String,
  sysinfo_size: String,
  system_family: String,
  system_size: String,
}

impl Fonts {
  fn read(paths: &SessionPaths) -> Self {
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

/// The wallpaper of the theme: from its manifest, its built-in name, or the
/// wallpaper directory. `None` when the theme has none.
fn find_theme_wallpaper(paths: &SessionPaths, themes_dir: &Path, theme: &str) -> Option<String> {
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
  if let Some(path) = theme_resolve::theme_wallpaper(themes_dir, &root, theme) {
    return Some(path.display().to_string());
  }
  let backgrounds = env::var_os("ARGVUS_BACKGROUNDS_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/backgrounds"))
    .join("argvus");
  appearance_runtime::fallback_wallpaper(paths, &backgrounds, theme)
    .map(|path| path.display().to_string())
}

/// The workspace root of a development checkout, or empty.
fn workspace_root(paths: &SessionPaths) -> String {
  let source = paths.system_config.join("appearance/sh");
  let text = source.display().to_string();
  match text.strip_suffix("/src/usr/share/argvus/appearance/sh") {
    Some(appearance_root) => Path::new(appearance_root)
      .parent()
      .map(|parent| parent.display().to_string())
      .unwrap_or_default(),
    None => String::new(),
  }
}

/// The user copy of a directory that a theme needs, or the path it would have.
fn optional_parent(paths: &SessionPaths, relative: &str, theme: &str) -> PathBuf {
  theme_dirs::theme_parent(paths, relative, theme).unwrap_or_else(|| path_or_empty(paths, relative))
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
fn message(paths: &SessionPaths, key: &str, arguments: &[String]) {
  let _ = paths;
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

fn on_path(program: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
    .unwrap_or(false)
}

/// Runs a command and reports whether it succeeded. Its output is discarded.
fn run_quiet(program: &str, arguments: &[&str]) -> bool {
  Command::new(program)
    .args(arguments)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}
