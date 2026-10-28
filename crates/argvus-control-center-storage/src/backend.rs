//! Isolated integration with system tools and APIs of the Storage page.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use serde_json::Value;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
  #[error("storage backend unavailable")]
  BackendUnavailable,
  #[error("command failed: {0}")]
  Command(String),
  #[error("invalid block device")]
  InvalidDevice,
  #[error("malformed structured output")]
  MalformedOutput,
}

pub fn collect(cap: &Capabilities) -> Result<StorageSnapshot, StorageError> {
  let devices = if executable(cap, "lsblk") {
    let output = run(
      "lsblk",
      &[
        "--json",
        "--bytes",
        "--output",
        "NAME,KNAME,PATH,TYPE,SIZE,MODEL,VENDOR,SERIAL,TRAN,ROTA,RM,RO,PKNAME,UUID,FSTYPE,LABEL,MOUNTPOINT",
      ],
    )?;
    parse_lsblk(&output)?
  } else {
    return Err(StorageError::BackendUnavailable);
  };
  let mounts = if executable(cap, "findmnt") {
    let output = run(
      "findmnt",
      &[
        "--json",
        "--bytes",
        "--output",
        "TARGET,SOURCE,FSTYPE,OPTIONS",
      ],
    )?;
    parse_findmnt(&output)?
  } else {
    Vec::new()
  };
  let mut filesystems = filesystems_from_mounts(&mounts);
  for filesystem in &mut filesystems {
    if let Ok(usage) = filesystem_usage(&filesystem.mountpoint.clone().unwrap_or_default()) {
      filesystem.total_bytes = Some(usage.0);
      filesystem.used_bytes = Some(usage.1);
      filesystem.available_bytes = Some(usage.2);
    }
  }
  let mut smart = Vec::new();
  if executable(cap, "smartctl") {
    for device in &devices {
      if device.kind != "disk"
        || device.name.starts_with("zram")
        || device.name.starts_with("loop")
        || device.name.starts_with("ram")
      {
        continue;
      }
      let path = format!("/dev/{}", device.name);
      if let Ok(output) = run("smartctl", &["--json", "--health", "--attributes", &path])
        && let Ok(status) = parse_smart(&path, &output)
      {
        smart.push(status);
      }
    }
  }
  Ok(StorageSnapshot {
    smart_available: cap.has_smartctl,
    smart,
    devices,
    filesystems,
    mounts,
    swap: parse_swaps(&fs::read_to_string("/proc/swaps").unwrap_or_default()),
  })
}

pub fn parse_lsblk(input: &str) -> Result<Vec<StorageDevice>, StorageError> {
  let root: Value = serde_json::from_str(input).map_err(|_| StorageError::MalformedOutput)?;
  let rows = root
    .get("blockdevices")
    .and_then(Value::as_array)
    .ok_or(StorageError::MalformedOutput)?;
  Ok(
    rows
      .iter()
      .filter_map(|row| parse_device(row, None))
      .collect(),
  )
}

fn parse_device(row: &Value, parent: Option<&str>) -> Option<StorageDevice> {
  let name = row.get("name").and_then(Value::as_str).map(terminal_text)?;
  let children = row
    .get("children")
    .and_then(Value::as_array)
    .map(|rows| {
      rows
        .iter()
        .filter_map(|child| parse_device(child, Some(&name)))
        .collect()
    })
    .unwrap_or_default();
  Some(StorageDevice {
    name,
    kind: row
      .get("type")
      .and_then(Value::as_str)
      .unwrap_or("unknown")
      .into(),
    model: text(row, "model"),
    vendor: text(row, "vendor"),
    serial: text(row, "serial"),
    size_bytes: row.get("size").and_then(Value::as_u64),
    transport: text(row, "tran"),
    rotational: boolish(row, "rota"),
    removable: boolish(row, "rm"),
    readonly: boolish(row, "ro"),
    driver: None,
    parent: parent.map(str::to_owned),
    uuid: text(row, "uuid"),
    fstype: text(row, "fstype"),
    label: text(row, "label"),
    mountpoint: text(row, "mountpoint"),
    children,
  })
}

fn text(row: &Value, key: &str) -> Option<String> {
  row
    .get(key)
    .and_then(Value::as_str)
    .filter(|v| !v.is_empty())
    .map(terminal_text)
}
fn boolish(row: &Value, key: &str) -> Option<bool> {
  row
    .get(key)
    .and_then(|v| v.as_bool().or_else(|| v.as_u64().map(|n| n != 0)))
}

pub fn parse_findmnt(input: &str) -> Result<Vec<Mount>, StorageError> {
  let root: Value = serde_json::from_str(input).map_err(|_| StorageError::MalformedOutput)?;
  let rows = root
    .get("filesystems")
    .and_then(Value::as_array)
    .ok_or(StorageError::MalformedOutput)?;
  Ok(
    rows
      .iter()
      .filter_map(|row| {
        Some(Mount {
          target: terminal_text(row.get("target")?.as_str()?),
          source: terminal_text(row.get("source")?.as_str().unwrap_or("")),
          fstype: terminal_text(row.get("fstype")?.as_str().unwrap_or("")),
          options: row
            .get("options")
            .and_then(Value::as_str)
            .unwrap_or("")
            .split(',')
            .map(terminal_text)
            .collect(),
          readonly: row
            .get("options")
            .and_then(Value::as_str)
            .is_some_and(|value| value.split(',').any(|option| option == "ro")),
        })
      })
      .collect(),
  )
}

