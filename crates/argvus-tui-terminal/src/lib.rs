//! `argvus-tui-terminal`: launches an ARGVUS-owned TUI window with its font
//! profile. The window runs in the terminal backend (`--term`): Kitty by
//! default, or `argvus-terminal`, Alacritty, or any command that takes `-e`.
//!
//! Usage: argvus-tui-terminal [--class C] [--term T] [--profile P] [--materialize-kitty] [--] COMMAND...

mod messages;

use argvus_tui_terminal_core::font::FontState;
use argvus_tui_terminal_core::kitty;
use argvus_tui_terminal_core::opacity::background_opacity;
use argvus_tui_terminal_core::profile::{self, Selection};
use std::env;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

/// The options of one launch.
struct Options {
  class: String,
  term: Option<String>,
  profile: String,
  materialize: bool,
  command: Vec<String>,
}

/// Runs one command of this crate. `arguments` are the words after the command
/// name. The `argvus` dispatcher and the binary both call this function.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  let options = match parse(&arguments) {
    Ok(options) => options,
    Err(code) => return code,
  };
  let selection = profile::select(&options.class, &options.command);
  if options.materialize {
    return match runtime_kitty_config(&options, selection) {
      Ok(_) => ExitCode::SUCCESS,
      Err(code) => code,
    };
  }
  let backend = options.term.clone().unwrap_or_else(|| "kitty".to_string());
  let mut command = match backend.as_str() {
    "argvus-terminal" => {
      let mut command = Command::new("argvus");
      command.args([
        "terminal",
        "--class",
        &options.class,
        "--font-profile",
        selection.font.name(),
        "--tui",
        "-e",
      ]);
      command
    }
    "kitty" => {
      let config = match runtime_kitty_config(&options, selection) {
        Ok(config) => config,
        Err(code) => return code,
      };
      let mut command = Command::new("kitty");
      command
        .args(["--class", &options.class, "--config"])
        .arg(config)
        .arg("-e");
      command
    }
    "alacritty" => {
      let mut command = Command::new("alacritty");
      command.args(["--class", &options.class, "-e"]);
      command
    }
    other => {
      let mut command = Command::new(other);
      command.arg("-e");
      command
    }
  };
  command.args(&options.command);
  let error = command.exec();
  eprintln!("argvus-tui-terminal: cannot run {backend}: {error}");
  ExitCode::from(127)
}

/// Reads the options in the order the launcher always read them. The first word
/// that is not an option starts the command.
fn parse(arguments: &[String]) -> Result<Options, ExitCode> {
  let mut options = Options {
    class: "argvus-tui".to_string(),
    term: None,
    profile: "default".to_string(),
    materialize: false,
    command: Vec::new(),
  };
  let mut index = 0;
  while index < arguments.len() {
    let argument = arguments[index].as_str();
    match argument {
      "--materialize-kitty" => {
        options.materialize = true;
        index += 1;
      }
      "--class" | "--term" | "--terminal" | "--profile" => {
        let Some(value) = arguments.get(index + 1) else {
          if argument == "--class" {
            messages::error("launcher.class_requires_value", &[]);
          } else {
            messages::error(
              "launcher.option_requires_value",
              &[format!("option={argument}")],
            );
          }
          return Err(ExitCode::from(1));
        };
        match argument {
          "--class" => options.class = value.clone(),
          "--profile" => options.profile = value.clone(),
          _ => options.term = Some(value.clone()),
        }
        index += 2;
      }
      "--" => {
        index += 1;
        break;
      }
      _ => {
        if let Some(value) = argument.strip_prefix("--class=") {
          options.class = value.to_string();
        } else if let Some(value) = argument
          .strip_prefix("--term=")
          .or_else(|| argument.strip_prefix("--terminal="))
        {
          options.term = Some(value.to_string());
        } else if let Some(value) = argument.strip_prefix("--profile=") {
          options.profile = value.to_string();
        } else {
          break;
        }
        index += 1;
      }
    }
  }
  options.command = arguments[index..].to_vec();
  if options.term.as_deref() == Some("") {
    options.term = None;
  }
  Ok(options)
}

/// The ARGVUS cache directory, where the runtime profiles are written.
fn cache_home() -> PathBuf {
  if let Some(home) = env::var_os("ARGVUS_CACHE_HOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(home);
  }
  let base = env::var_os("XDG_CACHE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"));
  base.join("argvus")
}

