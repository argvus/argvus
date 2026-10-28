//! Running a row of the Users & Groups or Firewall pages: toggles, edits, confirmations and the requests they send.

use super::editor::{EditTarget, Editor};
use super::*;

impl App {
  pub(super) fn admin_confirm(&mut self, firewall: bool, value: Value, message: String) {
    self.open_admin_confirm(firewall, value, message, false);
  }

  /// Confirmation of a deletion: drawn with the danger style.
  pub(super) fn admin_confirm_danger(&mut self, value: Value, message: String) {
    self.open_admin_confirm(false, value, message, true);
  }

  pub(super) fn open_admin_confirm(
    &mut self,
    firewall: bool,
    value: Value,
    message: String,
    danger: bool,
  ) {
    self.admin.pending = Some((firewall, value));
    self.confirm = Some(PendingAction::Administration { message, danger });
    self.confirm_focus = argvus_tui::confirm::ConfirmState::new();
  }

  /// Runs the row `item` of an account or firewall page. Toggles, actions,
  /// submenus and destructive rows all land here; the effect and its timing
  /// (draft edit, confirmation, page change) are the same as before rows
  /// replaced the button bar.
  pub(crate) fn admin_activate(&mut self, item: Item) {
    if self.admin.busy() {
      return;
    }
    match item {
      Item::LoadFirewall
      | Item::DiscardFirewall
      | Item::FirewallService
      | Item::FirewallBoot
      | Item::FirewallField(_)
      | Item::SaveFirewall
      | Item::AddRules
      | Item::ApplyRules => self.activate_firewall_row(item),
      Item::CreateUser
      | Item::UserList
      | Item::SystemUsers
      | Item::UserEntry(_)
      | Item::Username
      | Item::UsernameField
      | Item::CancelCreate
      | Item::FullName
      | Item::FullNameField
      | Item::Shell
      | Item::PrimaryGroup
      | Item::SupplementaryGroups
      | Item::Password(_)
      | Item::CreateAccount
      | Item::SaveUser
      | Item::CancelUser
      | Item::CreateAdmin
      | Item::UserAdmin
      | Item::ToggleAutoLogin
      | Item::ChangePassword
      | Item::OkPassword
      | Item::DeleteUserMenu
      | Item::LockPassword
      | Item::UnlockPassword
      | Item::ExpirePassword
      | Item::AvatarImage
      | Item::EditAvatar
      | Item::RemoveAvatar
      | Item::DeleteUser
      | Item::DeleteUserAndHome
      | Item::ShellOption(_)
      | Item::PrimaryGroupOption(_)
      | Item::GroupOption(_) => self.activate_user_row(item),
      Item::CreateGroup
      | Item::GroupList
      | Item::GroupEntry(_)
      | Item::GroupName
      | Item::Members
      | Item::Member(_)
      | Item::SubmitGroup
      | Item::SaveGroup
      | Item::DeleteGroup => self.activate_group_row(item),
      _ => {}
    }
  }

  /// Rows of the Firewall page: service and boot toggles, configuration fields and rules.
  fn activate_firewall_row(&mut self, item: Item) {
    let lang = self.lang;
    match item {
      Item::LoadFirewall | Item::DiscardFirewall => self.admin.load(true),
      Item::FirewallService | Item::FirewallBoot => {
        let value = if item == Item::FirewallService {
          json!({"action": if self.admin.firewall["active"] == true { "stop" } else { "start" }})
        } else {
          json!({"action": if self.admin.firewall["enabled"] == true { "disable" } else { "enable" }})
        };
        self.admin_confirm(
          true,
          value,
          tr(
            lang,
            "control_center.change_the_firewall_network_connectivity_may_be_interrupted_saving_doe",
          )
          .into(),
        );
      }
      Item::FirewallField(index) => {
        let Some((key, field_label)) = FIREWALL_FIELDS.get(index) else {
          return;
        };
        let value = text(&self.admin.config, key);
        if matches!(value.as_str(), "y" | "n") {
          self.admin.config[*key] = json!(if value == "y" { "n" } else { "y" });
        } else if *key == "PROTECTION_LEVEL" {
          self.cycle_protection_level(1);
        } else {
          self.admin.editor = Some(Editor::new(
            tr(lang, field_label).into(),
            value,
            EditTarget::Config((*key).into()),
            false,
          ));
        }
      }
      Item::SaveFirewall => {
        let value = json!({"action":"save-config", "original":self.admin.firewall["config_text"], "config":self.admin.config});
        self.admin_confirm(
          true,
          value,
          format!("{}?", tr(lang, "control_center.save_configuration")),
        );
      }
      Item::AddRules => {
        self.admin.editor = Some(Editor::new(
          tr(lang, "control_center.add_iptables_rules").into(),
          text(&self.admin.firewall, "rules"),
          EditTarget::Rules,
          true,
        ));
      }
      Item::ApplyRules => {
        self.admin_confirm(
          true,
          json!({"action":"restart"}),
          format!(
            "{}?\n{}",
            tr(lang, "control_center.apply_saved_rules"),
            tr(lang, "control_center.apply_saved_rules_description")
          ),
        );
      }
      // `admin_activate` routes only this page's rows here.
      _ => {}
    }
  }

