//! The messages of the command, from argvus-i18n (domain `launcher`), and the
//! mapping of each key error to its message. A missing translator shows the key,
//! as the script's `argvus_tr` did.

use argvus_keys_core::KeyError;
use std::io::Write;
use std::process::{Command, Stdio};

/// Where a message is written.
#[derive(Clone, Copy)]
pub enum Stream {
  Stdout,
  Stderr,
}

/// The translator's output for a key, or `None` when `argvus` is not installed.
fn translate(key: &str, arguments: &[String]) -> Option<Vec<u8>> {
  let output = Command::new("argvus")
    .args(["i18n", "get", "launcher", key])
    .args(arguments)
    .stderr(Stdio::inherit())
    .output()
    .ok()?;
  Some(output.stdout)
}

/// Writes the message of a launcher key.
pub fn print_launcher(key: &str, arguments: &[String], stream: Stream) {
  let bytes = translate(key, arguments).unwrap_or_else(|| format!("{key}\n").into_bytes());
  let _ = match stream {
    Stream::Stdout => std::io::stdout().write_all(&bytes),
    Stream::Stderr => std::io::stderr().write_all(&bytes),
  };
}

/// The text of a launcher key, without its trailing newline.
pub fn text_launcher(key: &str, arguments: &[String]) -> String {
  match translate(key, arguments) {
    Some(bytes) => String::from_utf8_lossy(&bytes).trim_end().to_string(),
    None => key.to_string(),
  }
}

/// The message of a key error, translated.
pub fn key_error(error: &KeyError) -> String {
  match error {
    KeyError::InvalidName(name) => {
      text_launcher("error.key_invalid_name", &[format!("name={name}")])
    }
    KeyError::InvalidType(key_type) => {
      text_launcher("error.key_bad_type", &[format!("type={key_type}")])
    }
    KeyError::InvalidComment => text_launcher("error.keys_usage", &[]),
    KeyError::Exists(name) => text_launcher("error.key_exists", &[format!("name={name}")]),
    KeyError::NotFound(name) => text_launcher("error.key_not_found", &[format!("name={name}")]),
    KeyError::NoPublic(name) => text_launcher("error.key_no_public", &[format!("name={name}")]),
    KeyError::Unreadable(name) => text_launcher("error.key_unreadable", &[format!("name={name}")]),
    KeyError::InUse(name) => text_launcher("error.key_in_use", &[format!("name={name}")]),
    other => other.to_string(),
  }
}

/// Shows an error for a shortcut run, where stderr is not visible, then the
/// caller exits. The notification is best effort.
pub fn report_error(message: &str) {
  report_error_as("argvus-keys", message);
}

/// The same report, under the name of the command that failed.
pub fn report_error_as(program: &str, message: &str) {
  eprintln!("{program}: {message}");
  if Command::new("notify-send")
    .args(["-a", program, message])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .is_err()
  {
    // No notification daemon or no notify-send: the message is on stderr.
  }
}
