//! `argvus-terminal`: starts kitty with the ARGVUS terminal configuration of the
//! active theme and font, and regenerates that configuration. The public command
//! is `argvus terminal`.
//!
//! The generated kitty configurations are derived state in the ARGVUS cache. They
//! are written under a lock, so a theme switch and a launch never read a half
//! written file.

use argvus_terminal_core::accent;
use argvus_terminal_core::command::{self, Error, Launch, Mode};
use argvus_terminal_core::font::{self, Font};
use argvus_terminal_core::kitty::{self, BaseSettings};
use argvus_terminal_core::profile;
use argvus_terminal_core::transparency::{self, Transparency};
use argvus_theme_core::paths::ThemePaths;
use argvus_theme_core::{build, discovery, render};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command as Process, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// The name every error message starts with.
const PROGRAM: &str = "argvus-terminal";
/// The theme used when the active theme is not set.
const DEFAULT_THEME: &str = "argvus-dark";
/// How long a launch waits for the lock held by a theme switch.
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
/// The pause between tries for the lock.
const LOCK_POLL: Duration = Duration::from_millis(50);

/// Runs one command of this crate. `arguments` are the words after the command
/// name. The `argvus` dispatcher and the binary both call this function.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  prepare_path();
  match run(&arguments) {
    Ok(code) => code,
    Err(message) => {
      eprintln!("{PROGRAM}: {message}");
      ExitCode::from(1)
    }
  }
}

/// Desktop launchers do not source the shell startup files, so the standard
/// executable directory is added to `PATH` when it is missing.
fn prepare_path() {
  let path = env::var_os("PATH").unwrap_or_default();
  let directories: Vec<PathBuf> = env::split_paths(&path).collect();
  if directories
    .iter()
    .any(|directory| directory == Path::new("/usr/bin"))
  {
    return;
  }
  let mut extended = directories;
  extended.push(PathBuf::from("/usr/bin"));
  if let Ok(joined) = env::join_paths(extended) {
    // SAFETY: called once at startup, before any other thread exists.
    unsafe { env::set_var("PATH", joined) };
  }
}

fn run(arguments: &[String]) -> Result<ExitCode, String> {
  let mode = command::parse(arguments).map_err(|error| error_message(&error))?;
  match mode {
    Mode::Help => {
      println!("{}", message("usage", &[]));
      Ok(ExitCode::SUCCESS)
    }
    Mode::Apply(theme) => {
      let paths = Paths::from_environment()?;
      let theme = theme.unwrap_or_else(|| active_theme(&paths));
      let _lock = acquire_lock(&paths)?;
      base_config(&paths, &theme, "terminal")?;
      tui_config(&paths, &theme, "terminal")?;
      tui_config(&paths, &theme, "apps")?;
      Ok(ExitCode::SUCCESS)
    }
    Mode::PrintConfig(theme) => {
      let paths = Paths::from_environment()?;
      let theme = theme.unwrap_or_else(|| active_theme(&paths));
      let _lock = acquire_lock(&paths)?;
      let path = base_config(&paths, &theme, "terminal")?;
      println!("{}", path.display());
      Ok(ExitCode::SUCCESS)
    }
    Mode::PrintTuiConfig(theme) => {
      let paths = Paths::from_environment()?;
      let theme = theme.unwrap_or_else(|| active_theme(&paths));
      let _lock = acquire_lock(&paths)?;
      let path = tui_config(&paths, &theme, "terminal")?;
      println!("{}", path.display());
      Ok(ExitCode::SUCCESS)
    }
    Mode::Launch(launch) => launch_kitty(launch),
  }
}

fn error_message(error: &Error) -> String {
  match error {
    Error::ClassRequiresValue => message("class_requires_value", &[]),
    Error::FontProfileRequiresValue => message("font_profile_requires_value", &[]),
    Error::UnsupportedFontProfile(profile) => {
      message("unsupported_font_profile", &[format!("profile={profile}")])
    }
  }
}

/// The directories the launcher reads and writes, as the ARGVUS session defines
/// them.
struct Paths {
  config_home: PathBuf,
  cache_home: PathBuf,
  /// The terminal package's own files: its kitty and theme configurations.
  terminal_root: PathBuf,
}

impl Paths {
  fn from_environment() -> Result<Self, String> {
    let home = env::var_os("HOME")
      .filter(|value| !value.is_empty())
      .map(PathBuf::from)
      .ok_or("HOME is not set")?;
    let config_home = non_empty_path("ARGVUS_CONFIG_HOME")
      .or_else(|| non_empty_path("XDG_CONFIG_HOME"))
      .unwrap_or_else(|| home.join(".config"));
    let cache_base = non_empty_path("ARGVUS_CACHE_HOME")
      .or_else(|| non_empty_path("XDG_CACHE_HOME").map(|base| base.join("argvus")))
      .unwrap_or_else(|| home.join(".cache/argvus"));
    Ok(Self {
      config_home,
      cache_home: cache_base,
      terminal_root: non_empty_path("ARGVUS_TERMINAL_SYSTEM_CONFIG")
        .unwrap_or_else(|| PathBuf::from("/usr/share/argvus/terminal")),
    })
  }

