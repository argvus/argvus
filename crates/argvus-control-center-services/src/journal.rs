//! Journal parsing and presentation of the Services page.
use argvus_control_center_core::sanitize::terminal_text;
use serde::Deserialize;
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct JournalEntry {
  #[serde(rename = "__REALTIME_TIMESTAMP")]
  pub timestamp: Option<String>,
  #[serde(rename = "_SYSTEMD_UNIT")]
  pub unit: Option<String>,
  #[serde(rename = "PRIORITY")]
  pub priority: Option<String>,
  #[serde(rename = "MESSAGE")]
  pub message: Option<String>,
  #[serde(rename = "_PID")]
  pub pid: Option<String>,
  #[serde(rename = "_EXE")]
  pub executable: Option<String>,
  #[serde(rename = "_BOOT_ID")]
  pub boot_id: Option<String>,
}
impl JournalEntry {
  pub fn sanitized(mut self) -> Self {
    self.message = self.message.map(|v| terminal_text(&v));
    self.unit = self.unit.map(|v| terminal_text(&v));
    self.executable = self.executable.map(|v| terminal_text(&v));
    self.pid = self.pid.map(|v| terminal_text(&v));
    self.boot_id = self.boot_id.map(|v| terminal_text(&v));
    self
  }
  pub fn priority_name(&self) -> &str {
    match self.priority.as_deref() {
      Some("0") => "emerg",
      Some("1") => "alert",
      Some("2") => "crit",
      Some("3") => "err",
      Some("4") => "warning",
      Some("5") => "notice",
      Some("6") => "info",
      Some("7") => "debug",
      _ => "unknown",
    }
  }
}
pub fn parse_line(line: &str) -> Result<JournalEntry, String> {
  serde_json::from_str::<JournalEntry>(line)
    .map(|v| v.sanitized())
    .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn parses_and_sanitizes() {
    let e = parse_line(
      r#"{"MESSAGE":"bad\u001b[31m text","PRIORITY":"3","_EXE":"/usr/bin/sshd\u001b[2J"}"#,
    )
    .unwrap();
    assert_eq!(e.message.as_deref(), Some("bad text"));
    assert_eq!(e.executable.as_deref(), Some("/usr/bin/sshd"));
    assert_eq!(e.priority_name(), "err");
  }
}
