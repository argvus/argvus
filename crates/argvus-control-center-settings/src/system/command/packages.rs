//! Package operations: validated pacman requests, the mirror list and service arguments.

use super::boot::{backup_file, boot_write_atomic};
use super::streaming::stream_command;
use super::*;

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
      let content = &values[0];
      (!content.is_empty()
        && content.len() <= 256 * 1024
        && content
          .lines()
          .any(|line| line.trim_start().starts_with("Server = "))
        && content
          .chars()
          .all(|c| !c.is_control() || c == '\n' || c == '\t'))
      .then_some(())
      .ok_or_else(|| "invalid mirrorlist".into())
    }
    _ => Err("unsupported package operation".into()),
  }
}

pub(super) fn run_package(action: &str, values: &[String]) -> Result<(), String> {
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
    "upgrade" => stream_pacman(&["-Syu"], &[]),
    "upgrade-package" => stream_pacman(&["-S"], values),
    "refresh-db" => stream_pacman(&["-Syy"], &[]),
    "downgrade" => stream_pacman(&["-U"], values),
    "clean-cache" => stream_command(
      "paccache",
      &[
        "-r",
        match values[0].as_str() {
          "keep-three" => "-k3",
          "keep-one" => "-k1",
          _ => "-u",
        },
      ],
    ),
    "mirror-apply" => {
      update_mirrorlist(&values[0])?;
      println!("ok");
      Ok(())
    }
    _ => Err("unsupported package operation".into()),
  }
}

pub(super) fn stream_pacman(prefix: &[&str], values: &[String]) -> Result<(), String> {
  let mut args: Vec<&str> = prefix.to_vec();
  args.push("--noconfirm");
  args.extend(values.iter().map(String::as_str));
  stream_command("pacman", &args)
}

pub(super) fn update_mirrorlist(content: &str) -> Result<(), String> {
  let path = Path::new("/etc/pacman.d/mirrorlist");
  if path.is_symlink() {
    return Err("refusing to update a symlinked mirrorlist".into());
  }
  backup_file(path)?;
  boot_write_atomic(path, content)
}

pub(super) fn service_args(action: &str, unit: &str) -> Result<Vec<String>, String> {
  if !unit.ends_with(".service")
    || unit.is_empty()
    || unit.len() > 255
    || unit
      .chars()
      .any(|value| value.is_control() || value.is_whitespace())
  {
    return Err("invalid systemd unit".into());
  }
  let args = match action {
    "start" | "stop" | "restart" | "enable" | "disable" => vec![action.into(), unit.into()],
    "enable-now" => vec!["enable".into(), "--now".into(), unit.into()],
    "disable-now" => vec!["disable".into(), "--now".into(), unit.into()],
    _ => return Err("unsupported service action".into()),
  };
  Ok(args)
}
