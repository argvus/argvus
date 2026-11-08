//! Session window actions and the recoverable minimization registry.
mod bar;
mod layout;
mod model;
mod service;

use crate::{ConfigError, ConfigResult, hyprland_ipc};
use serde_json::{Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

pub(crate) fn invalid(message: impl Into<String>) -> ConfigError {
  ConfigError::Invalid(message.into())
}

pub(crate) fn runtime_path(extension: &str) -> ConfigResult<PathBuf> {
  let runtime = std::env::var_os("XDG_RUNTIME_DIR")
    .map(PathBuf::from)
    .filter(|path| path.is_absolute())
    .ok_or_else(|| invalid("XDG_RUNTIME_DIR is unavailable"))?;
  let signature =
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| invalid("Hyprland is unavailable"))?;
  hyprland_ipc::socket_path(&runtime, &signature)?;
  // Socket paths are limited to 108 bytes. Hash the entire instance, not just
  // its commit prefix (which is shared by concurrent compositors).
  let mut hasher = DefaultHasher::new();
  signature.hash(&mut hasher);
  Ok(
    runtime
      .join("argvus")
      .join(format!("windows-{:016x}.{extension}", hasher.finish())),
  )
}

pub(crate) fn address(value: &str) -> ConfigResult<&str> {
  if value.starts_with("0x")
    && value.len() > 2
    && value.len() <= 18
    && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
  {
    Ok(value)
  } else {
    Err(invalid("invalid window address"))
  }
}

pub(crate) fn connect() -> ConfigResult<UnixStream> {
  let stream = UnixStream::connect(runtime_path("sock")?)?;
  stream.set_read_timeout(Some(Duration::from_secs(10)))?;
  stream.set_write_timeout(Some(Duration::from_secs(2)))?;
  Ok(stream)
}

pub fn request(action: &str, target: Option<&str>) -> ConfigResult<Value> {
  if let Some(target) = target {
    address(target)?;
  }
  let mut stream = connect()?;
  writeln!(stream, "{}", json!({"action":action,"address":target}))?;
  let mut reply = String::new();
  BufReader::new(stream)
    .take(4 * 1024 * 1024)
    .read_line(&mut reply)?;
  let reply: Value = serde_json::from_str(&reply)?;
  if let Some(error) = reply.get("error").and_then(Value::as_str) {
    return Err(invalid(error));
  }
  Ok(reply)
}

pub(crate) fn mode_applied() -> bool {
  let Ok(signature) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") else {
    return false;
  };
  let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR") else {
    return false;
  };
  std::fs::read_to_string(
    PathBuf::from(runtime)
      .join("argvus")
      .join(format!("window-mode-{signature}.status")),
  )
  .is_ok_and(|contents| contents.lines().next() == Some("floating"))
}

pub fn run(args: &[String]) -> ExitCode {
  let result = (|| -> ConfigResult<()> {
    match args
      .iter()
      .map(String::as_str)
      .collect::<Vec<_>>()
      .as_slice()
    {
      ["service"] => service::run()?,
      ["bar"] => bar::run()?,
      ["decorations"] => {
        hyprland_ipc::eval(&format!(
          "if argvus_window_controls then argvus_window_controls({:?}) end",
          if mode_applied() { "floating" } else { "tile" }
        ))?;
      }
      ["position", position] => bar::position(position, bar::PositionMode::Apply)?,
      ["position", position, "--persist"] => bar::position(position, bar::PositionMode::Save)?,
      ["watch", "--json"] => {
        let mut stream = connect()?;
        stream.set_read_timeout(None)?;
        writeln!(stream, "{}", json!({"action":"watch"}))?;
        for line in BufReader::new(stream).lines() {
          println!("{}", line?);
          std::io::stdout().flush()?;
        }
      }
      ["list", "--json"] => println!("{}", request("list", None)?),
      [
        action @ ("minimize" | "restore" | "activate" | "maximize" | "close"),
        target,
      ] => {
        request(action, Some(target))?;
      }
      [action @ ("restore-all" | "ready")] => {
        request(action, None)?;
      }
      _ => {
        return Err(invalid(
          "usage: argvus windows service|bar|position top|bottom|list --json|watch --json|restore-all|minimize|restore|activate|maximize|close ADDRESS",
        ));
      }
    }
    Ok(())
  })();
  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-windows: {error}");
      if args.first().is_some_and(|action| {
        matches!(
          action.as_str(),
          "minimize" | "restore" | "activate" | "maximize" | "close"
        )
      }) && let Ok(i18n) = argvus_i18n::I18n::new("taskbar")
      {
        let _ = std::process::Command::new("notify-send")
          .args([
            "--app-name=ARGVUS",
            &i18n.tr("window_mode.title"),
            &i18n.tr("windows.failed"),
          ])
          .status();
      }
      ExitCode::FAILURE
    }
  }
}

/// A live service freezes minimization while Tile is applied. If it is stopped,
/// an empty registry permits Tile; a nonempty one must remain recoverable.
pub(crate) fn prepare_tile() -> ConfigResult<()> {
  if runtime_path("sock").is_err() {
    return Ok(());
  }
  match request("prepare-tile", None) {
    Ok(_) => Ok(()),
    Err(_) if model::load()?.is_empty() => Ok(()),
    Err(error) => Err(error),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn addresses_cannot_inject_lua_or_commands() {
    for value in ["0x1", "0xABC012"] {
      assert!(address(value).is_ok());
    }
    for value in ["", "0x", "1", "0x1;exit()", "0x1\n", "0x12345678901234567"] {
      assert!(address(value).is_err());
    }
  }
}