  fn data_dir(&self) -> PathBuf {
    self.config_home.join("argvus/data")
  }

  fn cached(&self, relative: &str) -> PathBuf {
    self.cache_home.join(relative)
  }

  /// The default configuration of the terminal package.
  fn terminal_config(&self) -> PathBuf {
    self.terminal_root.join("config")
  }
}

fn non_empty_path(variable: &str) -> Option<PathBuf> {
  env::var_os(variable)
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
}

/// The message of a key of the `terminal` domain, from argvus-i18n. Without the
/// translator the key itself is shown.
fn message(key: &str, arguments: &[String]) -> String {
  let output = Process::new("argvus")
    .args(["i18n", "get", "terminal", key])
    .args(arguments)
    .stderr(Stdio::null())
    .output();
  match output {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
      .trim_end()
      .to_string(),
    _ => key.to_string(),
  }
}

/// The active theme, from the canonical configuration or the fallback file.
/// The canonical configuration first, then the legacy file, then the default.
fn active_theme(paths: &Paths) -> String {
  config_get("/appearance/theme")
    .or_else(|| read_first_line(&paths.data_dir().join(".active-theme")))
    .unwrap_or_else(|| DEFAULT_THEME.to_string())
}

/// The first line of a file when the file has content, without surrounding
/// blanks and a carriage return; otherwise the fallback.
fn read_first_line(path: &Path) -> Option<String> {
  let text = fs::read_to_string(path).ok()?;
  if text.is_empty() {
    return None;
  }
  Some(text.lines().next().unwrap_or_default().trim().to_string())
}

/// The accent of a theme: the accent the user stored, or the theme's own. Only the
/// stored value counts: the effective value falls back to the packaged default, which
/// would hide a custom accent saved in the marker file.
fn accent_for(paths: &Paths, theme: &str) -> String {
  config_stored("/appearance/accent")
    .filter(|accent| accent != "null")
    .or_else(|| read_first_line(&paths.data_dir().join(".accent-color")))
    .unwrap_or_else(|| theme_default_accent(theme))
}

/// The accent a theme declares: `palette.accent` of its manifest.
fn theme_default_accent(theme: &str) -> String {
  ThemePaths::from_environment()
    .and_then(|paths| discovery::find(&paths, accent::theme_id(theme)))
    .and_then(|manifest| manifest.token("accent").ok())
    .map_or_else(|| accent::FALLBACK_ACCENT.to_string(), |color| color.hex())
}

/// The value a user stored at a canonical pointer (`null` when unset), or `None`
/// when `argvus` does not answer.
fn config_stored(pointer: &str) -> Option<String> {
  config_read(pointer, &["--raw"])
}

/// The value of a canonical configuration pointer, or `None` when `argvus` does
/// not answer.
fn config_get(pointer: &str) -> Option<String> {
  config_read(pointer, &["--effective", "--raw"])
}

fn config_read(pointer: &str, options: &[&str]) -> Option<String> {
  let output = Process::new("argvus")
    .args(["config", "get", pointer])
    .args(options)
    .stderr(Stdio::null())
    .output()
    .ok()?;
  output.status.success().then(|| {
    String::from_utf8_lossy(&output.stdout)
      .trim_end()
      .to_string()
  })
}

fn transparency_state() -> Transparency {
  let argvus_present = program_exists("argvus");
  let enabled = config_get("/effects/transparency_terminal_enabled").unwrap_or_default();
  let value = config_get("/effects/transparency_terminal_value").unwrap_or_default();
  Transparency {
    argvus_present,
    enabled,
    value,
  }
}

fn program_exists(program: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
    .unwrap_or(false)
}

/// The font of a profile from the desktop's generated fonts file.
fn font_for(paths: &Paths, profile: &str) -> Font {
  let fonts = fs::read_to_string(paths.data_dir().join("generated/fonts.conf")).ok();
  font::for_profile(profile, fonts.as_deref())
}

/// The first existing file among the user's override and the system default.
fn first_existing(candidates: &[PathBuf]) -> Option<PathBuf> {
  candidates.iter().find(|path| path.is_file()).cloned()
}

/// The theme colours: the current theme build when it is this theme, else the
/// theme rendered into the cache (another theme than the active one, or a
/// session before its first switch).
fn theme_file(paths: &Paths, theme: &str) -> Result<PathBuf, String> {
  let current = paths.data_dir().join("generated/theme/current");
  if let Some(build) = build::current_file_for(&current, theme, "terminal/theme.conf") {
    return Ok(build);
  }
  let theme_paths = ThemePaths::from_environment().ok_or("HOME is not set")?;
  let text = render::render_file(&theme_paths, theme, None, "terminal")?;
  let destination = paths.cached(&format!("terminal/themes/{theme}.conf"));
  write_generated(&destination, &text)?;
  Ok(destination)
}

