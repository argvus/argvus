//! Structured administration API. Secrets travel over stdin, never argv.
use std::process::Command;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
  AccountManager, Error, Result, autologin, avatar, groups, metadata, passwd, password, validation,
};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
enum Request {
  Snapshot,
  Create {
    user: String,
    name: String,
    shell: String,
    groups: Vec<String>,
    #[serde(default)]
    password: Option<String>,
  },
  Edit {
    user: String,
    name: String,
    shell: String,
    primary_group: String,
    groups: Vec<String>,
  },
  Password {
    user: String,
    old: String,
    new: String,
    confirm: String,
  },
  Lock {
    user: String,
    locked: bool,
  },
  ExpirePassword {
    user: String,
  },
  Delete {
    user: String,
    #[serde(default)]
    remove_home: bool,
  },
  CreateGroup {
    group: String,
  },
  EditGroup {
    group: String,
    name: String,
    members: Vec<String>,
  },
  DeleteGroup {
    group: String,
  },
  Avatar {
    user: String,
    path: String,
  },
  AutoLogin {
    user: String,
    enabled: bool,
  },
}

/// The deployed avatar of `info` as a path string, or `""` when none is set.
/// The snapshot field is always present so that clients can tell "no avatar"
/// from a missing field.
fn avatar_of(info: &passwd::UserInfo) -> String {
  avatar::find_avatar(&info.home)
    .map(|path| path.display().to_string())
    .unwrap_or_default()
}

fn invalid(message: impl Into<String>) -> Error {
  Error::InvalidOperation(message.into())
}

fn actor_uid() -> Result<u32> {
  if passwd::effective_uid() == 0
    && let Ok(uid) = std::env::var("PKEXEC_UID")
  {
    return uid.parse().map_err(|_| invalid("invalid polkit caller"));
  }
  Ok(passwd::effective_uid())
}

fn shells() -> Result<Vec<String>> {
  let contents =
    std::fs::read_to_string("/etc/shells").map_err(|err| Error::system(err.to_string()))?;
  Ok(
    contents
      .lines()
      .map(str::trim)
      .filter(|line| line.starts_with('/'))
      .map(str::to_owned)
      .collect(),
  )
}

fn validate_shell(shell: &str) -> Result<()> {
  if !shells()?.iter().any(|candidate| candidate == shell) {
    return Err(invalid("select a login shell listed in /etc/shells"));
  }
  Ok(())
}

fn validate_groups<Name: AsRef<str>>(names: &[Name]) -> Result<()> {
  for name in names {
    let name = name.as_ref();
    validation::validate_group_name(name)?;
    if groups::get_group_by_name(name)?.is_none() {
      return Err(Error::GroupNotFound(name.to_owned()));
    }
  }
  Ok(())
}

/// System accounts: every uid below 1000 and `nobody` (65534).
fn is_system_uid(uid: u32) -> bool {
  uid < 1000 || uid == 65534
}

fn target(user: &str) -> Result<passwd::UserInfo> {
  validation::validate_username(user)?;
  passwd::get_user_by_name(user)?.ok_or_else(|| Error::UserNotFound(user.into()))
}

/// Refuses system accounts (uid below 1000 and `nobody`). Used for edits, where
/// the administrator may still change its own profile.
fn protect_system_account(user: &str) -> Result<()> {
  let info = target(user)?;
  if is_system_uid(info.uid) {
    return Err(invalid("cannot modify a system account"));
  }
  Ok(())
}

fn protect_account(user: &str) -> Result<()> {
  let info = target(user)?;
  if is_system_uid(info.uid) || info.uid == actor_uid()? {
    return Err(invalid(
      "cannot delete or lock a system account or your own account",
    ));
  }
  Ok(())
}

fn run(program: &str, args: &[&str]) -> Result<()> {
  metadata::run_command(Command::new(program).args(args), "account operation failed")
}

/// Executes a bounded JSON request. Only snapshots are available without root.
pub fn execute(input: &str) -> Result<Value> {
  if input.len() > 65536 {
    return Err(invalid("request too large"));
  }
  let request = serde_json::from_str(input).map_err(|_| invalid("invalid account request"))?;
  execute_request(request)
}

