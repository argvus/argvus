//! Rows of the Users & Groups and Firewall pages, built from the loaded data and the drafts.

use super::*;

impl Administration {
  /// Rows of an account or firewall page. `search` narrows the user and
  /// group lists, whose items keep their index in the unfiltered list.
  pub fn rows(&self, page: Page, lang: Lang, search: &str) -> Vec<Row<Item>> {
    let label = |key: &str| tr(lang, key);
    // Keep the already-loaded rows on screen while a write is running. The
    // placeholder is only useful when there is nothing to show yet; replacing
    // loaded rows made the user's selections vanish until the snapshot returned.
    if self.busy() && !self.has_loaded_data(page) {
      return vec![Row::info(
        label("control_center.waiting_for_backend_authentication"),
        "",
      )];
    }
    match page {
      Page::Firewall => self.firewall_rows(lang),
      Page::Users => vec![
        Row::submenu(Item::CreateUser, label("control_center.create")).icon(icons::ADD),
        Row::submenu(Item::UserList, label("control_center.list")).icon(icons::USERS),
        Row::submenu(Item::SystemUsers, label("control_center.system_accounts"))
          .icon(icons::ACCOUNT_COG),
      ],
      Page::UserList | Page::SystemUsers => self
        .users(page == Page::SystemUsers)
        .iter()
        .enumerate()
        .filter(|(_, user)| matches_search(search, &[&text(user, "user"), &text(user, "name")]))
        .map(|(index, user)| {
          Row::submenu(Item::UserEntry(index), text(user, "user")).detail(text(user, "name"))
        })
        .collect(),
      Page::CreateUser => self.create_rows(lang),
      Page::UserUsername => self.username_rows(lang),
      Page::User => self.user_rows(lang),
      Page::UserAvatar => self.avatar_rows(lang),
      Page::UserFullName => self.full_name_rows(lang),
      Page::UserDelete => self.delete_rows(lang),
      // The password page edits drafts only: Ok returns to the user page and
      // Save applies the password and the security options.
      // Creating an account only asks for the new password.
      Page::UserPassword if self.creating_password => vec![
        Row::info(label("control_center.esc_cancels"), String::new()),
        self.password_row(lang, 1),
        self.password_row(lang, 2),
        Row::separator(),
        Row::action(Item::OkPassword, label("control_center.password_ok")).icon(icons::APPLY),
      ],
      Page::UserPassword => vec![
        Row::section(label("control_center.password_configuration")),
        Row::info(label("control_center.esc_cancels"), String::new()),
        self.password_row(lang, 0),
        self.password_row(lang, 1),
        self.password_row(lang, 2),
        Row::section(label("control_center.security")),
        Row::toggle(
          Item::LockPassword,
          label("control_center.lock_password"),
          self.lock_draft == Some(true),
        )
        .icon(icons::LOCK),
        Row::toggle(
          Item::UnlockPassword,
          label("control_center.unlock_password"),
          self.lock_draft == Some(false),
        )
        .icon(icons::LOCK_OPEN),
        Row::toggle(
          Item::ExpirePassword,
          label("control_center.require_password_change_at_login"),
          self.expire_draft,
        )
        .icon(icons::LOCK_RESET),
        Row::separator(),
        Row::action(Item::OkPassword, label("control_center.password_ok")).icon(icons::APPLY),
      ],
      Page::UserShell | Page::UserPrimaryGroup => {
        let (source, field) = if page == Page::UserShell {
          ("shells", "shell")
        } else {
          ("groups", "primary_group")
        };
        let current = text(&self.user, field);
        strings(&self.accounts[source])
          .into_iter()
          .enumerate()
          .map(|(index, value)| {
            let item = if page == Page::UserShell {
              Item::ShellOption(index)
            } else {
              Item::PrimaryGroupOption(index)
            };
            let is_current = value == current;
            Row::choice(item, value, is_current)
          })
          .collect()
      }
      Page::UserGroups => {
        let primary = text(&self.user, "primary_group");
        let groups = strings(&self.user["groups"]);
        strings(&self.accounts["groups"])
          .into_iter()
          .enumerate()
          .map(|(index, group)| {
            let is_primary = group == primary;
            let on = is_primary || groups.contains(&group);
            // The primary group always stays a member; it cannot be removed here.
            Row::toggle(Item::GroupOption(index), group, on).enabled(!is_primary)
          })
          .collect()
      }
      Page::Groups => vec![
        Row::submenu(Item::CreateGroup, label("control_center.create"))
          .icon(icons::ADD)
          .enabled(self.is_admin()),
        Row::submenu(Item::GroupList, label("control_center.list")).icon(icons::GROUP),
      ],
      Page::GroupList | Page::SystemGroups => self
        .groups(true)
        .iter()
        .enumerate()
        .map(|(index, group)| {
          let members = strings(&group["members"]);
          let detail = if members.is_empty() {
            format!("GID {}", group["gid"])
          } else {
            format!("GID {} | {}", group["gid"], members.join(", "))
          };
          (index, text(group, "name"), detail)
        })
        .filter(|(_, name, detail)| matches_search(search, &[name, detail]))
        .map(|(index, name, detail)| Row::submenu(Item::GroupEntry(index), name).detail(detail))
        .collect(),
      Page::CreateGroup => {
        let mut rows = vec![
          Row::section(label("control_center.group")),
          Row::value(
            Item::GroupName,
            label("control_center.group_name"),
            text(&self.group, "name"),
            None,
          )
          .icon(icons::EDIT),
        ];
        rows.extend(self.draft_rows(
          Item::SubmitGroup,
          label("control_center.create_group"),
          self.create_group_changed(),
          lang,
        ));
        rows
      }
      Page::Group => {
        let mut rows = vec![
          Row::section(label("control_center.group")),
          Row::value(
            Item::GroupName,
            label("control_center.name_8d6abd"),
            text(&self.group, "name"),
            None,
          )
          .icon(icons::EDIT),
          Row::info(label("control_center.gid"), self.group["gid"].to_string()),
          Row::submenu(Item::Members, label("control_center.members"))
            .icon(icons::USERS)
            .detail(strings(&self.group["members"]).join(", ")),
        ];
        rows.extend(self.draft_rows(
          Item::SaveGroup,
          label("control_center.save_changes"),
          self.group_draft_changed(),
          lang,
        ));
        rows.push(Row::section(label("control_center.danger_zone")));
        rows.push(
          Row::destructive(Item::DeleteGroup, label("control_center.delete_group"))
            .icon(icons::DELETE),
        );
        rows
      }
      Page::GroupMembers => {
        let members = strings(&self.group["members"]);
        self
          .users(true)
          .iter()
          .enumerate()
          .map(|(index, user)| {
            let name = text(user, "user");
            let on = members.contains(&name);
            Row::toggle(Item::Member(index), name, on)
          })
          .collect()
      }
      _ => Vec::new(),
    }
  }