fn base_source(paths: &Paths) -> Result<PathBuf, String> {
  first_existing(&[
    paths.data_dir().join("terminal/kitty/kitty.conf"),
    paths.terminal_config().join("kitty.conf"),
  ])
  .ok_or_else(|| {
    message(
      "config_not_found",
      &[format!("path={}", paths.terminal_config().display())],
    )
  })
}

fn tui_source(paths: &Paths) -> Result<PathBuf, String> {
  first_existing(&[
    paths.data_dir().join("terminal/kitty-tui/kitty.conf"),
    paths.terminal_config().join("kitty-tui/kitty.conf"),
  ])
  .ok_or_else(|| {
    message(
      "tui_config_not_found",
      &[format!(
        "path={}",
        paths.terminal_config().join("kitty-tui").display()
      )],
    )
  })
}

/// Generates the base configuration of a theme and a profile, and returns its
/// path.
fn base_config(paths: &Paths, theme: &str, profile: &str) -> Result<PathBuf, String> {
  let source = fs::read_to_string(base_source(paths)?).map_err(|error| error.to_string())?;
  let font = font_for(paths, profile);
  let size = format!("{}.0", font.size);
  let opacity = transparency::opacity(&transparency_state());
  let accent = accent_for(paths, theme);
  let theme_file = theme_file(paths, theme)?;
  let settings = BaseSettings {
    theme_file: &theme_file.display().to_string(),
    family: &font.family,
    size: &size,
    opacity: opacity.as_deref(),
    accent: &accent,
  };
  let generated = kitty::base_config(&source, &settings);
  let destination = paths.cached(&profile::base_file(profile));
  write_generated(&destination, &generated)?;
  Ok(destination)
}

/// Generates the TUI configuration of a theme and a profile, and returns its
/// path. The base configuration it includes is generated first.
fn tui_config(paths: &Paths, theme: &str, profile: &str) -> Result<PathBuf, String> {
  let source = fs::read_to_string(tui_source(paths)?).map_err(|error| error.to_string())?;
  let base = base_config(paths, theme, profile)?;
  let font = font_for(paths, profile);
  let size = format!("{}.0", font.size);
  let generated = kitty::tui_config(&source, &base.display().to_string(), &font.family, &size);
  let destination = paths.cached(&profile::tui_file(profile));
  write_generated(&destination, &generated)?;
  Ok(destination)
}

/// Writes a generated file through a temporary file beside it. An empty result
/// is an error and leaves the previous file in place.
fn write_generated(destination: &Path, content: &str) -> Result<(), String> {
  let parent = destination.parent().ok_or_else(|| {
    message(
      "cache_write_failed",
      &[format!("path={}", destination.display())],
    )
  })?;
  fs::create_dir_all(parent).map_err(|_| {
    message(
      "cache_create_failed",
      &[format!("path={}", destination.display())],
    )
  })?;
  let file_name = destination
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or("generated");
  let temporary = parent.join(format!("{file_name}.tmp-{}", std::process::id()));
  let written = fs::File::create(&temporary).and_then(|mut file| {
    file
      .write_all(content.as_bytes())
      .and_then(|()| file.sync_all())
  });
  if written.is_err() || content.is_empty() {
    let _ = fs::remove_file(&temporary);
    return Err(message(
      "cache_write_failed",
      &[format!("path={}", destination.display())],
    ));
  }
  fs::rename(&temporary, destination).map_err(|_| {
    let _ = fs::remove_file(&temporary);
    message(
      "cache_write_failed",
      &[format!("path={}", destination.display())],
    )
  })
}

/// The lock that serializes generation with launches. It is held until the
/// returned file is dropped.
struct Lock {
  _file: File,
}

fn acquire_lock(paths: &Paths) -> Result<Lock, String> {
  let path = paths.cached("argvus-terminal.lock");
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let file = File::options()
    .create(true)
    .append(true)
    .open(&path)
    .map_err(|_| message("cache_lock_failed", &[format!("path={}", path.display())]))?;
  let deadline = Instant::now() + LOCK_TIMEOUT;
  loop {
    // SAFETY: the descriptor belongs to `file`, which is alive for the call.
    let locked = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0;
    if locked {
      return Ok(Lock { _file: file });
    }
    if Instant::now() >= deadline {
      return Err(message(
        "cache_lock_failed",
        &[format!("path={}", path.display())],
      ));
    }
    thread::sleep(LOCK_POLL);
  }
}

/// Starts kitty with the configuration of the active theme and the profile.
fn launch_kitty(launch: Launch) -> Result<ExitCode, String> {
  if !program_exists("kitty") {
    return Err(message("kitty_not_installed", &[]));
  }
  let paths = Paths::from_environment()?;
  let theme = active_theme(&paths);
  let config = {
    let _lock = acquire_lock(&paths)?;
    if launch.tui {
      tui_config(&paths, &theme, &launch.font_profile)?
    } else {
      base_config(&paths, &theme, &launch.font_profile)?
    }
  };
  let error = Process::new("kitty")
    .args(["--class", &launch.class, "--config"])
    .arg(&config)
    .args(&launch.kitty_arguments)
    .exec();
  Err(format!("cannot run kitty: {error}"))
}
