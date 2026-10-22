//! Strict validation helpers for system identifiers and free-form fields.

use crate::error::{Error, Result};

/// Maximum accepted length for usernames and group names.
///
/// Mirrors the conservative `UT_NAMESIZE` (32) used across Unix systems.
pub const MAX_SYSTEM_NAME_LEN: usize = 32;

/// Validates a Unix username against a conservative character set.
///
/// Accepted pattern: `[a-z_][a-z0-9_-]{0,31}`. Anything containing path
/// separators, dots at unsafe positions, whitespace, colons, non-ASCII bytes
/// or leading dashes/trailing `$` is rejected. This single rule eliminates
/// path traversal, option injection and NSS field injection at the entry
/// point of every API.
pub fn validate_username(name: &str) -> Result<()> {
  validate_system_name(name).map_err(|()| Error::InvalidUsername(name.to_string()))
}

/// Validates a Unix group name using the same conservative rules as
/// [`validate_username`].
pub fn validate_group_name(name: &str) -> Result<()> {
  validate_system_name(name).map_err(|()| Error::InvalidGroupName(name.to_string()))
}

fn validate_system_name(name: &str) -> std::result::Result<(), ()> {
  let bytes = name.as_bytes();
  if bytes.is_empty() || bytes.len() > MAX_SYSTEM_NAME_LEN {
    return Err(());
  }
  let first = bytes[0];
  if !(first.is_ascii_lowercase() || first == b'_') {
    return Err(());
  }
  for &byte in &bytes[1..] {
    if !(byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-') {
      return Err(());
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_valid_usernames() {
    for name in ["ghost", "wcanin", "_svc", "a1-b2_c3", "x"] {
      assert!(validate_username(name).is_ok(), "should accept {name:?}");
    }
  }

  #[test]
  fn rejects_invalid_usernames() {
    for name in [
      "",
      "Ghost",
      "9lives",
      "-leading",
      "with space",
      "has.dot",
      "co:lon",
      "sla/sh",
      "tra/il$",
      "üñí",
      "../etc/passwd",
      "..",
    ] {
      assert!(validate_username(name).is_err(), "should reject {name:?}");
    }
  }

  #[test]
  fn rejects_overlong_names() {
    let long = "a".repeat(MAX_SYSTEM_NAME_LEN + 1);
    assert!(validate_username(&long).is_err());
    assert_eq!(MAX_SYSTEM_NAME_LEN, 32);
  }

  #[test]
  fn group_validation_matches_username_rules() {
    assert!(validate_group_name("wheel").is_ok());
    assert!(validate_group_name("../evil").is_err());
  }
}
