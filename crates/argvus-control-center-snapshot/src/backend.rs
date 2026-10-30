//! Structured, bounded Snapper commands. No shell or direct snapshot-directory writes.
use crate::model::*;
use argvus_control_center_core::{
  process::{ProcessError, ProcessOutput, ProcessRequest, ProcessRunner},
  sanitize::terminal_text,
};
use serde_json::Value;
use std::{collections::BTreeSet, io::ErrorKind as IoErrorKind, time::Duration};

const FINDMNT: &str = "/usr/bin/findmnt";
const SNAPPER: &str = "/usr/bin/snapper";
const LIMIT: usize = 1024 * 1024;
const COLUMNS: &str = "number,type,date,description,cleanup,active,default";

fn invalid(detail: impl Into<String>) -> SnapshotError {
  SnapshotError::new(ErrorKind::Invalid, detail)
}
fn json(output: &[u8]) -> SnapshotResult<Value> {
  serde_json::from_slice(output).map_err(|error| invalid(error.to_string()))
}
fn string<'a>(value: &'a Value, field: &str) -> SnapshotResult<&'a str> {
  value
    .get(field)
    .and_then(Value::as_str)
    .ok_or_else(|| invalid(format!("missing or invalid {field}")))
}

/// Only a positively identified mount containing `/` is accepted.
pub fn parse_root(output: &[u8]) -> SnapshotResult<String> {
  let parse = || {
    let value = json(output)?;
    let mounts = value["filesystems"]
      .as_array()
      .ok_or_else(|| invalid("missing filesystems"))?;
    if mounts.len() != 1 || string(&mounts[0], "target")? != "/" {
      return Err(invalid("ambiguous root mount"));
    }
    let fstype = string(&mounts[0], "fstype")?;
    if fstype.is_empty() || matches!(fstype, "unknown" | "none" | "auto") {
      return Err(invalid("unknown filesystem"));
    }
    string(&mounts[0], "source")?;
    string(&mounts[0], "options")?;
    Ok(fstype.to_owned())
  };
  parse().map_err(|error: SnapshotError| SnapshotError::new(ErrorKind::Detection, error.detail))
}

pub fn validate_config(config: &str) -> SnapshotResult<()> {
  if config.is_empty()
    || config.len() > 255
    || config.starts_with('-')
    || !config
      .chars()
      .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
  {
    return Err(invalid("invalid configuration name"));
  }
  Ok(())
}

pub fn validate_description(description: &str) -> SnapshotResult<()> {
  if description.trim().is_empty()
    || description.chars().count() > 256
    || description.chars().any(char::is_control)
  {
    return Err(invalid(
      "description must contain 1–256 printable characters",
    ));
  }
  Ok(())
}

/// Parse Snapper's config-keyed JSON and require explicit boolean protection fields.
pub fn parse_snapshots(output: &[u8], config: &str) -> SnapshotResult<Vec<Snapshot>> {
  let value = json(output)?;
  let rows = value
    .get(config)
    .and_then(Value::as_array)
    .ok_or_else(|| invalid("missing configuration snapshot array"))?;
  let mut numbers = BTreeSet::new();
  rows
    .iter()
    .map(|row| {
      let number = row["number"]
        .as_u64()
        .and_then(|number| u32::try_from(number).ok())
        .ok_or_else(|| invalid("invalid snapshot number"))?;
      if !numbers.insert(number) {
        return Err(invalid("duplicate snapshot number"));
      }
      let boolean = |field: &str| {
        row[field]
          .as_bool()
          .ok_or_else(|| invalid(format!("missing protection field {field}")))
      };
      let text = |field: &str| {
        string(row, field).map(|value| terminal_text(value).replace(['\n', '\t'], " "))
      };
      let kind = text("type")?;
      if !matches!(kind.as_str(), "single" | "pre" | "post") {
        return Err(invalid("invalid snapshot type"));
      }
      Ok(Snapshot {
        number,
        kind,
        date: text("date")?,
        description: text("description")?,
        cleanup: text("cleanup")?,
        active: boolean("active")?,
        default: boolean("default")?,
      })
    })
    .collect()
}

