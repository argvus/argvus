//! `argvus-system-monitor`: starts btop with the ARGVUS configuration and the
//! active ARGVUS theme and accent. The public command is `argvus system-monitor`.
//!
//! The selected theme and accent come from the canonical ARGVUS configuration,
//! with the earlier `.active-theme` and `.accent-color` files as fallbacks. The
//! btop files are generated into the cache, so the configuration stays the only
//! source.

use argvus_system_monitor_core::btop::render_config;
use argvus_system_monitor_core::command::{self, ALL_BOXES, Command, has_explicit_paths};
use argvus_system_monitor_core::theme::{self, Selection};
use argvus_theme_core::paths::ThemePaths;
use argvus_theme_core::{build, discovery, render};
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command as Process, ExitCode, Stdio};

mod panel;

/// The name every message starts with.
const PROGRAM: &str = "argvus-system-monitor";
/// The btop configuration that ships with the package.
const DEFAULT_BTOP_ROOT: &str = "/usr/share/argvus-system-monitor/btop";

const USAGE: &str = "\
Usage:
  argvus-system-monitor [btop options]
  argvus-system-monitor --panel [--cpu|--mem] [btop options]
  argvus-system-monitor --mem [btop options]
  argvus-system-monitor --cpu [btop options]
  argvus-system-monitor --net [btop options]
  argvus-system-monitor --proc [btop options]
  argvus-system-monitor --print-config
  argvus-system-monitor --print-theme

Module shortcuts:
  --mem   Show only memory/disks
  --cpu   Show only CPU
  --net   Show only network
  --proc  Show only processes

--panel keeps btop inside an ARGVUS TUI frame; without it, btop occupies the terminal directly.

With no module shortcut, all standard boxes are shown.

The active theme is read from:
  ~/.config/argvus/data/.active-theme
";

/// The places the monitor reads and writes.
struct Paths {
  btop_root: PathBuf,
  argvus_data: PathBuf,
  active_theme_file: PathBuf,
  runtime_dir: PathBuf,
}

impl Paths {
  fn from_environment() -> Result<Self, String> {
    let home = env::var_os("HOME")
      .filter(|value| !value.is_empty())
      .map(PathBuf::from)
      .ok_or("HOME is not set")?;
    let cache_home = non_empty_path("XDG_CACHE_HOME").unwrap_or_else(|| home.join(".cache"));
    let argvus_data =
      non_empty_path("ARGVUS_CONFIG_DIR").unwrap_or_else(|| home.join(".config/argvus/data"));
    Ok(Self {
      btop_root: non_empty_path("ARGVUS_SYSTEM_MONITOR_CONFIG")
        .unwrap_or_else(|| PathBuf::from(DEFAULT_BTOP_ROOT)),
      active_theme_file: non_empty_path("ARGVUS_ACTIVE_THEME_FILE")
        .unwrap_or_else(|| argvus_data.join(".active-theme")),
      runtime_dir: cache_home.join("argvus/argvus-system-monitor"),
      argvus_data,
    })
  }

  fn runtime_config(&self) -> PathBuf {
    self.runtime_dir.join("btop.conf")
  }

  fn runtime_themes(&self) -> PathBuf {
    self.runtime_dir.join("themes")
  }

  /// The btop theme: the current theme build when it is this theme, else the
  /// theme rendered in memory (another theme, or before the first switch).
  fn theme_text(&self, theme: &str) -> Result<String, String> {
    let current = self.argvus_data.join("generated/theme/current");
    if let Some(build) = build::current_file_for(&current, theme, "system-monitor/btop.theme") {
      return fs::read_to_string(&build)
        .map_err(|_| format!("btop theme not found: {}", build.display()));
    }
    let paths = ThemePaths::from_environment().ok_or("HOME is not set")?;
    render::render_file(&paths, theme, None, "system-monitor")
  }
}

fn non_empty_path(variable: &str) -> Option<PathBuf> {
  env::var_os(variable)
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
}

/// Runs one command of this crate. `arguments` are the words after the command
/// name. The `argvus` dispatcher and the binary both call this function.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  match run(&arguments) {
    Ok(code) => code,
    Err(message) => {
      eprintln!("{PROGRAM}: {message}");
      ExitCode::from(1)
    }
  }
}

