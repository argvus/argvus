//! Read-only access to the local user database through NSS.
//!
//! Lookups use `getpwnam()`/`getpwuid()` (via `nix`) and enumeration uses
//! `getpwent()`, so any NSS source configured on the system (`/etc/passwd`,
//! SSSD, LDAP, ...) behaves exactly as it does for every other tool.

use std::ffi::{CStr, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use nix::unistd::{Uid, User};

use crate::error::{Error, Result};
use crate::validation;

/// Snapshot of a local account as reported by NSS.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserInfo {
  /// Login name.
  pub username: String,
  /// Numeric user id.
  pub uid: u32,
  /// Numeric primary group id.
  pub gid: u32,
  /// Raw GECOS field contents.
  pub gecos: String,
  /// Home directory path.
  pub home: PathBuf,
  /// Login shell.
  pub shell: String,
}

impl UserInfo {
  /// Returns the display name stored in the GECOS full-name field, if set.
  pub fn display_name(&self) -> Option<&str> {
    crate::metadata::display_name_from_gecos(&self.gecos)
  }
}

fn nss_failure(context: &'static str) -> impl Fn(nix::Error) -> Error {
  move |err| Error::system(format!("{context}: {err}"))
}

/// Looks up a user by name through NSS.
pub fn get_user_by_name(name: &str) -> Result<Option<UserInfo>> {
  validation::validate_username(name)?;
  Ok(
    User::from_name(name)
      .map_err(nss_failure("NSS user lookup failed"))?
      .map(|user| UserInfo {
        username: user.name,
        uid: user.uid.as_raw(),
        gid: user.gid.as_raw(),
        gecos: user.gecos.to_string_lossy().into_owned(),
        home: user.dir,
        shell: user.shell.to_string_lossy().into_owned(),
      }),
  )
}

/// Looks up a user by numeric id through NSS.
pub fn get_user_by_uid(uid: u32) -> Result<Option<UserInfo>> {
  Ok(
    User::from_uid(Uid::from_raw(uid))
      .map_err(nss_failure("NSS user lookup failed"))?
      .map(|user| UserInfo {
        username: user.name,
        uid: user.uid.as_raw(),
        gid: user.gid.as_raw(),
        gecos: user.gecos.to_string_lossy().into_owned(),
        home: user.dir,
        shell: user.shell.to_string_lossy().into_owned(),
      }),
  )
}

/// Lists all accounts known to NSS, sorted by username.
pub fn list_users() -> Result<Vec<UserInfo>> {
  let mut users = Vec::new();
  // SAFETY: single-threaded iteration over libc global state; every string
  // is copied out of the returned structure before the next call.
  unsafe {
    libc::setpwent();
    loop {
      let pw = libc::getpwent();
      if pw.is_null() {
        break;
      }
      if let Some(info) = user_from_ptr(pw) {
        users.push(info);
      }
    }
    libc::endpwent();
  }
  users.sort_by(|a, b| a.username.cmp(&b.username));
  Ok(users)
}

/// Resolves the current effective identity to a [`UserInfo`].
pub fn current_user() -> Result<UserInfo> {
  let euid = Uid::effective();
  get_user_by_uid(euid.as_raw())?.ok_or_else(|| Error::system("unable to resolve current user"))
}

/// Effective uid of the running process.
#[must_use]
pub fn effective_uid() -> u32 {
  Uid::effective().as_raw()
}

/// # Safety
/// `pw` must point to a valid `struct passwd` whose backing storage outlives
/// this call (as guaranteed by `getpwent`/`getpwnam` semantics).
unsafe fn user_from_ptr(pw: *const libc::passwd) -> Option<UserInfo> {
  if pw.is_null() {
    return None;
  }
  // SAFETY: the caller guarantees `pw` points to valid storage per
  // getpwent/getpwnam semantics for the duration of this call.
  unsafe {
    let pw = &*pw;
    Some(UserInfo {
      username: CStr::from_ptr(pw.pw_name).to_string_lossy().into_owned(),
      uid: pw.pw_uid,
      gid: pw.pw_gid,
      gecos: if pw.pw_gecos.is_null() {
        String::new()
      } else {
        CStr::from_ptr(pw.pw_gecos).to_string_lossy().into_owned()
      },
      home: PathBuf::from(OsStr::from_bytes(CStr::from_ptr(pw.pw_dir).to_bytes())),
      shell: CStr::from_ptr(pw.pw_shell).to_string_lossy().into_owned(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn invalid_username_short_circuits_before_nss() {
    let err = get_user_by_name("../etc/passwd").unwrap_err();
    assert!(matches!(err, Error::InvalidUsername(_)));
  }

  #[test]
  fn root_is_resolvable_on_any_linux_system() {
    let root = get_user_by_name("root")
      .expect("nss works")
      .expect("root exists");
    assert_eq!(root.uid, 0);
    assert_eq!(root.username, "root");
  }

  #[test]
  fn enumeration_includes_root() {
    let users = list_users().expect("enumeration works");
    assert!(users.iter().any(|u| u.username == "root"));
    assert!(
      users
        .windows(2)
        .all(|pair| pair[0].username <= pair[1].username)
    );
  }

  #[test]
  fn current_user_resolves() {
    let me = current_user().expect("current user resolves");
    assert_eq!(me.uid, Uid::effective().as_raw());
  }
}