pub struct SnapshotBackend<R> {
  runner: R,
}
impl<R: ProcessRunner> SnapshotBackend<R> {
  pub fn new(runner: R) -> Self {
    Self { runner }
  }

  fn request(&self, program: &str, args: &[&str], mutation: bool) -> SnapshotResult<ProcessOutput> {
    let request = args
      .iter()
      .fold(ProcessRequest::new(program), |request, arg| {
        request.arg(*arg)
      })
      .env("LC_ALL", "C.UTF-8")
      .timeout(Duration::from_secs(if mutation { 120 } else { 15 }))
      .output_limit(LIMIT);
    self.runner.run(&request).map_err(|error| {
      let kind = match &error {
        ProcessError::Io(error) if error.kind() == IoErrorKind::PermissionDenied => {
          ErrorKind::Permission
        }
        _ => ErrorKind::Snapper,
      };
      SnapshotError::new(kind, error.to_string())
    })
  }

  fn checked(&self, program: &str, args: &[&str], mutation: bool) -> SnapshotResult<Vec<u8>> {
    let output = self.request(program, args, mutation)?;
    check_output(output)
  }

  pub fn root_filesystem(&self) -> SnapshotResult<String> {
    self
      .checked(
        FINDMNT,
        &[
          "--json",
          "--target",
          "/",
          "--output",
          "FSTYPE,SOURCE,TARGET,OPTIONS",
        ],
        false,
      )
      .and_then(|output| parse_root(&output))
      .map_err(|error| SnapshotError::new(ErrorKind::Detection, error.detail))
  }

  fn snapper(&self, args: &[&str], mutation: bool) -> SnapshotResult<Vec<u8>> {
    self.checked(SNAPPER, args, mutation)
  }

  /// Query config values through Snapper rather than sourcing `/etc` files.
  fn root_configs(&self) -> SnapshotResult<Vec<String>> {
    let output = self.snapper(
      &["--jsonout", "list-configs", "--columns", "config,subvolume"],
      false,
    )?;
    let value = json(&output)?;
    let rows = value["configs"]
      .as_array()
      .ok_or_else(|| invalid("missing configs array"))?;
    let mut configs = BTreeSet::new();
    for row in rows {
      let name = string(row, "config")?;
      validate_config(name)?;
      if string(row, "subvolume")? != "/" {
        continue;
      }
      let settings = json(&self.snapper(&["--jsonout", "-c", name, "get-config"], false)?)?;
      if string(&settings, "SUBVOLUME")? == "/" && string(&settings, "FSTYPE")? == "btrfs" {
        configs.insert(name.to_owned());
      }
    }
    Ok(configs.into_iter().collect())
  }

  pub fn inspect(&self, selected: Option<&str>) -> SnapshotResult<Integration> {
    let filesystem = self.root_filesystem()?;
    if filesystem != "btrfs" {
      return Ok(Integration::NonBtrfs(filesystem));
    }
    // NotFound is the only condition interpreted as an absent executable.
    let help_request = ProcessRequest::new(SNAPPER)
      .arg("--help")
      .env("LC_ALL", "C.UTF-8")
      .timeout(Duration::from_secs(15))
      .output_limit(LIMIT);
    let help = match self.runner.run(&help_request) {
      Err(ProcessError::Io(error)) if error.kind() == IoErrorKind::NotFound => {
        return Ok(Integration::MissingSnapper);
      }
      Err(error) => return Err(SnapshotError::new(ErrorKind::Snapper, error.to_string())),
      Ok(output) => check_output(output)?,
    };
    if !String::from_utf8_lossy(&help).contains("--jsonout") {
      return Err(SnapshotError::new(
        ErrorKind::Incompatible,
        "Snapper JSON output is required",
      ));
    }
    let configs = self.root_configs()?;
    if configs.is_empty() {
      return Ok(Integration::NoConfig);
    }
    let config = match selected {
      Some(config) if configs.iter().any(|name| name == config) => config.to_owned(),
      Some(_) => return Err(invalid("configuration no longer belongs to /")),
      None if configs.len() == 1 => configs[0].clone(),
      None => return Ok(Integration::ChooseConfig(configs)),
    };
    let snapshots = self.list(&config)?;
    Ok(Integration::Ready { config, snapshots })
  }

