//! Theme subcommands: apply, runtime, queries, theme files, GTK, wallpaper and the transition overlay.

use super::*;

/// `theme-apply THEME`: applies a theme with the current layout variant.
pub(super) fn theme_apply_command(args: &[String]) -> ExitCode {
  let theme = args.first().map(String::as_str).unwrap_or("");
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
  ExitCode::from(theme_apply::run(&paths, theme, &variant) as u8)
}

/// The effective theme name, falling back to the ARGVUS default.
pub(super) fn effective_theme(store: &ConfigStore) -> Result<String, Box<dyn std::error::Error>> {
  let document = store.load_effective()?;
  Ok(
    document
      .get(json_pointers::APPEARANCE_THEME)
      .and_then(Value::as_str)
      .filter(|theme| !theme.is_empty())
      .unwrap_or(session_splash::DEFAULT_THEME)
      .to_string(),
  )
}

/// The theme the session splash shows. A configuration that cannot be read
/// falls back to the ARGVUS default, so the cover still goes up before the
/// projection that would create it.
pub(super) fn splash_theme(store: &ConfigStore) -> String {
  effective_theme(store).unwrap_or_else(|_| session_splash::DEFAULT_THEME.to_string())
}

/// The ARGVUS cache directory, resolved as the session scripts resolve it.
pub(super) fn cache_home() -> Result<PathBuf, Box<dyn std::error::Error>> {
  if let Some(cache) = env::var_os("ARGVUS_CACHE_HOME") {
    return Ok(PathBuf::from(cache));
  }
  if let Some(cache) = env::var_os("XDG_CACHE_HOME") {
    return Ok(PathBuf::from(cache).join("argvus"));
  }
  let home = env::var_os("HOME").ok_or("HOME is not set")?;
  Ok(PathBuf::from(home).join(".cache/argvus"))
}

/// `gtk-theme-name THEME`: the GTK theme the theme uses. Exits 1 for an unknown theme.
pub(super) fn gtk_theme_name(arguments: &[String]) -> ExitCode {
  let [theme] = arguments else {
    eprintln!("usage: argvus config gtk-theme-name THEME");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    return ExitCode::from(1);
  };
  let theme_paths = crate::theme_build::theme_paths(&paths);
  if theme_resolve::manifest(&theme_paths, theme).is_none() {
    return ExitCode::from(1);
  }
  let light = theme_resolve::gtk_mode(&theme_paths, theme) == "light";
  println!("{}", appearance_runtime::gtk_theme_name(light));
  ExitCode::SUCCESS
}

