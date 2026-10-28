//! Session preparation: the work that ran before the bars
//! and the wallpaper start.
//!
//! The order is the one the shell used. The theme itself is applied by the
//! `argvus theme-switch` route, and the spacing, borders, display and
//! icon helpers are still the shell scripts of their own domains. This module
//! orchestrates them and owns the state around them: the startup stamp, the
//! GTK settings, the monitor state migration and the session environment.

use crate::accent::DEFAULT_THEME;
use crate::json_pointers;
use crate::session_paths::{SessionPaths, font_default_value, font_state_value};
use crate::session_reload::{ReloadContext, active_theme, run_script, sync_user_files};
use crate::session_timing::timed;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Icon theme when the appearance helper is missing.
const DEFAULT_ICON_THEME: &str = "Argvus Dark Icons";

/// Message keys the shell printed through `argvus_tr`.
const MESSAGE_GTK_APPLIED: &str = "gtk.theme_applied";
const MESSAGE_GTK_FAILED: &str = "gtk.theme_failed";
const MESSAGE_GTK_MISSING: &str = "gtk.gsettings_missing";
const MESSAGE_BUTTONS_DISABLED: &str = "gtk.buttons_disabled";

/// Runs the whole session preparation. Every step is best effort, as the shell
/// steps were, so the result is always success.
pub fn prepare(paths: &SessionPaths, commands_dir: &Path, cache_home: &Path) -> i32 {
  timed("prepare.xdg-user-dirs", || {
    let _ = Command::new("xdg-user-dirs-update")
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
  });

  timed("prepare.monitor-state", || {
    migrate_monitor_state(paths, commands_dir)
  });

  let theme = crate::accent::effective_theme(&active_theme(paths));
  let theme_reapplied = timed("prepare.startup-theme", || {
    apply_startup_theme(paths, cache_home, &theme)
  });
  // Hyprland started on the theme files of the previous login. A reapplied
  // theme rewrote them, so the compositor reads them again. The login reload
  // service skips this case because it only follows the projection manifest.
  if theme_reapplied {
    timed("prepare.compositor-reload", || {
      let _ = crate::hyprland_ipc::reload();
    });
  }

  // GTK settings are kept by dconf between logins. They are written when the
  // startup theme was reapplied, which is when they can have changed, and not
  // on every stable login.
  if theme_reapplied {
    timed("prepare.gsettings", || set_gsettings(paths));
  }

  let context = ReloadContext {
    paths,
    commands_dir,
  };
  let mut environment: Vec<(String, String)> = Vec::new();
  if let Some(config) = timed("prepare.user-files", || sync_user_files(&context)) {
    environment.push(("YAZI_CONFIG_HOME".into(), config.display().to_string()));
  }

  let data = paths.config_home.join("argvus/data");
  let run_spaces = data.join(".active-theme").is_file() || data.join(".spaces").is_file();
  let run_borders = data.join(".active-theme").is_file() || data.join(".borders").is_file();
  // The two scripts write disjoint files with `--apply-static`: spaces edits
  // the bars' JSONC margins and the effective geometry, borders the bars' CSS
  // and the Rofi theme. Running them side by side halves this step.
  std::thread::scope(|scope| {
    if run_spaces {
      scope.spawn(|| {
        timed("prepare.spaces-switch", || {
          let _ = run_script(paths, "hyprland/sh/spaces-switch.sh", &environment);
        });
      });
    }
    if run_borders {
      timed("prepare.borders-switch", || {
        let _ = run_script(paths, "hyprland/sh/borders-switch.sh", &environment);
      });
    }
  });

  timed("prepare.display", || apply_display_at_prepare(&environment));

  let editor = timed("prepare.terminal-editor", || terminal_editor(paths));
  let path = augmented_path();
  let mut session_environment = environment;
  session_environment.extend([
    ("QT_QPA_PLATFORM".into(), "wayland".into()),
    ("QT_QPA_PLATFORMTHEME".into(), "qt6ct".into()),
    (
      "QT_QUICK_CONTROLS_STYLE".into(),
      "org.hyprland.style".into(),
    ),
    ("EDITOR".into(), editor.clone()),
    ("VISUAL".into(), editor.clone()),
    ("PATH".into(), path.clone()),
  ]);

  if on_path("hyprctl") {
    for (name, value) in [("EDITOR", &editor), ("VISUAL", &editor), ("PATH", &path)] {
      let _ = Command::new("hyprctl")
        .arg("setenv")
        .arg(format!("{name},{value}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
  }

  // PolicyKit and Quickshell read these through the systemd user environment,
  // which the session command imports from the environment it is given.
  timed("prepare.import-environment", || {
    let _ = Command::new(commands_dir.join("sessionctl"))
      .arg("import-environment")
      .envs(session_environment)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
  });
  0
}

/// Moves a legacy user monitor file to the generated location, and migrates
/// the legacy monitor file into the ARGVUS state once.
fn migrate_monitor_state(paths: &SessionPaths, commands_dir: &Path) {
  let legacy_user_lua = paths.config_home.join("argvus/hypr/monitors.lua");
  let generated_lua = paths
    .config_home
    .join("argvus/data/generated/hypr/monitors.lua");
  if legacy_user_lua.is_file() && !generated_lua.is_file() {
    if let Some(parent) = generated_lua.parent() {
      let _ = fs::create_dir_all(parent);
    }
    let _ = fs::rename(&legacy_user_lua, &generated_lua);
  }

  let legacy_monitors = paths.config_home.join("hypr/monitors.conf");
  let displayctl = commands_dir.join("displayctl");
  if legacy_monitors.is_file() && is_executable(&displayctl) {
    let state = paths.config_home.join("argvus/.monitors");
    let state_is_empty = fs::metadata(&state).map(|m| m.len() == 0).unwrap_or(true);
    if !state.is_file() || state_is_empty {
      let _ = Command::new(&displayctl)
        .arg("--migrate-nwg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
  }
}

/// Runs the accent change in startup mode, as the shell did when the user had
/// chosen an accent colour. It runs this same binary, so the rules are not
/// duplicated.
fn run_accent_startup() -> bool {
  let Ok(executable) = env::current_exe() else {
    return false;
  };
  Command::new(executable)
    .args(["config", "accent", "--startup"])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

/// Applies the startup theme, then the user's accent on top of it, unless the
/// stamp matches the current theme files and the user copies are already in
/// place. The stamp is written only when the theme switch succeeds.
///
/// When the stamp matches, the accent is skipped too: the files and the accent
/// marker (part of the fingerprint) are the ones the last login left.
///
/// Returns whether the theme switch ran.
fn apply_startup_theme(paths: &SessionPaths, cache_home: &Path, theme: &str) -> bool {
  let stamp = cache_home.join("session/theme-startup.stamp");
  let fingerprint = theme_fingerprint(paths, theme);
  let stamp_matches = fs::read_to_string(&stamp)
    .map(|text| text.trim_end_matches('\n') == fingerprint)
    .unwrap_or(false);
  if stamp.is_file() && stamp_matches && theme_materialized(paths, theme) {
    return false;
  }

  // The `argvus theme-switch` route resolves the implementation.
  // At startup the canonical configuration already holds the user's choice, so
  // this run only materializes the files. It must not commit the theme again,
  // because that would reset a custom wallpaper on every login.
  let status = Command::new("argvus")
    .arg("theme-switch")
    .arg(theme)
    .env("ARGVUS_NO_RUNTIME", "1")
    .env("ARGVUS_PROJECTING", "1")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();

  // The theme switch writes the theme's own accent; the user's choice goes back
  // on top of it.
  if accent_color_file(paths).is_file() {
    timed("prepare.accent", || {
      let _ = run_accent_startup();
    });
  }

  if matches!(status, Ok(status) if status.success()) {
    if let Some(parent) = stamp.parent() {
      let _ = fs::create_dir_all(parent);
    }
    // Fingerprint the files as the switch and the accent left them: a stamp
    // taken before them would never match again and repeat the switch on
    // every login.
    let _ = fs::write(&stamp, theme_fingerprint(paths, theme));
  }
  true
}

/// The marker of the accent the user chose, read by `config accent --startup`.
fn accent_color_file(paths: &SessionPaths) -> PathBuf {
  paths.config_home.join("argvus/data/.accent-color")
}

/// The text that identifies the theme files. It is stored in the stamp, so it
/// keeps the shell's order and `path:mtime:size` format.
pub fn theme_fingerprint(paths: &SessionPaths, theme: &str) -> String {
  let mut lines = vec![format!("theme={theme}")];
  for root in fingerprint_roots(paths, theme) {
    if root.is_dir() {
      let mut files = Vec::new();
      collect_files(&root, &mut files);
      // The shell sorted the stat lines, not the paths: the sort key is
      // the whole `path:mtime:size` text.
      let mut stats: Vec<String> = files
        .iter()
        .map(|file| stat_line(Path::new(file)))
        .collect();
      stats.sort();
      lines.extend(stats);
    } else if root.is_file() {
      lines.push(stat_line(&root));
    } else {
      lines.push(format!("{}:missing", root.display()));
    }
  }
  lines.join("\n")
}

/// The fingerprinted paths: the packaged files the switch reads, the theme
/// manifest, the registry, the layouts and every registered template.
fn fingerprint_roots(paths: &SessionPaths, theme: &str) -> Vec<PathBuf> {
  let theme_paths = crate::theme_build::theme_paths(paths);
  let system = |relative: &str| paths.system_config(relative);
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  let mut roots = vec![
    paths.read_config("hyprland/sh/spaces-switch.sh"),
    system("session/sh/paths.sh"),
    system("taskbar/config/argvus-taskbar.jsonc"),
    system("taskbar/config/argvus-taskbar.css"),
    system("widget-telemetry/config/argvus-widget-telemetry.jsonc"),
    system("widget-telemetry/config/argvus-widget-telemetry.css"),
    system("launcher/config/config.rasi"),
    system("launcher/config/theme.rasi"),
    theme_paths.system_themes().join(family),
    theme_paths.user_themes.join(family),
    theme_paths.registry(),
    theme_paths.layouts(),
  ];
  if let Ok(consumers) =
    argvus_theme_core::registry::load(&theme_paths.system_root, &theme_paths.registry())
  {
    roots.extend(consumers.into_iter().map(|consumer| consumer.template));
  }
  // The accent is reapplied only with the theme, so a new accent must
  // invalidate the stamp as well.
  roots.push(accent_color_file(paths));
  roots
}

fn collect_files(directory: &Path, files: &mut Vec<String>) {
  let Ok(entries) = fs::read_dir(directory) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    let Ok(file_type) = entry.file_type() else {
      continue;
    };
    if file_type.is_dir() {
      collect_files(&path, files);
    } else if file_type.is_file() {
      files.push(path.display().to_string());
    }
  }
}

fn stat_line(path: &Path) -> String {
  match fs::metadata(path) {
    Ok(metadata) => {
      let mtime = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
      format!("{}:{}:{}", path.display(), mtime, metadata.len())
    }
    Err(_) => format!("{}:missing", path.display()),
  }
}

/// Whether the files of the theme are in place: the build of the theme is
/// current and the user copies of the bars and the launcher exist, so a
/// matching stamp means the startup has nothing left to write.
///
/// The switch renders the theme with the canonical Sticky/Float variant, so the
/// build expected here is the id that variant resolves to (`argvus-dark-float`
/// on Float). Comparing the bare id never matches a Float build, and the switch
/// would then repeat on every login.
pub(crate) fn theme_materialized(paths: &SessionPaths, theme: &str) -> bool {
  let Some(built_theme) = built_theme(paths, theme) else {
    return false;
  };
  if !paths.data_home.join(".active-theme").is_file()
    || crate::theme_build::current_file_for(paths, &built_theme, "build.toml").is_none()
  {
    return false;
  }
  let required = [
    "taskbar/config/argvus-taskbar.jsonc",
    "taskbar/config/argvus-taskbar.css",
    "widget-telemetry/config/argvus-widget-telemetry.jsonc",
    "widget-telemetry/config/argvus-widget-telemetry.css",
    "launcher/config/config.rasi",
    "launcher/config/theme.rasi",
    "appearance/config/waybar/mode.css",
  ];
  for relative in required {
    if paths.system_config(relative).is_file() && !paths.user_config(relative).is_file() {
      return false;
    }
  }
  paths.user_config("gtk-4.0/settings.ini").is_file()
}

/// The theme id that `theme-switch` renders for `theme` under the canonical
/// Sticky/Float variant: `<id>-float` on Float, `<id>` on Sticky. The build and
/// the effects file are named by this id. `None` when the configuration cannot
/// be read, so the caller treats the outputs as missing.
pub(crate) fn built_theme(paths: &SessionPaths, theme: &str) -> Option<String> {
  let document = crate::ConfigStore::at(&paths.config_home).load().ok()?;
  let variant = document
    .get(json_pointers::LAYOUT_VARIANT)
    .and_then(serde_json::Value::as_str)
    .unwrap_or_default();
  Some(crate::theme_resolve::resolve_theme(theme, variant))
}

/// Sets the GTK interface through gsettings and reports the outcome in the
/// user's language, as the shell did.
fn set_gsettings(paths: &SessionPaths) {
  let theme = active_theme_name(paths);
  let light =
    crate::theme_resolve::gtk_mode(&crate::theme_build::theme_paths(paths), &theme) == "light";
  let gtk_theme = crate::appearance_runtime::gtk_theme_name(light);
  let scheme = if light { "prefer-light" } else { "prefer-dark" };
  let default_font = font_default_value(paths);
  let monospace_font = font_monospace_value(paths);
  let icon_theme = icon_theme(paths);

  if !on_path("gsettings") {
    message(MESSAGE_GTK_MISSING);
    return;
  }
  let interface = [
    ("icon-theme", icon_theme.as_str()),
    ("gtk-theme", gtk_theme),
    ("color-scheme", scheme),
    ("font-name", default_font.as_str()),
    ("document-font-name", default_font.as_str()),
    ("monospace-font-name", monospace_font.as_str()),
    ("cursor-theme", "Adwaita"),
  ];
  if interface
    .iter()
    .all(|(key, value)| gsettings_set("org.gnome.desktop.interface", key, value))
  {
    message(MESSAGE_GTK_APPLIED);
  } else {
    message(MESSAGE_GTK_FAILED);
  }
  // Only the minimize, maximize and close buttons stay in the title bar.
  if gsettings_set("org.gnome.desktop.wm.preferences", "button-layout", ":") {
    message(MESSAGE_BUTTONS_DISABLED);
  }
}

fn gsettings_set(schema: &str, key: &str, value: &str) -> bool {
  Command::new("gsettings")
    .args(["set", schema, key, value])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

/// The theme name the GTK settings follow. The renamed Catppuccin Latte
/// identifiers map back to the names GTK knows.
pub fn active_theme_name(paths: &SessionPaths) -> String {
  let file = paths.config_home.join("argvus/data/.active-theme");
  if !file.is_file() {
    return DEFAULT_THEME.to_string();
  }
  let first = fs::read_to_string(file)
    .ok()
    .and_then(|text| text.lines().next().map(str::to_string))
    .unwrap_or_default();
  match first.as_str() {
    "argvus-catppuccin-latte" | "argvus-light-catppuccin-latte" => "catppuccin-latte".into(),
    "argvus-catppuccin-latte-float" | "argvus-light-catppuccin-latte-float" => {
      "catppuccin-latte-float".into()
    }
    other => other.to_string(),
  }
}

/// The monospace font as `"<family> <size>"`.
pub fn font_monospace_value(paths: &SessionPaths) -> String {
  let fonts = paths.generated_fonts();
  let family = font_state_value(
    &fonts,
    "terminal_name",
    &font_state_value(&fonts, "monospace_name", "IBM Plex Mono"),
  );
  let size = font_state_value(
    &fonts,
    "terminal_size",
    &font_state_value(&fonts, "monospace_size", "13"),
  );
  format!("{family} {size}")
}

/// The icon theme from the appearance helper, resolved for the active theme
/// as the helper does by default. The helper is sourced, never executed, so
/// its entry point does not run.
fn icon_theme(paths: &SessionPaths) -> String {
  let helper = paths.system_config.join("appearance/sh/icon-switch.sh");
  if !helper.is_file() {
    return DEFAULT_ICON_THEME.into();
  }
  let output = Command::new("sh")
    .arg("-c")
    .arg(". \"$1\" && icon_theme_effective")
    .arg("sh")
    .arg(&helper)
    .stderr(Stdio::null())
    .output();
  match output {
    Ok(output) if output.status.success() => {
      let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
      if name.is_empty() {
        DEFAULT_ICON_THEME.into()
      } else {
        name
      }
    }
    _ => DEFAULT_ICON_THEME.into(),
  }
}

/// The editor the session exports: the default terminal editor, or `vim`.
fn terminal_editor(paths: &SessionPaths) -> String {
  let bootstrap_dir = std::env::var_os("BOOTSTRAP_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| paths.system_config.join("session/sh"));
  let helper = bootstrap_dir.join("get-default.sh");
  let output = if is_executable(&helper) {
    Command::new("sh")
      .arg(&helper)
      .arg("terminal_editor")
      .stderr(Stdio::null())
      .output()
      .ok()
  } else if on_path("get-default") {
    Command::new("get-default")
      .arg("terminal_editor")
      .stderr(Stdio::null())
      .output()
      .ok()
  } else {
    None
  };
  output
    .and_then(|output| {
      String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|line| !line.is_empty())
        .map(str::to_string)
    })
    .unwrap_or_else(|| "vim".into())
}

/// `PATH` with the user's development bin directories in front, when they
/// exist and are not already listed. The later directory ends up first.
fn augmented_path() -> String {
  let mut path = std::env::var("PATH").unwrap_or_default();
  let home = std::env::var("HOME").unwrap_or_default();
  for directory in [format!("{home}/.local/bin"), format!("{home}/.cargo/bin")] {
    if !Path::new(&directory).is_dir() {
      continue;
    }
    if !format!(":{path}:").contains(&format!(":{directory}:")) {
      path = format!("{directory}:{path}");
    }
  }
  path
}

/// Applies the saved monitor layout for preparation: the display command when
/// it is installed, and the monitor script otherwise. Errors are ignored.
fn apply_display_at_prepare(environment: &[(String, String)]) {
  crate::display::run_step("session-prepare", environment);
}

/// Prints a message the way `argvus_tr` did: through the translation command
/// when it is installed, otherwise the key itself.
fn message(key: &str) {
  if on_path("argvus") {
    let _ = Command::new("argvus")
      .args(["i18n", "get", "session", key])
      .stderr(Stdio::null())
      .status();
  } else {
    println!("{key}");
  }
}

fn on_path(program: &str) -> bool {
  std::env::var_os("PATH")
    .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
    .unwrap_or(false)
}

fn is_executable(path: &Path) -> bool {
  fs::metadata(path)
    .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn paths_under(root: &Path) -> SessionPaths {
    SessionPaths {
      config_home: root.join("config"),
      data_home: root.join("config/argvus/data"),
      system_config: root.join("system"),
    }
  }

  #[test]
  fn catppuccin_identifiers_map_to_their_canonical_id() {
    let root = std::env::temp_dir().join(format!("argvus-gtk-theme-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_under(&root);
    fs::create_dir_all(paths.data_home.clone()).expect("data dir");
    fs::write(
      paths.data_home.join(".active-theme"),
      "argvus-light-catppuccin-latte\n",
    )
    .expect("theme");
    assert_eq!(active_theme_name(&paths), "catppuccin-latte");
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn fingerprint_keeps_the_shell_order_and_marks_missing_paths() {
    let root = std::env::temp_dir().join(format!("argvus-fingerprint-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_under(&root);
    let css = paths
      .system_config
      .join("taskbar/config/argvus-taskbar.css");
    fs::create_dir_all(css.parent().expect("parent")).expect("system dir");
    fs::write(&css, "x").expect("file");

    let fingerprint = theme_fingerprint(&paths, "argvus-dark");
    let lines: Vec<&str> = fingerprint.lines().collect();
    assert_eq!(lines[0], "theme=argvus-dark");
    let css_index = lines
      .iter()
      .position(|line| line.starts_with(&format!("{}:", css.display())))
      .expect("present file is listed");
    let jsonc_index = lines
      .iter()
      .position(|line| line.ends_with("argvus-taskbar.jsonc:missing"))
      .expect("missing file is marked");
    assert!(
      jsonc_index < css_index,
      "jsonc is listed before css, as in the shell"
    );
    let _ = fs::remove_dir_all(root);
  }

  /// Lays out a materialized theme: the canonical layout variant, the active
  /// marker, the user GTK copy and a current build rendered for `build_variant`.
  fn materialize_theme(paths: &SessionPaths, layout_variant: &str, build_variant: &str) {
    let layout_directory = paths.config_home.join("argvus/config");
    fs::create_dir_all(&layout_directory).expect("layout dir");
    fs::write(
      layout_directory.join("layout.json"),
      format!("{{\"variant\": \"{layout_variant}\"}}"),
    )
    .expect("layout");
    fs::create_dir_all(&paths.data_home).expect("data dir");
    fs::write(paths.data_home.join(".active-theme"), "argvus-dark-float\n").expect("active");
    let current = paths.data_home.join("generated/theme/current");
    fs::create_dir_all(&current).expect("build dir");
    fs::write(
      current.join("build.toml"),
      format!("theme = \"argvus-dark\"\nvariant = \"{build_variant}\"\n"),
    )
    .expect("build");
    let settings = paths.user_config("gtk-4.0/settings.ini");
    fs::create_dir_all(settings.parent().expect("parent")).expect("gtk dir");
    fs::write(settings, "[Settings]\n").expect("settings");
  }

  #[test]
  fn float_build_is_materialized_for_the_float_layout() {
    let root = std::env::temp_dir().join(format!("argvus-materialized-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_under(&root);
    materialize_theme(&paths, "float", "float");
    assert!(theme_materialized(&paths, "argvus-dark"));
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn built_theme_follows_the_canonical_layout_variant() {
    let root = std::env::temp_dir().join(format!("argvus-built-theme-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_under(&root);
    materialize_theme(&paths, "float", "float");
    assert_eq!(
      built_theme(&paths, "argvus-dark").as_deref(),
      Some("argvus-dark-float")
    );
    materialize_theme(&paths, "sticky", "sticky");
    assert_eq!(
      built_theme(&paths, "argvus-dark").as_deref(),
      Some("argvus-dark")
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn float_build_is_not_materialized_for_the_sticky_layout() {
    let root = std::env::temp_dir().join(format!("argvus-sticky-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_under(&root);
    materialize_theme(&paths, "sticky", "float");
    assert!(!theme_materialized(&paths, "argvus-dark"));
    let _ = fs::remove_dir_all(root);
  }
}