  /// System > Users > (user): account fields and their Save row, then the
  /// password and avatar actions, with the deletions in the Danger zone.
  /// The user page. The "User info" block holds the readonly identity and the
  /// fields that open their own page; the "Actions" block (after a section
  /// row, reached with Tab) holds Save, Cancel and Delete.
  pub(super) fn user_rows(&self, lang: Lang) -> Vec<Row<Item>> {
    let label = |key: &str| tr(lang, key);
    let avatar = if text(&self.user, "avatar").is_empty() {
      label("control_center.avatar_none")
    } else {
      label("control_center.avatar_loaded")
    };
    let mut rows = vec![
      Row::info(label("control_center.username"), text(&self.user, "user")),
      Row::info(
        label("control_center.uid_gid"),
        format!("{} / {}", self.user["uid"], self.user["gid"]),
      ),
      Row::info(
        label("control_center.home_directory"),
        text(&self.user, "home"),
      ),
      Row::submenu(Item::AvatarImage, label("control_center.avatar_image"))
        .icon(icons::AVATAR)
        .detail(avatar),
      Row::submenu(Item::FullName, label("control_center.full_name"))
        .icon(icons::ID_CARD)
        .detail(text(&self.user, "name")),
      Row::submenu(Item::Shell, label("control_center.shell"))
        .icon(icons::TERMINAL)
        .detail(text(&self.user, "shell")),
      Row::submenu(Item::PrimaryGroup, label("control_center.primary_group"))
        .icon(icons::ACCOUNT_STAR)
        .detail(text(&self.user, "primary_group")),
      Row::submenu(
        Item::SupplementaryGroups,
        label("control_center.user_groups"),
      )
      .icon(icons::GROUP),
      Row::toggle(
        Item::UserAdmin,
        label("control_center.make_admin"),
        has_admin_group(&strings(&self.user["groups"])),
      )
      .icon(icons::SHIELD)
      .enabled(self.is_admin()),
      Row::toggle(
        Item::ToggleAutoLogin,
        label("control_center.automatic_login"),
        self.is_autologin_user(&text(&self.user, "user")),
      )
      .icon(icons::BOOT)
      .enabled(self.is_admin()),
      Row::submenu(Item::ChangePassword, label("control_center.password"))
        .icon(icons::KEY)
        .detail("****"),
      Row::section(label("control_center.actions")),
    ];
    // Save is always selectable: with nothing to save it says so, instead of
    // being dead in the bar. The draft note appears only when there is a draft.
    let mut save = draft_actions(Item::SaveUser, label("control_center.save_user"), true);
    if self.any_user_draft_changed()
      && let Some(row) = save.pop()
    {
      save.push(row.detail(label("control_center.draft_changed")));
    }
    rows.extend(save);
    rows.extend([
      Row::action(Item::CancelUser, label("control_center.cancel")).icon(icons::CANCEL),
      Row::destructive(Item::DeleteUserMenu, label("control_center.delete_user"))
        .icon(icons::ACCOUNT_REMOVE),
    ]);
    rows
  }

