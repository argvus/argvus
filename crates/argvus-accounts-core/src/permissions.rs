//! Authorization abstraction for account operations.
//!
//! The library never decides privileges inline; every mutating operation goes
//! through an [`AuthorizationProvider`]. The shipped implementation,
//! [`UnixAuthorizationProvider`], follows classic Unix semantics: effective
//! root is an administrator, and unprivileged callers may only act on their
//! own account for profile-level actions. A future `PolkitAuthorizationProvider`
//! can slot in without touching any call site, e.g. mapping:
//!
//! - `ModifyOwnAccount`    -> `com.argvus.accounts.change-own-data` (`allow_active=yes`)
//! - `ModifyOtherAccount`  -> `com.argvus.accounts.administer`     (`auth_admin`)
//! - `AdministerGroups`    -> `com.argvus.accounts.administer`     (`auth_admin`)

use crate::error::{Error, Result};

/// An operation that may require authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
  /// Read basic (public) account information.
  ReadAccount,
  /// Change profile data of the caller's own account.
  ModifyOwnAccount,
  /// Change profile data of another user's account.
  ModifyOtherAccount,
  /// Change group membership of any account.
  AdministerGroups,
  /// Change an account password.
  ///
  /// Self-service changes additionally require proof of the current
  /// password (enforced by the manager, not by the provider).
  ChangePassword,
}

/// Identity of the acting user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorIdentity {
  /// Username of the actor.
  pub username: String,
  /// Effective uid of the actor.
  pub uid: u32,
  /// Whether the actor holds administrator privileges.
  pub is_privileged: bool,
}

/// Authorization request context binding the actor to the target account.
#[derive(Clone, Copy, Debug)]
pub struct AuthContext<'a> {
  /// Who performs the action.
  pub actor: &'a ActorIdentity,
  /// Whose account is targeted.
  pub target: &'a str,
}

impl AuthContext<'_> {
  /// Whether the action targets the actor's own account.
  #[must_use]
  pub fn is_self(&self) -> bool {
    self.actor.username == self.target
  }
}

/// Pluggable policy decision point.
///
/// Implementations decide whether `actor` may perform `action` on `target`.
pub trait AuthorizationProvider: Send + Sync {
  /// Authorizes the requested action or fails with a descriptive error.
  fn authorize(&self, action: Action, ctx: &AuthContext<'_>) -> Result<()>;

  /// Short human-readable name of the provider (for diagnostics).
  fn describe(&self) -> &'static str;
}

/// Default provider implementing standard Unix semantics.
///
/// - root may do everything;
/// - everyone may read public account information;
/// - unprivileged users may modify only their own profile;
/// - everything else is denied with actionable guidance.
pub struct UnixAuthorizationProvider;

impl AuthorizationProvider for UnixAuthorizationProvider {
  fn authorize(&self, action: Action, ctx: &AuthContext<'_>) -> Result<()> {
    if ctx.actor.is_privileged || matches!(action, Action::ReadAccount) {
      return Ok(());
    }
    match action {
      Action::ReadAccount => Ok(()),
      Action::ModifyOwnAccount if ctx.is_self() => Ok(()),
      Action::ModifyOwnAccount => Err(deny(
        "you may only modify your own account; use 'self' subcommands",
      )),
      Action::ModifyOtherAccount => Err(deny(
        "modifying another user's account requires administrator privileges (re-run with sudo)",
      )),
      Action::AdministerGroups => Err(deny(
        "group administration requires administrator privileges (re-run with sudo)",
      )),
      // Self-service password changes are allowed here; the manager
      // additionally requires proof of the current password.
      Action::ChangePassword if ctx.is_self() => Ok(()),
      Action::ChangePassword => Err(deny(
        "changing another user's password requires administrator privileges \
                 (re-run with sudo)",
      )),
    }
  }

  fn describe(&self) -> &'static str {
    "unix"
  }
}

fn deny(reason: &str) -> Error {
  Error::PermissionDenied(reason.to_string())
}

/// Provider that allows everything. Intended for tests and explicitly
/// sandboxed embeddings; never used by default.
pub struct AllowAllProvider;

impl AuthorizationProvider for AllowAllProvider {
  fn authorize(&self, _action: Action, _ctx: &AuthContext<'_>) -> Result<()> {
    Ok(())
  }

  fn describe(&self) -> &'static str {
    "allow-all"
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn actor(name: &str, privileged: bool) -> ActorIdentity {
    ActorIdentity {
      username: name.to_string(),
      uid: if privileged { 0 } else { 1000 },
      is_privileged: privileged,
    }
  }

  fn ctx<'a>(actor: &'a ActorIdentity, target: &'a str) -> AuthContext<'a> {
    AuthContext { actor, target }
  }

  #[test]
  fn unix_provider_allows_reads_for_everyone() {
    let me = actor("ghost", false);
    let provider = UnixAuthorizationProvider;
    assert!(
      provider
        .authorize(Action::ReadAccount, &ctx(&me, "william"))
        .is_ok()
    );
  }

  #[test]
  fn unix_provider_allows_own_profile_changes_only() {
    let me = actor("ghost", false);
    let provider = UnixAuthorizationProvider;
    assert!(
      provider
        .authorize(Action::ModifyOwnAccount, &ctx(&me, "ghost"))
        .is_ok()
    );

    let err = provider
      .authorize(Action::ModifyOwnAccount, &ctx(&me, "william"))
      .unwrap_err();
    assert!(err.to_string().contains("only modify your own"));
  }

  #[test]
  fn unix_provider_denies_cross_user_and_group_administration() {
    let me = actor("ghost", false);
    let provider = UnixAuthorizationProvider;

    let err = provider
      .authorize(Action::ModifyOtherAccount, &ctx(&me, "william"))
      .unwrap_err();
    assert!(matches!(err, Error::PermissionDenied(_)));

    let err = provider
      .authorize(Action::AdministerGroups, &ctx(&me, "ghost"))
      .unwrap_err();
    assert!(matches!(err, Error::PermissionDenied(_)));
  }

  #[test]
  fn unix_provider_root_is_administrator() {
    let root = actor("root", true);
    let provider = UnixAuthorizationProvider;
    for action in [
      Action::ReadAccount,
      Action::ModifyOwnAccount,
      Action::ModifyOtherAccount,
      Action::AdministerGroups,
    ] {
      assert!(provider.authorize(action, &ctx(&root, "ghost")).is_ok());
    }
  }

  #[test]
  fn allow_all_provider_grants_everything() {
    let me = actor("ghost", false);
    assert!(
      AllowAllProvider
        .authorize(Action::AdministerGroups, &ctx(&me, "root"))
        .is_ok()
    );
  }
}
