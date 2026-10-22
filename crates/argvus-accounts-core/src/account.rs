//! High-level account management facade.
//!
//! [`AccountManager`] composes NSS lookups, the avatar store, display-name
//! handling and authorization into one reusable API suitable for both the CLI
//! and future Argvus frontends (GUI, greeter helpers, ...).

use std::path::{Path, PathBuf};

use crate::avatar;
use crate::error::{Error, Result};
use crate::groups::{self, GpasswdAction};
use crate::metadata;
use crate::passwd::{self, UserInfo};
use crate::password;
use crate::permissions::{
  Action, ActorIdentity, AuthContext, AuthorizationProvider, UnixAuthorizationProvider,
};
use crate::validation;

/// System operations that require privileges, abstracted for testability.
pub trait PrivilegeOps: Send + Sync {
  /// Sets the GECOS comment (display name) using administrative tools.
  fn set_display_name_admin(&self, user: &str, name: &str) -> Result<()>;
  /// Sets the caller's own full name via `chfn` (may prompt via PAM).
  fn set_own_display_name(&self, user: &str, name: &str) -> Result<()>;
  /// Adds `user` to `group`.
  fn add_to_group(&self, user: &str, group: &str) -> Result<()>;
  /// Removes `user` from `group`.
  fn remove_from_group(&self, user: &str, group: &str) -> Result<()>;
  /// Sets `user`'s password (administrative backend; the secret is
  /// transferred through stdin, never argv).
  fn set_password(&self, user: &str, new: &str) -> Result<()>;
}

/// Default backend that shells out to shadow/util-linux tools
/// (`usermod`, `chfn`, `gpasswd`) using argument vectors only — never a shell.
pub struct ShadowOps;

impl PrivilegeOps for ShadowOps {
  fn set_display_name_admin(&self, user: &str, name: &str) -> Result<()> {
    let mut cmd = metadata::usermod_set_comment(user, name);
    metadata::run_command(&mut cmd, "failed to update account comment")
  }

  fn set_own_display_name(&self, user: &str, name: &str) -> Result<()> {
    let mut cmd = metadata::chfn_set_full_name(user, name);
    metadata::run_command(&mut cmd, "failed to update own full name")
  }

  fn add_to_group(&self, user: &str, group: &str) -> Result<()> {
    let mut cmd = groups::gpasswd_command(GpasswdAction::Add, user, group);
    metadata::run_command(&mut cmd, "failed to add group member")
  }

  fn remove_from_group(&self, user: &str, group: &str) -> Result<()> {
    let mut cmd = groups::gpasswd_command(GpasswdAction::Remove, user, group);
    metadata::run_command(&mut cmd, "failed to remove group member")
  }

  fn set_password(&self, user: &str, new: &str) -> Result<()> {
    password::set_password_admin(user, new)
  }
}

/// Aggregated view of an account, safe to present to end users.
#[derive(Clone, Debug)]
pub struct UserView {
  /// Core NSS data.
  pub info: UserInfo,
  /// Display name from the GECOS full-name field, if set.
  pub display_name: Option<String>,
  /// Group names (primary + supplementary), sorted case-insensitively.
  pub groups: Vec<String>,
  /// Avatar path when one is deployed.
  pub avatar: Option<PathBuf>,
}

enum DisplayNameStrategy {
  SystemWide,
  OwnAccount,
}

fn display_name_strategy(privileged: bool, _is_self: bool) -> DisplayNameStrategy {
  if privileged {
    DisplayNameStrategy::SystemWide
  } else {
    DisplayNameStrategy::OwnAccount
  }
}

/// Facade over local account administration.
pub struct AccountManager {
  authz: Box<dyn AuthorizationProvider>,
  ops: Box<dyn PrivilegeOps>,
  strict_checks: bool,
  home_override: Option<PathBuf>,
}

impl Default for AccountManager {
  fn default() -> Self {
    Self::new()
  }
}

impl AccountManager {
  /// Creates a manager with the default Unix authorization provider and
  /// shadow-tools backend.
  pub fn new() -> Self {
    Self {
      authz: Box::new(UnixAuthorizationProvider),
      ops: Box::new(ShadowOps),
      strict_checks: true,
      home_override: None,
    }
  }

  /// Creates a manager with injected components. Useful for alternative
  /// backends, GUI embedding and hermetic tests.
  pub fn with_components(
    authz: Box<dyn AuthorizationProvider>,
    ops: Box<dyn PrivilegeOps>,
  ) -> Self {
    Self {
      authz,
      ops,
      strict_checks: false,
      home_override: None,
    }
  }

  /// Overrides the home directory used for avatar deployment. Intended for
  /// tests and sandboxed environments; production code should leave it unset.
  #[must_use]
  pub fn with_home_override(mut self, home: PathBuf) -> Self {
    self.home_override = Some(home);
    self
  }

