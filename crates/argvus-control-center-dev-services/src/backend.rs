//! Implements isolated integration with system tools and APIs in crate `argvus control center dev services`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::ContainerSummary;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use argvus_control_center_services::Unit;
use serde_json::Value;
use std::time::Duration;

/// Lists the user-scoped systemd services, reusing the same listing the
/// Services page already exposes so both pages stay in sync with a single
/// `systemctl` call shape.
pub fn list_user_services() -> Result<Vec<Unit>, String> {
  argvus_control_center_services::list_units(true)
}

/// Executes the `listening_ports` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn listening_ports() -> Result<Vec<u16>, String> {
  let request = ProcessRequest::new("ss")
    .arg("-tln")
    .timeout(Duration::from_secs(3));
  let output = SystemProcessRunner
    .run(&request)
    .map_err(|e| e.to_string())?;
  if output.timed_out {
    return Err("ss timed out".into());
  }
  if output.status.is_none_or(|s| s != 0) {
    return Err(
      terminal_text(&String::from_utf8_lossy(&output.stderr))
        .trim()
        .into(),
    );
  }
  Ok(parse_listening_ports(&String::from_utf8_lossy(
    &output.stdout,
  )))
}

/// Converts input data into `parse_listening_ports` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_listening_ports(output: &str) -> Vec<u16> {
  let mut ports: Vec<u16> = output
    .lines()
    .skip(1) // header row
    .filter_map(|line| {
      let local_address = line.split_whitespace().nth(3)?;
      local_address.rsplit(':').next()?.parse::<u16>().ok()
    })
    .collect();
  ports.sort_unstable();
  ports.dedup();
  ports
}

/// Lists running containers across every container runtime detected by
/// `capabilities`. One runtime failing to respond does not hide the other.
pub fn containers(capabilities: &Capabilities) -> (Vec<ContainerSummary>, Option<String>) {
  let mut rows = Vec::new();
  let mut errors = Vec::new();
  for runtime in ["podman", "docker"] {
    let available = match runtime {
      "podman" => capabilities.has_podman,
      _ => capabilities.has_docker,
    };
    if !available {
      continue;
    }
    match list_containers(runtime) {
      Ok(found) => rows.extend(found),
      Err(error) => errors.push(format!("{runtime}: {error}")),
    }
  }
  let error = if errors.is_empty() {
    None
  } else {
    Some(errors.join("; "))
  };
  (rows, error)
}

/// Executes the `list_containers` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn list_containers(runtime: &str) -> Result<Vec<ContainerSummary>, String> {
  let request = ProcessRequest::new(runtime)
    .arg("ps")
    .arg("--format")
    .arg("json")
    .timeout(Duration::from_secs(3));
  let output = SystemProcessRunner
    .run(&request)
    .map_err(|e| e.to_string())?;
  if output.timed_out {
    return Err("timed out".into());
  }
  if output.status.is_none_or(|s| s != 0) {
    return Err(
      terminal_text(&String::from_utf8_lossy(&output.stderr))
        .trim()
        .into(),
    );
  }
  Ok(parse_container_rows(
    runtime,
    &String::from_utf8_lossy(&output.stdout),
  ))
}

/// Parses container listings in either shape a container runtime may print
/// for `ps --format json`: a single JSON array (Podman), or one JSON object
/// per line (Docker's NDJSON).
fn parse_container_rows(runtime: &str, text: &str) -> Vec<ContainerSummary> {
  let rows: Vec<Value> = match serde_json::from_str::<Vec<Value>>(text) {
    Ok(rows) => rows,
    Err(_) => text
      .lines()
      .filter(|line| !line.trim().is_empty())
      .filter_map(|line| serde_json::from_str::<Value>(line).ok())
      .collect(),
  };
  rows
    .iter()
    .map(|row| ContainerSummary {
      runtime: runtime.into(),
      name: terminal_text(&container_name(row)),
      image: terminal_text(row.get("Image").and_then(Value::as_str).unwrap_or_default()),
      status: terminal_text(
        row
          .get("Status")
          .and_then(Value::as_str)
          .or_else(|| row.get("State").and_then(Value::as_str))
          .unwrap_or_default(),
      ),
    })
    .collect()
}

/// Podman reports `Names` as an array, Docker as a comma-separated string.
fn container_name(row: &Value) -> String {
  match row.get("Names") {
    Some(Value::Array(names)) => names
      .first()
      .and_then(Value::as_str)
      .unwrap_or_default()
      .into(),
    Some(Value::String(names)) => names.split(',').next().unwrap_or(names).into(),
    _ => String::new(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `parses_ss_local_addresses_including_ipv6` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_ss_local_addresses_including_ipv6() {
    let output = "State   Recv-Q Send-Q Local Address:Port  Peer Address:Port Process\n\
                   LISTEN  0      128    0.0.0.0:22          0.0.0.0:*\n\
                   LISTEN  0      128    127.0.0.1:631       0.0.0.0:*\n\
                   LISTEN  0      128    [::]:22             [::]:*\n";
    assert_eq!(parse_listening_ports(output), vec![22, 631]);
  }

  #[test]
  /// Executes the `parses_podman_json_array` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_podman_json_array() {
    let text = r#"[{"Names":["web"],"Image":"nginx:latest","Status":"Up 2 hours"}]"#;
    let rows = parse_container_rows("podman", text);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "web");
    assert_eq!(rows[0].image, "nginx:latest");
    assert_eq!(rows[0].status, "Up 2 hours");
  }

  #[test]
  /// Executes the `parses_docker_ndjson` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_docker_ndjson() {
    let text = "{\"Names\":\"db,db-alias\",\"Image\":\"postgres:16\",\"State\":\"running\"}\n";
    let rows = parse_container_rows("docker", text);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "db");
    assert_eq!(rows[0].status, "running");
  }
}