/// `gtk-files MODE NAME PREFER_DARK ICON FONT`: the GTK settings and mode
/// marker of a theme.
pub(super) fn gtk_files(arguments: &[String]) -> ExitCode {
  let [mode, gtk_theme_name, prefer_dark, icon_theme, font] = arguments else {
    eprintln!("usage: argvus config gtk-files MODE NAME PREFER_DARK ICON FONT");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  appearance_runtime::apply_gtk_theme_files(
    &paths,
    &appearance_runtime::GtkFiles {
      mode: mode.clone(),
      gtk_theme_name: gtk_theme_name.clone(),
      prefer_dark: prefer_dark.clone(),
      icon_theme: icon_theme.clone(),
      font: font.clone(),
    },
  );
  ExitCode::SUCCESS
}

/// `gtk-runtime SCHEME THEME FALLBACK FONT ICON`: the GTK settings of a theme,
/// applied through gsettings. See `appearance_runtime::apply_gtk_runtime`.
pub(super) fn gtk_runtime(arguments: &[String]) -> ExitCode {
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
pub(super) fn wallpaper_apply(arguments: &[String]) -> ExitCode {
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

/// `transition-overlay start THEME|hold THEME|stop`: the loading overlay of a
/// theme switch. `start` prints `started` only when this call started the
/// overlay, so the caller stops only what it started and never a session splash.
///
/// `hold` is for a caller that changes the desktop through several short
/// commands, such as a Control Center `Apply`. The overlay follows the process
/// that started it, so `start` from a short-lived command would close it at
/// once. `hold` keeps this process, and the overlay, alive until its stdin
/// closes, then stops the overlay. A caller that dies closes the pipe too, so
/// the overlay cannot be left covering the screen.
pub(super) fn transition_overlay(arguments: &[String]) -> ExitCode {
  match arguments {
    [action, theme] if action.as_str() == "hold" => {
      let outcome = hold_overlay(
        || theme_transition::start_overlay_with(theme, true),
        || {
          let _ = std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink());
        },
        theme_transition::stop_overlay,
        &mut std::io::stdout(),
      );
      if let Some(message) = theme_transition::overlay_warning(outcome) {
        eprintln!("{message}");
      }
      ExitCode::SUCCESS
    }
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
      eprintln!("usage: argvus config transition-overlay start THEME|hold THEME|stop");
      ExitCode::from(64)
    }
  }
}

/// Starts the overlay and, only when this call started it, prints `started`,
/// waits and then stops it. An overlay that another owner started is left as it
/// is. The steps are passed in so the rule can be tested without a display.
pub(super) fn hold_overlay(
  start: impl FnOnce() -> session_splash::StartOutcome,
  wait: impl FnOnce(),
  stop: impl FnOnce(),
  out: &mut impl std::io::Write,
) -> session_splash::StartOutcome {
  let outcome = start();
  if outcome == session_splash::StartOutcome::Started {
    // The caller reads this line to know the overlay is up before it writes.
    let _ = writeln!(out, "started");
    let _ = out.flush();
    wait();
    stop();
  }
  outcome
}

/// Sets one named line of an appearance file: `theme-file TARGET FILE VALUE`.
/// The targets are listed in `text_edit::Target`.
pub(super) fn theme_file(arguments: &[String]) -> ExitCode {
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

/// `theme-runtime hyprland|consumers`: applies the active theme to the running
/// desktop without restarting it. `hyprland` evaluates the colours in the
/// compositor; `consumers` runs the reload of every registered consumer.
pub(super) fn theme_runtime_command(arguments: &[String]) -> ExitCode {
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  match arguments {
    [target] if target == "hyprland" => match crate::theme_runtime::apply_hyprland_colors(&paths) {
      Ok(()) => ExitCode::SUCCESS,
      Err(error) => {
        eprintln!("argvus-config: hyprland theme: {error}");
        ExitCode::from(1)
      }
    },
    [target] if target == "consumers" => {
      crate::theme_runtime::reload_consumers(&paths, None);
      ExitCode::SUCCESS
    }
    // The wallpaper of the configuration, shown through hyprpaper's IPC.
    [target] if target == "wallpaper" => {
      let wallpaper = ConfigStore::at(&paths.config_home)
        .load_effective()
        .ok()
        .and_then(|document| {
          document
            .get(json_pointers::APPEARANCE_WALLPAPER)
            .and_then(Value::as_str)
            .map(str::to_owned)
        })
        .filter(|wallpaper| !wallpaper.is_empty());
      match wallpaper {
        Some(wallpaper) => {
          crate::theme_runtime::show_wallpaper(&paths, &wallpaper);
          ExitCode::SUCCESS
        }
        None => {
          eprintln!("argvus-config: no wallpaper is configured");
          ExitCode::from(1)
        }
      }
    }
    _ => {
      eprintln!("usage: argvus config theme-runtime hyprland|consumers|wallpaper");
      ExitCode::from(64)
    }
  }
}

/// Theme identity queries used by the appearance scripts. Each one prints a
/// single value and exits 1 when there is none.
pub(super) fn theme_command(command: &str, arguments: &[String]) -> ExitCode {
  let Some(id) = arguments.first() else {
    eprintln!("usage: argvus config {command} THEME");
    return ExitCode::from(64);
  };
  let theme_paths = theme_resolve::theme_paths();
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
      println!("{}", theme_resolve::gtk_mode(&theme_paths, id));
      ExitCode::SUCCESS
    }
    "theme-wallpaper" => {
      match theme_resolve::theme_wallpaper(&theme_paths, &wallpaper_root(), id) {
        Some(path) => {
          println!("{}", path.display());
          ExitCode::SUCCESS
        }
        None => ExitCode::from(1),
      }
    }
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
pub(super) fn layout_variant() -> Result<String, Box<dyn std::error::Error>> {
  let document = ConfigStore::from_environment()?.load()?;
  Ok(
    document
      .get(json_pointers::LAYOUT_VARIANT)
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap_or_default(),
  )
}

/// The wallpaper root: `WALLPAPER_ROOT`, or `<backgrounds>/argvus`.
pub(super) fn wallpaper_root() -> PathBuf {
  if let Some(root) = env::var_os("WALLPAPER_ROOT").filter(|value| !value.is_empty()) {
    return PathBuf::from(root);
  }
  let backgrounds = env::var_os("ARGVUS_BACKGROUNDS_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/backgrounds"));
  backgrounds.join("argvus")
}
