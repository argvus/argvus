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

fn validate_groups(names: &[String]) -> Result<()> {
  for name in names {
    validation::validate_group_name(name)?;
    if groups::get_group_by_name(name)?.is_none() {
      return Err(Error::GroupNotFound(name.clone()));
    }
  }
  Ok(())
}

fn target(user: &str) -> Result<passwd::UserInfo> {
  validation::validate_username(user)?;
  passwd::get_user_by_name(user)?.ok_or_else(|| Error::UserNotFound(user.into()))
}

/// Refuses system accounts (uid below 1000 and `nobody`). Used for edits, where
/// the administrator may still change its own profile.
fn protect_system_account(user: &str) -> Result<()> {
  let info = target(user)?;
  if info.uid < 1000 || info.uid == 65534 {
    return Err(invalid("cannot modify a system account"));
  }
  Ok(())
}

fn protect_account(user: &str) -> Result<()> {
  let info = target(user)?;
  if info.uid < 1000 || info.uid == 65534 || info.uid == actor_uid()? {
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
    return Ok(json!({
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
    }));
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
    Request::Snapshot => unreachable!(),
    Request::Create {
      user,
      name,
      shell,
      groups,
      password,
    } => {
      validation::validate_username(&user)?;
      let name = metadata::validate_display_name(&name)?;
      validate_shell(&shell)?;
      validate_groups(&groups)?;
      if passwd::get_user_by_name(&user)?.is_some() {
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
          &shell,
          "--groups",
          &groups.join(","),
          "--",
          &user,
        ],
      )?;
      if let Some(new_password) = new_password {
        set_password(&user, &new_password).map_err(|err| {
          Error::system(format!(
            "account '{user}' was created, but setting its password failed: {err}"
          ))
        })?;
      }
    }
    Request::Edit {
      user,
      name,
      shell,
      primary_group,
      groups,
    } => {
      protect_system_account(&user)?;
      let name = metadata::validate_display_name(&name)?;
      validate_shell(&shell)?;
      validate_groups(&groups)?;
      validate_groups(std::slice::from_ref(&primary_group))?;
      if target(&user)?.uid == actor_uid()? {
        // Keep the caller from dropping its own administrator membership, which
        // would lock it out of the next elevation (see `ADMIN_GROUPS`).
        let current = groups::groups_of_user(&user, target(&user)?.gid)?;
        if current.iter().any(|group| is_admin_group(&group.name))
          && !is_admin_group(&primary_group)
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
          &shell,
          "--gid",
          &primary_group,
          "--groups",
          &groups.join(","),
          "--",
          &user,
        ],
      )?;
    }
    Request::Password {
      user,
      old,
      new,
      confirm,
    } => {
      let info = target(&user)?;
      if new != confirm {
        return Err(invalid("new password and confirmation do not match"));
      }
      password::validate_new_password(&new)?;
      if info.uid == actor_uid()? && !password::verify_own_password(&user, &old)? {
        return Err(Error::PermissionDenied(
          "current password does not match".into(),
        ));
      }
      set_password(&user, &new)?;
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
      validation::validate_group_name(&group)?;
      validation::validate_group_name(&name)?;
      for member in &members {
        validation::validate_username(member)?;
      }
      let (gid, existing) = group_info(&group)?;
      if name != group {
        protect_system_group(&group, gid)?;
      }
      // Same guard as `Edit`: the caller cannot drop its own administrator
      // membership, which would lock it out of the next elevation.
      if is_admin_group(&group)
        && let Some(actor) = actor_name()?
        && existing.contains(&actor)
        && !members.contains(&actor)
      {
        return Err(invalid("cannot remove your own administrator membership"));
      }
      if name != group {
        run("/usr/bin/groupmod", &["--new-name", &name, "--", &group])?;
      }
      for member in &existing {
        if !members.iter().any(|candidate| candidate == member) {
          run("/usr/bin/gpasswd", &["--delete", member, "--", &name])?;
        }
      }
      for member in &members {
        if !existing.iter().any(|candidate| candidate == member) {
          run(
            "/usr/bin/usermod",
            &["--append", "--groups", &name, "--", member],
          )?;
        }
      }
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
      if info.uid < 1000 || info.uid == 65534 {
        return Err(invalid(
          "automatic login is only available for regular user accounts",
        ));
      }
      autologin::set_user(enabled.then_some(user.as_str()))?;
    }
  }
  Ok(json!({"ok": true}))
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn protocol_rejects_unknown_operations_and_fields() {
    assert!(serde_json::from_str::<Request>(r#"{"action":"shell","command":"id"}"#).is_err());
    assert!(
      serde_json::from_str::<Request>(r#"{"action":"delete","user":"root","force":true}"#).is_err()
    );
  }
  #[test]
  fn wheel_and_sudo_are_the_admin_groups() {
    assert!(is_admin_group("wheel"));
    assert!(is_admin_group("sudo"));
    assert!(!is_admin_group("users"));
    assert!(!is_admin_group("Wheel"));
  }

  #[test]
  fn protects_root_and_system_users() {
    assert!(protect_account("root").is_err());
    assert!(protect_account("../etc/passwd").is_err());
  }

  #[test]
  fn edits_cannot_target_system_accounts() {
    assert!(protect_system_account("root").is_err());
    assert!(protect_system_account("../etc/passwd").is_err());
  }

  #[test]
  fn create_uses_shadow_argument_vector_and_leaves_password_locked() {
    let mut calls = Vec::new();
    let request = Request::Create {
      user: "argvus_test_created".into(),
      name: "Alice $(id)".into(),
      shell: shells().unwrap()[0].clone(),
      groups: Vec::new(),
      password: None,
    };
    let mut set_password_calls = Vec::new();
    apply_request(
      request,
      &|_| Ok((1000, Vec::new())),
      &mut |program, args| {
        calls.push((
          program.to_string(),
          args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
        ));
        Ok(())
      },
      &mut |user, new| {
        set_password_calls.push((user.to_string(), new.to_string()));
        Ok(())
      },
    )
    .unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "/usr/bin/useradd");
    assert!(calls[0].1.contains(&"--create-home".into()));
    assert!(calls[0].1.contains(&"Alice $(id)".into()));
    assert!(!calls[0].1.contains(&"--password".into()));
    assert_eq!(calls[0].1.last().unwrap(), "argvus_test_created");
    assert!(set_password_calls.is_empty());
  }

  #[test]
  fn create_with_password_sets_it_without_passing_secret_via_argv() {
    let secret = "correct horse battery staple";
    let mut calls = Vec::new();
    let mut set_password_calls = Vec::new();
    let request = Request::Create {
      user: "argvus_test_created".into(),
      name: "Alice".into(),
      shell: shells().unwrap()[0].clone(),
      groups: Vec::new(),
      password: Some(secret.into()),
    };
    apply_request(
      request,
      &|_| Ok((1000, Vec::new())),
      &mut |program, args| {
        calls.push((
          program.to_string(),
          args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
        ));
        Ok(())
      },
      &mut |user, new| {
        set_password_calls.push((user.to_string(), new.to_string()));
        Ok(())
      },
    )
    .unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "/usr/bin/useradd");
    assert!(!calls[0].1.contains(&"--password".into()));
    assert_eq!(calls[0].1.last().unwrap(), "argvus_test_created");
    for (_program, args) in &calls {
      for arg in args {
        assert!(
          !arg.contains(secret),
          "secret must never reach argv: {arg:?}"
        );
      }
    }
    assert_eq!(set_password_calls.len(), 1);
    assert_eq!(set_password_calls[0].0, "argvus_test_created");
    assert_eq!(set_password_calls[0].1, secret);
  }

  #[test]
  fn create_rejects_invalid_password_before_running_any_system_command() {
    let request = Request::Create {
      user: "argvus_test_created".into(),
      name: "Alice".into(),
      shell: shells().unwrap()[0].clone(),
      groups: Vec::new(),
      password: Some("line\nbreak".into()),
    };
    assert!(
      apply_request(
        request,
        &|_| Ok((1000, Vec::new())),
        &mut |_, _| panic!("must not run a system command for an invalid password"),
        &mut |_, _| panic!("must not set a password for an invalid password"),
      )
      .is_err()
    );
  }

  #[test]
  fn invalid_creation_never_reaches_shadow_tools() {
    let request = Request::Create {
      user: "--root".into(),
      name: "Invalid".into(),
      shell: "/bin/sh".into(),
      groups: Vec::new(),
      password: None,
    };
    assert!(
      apply_request(
        request,
        &|_| Ok((1000, Vec::new())),
        &mut |_, _| panic!("must not run a system command"),
        &mut |_, _| panic!("must not set a password"),
      )
      .is_err()
    );
  }

  #[test]
  fn cannot_delete_root_even_after_authorization() {
    assert!(
      apply_request(
        Request::Delete {
          user: "root".into(),
          remove_home: false,
        },
        &|_| Ok((1000, Vec::new())),
        &mut |_, _| panic!("must not delete root"),
        &mut |_, _| panic!("must not set a password"),
      )
      .is_err()
    );
  }

  #[test]
  fn edit_group_renames_and_syncs_membership() {
    let mut calls = Vec::new();
    let request = Request::EditGroup {
      group: "staff".into(),
      name: "team".into(),
      members: vec!["alice".into(), "bob".into()],
    };
    let current = |_group: &str| Ok((1001, vec!["carol".into()]));
    apply_request(
      request,
      &current,
      &mut |program, args| {
        calls.push((
          program.to_string(),
          args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
        ));
        Ok(())
      },
      &mut |_, _| panic!("EditGroup must not set a password"),
    )
    .unwrap();
    assert!(calls.contains(&(
      "/usr/bin/groupmod".into(),
      vec![
        "--new-name".into(),
        "team".into(),
        "--".into(),
        "staff".into()
      ]
    )));
    assert!(calls.contains(&(
      "/usr/bin/gpasswd".into(),
      vec![
        "--delete".into(),
        "carol".into(),
        "--".into(),
        "team".into()
      ]
    )));
    assert!(calls.iter().any(|call| call.0 == "/usr/bin/usermod"));
    assert!(calls.iter().any(|call| call.1.contains(&"--append".into())));
  }

  #[test]
  fn delete_group_runs_groupdel() {
    let mut calls = Vec::new();
    let current = |_group: &str| Ok((1001, vec!["alice".into()]));
    apply_request(
      Request::DeleteGroup {
        group: "legacy".into(),
      },
      &current,
      &mut |program, _| {
        calls.push(program.to_string());
        Ok(())
      },
      &mut |_, _| panic!("DeleteGroup must not set a password"),
    )
    .unwrap();
    assert!(calls.contains(&"/usr/bin/groupdel".into()));
  }

  #[test]
  fn system_and_administrator_groups_cannot_be_renamed_or_deleted() {
    for (group, gid) in [("video", 985), ("wheel", 998), ("sudo", 1005)] {
      assert!(
        apply_request(
          Request::DeleteGroup {
            group: group.into(),
          },
          &|_| Ok((gid, Vec::new())),
          &mut |_, _| panic!("must not delete {group}"),
          &mut |_, _| panic!("must not set a password"),
        )
        .is_err(),
        "{group}"
      );
      assert!(
        apply_request(
          Request::EditGroup {
            group: group.into(),
            name: "renamed".into(),
            members: Vec::new(),
          },
          &|_| Ok((gid, Vec::new())),
          &mut |_, _| panic!("must not rename {group}"),
          &mut |_, _| panic!("must not set a password"),
        )
        .is_err(),
        "{group}"
      );
    }
  }

  #[test]
  fn system_group_membership_can_still_change() {
    let mut calls = Vec::new();
    apply_request(
      Request::EditGroup {
        group: "video".into(),
        name: "video".into(),
        members: vec!["alice".into()],
      },
      &|_| Ok((985, Vec::new())),
      &mut |program, _| {
        calls.push(program.to_string());
        Ok(())
      },
      &mut |_, _| panic!("EditGroup must not set a password"),
    )
    .unwrap();
    assert_eq!(calls, ["/usr/bin/usermod"]);
  }

  #[test]
  fn administrator_cannot_leave_an_admin_group_through_a_group_edit() {
    let me = passwd::current_user().unwrap().username;
    let existing = vec![me.clone()];
    assert!(
      apply_request(
        Request::EditGroup {
          group: "wheel".into(),
          name: "wheel".into(),
          members: Vec::new(),
        },
        &|_| Ok((998, existing.clone())),
        &mut |_, _| panic!("must not drop {me} from wheel"),
        &mut |_, _| panic!("must not set a password"),
      )
      .is_err()
    );
  }

  #[test]
  fn group_operations_reject_missing_or_invalid_groups() {
    assert!(
      apply_request(
        Request::DeleteGroup {
          group: "../etc/passwd".into()
        },
        &|_| Ok((1000, Vec::new())),
        &mut |_, _| panic!("must not delete an invalid group"),
        &mut |_, _| panic!("must not set a password"),
      )
      .is_err()
    );
  }

  fn user_with_home(home: &std::path::Path) -> passwd::UserInfo {
    passwd::UserInfo {
      username: "alice".into(),
      uid: 1000,
      gid: 1000,
      gecos: String::new(),
      home: home.to_path_buf(),
      shell: "/bin/bash".into(),
    }
  }

  #[test]
  fn snapshot_avatar_is_the_deployed_face_path_or_empty() {
    let home = tempfile::tempdir().expect("temporary home");
    let info = user_with_home(home.path());
    assert_eq!(avatar_of(&info), "");

    let face = home.path().join(avatar::AVATAR_FILENAME);
    std::fs::write(&face, b"png").expect("avatar file");
    assert_eq!(avatar_of(&info), face.display().to_string());
  }
}
