//! Subcommands that read or change the configuration store: get, set, patch, theme and mode, import and export, Hyprland power state.

use super::session_commands::print_session_plan;
use super::theme_commands::{cache_home, splash_theme};
use super::*;

pub(super) fn run(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
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
        let theme = splash_theme(&store);
        let environment = session_splash::SplashEnvironment::resolve();
        let request = session_splash::SplashRequest::session(&theme);
        // Printed only when this call started the splash; the caller
        // then owns it and stops it when the reload ends. A splash that is
        // already running is the normal case inside a reload, so it is silent.
        match session_splash::start_with(
          &environment,
          &crate::session_loading::splash_binary(),
          &request,
        ) {
          session_splash::StartOutcome::Started => println!("started"),
          session_splash::StartOutcome::AlreadyRunning => {}
          outcome => eprintln!("loading splash not shown: {outcome:?}"),
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
      let enabled = session_units::telemetry_enabled(
        &cache_home()?,
        session_units::configured_telemetry_default,
      );
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

pub(super) fn get_value(
  store: &ConfigStore,
  arguments: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
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

pub(super) fn set_value(
  store: &ConfigStore,
  arguments: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
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

pub(super) fn patch_value(
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

pub(super) fn apply_theme(
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
  // The theme's own defaults come from its manifest (themes/<family>/theme.toml),
  // the same source the accent and GTK mode fallbacks use everywhere else.
  if !theme_resolve::is_valid_theme_id(theme) {
    return Err(format!("invalid theme id '{theme}'").into());
  }
  let theme_paths = theme_resolve::theme_paths();
  let default_accent = accent::theme_default_accent(&theme_paths, theme);
  let default_mode = theme_resolve::gtk_mode(&theme_paths, theme);
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

pub(super) fn apply_mode(
  store: &ConfigStore,
  arguments: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
  let variant = arguments
    .first()
    .ok_or("usage: argvus config apply-mode <sticky|float>")?;
  store.apply_layout_variant(variant)?;
  Ok(())
}

pub(super) fn export_scope(
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

pub(super) fn import_scope(
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
pub(super) fn hyprland_command(arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
pub(super) fn parse_dpms_state(state: &str) -> Result<bool, Box<dyn std::error::Error>> {
  match state {
    "on" => Ok(true),
    "off" => Ok(false),
    other => Err(format!("dpms state must be on or off, got {other}").into()),
  }
}

pub(super) fn print_help() {
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