/// Groups whose members administer the system: `wheel` is Arch's sudoers and
/// polkit administrator group, `sudo` the group earlier ARGVUS releases used.
/// The polkit rule `50-argvus-accounts.rules` and the Control Center use the
/// same pair.
pub const ADMIN_GROUPS: [&str; 2] = ["wheel", "sudo"];

/// Whether `name` is one of the [`ADMIN_GROUPS`].
#[must_use]
pub fn is_admin_group(name: &str) -> bool {
  ADMIN_GROUPS.contains(&name)
}

/// Whether `uid` may administer accounts: root, or a member of an admin group.
fn is_administrator(uid: u32) -> Result<bool> {
  if uid == 0 {
    return Ok(true);
  }
  let Some(user) = passwd::get_user_by_uid(uid)? else {
    return Ok(false);
  };
  Ok(
    groups::groups_of_user(&user.username, user.gid)?
      .iter()
      .any(|group| is_admin_group(&group.name)),
  )
}

fn execute_request(request: Request) -> Result<Value> {
  if matches!(request, Request::Snapshot) {
    return snapshot();
  }
  if passwd::effective_uid() != 0 {
    return Err(Error::PermissionDenied(
      "administration requires pkexec /usr/lib/argvus/commands/accounts manage".into(),
    ));
  }
  apply_request(
    request,
    &group_facts,
    &mut run,
    &mut password::set_password_admin,
  )
}

/// Users, groups and shells visible to the caller. Without administration
/// rights an account only sees itself and the groups it belongs to.
fn snapshot() -> Result<Value> {
  let manager = AccountManager::new();
  let actor = actor_uid()?;
  let is_admin = is_administrator(actor)?;
  let mut users = Vec::new();
  for info in manager.list_users(true)? {
    // Without administration rights an account only sees its own entry.
    if !is_admin && info.uid != actor {
      continue;
    }
    let memberships = groups::groups_of_user(&info.username, info.gid)?;
    users.push(json!({
              "user": info.username, "name": info.display_name().unwrap_or(""),
              "uid": info.uid, "gid": info.gid, "home": info.home, "shell": info.shell,
              "avatar": avatar_of(&info),
              "primary_group": memberships.iter().find(|group| group.gid == info.gid).map(|group| group.name.clone()).unwrap_or_default(),
              "groups": memberships.iter().filter(|group| group.gid != info.gid).map(|group| group.name.clone()).collect::<Vec<_>>()
          }));
  }
  // Without administration rights only the groups the caller belongs to are listed.
  let own_groups: Vec<String> = match passwd::get_user_by_uid(actor)? {
    Some(user) => groups::groups_of_user(&user.username, user.gid)?
      .into_iter()
      .map(|group| group.name)
      .collect(),
    None => Vec::new(),
  };
  let group_details: Vec<_> = groups::list_groups()?
    .into_iter()
    .filter(|group| is_admin || own_groups.contains(&group.name))
    .collect();
  Ok(json!({
      "users": users,
      "groups": group_details.iter().map(|group| group.name.clone()).collect::<Vec<_>>(),
      "group_details": group_details.into_iter().map(|group| json!({
          "name": group.name,
          "gid": group.gid,
          // Membership of other accounts is only shown to administrators.
          "members": if is_admin { group.members } else { Vec::new() }
      })).collect::<Vec<_>>(),
      "shells": shells()?,
      "actor_uid": actor,
      "actor_is_admin": is_admin,
      // Best-effort: a missing/unreadable greetd config must not break the snapshot.
      "autologin_user": autologin::current_user().unwrap_or(None)
  }))
}

/// The GID and the supplementary members of a group.
type GroupFacts = (u32, Vec<String>);

/// The [`GroupFacts`] of `group`.
fn group_facts(group: &str) -> Result<GroupFacts> {
  let group =
    groups::get_group_by_name(group)?.ok_or_else(|| Error::GroupNotFound(group.into()))?;
  Ok((group.gid, group.members))
}

