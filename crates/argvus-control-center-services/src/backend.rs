//! Implements isolated integration with system tools and APIs in crate `argvus control center services`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::{
  journal::{self, JournalEntry},
  model::Unit,
};
use argvus_control_center_core::{
  process::{ProcessOutput, ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use serde_json::Value;
use std::time::Duration;

/// Loads one service scope and publishes it as soon as it is available.
/// Callers can merge the result with another scope to render useful partial
/// state while the remaining systemctl query is still running.
pub fn list_progressive(user: bool, publish: impl Fn(Vec<Unit>)) -> Result<Vec<Unit>, String> {
  let (units, files) = rayon::join(|| list_units(user), || list_unit_files(user));
  let units = units?;
  let session = if user { "user" } else { "system" };
  let mut units = units
    .into_iter()
    .map(|mut unit| {
      unit.scope = session.into();
      unit
    })
    .collect::<Vec<_>>();
  if let Ok(file_states) = files {
    for (name, state) in file_states {
      if let Some(unit) = units.iter_mut().find(|unit| unit.name == name) {
        unit.file_state = state;
      }
    }
  }
  publish(units.clone());
  Ok(units)
}
/// Executes the `list` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn list(user: bool) -> Result<Vec<Unit>, String> {
  let (units, files) = rayon::join(|| list_units(user), || list_unit_files(user));
  let mut units = units?;
  let session = if user { "user" } else { "system" };
  for unit in &mut units {
    unit.scope = session.into();
  }
  if let Ok(file_states) = files {
    for (name, state) in file_states {
      if let Some(unit) = units.iter_mut().find(|unit| unit.name == name) {
        unit.file_state = state;
      }
    }
  }
  Ok(units)
}

/// Executes the `list_units` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn list_units(user: bool) -> Result<Vec<Unit>, String> {
  let mut r = ProcessRequest::new("systemctl")
    .arg("--no-pager")
    .arg("--type=service")
    .arg("--all")
    .arg("--output=json")
    .timeout(Duration::from_secs(5));
  if user {
    r = r.arg("--user")
  }
  let out = SystemProcessRunner.run(&r).map_err(|e| e.to_string())?;
  parse_units(&out)
}

/// Executes the `list_unit_files` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn list_unit_files(user: bool) -> Result<Vec<(String, String)>, String> {
  let mut files = ProcessRequest::new("systemctl")
    .arg("--no-pager")
    .arg("--type=service")
    .arg("--output=json")
    .arg("list-unit-files")
    .timeout(Duration::from_secs(5));
  if user {
    files = files.arg("--user");
  }
  let output = SystemProcessRunner.run(&files).map_err(|e| e.to_string())?;
  Ok(parse_unit_files(&output))
}

/// Converts input data into `parse_unit_files` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_unit_files(output: &ProcessOutput) -> Vec<(String, String)> {
  serde_json::from_slice::<Value>(&output.stdout)
    .ok()
    .and_then(|value| value.as_array().cloned())
    .unwrap_or_default()
    .into_iter()
    .filter_map(|row| {
      Some((
        row.get("unit_file")?.as_str()?.into(),
        row.get("state")?.as_str()?.into(),
      ))
    })
    .collect()
}
/// Converts input data into `parse_units` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_units(out: &ProcessOutput) -> Result<Vec<Unit>, String> {
  if out.timed_out {
    return Err("systemctl timed out".into());
  }
  let text = String::from_utf8_lossy(&out.stdout);
  if out.status.is_none_or(|s| s != 0) {
    return Err(
      terminal_text(&String::from_utf8_lossy(&out.stderr))
        .trim()
        .into(),
    );
  }
  let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
  let rows = value.as_array().ok_or("systemctl JSON was not an array")?;
  Ok(
    rows
      .iter()
      .filter_map(|v| {
        Some(Unit {
          name: terminal_text(v.get("unit")?.as_str()?),
          description: v
            .get("description")
            .and_then(Value::as_str)
            .map(terminal_text)
            .unwrap_or_default(),
          load: v
            .get("load")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
          active: v
            .get("active")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
          sub: v
            .get("sub")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
          file_state: v
            .get("unit_file_state")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
          main_pid: v.get("main_pid").and_then(Value::as_u64).map(|v| v as u32),
          fragment: v
            .get("fragment_path")
            .and_then(Value::as_str)
            .map(Into::into),
          scope: String::new(),
        })
      })
      .collect(),
  )
}
/// Executes the `logs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn logs(
  unit: Option<&str>,
  previous: bool,
  priority: Option<&str>,
  limit: u32,
) -> Result<Vec<JournalEntry>, String> {
  let mut r = ProcessRequest::new("journalctl")
    .arg("--no-pager")
    .arg("--output=json")
    .arg("-n")
    .arg(limit.to_string());
  r = r.timeout(Duration::from_secs(5));
  if previous {
    r = r.arg("-b").arg("-1")
  } else {
    r = r.arg("-b").arg("0")
  }
  if let Some(unit) = unit {
    r = r.arg("-u").arg(unit)
  }
  if let Some(priority) = priority {
    r = r.arg("-p").arg(priority);
  }
  let out = SystemProcessRunner.run(&r).map_err(|e| e.to_string())?;
  if out.status.is_none_or(|s| s != 0) {
    return Err(
      terminal_text(&String::from_utf8_lossy(&out.stderr))
        .trim()
        .into(),
    );
  }
  Ok(
    String::from_utf8_lossy(&out.stdout)
      .lines()
      .filter_map(|l| journal::parse_line(l).ok())
      .collect(),
  )
}
/// Executes the `action_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn action_args(action: &str, unit: &str) -> Result<Vec<String>, String> {
  if !Unit::valid_name(unit) {
    return Err("invalid systemd unit name".into());
  }
  if ![
    "start",
    "stop",
    "restart",
    "enable",
    "disable",
    "enable-now",
    "disable-now",
  ]
  .contains(&action)
  {
    return Err("unsupported systemd action".into());
  }
  Ok(match action {
    "enable-now" => vec!["enable".into(), "--now".into(), unit.into()],
    "disable-now" => vec!["disable".into(), "--now".into(), unit.into()],
    _ => vec![action.into(), unit.into()],
  })
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  /// Executes the `normalizes_units` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn normalizes_units() {
    let o=ProcessOutput{stdout:br#"[{"unit":"a.service","description":"A","load":"loaded","active":"failed","sub":"failed","unit_file_state":"enabled"}]"#.to_vec(),stderr:vec![],status:Some(0),timed_out:false};
    let u = parse_units(&o).unwrap();
    assert!(u[0].failed());
    assert_eq!(u[0].file_state, "enabled");
  }
  #[test]
  /// Executes the `validates_actions` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn validates_actions() {
    assert!(action_args("restart", "NetworkManager.service").is_ok());
    assert!(action_args("restart", "bad unit.service").is_err());
  }
}
