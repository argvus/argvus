//! Structured, bounded Snapper commands. No shell or direct snapshot-directory writes.
use crate::model::*;
use argvus_control_center_core::{
  process::{ProcessError, ProcessOutput, ProcessRequest, ProcessRunner},
  sanitize::terminal_text,
};
use serde_json::Value;
use std::{
  collections::BTreeSet,
  fs,
  io::ErrorKind as IoErrorKind,
  os::unix::fs::MetadataExt,
  path::{Path, PathBuf},
  time::Duration,
};

const FINDMNT: &str = "/usr/bin/findmnt";
const SNAPPER: &str = "/usr/bin/snapper";
const LIMIT: usize = 1024 * 1024;
const DIFF_LIMIT: usize = 256 * 1024;
const COLUMNS: &str = "number,type,date,description,cleanup,active,default";
const SNAPSHOT_ROOT: &str = "/.snapshots";
const RESTORE_LOCK: &str = "/run/lock/argvus-selective-snapshot-restore.lock";
const PACKAGE_UPDATE_LOCK: &str = "/run/lock/argvus-package-update-snapshot.lock";
const RESTORE_DESCRIPTION: &str = "ARGVUS before selective file restore";
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
  restore_paths: RestorePaths,
}

#[derive(Debug, Clone)]
struct RestorePaths {
  root: PathBuf,
  snapshots: PathBuf,
  lock: PathBuf,
  package_lock: PathBuf,
  mountinfo: PathBuf,
}
impl<R: ProcessRunner> SnapshotBackend<R> {
  pub fn new(runner: R) -> Self {
    Self {
      runner,
      restore_paths: RestorePaths {
        root: PathBuf::from("/"),
        snapshots: PathBuf::from(SNAPSHOT_ROOT),
        lock: PathBuf::from(RESTORE_LOCK),
        package_lock: PathBuf::from(PACKAGE_UPDATE_LOCK),
        mountinfo: PathBuf::from("/proc/self/mountinfo"),
      },
    }
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

  /// Recheck the comparison and return a bounded review of the destination file.
  pub fn restore_preview(
    &self,
    config: &str,
    origin: u32,
    destination: u32,
    path: &str,
  ) -> SnapshotResult<RestoreReview> {
    validate_restore_path(path)?;
    let snapshots = self.root_snapshot_ids(config)?;
    if origin == destination || !snapshots.contains(&origin) || !snapshots.contains(&destination) {
      return Err(invalid(
        "comparison snapshots are invalid or no longer available",
      ));
    }

    let pair_changes = self.status(config, origin, destination)?;
    let Some(change) = pair_changes.iter().find(|change| change.path == path) else {
      return Err(invalid(
        "file is no longer present in the selected comparison",
      ));
    };
    if change.kind == ChangeKind::Removed {
      return Err(invalid(
        "the selected destination snapshot does not contain this file",
      ));
    }
    if change.file_type {
      return Err(invalid(
        "file type changes are not supported for selective restore",
      ));
    }

    let root_owner = fs::symlink_metadata(&self.restore_paths.root)
      .map_err(|error| invalid(format!("cannot inspect root directory: {error}")))?
      .uid();
    let source_path = self.snapshot_file_path(destination, path);
    let source = secure_regular_file(&source_path, &self.restore_paths.root, root_owner)
      .map_err(|error| invalid(format!("snapshot source is unsafe or unavailable: {error}")))?;
    if source.device
      != fs::metadata(&self.restore_paths.root)
        .map_err(|error| invalid(format!("cannot inspect root filesystem: {error}")))?
        .dev()
    {
      return Err(invalid(
        "snapshot source is outside the root Btrfs filesystem",
      ));
    }

    let host_path = self.host_path(path);
    ensure_not_mountpoint(&host_path, &self.restore_paths.mountinfo)?;
    let destination_identity =
      match secure_destination(&host_path, &self.restore_paths.root, root_owner) {
        Ok(identity) => identity,
        Err(error) => return Err(invalid(format!("destination is unsafe: {error}"))),
      };
    let diff = self.file_diff(config, destination, 0, path)?;
    Ok(RestoreReview {
      source_snapshot: destination,
      path: path.into(),
      destination: destination_identity,
      diff,
    })
  }

  /// Restore one regular file through Snapper after recording a root safety snapshot.
  pub fn restore_file(
    &self,
    config: &str,
    origin: u32,
    destination: u32,
    path: &str,
    expected_destination: Option<&RestoreFileIdentity>,
  ) -> SnapshotResult<RestoreResult> {
    let lock_owner = fs::symlink_metadata(&self.restore_paths.root)
      .map_err(|error| SnapshotError::new(ErrorKind::Detection, error.to_string()))?
      .uid();
    let _package_lock = RestoreLock::acquire(
      &self.restore_paths.package_lock,
      &self.restore_paths.root,
      lock_owner,
    )?;
    let _lock = RestoreLock::acquire(
      &self.restore_paths.lock,
      &self.restore_paths.root,
      lock_owner,
    )?;
    let review = self.restore_preview(config, origin, destination, path)?;
    if review.destination.as_ref() != expected_destination {
      return Err(invalid(
        "destination changed since review; review and confirm again",
      ));
    }

    let safety_snapshot = self
      .change("create", config, RESTORE_DESCRIPTION)?
      .ok_or_else(|| invalid("Snapper did not return the safety snapshot number"))?;
    let snapshots = self.root_snapshot_ids(config).map_err(|error| {
      restore_after_safety_error(safety_snapshot, "could not verify snapshots", error)
    })?;
    if !snapshots.contains(&destination) || !snapshots.contains(&safety_snapshot) {
      return Err(SnapshotError::new(
        ErrorKind::Invalid,
        format!("safety snapshot #{safety_snapshot} remains; selected snapshot changed"),
      ));
    }

    let root_owner = fs::symlink_metadata(&self.restore_paths.root)
      .map_err(|error| {
        restore_after_safety_error(
          safety_snapshot,
          "cannot inspect root",
          SnapshotError::new(ErrorKind::Detection, error.to_string()),
        )
      })?
      .uid();
    let host_path = self.host_path(path);
    ensure_not_mountpoint(&host_path, &self.restore_paths.mountinfo).map_err(|error| {
      restore_after_safety_error(safety_snapshot, "destination became a mountpoint", error)
    })?;
    let current_destination = secure_destination(&host_path, &self.restore_paths.root, root_owner)
      .map_err(|error| {
        restore_after_safety_error(safety_snapshot, "destination became unsafe", invalid(error))
      })?;
    if current_destination.as_ref() != expected_destination {
      return Err(SnapshotError::new(
        ErrorKind::Invalid,
        format!("destination changed after review; safety snapshot #{safety_snapshot} remains"),
      ));
    }

    // Snapper undoes changes from the selected snapshot forward to the safety
    // snapshot, bringing the current path back to the selected snapshot state.
    let range = format!("{destination}..{safety_snapshot}");
    self
      .snapper(&["-c", config, "undochange", &range, path], true)
      .map_err(|error| {
        restore_after_safety_error(safety_snapshot, "Snapper undochange failed", error)
      })?;

    if !restored_files_match(
      &self.snapshot_file_path(destination, path),
      &host_path,
      &self.restore_paths.root,
      root_owner,
    )
    .map_err(|error| {
      restore_after_safety_error(
        safety_snapshot,
        "could not verify restored file",
        invalid(error),
      )
    })? {
      return Err(SnapshotError::new(
        ErrorKind::Snapper,
        format!(
          "Snapper returned success, but the file still differs from snapshot #{destination}; safety snapshot #{safety_snapshot} remains"
        ),
      ));
    }

    Ok(RestoreResult {
      source_snapshot: destination,
      safety_snapshot,
      path: path.into(),
    })
  }

  fn root_snapshot_ids(&self, config: &str) -> SnapshotResult<BTreeSet<u32>> {
    let Integration::Ready {
      config: validated_config,
      snapshots,
    } = self.inspect(Some(config))?
    else {
      return Err(invalid("root snapshot integration is unavailable"));
    };
    if validated_config != config {
      return Err(invalid("Snapper configuration changed"));
    }
    Ok(
      snapshots
        .into_iter()
        .map(|snapshot| snapshot.number)
        .collect(),
    )
  }

  fn status(&self, config: &str, first: u32, second: u32) -> SnapshotResult<Vec<ChangedPath>> {
    validate_config(config)?;
    let range = format!("{first}..{second}");
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
    parse_status(&output)
  }

  fn file_diff(
    &self,
    config: &str,
    first: u32,
    second: u32,
    path: &str,
  ) -> SnapshotResult<FileDiff> {
    let range = format!("{first}..{second}");
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

  fn host_path(&self, path: &str) -> PathBuf {
    if self.restore_paths.root == Path::new("/") {
      PathBuf::from(path)
    } else {
      self.restore_paths.root.join(path.trim_start_matches('/'))
    }
  }

  fn snapshot_file_path(&self, number: u32, path: &str) -> PathBuf {
    self
      .restore_paths
      .snapshots
      .join(number.to_string())
      .join("snapshot")
      .join(path.trim_start_matches('/'))
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

fn restore_after_safety_error(
  safety_snapshot: u32,
  context: &str,
  error: SnapshotError,
) -> SnapshotError {
  SnapshotError::new(
    error.kind,
    format!(
      "{context}; safety snapshot #{safety_snapshot} remains: {}",
      error.detail
    ),
  )
}

fn validate_restore_path(path: &str) -> SnapshotResult<()> {
  validate_change_path(path)?;
  if path
    .split('/')
    .any(|component| component.eq_ignore_ascii_case(".snapshots"))
  {
    return Err(invalid(
      "Snapper internal snapshot paths cannot be restored",
    ));
  }
  Ok(())
}

fn secure_parent_directories(path: &Path, scope_root: &Path, owner: u32) -> Result<(), String> {
  let parent = path
    .parent()
    .ok_or_else(|| "destination has no parent directory".to_owned())?;
  let relative = parent
    .strip_prefix(scope_root)
    .map_err(|_| "path escapes its filesystem scope".to_owned())?;
  let mut current = scope_root.to_path_buf();
  for component in relative.components() {
    current.push(component.as_os_str());
    let metadata = fs::symlink_metadata(&current)
      .map_err(|error| format!("cannot inspect parent {}: {error}", current.display()))?;
    if !metadata.file_type().is_dir() || metadata.uid() != owner || metadata.mode() & 0o022 != 0 {
      return Err(format!(
        "parent directory {} is not a trusted root-owned directory",
        current.display()
      ));
    }
  }
  Ok(())
}

fn regular_identity(path: &Path, owner: u32) -> Result<RestoreFileIdentity, String> {
  let metadata = fs::symlink_metadata(path)
    .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
  if !metadata.file_type().is_file() {
    return Err("only regular files are supported".into());
  }
  if metadata.uid() != owner || metadata.mode() & 0o022 != 0 {
    return Err("file is not root-owned or is writable by a non-owner".into());
  }
  if metadata.nlink() != 1 {
    return Err("hard-linked files are not supported".into());
  }
  Ok(RestoreFileIdentity {
    device: metadata.dev(),
    inode: metadata.ino(),
    size: metadata.size(),
    modified_seconds: metadata.mtime(),
    modified_nanoseconds: metadata.mtime_nsec(),
    changed_seconds: metadata.ctime(),
    changed_nanoseconds: metadata.ctime_nsec(),
    owner: metadata.uid(),
    group: metadata.gid(),
    mode: metadata.mode(),
    links: metadata.nlink(),
  })
}

fn secure_regular_file(
  path: &Path,
  scope_root: &Path,
  owner: u32,
) -> Result<RestoreFileIdentity, String> {
  secure_parent_directories(path, scope_root, owner)?;
  regular_identity(path, owner)
}

fn secure_destination(
  path: &Path,
  scope_root: &Path,
  owner: u32,
) -> Result<Option<RestoreFileIdentity>, String> {
  secure_parent_directories(path, scope_root, owner)?;
  match regular_identity(path, owner) {
    Ok(identity) => Ok(Some(identity)),
    Err(_error)
      if fs::symlink_metadata(path).is_err_and(|cause| cause.kind() == IoErrorKind::NotFound) =>
    {
      Ok(None)
    }
    Err(error) => Err(error),
  }
}

fn decode_mount_path(path: &str) -> Option<PathBuf> {
  let mut decoded = String::with_capacity(path.len());
  let mut characters = path.chars();
  while let Some(character) = characters.next() {
    if character != '\\' {
      decoded.push(character);
      continue;
    }
    let octal = [characters.next()?, characters.next()?, characters.next()?];
    if !octal.iter().all(|digit| matches!(digit, '0'..='7')) {
      return None;
    }
    let value = u8::from_str_radix(&octal.iter().collect::<String>(), 8).ok()?;
    decoded.push(char::from(value));
  }
  Some(PathBuf::from(decoded))
}

fn ensure_not_mountpoint(path: &Path, mountinfo: &Path) -> SnapshotResult<()> {
  let text = fs::read_to_string(mountinfo)
    .map_err(|error| invalid(format!("cannot validate filesystem boundaries: {error}")))?;
  for line in text.lines() {
    let Some(mount_path) = line.split_whitespace().nth(4).and_then(decode_mount_path) else {
      continue;
    };
    if mount_path != Path::new("/") && path.starts_with(&mount_path) {
      return Err(invalid(
        "restoration across a mountpoint or subvolume is not supported",
      ));
    }
  }
  Ok(())
}

struct RestoreLock {
  _file: fs::File,
}

impl RestoreLock {
  fn acquire(path: &Path, scope_root: &Path, owner: u32) -> SnapshotResult<Self> {
    use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
    secure_parent_directories(path, scope_root, owner)
      .map_err(|error| invalid(format!("restore lock directory is unsafe: {error}")))?;
    let file = fs::OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .mode(0o600)
      .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
      .open(path)
      .map_err(|error| SnapshotError::new(ErrorKind::Permission, error.to_string()))?;
    let metadata = file
      .metadata()
      .map_err(|error| SnapshotError::new(ErrorKind::Permission, error.to_string()))?;
    if !metadata.file_type().is_file() || metadata.uid() != owner || metadata.mode() & 0o077 != 0 {
      return Err(invalid("restore lock file is not secure"));
    }
    // SAFETY: the file owns a valid descriptor and remains alive for the guard.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
      return Err(SnapshotError::new(
        ErrorKind::Invalid,
        "another selective restore is in progress",
      ));
    }
    Ok(Self { _file: file })
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
    ("restore-preview", [config, origin, destination, path]) => {
      validate_restore_request(config, origin, destination, path)
    }
    ("restore-file", [config, origin, destination, path, expected]) => {
      validate_restore_request(config, origin, destination, path)?;
      let identity = serde_json::from_str::<Option<RestoreFileIdentity>>(expected)
        .map_err(|_| invalid("invalid destination review token"))?;
      if let Some(identity) = identity
        && (identity.links != 1 || identity.owner != 0 || identity.mode & 0o022 != 0)
      {
        return Err(invalid("unsafe destination review token"));
      }
      Ok(())
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

fn validate_restore_request(
  config: &str,
  origin: &str,
  destination: &str,
  path: &str,
) -> SnapshotResult<()> {
  validate_config(config)?;
  let origin = parse_number(origin)?;
  let destination = parse_number(destination)?;
  if origin == destination {
    return Err(invalid("restoration requires two different snapshots"));
  }
  validate_restore_path(path)
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

// Compare only the selected file; a root-wide Snapper status can exceed its output limit.
fn restored_files_match(
  source: &Path,
  target: &Path,
  root: &Path,
  owner: u32,
) -> Result<bool, String> {
  use std::io::Read;
  use std::os::unix::fs::OpenOptionsExt;
  let source_identity = secure_destination(source, root, owner)?;
  let target_identity = secure_destination(target, root, owner)?;
  let (Some(source_identity), Some(target_identity)) = (&source_identity, &target_identity) else {
    return Ok(source_identity.is_none() && target_identity.is_none());
  };
  if source_identity.size != target_identity.size
    || source_identity.mode != target_identity.mode
    || source_identity.owner != target_identity.owner
    || source_identity.group != target_identity.group
  {
    return Ok(false);
  }
  let open = |path: &Path| {
    fs::OpenOptions::new()
      .read(true)
      .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
      .open(path)
      .map_err(|error| error.to_string())
  };
  let mut source_file = open(source)?;
  let mut target_file = open(target)?;
  let unchanged = |file: &fs::File, expected: &RestoreFileIdentity| -> Result<bool, String> {
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    Ok(
      metadata.is_file()
        && metadata.dev() == expected.device
        && metadata.ino() == expected.inode
        && metadata.size() == expected.size
        && metadata.mode() == expected.mode
        && metadata.uid() == expected.owner
        && metadata.gid() == expected.group
        && metadata.nlink() == expected.links
        && metadata.mtime() == expected.modified_seconds
        && metadata.mtime_nsec() == expected.modified_nanoseconds
        && metadata.ctime() == expected.changed_seconds
        && metadata.ctime_nsec() == expected.changed_nanoseconds,
    )
  };
  if !unchanged(&source_file, source_identity)? || !unchanged(&target_file, target_identity)? {
    return Ok(false);
  }
  let mut remaining = source_identity.size;
  let mut source_buffer = [0u8; 65536];
  let mut target_buffer = [0u8; 65536];
  while remaining > 0 {
    let count = remaining.min(source_buffer.len() as u64) as usize;
    source_file
      .read_exact(&mut source_buffer[..count])
      .map_err(|error| error.to_string())?;
    target_file
      .read_exact(&mut target_buffer[..count])
      .map_err(|error| error.to_string())?;
    if source_buffer[..count] != target_buffer[..count] {
      return Ok(false);
    }
    remaining -= count as u64;
  }
  Ok(unchanged(&source_file, source_identity)? && unchanged(&target_file, target_identity)?)
}
