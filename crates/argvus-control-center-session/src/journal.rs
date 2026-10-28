//! Journal parsing and presentation of the Session page.
use argvus_control_center_core::sanitize::terminal_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
  /// Raw `__REALTIME_TIMESTAMP` value in microseconds.
  pub timestamp_micros: Option<u64>,
  pub unit: Option<String>,
  pub pid: Option<String>,
  pub message: Option<String>,
}

impl JournalEntry {
  pub fn rendered_timestamp(&self) -> String {
    self
      .timestamp_micros
      .map(relative_timestamp)
      .unwrap_or_else(|| "—".into())
  }
}

/// Formats the journal timestamp as a compact relative age ("now", "5 min",
/// "2 h", "3 d"), which is the clearest without pulling in a calendar crate.
fn relative_timestamp(micros: u64) -> String {
  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .ok()
    .map(|elapsed| elapsed.as_micros() as u64)
    .unwrap_or(micros);
  let elapsed_seconds = micros.abs_diff(now) / 1_000_000;
  match elapsed_seconds {
    0..=45 => "agora".into(),
    46..=3599 => format!("{} min", elapsed_seconds / 60),
    3600..=86_399 => format!("{} h", elapsed_seconds / 3600),
    _ => format!("{} d", elapsed_seconds / 86_400),
  }
}

/// Parses one line of `journalctl --output=json --user` output.
pub fn parse_line(line: &str) -> Result<JournalEntry, String> {
  let value: serde_json::Value = serde_json::from_str(line).map_err(|error| error.to_string())?;
  let object = value.as_object().ok_or("journal line was not an object")?;
  Ok(JournalEntry {
    timestamp_micros: object
      .get("__REALTIME_TIMESTAMP")
      .and_then(serde_json::Value::as_str)
      .and_then(|value| value.parse::<u64>().ok()),
    unit: object
      .get("_SYSTEMD_UNIT")
      .and_then(serde_json::Value::as_str)
      .map(terminal_text),
    pid: object
      .get("_PID")
      .and_then(serde_json::Value::as_str)
      .map(terminal_text),
    message: object
      .get("MESSAGE")
      .and_then(serde_json::Value::as_str)
      .map(terminal_text),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_journal_lines_and_sanitizes() {
    let entry = parse_line(
      r#"{"__REALTIME_TIMESTAMP":"1700000000000000","_SYSTEMD_UNIT":"waybar.service","_PID":"1234","MESSAGE":"bad\u001b[31m text"}"#,
    )
    .unwrap();
    assert_eq!(entry.unit.as_deref(), Some("waybar.service"));
    assert_eq!(entry.pid.as_deref(), Some("1234"));
    assert_eq!(entry.message.as_deref(), Some("bad text"));
    assert_eq!(entry.timestamp_micros, Some(1_700_000_000_000_000));
  }

  #[test]
  fn rejects_invalid_json() {
    assert!(parse_line("not json").is_err());
  }

  #[test]
  fn recent_entries_render_as_relative_time() {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_micros() as u64;
    let recent = relative_timestamp(now - 6_000_000);
    assert!(!recent.is_empty());
    let old = relative_timestamp(now - 3 * 86_400 * 1_000_000);
    assert_eq!(old, "3 d");
  }
}