  fn list(&self, config: &str) -> SnapshotResult<Vec<Snapshot>> {
    parse_snapshots(
      &self.snapper(
        &[
          "--jsonout",
          "-c",
          config,
          "list",
          "--disable-used-space",
          "--columns",
          COLUMNS,
        ],
        false,
      )?,
      config,
    )
  }

  /// Repeat all root/config/protection checks in the elevated process, immediately before mutation.
  pub fn change(&self, action: &str, config: &str, value: &str) -> SnapshotResult<Option<u32>> {
    validate_config(config)?;
    let number = match action {
      "create" => {
        validate_description(value)?;
        None
      }
      "delete" => Some(parse_number(value)?),
      _ => return Err(invalid("unsupported snapshot operation")),
    };
    let Integration::Ready { snapshots, .. } = self.inspect(Some(config))? else {
      return Err(invalid("root snapshot integration is unavailable"));
    };
    match number {
      Some(number) => {
        if !snapshots
          .iter()
          .any(|snapshot| snapshot.number == number && snapshot.deletable())
        {
          return Err(invalid("snapshot is absent, active or protected"));
        }
        self.snapper(&["-c", config, "delete", &number.to_string()], true)?;
        Ok(None)
      }
      None => {
        let output = self.snapper(
          &[
            "-c",
            config,
            "create",
            "--type",
            "single",
            "--description",
            value,
            "--print-number",
          ],
          true,
        )?;
        let text = std::str::from_utf8(&output).map_err(|_| invalid("invalid created number"))?;
        Ok(Some(parse_number(text.trim())?))
      }
    }
  }
}

pub fn parse_number(value: &str) -> SnapshotResult<u32> {
  if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
    return Err(invalid("expected one positive snapshot number"));
  }
  value
    .parse::<u32>()
    .ok()
    .filter(|number| *number > 0)
    .ok_or_else(|| invalid("expected one positive snapshot number"))
}

pub fn check_output(output: ProcessOutput) -> SnapshotResult<Vec<u8>> {
  if output.timed_out {
    return Err(SnapshotError::new(
      ErrorKind::Timeout,
      "operation interrupted; refresh before retrying",
    ));
  }
  if output.status == Some(0) {
    return Ok(output.stdout);
  }
  let detail: String = terminal_text(&String::from_utf8_lossy(&output.stderr))
    .chars()
    .take(512)
    .collect();
  let lower = detail.to_ascii_lowercase();
  let kind = if matches!(output.status, Some(126 | 127)) {
    ErrorKind::Authentication
  } else if lower.contains("permission")
    || lower.contains("not authorized")
    || lower.contains("access denied")
    || lower.contains("accessdenied")
  {
    ErrorKind::Permission
  } else {
    ErrorKind::Snapper
  };
  Err(SnapshotError::new(kind, detail))
}

/// Restricted system-settings grammar, validated again after elevation.
pub fn validate_helper(action: &str, values: &[String]) -> SnapshotResult<()> {
  match (action, values) {
    ("inspect", []) => Ok(()),
    ("inspect", [config]) => validate_config(config),
    ("create", [config, description]) => {
      validate_config(config)?;
      validate_description(description)
    }
    ("delete", [config, number]) => {
      validate_config(config)?;
      parse_number(number).map(|_| ())
    }
    _ => Err(invalid("unsupported snapshot request")),
  }
}

#[cfg(test)]
mod tests;
