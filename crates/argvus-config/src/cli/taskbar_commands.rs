//! Taskbar and Waybar subcommands: refresh, layout, font, shape, margins and CSS blocks.

use super::*;

/// `waybar-refresh RELATIVE`: copies the packaged Waybar file over its managed copy.
pub(super) fn waybar_refresh(arguments: &[String]) -> ExitCode {
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
pub(super) fn waybar_layout(arguments: &[String]) -> ExitCode {
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
pub(super) fn waybar_font(arguments: &[String]) -> ExitCode {
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

/// Sets a waybar margin in a JSONC file: `jsonc-margin [--signed] FILE EDGE VALUE`.
pub(super) fn jsonc_margin(arguments: &[String]) -> ExitCode {
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

/// `taskbar-height HEIGHT`: stores the taskbar height and sets it in the live
/// taskbar. Only the heights in `waybar::TASKBAR_HEIGHTS` are accepted.
pub(super) fn taskbar_height(arguments: &[String]) -> ExitCode {
  let [value] = arguments else {
    eprintln!("usage: argvus config taskbar-height 24|28|32|36");
    return ExitCode::from(64);
  };
  let Some(value) = value
    .parse::<u32>()
    .ok()
    .filter(|value| waybar::TASKBAR_HEIGHTS.contains(value))
  else {
    eprintln!("argvus-config: taskbar height must be one of 24, 28, 32 or 36");
    return ExitCode::from(64);
  };
  taskbar_setting(json_pointers::TASKBAR_SHAPE_HEIGHT, value, |paths| {
    waybar::set_taskbar_height(paths, value)
  })
}

/// `taskbar-border WIDTH`: stores the taskbar border and sets it in the live
/// taskbar. `0` turns it off; `1` to `3` is its thickness in pixels.
pub(super) fn taskbar_border(arguments: &[String]) -> ExitCode {
  let [value] = arguments else {
    eprintln!("usage: argvus config taskbar-border 0|1|2|3");
    return ExitCode::from(64);
  };
  let Some(value) = value
    .parse::<u32>()
    .ok()
    .filter(|value| waybar::TASKBAR_BORDER_WIDTHS.contains(value))
  else {
    eprintln!("argvus-config: taskbar border width must be 0 (off), 1, 2 or 3");
    return ExitCode::from(64);
  };
  taskbar_setting(json_pointers::TASKBAR_SHAPE_BORDER_WIDTH, value, |paths| {
    waybar::set_taskbar_border(paths, value)
  })
}

/// `taskbar-shape`: prints the live taskbar height and border width, one
/// `key=value` line each. A value that cannot be read is printed empty.
pub(super) fn taskbar_shape(arguments: &[String]) -> ExitCode {
  if !arguments.is_empty() {
    eprintln!("usage: argvus config taskbar-shape");
    return ExitCode::from(64);
  }
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  let shape = waybar::taskbar_shape(&paths);
  let show = |value: Option<u32>| value.map(|value| value.to_string()).unwrap_or_default();
  println!("height={}", show(shape.height));
  println!("border={}", show(shape.border_width));
  ExitCode::SUCCESS
}

/// Runs one edit of the live taskbar files and reports its failure.
pub(super) fn taskbar_setting(
  pointer: &str,
  value: u32,
  edit: impl Fn(&session_paths::SessionPaths) -> std::io::Result<()>,
) -> ExitCode {
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  // The canonical value is what a theme switch writes back after it refreshes
  // the managed Waybar files, so it is stored before the live file is edited.
  if let Err(error) =
    ConfigStore::from_environment().and_then(|store| store.update(pointer, Value::from(value)))
  {
    eprintln!("argvus-config: taskbar: {error}");
    return ExitCode::from(1);
  }
  match edit(&paths) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: taskbar: {error}");
      ExitCode::from(1)
    }
  }
}

/// Sets one `key = value` line: `setting [--ini] FILE KEY VALUE`. A plain file
/// that does not exist is left alone; an INI file is created with its
/// `[Settings]` header. The file is replaced atomically and keeps its mode.
pub(super) fn setting(arguments: &[String]) -> ExitCode {
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
pub(super) fn css_block(arguments: &[String]) -> ExitCode {
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