/// Refuses renaming or deleting a system group (GID below 1000) or an
/// administrator group: other accounts and services depend on their names, and
/// losing an administrator group locks every administrator out.
fn protect_system_group(group: &str, gid: u32) -> Result<()> {
  if gid < 1000 || is_admin_group(group) {
    return Err(invalid(
      "cannot rename or delete a system or administrator group",
    ));
  }
  Ok(())
}

/// Username of the account that requested the operation.
fn actor_name() -> Result<Option<String>> {
  Ok(passwd::get_user_by_uid(actor_uid()?)?.map(|user| user.username))
}

fn apply_request(
  request: Request,
  group_info: &dyn Fn(&str) -> Result<GroupFacts>,
  run: &mut impl FnMut(&str, &[&str]) -> Result<()>,
  set_password: &mut impl FnMut(&str, &str) -> Result<()>,
) -> Result<Value> {
  match request {
    // `execute_request` answers snapshots before it calls this function.
    Request::Snapshot => unreachable!(),
    Request::Create {
      user,
      name,
      shell,
      groups,
      password,
    } => {
      create_user(&user, &name, &shell, &groups, password, run, set_password)?;
    }
    Request::Edit {
      user,
      name,
      shell,
      primary_group,
      groups,
    } => {
      edit_user(&user, &name, &shell, &primary_group, &groups, run)?;
    }
    Request::Password {
      user,
      old,
      new,
      confirm,
    } => {
      change_password(&user, &old, &new, &confirm, set_password)?;
    }
    Request::Lock { user, locked } => {
      protect_account(&user)?;
      run(
        "/usr/bin/usermod",
        &[if locked { "--lock" } else { "--unlock" }, "--", &user],
      )?;
    }
    Request::ExpirePassword { user } => {
      protect_system_account(&user)?;
      run("/usr/bin/chage", &["--lastday", "0", "--", &user])?;
    }
    Request::Delete { user, remove_home } => {
      protect_account(&user)?;
      // Never force deletion of logged-in users; home removal is explicit in the UI.
      let args = if remove_home {
        vec!["--remove", "--", user.as_str()]
      } else {
        vec!["--", user.as_str()]
      };
      run("/usr/bin/userdel", &args)?;
    }
    Request::CreateGroup { group } => {
      validation::validate_group_name(&group)?;
      run("/usr/bin/groupadd", &["--", &group])?;
    }
    Request::EditGroup {
      group,
      name,
      members,
    } => {
      edit_group(&group, &name, &members, group_info, run)?;
    }
    Request::DeleteGroup { group } => {
      validation::validate_group_name(&group)?;
      let (gid, _) = group_info(&group)?;
      protect_system_group(&group, gid)?;
      run("/usr/bin/groupdel", &["--", &group])?;
    }
    Request::Avatar { user, path } => {
      let manager = AccountManager::new();
      if path.is_empty() {
        manager.remove_avatar(&user)?;
      } else {
        manager.set_avatar(&user, std::path::Path::new(&path))?;
      }
    }
    Request::AutoLogin { user, enabled } => {
      let info = target(&user)?;
      if is_system_uid(info.uid) {
        return Err(invalid(
          "automatic login is only available for regular user accounts",
        ));
      }
      autologin::set_user(enabled.then_some(user.as_str()))?;
    }
  }
  Ok(json!({"ok": true}))
}

