//! Runtime actions on the running Hyprland compositor.
//!
//! The Lua configuration owns static state (binds, rules, monitors loaded at
//! startup). Runtime actions that scripts need (reload, DPMS, focus, exit,
//! applying one monitor line) go through this module instead of shelling out
//! to `hyprctl`, so the compositor is reached by one typed path.
//!
//! Requests are plain text sent to the compositor control socket. A `/eval`
//! request carries a Lua snippet written against the Hyprland 0.56 `hl` API;
//! the compositor answers `ok` on success. Hyprland 0.56 rejects `keyword`
//! requests ("use eval"), which is why monitor changes are sent as `hl.monitor`.

use crate::{ConfigError, ConfigResult};
use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Name of the compositor control socket inside the instance directory.
const CONTROL_SOCKET_NAME: &str = ".socket.sock";
/// Reply the compositor sends for an accepted request.
const REPLY_OK: &str = "ok";
/// Prefix of a reply for an action that ran with nothing to act on.
const REPLY_WARNING_PREFIX: &str = "warning:";
/// Bounds how long a script can wait on a compositor that stopped answering.
const IO_TIMEOUT: Duration = Duration::from_secs(3);

/// A monitor line in Hyprland's legacy form, parsed and checked.
///
/// `name,disabled` disables the output. `name,mode,position,scale` applies a
/// mode, position and scale. Both forms come from nwg-displays and the ARGVUS
/// monitor state, so any other field count is rejected rather than guessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorLine {
  Disabled {
    output: String,
  },
  Configured {
    output: String,
    mode: String,
    position: String,
    scale: String,
  },
}

/// Returns the control socket path for a Hyprland instance.
///
/// The instance signature becomes a path component, so anything that could
/// escape the instance directory is rejected.
///
/// # Errors
///
/// Returns an error when the signature is empty or contains a path separator.
pub fn socket_path(runtime_dir: &Path, signature: &str) -> ConfigResult<PathBuf> {
  let escapes_directory = signature.is_empty()
    || signature == "."
    || signature == ".."
    || signature.contains('/')
    || signature.contains('\0');
  if escapes_directory {
    return Err(ConfigError::Invalid(format!(
      "invalid Hyprland instance signature: {signature:?}"
    )));
  }
  Ok(
    runtime_dir
      .join("hypr")
      .join(signature)
      .join(CONTROL_SOCKET_NAME),
  )
}

/// Sends one request and returns the trimmed reply.
///
/// # Errors
///
/// Returns an error when Hyprland is not running for this session or the
/// socket does not answer within [`IO_TIMEOUT`].
fn send(payload: &str) -> ConfigResult<String> {
  let runtime_dir = env::var_os("XDG_RUNTIME_DIR")
    .map(PathBuf::from)
    .ok_or_else(|| ConfigError::Invalid("XDG_RUNTIME_DIR is not set".into()))?;
  let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| {
    ConfigError::Invalid("HYPRLAND_INSTANCE_SIGNATURE is not set; Hyprland is not running".into())
  })?;
  let path = socket_path(&runtime_dir, &signature)?;

  let mut stream = UnixStream::connect(&path)?;
  stream.set_read_timeout(Some(IO_TIMEOUT))?;
  stream.set_write_timeout(Some(IO_TIMEOUT))?;
  stream.write_all(payload.as_bytes())?;
  // The compositor closes the stream after replying, so reading to EOF
  // gives the full answer.
  let mut reply = String::new();
  stream.read_to_string(&mut reply)?;
  Ok(reply.trim().to_owned())
}

/// Interprets a compositor reply.
///
/// `ok` is success. A `warning:` reply means the action ran but had nothing to
/// act on (for example, no window matched a focus request); it is returned so
/// the caller can show it. An `error:` reply, or any other text, is a failure.
///
/// # Errors
///
/// Returns an error when the reply is neither `ok` nor a warning.
fn interpret_reply(reply: &str) -> ConfigResult<Option<String>> {
  if reply == REPLY_OK {
    Ok(None)
  } else if reply.starts_with(REPLY_WARNING_PREFIX) {
    Ok(Some(reply.to_owned()))
  } else {
    Err(ConfigError::Invalid(format!(
      "Hyprland rejected the request: {reply}"
    )))
  }
}

