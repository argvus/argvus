//! The messages of the telemetry commands, from argvus-i18n, read through the
//! `argvus i18n` route as the shell helper of the session reads them. Without a
//! translator the key itself is shown.

use std::process::{Command, Stdio};

/// The domain of the telemetry messages.
pub const DOMAIN: &str = "widget-telemetry";

/// The text of a key of the telemetry domain, or the key itself.
pub fn text(key: &str, arguments: &[String]) -> String {
  text_in(DOMAIN, key, arguments)
}

/// The text of a key of a domain, or the key itself.
pub fn text_in(domain: &str, key: &str, arguments: &[String]) -> String {
  let output = Command::new("argvus")
    .args(["i18n", "get", domain, key])
    .args(arguments)
    .stderr(Stdio::null())
    .output();
  match output {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
      .trim_end()
      .to_string(),
    _ => key.to_string(),
  }
}
