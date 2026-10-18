use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

use crate::config::Paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcCommand {
  Toggle,
  Show,
  Hide,
  ReloadTheme,
  ReloadConfig,
}

impl IpcCommand {
  fn as_str(self) -> &'static str {
    match self {
      IpcCommand::Toggle => "toggle",
      IpcCommand::Show => "show",
      IpcCommand::Hide => "hide",
      IpcCommand::ReloadTheme => "reload-theme",
      IpcCommand::ReloadConfig => "reload-config",
    }
  }

  fn parse(value: &str) -> Option<Self> {
    match value.trim() {
      "toggle" => Some(IpcCommand::Toggle),
      "show" => Some(IpcCommand::Show),
      "hide" => Some(IpcCommand::Hide),
      "reload" | "reload-theme" => Some(IpcCommand::ReloadTheme),
      "reload-config" => Some(IpcCommand::ReloadConfig),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopupAnchor {
  pub x: i32,
  pub y: i32,
  pub bar_bottom: Option<i32>,
  pub bar_right: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpcMessage {
  pub command: IpcCommand,
  /// Pointer position captured by the Waybar launcher, so the running UI can
  /// select the correct monitor without re-reading the live cursor.
  pub position: Option<PopupAnchor>,
}

impl IpcMessage {
  pub fn parse(line: &str) -> Option<Self> {
    let mut parts = line.split_whitespace();
    let command = IpcCommand::parse(parts.next()?)?;
    let mut position = None;
    if let (Some(x), Some(y)) = (parts.next(), parts.next())
      && let (Ok(x), Ok(y)) = (x.parse(), y.parse())
    {
      let bar_bottom = parts.next().and_then(parse_optional_coordinate);
      let bar_right = parts.next().and_then(parse_optional_coordinate);
      position = Some(PopupAnchor {
        x,
        y,
        bar_bottom,
        bar_right,
      });
    }
    Some(Self { command, position })
  }

  fn as_str(&self) -> String {
    match self.position {
      Some(anchor) => {
        let mut message = format!("{} {} {}", self.command.as_str(), anchor.x, anchor.y);
        if anchor.bar_bottom.is_some() || anchor.bar_right.is_some() {
          message.push_str(&format!(
            " {} {}",
            anchor
              .bar_bottom
              .map_or_else(|| "-".to_string(), |value| value.to_string()),
            anchor
              .bar_right
              .map_or_else(|| "-".to_string(), |value| value.to_string())
          ));
        }
        message
      }
      None => self.command.as_str().to_string(),
    }
  }
}

fn parse_optional_coordinate(value: &str) -> Option<i32> {
  (value != "-" && value != "none")
    .then(|| value.parse().ok())
    .flatten()
}

fn socket_path(paths: &Paths) -> PathBuf {
  paths.cache_dir.join("argvus-taskbar-calendar.sock")
}

/// Send a command to the running instance. Returns true when a running
/// instance accepted and acknowledged the command.
pub fn notify(paths: &Paths, message: IpcMessage) -> bool {
  let Ok(mut stream) = UnixStream::connect(socket_path(paths)) else {
    return false;
  };
  let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
  let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
  if writeln!(stream, "{}", message.as_str()).is_err() {
    return false;
  }
  let _ = stream.flush();
  let mut response = String::new();
  let _ = BufReader::new(&mut stream).read_line(&mut response);
  !response.is_empty()
}

/// Bind the IPC socket for this instance. Returns None when another
/// instance is already running.
pub fn listen(paths: &Paths) -> Option<UnixListener> {
  let path = socket_path(paths);
  if let Ok(listener) = UnixListener::bind(&path) {
    return Some(listener);
  }
  // The socket file exists: either a live daemon or a stale file.
  if UnixStream::connect(&path).is_ok() {
    return None;
  }
  let _ = std::fs::remove_file(&path);
  UnixListener::bind(&path).ok()
}

/// Serve commands from the IPC socket on a background thread, dispatching
/// each received command to `on_command`.
pub fn serve(listener: UnixListener, on_command: impl Fn(IpcMessage) + Send + 'static) {
  std::thread::spawn(move || {
    for stream in listener.incoming().flatten() {
      let mut stream = stream;
      let mut line = String::new();
      if BufReader::new(&mut stream).read_line(&mut line).is_ok() {
        if let Some(message) = IpcMessage::parse(&line) {
          on_command(message);
        }
        let _ = writeln!(stream, "ok");
      }
    }
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn command_round_trip() {
    for command in [
      IpcCommand::Toggle,
      IpcCommand::Show,
      IpcCommand::Hide,
      IpcCommand::ReloadTheme,
      IpcCommand::ReloadConfig,
    ] {
      let message = IpcMessage {
        command,
        position: Some(PopupAnchor {
          x: 123,
          y: 456,
          bar_bottom: Some(64),
          bar_right: Some(1902),
        }),
      };
      assert_eq!(IpcMessage::parse(&message.as_str()), Some(message));
    }
    assert_eq!(IpcCommand::parse("reload"), Some(IpcCommand::ReloadTheme));
    assert_eq!(
      IpcCommand::parse("reload-config"),
      Some(IpcCommand::ReloadConfig)
    );
    assert_eq!(IpcCommand::parse("unknown"), None);
  }

  #[test]
  fn position_is_optional_for_backward_compatibility() {
    let message = IpcMessage {
      command: IpcCommand::Toggle,
      position: None,
    };
    assert_eq!(IpcMessage::parse(&message.as_str()), Some(message));
    assert_eq!(
      IpcMessage::parse("toggle 12 34"),
      Some(IpcMessage {
        command: IpcCommand::Toggle,
        position: Some(PopupAnchor {
          x: 12,
          y: 34,
          bar_bottom: None,
          bar_right: None,
        }),
      })
    );
    assert_eq!(
      IpcMessage::parse("toggle 12 34 - 1902"),
      Some(IpcMessage {
        command: IpcCommand::Toggle,
        position: Some(PopupAnchor {
          x: 12,
          y: 34,
          bar_bottom: None,
          bar_right: Some(1902),
        }),
      })
    );
    assert_eq!(IpcMessage::parse("garbage"), None);
  }
}