fn run(arguments: &[String]) -> Result<ExitCode, String> {
  match command::parse(arguments) {
    Command::Help => {
      print!("{USAGE}");
      Ok(ExitCode::SUCCESS)
    }
    Command::PrintTheme => {
      let paths = Paths::from_environment()?;
      println!("{}", active_theme(&paths));
      Ok(ExitCode::SUCCESS)
    }
    Command::PrintConfig => {
      let paths = Paths::from_environment()?;
      let theme = active_theme(&paths);
      build_runtime_config(&paths, &theme, ALL_BOXES)?;
      println!("{}", paths.runtime_config().display());
      Ok(ExitCode::SUCCESS)
    }
    Command::Apply(theme) => {
      let paths = Paths::from_environment()?;
      let theme = theme.unwrap_or_else(|| active_theme(&paths));
      build_runtime_theme(&paths, &theme)?;
      Ok(ExitCode::SUCCESS)
    }
    Command::Run {
      panel: use_panel,
      shown_boxes,
      arguments,
    } => {
      let paths = Paths::from_environment()?;
      let mut btop_arguments = Vec::new();
      if !has_explicit_paths(&arguments) {
        let theme = active_theme(&paths);
        build_runtime_config(&paths, &theme, shown_boxes)?;
        btop_arguments.extend([
          "--config".to_string(),
          paths.runtime_config().display().to_string(),
          "--themes-dir".to_string(),
          paths.runtime_themes().display().to_string(),
        ]);
      }
      btop_arguments.extend(arguments);
      if use_panel {
        let _ = env_logger::try_init();
        let catalog = panel::Catalog::load();
        let title_key = match shown_boxes {
          "cpu" => "system-monitor.title.cpu",
          "mem" => "system-monitor.title.memory",
          _ => "system-monitor.title.overview",
        };
        panel::run(&btop_arguments, catalog.tr(title_key), catalog)
      } else {
        exec_btop(&btop_arguments)
      }
    }
  }
}

/// Replaces this process with btop. Returns only when btop cannot start.
fn exec_btop(arguments: &[String]) -> Result<ExitCode, String> {
  let error = Process::new("btop").args(arguments).exec();
  Err(format!("cannot run btop: {error}"))
}

/// The theme to show: the canonical one, the fallback file, or the default.
fn active_theme(paths: &Paths) -> String {
  let canonical = config_get("/appearance/theme");
  let fallback = read_first_line(&paths.active_theme_file);
  let installed = ThemePaths::from_environment();
  let selection = theme::select_theme(canonical.as_deref(), fallback.as_deref(), |name| {
    installed.as_ref().is_some_and(|installed| {
      discovery::find(installed, name.strip_suffix("-float").unwrap_or(name)).is_some()
    })
  });
  match selection {
    Selection::Found(theme) => theme,
    Selection::Missing { requested } => {
      let source = if canonical.is_some() {
        "argvus config /appearance/theme".to_string()
      } else {
        paths.active_theme_file.display().to_string()
      };
      eprintln!(
        "{PROGRAM}: theme \"{requested}\" from {source} was not found; using \"{}\"",
        theme::DEFAULT_THEME
      );
      theme::DEFAULT_THEME.to_string()
    }
  }
}

/// The accent colour in `#RRGGBB`, from the canonical configuration or the
/// fallback file.
fn accent(paths: &Paths) -> Option<String> {
  config_get("/appearance/accent")
    .and_then(|value| theme::normalize_accent(&value))
    .or_else(|| {
      read_first_line(&paths.argvus_data.join(".accent-color"))
        .and_then(|value| theme::normalize_accent(&value))
    })
}

/// Writes the btop theme of `theme` with the accent applied, into the cache.
fn build_runtime_theme(paths: &Paths, theme: &str) -> Result<(), String> {
  let text = paths.theme_text(theme)?;
  fs::create_dir_all(paths.runtime_themes()).map_err(|error| error.to_string())?;
  let accent = accent(paths);
  let themed = theme::apply_accent(&text, accent.as_deref());
  write_atomic(
    &paths.runtime_themes().join(format!("{theme}.theme")),
    &themed,
  )
}

/// Writes the btop configuration with the theme and the boxes, and the theme it
/// uses, into the cache.
fn build_runtime_config(paths: &Paths, theme: &str, shown_boxes: &str) -> Result<(), String> {
  let source = paths.btop_root.join("btop.conf");
  if !source.is_file() {
    return Err(format!("btop config not found: {}", source.display()));
  }
  fs::create_dir_all(&paths.runtime_dir).map_err(|error| error.to_string())?;
  build_runtime_theme(paths, theme)?;
  let text = fs::read_to_string(&source).map_err(|error| error.to_string())?;
  write_atomic(
    &paths.runtime_config(),
    &render_config(&text, theme, shown_boxes),
  )
}

/// Writes a file through a temporary file beside it, so btop reads either the
/// old or the new content.
fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
  let file_name = path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| format!("invalid file name: {}", path.display()))?;
  let temporary = path.with_file_name(format!("{file_name}.tmp-{}", std::process::id()));
  let mut file = fs::File::create(&temporary).map_err(|error| error.to_string())?;
  file
    .write_all(content.as_bytes())
    .and_then(|()| file.sync_all())
    .map_err(|error| error.to_string())?;
  fs::rename(&temporary, path).map_err(|error| {
    let _ = fs::remove_file(&temporary);
    error.to_string()
  })
}

/// The value of a canonical configuration pointer, or `None` when it is unset or
/// `argvus` cannot answer.
fn config_get(pointer: &str) -> Option<String> {
  let output = Process::new("argvus")
    .args(["config", "get", pointer, "--effective", "--raw"])
    .stderr(Stdio::null())
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
  (!value.is_empty() && value != "null").then_some(value)
}

/// The first line of a file, without surrounding blanks and a carriage return.
fn read_first_line(path: &Path) -> Option<String> {
  let text = fs::read_to_string(path).ok()?;
  let line = text.lines().next()?.trim().to_string();
  (!line.is_empty()).then_some(line)
}
