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
const DIFF_LIMIT: usize = 256 * 1024;
const COLUMNS: &str = "number,type,date,description,cleanup,active,default";
const RETENTION_DEFAULTS: &[(&str, &str)] = &[
  ("NUMBER_CLEANUP", "no"),
  ("NUMBER_MIN_AGE", "3600"),
  ("NUMBER_LIMIT", "50"),
  ("NUMBER_LIMIT_IMPORTANT", "10"),
  ("TIMELINE_CREATE", "no"),
  ("TIMELINE_CLEANUP", "no"),
  ("TIMELINE_MIN_AGE", "3600"),
  ("TIMELINE_LIMIT_HOURLY", "10"),
  ("TIMELINE_LIMIT_DAILY", "10"),
  ("TIMELINE_LIMIT_WEEKLY", "0"),
  ("TIMELINE_LIMIT_MONTHLY", "10"),
  ("TIMELINE_LIMIT_QUARTERLY", "0"),
  ("TIMELINE_LIMIT_YEARLY", "10"),
  ("EMPTY_PRE_POST_CLEANUP", "no"),
  ("EMPTY_PRE_POST_MIN_AGE", "3600"),
];
const EDITABLE_RETENTION_KEYS: &[&str] = &[
  "NUMBER_CLEANUP",
  "NUMBER_MIN_AGE",
  "NUMBER_LIMIT",
  "NUMBER_LIMIT_IMPORTANT",
  "TIMELINE_CREATE",
  "TIMELINE_CLEANUP",
  "TIMELINE_MIN_AGE",
  "TIMELINE_LIMIT_HOURLY",
  "TIMELINE_LIMIT_DAILY",
  "TIMELINE_LIMIT_WEEKLY",
  "TIMELINE_LIMIT_MONTHLY",
  "TIMELINE_LIMIT_QUARTERLY",
  "TIMELINE_LIMIT_YEARLY",
  "EMPTY_PRE_POST_CLEANUP",
  "EMPTY_PRE_POST_MIN_AGE",
];

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

/// Parse the JSON object produced by `snapper --jsonout get-config`.
pub fn parse_retention_config(output: &[u8]) -> SnapshotResult<RetentionConfig> {
  let object = json(output)?;
  let object = object
    .as_object()
    .ok_or_else(|| invalid("get-config must return an object"))?;
  let mut values = std::collections::BTreeMap::new();
  let mut unknown = std::collections::BTreeMap::new();
  for (key, raw) in object {
    let value = raw
      .as_str()
      .or_else(|| raw.get("value").and_then(Value::as_str));
    let Some(value) = value else {
      return Err(invalid(format!("invalid get-config value for {key}")));
    };
    if EDITABLE_RETENTION_KEYS.contains(&key.as_str()) {
      validate_retention_value(key, value)?;
      values.insert(
        key.clone(),
        RetentionValue {
          value: value.to_owned(),
          origin: SettingOrigin::Explicit,
        },
      );
    } else {
      unknown.insert(key.clone(), value.to_owned());
    }
  }
  for (key, default) in RETENTION_DEFAULTS {
    if !values.contains_key(*key) {
      if unknown.contains_key(*key) {
        return Err(invalid(format!("unsupported Snapper setting {key}")));
      }
      values.insert(
        (*key).into(),
        RetentionValue {
          value: (*default).into(),
          origin: SettingOrigin::SnapperDefault,
        },
      );
    }
  }
  Ok(RetentionConfig { values, unknown })
}

pub fn validate_retention_value(key: &str, value: &str) -> SnapshotResult<()> {
  if !EDITABLE_RETENTION_KEYS.contains(&key) {
    return Err(invalid("retention key is not editable"));
  }
  if matches!(
    key,
    "NUMBER_CLEANUP" | "TIMELINE_CREATE" | "TIMELINE_CLEANUP" | "EMPTY_PRE_POST_CLEANUP"
  ) {
    return if matches!(value, "yes" | "no") {
      Ok(())
    } else {
      Err(invalid("boolean setting must be yes or no"))
    };
  }
  if value.contains('-')
    && matches!(
      key,
      "NUMBER_LIMIT"
        | "NUMBER_LIMIT_IMPORTANT"
        | "TIMELINE_LIMIT_HOURLY"
        | "TIMELINE_LIMIT_DAILY"
        | "TIMELINE_LIMIT_WEEKLY"
        | "TIMELINE_LIMIT_MONTHLY"
        | "TIMELINE_LIMIT_QUARTERLY"
        | "TIMELINE_LIMIT_YEARLY"
    )
  {
    let mut parts = value.split('-');
    let min = parts.next().and_then(|part| part.parse::<u32>().ok());
    let max = parts.next().and_then(|part| part.parse::<u32>().ok());
    return match (min, max, parts.next()) {
      (Some(min), Some(max), None) if max >= min => Ok(()),
      _ => Err(invalid("invalid retention range")),
    };
  }
  value
    .parse::<u32>()
    .map(|_| ())
    .map_err(|_| invalid("setting must be a non-negative integer or supported range"))
}

