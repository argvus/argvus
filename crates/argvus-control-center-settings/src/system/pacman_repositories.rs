//! Root side of the unofficial pacman repositories of the Control Center
//! (`system-settings package repo-*`).
//!
//! - `repo-add <name> <conf> [<fingerprint> <key-hex>]` imports and locally
//!   signs the key, writes `/etc/pacman.d/<name>.conf` and includes it.
//! - `repo-enable <name>` and `repo-disable <name>` comment or restore the
//!   repository in `/etc/pacman.conf`.
//! - `repo-remove <name> [delete-key]` removes it, and the key it recorded.
//!
//! Parsing and editing live in `argvus_control_center_core::pacman_conf`, so
//! these operations agree with the list the Control Center shows. Every
//! change to `pacman.conf` keeps a backup and replaces the file atomically.

use std::fs;
use std::io::Write as _;
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _};
use std::path::{Path, PathBuf};

use argvus_control_center_core::pacman_conf::{
  self, KEY_MARKER, PACMAN_CONF, Repository, RepositoryOrigin,
};

use super::command::{backup_file, boot_write_atomic, stream_command};

/// pacman's keyring, used read-only to read the fingerprint of a key file.
const PACMAN_GNUPG: &str = "/etc/pacman.d/gnupg";
/// Argument of `repo-remove` that also deletes the recorded key.
const DELETE_KEY: &str = "delete-key";

/// Checks the arguments of a `repo-*` action before anything runs.
pub(super) fn validate(action: &str, values: &[String]) -> Result<(), String> {
  match (action, values) {
    ("repo-add", [name, conf, rest @ ..]) => {
      validate_name(name)?;
      if conf
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\t'))
      {
        return Err("the configuration must be plain text".into());
      }
      if conf.contains(KEY_MARKER) {
        return Err("the configuration must not carry a key marker".into());
      }
      let sections = pacman_conf::parse_repository_conf(conf).map_err(|error| error.to_string())?;
      if sections.first().map(|section| section.name.as_str()) != Some(name.as_str()) {
        return Err(format!("the configuration must start with [{name}]"));
      }
      match rest {
        [] => Ok(()),
        [fingerprint, key] if pacman_conf::is_fingerprint(fingerprint) => {
          pacman_conf::decode_key(key)
            .map(|_| ())
            .ok_or_else(|| "invalid key".into())
        }
        _ => Err("repo-add takes a fingerprint and a key, or neither".into()),
      }
    }
    ("repo-enable" | "repo-disable", [name]) => validate_name(name),
    ("repo-remove", [name]) => validate_name(name),
    ("repo-remove", [name, flag]) if flag == DELETE_KEY => validate_name(name),
    _ => Err("unsupported repository operation".into()),
  }
}

fn validate_name(name: &str) -> Result<(), String> {
  if !pacman_conf::is_valid_name(name) || pacman_conf::is_official(name) {
    return Err(format!("invalid repository: {name}"));
  }
  Ok(())
}

/// Runs a `repo-*` action already checked by [`validate`].
pub(super) fn run(action: &str, values: &[String]) -> Result<(), String> {
  match (action, values) {
    ("repo-add", [name, conf]) => add(name, conf, None),
    ("repo-add", [name, conf, fingerprint, key]) => add(name, conf, Some((fingerprint, key))),
    ("repo-enable", [name]) => set_enabled(name, true),
    ("repo-disable", [name]) => set_enabled(name, false),
    ("repo-remove", [name]) => remove(name, false),
    ("repo-remove", [name, _]) => remove(name, true),
    _ => Err("unsupported repository operation".into()),
  }
}

fn read_pacman_conf() -> Result<String, String> {
  fs::read_to_string(PACMAN_CONF).map_err(|error| format!("{PACMAN_CONF}: {error}"))
}

fn repositories(pacman_conf_text: &str) -> Vec<Repository> {
  pacman_conf::list_repositories(pacman_conf_text, |path| fs::read_to_string(path).ok())
}

fn find_repository(pacman_conf_text: &str, name: &str) -> Result<Repository, String> {
  repositories(pacman_conf_text)
    .into_iter()
    .find(|repository| repository.name == name)
    .ok_or_else(|| format!("[{name}] is not configured in {PACMAN_CONF}"))
}