fn config_home() -> PathBuf {
  if let Some(home) = env::var_os("ARGVUS_CONFIG_HOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(home);
  }
  env::var_os("XDG_CONFIG_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"))
}

/// The file a reader sees for a packaged configuration path.
fn packaged_config(relative: &str) -> PathBuf {
  let output = Command::new("argvus")
    .args(["config", "read-path", relative])
    .stderr(Stdio::null())
    .output();
  match output {
    Ok(output) if output.status.success() => PathBuf::from(
      String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string(),
    ),
    _ => {
      let system = env::var("ARGVUS_SYSTEM_CONFIG").unwrap_or_else(|_| "/usr/share/argvus".into());
      PathBuf::from(system).join(relative)
    }
  }
}

/// Whether `argvus` is on the path, as `command -v argvus` checked.
fn argvus_on_path() -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join("argvus").is_file()))
    .unwrap_or(false)
}

/// A configuration value of `argvus config get`, or empty when it fails.
fn config_value(pointer: &str) -> String {
  let output = Command::new("argvus")
    .args(["config", "get", pointer, "--effective", "--raw"])
    .stderr(Stdio::null())
    .output();
  match output {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
      .trim_end_matches('\n')
      .to_string(),
    _ => String::new(),
  }
}

/// The Control Center background opacity from the effect settings. Without the
/// `argvus` command the defaults apply: transparency at 15 percent, both on.
fn control_center_opacity() -> String {
  let (transparency, value, blur) = if argvus_on_path() {
    (
      config_value("/effects/transparency_control-center_enabled"),
      config_value("/effects/transparency_control-center_value"),
      config_value("/effects/blur_control-center_enabled"),
    )
  } else {
    ("true".to_string(), "15".to_string(), "true".to_string())
  };
  background_opacity(&transparency, &value, &blur)
}

/// The Kitty configuration of a window. It is written from the source
/// configuration with the font of the profile, and the policy of the greeter or
/// the Control Center is appended. Returns the path of the written file.
fn runtime_kitty_config(options: &Options, selection: Selection) -> Result<PathBuf, ExitCode> {
  let fonts =
    FontState::new(fs::read_to_string(config_home().join("argvus/data/generated/fonts.conf")).ok());
  let family = fonts.family(selection.font);
  let size = fonts.size(selection);
  let cache = cache_home();

  let source = match options.profile.as_str() {
    "greeter" | "control-center" if argvus_on_path() => {
      let output = Command::new("argvus")
        .args(["terminal", "--print-tui-config"])
        .stderr(Stdio::inherit())
        .output()
        .map_err(|_| ExitCode::from(1))?;
      if !output.status.success() {
        return Err(ExitCode::from(output.status.code().unwrap_or(1) as u8));
      }
      PathBuf::from(
        String::from_utf8_lossy(&output.stdout)
          .trim_end_matches('\n')
          .to_string(),
      )
    }
    _ => {
      let runtime = cache.join("argvus-terminal/kitty-tui-apps.conf");
      if runtime.is_file() {
        runtime
      } else {
        packaged_config("terminal/config/kitty-tui/kitty.conf")
      }
    }
  };

  let destination = match options.profile.as_str() {
    "greeter" => cache.join("kitty-tui/kitty-greeter.conf"),
    "control-center" => cache.join("kitty-tui/kitty-control-center.conf"),
    _ if selection.file_manager => cache.join("kitty-tui/kitty-file-manager.conf"),
    _ => cache.join(format!("kitty-tui/kitty-{}.conf", selection.font.name())),
  };

  let text = match fs::read_to_string(&source) {
    Ok(text) => text,
    Err(error) => {
      eprintln!(
        "argvus-tui-terminal: cannot read {}: {error}",
        source.display()
      );
      return Err(ExitCode::from(1));
    }
  };
  let mut output = kitty::rewrite_fonts(&text, &family, &size);
  match options.profile.as_str() {
    "greeter" => output.push_str(kitty::greeter_policy()),
    "control-center" => output.push_str(&kitty::control_center_policy(&control_center_opacity())),
    _ => {}
  }
  write_file(&destination, &output)?;
  Ok(destination)
}

/// Writes a file, creating its directory first.
fn write_file(path: &Path, content: &str) -> Result<(), ExitCode> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|_| ExitCode::from(1))?;
  }
  fs::write(path, content).map_err(|error| {
    eprintln!(
      "argvus-tui-terminal: cannot write {}: {error}",
      path.display()
    );
    ExitCode::from(1)
  })
}
