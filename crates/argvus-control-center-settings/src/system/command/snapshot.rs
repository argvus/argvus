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
  let reply = match action {
    "inspect" => serde_json::to_string(&backend.inspect(values.first().map(String::as_str))),
    "status" => serde_json::to_string(&backend.compare(
      &values[0],
      values[1].parse().map_err(|error| format!("{error}"))?,
      values[2].parse().map_err(|error| format!("{error}"))?,
    )),
    "diff" => serde_json::to_string(&backend.diff(
      &values[0],
      values[1].parse().map_err(|error| format!("{error}"))?,
      values[2].parse().map_err(|error| format!("{error}"))?,
      &values[3],
    )),
    "get-retention" => serde_json::to_string(&backend.retention(&values[0])),
    "set-retention" => {
      serde_json::to_string(&backend.set_retention(&values[0], &parse_changes(&values[1])?))
    }
    "cleanup" => {
      let algorithms = values[1..]
        .iter()
        .map(|value| argvus_control_center_snapshot::backend::parse_cleanup_algorithm(value))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
      serde_json::to_string(&backend.cleanup(&values[0], &algorithms))
    }
    _ => serde_json::to_string(&backend.change(action, &values[0], &values[1])),
  }
  .map_err(|error| error.to_string())?;
  println!("{reply}");
  Ok(())
}

fn parse_changes(configdata: &str) -> Result<Vec<(String, String)>, String> {
  configdata
    .split(' ')
    .map(|pair| {
      let (key, value) = pair
        .split_once('=')
        .ok_or_else(|| "invalid configdata".to_owned())?;
      Ok((key.to_owned(), value.to_owned()))
    })
    .collect()
}