/// Replaces `pacman.conf` with `updated`, keeping a backup of the old one.
fn write_pacman_conf(updated: &str) -> Result<(), String> {
  let path = Path::new(PACMAN_CONF);
  let backup = backup_file(path)?;
  println!("Backup: {}", backup.display());
  boot_write_atomic(path, updated)
}

fn add(name: &str, conf: &str, key: Option<(&String, &String)>) -> Result<(), String> {
  let pacman_conf_text = read_pacman_conf()?;
  let sections = pacman_conf::parse_repository_conf(conf).map_err(|error| error.to_string())?;
  let configured = repositories(&pacman_conf_text);
  if let Some(section) = sections.iter().find(|section| {
    configured
      .iter()
      .any(|repository| repository.name == section.name)
  }) {
    return Err(format!("[{}] is already configured", section.name));
  }
  let file = pacman_conf::repository_file(name);
  if file.exists() {
    return Err(format!("{} already exists", file.display()));
  }

  // The key goes first: a repository whose packages pacman cannot verify
  // would make every later upgrade fail.
  let contents = match key {
    Some((fingerprint, key_hex)) => {
      import_key(fingerprint, key_hex)?;
      pacman_conf::with_key_marker(conf, &fingerprint.to_ascii_uppercase())
    }
    None => conf.to_owned(),
  };

  write_new_file(&file, &contents)?;
  println!("Wrote {}", file.display());
  let updated = pacman_conf::add_include(&pacman_conf_text, &file);
  if let Err(error) = write_pacman_conf(&updated) {
    // Without its include the file is unused; leave nothing half-added.
    let _ = fs::remove_file(&file);
    return Err(error);
  }
  println!("Included {} in {PACMAN_CONF}", file.display());
  Ok(())
}

/// Imports `key_hex` into pacman's keyring and signs it locally, after
/// checking that it is the key whose `fingerprint` the user confirmed.
fn import_key(fingerprint: &str, key_hex: &str) -> Result<(), String> {
  let key = pacman_conf::decode_key(key_hex).ok_or("invalid key")?;
  let directory = private_directory()?;
  let result = (|| {
    let key_file = directory.join("repository-key.gpg");
    write_new_file(&key_file, &key)?;
    let found = key_fingerprints(&key_file)?;
    if found.len() != 1 || !found[0].eq_ignore_ascii_case(fingerprint) {
      return Err(format!(
        "the key file does not hold exactly the key {fingerprint} (found: {})",
        found.join(", ")
      ));
    }
    let key_path = key_file.to_string_lossy();
    stream_command("pacman-key", &["--add", &key_path])?;
    stream_command("pacman-key", &["--lsign-key", fingerprint])
  })();
  let _ = fs::remove_dir_all(&directory);
  result
}

/// The primary fingerprints of `key_file`, read with pacman's own keyring
/// without importing anything (`--show-keys` is a dry run).
fn key_fingerprints(key_file: &Path) -> Result<Vec<String>, String> {
  let output = argvus_control_center_core::process::command("gpg")
    .args(["--homedir", PACMAN_GNUPG, "--batch", "--no-autostart"])
    .args(["--with-colons", "--show-keys"])
    .arg(key_file)
    .output()
    .map_err(|error| format!("gpg: {error}"))?;
  if !output.status.success() {
    return Err(format!(
      "gpg: {}",
      String::from_utf8_lossy(&output.stderr).trim()
    ));
  }
  Ok(
    pacman_conf::parse_key_listing(&String::from_utf8_lossy(&output.stdout))
      .into_iter()
      .map(|key| key.fingerprint)
      .collect(),
  )
}

/// A new directory only root can read, for the key file being imported.
fn private_directory() -> Result<PathBuf, String> {
  let directory = std::env::temp_dir().join(format!("argvus-repository-{}", std::process::id()));
  fs::DirBuilder::new()
    .mode(0o700)
    .create(&directory)
    .map_err(|error| format!("{}: {error}", directory.display()))?;
  Ok(directory)
}

