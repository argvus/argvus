//! Unsaved edits of users, groups and the firewall: what changed, discarding them and when leaving a page drops them.

use super::*;

impl Administration {
  /// The loaded account the user page edits.
  pub(super) fn loaded_user(&self) -> Option<&Value> {
    self.accounts["users"]
      .as_array()?
      .iter()
      .find(|user| user["user"] == self.user["user"])
  }

  /// Whether the user page has unsaved changes.
  pub(crate) fn user_draft_changed(&self) -> bool {
    let Some(loaded) = self.loaded_user() else {
      return false;
    };
    let set = |value: &Value| strings(value).into_iter().collect::<BTreeSet<_>>();
    ["name", "shell", "primary_group"]
      .iter()
      .any(|field| text(loaded, field) != text(&self.user, field))
      || set(&loaded["groups"]) != set(&self.user["groups"])
  }

  /// Whether the signed-in account administers accounts (root or `sudo`), as
  /// the backend reported it. Administrators only see and change other accounts.
  /// The group a new administrator joins: `wheel` when the system has it (as
  /// stock Arch does), otherwise `sudo`.
  pub(crate) fn admin_group(&self) -> &'static str {
    if strings(&self.accounts["groups"])
      .iter()
      .any(|group| group == "wheel")
    {
      "wheel"
    } else {
      "sudo"
    }
  }

  pub(crate) fn is_admin(&self) -> bool {
    self.accounts["actor_is_admin"].as_bool().unwrap_or(false)
  }

  /// Whether `user` is the account greetd starts automatically at boot,
  /// skipping the login screen (its `initial_session`, reported by the backend).
  pub(crate) fn is_autologin_user(&self, user: &str) -> bool {
    !user.is_empty() && text(&self.accounts, "autologin_user") == user
  }

  /// Whether the user page has any draft to save: fields, avatar or a typed password.
  pub(crate) fn any_user_draft_changed(&self) -> bool {
    self.user_draft_changed()
      || self.avatar_draft_changed()
      || self.passwords_changed()
      || self.security_changed()
  }

  /// Whether the lock or the forced password change has a pending draft.
  pub(crate) fn security_changed(&self) -> bool {
    self.lock_draft.is_some() || self.expire_draft
  }

  /// Whether the avatar draft differs from the saved avatar ("" means none).
  pub(crate) fn avatar_draft_changed(&self) -> bool {
    let Some(loaded) = self.loaded_user() else {
      return false;
    };
    text(loaded, "avatar") != text(&self.user, "avatar")
  }

  /// Whether the create-user form has anything typed or chosen.
  pub(crate) fn create_user_changed(&self) -> bool {
    let default_shell = strings(&self.accounts["shells"])
      .first()
      .cloned()
      .unwrap_or_else(|| "/bin/bash".into());
    !text(&self.user, "user").is_empty()
      || !text(&self.user, "name").is_empty()
      || text(&self.user, "shell") != default_shell
      || !strings(&self.user["groups"]).is_empty()
      || self.create_admin
      || !text(&self.user, "avatar").is_empty()
      || self.passwords_changed()
  }

  /// Whether any password field has been typed.
  pub(crate) fn passwords_changed(&self) -> bool {
    self.passwords.iter().any(|password| !password.is_empty())
  }

  /// The loaded group the group page edits.
  pub(super) fn loaded_group(&self) -> Option<&Value> {
    self.accounts["group_details"]
      .as_array()?
      .iter()
      .find(|group| group["name"] == self.group["original"])
  }

  /// Whether the group page has unsaved changes.
  pub(crate) fn group_draft_changed(&self) -> bool {
    let Some(loaded) = self.loaded_group() else {
      return false;
    };
    let set = |value: &Value| strings(value).into_iter().collect::<BTreeSet<_>>();
    text(loaded, "name") != text(&self.group, "name")
      || set(&loaded["members"]) != set(&self.group["members"])
  }

  /// Whether the create-group form has a name.
  pub(crate) fn create_group_changed(&self) -> bool {
    !text(&self.group, "name").is_empty()
  }

  /// Whether the firewall configuration differs from the saved one.
  pub(crate) fn firewall_draft_changed(&self) -> bool {
    !self.firewall.is_null() && self.config != self.firewall["config"]
  }

  /// Drops the unsaved changes of `page`, back to the loaded state.
  pub(crate) fn discard_draft(&mut self, page: Page) {
    match page {
      Page::User => {
        if let Some(loaded) = self.loaded_user().cloned() {
          self.user = loaded;
        }
      }
      Page::Group => {
        if let Some(loaded) = self.loaded_group().cloned() {
          let original = self.group["original"].clone();
          self.group = loaded;
          self.group["original"] = original;
        }
      }
      Page::Firewall => self.config = self.firewall["config"].clone(),
      Page::CreateGroup => self.group = json!({"name":""}),
      Page::CreateUser => {
        // A discarded form starts again empty, with the default shell.
        self.create_admin = false;
        self.creating_password = false;
        let shell = strings(&self.accounts["shells"])
          .first()
          .cloned()
          .unwrap_or_else(|| "/bin/bash".into());
        self.user = json!({"user":"", "name":"", "shell":shell, "groups":[]});
      }
      _ => {}
    }
    self.passwords = Default::default();
    self.queued.clear();
    self.lock_draft = None;
    self.expire_draft = false;
  }

  /// Whether leaving `page` would drop unsaved changes. Subpages that edit
  /// the same draft (shell, groups, members) keep it and do not ask.
  pub(crate) fn leaving_drops_draft(&self, page: Page) -> bool {
    match page {
      Page::User => self.any_user_draft_changed(),
      Page::CreateUser => self.create_user_changed(),
      Page::UserPassword => self.passwords_changed() || self.security_changed(),
      Page::Group => self.group_draft_changed(),
      Page::CreateGroup => self.create_group_changed(),
      Page::Firewall => self.firewall_draft_changed(),
      _ => false,
    }
  }

  /// Whether the new password is non-empty and matches its confirmation.
  pub(crate) fn new_password_is_valid(&self) -> bool {
    !self.passwords[1].is_empty() && self.passwords[1] == self.passwords[2]
  }
}
