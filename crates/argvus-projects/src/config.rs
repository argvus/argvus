//! The projects configuration, read and written through `argvus config`, the
//! owner of the configuration. Paths are the pointers of the `projects`
//! section of the canonical configuration.

use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The `argvus` binary that serves `argvus config`: `ARGVUS_CONFIG_BIN`, else
/// the one on `PATH`.
fn binary() -> PathBuf {
  if let Some(path) = env::var_os("ARGVUS_CONFIG_BIN").filter(|value| !value.is_empty()) {
    return PathBuf::from(path);
  }
  PathBuf::from("argvus")
}

/// A command that runs `argvus config`; the caller adds the subcommand arguments.
fn config_command() -> Command {
  let mut command = Command::new(binary());
  command.arg("config");
  command
}

/// The effective value of a pointer as text, or `null` when it is missing.
pub fn get(pointer: &str) -> String {
  let output = config_command()
    .args(["get", pointer, "--effective", "--raw"])
    .stderr(Stdio::null())
    .output();
  match output {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
      .trim_end_matches('\n')
      .to_string(),
    _ => "null".to_string(),
  }
}

/// The strings of a configured array. A missing or unreadable value is empty.
pub fn array(pointer: &str) -> Vec<String> {
  let output = config_command()
    .args(["get", pointer, "--effective"])
    .stderr(Stdio::null())
    .output();
  let Ok(output) = output else {
    return Vec::new();
  };
  serde_json::from_slice::<serde_json::Value>(&output.stdout)
    .ok()
    .and_then(|value| value.as_array().cloned())
    .map(|items| {
      items
        .into_iter()
        .filter_map(|item| item.as_str().map(str::to_string))
        .collect()
    })
    .unwrap_or_default()
}

/// The same strings as `array`, named for the listing commands.
pub fn list(pointer: &str) -> Vec<String> {
  array(pointer)
}

/// Writes an array of strings. The result is `false` when `argvus config`
/// refused it.
pub fn set_array(pointer: &str, entries: &[String]) -> bool {
  let value = serde_json::Value::Array(
    entries
      .iter()
      .map(|entry| serde_json::Value::String(entry.clone()))
      .collect(),
  );
  config_command()
    .args(["set", pointer, &value.to_string()])
    .stdout(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

/// The file a reader sees for a packaged config path, through `argvus config`.
/// The packaged file is used when that fails.
pub fn read_path(relative: &str) -> String {
  let output = config_command()
    .args(["read-path", relative])
    .stderr(Stdio::null())
    .output();
  match output {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
      .trim_end()
      .to_string(),
    _ => {
      let system = env::var("ARGVUS_SYSTEM_CONFIG").unwrap_or_else(|_| "/usr/share/argvus".into());
      format!("{system}/{relative}")
    }
  }
}
