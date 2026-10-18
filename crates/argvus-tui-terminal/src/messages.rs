//! The messages of the launcher, from `argvus i18n` (domain `app-profiles`).
//! Without the translator the key itself is printed, as the script did.

use std::io::Write;
use std::process::{Command, Stdio};

/// Prints a message of the `app-profiles` domain on standard error.
pub fn error(key: &str, arguments: &[String]) {
  let bytes = translate(key, arguments).unwrap_or_else(|| format!("{key}\n").into_bytes());
  let _ = std::io::stderr().write_all(&bytes);
}

fn translate(key: &str, arguments: &[String]) -> Option<Vec<u8>> {
  // The translator is the `argvus` command itself. A missing `argvus` yields no
  // output, so the key is printed as the shell launcher printed it.
  let output = Command::new("argvus")
    .args(["i18n", "get", "app-profiles", key])
    .args(arguments)
    .stderr(Stdio::inherit())
    .output()
    .ok()?;
  Some(output.stdout)
}