  /// Rows of the user pages: the account list, the user drafts, password, avatar, lock and delete.
  fn activate_user_row(&mut self, item: Item) {
    let page = self.page();
    let lang = self.lang;
    let user = text(&self.admin.user, "user");
    let confirm_user = |key: &str| format!("{}: {user}?", tr(lang, key));
    match item {
      Item::CreateUser => {
        self.admin.user = json!({"user":"", "name":"", "shell":strings(&self.admin.accounts["shells"]).first().cloned().unwrap_or("/bin/bash".into()), "groups":[]});
        self.navigation.push(Page::CreateUser);
      }
      Item::UserList => self.navigation.push(Page::UserList),
      Item::SystemUsers => self.navigation.push(Page::SystemUsers),
      Item::UserEntry(index) => {
        if let Some(user) = self.admin.users(page == Page::SystemUsers).get(index) {
          self.admin.user = user.clone();
          self.navigation.push(Page::User);
        }
      }
      Item::Username => self.navigation.push(Page::UserUsername),
      Item::UsernameField => {
        self.admin.editor = Some(Editor::new(
          tr(lang, "control_center.username").into(),
          user,
          EditTarget::User("user".into()),
          false,
        ));
      }
      Item::CancelCreate => {
        self.admin.discard_draft(Page::CreateUser);
        self.navigation.back();
      }
      Item::FullName => self.navigation.push(Page::UserFullName),
      Item::FullNameField => {
        self.admin.editor = Some(Editor::new(
          tr(lang, "control_center.full_name").into(),
          text(&self.admin.user, "name"),
          EditTarget::User("name".into()),
          false,
        ));
      }
      Item::Shell => self.navigation.push(Page::UserShell),
      Item::PrimaryGroup => self.navigation.push(Page::UserPrimaryGroup),
      Item::SupplementaryGroups => self.navigation.push(Page::UserGroups),
      Item::Password(index) => {
        let key = match index {
          0 => "control_center.current_password_own_account",
          1 => "control_center.new_password",
          _ => "control_center.confirm_password",
        };
        self.admin.editor = Some(Editor::new(
          tr(lang, key).into(),
          self.admin.passwords[index].clone(),
          EditTarget::Password(index),
          false,
        ));
      }
      Item::CreateAccount => {
        if user.is_empty() {
          self.fail(tr(lang, "control_center.username_required"));
          return;
        }
        // A chosen avatar is set once the account exists, after the create.
        let avatar = text(&self.admin.user, "avatar");
        let queue = if avatar.is_empty() {
          Vec::new()
        } else {
          vec![json!({"action":"avatar", "user":user, "path":avatar})]
        };
        let title = self.admin.create_account_label(lang);
        let mut groups = strings(&self.admin.user["groups"]);
        if self.admin.create_admin && !has_admin_group(&groups) {
          groups.push(self.admin.admin_group().into());
        }
        let mut value = json!({"action":"create", "user":user, "name":self.admin.user["name"], "shell":self.admin.user["shell"], "groups":groups});
        if self.admin.passwords[1].is_empty() && self.admin.passwords[2].is_empty() {
          self.admin.queued = queue;
          self.admin_confirm(false, value, format!("{title}: {user}?"));
          return;
        }
        if !self.admin.new_password_is_valid() {
          self.reject_password();
          return;
        }
        value["password"] = self.admin.passwords[1].clone().into();
        self.admin.passwords = Default::default();
        self.admin.queued = queue;
        self.admin_confirm(false, value, format!("{title}: {user}?"));
      }
      // Save is the only place where the user page writes: the field edit,
      // the avatar and a typed password run in that order, one confirmation.
      Item::SaveUser => {
        if !self.admin.any_user_draft_changed() {
          self.success(tr(lang, "control_center.nothing_to_save").into());
          return;
        }
        if self.admin.passwords_changed() && !self.admin.new_password_is_valid() {
          self.reject_password();
          return;
        }
        let mut requests = Vec::new();
        if self.admin.user_draft_changed() {
          requests.push(json!({"action":"edit", "user":user, "name":self.admin.user["name"], "shell":self.admin.user["shell"], "groups":self.admin.user["groups"], "primary_group":self.admin.user["primary_group"]}));
        }
        if self.admin.avatar_draft_changed() {
          requests
            .push(json!({"action":"avatar", "user":user, "path":text(&self.admin.user, "avatar")}));
        }
        if self.admin.passwords_changed() {
          requests.push(json!({"action":"password", "user":user, "old":self.admin.passwords[0], "new":self.admin.passwords[1], "confirm":self.admin.passwords[2]}));
        }
        if let Some(locked) = self.admin.lock_draft {
          requests.push(json!({"action":"lock", "user":user, "locked":locked}));
        }
        if self.admin.expire_draft {
          requests.push(json!({"action":"expire-password", "user":user}));
        }
        let first = requests.remove(0);
        self.admin.queued = requests;
        self.admin_confirm(false, first, confirm_user("control_center.save_changes"));
      }
      // Cancel only discards the edits; the user page stays open.
      Item::CancelUser => self.admin.discard_draft(Page::User),
      Item::CreateAdmin => self.admin.create_admin = !self.admin.create_admin,
      // The administrator membership is part of the groups draft, so Save
      // applies it. Turning it off leaves every administrator group.
      Item::UserAdmin => {
        let mut groups = strings(&self.admin.user["groups"]);
        if has_admin_group(&groups) {
          groups.retain(|group| !ADMIN_GROUPS.contains(&group.as_str()));
        } else {
          groups.push(self.admin.admin_group().into());
        }
        self.admin.user["groups"] = json!(groups);
      }
      // Not a draft: applied immediately, like the firewall toggles, since it
      // changes greetd's boot-time behavior rather than this account's fields.
      Item::ToggleAutoLogin => {
        let enabled = !self.admin.is_autologin_user(&user);
        let key = if enabled {
          "control_center.automatic_login_confirm_enable"
        } else {
          "control_center.automatic_login_confirm_disable"
        };
        self.admin_confirm(
          false,
          json!({"action":"auto-login", "user":user, "enabled":enabled}),
          confirm_user(key),
        );
      }
      Item::ChangePassword => {
        self.admin.creating_password = page == Page::CreateUser;
        self.navigation.push(Page::UserPassword);
      }
      // Ok returns to the user page; the password is applied by its Save.
      Item::OkPassword => {
        self.navigation.back();
      }
      Item::DeleteUserMenu => self.navigation.push(Page::UserDelete),
      // Lock and Unlock are exclusive options: choosing one again clears it.
      Item::LockPassword => {
        self.admin.lock_draft = if self.admin.lock_draft == Some(true) {
          None
        } else {
          Some(true)
        };
      }
      Item::UnlockPassword => {
        self.admin.lock_draft = if self.admin.lock_draft == Some(false) {
          None
        } else {
          Some(false)
        };
      }
      Item::ExpirePassword => self.admin.expire_draft = !self.admin.expire_draft,
      Item::AvatarImage => self.navigation.push(Page::UserAvatar),
      Item::EditAvatar => {
        self.admin.editor = Some(Editor::new(
          tr(lang, "control_center.avatar_image").into(),
          String::new(),
          EditTarget::Avatar,
          false,
        ));
      }
      // Removing the avatar is a draft too: Save applies it.
      Item::RemoveAvatar => self.admin.user["avatar"] = json!(""),
      // Either choice asks the same irreversible-delete confirmation; yes and
      // no both return to the user list.
      Item::DeleteUser | Item::DeleteUserAndHome => {
        let remove_home = item == Item::DeleteUserAndHome;
        self.admin.delete_confirm = true;
        self.admin_confirm_danger(
          json!({"action":"delete", "user":user, "remove_home":remove_home}),
          tr(lang, "control_center.delete_irreversible").into(),
        );
      }
      Item::ShellOption(index) | Item::PrimaryGroupOption(index) => {
        let (source, field) = if matches!(item, Item::ShellOption(_)) {
          ("shells", "shell")
        } else {
          ("groups", "primary_group")
        };
        if let Some(value) = strings(&self.admin.accounts[source]).get(index) {
          self.admin.user[field] = json!(value);
          self.navigation.back();
        }
      }
      Item::GroupOption(index) => {
        if let Some(group) = strings(&self.admin.accounts["groups"]).get(index) {
          if *group == text(&self.admin.user, "primary_group") {
            return;
          }
          let mut groups = strings(&self.admin.user["groups"]);
          if groups.contains(group) {
            groups.retain(|value| value != group);
          } else {
            groups.push(group.clone());
          }
          self.admin.user["groups"] = json!(groups);
        }
      }
      // `admin_activate` routes only this page's rows here.
      _ => {}
    }
  }

