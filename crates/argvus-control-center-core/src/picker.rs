//! Picks one file through the desktop's file manager. The Control Center uses it
//! for wallpapers, launcher icons and the GPG keys. A terminal file manager runs
//! in a floating window and writes the chosen path to a chooser file. A GUI file
//! manager opens and reports no choice, so the caller gets `None`.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::thread;
use std::time::Duration;

use serde_json::Value;

use crate::paths::{cache_home, home, system_config_root};
use crate::process::{ProcessRequest, ProcessRunner, SystemProcessRunner, command};
use crate::sanitize::terminal_text;

/// Opens the default file manager to choose one file and waits for its choice.
/// `class` is the window class of the floating terminal of a terminal file
/// manager, and `prefix` names its chooser file under the cache. Returns `None`
/// when the file manager is a GUI, which reports no choice.
pub fn pick_file(class: &str, prefix: &str) -> Result<Option<PathBuf>, String> {
  let file_manager =
    default_app("file_manager").ok_or_else(|| "file manager padrão não encontrado".to_string())?;
  let home = home();

  if !is_tui_file_manager(&file_manager) {
    let binary = &file_manager[0];
    let child = command(binary)
      .args(file_manager.iter().skip(1))
      .arg(&home)
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|error| format!("não foi possível abrir o File Manager padrão: {error}"))?;
    raise_file_manager_window(child.id(), gui_file_manager_class(binary));
    return Ok(None);
  }

  let selection = cache_home()
    .join("argvus")
    .join(format!("{prefix}-{}", std::process::id()));
  if let Some(parent) = selection.parent() {
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let _ = fs::remove_file(&selection);

  let mut terminal_args = vec![
    "tui-terminal".to_string(),
    "--class".to_string(),
    class.to_string(),
    "--term".to_string(),
    "kitty".to_string(),
    "--".to_string(),
  ];
  terminal_args.extend(file_manager.iter().cloned());
  terminal_args.push(format!("--chooser-file={}", selection.display()));
  terminal_args.push(home.to_string_lossy().to_string());

  let child = command("argvus")
    .args(terminal_args)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .map_err(|error| format!("não foi possível abrir o File Manager padrão: {error}"))?;
  focus_picker(child.id(), class);
  let outcome = wait_for_choice(child, &selection);
  // The chooser file belongs to this call only, so it goes whatever the outcome.
  let _ = fs::remove_file(&selection);
  outcome
}

/// Waits for the terminal file manager and reads the path it chose.
fn wait_for_choice(mut child: Child, selection: &Path) -> Result<Option<PathBuf>, String> {
  let status = child
    .wait()
    .map_err(|error| format!("File Manager padrão falhou: {error}"))?;
  if status.code().is_some_and(|code| code != 0) {
    return Err("File Manager padrão falhou".into());
  }
  let selected =
    fs::read_to_string(selection).map_err(|_| "nenhum arquivo foi selecionado".to_string())?;
  let selected = selected.trim();
  if selected.is_empty() {
    return Err("nenhum arquivo foi selecionado".into());
  }
  let selected = PathBuf::from(selected);
  if !selected.is_file() {
    return Err("o arquivo selecionado não existe".into());
  }
  Ok(Some(selected))
}

/// Runs a shell script from the system config root and returns its output, or
/// `None` when the script is missing, fails, or receives an unsafe argument.
fn run_script_output(script_path: &Path, args: &[&str]) -> Option<String> {
  if !script_path.is_file() {
    return None;
  }
  let mut request = ProcessRequest::new("sh").arg(script_path.to_string_lossy().to_string());
  for argument in args {
    if argument.is_empty()
      || argument
        .chars()
        .any(|character| character.is_control() || character == '\n')
    {
      return None;
    }
    request = request.arg(*argument);
  }
  let output = SystemProcessRunner
    .run(&request.timeout(Duration::from_secs(5)))
    .ok()?;
  if output.status.is_none_or(|status| status != 0) {
    return None;
  }
  Some(terminal_text(&String::from_utf8_lossy(&output.stdout)))
}

/// Splits a command line into its words, dropping empty ones.
fn command_words(command: &str) -> Vec<String> {
  command
    .split_whitespace()
    .map(str::to_string)
    .filter(|word| !word.is_empty())
    .collect()
}

/// Whether the file manager is a terminal program, which runs in its own window.
fn is_tui_file_manager(command: &[String]) -> bool {
  matches!(
    command.first().map(String::as_str),
    Some(
      "argvus"
        | "spf"
        | "superfile"
        | "yazi"
        | "ranger"
        | "lf"
        | "joshuto"
        | "broot"
        | "mc"
        | "nnn"
    )
  )
}

/// The command of the default app for `category`, as the session configures it.
fn default_app(category: &str) -> Option<Vec<String>> {
  let output = run_script_output(
    &system_config_root().join("session/sh/get-default.sh"),
    &[category],
  )?;
  let words = command_words(output.trim());
  (!words.is_empty()).then_some(words)
}

/// Runs a Hyprland Lua `dispatch` and reports whether it was accepted. Places
/// that dropped a dispatcher (e.g. `window not found`) still exit 0, logging a
/// `warning:`/`error:` on stderr, so only stderr-clean runs count as success.
fn hyprctl_lua_dispatch(expression: &str) -> bool {
  let Ok(output) = SystemProcessRunner.run(
    &ProcessRequest::new("hyprctl")
      .arg("dispatch")
      .arg(expression),
  ) else {
    return false;
  };
  if output.timed_out || output.status != Some(0) {
    return false;
  }
  let stderr = String::from_utf8_lossy(&output.stderr);
  !stderr.contains("warning:") && !stderr.contains("error:")
}

