//! Package operations: validated pacman requests, the mirror list and service arguments.

use super::boot::{backup_file, boot_write_atomic};
use super::*;
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;

struct PackageUpdateLock {
  file: File,
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
    Ok(Self { file })
  }
}

impl Drop for PackageUpdateLock {
  fn drop(&mut self) {
    // Closing alone leaves flock held if a concurrent fork inherited the open
    // file description. CLOEXEC only closes that duplicate when the child execs.
    loop {
      // SAFETY: this guard still owns the descriptor until Drop finishes.
      if unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) } == 0 {
        break;
      }
      let error = std::io::Error::last_os_error();
      if error.kind() != std::io::ErrorKind::Interrupted {
        eprintln!("package update lock release: {error}");
        break;
      }
    }
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

pub(super) fn run_package_interactive(action: &str, values: &[String]) -> Result<(), String> {
  use std::io::IsTerminal;
  if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
    return Err("interactive package operations require a terminal".into());
  }
  run_package_mode(
    action,
    values,
    Path::new("/run/lock/argvus-package-update-snapshot.lock"),
    CommandIo::Terminal,
  )
}

fn run_package_at(action: &str, values: &[String], lock_path: &Path) -> Result<(), String> {
  run_package_mode(action, values, lock_path, CommandIo::Captured)
}

fn run_package_mode(
  action: &str,
  values: &[String],
  lock_path: &Path,
  io: CommandIo,
) -> Result<(), String> {
  // Serialize every package mutation with selective restore. Keep the guard
  // through the package command; no filesystem or snapshot checks are needed.
  let _lock = PackageUpdateLock::acquire(lock_path)?;
  if let Some(prefix) = pacman_prefix(action) {
    return io.run("pacman", &pacman_args(prefix, values, io));
  }
  match action {
    repository if repository.starts_with("repo-") => pacman_repositories::run(action, values, io),
    "clean-cache" => io.run("paccache", &cache_cleanup_args(&values[0])?),
    "mirror-apply" => {
      update_mirrorlist(&values[0])?;
      println!("ok");
      Ok(())
    }
    _ => Err("unsupported package operation".into()),
  }
}

/// Pacman operations are independent of Snapper and the underlying filesystem.
fn pacman_prefix(action: &str) -> Option<&'static [&'static str]> {
  match action {
    "install" => Some(&["-S", "--needed"]),
    "remove" => Some(&["-Rns"]),
    "remove-only" => Some(&["-Rdd"]),
    "remove-cascade" => Some(&["-Rcns"]),
    "reinstall" | "upgrade-package" => Some(&["-S"]),
    "upgrade" => Some(&["-Syyu"]),
    "refresh-db" => Some(&["-Syy"]),
    "downgrade" => Some(&["-U"]),
    _ => None,
  }
}

#[cfg(test)]
mod package_transaction_tests {
  use super::*;

  #[test]
  fn interactive_pacman_preserves_prompts_and_package_argument_boundaries() {
    let packages = ["firefox".to_owned(), "jq".to_owned()];
    assert_eq!(
      pacman_args(&["-S", "--needed"], &packages, CommandIo::Terminal),
      ["-S", "--needed", "firefox", "jq"]
    );
    assert_eq!(
      pacman_args(&["-S"], &packages, CommandIo::Captured),
      ["-S", "--noconfirm", "firefox", "jq"]
    );
    assert_eq!(pacman_args(&["-Syyu"], &[], CommandIo::Terminal), ["-Syyu"]);
  }

  #[test]
  fn package_commands_do_not_require_snapshot_integration() {
    for (action, expected) in [
      ("upgrade", vec!["-Syyu"]),
      ("upgrade-package", vec!["-S"]),
      ("install", vec!["-S", "--needed"]),
      ("reinstall", vec!["-S"]),
      ("remove", vec!["-Rns"]),
      ("remove-only", vec!["-Rdd"]),
      ("remove-cascade", vec!["-Rcns"]),
      ("downgrade", vec!["-U"]),
      ("refresh-db", vec!["-Syy"]),
    ] {
      assert_eq!(pacman_prefix(action), Some(expected.as_slice()), "{action}");
    }
    assert_eq!(pacman_prefix("unsupported"), None);
  }
}

fn pacman_args<'a>(prefix: &[&'a str], values: &'a [String], io: CommandIo) -> Vec<&'a str> {
  let mut args = prefix.to_vec();
  if io == CommandIo::Captured {
    args.push("--noconfirm");
  }
  args.extend(values.iter().map(String::as_str));
  args
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
  fn dropping_update_guard_unlocks_while_an_inherited_descriptor_remains_open() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("inherited.lock");
    let guard = PackageUpdateLock::acquire(&path).unwrap();
    // A fork in another test thread duplicates the same open file description
    // until exec closes its CLOEXEC descriptor. Cloning models that deterministically.
    let inherited = guard.file.try_clone().unwrap();
    assert!(PackageUpdateLock::acquire(&path).is_err());
    drop(guard);
    let next = PackageUpdateLock::acquire(&path)
      .expect("dropping the guard must unlock even before inherited descriptors close");
    drop(inherited);
    assert!(
      PackageUpdateLock::acquire(&path).is_err(),
      "closing an old duplicate must not unlock the next transaction"
    );
    drop(next);
    assert!(PackageUpdateLock::acquire(&path).is_ok());
  }

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
