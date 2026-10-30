//! Closed Snapper actions; this entry is reached only through validated system-settings.
use argvus_control_center_core::process::SystemProcessRunner;
use argvus_control_center_snapshot::backend::{SnapshotBackend, validate_helper};

pub(super) fn run(action: &str, values: &[String]) -> Result<(), String> {
  validate_helper(action, values).map_err(|error| error.to_string())?;
  // Do not trust the inherited elevation marker for this new domain.
  if super::privileges::effective_uid() != 0 {
    return Err("snapshot helper requires authenticated root privileges".into());
  }
  let backend = SnapshotBackend::new(SystemProcessRunner);
  let reply = if action == "inspect" {
    serde_json::to_string(&backend.inspect(values.first().map(String::as_str)))
  } else {
    serde_json::to_string(&backend.change(action, &values[0], &values[1]))
  }
  .map_err(|error| error.to_string())?;
  println!("{reply}");
  Ok(())
}