pub fn build_retention_configdata(changes: &[(String, String)]) -> SnapshotResult<String> {
  if changes.is_empty() {
    return Err(invalid("no retention changes"));
  }
  let mut seen = BTreeSet::new();
  for (key, value) in changes {
    validate_retention_value(key, value)?;
    if !seen.insert(key) {
      return Err(invalid("duplicate retention key"));
    }
  }
  Ok(
    changes
      .iter()
      .map(|(key, value)| format!("{key}={value}"))
      .collect::<Vec<_>>()
      .join(" "),
  )
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

  fn comparison_snapshots(
    &self,
    config: &str,
    origin: u32,
    destination: u32,
  ) -> SnapshotResult<()> {
    validate_config(config)?;
    if origin == 0 || destination == 0 || origin == destination {
      return Err(invalid(
        "comparison requires two distinct positive snapshot IDs",
      ));
    }
    let Integration::Ready { snapshots, .. } = self.inspect(Some(config))? else {
      return Err(invalid("root snapshot integration is unavailable"));
    };
    if ![origin, destination]
      .iter()
      .all(|number| snapshots.iter().any(|snapshot| snapshot.number == *number))
    {
      return Err(invalid("snapshot no longer exists"));
    }
    Ok(())
  }

  pub fn compare(&self, config: &str, origin: u32, destination: u32) -> SnapshotResult<Comparison> {
    self.comparison_snapshots(config, origin, destination)?;
    let range = format!("{origin}..{destination}");
    let request = ProcessRequest::new(SNAPPER)
      .arg("-c")
      .arg(config)
      .arg("status")
      .arg(&range)
      .env("LC_ALL", "C.UTF-8")
      .timeout(Duration::from_secs(120))
      .output_limit(LIMIT);
    let output = self
      .runner
      .run(&request)
      .map_err(|error| SnapshotError::new(ErrorKind::Snapper, error.to_string()))
      .and_then(check_output)?;
    Ok(Comparison {
      origin,
      destination,
      changes: parse_status(&output)?,
    })
  }

  pub fn diff(
    &self,
    config: &str,
    origin: u32,
    destination: u32,
    path: &str,
  ) -> SnapshotResult<FileDiff> {
    validate_change_path(path)?;
    let comparison = self.compare(config, origin, destination)?;
    if !comparison.changes.iter().any(|change| change.path == path) {
      return Err(invalid("path is not in the current comparison"));
    }
    let range = format!("{origin}..{destination}");
    let request = ProcessRequest::new(SNAPPER)
      .arg("-c")
      .arg(config)
      .arg("diff")
      .arg(&range)
      .arg(path)
      .env("LC_ALL", "C.UTF-8")
      .timeout(Duration::from_secs(120))
      .output_limit(DIFF_LIMIT);
    let output = match self.runner.run(&request) {
      Err(ProcessError::OutputLimit) => return Ok(FileDiff::Truncated),
      Err(error) => return Err(SnapshotError::new(ErrorKind::Snapper, error.to_string())),
      Ok(output) => check_output(output)?,
    };
    let text = match String::from_utf8(output) {
      Ok(text) => text,
      Err(_) => return Ok(FileDiff::Binary),
    };
    if text.contains('\0') || text.contains("Binary files") || text.contains("Binary file") {
      return Ok(FileDiff::Binary);
    }
    Ok(FileDiff::Text(terminal_text(&text)))
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

  /// Create one verified root snapshot before an ARGVUS package update.
  pub fn create_pre_upgrade(&self) -> SnapshotResult<u32> {
    const DESCRIPTION: &str = "ARGVUS pre-upgrade package snapshot";
    let config = match self.inspect(None)? {
      Integration::Ready { config, .. } => config,
      Integration::NonBtrfs(filesystem) => {
        return Err(SnapshotError::new(
          ErrorKind::Incompatible,
          format!("root filesystem {filesystem} is not supported for pre-update snapshots"),
        ));
      }
      Integration::MissingSnapper => {
        return Err(SnapshotError::new(
          ErrorKind::Incompatible,
          "Snapper is not installed; package update was not started",
        ));
      }
      Integration::NoConfig => {
        return Err(SnapshotError::new(
          ErrorKind::Incompatible,
          "no Snapper configuration manages /; package update was not started",
        ));
      }
      Integration::ChooseConfig(_) => {
        return Err(SnapshotError::new(
          ErrorKind::Incompatible,
          "multiple Snapper configurations manage /; select one before updating packages",
        ));
      }
    };
    self.change("create", &config, DESCRIPTION)?.ok_or_else(|| {
      SnapshotError::new(
        ErrorKind::Invalid,
        "Snapper did not return the created snapshot number",
      )
    })
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

  pub fn retention(&self, config: &str) -> SnapshotResult<RetentionConfig> {
    self.comparison_snapshots_config(config)?;
    parse_retention_config(&self.snapper(&["--jsonout", "-c", config, "get-config"], false)?)
  }

  pub fn timers(&self) -> SnapperTimers {
    SnapperTimers {
      timeline: self.timer_state("snapper-timeline.timer"),
      cleanup: self.timer_state("snapper-cleanup.timer"),
    }
  }

  fn timer_state(&self, unit: &str) -> TimerState {
    let request = ProcessRequest::new("/usr/bin/systemctl")
      .arg("show")
      .arg(unit)
      .arg("--property=LoadState,UnitFileState,ActiveState")
      .arg("--no-pager")
      .env("LC_ALL", "C.UTF-8")
      .timeout(Duration::from_secs(10))
      .output_limit(16 * 1024);
    let output = match self.runner.run(&request) {
      Ok(output) => match check_output(output) {
        Ok(bytes) => bytes,
        Err(error) => return TimerState::Unavailable(error.detail),
      },
      Err(error) => return TimerState::Unavailable(error.to_string()),
    };
    let Ok(text) = std::str::from_utf8(&output) else {
      return TimerState::Unavailable("invalid systemd response".into());
    };
    let properties = text
      .lines()
      .filter_map(|line| line.split_once('='))
      .collect::<std::collections::BTreeMap<_, _>>();
    if properties.get("LoadState") == Some(&"not-found") {
      return TimerState::Missing;
    }
    if !matches!(properties.get("LoadState"), Some(&"loaded")) {
      return TimerState::Unavailable("systemd did not report a loaded unit".into());
    }
    if properties.get("ActiveState") == Some(&"active") {
      return TimerState::Active;
    }
    if matches!(
      properties.get("UnitFileState"),
      Some(&"enabled" | &"enabled-runtime")
    ) {
      TimerState::EnabledInactive
    } else {
      TimerState::Disabled
    }
  }

  fn comparison_snapshots_config(&self, config: &str) -> SnapshotResult<()> {
    validate_config(config)?;
    match self.inspect(Some(config))? {
      Integration::Ready { .. } => Ok(()),
      _ => Err(invalid("root snapshot integration is unavailable")),
    }
  }

  pub fn set_retention(
    &self,
    config: &str,
    changes: &[(String, String)],
  ) -> SnapshotResult<RetentionConfig> {
    self.comparison_snapshots_config(config)?;
    let configdata = build_retention_configdata(changes)?;
    self.snapper(&["-c", config, "set-config", &configdata], true)?;
    let saved = self.retention(config)?;
    for (key, expected) in changes {
      if saved.values.get(key).map(|value| value.value.as_str()) != Some(expected.as_str()) {
        return Err(invalid(format!("Snapper did not persist {key}")));
      }
    }
    Ok(saved)
  }

  pub fn cleanup(
    &self,
    config: &str,
    algorithms: &[CleanupAlgorithm],
  ) -> SnapshotResult<(Vec<u32>, Vec<u32>)> {
    self.comparison_snapshots_config(config)?;
    if algorithms.is_empty() {
      return Err(invalid("select at least one cleanup algorithm"));
    }
    let before = self
      .list(config)?
      .into_iter()
      .map(|snapshot| snapshot.number)
      .collect::<BTreeSet<_>>();
    for algorithm in algorithms {
      self.snapper(&["-c", config, "cleanup", algorithm.as_snapper_arg()], true)?;
    }
    let after = self
      .list(config)?
      .into_iter()
      .map(|snapshot| snapshot.number)
      .collect::<BTreeSet<_>>();
    let removed = before.difference(&after).copied().collect();
    let remaining = after.iter().copied().collect();
    Ok((removed, remaining))
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

/// Run the package transaction only after Snapper returned a validated snapshot ID.
pub fn run_pre_upgrade_transaction<R, F>(
  backend: &SnapshotBackend<R>,
  transaction: F,
) -> Result<u32, String>
where
  R: ProcessRunner,
  F: FnOnce(u32) -> Result<(), String>,
{
  let number = backend
    .create_pre_upgrade()
    .map_err(|error| format!("ARGVUS_PRE_UPDATE_BLOCKED: {error}"))?;
  transaction(number).map_err(|error| format!("ARGVUS_PRE_UPDATE_FAILED:{number}:{error}"))?;
  Ok(number)
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
    ("status", [config, origin, destination]) => {
      validate_config(config)?;
      let (origin, destination) = (parse_number(origin)?, parse_number(destination)?);
      if origin == destination {
        return Err(invalid("snapshots must differ"));
      }
      Ok(())
    }
    ("diff", [config, origin, destination, path]) => {
      validate_config(config)?;
      let (origin, destination) = (parse_number(origin)?, parse_number(destination)?);
      if origin == destination {
        return Err(invalid("snapshots must differ"));
      }
      validate_change_path(path)
    }
    ("get-retention", [config]) => validate_config(config),
    ("set-retention", [config, configdata]) => {
      validate_config(config)?;
      let changes = parse_configdata(configdata)?;
      build_retention_configdata(&changes).map(|_| ())
    }
    ("cleanup", [config, algorithms @ ..]) if !algorithms.is_empty() => {
      validate_config(config)?;
      for algorithm in algorithms {
        parse_cleanup_algorithm(algorithm)?;
      }
      Ok(())
    }
    _ => Err(invalid("unsupported snapshot request")),
  }
}

pub fn parse_cleanup_algorithm(value: &str) -> SnapshotResult<CleanupAlgorithm> {
  match value {
    "number" => Ok(CleanupAlgorithm::Number),
    "timeline" => Ok(CleanupAlgorithm::Timeline),
    "empty-pre-post" => Ok(CleanupAlgorithm::EmptyPrePost),
    _ => Err(invalid("unsupported cleanup algorithm")),
  }
}

fn parse_configdata(configdata: &str) -> SnapshotResult<Vec<(String, String)>> {
  configdata
    .split(' ')
    .map(|pair| {
      let (key, value) = pair
        .split_once('=')
        .ok_or_else(|| invalid("invalid configdata"))?;
      Ok((key.to_owned(), value.to_owned()))
    })
    .collect()
}

/// Snapper's six fixed status positions precede one separating space and the path.
pub fn parse_status(output: &[u8]) -> SnapshotResult<Vec<ChangedPath>> {
  let text = std::str::from_utf8(output).map_err(|_| invalid("status is not UTF-8"))?;
  let mut changes = Vec::new();
  let mut seen = BTreeSet::new();
  for (index, line) in text.lines().enumerate() {
    if line.is_empty() {
      continue;
    }
    let bytes = line.as_bytes();
    if bytes.len() < 9 || bytes[6] != b' ' {
      return Err(invalid(format!("invalid status line {}", index + 1)));
    }
    let indicators = &line[..6];
    let valid = [b".+-ct".as_slice(), b".p", b".u", b".g", b".x", b".a"];
    if !bytes[..6]
      .iter()
      .enumerate()
      .all(|(position, byte)| valid[position].contains(byte))
    {
      return Err(invalid(format!(
        "invalid status indicators on line {}",
        index + 1
      )));
    }
    let path = &line[7..];
    validate_change_path(path)
      .map_err(|_| invalid(format!("invalid status path on line {}", index + 1)))?;
    if !seen.insert(path) || indicators == "......" {
      return Err(invalid(format!("ambiguous status line {}", index + 1)));
    }
    let kind = match bytes[0] {
      b'+' => ChangeKind::Created,
      b'-' => ChangeKind::Removed,
      _ => ChangeKind::Modified,
    };
    changes.push(ChangedPath {
      path: path.into(),
      indicators: indicators.into(),
      kind,
      content: bytes[0] == b'c',
      file_type: bytes[0] == b't',
      permissions: bytes[1] == b'p',
      owner: bytes[2] == b'u',
      group: bytes[3] == b'g',
      xattrs: bytes[4] == b'x',
      acl: bytes[5] == b'a',
    });
  }
  Ok(changes)
}

pub fn validate_change_path(path: &str) -> SnapshotResult<()> {
  if !path.starts_with('/')
    || path == "/"
    || path.len() > 4096
    || path.chars().any(char::is_control)
    || path
      .split('/')
      .skip(1)
      .any(|part| part.is_empty() || matches!(part, "." | ".."))
  {
    return Err(invalid("invalid comparison path"));
  }
  Ok(())
}

#[cfg(test)]
mod tests;
