//! Translated messages of the commands, from `argvus-i18n`.
//!
//! Without the translator, the message is its key, as the shell's `argvus_tr`
//! always printed it.

use std::io::{self, Write};
use std::process::{Command, Stdio};

/// Where the message goes: the normal output or the error output.
#[derive(Clone, Copy)]
pub enum Stream {
  Stdout,
  Stderr,
}

/// The translator's output for a key of a domain, or `None` when `argvus` is
/// not installed.
fn translate(domain: &str, key: &str, arguments: &[String]) -> Option<Vec<u8>> {
  let output = Command::new("argvus")
    .args(["i18n", "get", domain, key])
    .args(arguments)
    .stderr(Stdio::inherit())
    .output()
    .ok()?;
  Some(output.stdout)
}

/// Writes the message for an `appearance` key, with its arguments as the
/// translator takes them (`name=value`).
pub fn print(key: &str, arguments: &[String], stream: Stream) {
  print_in("appearance", key, arguments, stream);
}

/// Writes the message of a key in a domain (`appearance`, `display`, ...).
pub fn print_in(domain: &str, key: &str, arguments: &[String], stream: Stream) {
  let bytes = translate(domain, key, arguments).unwrap_or_else(|| format!("{key}\n").into_bytes());
  // A closed output is not an error of the change itself, so it is ignored.
  let _ = match stream {
    Stream::Stdout => io::stdout().write_all(&bytes),
    Stream::Stderr => io::stderr().write_all(&bytes),
  };
}

/// The message text for an `appearance` key without its trailing newline, for
/// labels and notifications.
pub fn text(key: &str, arguments: &[String]) -> String {
  text_in("appearance", key, arguments)
}

/// The message text of a key in a domain, without its trailing newline.
pub fn text_in(domain: &str, key: &str, arguments: &[String]) -> String {
  match translate(domain, key, arguments) {
    Some(bytes) => String::from_utf8_lossy(&bytes).trim_end().to_string(),
    None => key.to_string(),
  }
}
