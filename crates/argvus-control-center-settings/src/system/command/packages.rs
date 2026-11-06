//! Package operations: validated pacman requests, the mirror list and service arguments.

use super::boot::{backup_file, boot_write_atomic};
use super::streaming::stream_command;
use super::*;
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;

struct PackageUpdateLock {
  _file: File,
}

impl PackageUpdateLock {
  fn acquire(path: &Path) -> Result<Self, String> {
    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .mode(0o600)
      .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
      .open(path)
      .map_err(|error| format!("package update lock: {error}"))?;
    // SAFETY: `file` owns a valid descriptor for the duration of the lock.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
      return Err("another ARGVUS package update is in progress".into());
    }
    Ok(Self { _file: file })
  }
}

pub(super) fn valid_package_name(value: &str) -> bool {
  !value.is_empty()
    && value.len() <= 128
    // pacman runs as root and reads a leading '-' as an option; Arch package
    // names never start with '-' or '.'.
    && !value.starts_with(['-', '.'])
    && value
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '+' | '-'))
}

pub(super) fn validate_package(action: &str, values: &[String]) -> Result<(), String> {
  match action {
    repository if repository.starts_with("repo-") => pacman_repositories::validate(action, values),
    "upgrade" if values.is_empty() => Ok(()),
    "upgrade-package" if values.len() == 1 && valid_package_name(&values[0]) => Ok(()),
    "refresh-db" if values.is_empty() => Ok(()),
    "install" | "remove" | "remove-only" | "remove-cascade" | "reinstall" => {
      if values.is_empty() || values.iter().any(|value| !valid_package_name(value)) {
        return Err("invalid package name".into());
      }
      Ok(())
    }
    "downgrade" => {
      if values.len() != 1
        || !values[0].starts_with("/var/cache/pacman/pkg/")
        || values[0].contains("..")
      {
        return Err("invalid cached package path".into());
      }
      let path = Path::new(&values[0]);
      (path.is_file()
        && !path.is_symlink()
        && path
          .extension()
          .is_some_and(|e| e == "zst" || e == "xz" || e == "gz"))
      .then_some(())
      .ok_or_else(|| "cached package was not found".into())
    }
    "clean-cache"
      if values.len() == 1
        && matches!(
          values[0].as_str(),
          "keep-three" | "keep-one" | "uninstalled"
        ) =>
    {
      Ok(())
    }
    "mirror-apply" if values.len() == 1 => {
      argvus_control_center_core::pacman_conf::validate_mirrorlist(&values[0])
    }
    _ => Err("unsupported package operation".into()),
  }
}

pub(super) fn run_package(action: &str, values: &[String]) -> Result<(), String> {
  run_package_at(
    action,
    values,
    Path::new("/run/lock/argvus-package-update-snapshot.lock"),
  )
}

fn run_package_at(action: &str, values: &[String], lock_path: &Path) -> Result<(), String> {
  // Serialize every package mutation with selective restore. Keep the guard
  // through snapshot creation and pacman, without reacquiring it in upgrades.
  let _lock = PackageUpdateLock::acquire(lock_path)?;
  match action {
    repository if repository.starts_with("repo-") => pacman_repositories::run(action, values),
    "install" => stream_pacman(&["-S", "--needed"], values),
    "remove" => stream_pacman(&["-Rns"], values),
    // The two ways the Control Center offers when other installed packages
    // depend on `values`, after it listed them: only `values`, skipping the
    // dependency checks, or `values` together with those packages.
    "remove-only" => stream_pacman(&["-Rdd"], values),
    "remove-cascade" => stream_pacman(&["-Rcns"], values),
    "reinstall" => stream_pacman(&["-S"], values),
    "upgrade" => run_pre_snapshot_upgrade(&[], true),
    "upgrade-package" => run_pre_snapshot_upgrade(values, false),
    "refresh-db" => stream_pacman(&["-Syy"], &[]),
    "downgrade" => stream_pacman(&["-U"], values),
    "clean-cache" => stream_command("paccache", &cache_cleanup_args(&values[0])?),
    "mirror-apply" => {
      update_mirrorlist(&values[0])?;
      println!("ok");
      Ok(())
    }
    _ => Err("unsupported package operation".into()),
  }
}

fn run_pre_snapshot_upgrade(packages: &[String], full_upgrade: bool) -> Result<(), String> {
  #[cfg(feature = "snapshot")]
  {
    use argvus_control_center_core::process::SystemProcessRunner;
    use argvus_control_center_snapshot::backend::{SnapshotBackend, run_pre_upgrade_transaction};

    println!("ARGVUS_PRE_UPDATE_SNAPSHOT_CREATING");
    let backend = SnapshotBackend::new(SystemProcessRunner);
    let snapshot_number = run_pre_upgrade_transaction(&backend, |snapshot_number| {
      println!("ARGVUS_PRE_UPDATE_SNAPSHOT_CREATED:{snapshot_number}");
      if full_upgrade {
        stream_pacman(&["-Syu"], &[])
      } else {
        stream_pacman(&["-S"], packages)
      }
    })?;
    println!("ARGVUS_PRE_UPDATE_COMPLETED:{snapshot_number}");
    Ok(())
  }
  #[cfg(not(feature = "snapshot"))]
  {
    let _ = (packages, full_upgrade);
    Err("ARGVUS_PRE_UPDATE_BLOCKED: snapshot integration is not compiled in".into())
  }
}

