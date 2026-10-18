//! The messages of the command, from argvus-i18n (domain `launcher`). Without
//! the translator the key itself is shown, as the script's `argvus_tr` did.

use std::io::Write;
use std::process::{Command, Stdio};

/// Where a message is written.
#[derive(Clone, Copy)]
pub enum Stream {
  Stdout,
  Stderr,
}

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
pub fn text(key: &str, arguments: &[String]) -> String {
  match translate(key, arguments) {
    Some(bytes) => String::from_utf8_lossy(&bytes).trim_end().to_string(),
    None => key.to_string(),
  }
}

/// Shows an error for a shortcut run, where stderr is not visible.
pub fn report_error(message: &str) {
  eprintln!("argvus-projects: {message}");
  let _ = Command::new("notify-send")
    .args(["-a", "argvus-projects", message])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}