  /// Rows of the group pages: the group list, its name, members, save and delete.
  fn activate_group_row(&mut self, item: Item) {
    let page = self.page();
    let lang = self.lang;
    match item {
      Item::CreateGroup => {
        self.admin.group = json!({"name":""});
        self.navigation.push(Page::CreateGroup);
      }
      Item::GroupList => self.navigation.push(Page::GroupList),
      Item::GroupEntry(index) => {
        if let Some(group) = self.admin.groups(true).get(index) {
          self.admin.group = group.clone();
          self.admin.group["original"] = self.admin.group["name"].clone();
          self.navigation.push(Page::Group);
        }
      }
      Item::GroupName => {
        let key = if page == Page::CreateGroup {
          "control_center.group_name"
        } else {
          "control_center.name_8d6abd"
        };
        self.admin.editor = Some(Editor::new(
          tr(lang, key).into(),
          text(&self.admin.group, "name"),
          EditTarget::Group,
          false,
        ));
      }
      Item::Members => self.navigation.push(Page::GroupMembers),
      Item::Member(index) => {
        if let Some(user) = self.admin.users(true).get(index)
          && let Some(member) = user["user"].as_str()
        {
          let mut members = strings(&self.admin.group["members"]);
          if members.iter().any(|value| value == member) {
            members.retain(|value| value != member);
          } else {
            members.push(member.to_string());
          }
          self.admin.group["members"] = json!(members);
        }
      }
      Item::SubmitGroup => {
        let group = text(&self.admin.group, "name");
        self.admin_confirm(
          false,
          json!({"action":"create-group", "group":group}),
          format!("{}: {group}?", tr(lang, "control_center.create_group")),
        );
      }
      Item::SaveGroup => {
        self.admin_confirm(
          false,
          json!({
            "action":"edit-group",
            "group": text(&self.admin.group, "original"),
            "name": text(&self.admin.group, "name"),
            "members": self.admin.group["members"]}),
          format!(
            "{}: {}?",
            tr(lang, "control_center.save_changes"),
            text(&self.admin.group, "original")
          ),
        );
      }
      Item::DeleteGroup => {
        let group = text(&self.admin.group, "original");
        self.admin_confirm_danger(
          json!({"action":"delete-group", "group":group}),
          format!("{}: {group}?", tr(lang, "control_center.delete_group")),
        );
      }
      // `admin_activate` routes only this page's rows here.
      _ => {}
    }
  }