fn filesystems_from_mounts(mounts: &[Mount]) -> Vec<Filesystem> {
  mounts
    .iter()
    .filter(|m| {
      !matches!(
        m.fstype.as_str(),
        "proc" | "sysfs" | "devtmpfs" | "devpts" | "cgroup2" | "tmpfs" | "efivarfs"
      )
    })
    .map(|m| Filesystem {
      source: m.source.clone(),
      fstype: m.fstype.clone(),
      mountpoint: Some(m.target.clone()),
      readonly: m.readonly,
      ..Default::default()
    })
    .collect()
}

fn filesystem_usage(path: &str) -> Result<(u64, u64, u64), StorageError> {
  if path.is_empty() {
    return Err(StorageError::Command("missing mountpoint".into()));
  }
  let output = run("df", &["-P", "-k", "--", path])?;
  output
    .lines()
    .skip(1)
    .find_map(|line| {
      let fields = line.split_whitespace().collect::<Vec<_>>();
      (fields.len() >= 5).then_some(()).and_then(|_| {
        Some((
          fields[1].parse::<u64>().ok()? * 1024,
          fields[2].parse::<u64>().ok()? * 1024,
          fields[3].parse::<u64>().ok()? * 1024,
        ))
      })
    })
    .ok_or(StorageError::MalformedOutput)
}

pub fn parse_swaps(input: &str) -> Vec<SwapDevice> {
  input
    .lines()
    .skip(1)
    .filter_map(|line| {
      let p = line.split_whitespace().collect::<Vec<_>>();
      if p.len() < 5 {
        return None;
      }
      Some(SwapDevice {
        source: terminal_text(p[0]),
        kind: if p[0].contains("zram") {
          "zram".into()
        } else if p[0].starts_with('/') && p[0].contains("swap") {
          "file".into()
        } else {
          "partition".into()
        },
        total_bytes: p[2].parse::<u64>().ok()? * 1024,
        used_bytes: p[3].parse::<u64>().ok()? * 1024,
      })
    })
    .collect()
}

pub fn parse_smart(device: &str, input: &str) -> Result<SmartStatus, StorageError> {
  if !is_block_device(device) {
    return Err(StorageError::InvalidDevice);
  }
  let root: Value = serde_json::from_str(input).map_err(|_| StorageError::MalformedOutput)?;
  let health = root
    .get("smart_status")
    .and_then(|v| v.get("passed"))
    .and_then(Value::as_bool)
    .map(|ok| {
      if ok {
        SmartHealth::Passed
      } else {
        SmartHealth::Failed
      }
    })
    .unwrap_or(SmartHealth::Unknown);
  let temperature_c = root
    .get("temperature")
    .and_then(|v| v.get("current"))
    .and_then(Value::as_i64)
    .or_else(|| {
      root
        .get("nvme_smart_health_information_log")
        .and_then(|v| v.get("temperature"))
        .and_then(Value::as_i64)
    });
  let power_on_hours = root
    .get("power_on_time")
    .and_then(|v| v.get("hours"))
    .and_then(Value::as_u64);
  let power_cycles = root.get("power_cycle_count").and_then(Value::as_u64);
  let percentage_used = root
    .get("nvme_smart_health_information_log")
    .and_then(|v| v.get("percentage_used"))
    .and_then(Value::as_u64)
    .and_then(|v| u8::try_from(v).ok());
  let critical_warning = root
    .get("nvme_smart_health_information_log")
    .and_then(|v| v.get("critical_warning"))
    .and_then(Value::as_u64);
  Ok(SmartStatus {
    device: device.into(),
    health,
    temperature_c,
    power_on_hours,
    power_cycles,
    percentage_used,
    critical_warning,
    details: Vec::new(),
  })
}

fn is_block_device(path: &str) -> bool {
  if !path.starts_with("/dev/") || path.contains("..") {
    return false;
  }
  #[cfg(unix)]
  {
    fs::metadata(path).is_ok_and(|m| m.file_type().is_block_device())
  }
  #[cfg(not(unix))]
  {
    false
  }
}
fn executable(cap: &Capabilities, name: &str) -> bool {
  matches!(name, "lsblk" if cap.has_lsblk)
    || matches!(name, "findmnt" if cap.has_findmnt)
    || matches!(name, "smartctl" if cap.has_smartctl)
}
fn run(program: &str, args: &[&str]) -> Result<String, StorageError> {
  let request = args
    .iter()
    .fold(ProcessRequest::new(program), |request, arg| {
      request.arg(*arg)
    });
  let out = SystemProcessRunner
    .run(&request)
    .map_err(|e| StorageError::Command(e.to_string()))?;
  if out.status != Some(0) {
    return Err(StorageError::Command(terminal_text(
      &String::from_utf8_lossy(&out.stderr),
    )));
  }
  Ok(terminal_text(&String::from_utf8_lossy(&out.stdout)))
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn parses_nested_devices_and_filters_no_fields() {
    let x = r#"{"blockdevices":[{"name":"nvme0n1","type":"disk","size":1000,"children":[{"name":"nvme0n1p1","type":"part","size":100}]}]}"#;
    let d = parse_lsblk(x).unwrap();
    assert_eq!(d[0].children[0].parent.as_deref(), Some("nvme0n1"));
  }
  #[test]
  fn parses_mounts_and_swap() {
    let m = r#"{"filesystems":[{"target":"/","source":"/dev/sda1","fstype":"ext4","options":"rw,relatime","ro":false}]}"#;
    assert_eq!(parse_findmnt(m).unwrap()[0].target, "/");
    assert_eq!(
      parse_swaps("Filename Type Size Used Priority\n/dev/zram0 partition 1024 2 -2\n")[0].kind,
      "zram"
    );
  }
  #[test]
  fn usage_thresholds_are_centralized() {
    assert_eq!(usage_level(100, 50), UsageLevel::Ok);
    assert_eq!(usage_level(100, 15), UsageLevel::Warning);
    assert_eq!(usage_level(100, 5), UsageLevel::Critical);
  }
}
