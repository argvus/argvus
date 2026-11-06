//! Toggles greetd's optional `[initial_session]` section, ARGVUS's auto-login.
//!
//! greetd starts the configured command for the configured user directly,
//! without authentication, the first time its virtual terminal is activated
//! (in practice, at boot). Logging out returns to the normal greeter for the
//! rest of the session. No credential is stored anywhere: this only changes
//! *which* account greetd starts without a login prompt, the same model GDM,
//! LightDM and SDDM use for auto-login.
//!
//! Only the `[initial_session]` section is touched; every other line of the
//! file (including comments) is preserved byte-for-byte.

use std::fs;
use std::io::Write;

use crate::error::{Error, Result};
use crate::validation;

/// Path to greetd's configuration file, owned and packaged by `argvus-greeter`.
pub const CONFIG_PATH: &str = "/etc/greetd/config.toml";

/// The session command started for an auto-logged-in user.
const SESSION_COMMAND: &str = "/usr/lib/argvus/commands/session";

const SECTION_HEADER: &str = "[initial_session]";

/// The user currently configured for auto-login, if the section is present.
pub fn current_user() -> Result<Option<String>> {
  let contents = read_config()?;
  Ok(section_body(&contents).and_then(|body| field(&body, "user")))
}

/// Enables auto-login for `user`, or disables it when `user` is `None`.
pub fn set_user(user: Option<&str>) -> Result<()> {
  if let Some(user) = user {
    validation::validate_username(user)?;
  }
  let contents = read_config()?;
  let without = remove_section(&contents);
  let updated = match user {
    None => without,
    Some(user) => {
      let mut out = without.trim_end().to_string();
      if !out.is_empty() {
        out.push_str("\n\n");
      }
      out.push_str(&format!(
        "{SECTION_HEADER}\ncommand = \"{SESSION_COMMAND}\"\nuser = \"{user}\"\n"
      ));
      out
    }
  };
  write_config(&updated)
}

/// Disables auto-login only when it names the account being locked/deleted.
pub(crate) fn clear_user(user: &str) -> Result<()> {
  clear_user_at(std::path::Path::new(CONFIG_PATH), user)
}

pub(crate) fn clear_user_at(path: &std::path::Path, user: &str) -> Result<()> {
  let contents = match fs::read_to_string(path) {
    Ok(contents) => contents,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
    Err(error) => return Err(error.into()),
  };
  if section_body(&contents)
    .and_then(|body| field(&body, "user"))
    .as_deref()
    == Some(user)
  {
    write_config_at(path, &remove_section(&contents))?;
  }
  Ok(())
}

fn read_config() -> Result<String> {
  fs::read_to_string(CONFIG_PATH).map_err(|err| {
    if err.kind() == std::io::ErrorKind::NotFound {
      Error::system(format!(
        "{CONFIG_PATH} not found; is argvus-greeter installed?"
      ))
    } else {
      Error::system(format!("reading {CONFIG_PATH}: {err}"))
    }
  })
}

fn write_config(contents: &str) -> Result<()> {
  write_config_at(std::path::Path::new(CONFIG_PATH), contents)
}

fn write_config_at(path: &std::path::Path, contents: &str) -> Result<()> {
  let dir = path
    .parent()
    .ok_or_else(|| Error::system(format!("invalid path: {CONFIG_PATH}")))?;
  let permissions = fs::metadata(path)
    .map_err(|err| Error::system(format!("reading {CONFIG_PATH}: {err}")))?
    .permissions();
  let mut temp = tempfile::Builder::new()
    .prefix(".config-tmp-")
    .suffix(".toml")
    .tempfile_in(dir)?;
  temp.write_all(contents.as_bytes())?;
  temp.as_file().sync_all()?;
  fs::set_permissions(temp.path(), permissions)?;
  if let Err(err) = temp.persist(path) {
    return Err(Error::Io(err.error));
  }
  Ok(())
}