  /// The create page: the same two blocks as the user page. The fields open
  /// their own pages, and Actions creates the account or cancels the form.
  pub(super) fn create_rows(&self, lang: Lang) -> Vec<Row<Item>> {
    let label = |key: &str| tr(lang, key);
    let user = text(&self.user, "user");
    let home = if user.is_empty() {
      String::new()
    } else {
      format!("/home/{user}")
    };
    let avatar = if text(&self.user, "avatar").is_empty() {
      label("control_center.avatar_none")
    } else {
      label("control_center.avatar_loaded")
    };
    let password = if self.passwords_changed() { "****" } else { "" };
    vec![
      Row::submenu(Item::AvatarImage, label("control_center.avatar_image"))
        .icon(icons::AVATAR)
        .detail(avatar),
      Row::submenu(Item::Username, label("control_center.username"))
        .icon(icons::USER)
        .detail(user),
      Row::submenu(Item::FullName, label("control_center.full_name"))
        .icon(icons::ID_CARD)
        .detail(text(&self.user, "name")),
      Row::submenu(Item::Shell, label("control_center.shell"))
        .icon(icons::TERMINAL)
        .detail(text(&self.user, "shell")),
      Row::info(label("control_center.home_directory"), home),
      Row::submenu(Item::ChangePassword, label("control_center.password"))
        .icon(icons::KEY)
        .detail(password),
      Row::submenu(
        Item::SupplementaryGroups,
        label("control_center.user_groups"),
      )
      .icon(icons::GROUP),
      Row::toggle(
        Item::CreateAdmin,
        label("control_center.make_admin"),
        self.create_admin,
      )
      .icon(icons::SHIELD),
      Row::section(label("control_center.actions")),
      Row::action(Item::CreateAccount, label("control_center.create")).icon(icons::ADD),
      Row::action(Item::CancelCreate, label("control_center.cancel")).icon(icons::CANCEL),
    ]
  }

  /// The username page of the create form: the hint says Esc cancels the edit.
  pub(super) fn username_rows(&self, lang: Lang) -> Vec<Row<Item>> {
    vec![
      Row::info(tr(lang, "control_center.esc_cancels"), String::new()),
      Row::value(
        Item::UsernameField,
        tr(lang, "control_center.username"),
        text(&self.user, "user"),
        None,
      )
      .icon(icons::USER),
    ]
  }

  /// The avatar page: edit the image path or remove the avatar.
  pub(super) fn avatar_rows(&self, lang: Lang) -> Vec<Row<Item>> {
    let has_avatar = !text(&self.user, "avatar").is_empty();
    vec![
      Row::action(Item::EditAvatar, tr(lang, "control_center.edit_avatar")).icon(icons::AVATAR),
      Row::action(Item::RemoveAvatar, tr(lang, "control_center.remove_avatar"))
        .icon(icons::IMAGE_REMOVE)
        .enabled(has_avatar),
    ]
  }

  /// The full name page: the hint says Esc cancels the edit.
  pub(super) fn full_name_rows(&self, lang: Lang) -> Vec<Row<Item>> {
    vec![
      Row::info(tr(lang, "control_center.esc_cancels"), String::new()),
      Row::value(
        Item::FullNameField,
        tr(lang, "control_center.full_name"),
        text(&self.user, "name"),
        None,
      )
      .icon(icons::ID_CARD),
    ]
  }