/// Writes `contents` to a file that must not exist yet, readable by all
/// (pacman reads repository files as the user too).
fn write_new_file(path: &Path, contents: impl AsRef<[u8]>) -> Result<(), String> {
  let mut file = fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .mode(0o644)
    .open(path)
    .map_err(|error| format!("{}: {error}", path.display()))?;
  file
    .write_all(contents.as_ref())
    .and_then(|()| file.sync_all())
    .map_err(|error| format!("{}: {error}", path.display()))
}

fn set_enabled(name: &str, enabled: bool) -> Result<(), String> {
  let pacman_conf_text = read_pacman_conf()?;
  let repository = find_repository(&pacman_conf_text, name)?;
  let updated = match &repository.origin {
    RepositoryOrigin::Inline => pacman_conf::set_inline_enabled(&pacman_conf_text, name, enabled)?,
    RepositoryOrigin::Include(path) => {
      pacman_conf::set_include_enabled(&pacman_conf_text, path, enabled)?
    }
  };
  write_pacman_conf(&updated)?;
  println!("[{name}] {}", if enabled { "enabled" } else { "disabled" });
  Ok(())
}

fn remove(name: &str, delete_key: bool) -> Result<(), String> {
  let pacman_conf_text = read_pacman_conf()?;
  let repository = find_repository(&pacman_conf_text, name)?;
  match &repository.origin {
    RepositoryOrigin::Inline => {
      write_pacman_conf(&pacman_conf::remove_inline(&pacman_conf_text, name)?)?;
    }
    RepositoryOrigin::Include(path) => {
      write_pacman_conf(&pacman_conf::remove_include(&pacman_conf_text, path)?)?;
      if path.exists() {
        let backup = backup_file(path)?;
        fs::remove_file(path).map_err(|error| format!("{}: {error}", path.display()))?;
        println!("Removed {} (backup: {})", path.display(), backup.display());
      }
    }
  }
  println!("[{name}] removed");
  if delete_key && let Some(fingerprint) = &repository.key {
    stream_command("pacman-key", &["--delete", fingerprint])?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  const CONF: &str = "[argvus]\nServer = https://argvus.github.io/packages/arch/$arch\n";

  fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
  }

  #[test]
  fn adding_checks_the_name_the_configuration_and_the_key() {
    assert!(validate("repo-add", &strings(&["argvus", CONF])).is_ok());
    let fingerprint = "0123456789ABCDEF0123456789ABCDEF01234567";
    assert!(
      validate(
        "repo-add",
        &strings(&["argvus", CONF, fingerprint, "99010d"])
      )
      .is_ok()
    );
    for values in [
      vec!["core", "[core]\nServer = https://a\n"],
      vec!["other", CONF],
      vec!["argvus", "[argvus]\nXferCommand = sh\n"],
      vec!["argvus", CONF, fingerprint],
      vec!["argvus", CONF, "short", "99"],
      vec!["argvus", CONF, fingerprint, "not-hex"],
      vec!["-x", CONF],
    ] {
      assert!(
        validate("repo-add", &strings(&values)).is_err(),
        "{values:?}"
      );
    }
    assert!(
      validate(
        "repo-add",
        &strings(&["argvus", "[argvus]\r\nServer = https://a\n"])
      )
      .is_err()
    );
    let marked = pacman_conf::with_key_marker(CONF, fingerprint);
    assert!(validate("repo-add", &strings(&["argvus", &marked])).is_err());
  }

  #[test]
  fn toggling_and_removing_take_one_repository_name() {
    assert!(validate("repo-enable", &strings(&["argvus"])).is_ok());
    assert!(validate("repo-disable", &strings(&["sublime-text"])).is_ok());
    assert!(validate("repo-remove", &strings(&["argvus", "delete-key"])).is_ok());
    assert!(validate("repo-remove", &strings(&["argvus", "--force"])).is_err());
    assert!(validate("repo-disable", &strings(&["extra"])).is_err());
    assert!(validate("repo-enable", &strings(&[])).is_err());
    assert!(validate("repo-wipe", &strings(&["argvus"])).is_err());
  }
}