/// Body of the `[initial_session]` section (lines between its header and the
/// next `[section]` header, or end of file), if present.
fn section_body(contents: &str) -> Option<String> {
  let mut body = String::new();
  let mut inside = false;
  for line in contents.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
      if inside {
        break;
      }
      inside = trimmed == SECTION_HEADER;
      continue;
    }
    if inside {
      body.push_str(line);
      body.push('\n');
    }
  }
  (!body.is_empty()).then_some(body)
}

/// Value of a `key = "value"` line inside a section body, quotes stripped.
fn field(body: &str, key: &str) -> Option<String> {
  body.lines().find_map(|line| {
    let (name, value) = line.split_once('=')?;
    (name.trim() == key).then(|| value.trim().trim_matches('"').to_string())
  })
}

/// Drops the `[initial_session]` section and its lines, keeping everything else.
fn remove_section(contents: &str) -> String {
  let mut out: Vec<&str> = Vec::new();
  let mut skipping = false;
  for line in contents.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
      skipping = trimmed == SECTION_HEADER;
      if skipping {
        continue;
      }
    } else if skipping {
      continue;
    }
    out.push(line);
  }
  let mut result = out.join("\n");
  if contents.ends_with('\n') && !result.is_empty() {
    result.push('\n');
  }
  result
}

#[cfg(test)]
mod tests {
  use super::*;

  const BASE: &str = "[general]\nsource_profile = false\n\n[terminal]\nvt = 1\n\n[default_session]\ncommand = \"/usr/lib/argvus/commands/greeter-session\"\nuser = \"greeter\"\n";

  #[test]
  fn adds_section_when_absent() {
    let updated = set_user_in(BASE, Some("ghost"));
    assert!(updated.contains(
      "[initial_session]\ncommand = \"/usr/lib/argvus/commands/session\"\nuser = \"ghost\"\n"
    ));
    assert!(updated.contains("[default_session]"));
  }

  #[test]
  fn replaces_existing_section() {
    let with_section = set_user_in(BASE, Some("ghost"));
    let replaced = set_user_in(&with_section, Some("alice"));
    assert_eq!(
      field(&section_body(&replaced).unwrap(), "user"),
      Some("alice".into())
    );
    assert_eq!(replaced.matches("[initial_session]").count(), 1);
  }

  #[test]
  fn removes_section_when_disabled() {
    let with_section = set_user_in(BASE, Some("ghost"));
    let removed = set_user_in(&with_section, None);
    assert!(!removed.contains("[initial_session]"));
    assert!(removed.contains("[default_session]"));
  }

  #[test]
  fn preserves_comments_outside_the_section() {
    let commented = format!("# a top comment\n{BASE}");
    let updated = set_user_in(&commented, Some("ghost"));
    assert!(updated.starts_with("# a top comment\n"));
  }

  /// Test-only helper mirroring `set_user`'s transform without touching the filesystem.
  fn set_user_in(contents: &str, user: Option<&str>) -> String {
    let without = remove_section(contents);
    match user {
      None => without,
      Some(user) => {
        let mut out = without.trim_end().to_string();
        if !out.is_empty() {
          out.push_str("\n\n");
        }
        out.push_str(&format!(
          "{SECTION_HEADER}\ncommand = \"{SESSION_COMMAND}\"\nuser = \"{user}\"\n"
        ));
        out
      }
    }
  }
  #[test]
  fn p9_cleanup_preserves_other_users_and_absent_configurations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    clear_user_at(&path, "alice").unwrap();
    assert!(!path.exists());
    let original = set_user_in(BASE, Some("bob"));
    fs::write(&path, &original).unwrap();
    clear_user_at(&path, "alice").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    fs::write(&path, BASE).unwrap();
    clear_user_at(&path, "alice").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), BASE);
  }

  #[test]
  fn p9_cleanup_preserves_mode_and_other_sections() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    fs::write(
      &path,
      format!("{}[other]\nvalue = 1\n", set_user_in(BASE, Some("alice"))),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    clear_user_at(&path, "alice").unwrap();
    assert_eq!(
      fs::read_to_string(&path).unwrap(),
      format!("{BASE}\n[other]\nvalue = 1\n")
    );
    assert_eq!(
      fs::metadata(&path).unwrap().permissions().mode() & 0o777,
      0o640
    );
  }
}