  /// Name of the active authorization provider.
  #[must_use]
  pub fn provider_name(&self) -> &'static str {
    self.authz.describe()
  }

  /// Username of the current effective identity.
  pub fn current_username() -> Result<String> {
    Ok(passwd::current_user()?.username)
  }

  /// Lists accounts. Human accounts (uid >= 1000) unless `include_system`.
  pub fn list_users(&self, include_system: bool) -> Result<Vec<UserInfo>> {
    let users = passwd::list_users()?;
    if include_system {
      Ok(users)
    } else {
      Ok(users.into_iter().filter(|u| u.uid >= 1000).collect())
    }
  }

  /// Full view of a single account.
  pub fn get_user(&self, username: &str) -> Result<UserView> {
    let info = self.require_target(username)?;
    self.authorize(Action::ReadAccount, &info.username)?;
    let avatar = self
      .home_of(&info)?
      .and_then(|home| avatar::find_avatar(&home));
    Ok(UserView {
      display_name: info.display_name().map(str::to_owned),
      groups: self.member_names(&info)?,
      avatar,
      info,
    })
  }

  /// Display name of `username`, if set.
  pub fn get_display_name(&self, username: &str) -> Result<Option<String>> {
    let info = self.require_target(username)?;
    self.authorize(Action::ReadAccount, &info.username)?;
    Ok(info.display_name().map(str::to_owned))
  }

  /// Sets the display name without ever touching the Unix username.
  pub fn set_display_name(&self, username: &str, name: &str) -> Result<()> {
    let name = metadata::validate_display_name(name)?;
    let info = self.require_target(username)?;
    let actor = self.actor()?;
    let action = if actor.username == info.username {
      Action::ModifyOwnAccount
    } else {
      Action::ModifyOtherAccount
    };
    self.authorize(action, &info.username)?;

    match display_name_strategy(actor.is_privileged, actor.username == info.username) {
      DisplayNameStrategy::SystemWide => self.ops.set_display_name_admin(&info.username, &name),
      DisplayNameStrategy::OwnAccount => self.ops.set_own_display_name(&info.username, &name),
    }
  }

  /// Path of the deployed avatar, when present.
  pub fn get_avatar(&self, username: &str) -> Result<Option<PathBuf>> {
    let info = self.require_target(username)?;
    self.authorize(Action::ReadAccount, &info.username)?;
    Ok(
      self
        .home_of(&info)?
        .and_then(|home| avatar::find_avatar(&home)),
    )
  }

  /// Validates and atomically deploys `source` as `username`'s avatar.
  ///
  /// The source image is copied/normalized; it is never moved or modified.
  pub fn set_avatar(&self, username: &str, source: &Path) -> Result<PathBuf> {
    let info = self.require_target(username)?;
    let actor = self.actor()?;
    let action = if actor.username == info.username {
      Action::ModifyOwnAccount
    } else {
      Action::ModifyOtherAccount
    };
    self.authorize(action, &info.username)?;
    let home = self
      .home_of(&info)?
      .ok_or_else(|| Error::HomeDirUnusable(info.username.clone()))?;
    avatar::set_avatar(&home, source, (info.uid, info.gid))
  }

  /// Atomically removes `username`'s avatar.
  pub fn remove_avatar(&self, username: &str) -> Result<()> {
    let info = self.require_target(username)?;
    let actor = self.actor()?;
    let action = if actor.username == info.username {
      Action::ModifyOwnAccount
    } else {
      Action::ModifyOtherAccount
    };
    self.authorize(action, &info.username)?;
    let home = self
      .home_of(&info)?
      .ok_or_else(|| Error::HomeDirUnusable(info.username.clone()))?;
    avatar::remove_avatar(&home).map_err(|err| match err {
      Error::AvatarNotFound(_) => Error::AvatarNotFound(info.username.clone()),
      other => other,
    })
  }

  /// Sorted group names of `username`.
  pub fn list_groups(&self, username: &str) -> Result<Vec<String>> {
    let info = self.require_target(username)?;
    self.authorize(Action::ReadAccount, &info.username)?;
    self.member_names(&info)
  }

  /// Adds `username` to `group` (administrative operation).
  pub fn add_group(&self, username: &str, group: &str) -> Result<()> {
    self.mutate_membership(username, group, MembershipChange::Add)
  }

  /// Removes `username` from `group` (administrative operation).
  pub fn remove_group(&self, username: &str, group: &str) -> Result<()> {
    self.mutate_membership(username, group, MembershipChange::Remove)
  }

  /// Changes `username`'s password.
  ///
  /// Semantics:
  ///
  /// - **own account**: `proof` (the current password) is mandatory and is
  ///   verified against the system before anything changes. This holds even
  ///   when the caller is root, so the elevated re-execution performed by
  ///   the CLI keeps the same guarantee;
  /// - **another user's account**: administrative reset; `proof` is ignored
  ///   and may be `None`. Authorization still applies.
  ///
  /// The final write goes through [`PrivilegeOps::set_password`] and
  /// therefore requires effective root; the CLI elevates via polkit before
  /// reaching this point.
  pub fn change_password(&self, username: &str, proof: Option<&str>, new: &str) -> Result<()> {
    password::validate_new_password(new)?;
    let info = self.require_target(username)?;
    let actor = self.actor()?;
    let is_self = actor.username == info.username;
    self.authorize(Action::ChangePassword, &info.username)?;

    if is_self {
      let Some(current) = proof else {
        return Err(Error::InvalidOperation(
          "refusing to change your own password without the current password; \
                     pass it as OLD_PASSWORD"
            .to_string(),
        ));
      };
      if !password::verify_own_password(&info.username, current)? {
        return Err(Error::PermissionDenied(
          "current password does not match".to_string(),
        ));
      }
    }

    // The final write requires effective root; when invoked directly by an
    // unprivileged embedder, chpasswd itself fails with a system error.
    // The CLI elevates via polkit before reaching this point.
    self.ops.set_password(&info.username, new)
  }

  // -- internals ----------------------------------------------------------

  fn mutate_membership(&self, username: &str, group: &str, change: MembershipChange) -> Result<()> {
    validation::validate_group_name(group)?;
    let info = self.require_target(username)?;
    self.authorize(Action::AdministerGroups, &info.username)?;

    if self.strict_checks && groups::get_group_by_name(group)?.is_none() {
      return Err(Error::GroupNotFound(group.to_string()));
    }

    match change {
      MembershipChange::Add => {
        if self.strict_checks && self.is_member(&info, group)? {
          return Err(Error::AlreadyMember {
            user: info.username.clone(),
            group: group.to_string(),
          });
        }
        self.ops.add_to_group(&info.username, group)?;
        if self.strict_checks && !self.is_member(&info, group)? {
          return Err(Error::system(format!(
            "'{}' still not listed in '{group}' after gpasswd",
            info.username
          )));
        }
      }
      MembershipChange::Remove => {
        if self.strict_checks && !self.is_member(&info, group)? {
          return Err(Error::NotMember {
            user: info.username.clone(),
            group: group.to_string(),
          });
        }
        self.ops.remove_from_group(&info.username, group)?;
        if self.strict_checks && self.is_member(&info, group)? {
          return Err(Error::system(format!(
            "'{}' still listed in '{group}' after gpasswd",
            info.username
          )));
        }
      }
    }
    Ok(())
  }

  fn is_member(&self, info: &UserInfo, group: &str) -> Result<bool> {
    let names = self.member_names(info)?;
    Ok(names.iter().any(|name| name == group))
  }

  fn member_names(&self, info: &UserInfo) -> Result<Vec<String>> {
    let groups = groups::groups_of_user(&info.username, info.gid)?;
    let mut names: Vec<String> = groups.into_iter().map(|g| g.name).collect();
    names.sort_by_key(|name| name.to_lowercase());
    Ok(names)
  }

  fn require_target(&self, username: &str) -> Result<UserInfo> {
    validation::validate_username(username)?;
    passwd::get_user_by_name(username)?.ok_or_else(|| Error::UserNotFound(username.to_string()))
  }

  fn actor(&self) -> Result<ActorIdentity> {
    let user = passwd::current_user()?;
    Ok(ActorIdentity {
      username: user.username,
      uid: user.uid,
      is_privileged: nix::unistd::Uid::effective().is_root(),
    })
  }

  fn authorize(&self, action: Action, target: &str) -> Result<()> {
    let actor = self.actor()?;
    self.authz.authorize(
      action,
      &AuthContext {
        actor: &actor,
        target,
      },
    )
  }

  fn home_of(&self, info: &UserInfo) -> Result<Option<PathBuf>> {
    if let Some(home) = &self.home_override {
      return Ok(Some(home.clone()));
    }
    if info.home.as_os_str().is_empty() {
      return Ok(None);
    }
    Ok(Some(info.home.clone()))
  }
}

enum MembershipChange {
  Add,
  Remove,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn root_uses_systemwide_tool_and_others_use_chfn() {
    assert!(matches!(
      display_name_strategy(true, true),
      DisplayNameStrategy::SystemWide
    ));
    assert!(matches!(
      display_name_strategy(false, true),
      DisplayNameStrategy::OwnAccount
    ));
  }
}