/// Reloads the Hyprland configuration, which re-applies monitors and other
/// generated Lua state.
///
/// # Errors
///
/// Returns an error when Hyprland is unavailable or rejects the reload.
pub fn reload() -> ConfigResult<Option<String>> {
  interpret_reply(&send("/reload")?)
}

/// Runs a Lua snippet on the compositor with the `/eval` request.
///
/// # Errors
///
/// Returns an error when Hyprland is unavailable or the snippet fails.
pub fn eval(lua: &str) -> ConfigResult<Option<String>> {
  interpret_reply(&send(&format!("/eval {lua}"))?)
}

/// Parses one legacy monitor line (`name,disabled` or `name,mode,position,scale`).
///
/// # Errors
///
/// Returns an error for any other field count or a field outside its allowed
/// character set.
pub fn parse_monitor_line(line: &str) -> ConfigResult<MonitorLine> {
  let fields: Vec<&str> = line.split(',').map(str::trim).collect();
  match fields.as_slice() {
    [output, "disabled"] => {
      check_output_name(output)?;
      Ok(MonitorLine::Disabled {
        output: (*output).to_owned(),
      })
    }
    [output, mode, position, scale] => {
      check_output_name(output)?;
      check_token("mode", mode, is_layout_char)?;
      check_token("position", position, is_layout_char)?;
      check_token("scale", scale, is_scale_char)?;
      Ok(MonitorLine::Configured {
        output: (*output).to_owned(),
        mode: (*mode).to_owned(),
        position: (*position).to_owned(),
        scale: (*scale).to_owned(),
      })
    }
    _ => Err(ConfigError::Invalid(format!(
      "unsupported monitor line (expected name,disabled or name,mode,position,scale): {line:?}"
    ))),
  }
}

/// Builds the `hl.monitor` call for a parsed monitor line.
pub fn monitor_lua(line: &MonitorLine) -> String {
  match line {
    MonitorLine::Disabled { output } => {
      format!("hl.monitor({{ output = \"{output}\", disabled = true }})")
    }
    MonitorLine::Configured {
      output,
      mode,
      position,
      scale,
    } => format!(
      "hl.monitor({{ output = \"{output}\", mode = \"{mode}\", position = \"{position}\", scale = \"{scale}\" }})"
    ),
  }
}

/// Builds the DPMS dispatch for one monitor, or for every monitor when `output`
/// is `None`.
///
/// # Errors
///
/// Returns an error when the monitor name is not a plain output identifier.
pub fn dpms_lua(output: Option<&str>, on: bool) -> ConfigResult<String> {
  let action = if on { "on" } else { "off" };
  let monitor_field = match output {
    Some(output) => {
      check_output_name(output)?;
      format!(", monitor = \"{output}\"")
    }
    None => String::new(),
  };
  Ok(format!(
    "hl.dispatch(hl.dsp.dpms({{ action = \"{action}\"{monitor_field} }}))"
  ))
}

/// Builds the focus dispatch for a window class.
///
/// # Errors
///
/// Returns an error when the class contains characters outside a window
/// class name.
pub fn focus_class_lua(class: &str) -> ConfigResult<String> {
  check_token("class", class, is_name_char)?;
  Ok(format!(
    "hl.dispatch(hl.dsp.focus({{ window = \"class:{class}\" }}))"
  ))
}

/// Builds the `hl.config` call that sets the keyboard layouts (`us,br`).
///
/// # Errors
///
/// Returns an error when the list is empty or a layout is not a plain XKB
/// layout identifier.
pub fn keyboard_layout_lua(layouts: &str) -> ConfigResult<String> {
  for layout in layouts.split(',') {
    check_token("keyboard layout", layout, is_name_char)?;
  }
  Ok(format!(
    "hl.config({{ input = {{ kb_layout = \"{layouts}\" }} }})"
  ))
}

/// Builds the dispatch that exits Hyprland.
pub fn exit_lua() -> &'static str {
  "hl.dispatch(hl.dsp.exit())"
}

/// Output names are restricted to what connector names use (`DP-1`, `eDP-1`,
/// `HDMI-A-1`). Anything else could break out of the Lua string literal.
fn check_output_name(output: &str) -> ConfigResult<()> {
  check_token("output", output, is_name_char)
}