  /// `←/→` on a Value row with a step of an account or firewall page.
  pub(crate) fn admin_adjust(&mut self, item: Item, delta: i32) {
    if self.admin.busy() {
      return;
    }
    if let Item::FirewallField(index) = item
      && FIREWALL_FIELDS
        .get(index)
        .is_some_and(|(key, _)| *key == "PROTECTION_LEVEL")
    {
      self.cycle_protection_level(delta);
    }
  }

  /// Moves the firewall protection level by `delta` steps, wrapping around.
  pub(super) fn cycle_protection_level(&mut self, delta: i32) {
    let levels = ["low", "medium", "high", "paranoid"];
    let value = text(&self.admin.config, "PROTECTION_LEVEL");
    let index = levels.iter().position(|level| *level == value).unwrap_or(0) as i32;
    let next = (index + delta.signum()).rem_euclid(levels.len() as i32) as usize;
    self.admin.config["PROTECTION_LEVEL"] = json!(levels[next]);
  }

  /// Rejects an empty or unconfirmed new password before any confirmation.
  pub(super) fn reject_password(&mut self) {
    self.error_modal = Some(
      tr(
        self.lang,
        "control_center.the_new_password_must_be_nonempty_and_match_its_confirmation",
      )
      .into(),
    );
  }
}