/// Creates an account with `useradd`, then sets its password when one was given.
fn create_user(
  user: &str,
  name: &str,
  shell: &str,
  groups: &[String],
  password: Option<String>,
  run: &mut impl FnMut(&str, &[&str]) -> Result<()>,
  set_password: &mut impl FnMut(&str, &str) -> Result<()>,
) -> Result<()> {
  validation::validate_username(user)?;
  let name = metadata::validate_display_name(name)?;
  validate_shell(shell)?;
  validate_groups(groups)?;
  if passwd::get_user_by_name(user)?.is_some() {
    return Err(invalid("user already exists"));
  }
  // A new account remains password-locked unless `password` was
  // supplied; validate any requestable secret before touching the
  // system so an unusable password fails fast.
  let new_password = password.filter(|value| !value.is_empty());
  if let Some(new_password) = new_password.as_deref() {
    password::validate_new_password(new_password)?;
  }
  run(
    "/usr/bin/useradd",
    &[
      "--create-home",
      "--user-group",
      "--comment",
      &name,
      "--shell",
      shell,
      "--groups",
      &groups.join(","),
      "--",
      user,
    ],
  )?;
  if let Some(new_password) = new_password {
    set_password(user, &new_password).map_err(|err| {
      Error::system(format!(
        "account '{user}' was created, but setting its password failed: {err}"
      ))
    })?;
  }
  Ok(())
}

/// Changes the name, shell and groups of an account with `usermod`.
fn edit_user(
  user: &str,
  name: &str,
  shell: &str,
  primary_group: &str,
  groups: &[String],
  run: &mut impl FnMut(&str, &[&str]) -> Result<()>,
) -> Result<()> {
  protect_system_account(user)?;
  let name = metadata::validate_display_name(name)?;
  validate_shell(shell)?;
  validate_groups(groups)?;
  validate_groups(&[primary_group])?;
  if target(user)?.uid == actor_uid()? {
    // Keep the caller from dropping its own administrator membership, which
    // would lock it out of the next elevation (see `ADMIN_GROUPS`).
    let current = groups::groups_of_user(user, target(user)?.gid)?;
    if current.iter().any(|group| is_admin_group(&group.name))
      && !is_admin_group(primary_group)
      && !groups.iter().any(|group| is_admin_group(group))
    {
      return Err(invalid("cannot remove your own administrator membership"));
    }
  }
  run(
    "/usr/bin/usermod",
    &[
      "--comment",
      &name,
      "--shell",
      shell,
      "--gid",
      primary_group,
      "--groups",
      &groups.join(","),
      "--",
      user,
    ],
  )?;
  Ok(())
}

/// Sets a new password; changing your own password requires the current one.
fn change_password(
  user: &str,
  old: &str,
  new: &str,
  confirm: &str,
  set_password: &mut impl FnMut(&str, &str) -> Result<()>,
) -> Result<()> {
  let info = target(user)?;
  if new != confirm {
    return Err(invalid("new password and confirmation do not match"));
  }
  password::validate_new_password(new)?;
  if info.uid == actor_uid()? && !password::verify_own_password(user, old)? {
    return Err(Error::PermissionDenied(
      "current password does not match".into(),
    ));
  }
  set_password(user, new)?;
  Ok(())
}

/// Renames a group and replaces its supplementary members.
fn edit_group(
  group: &str,
  name: &str,
  members: &[String],
  group_info: &dyn Fn(&str) -> Result<GroupFacts>,
  run: &mut impl FnMut(&str, &[&str]) -> Result<()>,
) -> Result<()> {
  validation::validate_group_name(group)?;
  validation::validate_group_name(name)?;
  for member in members {
    validation::validate_username(member)?;
  }
  let (gid, existing) = group_info(group)?;
  if name != group {
    protect_system_group(group, gid)?;
  }
  // Same guard as `Edit`: the caller cannot drop its own administrator
  // membership, which would lock it out of the next elevation.
  if is_admin_group(group)
    && let Some(actor) = actor_name()?
    && existing.contains(&actor)
    && !members.contains(&actor)
  {
    return Err(invalid("cannot remove your own administrator membership"));
  }
  if name != group {
    run("/usr/bin/groupmod", &["--new-name", name, "--", group])?;
  }
  for member in &existing {
    if !members.iter().any(|candidate| candidate == member) {
      run("/usr/bin/gpasswd", &["--delete", member, "--", name])?;
    }
  }
  for member in members {
    if !existing.iter().any(|candidate| candidate == member) {
      run(
        "/usr/bin/usermod",
        &["--append", "--groups", name, "--", member],
      )?;
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests;