/// Focuses a window (by address or class) and raises it to the front, above the
/// Control Center window that launched it.
fn focus_and_raise(target: &str) {
  // Only raise once the target is actually focused; otherwise the dispatcher
  // below would raise whatever window currently has focus (the Control Center).
  if hyprctl_lua_dispatch(&format!("hl.dsp.focus({{ window = '{target}' }})")) {
    let _ = hyprctl_lua_dispatch("hl.dsp.window.bring_to_top({})");
  }
}

/// Floats the focused picker window at the standard 1024x768 File Manager size,
/// centers it and raises it above the Control Center.
fn float_center_raise() {
  let _ = hyprctl_lua_dispatch("hl.dsp.window.float({ action = 'enable' })");
  let _ = hyprctl_lua_dispatch("hl.dsp.window.resize({ x = 1024, y = 768 })");
  let _ = hyprctl_lua_dispatch("hl.dsp.window.center({})");
  let _ = hyprctl_lua_dispatch("hl.dsp.window.bring_to_top({})");
}

/// Brings a terminal picker to the front. `child_id` is the PID of the Wayland
/// surface, because the terminal execs. Cold start takes seconds, so the PID is
/// polled first; the class is used only when the wrapper forks instead.
fn focus_picker(child_id: u32, class: &str) {
  for _ in 0..80 {
    thread::sleep(Duration::from_millis(100));
    let Some(address) = client_address_for_pid(child_id) else {
      continue;
    };
    if hyprctl_lua_dispatch(&format!("hl.dsp.focus({{ window = 'address:{address}' }})")) {
      float_center_raise();
    }
    return;
  }
  for _ in 0..40 {
    thread::sleep(Duration::from_millis(100));
    if hyprctl_lua_dispatch(&format!("hl.dsp.focus({{ window = 'class:{class}' }})")) {
      float_center_raise();
      break;
    }
  }
}

/// Best-effort Wayland class (`class:`) used by the GUI File Managers Argvus
/// knows about. TUI managers are matched by their own terminal window instead.
fn gui_file_manager_class(command: &str) -> Option<&'static str> {
  let name = Path::new(command)
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or(command);
  match name {
    "nautilus" => Some("org.gnome.Nautilus"),
    "nemo" => Some("org.nemo.Nemo"),
    "thunar" => Some("Thunar"),
    "dolphin" => Some("org.kde.dolphin"),
    "pcmanfm" | "pcmanfm-qt" => Some("pcmanfm"),
    "krusader" => Some("org.kde.krusader"),
    "caja" => Some("org.mate.Caja"),
    "doublecmd" => Some("doublecmd"),
    _ => None,
  }
}

/// Resolves the Hyprland client address (`0x…`) mapped to `pid`, if the
/// window already exists.
fn client_address_for_pid(pid: u32) -> Option<String> {
  let output = SystemProcessRunner
    .run(&ProcessRequest::new("hyprctl").arg("-j").arg("clients"))
    .ok()?;
  if output.status.is_none_or(|status| status != 0) {
    return None;
  }
  let value: Value = serde_json::from_slice(&output.stdout).ok()?;
  value
    .as_array()?
    .iter()
    .find(|client| client.get("pid").and_then(Value::as_u64) == Some(pid as u64))
    .and_then(|client| client.get("address").and_then(Value::as_str))
    .map(str::to_string)
}

/// Brings a freshly launched GUI File Manager window to the front, focused, on
/// top of the Control Center window. The Wayland surface appears asynchronously,
/// so the exact process is polled first; GTK/KDE apps may be D-Bus activated, in
/// which case the known class is used as a fallback.
fn raise_file_manager_window(pid: u32, class: Option<&str>) {
  for _ in 0..80 {
    thread::sleep(Duration::from_millis(100));
    if let Some(address) = client_address_for_pid(pid) {
      focus_and_raise(&format!("address:{address}"));
      return;
    }
  }
  if let Some(class) = class {
    let target = format!("class:{class}");
    for _ in 0..40 {
      thread::sleep(Duration::from_millis(100));
      focus_and_raise(&target);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn gui_file_manager_class_known_apps() {
    assert_eq!(
      gui_file_manager_class("nautilus"),
      Some("org.gnome.Nautilus")
    );
    assert_eq!(gui_file_manager_class("thunar"), Some("Thunar"));
    assert_eq!(gui_file_manager_class("nemo"), Some("org.nemo.Nemo"));
    assert_eq!(gui_file_manager_class("dolphin"), Some("org.kde.dolphin"));
    assert_eq!(gui_file_manager_class("pcmanfm"), Some("pcmanfm"));
    assert_eq!(
      gui_file_manager_class("/usr/bin/nautilus"),
      Some("org.gnome.Nautilus")
    );
    assert_eq!(gui_file_manager_class("unknown-fm"), None);
    assert_eq!(gui_file_manager_class("spf"), None);
  }

  #[test]
  fn only_terminal_programs_open_in_their_own_window() {
    assert!(is_tui_file_manager(&["yazi".to_string()]));
    assert!(is_tui_file_manager(&[
      "argvus".to_string(),
      "lf".to_string()
    ]));
    assert!(!is_tui_file_manager(&["nautilus".to_string()]));
    assert!(!is_tui_file_manager(&[]));
  }

  #[test]
  fn command_words_drop_the_empty_ones() {
    assert_eq!(
      command_words("  nemo   --new-window "),
      ["nemo", "--new-window"]
    );
    assert!(command_words("   ").is_empty());
  }
}