pub(super) fn stream_pacman(prefix: &[&str], values: &[String]) -> Result<(), String> {
  let mut args: Vec<&str> = prefix.to_vec();
  args.push("--noconfirm");
  args.extend(values.iter().map(String::as_str));
  stream_command("pacman", &args)
}

pub(super) fn update_mirrorlist(content: &str) -> Result<(), String> {
  argvus_control_center_core::pacman_conf::validate_mirrorlist(content)?;
  let path = Path::new("/etc/pacman.d/mirrorlist");
  if path.is_symlink() {
    return Err("refusing to update a symlinked mirrorlist".into());
  }
  backup_file(path)?;
  boot_write_atomic(path, content)
}

pub(super) fn service_args(action: &str, unit: &str) -> Result<Vec<String>, String> {
  super::super::validate_plain_arg(unit)?;
  if !unit.ends_with(".service")
    || unit.starts_with('.')
    || unit.len() > 255
    || !unit
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || b":_.@-".contains(&byte))
  {
    return Err("invalid systemd unit".into());
  }
  let args = match action {
    "start" | "stop" | "restart" | "enable" | "disable" => {
      vec![action.into(), "--".into(), unit.into()]
    }
    "enable-now" => vec!["enable".into(), "--now".into(), "--".into(), unit.into()],
    "disable-now" => vec!["disable".into(), "--now".into(), "--".into(), unit.into()],
    _ => return Err("unsupported service action".into()),
  };
  Ok(args)
}

#[cfg(test)]
mod update_lock_tests {
  use super::*;

  #[test]
  fn update_lock_prevents_concurrent_transactions_and_releases_on_drop() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("update.lock");
    let first = PackageUpdateLock::acquire(&path).unwrap();
    assert!(PackageUpdateLock::acquire(&path).is_err());
    drop(first);
    assert!(PackageUpdateLock::acquire(&path).is_ok());
  }
}

#[cfg(test)]
mod audit_h1_tests {
  use super::*;
  #[test]
  fn rejects_option_and_path_units_in_validator_and_executor_arguments() {
    for unit in [
      "-Hfoo.service",
      "--help",
      "../x.service",
      "/abs/x.service",
      "-M host",
      ".hidden.service",
    ] {
      assert!(service_args("start", unit).is_err(), "{unit}");
      assert!(
        super::super::validate_args(&["service".into(), "start".into(), unit.into()]).is_err(),
        "{unit}"
      );
    }
  }
}

#[cfg(test)]
mod audit_h1_operand_tests {
  use super::*;
  #[test]
  fn option_operands_are_rejected_before_execution() {
    for value in ["--help", "-M host", "-Hfoo.service"] {
      assert!(!crate::system::time::is_valid_timezone_name(value));
      assert!(!crate::system::keyboard::is_xkb_name(value));
      assert!(!crate::system::keyboard::is_keymap_name(value));
      assert!(super::super::dns::set_resolv_conf_manual(value, "1.1.1.1").is_err());
      assert!(super::super::dns::restore_resolv_conf(value).is_err());
      for area in ["timezone", "keyboard"] {
        let action = if area == "timezone" { "set" } else { "layout" };
        assert!(super::super::run(&[area.into(), action.into(), value.into()]).is_err());
      }
    }
    assert_eq!(
      service_args("enable-now", "dbus.service").unwrap(),
      ["enable", "--now", "--", "dbus.service"]
    );
  }
}

fn cache_cleanup_args(policy: &str) -> Result<Vec<&'static str>, String> {
  match policy {
    "keep-three" => Ok(vec!["-r", "-k3"]),
    "keep-one" => Ok(vec!["-r", "-k1"]),
    "uninstalled" => Ok(vec!["-r", "-u", "-k0"]),
    _ => Err("unsupported cache policy".into()),
  }
}

#[cfg(test)]
mod audit_h5_tests {
  use super::*;
  #[test]
  fn uninstalled_cache_keeps_no_versions() {
    assert_eq!(
      cache_cleanup_args("uninstalled").unwrap(),
      ["-r", "-u", "-k0"]
    );
    assert_eq!(cache_cleanup_args("keep-three").unwrap(), ["-r", "-k3"]);
    assert!(cache_cleanup_args("unknown").is_err());
  }
}

#[cfg(test)]
mod audit_h6_tests {
  use super::*;

  #[test]
  fn package_dispatch_releases_the_lock_after_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("shared.lock");
    assert_eq!(
      run_package_at("unsupported", &[], &path).unwrap_err(),
      "unsupported package operation"
    );
    assert!(PackageUpdateLock::acquire(&path).is_ok());
  }

  #[test]
  fn package_mutations_refuse_a_lock_held_by_restore_before_execution() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("shared.lock");
    let _restore_lock = PackageUpdateLock::acquire(&path).unwrap();
    for action in [
      "install",
      "remove",
      "remove-only",
      "remove-cascade",
      "reinstall",
      "downgrade",
      "upgrade",
      "upgrade-package",
      "refresh-db",
      "clean-cache",
      "mirror-apply",
      "repo-remove",
    ] {
      let error = run_package_at(action, &["example".into()], &path).unwrap_err();
      assert!(error.contains("in progress"), "{action}: {error}");
    }
  }
}