fn check_token(field: &str, value: &str, allowed: fn(char) -> bool) -> ConfigResult<()> {
  if !value.is_empty() && value.chars().all(allowed) {
    Ok(())
  } else {
    Err(ConfigError::Invalid(format!("invalid {field}: {value:?}")))
  }
}

/// Connector names and window classes: letters, digits, `-`, `_` and `.`.
fn is_name_char(character: char) -> bool {
  character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
}

/// Modes and positions: name characters plus `@` (refresh rate), for values
/// such as `1920x1080@60` or `auto-left`.
fn is_layout_char(character: char) -> bool {
  is_name_char(character) || character == '@'
}

fn is_scale_char(character: char) -> bool {
  character.is_ascii_digit() || character == '.'
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn socket_path_is_inside_the_instance_directory() {
    let path = socket_path(Path::new("/run/user/1000"), "abc_123").expect("valid signature");
    assert_eq!(path, Path::new("/run/user/1000/hypr/abc_123/.socket.sock"));
  }

  #[test]
  fn socket_path_rejects_signatures_that_escape_the_directory() {
    for signature in ["", ".", "..", "../other", "a/b", "a\0b"] {
      assert!(
        socket_path(Path::new("/run/user/1000"), signature).is_err(),
        "{signature:?} must be rejected"
      );
    }
  }

  #[test]
  fn parses_disabled_monitor_line() {
    let line = parse_monitor_line("DP-1,disabled").expect("disabled line");
    assert_eq!(
      line,
      MonitorLine::Disabled {
        output: "DP-1".into()
      }
    );
    assert_eq!(
      monitor_lua(&line),
      "hl.monitor({ output = \"DP-1\", disabled = true })"
    );
  }

  #[test]
  fn parses_configured_monitor_line_with_spaces() {
    let line = parse_monitor_line("eDP-1, 1920x1080@60, auto, 1.25").expect("configured line");
    assert_eq!(
      monitor_lua(&line),
      "hl.monitor({ output = \"eDP-1\", mode = \"1920x1080@60\", position = \"auto\", scale = \"1.25\" })"
    );
  }

  #[test]
  fn rejects_monitor_lines_that_could_inject_lua() {
    for line in [
      "DP-1\", disabled = false }) os.execute(\"x\") --,disabled",
      "DP-1,preferred,auto,1\"; os.exit()",
      "DP-1,preferred,auto,1,extra",
      "DP-1",
      "",
    ] {
      assert!(
        parse_monitor_line(line).is_err(),
        "{line:?} must be rejected"
      );
    }
  }

  #[test]
  fn interprets_ok_warning_and_error_replies() {
    assert_eq!(interpret_reply("ok").expect("ok is success"), None);
    assert_eq!(
      interpret_reply("warning: =[C]:-1: hl.focus: window not found")
        .expect("warnings are not failures"),
      Some("warning: =[C]:-1: hl.focus: window not found".to_owned())
    );
    for reply in ["error: [string \"x\"]:1: probe", "", "unexpected"] {
      assert!(interpret_reply(reply).is_err(), "{reply:?} must fail");
    }
  }

  #[test]
  fn keyboard_layout_builder_accepts_layout_lists_only() {
    assert_eq!(
      keyboard_layout_lua("br,us").expect("valid layouts"),
      "hl.config({ input = { kb_layout = \"br,us\" } })"
    );
    for invalid in ["", "br,", "br us", "br\" })", "br,,us"] {
      assert!(
        keyboard_layout_lua(invalid).is_err(),
        "{invalid:?} must be rejected"
      );
    }
  }

  #[test]
  fn dispatch_builders_quote_validated_values_only() {
    assert_eq!(
      dpms_lua(Some("HDMI-A-1"), false).expect("valid output"),
      "hl.dispatch(hl.dsp.dpms({ action = \"off\", monitor = \"HDMI-A-1\" }))"
    );
    assert_eq!(
      dpms_lua(None, true).expect("all monitors"),
      "hl.dispatch(hl.dsp.dpms({ action = \"on\" }))"
    );
    assert_eq!(
      focus_class_lua("Spotify").expect("valid class"),
      "hl.dispatch(hl.dsp.focus({ window = \"class:Spotify\" }))"
    );
    assert!(dpms_lua(Some("DP-1\" })"), true).is_err());
    assert!(focus_class_lua("Spotify\"").is_err());
    assert_eq!(exit_lua(), "hl.dispatch(hl.dsp.exit())");
  }
}
