//! Password management primitives.
//!
//! Two concerns live here:
//!
//! - verifying the caller's *current* password without linking against PAM:
//!   the shadow suite ships the setuid helper `unix_chkpwd` (the very helper
//!   `pam_unix` uses), which verifies a password for the invoking uid and
//!   refuses to check other users' passwords. This keeps self-service
//!   password changes honest without new native dependencies;
//! - setting a password as root via `chpasswd`, feeding the secret through a
//!   pipe. Passwords never travel through argument vectors or shell strings.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

/// Maximum length accepted by the verification/write protocols.
///
/// `unix_chkpwd` allocates `PAM_MAX_RESP_SIZE` (512) bytes per password plus
/// room for our NUL terminator; longer secrets cannot be verified against
/// the system, so accepting them would produce accounts that can never be
/// logged into with this tool again. This is a hard protocol limit, not a
/// complexity policy.
const MAX_PASSWORD_LEN: usize = 511;

/// Well-known locations of the setuid verification helper.
const CHKPWD_CANDIDATES: [&str; 3] = [
  "/usr/bin/unix_chkpwd",
  "/usr/sbin/unix_chkpwd",
  "/sbin/unix_chkpwd",
];

/// Validates a password chosen by a user.
///
/// This tool deliberately does NOT enforce complexity policies (minimum
/// length, classes, dictionaries): that belongs to frontends and the PAM
/// stack. Only constraints required by the transport protocols are checked:
///
/// - non-empty (an empty value would create a passwordless account);
/// - no `\n`/`\r` (`chpasswd` reads line-based input);
/// - no `\0` (`unix_chkpwd` splits passwords on NUL bytes);
/// - within [`MAX_PASSWORD_LEN`] so the secret fits the helper's buffer.
pub fn validate_new_password(password: &str) -> Result<()> {
  if password.is_empty() {
    return Err(Error::InvalidPassword(
      "password cannot be empty".to_string(),
    ));
  }
  if password.len() > MAX_PASSWORD_LEN {
    return Err(Error::InvalidPassword(format!(
      "password is too long (the verification protocol accepts up to {MAX_PASSWORD_LEN} bytes)"
    )));
  }
  if password
    .chars()
    .any(|c| c == '\n' || c == '\r' || c == '\0')
  {
    return Err(Error::InvalidPassword(
      "password must not contain line breaks or NUL bytes".to_string(),
    ));
  }
  Ok(())
}

/// Returns the path of the setuid verification helper, when present.
#[must_use]
pub fn chkpwd_helper_path() -> Option<std::path::PathBuf> {
  CHKPWD_CANDIDATES
    .iter()
    .map(std::path::PathBuf::from)
    .find(|path| path.exists())
}

/// Verifies `password` for `username` using the system's setuid helper
/// (`unix_chkpwd`, shipped by linux-pam).
///
/// Contract notes derived from the helper itself:
///
/// - valid modes are `nullok`, `nonull` and `chkexpiry`; anything else makes
///   it fail unconditionally with `PAM_SYSTEM_ERR`;
/// - stdin must NOT be a tty (we always pipe);
/// - an unprivileged caller may only verify its own account; other targets
///   simply fail, which we surface as `Ok(false)`.
///
/// Returns `Ok(false)` for wrong passwords, unknown users and locked
/// accounts; only infrastructure failures (helper missing, spawn error)
/// surface as [`Error`].
pub fn verify_own_password(username: &str, password: &str) -> Result<bool> {
  let Some(helper) = chkpwd_helper_path() else {
    return Err(Error::system(
      "password verification helper unix_chkpwd was not found; \
             is the 'shadow' package installed?",
    ));
  };

  let mut child = Command::new(&helper)
    .args([username, "nullok"])
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .map_err(|err| Error::system(format!("failed to launch {}: {err}", helper.display())))?;

  if let Some(mut stdin) = child.stdin.take() {
    // Wire format (see pam_read_passwords in linux-pam): each password
    // is NUL-terminated; the helper counts passwords by splitting on
    // '\0' and reports "no password supplied" otherwise.
    stdin
      .write_all(password.as_bytes())
      .and_then(|_| stdin.write_all(b"\0"))
      .and_then(|_| stdin.flush())
      .ok();
    drop(stdin);
  }

  match child.wait() {
    Ok(status) => Ok(status.success()),
    Err(err) => Err(Error::system(format!(
      "failed to run {}: {err}",
      helper.display()
    ))),
  }
}

/// Sets `username`'s password to `new` using `chpasswd` (root context).
///
/// The secret travels through an in-memory pipe, never through argv.
pub fn set_password_admin(username: &str, new: &str) -> Result<()> {
  validate_new_password(new)?;

  // Absolute path: this runs as root, so PATH must not select the binary.
  let mut child = Command::new("/usr/bin/chpasswd")
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|err| Error::system(format!("failed to launch chpasswd: {err}")))?;

  if let Some(mut stdin) = child.stdin.take() {
    stdin
      .write_all(format!("{username}:{new}\n").as_bytes())
      .and_then(|_| stdin.flush())
      .map_err(|err| Error::system(format!("failed to send payload to chpasswd: {err}")))?;
    drop(stdin);
  }

  let output = child
    .wait_with_output()
    .map_err(|err| Error::system(format!("failed to run chpasswd: {err}")))?;
  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(Error::system(format!(
      "chpasswd rejected the new password{}",
      if stderr.trim().is_empty() {
        String::new()
      } else {
        format!(": {}", stderr.trim())
      }
    )));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_empty_linebreak_and_nul_passwords() {
    assert!(validate_new_password("").is_err());
    assert!(validate_new_password("a1b2c3d4\n").is_err());
    assert!(validate_new_password("a1b2c3d4\r").is_err());
    assert!(validate_new_password("a1b2\0c3").is_err());
    let long = "x".repeat(MAX_PASSWORD_LEN + 1);
    assert!(validate_new_password(&long).is_err());
  }

  #[test]
  fn complexity_policy_is_left_to_frontends() {
    // Single character: allowed. Complexity is not this tool's job.
    assert!(validate_new_password("a").is_ok());
    assert!(validate_new_password("12345678901234567890").is_ok());
    assert!(validate_new_password("pa ss:word\twith-weird\"chars'!").is_ok());
    assert!(validate_new_password(&"x".repeat(MAX_PASSWORD_LEN)).is_ok());
  }

  #[test]
  fn colons_and_spaces_are_valid_password_bytes() {
    assert!(validate_new_password("pa ss:word1").is_ok());
  }

  #[test]
  fn wrong_password_is_reported_without_error_when_helper_exists() {
    let Some(helper) = chkpwd_helper_path() else {
      return; // non-shadow environment: nothing to verify against
    };
    let me = crate::passwd::current_user().expect("current user resolves");
    let ok = verify_own_password(&me.username, "definitely-not-my-password").expect("helper runs");
    assert!(!ok, "wrong password must not verify ({})", helper.display());
  }

  #[test]
  fn unknown_user_verifies_false_instead_of_erroring() {
    if chkpwd_helper_path().is_none() {
      return;
    }
    let ok = verify_own_password("argvus-no-such-user-42", "whatever-1").expect("helper runs");
    assert!(!ok);
  }
}
