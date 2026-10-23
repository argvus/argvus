//! The `argvus-config` command set. The `argvus` dispatcher and the `argvus-config`
//! binary both call [`run_command`].

use crate::{
  ConfigDocument, ConfigScope, ConfigStore, appearance_runtime, css_edit, session_loading,
  session_paths, session_prepare, session_reload, session_splash, session_units,
  text_edit::{self, replace_file},
  theme_apply, theme_dirs, theme_resolve, theme_surfaces, theme_transition, user_copy,
};
use crate::{ConfigError, ConfigResult};
use crate::{accent, accent_switch, display, effects, monitor_state, theme_files, waybar};
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Runs one `argvus-config` command. `arguments` are the words after the program name.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  if arguments.first().map(String::as_str) == Some("session-restart") {
    return session_restart(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("session-reload-config") {
    return session_reload_config(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("session-sync-config") {
    return session_sync_config();
  }
  if arguments.first().map(String::as_str) == Some("effects") {
    return ExitCode::from(effects::run(&arguments[1..]) as u8);
  }
  if arguments.first().map(String::as_str) == Some("gtk-theme-name") {
    return gtk_theme_name(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("appearance-hypr") {
    return appearance_hypr(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("appearance-tail") {
    return appearance_tail(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("waybar-refresh") {
    return waybar_refresh(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("waybar-layout") {
    return waybar_layout(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("waybar-font") {
    return waybar_font(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("accent") {
    return accent_switch::run(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("display") {
    return ExitCode::from(display::run(&arguments[1..]) as u8);
  }
  if arguments.first().map(String::as_str) == Some("monitor-set") {
    return monitor_command(&arguments[1..], |store, args| {
      let [monitor, key, value] = args else {
        return Err(ConfigError::Invalid(
          "usage: argvus config monitor-set MONITOR KEY VALUE".into(),
        ));
      };
      monitor_state::set(store, monitor, key, value)
    });
  }
  if arguments.first().map(String::as_str) == Some("monitor-import") {
    return monitor_command(&arguments[1..], |store, args| {
      let [file] = args else {
        return Err(ConfigError::Invalid(
          "usage: argvus config monitor-import STATE_FILE".into(),
        ));
      };
      monitor_state::import(store, Path::new(file))
    });
  }
  if arguments.first().map(String::as_str) == Some("monitor-primary") {
    return monitor_command(&arguments[1..], |store, args| {
      let [monitor] = args else {
        return Err(ConfigError::Invalid(
          "usage: argvus config monitor-primary MONITOR".into(),
        ));
      };
      monitor_state::primary(store, monitor)
    });
  }
  if arguments.first().map(String::as_str) == Some("monitor-workspaces") {
    return monitor_command(&arguments[1..], |store, args| {
      let [monitor, ids] = args else {
        return Err(ConfigError::Invalid(
          "usage: argvus config monitor-workspaces MONITOR IDS".into(),
        ));
      };
      monitor_state::workspaces(store, monitor, ids)
    });
  }
  if arguments.first().map(String::as_str) == Some("monitor-remove") {
    return monitor_command(&arguments[1..], |store, args| {
      let [monitor] = args else {
        return Err(ConfigError::Invalid(
          "usage: argvus config monitor-remove MONITOR".into(),
        ));
      };
      monitor_state::remove(store, monitor)
    });
  }
  if arguments.first().map(String::as_str) == Some("monitor-sync") {
    return monitor_command(&arguments[1..], |store, args| {
      let [file] = args else {
        return Err(ConfigError::Invalid(
          "usage: argvus config monitor-sync STATE_FILE".into(),
        ));
      };
      monitor_state::sync_legacy(store, Path::new(file))
    });
  }
  if arguments.first().map(String::as_str) == Some("read-path") {
    return read_path(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("mutable-path") {
    return mutable_path(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("storage-theme") {
    return storage_theme(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("calendar-theme") {
    return calendar_theme(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("greeter-publish") {
    return greeter_publish(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("gtk-files") {
    return gtk_files(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("wallpaper-fallback") {
    return wallpaper_fallback(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("gtk-runtime") {
    return gtk_runtime(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("wallpaper-apply") {
    return wallpaper_apply(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("transition-units") {
    return transition_units(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("transition-overlay") {
    return transition_overlay(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("theme-parent") {
    return theme_parent(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("theme-apply") {
    let theme = arguments.get(1).map(String::as_str).unwrap_or("");
    let Some(paths) = session_paths::SessionPaths::from_environment() else {
      eprintln!("argvus-config: cannot resolve the configuration directory");
      return ExitCode::from(1);
    };
    let variant = match layout_variant() {
      Ok(variant) => variant,
      Err(error) => {
        eprintln!("argvus-config: cannot read the layout mode: {error}");
        return ExitCode::from(1);
      }
    };
    return ExitCode::from(theme_apply::run(&paths, theme, &variant) as u8);
  }
  // Checked before the `theme-` prefix below, which routes theme queries only.
  if arguments.first().map(String::as_str) == Some("theme-file") {
    return theme_file(&arguments[1..]);
  }
  if let Some(command) = arguments.first().map(String::as_str)
    && command.starts_with("theme-")
  {
    return theme_command(command, &arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("jsonc-margin") {
    return jsonc_margin(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("setting") {
    return setting(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("css-block") {
    return css_block(&arguments[1..]);
  }
  if arguments.first().map(String::as_str) == Some("session-prepare") {
    return session_prepare();
  }
  if arguments.first().map(String::as_str) == Some("session-loading") {
    return session_loading();
  }
  match run(arguments) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      ExitCode::from(1)
    }
  }
}

fn run(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
  // Runtime actions talk to the compositor only, so they run without
  // loading the configuration store.
  if arguments.first().map(String::as_str) == Some("hyprland") {
    return hyprland_command(&arguments[1..]);
  }
  let store = ConfigStore::from_environment()?;
  let command = arguments.first().map(String::as_str).unwrap_or("help");
  match command {
    "path" => println!("{}", store.path.display()),
    "validate" => {
      store.load()?.validate()?;
      println!("valid");
    }
    "ensure" | "init" => {
      store.ensure()?;
      println!("ready");
    }
    "migrate" => {
      store.migrate_legacy()?;
      println!("migrated");
    }
    "session-plan" => print_session_plan(&store)?,
    "session-splash" => match arguments.get(1).map(String::as_str) {
      Some("start") => {
        let theme = effective_theme(&store)?;
        let environment = session_splash::SplashEnvironment::resolve();
        // Printed only when this call started the splash; the caller
        // then owns the overlay and stops it later.
        if session_splash::start(&environment, &theme) {
          println!("started");
        }
      }
      Some("stop") => {
        session_splash::stop(
          &session_splash::SplashEnvironment::resolve(),
          session_splash::Owner::Session,
        );
      }
      _ => return Err("usage: argvus config session-splash start|stop".into()),
    },
    "session-telemetry" => {
      let enabled = session_units::telemetry_enabled(&cache_home()?);
      println!("{}", if enabled { "enabled" } else { "disabled" });
    }
    "audio" => crate::audio::run(&store, &arguments[1..])?,
    "get" => get_value(&store, &arguments[1..])?,
    "set" => set_value(&store, &arguments[1..])?,
    "patch" => patch_value(&store, &arguments[1..])?,
    "apply-theme" => apply_theme(&store, &arguments[1..])?,
    "apply-mode" => apply_mode(&store, &arguments[1..])?,
    "unset" => {
      let pointer = arguments.get(1).ok_or("usage: argvus config unset /path")?;
      store.unset(pointer)?;
    }
    "export" => export_scope(&store, &arguments[1..])?,
    "import" => import_scope(&store, &arguments[1..])?,
    "project" => {
      let only = arguments
        .iter()
        .position(|argument| argument == "--only")
        .and_then(|index| arguments.get(index + 1))
        .map(String::as_str);
      let message = match only {
        Some("keyboard_shortcuts") => crate::project::project_shortcuts_only(&store)?,
        Some(section) => {
          return Err(format!("unsupported projection section: {section}").into());
        }
        None => crate::project::project(
          &store,
          arguments.iter().any(|argument| argument == "--force"),
        )?,
      };
      println!("{message}");
    }
    "help" | "--help" | "-h" => print_help(),
    other => return Err(format!("unknown command: {other}").into()),
  }
  Ok(())
}

fn get_value(store: &ConfigStore, arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
  let pointer = arguments
    .first()
    .ok_or("usage: argvus config get /path [--effective]")?;
  let effective = arguments.iter().any(|argument| argument == "--effective");
  let document = if effective {
    store.load_effective()?
  } else {
    store.load()?
  };
  let value = document.get(pointer).cloned().unwrap_or(Value::Null);
  if arguments.iter().any(|argument| argument == "--raw") {
    match value {
      Value::String(value) => println!("{value}"),
      other => println!("{other}"),
    }
  } else {
    println!(
      "{}",
      serde_json::to_string_pretty(&crate::sorted_value(&value))?
    );
  }
  Ok(())
}

fn set_value(store: &ConfigStore, arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
  let pointer = arguments
    .first()
    .ok_or("usage: argvus config set /path <json-value>")?;
  let raw_value = arguments
    .get(1)
    .ok_or("usage: argvus config set /path <json-value>")?;
  let value = serde_json::from_str(raw_value).unwrap_or_else(|_| Value::String(raw_value.clone()));
  store.update(pointer, value)?;
  Ok(())
}

fn patch_value(
  store: &ConfigStore,
  arguments: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
  let input = arguments
    .first()
    .ok_or("usage: argvus config patch FILE|- [JSON]")?;
  let text = if input == "-" {
    std::io::read_to_string(std::io::stdin())?
  } else if input.starts_with('{') {
    input.clone()
  } else {
    fs::read_to_string(input)?
  };
  let values: Map<String, Value> = serde_json::from_str(&text)?;
  store.patch(&values)?;
  Ok(())
}

fn apply_theme(
  store: &ConfigStore,
  arguments: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
  let theme = arguments.first().ok_or("usage: argvus config apply-theme THEME [--accent HEX] [--gtk-mode MODE] [--wallpaper PATH] [--reset-wallpaper] [--reset-accent]")?;
  let option = |name: &str| {
    arguments
      .iter()
      .position(|value| value == name)
      .and_then(|index| arguments.get(index + 1))
  };
  // The theme's own defaults come from its manifest (themes.d/<family>/theme.toml),
  // the same source the accent and GTK mode fallbacks use everywhere else.
  if !theme_resolve::is_valid_theme_id(theme) {
    return Err(format!("invalid theme id '{theme}'").into());
  }
  let themes_dir = system_config_dir().join("appearance/themes.d");
  let default_accent = accent::theme_default_accent(&themes_dir, theme);
  let default_mode = theme_resolve::gtk_mode(&themes_dir, theme);
  let accent = option("--accent")
    .map(String::as_str)
    .or(default_accent.as_deref());
  let gtk_mode = option("--gtk-mode")
    .map(String::as_str)
    .or(Some(default_mode));
  let flag = |name: &str| arguments.iter().any(|argument| argument == name);
  store.apply_theme(
    theme,
    accent,
    gtk_mode,
    option("--wallpaper").map(String::as_str),
    flag("--reset-wallpaper"),
    flag("--reset-accent"),
  )?;
  Ok(())
}

fn apply_mode(store: &ConfigStore, arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
  let variant = arguments
    .first()
    .ok_or("usage: argvus config apply-mode <sticky|float>")?;
  store.apply_layout_variant(variant)?;
  Ok(())
}

fn system_config_dir() -> PathBuf {
  env::var_os("ARGVUS_SYSTEM_CONFIG")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
}

fn export_scope(
  store: &ConfigStore,
  arguments: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
  let scope_name = arguments
    .iter()
    .position(|argument| argument == "--scope")
    .and_then(|index| arguments.get(index + 1))
    .map(String::as_str)
    .unwrap_or("desktop");
  let scope = ConfigScope::parse(scope_name).ok_or("invalid scope")?;
  let document = store.load()?.scoped(scope);
  let output = arguments
    .iter()
    .position(|argument| argument == "--output")
    .and_then(|index| arguments.get(index + 1));
  let bytes = serde_json::to_vec_pretty(&document.sorted_sections())?;
  if let Some(output) = output {
    fs::write(
      Path::new(output),
      format!("{}\n", String::from_utf8(bytes)?),
    )?;
  } else {
    println!("{}", String::from_utf8(bytes)?);
  }
  Ok(())
}

fn import_scope(
  store: &ConfigStore,
  arguments: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
  let scope_name = arguments
    .iter()
    .position(|argument| argument == "--scope")
    .and_then(|index| arguments.get(index + 1))
    .map(String::as_str)
    .unwrap_or("desktop");
  let scope = ConfigScope::parse(scope_name).ok_or("invalid scope")?;
  let input = arguments
    .iter()
    .rev()
    .find(|argument| !argument.starts_with('-'))
    .ok_or("missing profile")?;
  let imported: ConfigDocument = serde_json::from_str(&fs::read_to_string(input)?)?;
  imported.validate()?;
  store.modify(|current| {
    for section in scope.sections() {
      if let Some(value) = imported.sections.get(*section) {
        current
          .sections
          .insert((*section).to_owned(), value.clone());
      }
    }
    current.validate()
  })?;
  Ok(())
}

/// Runs one runtime action on the compositor. Every value is validated by
/// `hyprland_ipc` before it reaches the Lua snippet that Hyprland evaluates.
fn hyprland_command(arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
  use crate::hyprland_ipc;
  let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
  // A warning means the compositor ran the action with nothing to act on.
  // It is not a failure, so it is printed and the command still succeeds.
  let warning = match words.as_slice() {
    ["reload"] => hyprland_ipc::reload()?,
    ["monitor", line] => {
      let monitor = hyprland_ipc::parse_monitor_line(line)?;
      hyprland_ipc::eval(&hyprland_ipc::monitor_lua(&monitor))?
    }
    ["dpms", state] => {
      let on = parse_dpms_state(state)?;
      hyprland_ipc::eval(&hyprland_ipc::dpms_lua(None, on)?)?
    }
    ["dpms", output, state] => {
      let on = parse_dpms_state(state)?;
      hyprland_ipc::eval(&hyprland_ipc::dpms_lua(Some(output), on)?)?
    }
    ["focus-class", class] => hyprland_ipc::eval(&hyprland_ipc::focus_class_lua(class)?)?,
    ["exit"] => hyprland_ipc::eval(hyprland_ipc::exit_lua())?,
    ["keyboard-layout", layouts] => {
      hyprland_ipc::eval(&hyprland_ipc::keyboard_layout_lua(layouts)?)?
    }
    _ => {
      return Err(
                "usage: argvus config hyprland reload|monitor LINE|dpms [OUTPUT] on|off|focus-class CLASS|keyboard-layout LAYOUTS|exit"
                    .into(),
            );
    }
  };
  if let Some(warning) = warning {
    eprintln!("argvus-config: {warning}");
  }
  Ok(())
}

/// Accepts exactly `on` or `off`, so a typo cannot turn monitors off.
fn parse_dpms_state(state: &str) -> Result<bool, Box<dyn std::error::Error>> {
  match state {
    "on" => Ok(true),
    "off" => Ok(false),
    other => Err(format!("dpms state must be on or off, got {other}").into()),
  }
}

/// The effective theme name, falling back to the ARGVUS default.
fn effective_theme(store: &ConfigStore) -> Result<String, Box<dyn std::error::Error>> {
  let document = store.load_effective()?;
  Ok(
    document
      .get("/appearance/theme")
      .and_then(Value::as_str)
      .filter(|theme| !theme.is_empty())
      .unwrap_or(session_splash::DEFAULT_THEME)
      .to_string(),
  )
}

/// The ARGVUS cache directory, resolved as the session scripts resolve it.
fn cache_home() -> Result<PathBuf, Box<dyn std::error::Error>> {
  if let Some(cache) = env::var_os("ARGVUS_CACHE_HOME") {
    return Ok(PathBuf::from(cache));
  }
  if let Some(cache) = env::var_os("XDG_CACHE_HOME") {
    return Ok(PathBuf::from(cache).join("argvus"));
  }
  let home = env::var_os("HOME").ok_or("HOME is not set")?;
  Ok(PathBuf::from(home).join(".cache/argvus"))
}

/// `theme-parent RELATIVE THEME`: prints the user directory that holds the
/// theme's files, after filling the missing ones. Exits 1 when no copy exists.
fn theme_parent(arguments: &[String]) -> ExitCode {
  let [relative, theme] = arguments else {
    eprintln!("usage: argvus config theme-parent RELATIVE THEME");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  match theme_dirs::theme_parent(&paths, relative, theme) {
    Some(parent) => {
      println!("{}", parent.display());
      ExitCode::SUCCESS
    }
    None => ExitCode::from(1),
  }
}

/// `gtk-theme-name THEME`: the GTK theme the theme uses. Exits 1 for an unknown theme.
fn gtk_theme_name(arguments: &[String]) -> ExitCode {
  let [theme] = arguments else {
    eprintln!("usage: argvus config gtk-theme-name THEME");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    return ExitCode::from(1);
  };
  let themes_dir = paths.system_config("appearance/themes.d");
  match appearance_runtime::gtk_theme_name(&themes_dir, theme) {
    Some(name) => {
      println!("{name}");
      ExitCode::SUCCESS
    }
    None => ExitCode::from(1),
  }
}

/// `appearance-hypr THEME SYSTEM_FAMILY SYSTEM_SIZE ICON`: the Hyprland toolkit,
/// its application style and the qt6ct icon theme of the theme.
fn appearance_hypr(arguments: &[String]) -> ExitCode {
  let [theme, family, size, icon] = arguments else {
    eprintln!("usage: argvus config appearance-hypr THEME SYSTEM_FAMILY SYSTEM_SIZE ICON");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  let hypr_themes = paths.system_config("appearance/config/hypr/themes");
  let hypr_themes =
    theme_dirs::theme_parent(&paths, "appearance/config/hypr/themes", theme).unwrap_or(hypr_themes);
  match theme_files::apply_hypr_stage(&paths, &hypr_themes, theme, family, size, icon) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: appearance: {error}");
      ExitCode::from(1)
    }
  }
}

/// `appearance-tail THEME`: yazi, superfile and the mode stylesheet.
/// Prints `missing=PATH` and exits 0 when the yazi flavor is not installed; the
/// shell reports that in the user's language.
fn appearance_tail(arguments: &[String]) -> ExitCode {
  let [theme] = arguments else {
    eprintln!("usage: argvus config appearance-tail THEME");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  match theme_files::apply_tail_stage(&paths, theme) {
    Ok(Some(missing)) => {
      println!("missing={}", missing.display());
      ExitCode::SUCCESS
    }
    Ok(None) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: appearance: {error}");
      ExitCode::from(1)
    }
  }
}

/// `waybar-refresh RELATIVE`: copies the packaged Waybar file over its managed copy.
fn waybar_refresh(arguments: &[String]) -> ExitCode {
  let [relative] = arguments else {
    eprintln!("usage: argvus config waybar-refresh RELATIVE");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  match waybar::refresh_managed(&paths, relative) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {relative}: {error}");
      ExitCode::from(1)
    }
  }
}

/// `waybar-layout THEME`: margins, corner radii and the taskbar border of the theme class.
fn waybar_layout(arguments: &[String]) -> ExitCode {
  let [theme] = arguments else {
    eprintln!("usage: argvus config waybar-layout THEME");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  match waybar::apply_layout(&paths, theme) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: waybar layout: {error}");
      ExitCode::from(1)
    }
  }
}

/// `waybar-font taskbar|telemetry BLOCK`: writes the font block of one Waybar stylesheet.
fn waybar_font(arguments: &[String]) -> ExitCode {
  let [target, block] = arguments else {
    eprintln!("usage: argvus config waybar-font taskbar|telemetry BLOCK");
    return ExitCode::from(64);
  };
  let relative = match target.as_str() {
    "taskbar" => "taskbar/config/argvus-taskbar.css",
    "telemetry" => "widget-telemetry/config/argvus-widget-telemetry.css",
    _ => {
      eprintln!("argvus-config: unknown waybar stylesheet: {target}");
      return ExitCode::from(64);
    }
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  let Some(path) = user_copy::config_path(&paths, relative) else {
    return ExitCode::SUCCESS;
  };
  match waybar::write_font_block(&path, block) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {}: {error}", path.display());
      ExitCode::from(1)
    }
  }
}

/// `mutable-path RELATIVE`: prints the user copy of a packaged file, creating it
/// first when needed. The shell helper `paths_ensure_generated_copy` delegates here.
fn mutable_path(arguments: &[String]) -> ExitCode {
  let [relative] = arguments else {
    eprintln!("usage: argvus config mutable-path RELATIVE");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  match user_copy::ensure_user_copy(&paths, relative) {
    Ok(path) => {
      println!("{}", path.display());
      ExitCode::SUCCESS
    }
    Err(error) => {
      eprintln!("argvus-config: {relative}: {error}");
      ExitCode::from(1)
    }
  }
}

/// `storage-theme THEME WORKSPACE_ROOT USER_DIR DESTINATION`: the removable-devices
/// panel theme. The shell resolves the user directory and the destination in
/// its mutable config mode; an empty workspace means none. A failed copy is
/// reported and the switch continues, as it always did.
fn storage_theme(arguments: &[String]) -> ExitCode {
  let [theme, workspace, user_dir, destination] = arguments else {
    eprintln!("usage: argvus config storage-theme THEME WORKSPACE_ROOT USER_DIR DESTINATION");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::SUCCESS;
  };
  let themes_dir = paths.system_config("appearance/themes.d");
  let system_dir = paths.system_config("removable-devices/config/themes");
  let workspace_root = (!workspace.is_empty()).then(|| PathBuf::from(workspace));
  let locations = theme_surfaces::StorageLocations {
    user_dir: Path::new(user_dir),
    system_dir: &system_dir,
    destination: Path::new(destination),
    workspace_root: workspace_root.as_deref(),
  };
  if let Err(error) = theme_surfaces::apply_storage_theme(&locations, &themes_dir, theme) {
    eprintln!("argvus-config: removable-devices theme: {error}");
  }
  ExitCode::SUCCESS
}

/// `calendar-theme THEME [WORKSPACE_ROOT]`: the taskbar calendar stylesheet.
fn calendar_theme(arguments: &[String]) -> ExitCode {
  let Some((theme, workspace)) = theme_and_workspace(arguments, "calendar-theme") else {
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::SUCCESS;
  };
  let cache_home = env::var_os("XDG_CACHE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"));
  let cache_file = cache_home.join("argvus-taskbar-calendar/theme.css");
  let themes_dir = paths.system_config("appearance/themes.d");
  if let Err(error) = theme_surfaces::apply_calendar_theme(
    &paths,
    &themes_dir,
    &cache_file,
    theme,
    workspace.as_deref(),
  ) {
    eprintln!("argvus-config: calendar theme: {error}");
  }
  ExitCode::SUCCESS
}

/// `greeter-publish THEME`: publishes the theme id the greeter may draw.
fn greeter_publish(arguments: &[String]) -> ExitCode {
  let [theme] = arguments else {
    eprintln!("usage: argvus config greeter-publish THEME");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    return ExitCode::SUCCESS;
  };
  let state_dir = env::var_os("ARGVUS_GREETER_THEME_STATE_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/var/lib/argvus/greeter/themes"));
  let themes_dir = paths.system_config("appearance/themes.d");
  if let Err(error) = theme_surfaces::publish_greeter_theme(
    &themes_dir,
    &state_dir,
    theme,
    theme_surfaces::current_user_id(),
  ) {
    eprintln!("argvus-config: greeter theme: {error}");
  }
  ExitCode::SUCCESS
}

/// Splits `THEME [WORKSPACE_ROOT]`. An empty workspace means none.
fn theme_and_workspace<'a>(
  arguments: &'a [String],
  command: &str,
) -> Option<(&'a str, Option<PathBuf>)> {
  match arguments {
    [theme] => Some((theme.as_str(), None)),
    [theme, workspace] => Some((
      theme.as_str(),
      (!workspace.is_empty()).then(|| PathBuf::from(workspace)),
    )),
    _ => {
      eprintln!("usage: argvus config {command} THEME [WORKSPACE_ROOT]");
      None
    }
  }
}

/// `gtk-files THEME MODE NAME PREFER_DARK ICON FONT`: the GTK stylesheets,
/// settings and mode marker of a theme.
fn gtk_files(arguments: &[String]) -> ExitCode {
  let [theme, mode, gtk_theme_name, prefer_dark, icon_theme, font] = arguments else {
    eprintln!("usage: argvus config gtk-files THEME MODE NAME PREFER_DARK ICON FONT");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  appearance_runtime::apply_gtk_theme_files(
    &paths,
    &appearance_runtime::GtkFiles {
      theme: theme.clone(),
      mode: mode.clone(),
      gtk_theme_name: gtk_theme_name.clone(),
      prefer_dark: prefer_dark.clone(),
      icon_theme: icon_theme.clone(),
      font: font.clone(),
    },
  );
  ExitCode::SUCCESS
}

/// `wallpaper-fallback THEME`: the theme wallpaper found without its manifest.
fn wallpaper_fallback(arguments: &[String]) -> ExitCode {
  let [theme] = arguments else {
    eprintln!("usage: argvus config wallpaper-fallback THEME");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  let backgrounds_base = env::var_os("ARGVUS_BACKGROUNDS_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/backgrounds"));
  match appearance_runtime::fallback_wallpaper(&paths, &backgrounds_base.join("argvus"), theme) {
    Some(wallpaper) => {
      println!("{}", wallpaper.display());
      ExitCode::SUCCESS
    }
    None => ExitCode::from(1),
  }
}

/// `gtk-runtime SCHEME THEME FALLBACK FONT ICON`: the GTK settings of a theme,
/// applied through gsettings. See `appearance_runtime::apply_gtk_runtime`.
fn gtk_runtime(arguments: &[String]) -> ExitCode {
  let [scheme, theme_name, fallback_theme, font, icon_theme] = arguments else {
    eprintln!("usage: argvus config gtk-runtime SCHEME THEME FALLBACK FONT ICON");
    return ExitCode::from(64);
  };
  appearance_runtime::apply_gtk_runtime(&appearance_runtime::GtkRuntime {
    scheme: scheme.clone(),
    theme_name: theme_name.clone(),
    fallback_theme: fallback_theme.clone(),
    font: font.clone(),
    icon_theme: icon_theme.clone(),
  });
  ExitCode::SUCCESS
}

/// `wallpaper-apply [--runtime] HYPRPAPER_FILE WALLPAPER`: points hyprpaper at the
/// wallpaper and, with `--runtime`, publishes it to the session. The shell
/// resolves the hyprpaper file in its mutable config mode.
fn wallpaper_apply(arguments: &[String]) -> ExitCode {
  let (runtime, hyprpaper_file, wallpaper) = match arguments {
    [flag, file, wallpaper] if flag == "--runtime" => (true, file, wallpaper),
    [file, wallpaper] => (false, file, wallpaper),
    _ => {
      eprintln!("usage: argvus config wallpaper-apply [--runtime] HYPRPAPER_FILE WALLPAPER");
      return ExitCode::from(64);
    }
  };
  appearance_runtime::apply_wallpaper(Path::new(hyprpaper_file), wallpaper, runtime);
  ExitCode::SUCCESS
}

/// `transition-units record|stop|restart [UNITS]`: the services a theme switch
/// stops and restarts. `record` prints the active ones, separated by blanks.
fn transition_units(arguments: &[String]) -> ExitCode {
  match arguments.first().map(String::as_str) {
    Some("record") => {
      println!("{}", theme_transition::active_units().join(" "));
      ExitCode::SUCCESS
    }
    Some("stop") => {
      theme_transition::stop_units();
      ExitCode::SUCCESS
    }
    Some("restart") => match theme_transition::restart_units(&arguments[1..].join(" ")) {
      Ok(()) => ExitCode::SUCCESS,
      Err(theme_transition::UnknownUnit(unit)) => {
        eprintln!("argvus-config: unknown theme transition unit: {unit}");
        ExitCode::from(64)
      }
    },
    _ => {
      eprintln!("usage: argvus config transition-units record|stop|restart [UNITS]");
      ExitCode::from(64)
    }
  }
}

/// `transition-overlay start THEME|stop`: the loading overlay of a theme switch.
/// `start` prints `started` only when this call started the overlay, so the
/// caller stops only what it started and never a session splash.
fn transition_overlay(arguments: &[String]) -> ExitCode {
  match arguments {
    [action, theme] if action.as_str() == "start" => {
      let outcome = theme_transition::start_overlay(theme);
      if outcome == session_splash::StartOutcome::Started {
        println!("started");
      }
      if let Some(message) = theme_transition::overlay_warning(outcome) {
        eprintln!("{message}");
      }
      ExitCode::SUCCESS
    }
    [action] if action.as_str() == "stop" => {
      theme_transition::stop_overlay();
      ExitCode::SUCCESS
    }
    _ => {
      eprintln!("usage: argvus config transition-overlay start THEME|stop");
      ExitCode::from(64)
    }
  }
}

/// Sets one named line of an appearance file: `theme-file TARGET FILE VALUE`.
/// The targets are listed in `text_edit::Target`.
fn theme_file(arguments: &[String]) -> ExitCode {
  let [target, file, value] = arguments else {
    eprintln!("usage: argvus config theme-file TARGET FILE VALUE");
    return ExitCode::from(64);
  };
  let Some(target) = text_edit::Target::parse(target) else {
    eprintln!("argvus-config: unknown theme file target: {target}");
    return ExitCode::from(64);
  };
  let path = PathBuf::from(file);
  let result = fs::read_to_string(&path)
    .and_then(|text| replace_file(&path, &text_edit::set_target(&text, target, value)));
  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {}: {error}", path.display());
      ExitCode::from(1)
    }
  }
}

/// Sets a waybar margin in a JSONC file: `jsonc-margin [--signed] FILE EDGE VALUE`.
fn jsonc_margin(arguments: &[String]) -> ExitCode {
  let (signed, rest) = match arguments.first().map(String::as_str) {
    Some("--signed") => (true, &arguments[1..]),
    _ => (false, arguments),
  };
  let [file, edge, value] = rest else {
    eprintln!("usage: argvus config jsonc-margin [--signed] FILE top|left|right|bottom VALUE");
    return ExitCode::from(64);
  };
  let (Ok(value), true) = (
    value.parse::<u32>(),
    matches!(edge.as_str(), "top" | "left" | "right" | "bottom"),
  ) else {
    eprintln!("argvus-config: margin edge must be top|left|right|bottom and value a whole number");
    return ExitCode::from(64);
  };
  let path = PathBuf::from(file);
  let result = fs::read_to_string(&path).and_then(|text| {
    replace_file(
      &path,
      &text_edit::set_jsonc_margin(&text, edge, value, signed),
    )
  });
  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {}: {error}", path.display());
      ExitCode::from(1)
    }
  }
}

/// Sets one `key = value` line: `setting [--ini] FILE KEY VALUE`. A plain file
/// that does not exist is left alone; an INI file is created with its
/// `[Settings]` header. The file is replaced atomically and keeps its mode.
fn setting(arguments: &[String]) -> ExitCode {
  let (ini, rest) = match arguments.first().map(String::as_str) {
    Some("--ini") => (true, &arguments[1..]),
    _ => (false, arguments),
  };
  let [file, key, value] = rest else {
    eprintln!("usage: argvus config setting [--ini] FILE KEY VALUE");
    return ExitCode::from(64);
  };
  let path = PathBuf::from(file);
  let result = (|| -> std::io::Result<()> {
    if ini {
      return text_edit::set_ini_setting(&path, key, value);
    }
    if !path.is_file() {
      return Ok(());
    }
    let text = fs::read_to_string(&path)?;
    let edited = text_edit::set_setting(&text, key, value, text_edit::Layout::Spaced);
    replace_file(&path, &edited)
  })();
  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {}: {error}", path.display());
      ExitCode::from(1)
    }
  }
}

/// Rewrites one declaration inside the named blocks of a CSS file:
/// `css-block radius|border FILE BLOCK_START VALUE`. The file is replaced
/// atomically and keeps its permissions.
fn css_block(arguments: &[String]) -> ExitCode {
  let [declaration, file, block, value] = arguments else {
    eprintln!("usage: argvus config css-block radius|border FILE BLOCK_START VALUE");
    return ExitCode::from(64);
  };
  let Some(declaration) = css_edit::Declaration::parse(declaration) else {
    eprintln!("argvus-config: unknown CSS declaration: {declaration}");
    return ExitCode::from(64);
  };
  let path = PathBuf::from(file);
  let result = (|| -> std::io::Result<()> {
    let text = fs::read_to_string(&path)?;
    let edited = css_edit::set_declaration_in_blocks(&text, block, declaration, value);
    if edited == text {
      return Ok(());
    }
    let temporary = path.with_extension("argvus-css-tmp");
    fs::write(&temporary, edited)?;
    fs::set_permissions(&temporary, fs::metadata(&path)?.permissions())?;
    fs::rename(&temporary, &path)
  })();
  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {}: {error}", path.display());
      ExitCode::from(1)
    }
  }
}

/// Theme identity queries used by the appearance scripts. Each one prints a
/// single value and exits 1 when there is none.
fn theme_command(command: &str, arguments: &[String]) -> ExitCode {
  let Some(id) = arguments.first() else {
    eprintln!("usage: argvus config {command} THEME");
    return ExitCode::from(64);
  };
  let themes_dir = env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
    .join("appearance/themes.d");
  match command {
    "theme-resolve" => match layout_variant() {
      Ok(variant) => {
        println!("{}", theme_resolve::resolve_theme(id, &variant));
        ExitCode::SUCCESS
      }
      Err(error) => {
        eprintln!("argvus-config: cannot read the layout mode: {error}");
        ExitCode::from(1)
      }
    },
    "theme-gtk-mode" => {
      println!("{}", theme_resolve::gtk_mode(&themes_dir, id));
      ExitCode::SUCCESS
    }
    "theme-wallpaper" => match theme_resolve::theme_wallpaper(&themes_dir, &wallpaper_root(), id) {
      Some(path) => {
        println!("{}", path.display());
        ExitCode::SUCCESS
      }
      None => ExitCode::from(1),
    },
    _ => {
      eprintln!("argvus-config: unknown theme command: {command}");
      ExitCode::from(64)
    }
  }
}

/// The Sticky/Float variant from the canonical layout, or `sticky` when unknown.
/// The Sticky/Float variant of the canonical layout. An unset value is Sticky (an
/// empty string). A configuration that cannot be read is an error: falling back to
/// Sticky would silently change the mode the user chose.
fn layout_variant() -> Result<String, Box<dyn std::error::Error>> {
  let document = ConfigStore::from_environment()?.load()?;
  Ok(
    document
      .get("/layout/variant")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap_or_default(),
  )
}

/// The wallpaper root: `WALLPAPER_ROOT`, or `<backgrounds>/argvus`.
fn wallpaper_root() -> PathBuf {
  if let Some(root) = env::var_os("WALLPAPER_ROOT").filter(|value| !value.is_empty()) {
    return PathBuf::from(root);
  }
  let backgrounds = env::var_os("ARGVUS_BACKGROUNDS_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/backgrounds"));
  backgrounds.join("argvus")
}

/// The commands directory the session scripts use (`ARGVUS_COMMANDS_DIR`).
fn commands_dir() -> PathBuf {
  env::var_os("ARGVUS_COMMANDS_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/lib/argvus/commands"))
}

/// Runs the session-loading spinner until the compositor accepts it.
fn session_loading() -> ExitCode {
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: configuration directory is unavailable");
    return ExitCode::from(1);
  };
  let runtime_dir = env::var_os("XDG_RUNTIME_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      // SAFETY: getuid has no preconditions and cannot fail.
      let uid = unsafe { libc::getuid() };
      PathBuf::from(format!("/run/user/{uid}"))
    });
  let status = session_loading::run(&paths, &runtime_dir, &session_loading::spinner_binary());
  ExitCode::from(status.clamp(0, 255) as u8)
}

/// Runs the session preparation (the startup step that used to live in hypr-init.sh).
fn session_prepare() -> ExitCode {
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: configuration directory is unavailable");
    return ExitCode::from(1);
  };
  let cache = match cache_home() {
    Ok(cache) => cache,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return ExitCode::from(1);
    }
  };
  let status = session_prepare::prepare(&paths, &commands_dir(), &cache);
  ExitCode::from(status.clamp(0, 255) as u8)
}

/// Synchronizes the Rofi, btop and yazi files without reloading anything.
/// Session preparation uses it; errors are ignored, as the shell ignored them.
fn session_sync_config() -> ExitCode {
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    return ExitCode::SUCCESS;
  };
  let commands_dir = commands_dir();
  let _ = session_reload::sync_user_files(&session_reload::ReloadContext {
    paths: &paths,
    commands_dir: &commands_dir,
  });
  ExitCode::SUCCESS
}

/// Runs the runtime half of a configuration reload (see `session_reload`).
/// `--login` runs the reduced login variant (`session_reload::reload_at_login`).
fn session_reload_config(arguments: &[String]) -> ExitCode {
  if arguments.iter().any(|argument| argument == "--login") {
    let status = session_reload::reload_at_login();
    return ExitCode::from(status.clamp(0, 255) as u8);
  }
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: configuration directory is unavailable");
    return ExitCode::from(1);
  };
  let commands_dir = commands_dir();
  let status = session_reload::reload_config(&session_reload::ReloadContext {
    paths: &paths,
    commands_dir: &commands_dir,
  });
  ExitCode::from(status.clamp(0, 255) as u8)
}

/// Restarts the user units that serve the named components.
///
/// Unknown components exit with 64 before any unit is touched, the code the
/// session script used before this moved to Rust.
fn session_restart(components: &[String]) -> ExitCode {
  let names: Vec<&str> = components.iter().map(String::as_str).collect();
  let cache = match cache_home() {
    Ok(cache) => cache,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return ExitCode::from(1);
    }
  };
  let units = match session_units::units_for(&names, || session_units::telemetry_enabled(&cache)) {
    Ok(units) => units,
    Err(unknown) => {
      eprintln!("argvus-config: unknown session component: {}", unknown.0);
      return ExitCode::from(64);
    }
  };
  if units.is_empty() {
    return ExitCode::SUCCESS;
  }
  match std::process::Command::new("systemctl")
    .args(["--user", "restart"])
    .args(&units)
    .status()
  {
    Ok(status) if status.success() => ExitCode::SUCCESS,
    Ok(status) => ExitCode::from(status.code().unwrap_or(1).clamp(1, 255) as u8),
    Err(error) => {
      eprintln!("argvus-config: failed to run systemctl: {error}");
      ExitCode::from(1)
    }
  }
}

/// Prints the reload decision for the latest projection. A missing or
/// unreadable manifest means a full reload, as the session scripts assumed.
fn print_session_plan(store: &ConfigStore) -> Result<(), Box<dyn std::error::Error>> {
  use crate::{project::ArgvusPaths, session_plan};
  let manifest_path = ArgvusPaths::from_store(store)?.projection_manifest;
  let manifest: Value = fs::read_to_string(&manifest_path)
    .ok()
    .and_then(|text| serde_json::from_str(&text).ok())
    .unwrap_or(Value::Null);
  print!(
    "{}",
    session_plan::render(&session_plan::decide_reload(&manifest))
  );
  Ok(())
}

fn print_help() {
  println!(
    "argvus-config path|validate|ensure|migrate|get|set|patch|apply-theme|apply-mode|unset|export|import|project [--force|--only keyboard_shortcuts]|accent|display|monitor-set|monitor-primary|monitor-workspaces|monitor-remove|monitor-import|monitor-sync"
  );
  println!(
    "argvus config hyprland reload|monitor LINE|dpms [OUTPUT] on|off|focus-class CLASS|keyboard-layout LAYOUTS|exit"
  );
  println!(
    "  runtime actions on the running compositor (replaces hyprctl eval/dispatch/keyword/reload)"
  );
  println!("  ensure  materialize canonical defaults without resetting explicit values");
  println!("  get /appearance/theme [--effective] [--raw]");
  println!("  set /appearance/theme \"argvus-dark\"");
  println!("  patch FILE|-                         atomically update multiple JSON pointers");
  println!(
    "  audio volume <N|+N|-N> [--limit PERCENT] | mute <on|off|toggle>   change the output sink and record it in /audio"
  );
  println!("  apply-theme THEME [theme options]     atomically apply theme-owned fields");
  println!(
    "  apply-mode <sticky|float>             switch Sticky/Float without changing the theme"
  );
  println!("  export --scope appearance|desktop [--output FILE]");
  println!("  import --scope appearance|desktop FILE");
  println!(
    "  project  incrementally generate native and generated files from modular configuration"
  );
  println!("           use --force to regenerate every projection");
  println!("           use --only keyboard_shortcuts for shortcut-only projection");
}

/// Runs one monitor command against the canonical store and prints its error,
/// so the display script can report it. The script keeps its own messages.
fn monitor_command(
  arguments: &[String],
  operation: impl FnOnce(&ConfigStore, &[String]) -> ConfigResult<()>,
) -> ExitCode {
  let store = match ConfigStore::from_environment() {
    Ok(store) => store,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return ExitCode::from(1);
    }
  };
  match operation(&store, arguments) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      ExitCode::from(1)
    }
  }
}

/// `read-path RELATIVE`: prints the file a reader sees for a packaged config
/// path, with the same cascade as the shell's `paths_config`. The packaged
/// file is printed when no copy exists.
fn read_path(arguments: &[String]) -> ExitCode {
  let [relative] = arguments else {
    eprintln!("usage: argvus config read-path RELATIVE");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  println!("{}", paths.read_config(relative).display());
  ExitCode::SUCCESS
}