  /// The delete page: a choice between keeping the home directory and
  /// deleting it too. Either one asks the same confirmation.
  pub(super) fn delete_rows(&self, lang: Lang) -> Vec<Row<Item>> {
    let home = text(&self.user, "home");
    vec![
      Row::section(tr(lang, "control_center.choice")),
      Row::action(
        Item::DeleteUser,
        format!("1 - {} {home}", tr(lang, "control_center.maintain")),
      ),
      Row::destructive(
        Item::DeleteUserAndHome,
        format!("2 - {} {home}", tr(lang, "control_center.delete")),
      ),
    ]
  }

  /// Service state, the configuration draft with its Save and Cancel rows,
  /// and the rule actions.
  pub(super) fn firewall_rows(&self, lang: Lang) -> Vec<Row<Item>> {
    let label = |key: &str| tr(lang, key);
    if self.firewall.is_null() {
      return vec![
        Row::action(Item::LoadFirewall, label("control_center.reload")).icon(icons::REFRESH),
      ];
    }
    let mut rows = vec![
      Row::section(label("control_center.service")),
      // Paired Start/Stop: one status row whose Enter switches the state.
      Row::action(Item::FirewallService, label("control_center.service"))
        .icon(icons::SHIELD)
        .detail(if self.firewall["active"] == true {
          label("control_center.active_095d39")
        } else {
          label("control_center.stopped_c2dfd3")
        }),
      Row::toggle(
        Item::FirewallBoot,
        label("control_center.start_at_boot"),
        self.firewall["enabled"] == true,
      )
      .icon(icons::AUTOSTART),
      Row::section(label("control_center.configuration")),
    ];
    rows.extend(
      FIREWALL_FIELDS
        .iter()
        .enumerate()
        .map(|(index, (key, field_label))| {
          let value = text(&self.config, key);
          let item = Item::FirewallField(index);
          match value.as_str() {
            "y" | "n" => Row::toggle(item, label(field_label), value == "y"),
            _ if *key == "PROTECTION_LEVEL" => Row::value(item, label(field_label), value, Some(1)),
            _ => Row::value(item, label(field_label), value, None),
          }
        }),
    );
    rows.extend(self.draft_rows(
      Item::SaveFirewall,
      label("control_center.save_configuration"),
      self.firewall_draft_changed(),
      lang,
    ));
    rows.extend([
      Row::action(
        Item::DiscardFirewall,
        label("control_center.discard_config_changes"),
      )
      .icon(icons::CANCEL),
      Row::section(label("control_center.section_rules")),
      Row::action(Item::AddRules, label("control_center.add_iptables_rules")).icon(icons::SCRIPT),
      Row::action(Item::ApplyRules, label("control_center.apply_saved_rules"))
        .icon(icons::SHIELD_REFRESH),
    ]);
    rows
  }

  /// A masked password field: 0 = current, 1 = new, 2 = confirmation.
  pub(super) fn password_row(&self, lang: Lang, index: usize) -> Row<Item> {
    let key = match index {
      0 => "control_center.current_password_own_account",
      1 => "control_center.new_password",
      _ => "control_center.confirm_password",
    };
    Row::value(
      Item::Password(index),
      tr(lang, key),
      "*".repeat(argvus_tui::text::display_width(&self.passwords[index])),
      None,
    )
    .icon(if index == 0 { icons::LOCK } else { icons::KEY })
  }

  /// Label of the create-account row: the account stays password-locked
  /// unless a new password was typed.
  pub(super) fn create_account_label(&self, lang: Lang) -> &'static str {
    if self.passwords[1].is_empty() {
      tr(lang, "control_center.create_account_password_locked")
    } else {
      tr(lang, "control_center.create_account_with_password")
    }
  }

  /// The Save/Create row that commits a draft: dimmed and skipped while
  /// nothing changed, marked "changed" otherwise.
  pub(super) fn draft_rows(
    &self,
    item: Item,
    label: &str,
    changed: bool,
    lang: Lang,
  ) -> Vec<Row<Item>> {
    let mut rows = draft_actions(item, label, changed);
    if changed && let Some(row) = rows.pop() {
      rows.push(row.detail(tr(lang, "control_center.draft_changed")));
    }
    rows
  }
}

impl App {
  /// The action buttons of the user page: the rows after its "Actions"
  /// section, disabled ones included so the index matches the drawn bar.
  pub(crate) fn user_action_rows(&self) -> Vec<Row<Item>> {
    self
      .rows()
      .into_iter()
      .skip_while(|row| !row.is_section())
      .skip(1)
      .filter(|row| row.id().is_some())
      .collect()
  }
}
