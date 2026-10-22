//! Group lookups via NSS and membership mutation through shadow tools.
//!
//! Reads go through `getgrnam()`/`getgrent()`/`getgrouplist()`. Writes never
//! edit `/etc/group` directly; they invoke `gpasswd -a/-d`, the canonical
//! shadow-utils tool, with strictly validated arguments and without any shell
//! involvement.

use std::ffi::{CStr, CString};
use std::process::Command;

use nix::unistd::Group;

use crate::error::{Error, Result};
use crate::validation;

/// Snapshot of a local group as reported by NSS.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupInfo {
  /// Group name.
  pub name: String,
  /// Numeric group id.
  pub gid: u32,
  /// Explicit member usernames (supplementary membership).
  pub members: Vec<String>,
}

/// Looks up a group by name through NSS.
pub fn get_group_by_name(name: &str) -> Result<Option<GroupInfo>> {
  validation::validate_group_name(name)?;
  Ok(
    Group::from_name(name)
      .map_err(|err| Error::system(format!("NSS group lookup failed: {err}")))?
      .map(group_from_nix),
  )
}

/// Lists every group known to NSS, sorted by name.
pub fn list_groups() -> Result<Vec<GroupInfo>> {
  let mut groups = Vec::new();
  // SAFETY: single-threaded iteration over libc global state; all data is
  // copied before the next getgrent call.
  unsafe {
    libc::setgrent();
    loop {
      let gr = libc::getgrent();
      if gr.is_null() {
        break;
      }
      if let Some(info) = group_from_ptr(gr) {
        groups.push(info);
      }
    }
    libc::endgrent();
  }
  groups.sort_by_key(|info| info.name.to_lowercase());
  Ok(groups)
}

/// Returns the groups of a user: primary group plus supplementary memberships.
pub fn groups_of_user(username: &str, primary_gid: u32) -> Result<Vec<GroupInfo>> {
  let mut gids = supplementary_gids(username, primary_gid)?;
  gids.push(primary_gid);
  gids.sort_unstable();
  gids.dedup();

  let mut groups = Vec::with_capacity(gids.len());
  for gid in gids {
    match Group::from_gid(nix::unistd::Gid::from_raw(gid)) {
      Ok(Some(group)) => groups.push(group_from_nix(group)),
      Ok(None) => groups.push(GroupInfo {
        name: format!("#{gid}"),
        gid,
        members: Vec::new(),
      }),
      Err(err) => return Err(Error::system(format!("NSS group lookup failed: {err}"))),
    }
  }
  groups.sort_by_key(|info| info.name.to_lowercase());
  Ok(groups)
}

/// Whether `group` appears among `groups`.
#[must_use]
pub fn is_member(groups: &[GroupInfo], group: &str) -> bool {
  groups.iter().any(|info| info.name == group)
}

pub(crate) enum GpasswdAction {
  Add,
  Remove,
}

pub(crate) fn gpasswd_command(action: GpasswdAction, user: &str, group: &str) -> Command {
  let flag = match action {
    GpasswdAction::Add => "-a",
    GpasswdAction::Remove => "-d",
  };
  // Absolute path: the caller's PATH must not decide which binary runs as root.
  let mut cmd = Command::new("/usr/bin/gpasswd");
  cmd.arg(flag).arg(user).arg("--").arg(group);
  cmd
}

fn supplementary_gids(username: &str, primary_gid: u32) -> Result<Vec<u32>> {
  let cname =
    CString::new(username).map_err(|err| Error::InvalidUsername(format!("{username} ({err})")))?;
  let mut size: libc::c_int = 64;
  for _ in 0..8 {
    let mut buffer = vec![0 as libc::gid_t; size.max(1) as usize];
    let mut count = size;
    // SAFETY: cname outlives the call; buffer/count are valid pointers
    // sized exactly as required by getgrouplist(3).
    let written = unsafe {
      libc::getgrouplist(
        cname.as_ptr(),
        primary_gid as libc::gid_t,
        buffer.as_mut_ptr(),
        &mut count,
      )
    };
    if written >= 0 {
      buffer.truncate(written as usize);
      return Ok(buffer);
    }
    if count > size {
      size = count;
    } else {
      size *= 2;
    }
  }
  Err(Error::system(format!(
    "could not enumerate supplementary groups for '{username}'"
  )))
}

fn group_from_nix(group: Group) -> GroupInfo {
  GroupInfo {
    name: group.name,
    gid: group.gid.as_raw(),
    members: group.mem,
  }
}

/// # Safety
/// `gr` must point to a valid `struct group` whose backing storage outlives
/// this call (as guaranteed by `getgrent`/`getgrnam` semantics).
unsafe fn group_from_ptr(gr: *const libc::group) -> Option<GroupInfo> {
  if gr.is_null() {
    return None;
  }
  // SAFETY: the caller guarantees `gr` points to valid storage per
  // getgrent/getgrnam semantics for the duration of this call.
  unsafe {
    let gr = &*gr;
    let mut members = Vec::new();
    if !gr.gr_mem.is_null() {
      let mut cursor = gr.gr_mem;
      while !(*cursor).is_null() {
        members.push(CStr::from_ptr(*cursor).to_string_lossy().into_owned());
        cursor = cursor.add(1);
      }
    }
    Some(GroupInfo {
      name: CStr::from_ptr(gr.gr_name).to_string_lossy().into_owned(),
      gid: gr.gr_gid,
      members,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn group_membership_tool_runs_from_absolute_path() {
    let command = gpasswd_command(GpasswdAction::Add, "alice", "sudo");
    assert_eq!(command.get_program().to_str(), Some("/usr/bin/gpasswd"));
  }

  #[test]
  fn invalid_group_name_short_circuits_before_nss() {
    let err = get_group_by_name("../evil").unwrap_err();
    assert!(matches!(err, Error::InvalidGroupName(_)));
  }

  #[test]
  fn root_group_is_resolvable_on_any_linux_system() {
    let root = get_group_by_name("root")
      .expect("nss works")
      .expect("root exists");
    assert_eq!(root.gid, 0);
  }

  #[test]
  fn enumeration_is_sorted_and_non_empty() {
    let groups = list_groups().expect("enumeration works");
    assert!(!groups.is_empty());
    assert!(groups.windows(2).all(|pair| pair[0].name <= pair[1].name));
  }

  #[test]
  fn user_groups_include_primary_group() {
    let mine = groups_of_user("root", 0).expect("membership works");
    assert!(is_member(&mine, "root"));
  }

  #[test]
  fn membership_helper_matches_names_only() {
    let groups = vec![GroupInfo {
      name: "wheel".to_string(),
      gid: 10,
      members: vec!["ghost".to_string()],
    }];
    assert!(is_member(&groups, "wheel"));
    assert!(!is_member(&groups, "audio"));
  }
}
