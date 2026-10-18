//! `argvus control-center`: opens the Control Center in a single terminal window.
//! A running window is focused instead of starting a second one. The check and the
//! start are serialized with a lock, because two key presses can arrive before
//! Hyprland reports the new client.

use crate::commands::{CONTROL_CENTER, required_executable};
use crate::exec::{Failure, Outcome, env_nonempty, find_on_path, io_failure};
use serde_json::Value;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

/// The window class the terminal gives the Control Center. It is how Hyprland
/// finds the window again.
const WINDOW_CLASS: &str = "argvus-control-center";
/// Some terminal backends expose their own class, so the title is the fallback.
const KITTY_CLASS: &str = "kitty";
const LOCK_FILE_NAME: &str = "argvus-control-center.lock";
const DEFAULT_RUNTIME_DIR: &str = "/tmp";
/// How long to wait for a new window to register with Hyprland (20 × 100 ms).
const REGISTRATION_ATTEMPTS: u32 = 20;
const REGISTRATION_INTERVAL: Duration = Duration::from_millis(100);
const ICON_SCALE: &str = "3/3";

/// Opens the Control Center with `arguments`, after the page alias is mapped to
/// the page name the Control Center understands.
pub fn run(arguments: Vec<OsString>) -> Outcome {
  let control_center = required_executable(CONTROL_CENTER)?;
  let argvus = std::env::current_exe()
    .map_err(|error| Failure::new(format!("cannot locate argvus: {error}")))?;
  let arguments = map_page_alias(arguments);

  if let Some(address) = find_window_address() {
    focus_window(&address);
    return Ok(ExitCode::SUCCESS);
  }

  // Another invocation holds the lock while it starts the window. Do not wait
  // for it: the window is already on its way.
  let Some(lock_file) = try_lock()? else {
    return Ok(ExitCode::SUCCESS);
  };
  // Recheck under the lock: another invocation may have started the window
  // while this one was waiting.
  if let Some(address) = find_window_address() {
    focus_window(&address);
    return Ok(ExitCode::SUCCESS);
  }

  let mut child = spawn_terminal(&argvus, &control_center, &arguments)?;
  // The window is registered before the lock is released. This closes the race
  // without holding the lock for the lifetime of the application.
  let exit_status = wait_for_registration(&mut child)?;
  // Dropping the file releases the lock. The child does not inherit it because the
  // descriptor is close-on-exec.
  drop(lock_file);

  // A terminal that exits before registering a window is a failure the user must
  // see, instead of silently leaving no window.
  match exit_status {
    Some(status) if !status.success() => Err(Failure::exit_status(status_code(status))),
    _ => Ok(ExitCode::SUCCESS),
  }
}

/// Maps the legacy `--apps`, `--displays` and similar flags to the page name the
/// Control Center accepts as its first argument.
fn map_page_alias(mut arguments: Vec<OsString>) -> Vec<OsString> {
  let alias = arguments
    .first()
    .and_then(|argument| argument.to_str())
    .and_then(page_for_alias);
  if let Some(page) = alias {
    arguments[0] = OsString::from(page);
  }
  arguments
}

fn page_for_alias(alias: &str) -> Option<&'static str> {
  Some(match alias {
    "--apps" => "apps",
    "--fonts" => "fonts",
    "--locale" => "locale",
    "--language" => "language",
    "--system" => "system",
    "--about" => "about",
    "--displays" | "--monitors" => "displays",
    "--timezone" | "--date-time" | "--regional-locale" | "--system-locales" | "--encoding"
    | "--keyboard" | "--keyboard-layout" | "--keyboard-variant" | "--console-keymap" => "locale",
    "--hostname" | "--firewall" | "--users" | "--groups" => "system",
    _ => return None,
  })
}

/// Whether a Hyprland client is the Control Center window.
fn is_control_center_window(client: &Value) -> bool {
  let class = client["class"].as_str().unwrap_or_default();
  let initial_class = client["initialClass"].as_str();
  let title = client["title"].as_str().unwrap_or_default();
  class == WINDOW_CLASS
    || initial_class == Some(WINDOW_CLASS)
    || (class == KITTY_CLASS && title.ends_with(WINDOW_CLASS))
}

