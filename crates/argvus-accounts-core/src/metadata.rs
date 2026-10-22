//! Display-name handling backed by the standard GECOS full-name field.
//!
//! The display ("fantasy") name of an account is stored in the first field of
//! the GECOS entry in `/etc/passwd`, which is the traditional Unix location
//! for the user's full name. Writes never touch the file directly: they go
//! through `usermod --comment` (privileged) or `chfn --full-name`
//! (unprivileged self-service), both from shadow/util-linux.

use std::process::Command;

use crate::error::{Error, Result};

/// Maximum accepted length for a display name.
pub const MAX_DISPLAY_NAME_LEN: usize = 128;

/// Extracts the display name (first, comma-separated GECOS field).
pub fn display_name_from_gecos(gecos: &str) -> Option<&str> {
  let first = gecos.split(',').next().unwrap_or_default();
  let trimmed = first.trim();
  (!trimmed.is_empty()).then_some(trimmed)
}

/// Validates and normalizes a display name before it reaches the system.
///
/// Rejects empty values, values longer than [`MAX_DISPLAY_NAME_LEN`], leading
/// dashes (option injection into external tools) and characters that are
/// illegal or ambiguous inside GECOS (`:`, `,`, newlines, control chars).
pub fn validate_display_name(name: &str) -> Result<String> {
  let trimmed = name.trim();
  if trimmed.is_empty() {
    return Err(Error::InvalidDisplayName("value is empty".to_string()));
  }
  if trimmed.len() > MAX_DISPLAY_NAME_LEN {
    return Err(Error::InvalidDisplayName(format!(
      "exceeds {MAX_DISPLAY_NAME_LEN} bytes"
    )));
  }
  if trimmed.starts_with('-') {
    return Err(Error::InvalidDisplayName(
      "must not start with '-'".to_string(),
    ));
  }
  if let Some(bad) = trimmed
    .chars()
    .find(|&c| c == ':' || c == ',' || c == '\n' || c == '\r' || c.is_control())
  {
    return Err(Error::InvalidDisplayName(format!(
      "forbidden character: {bad:?}"
    )));
  }
  Ok(trimmed.to_string())
}

/// Runs an external system tool and translates failures into typed errors.
pub(crate) fn run_command(cmd: &mut Command, context: &str) -> Result<()> {
  let program = cmd.get_program().to_string_lossy().into_owned();
  match cmd.output() {
    Ok(output) => {
      if output.status.success() {
        Ok(())
      } else {
        Err(Error::command_failed(
          &program,
          &output.status.to_string(),
          &String::from_utf8_lossy(&output.stderr),
        ))
      }
    }
    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
      Err(Error::command_not_found(&program))
    }
    Err(err) => Err(Error::system(format!("{context}: {err}"))),
  }
}

/// Builds the `usermod` invocation that sets the GECOS comment field.
pub(crate) fn usermod_set_comment(user: &str, name: &str) -> Command {
  // Absolute path: this runs as root, so PATH must not select the binary.
  let mut cmd = Command::new("/usr/bin/usermod");
  cmd.arg("--comment").arg(name).arg("--").arg(user);
  cmd
}

/// Builds the `chfn` invocation that sets the caller's own full name.
pub(crate) fn chfn_set_full_name(user: &str, name: &str) -> Command {
  // Absolute path: the caller's PATH must not decide which binary runs.
  let mut cmd = Command::new("/usr/bin/chfn");
  cmd.arg("--full-name").arg(name).arg("--").arg(user);
  cmd
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn name_tools_run_from_absolute_paths() {
    assert_eq!(
      usermod_set_comment("alice", "Alice").get_program().to_str(),
      Some("/usr/bin/usermod")
    );
    assert_eq!(
      chfn_set_full_name("alice", "Alice").get_program().to_str(),
      Some("/usr/bin/chfn")
    );
  }

  #[test]
  fn extracts_first_gecos_field() {
    assert_eq!(
      display_name_from_gecos("William Canin,,,"),
      Some("William Canin")
    );
    assert_eq!(display_name_from_gecos("  Ghost  "), Some("Ghost"));
    assert_eq!(display_name_from_gecos(""), None);
    assert_eq!(display_name_from_gecos(",,,stuff"), None);
  }

  #[test]
  fn trims_and_accepts_reasonable_names() {
    assert_eq!(
      validate_display_name("  William Canin  ").unwrap(),
      "William Canin"
    );
    assert!(validate_display_name("Ana").is_ok());
  }

  #[test]
  fn rejects_unsafe_names() {
    for bad in [
      "",
      "   ",
      "-flag-like",
      "co:lon",
      "co,mma",
      "line\nbreak",
      "\u{7}bell",
    ] {
      assert!(validate_display_name(bad).is_err(), "should reject {bad:?}");
    }
    let long = "a".repeat(MAX_DISPLAY_NAME_LEN + 1);
    assert!(validate_display_name(&long).is_err());
  }

  #[test]
  fn external_tools_receive_validated_arguments() {
    let cmd = usermod_set_comment("wcanin", "William Canin");
    let rendered = format!("{cmd:?}");
    assert!(rendered.contains("\"/usr/bin/usermod\""));
    assert!(rendered.contains("\"--comment\""));

    let cmd = chfn_set_full_name("wcanin", "William Canin");
    let rendered = format!("{cmd:?}");
    assert!(rendered.contains("\"/usr/bin/chfn\""));
    assert!(rendered.contains("\"--\""), "end-of-options guard expected");
  }
}