/// The Hyprland address of the first Control Center window, if one is open.
/// Without `hyprctl` there is no window to find.
fn find_window_address() -> Option<String> {
  find_on_path("hyprctl")?;
  let output = Command::new("hyprctl")
    .args(["clients", "-j"])
    .stderr(Stdio::null())
    .output()
    .ok()?;
  let clients: Value = serde_json::from_slice(&output.stdout).ok()?;
  let window = clients
    .as_array()?
    .iter()
    .find(|client| is_control_center_window(client))?;
  window["address"].as_str().map(str::to_owned)
}

/// Focuses the window at `address`. Hyprland also switches to its workspace.
fn focus_window(address: &str) {
  let focus = format!("hl.dispatch(hl.dsp.focus({{ window = \"address:{address}\" }}))");
  // Focusing is best effort: a failed focus must not block opening the window.
  let _ = Command::new("hyprctl")
    .arg("eval")
    .arg(focus)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

/// Takes the non-blocking lock. `Ok(None)` means another invocation holds it.
fn try_lock() -> Result<Option<File>, Failure> {
  let runtime_dir = env_nonempty("XDG_RUNTIME_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(DEFAULT_RUNTIME_DIR));
  let lock_path = runtime_dir.join(LOCK_FILE_NAME);
  let lock_file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(&lock_path)
    .map_err(|error| io_failure(&lock_path, error))?;
  // SAFETY: the descriptor belongs to `lock_file`, which stays open for the call.
  let locked = unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0;
  Ok(locked.then_some(lock_file))
}

/// Starts the terminal that hosts the Control Center. The terminal class and
/// profile come from the terminal wrapper's own options.
fn spawn_terminal(
  argvus: &Path,
  control_center: &Path,
  arguments: &[OsString],
) -> Result<Child, Failure> {
  Command::new(argvus)
    .arg("tui-terminal")
    .env("ARGVUS_ICON_SCALE", ICON_SCALE)
    .args([
      "--class",
      WINDOW_CLASS,
      "--term",
      "kitty",
      "--profile",
      "control-center",
      "--",
    ])
    .arg(control_center)
    .args(arguments)
    .spawn()
    .map_err(|error| Failure::with_code(format!("cannot run {}: {error}", argvus.display()), 127))
}

/// Waits until the window registers or the terminal exits, polling every
/// `REGISTRATION_INTERVAL`. Returns the exit status when the terminal has exited.
fn wait_for_registration(child: &mut Child) -> Result<Option<ExitStatus>, Failure> {
  if find_on_path("hyprctl").is_some() {
    for _ in 0..REGISTRATION_ATTEMPTS {
      if find_window_address().is_some() {
        break;
      }
      if child
        .try_wait()
        .map_err(|error| Failure::new(format!("cannot wait for the terminal: {error}")))?
        .is_some()
      {
        break;
      }
      thread::sleep(REGISTRATION_INTERVAL);
    }
  }
  child
    .try_wait()
    .map_err(|error| Failure::new(format!("cannot wait for the terminal: {error}")))
}

/// The exit status of a child as a dispatcher status. A signal yields `128 + n`,
/// the shell convention.
fn status_code(status: ExitStatus) -> u8 {
  status
    .code()
    .or_else(|| status.signal().map(|signal| 128 + signal))
    .and_then(|code| u8::try_from(code).ok())
    .unwrap_or(1)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn matches_the_control_center_class() {
    let client = json!({"class": "argvus-control-center", "title": "Control Center"});
    assert!(is_control_center_window(&client));
  }

  #[test]
  fn matches_a_kitty_window_by_its_title() {
    let client = json!({"class": "kitty", "title": "argvus-control-center"});
    assert!(is_control_center_window(&client));
  }

  #[test]
  fn ignores_other_kitty_windows() {
    let client = json!({"class": "kitty", "title": "fish"});
    assert!(!is_control_center_window(&client));
  }

  #[test]
  fn maps_legacy_page_flags_to_page_names() {
    assert_eq!(page_for_alias("--keyboard-layout"), Some("locale"));
    assert_eq!(page_for_alias("--monitors"), Some("displays"));
    assert_eq!(page_for_alias("apps"), None);
  }
}
