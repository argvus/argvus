//! Implements user and group administration in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::io::Write;
use std::process::Stdio;
use std::sync::mpsc::{self, Receiver};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
  Frame,
  layout::Rect,
  style::Style,
  widgets::{Block, Clear, Paragraph},
};
use serde_json::{Value, json};

use argvus_tui::icons;
use argvus_tui::menu::{Row, draft_actions};

use std::collections::BTreeSet;

use crate::{
  Page,
  app::{App, PendingAction},
  i18n::{Lang, tr},
  item::Item,
};

/// Defines the constant `FIREWALL_FIELDS`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const FIREWALL_FIELDS: [(&str, &str); 16] = [
  ("INTERFACE_WAN", "control_center.wan_interface"),
  ("INTERFACE_LAN", "control_center.lan_interface"),
  ("MASQUERADE_ENABLE", "control_center.nat_masquerading"),
  ("PROTECTION_LEVEL", "control_center.protection_level"),
  ("ALLOW_SSH", "control_center.allow_ssh"),
  ("SSH_CLIENTS_IP", "control_center.ssh_ipv4_networks"),
  ("SSH_PORT", "control_center.ssh_port"),
  ("ALLOW_SAMBA", "control_center.allow_samba"),
  ("SAMBA_CLIENTS_IP", "control_center.samba_ipv4_networks"),
  ("ALLOW_ICMP", "control_center.allow_icmp"),
  ("OPEN_PORTS_UDP", "control_center.udp_ports"),
  ("OPEN_PORTS_TCP", "control_center.tcp_ports"),
  (
    "SYN_FLOOD_PROTECTION",
    "control_center.syn_flood_protection",
  ),
  ("DDOS_PROTECTION", "control_center.ddos_protection"),
  (
    "PORT_SCAN_PROTECTION",
    "control_center.port_scan_protection",
  ),
  ("ANTI_SPOOFING", "control_center.anti_spoofing"),
];

/// Checks the condition represented by `is_page` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
/// Pages drawn with the two-block layout (User Data and Actions): the user
/// page and the create form.
pub fn is_user_form(page: Page) -> bool {
  matches!(page, Page::User | Page::CreateUser)
}

pub fn is_page(page: Page) -> bool {
  matches!(
    page,
    Page::Firewall
      | Page::Users
      | Page::UserList
      | Page::SystemUsers
      | Page::User
      | Page::CreateUser
      | Page::UserUsername
      | Page::UserGroups
      | Page::UserPassword
      | Page::UserShell
      | Page::UserPrimaryGroup
      | Page::UserAvatar
      | Page::UserFullName
      | Page::UserDelete
      | Page::Groups
      | Page::GroupList
      | Page::SystemGroups
      | Page::Group
      | Page::GroupMembers
      | Page::CreateGroup
  )
}

/// Executes the `text` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn text(value: &Value, key: &str) -> String {
  value[key].as_str().unwrap_or_default().into()
}
/// Executes the `strings` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn strings(value: &Value) -> Vec<String> {
  value
    .as_array()
    .map(|items| {
      items
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
    })
    .unwrap_or_default()
}
/// Case-insensitive search over the visible texts of a list entry.
fn matches_search(query: &str, values: &[&str]) -> bool {
  crate::app::search_matches(query.trim(), values)
}

/// Executes the `request` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn request(firewall: bool, value: Value, privileged: bool) -> Result<Value, String> {
  let binary = if firewall {
    "/usr/lib/argvus/commands/argvus-firewall"
  } else {
    "/usr/lib/argvus/commands/argvus-accounts"
  };
  let mut command = argvus_control_center_core::process::command(if privileged {
    "/usr/bin/pkexec"
  } else {
    binary
  });
  if privileged {
    command
      .arg("--disable-internal-agent")
      .arg(binary)
      .env_clear();
    for name in [
      "PATH",
      "DBUS_SESSION_BUS_ADDRESS",
      "DBUS_SYSTEM_BUS_ADDRESS",
      "XDG_RUNTIME_DIR",
    ] {
      if let Some(value) = std::env::var_os(name) {
        command.env(name, value);
      }
    }
    if std::env::var_os("PATH").is_none() {
      command.env(
        "PATH",
        "/usr/local/sbin:/usr/local/bin:/usr/bin:/usr/sbin:/sbin:/bin",
      );
    }
  }
  let mut child = command
    .arg("manage")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("{binary}: {error}"))?;
  let payload = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
  let written = child
    .stdin
    .take()
    .ok_or("missing backend input")?
    .write_all(&payload);
  let output = child
    .wait_with_output()
    .map_err(|error| error.to_string())?;
  if !output.status.success() {
    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    return Err(if message.is_empty() {
      format!("{binary}: {}", output.status)
    } else {
      message
    });
  }
  written.map_err(|error| error.to_string())?;
  serde_json::from_slice(&output.stdout).map_err(|error| format!("{binary}: {error}"))
}

/// Represents `Administration`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Administration {
  firewall: Value,
  pub(crate) accounts: Value,
  config: Value,
  pub(crate) user: Value,
  group: Value,
  passwords: [String; 3],
  pub editor: Option<Editor>,
  pending: Option<(bool, Value)>,
  /// Requests of the same Save still waiting: each runs after the previous one succeeds.
  queued: Vec<Value>,
  /// Index of the focused action button of the user page (see `user_action_rows`).
  user_button: usize,
  /// Whether Left/Right and Enter act on the action buttons instead of the info list.
  user_buttons_focused: bool,
  /// The open confirmation is the delete one: its yes and no both leave the user pages.
  pub(crate) delete_confirm: bool,
  /// Create form: the new account is made an administrator (joins `sudo`).
  pub(crate) create_admin: bool,
  /// The password page was opened from the create form (only new password rows).
  pub(crate) creating_password: bool,
  /// Lock draft: `Some(true)` locks, `Some(false)` unlocks, `None` keeps the account as is.
  lock_draft: Option<bool>,
  /// Require a password change at the next login, applied by Save.
  expire_draft: bool,
  worker: Option<Receiver<Result<(bool, Value), String>>>,
  operation: String,
  pub completed: Option<String>,
}

impl Administration {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(page: Page) -> Self {
    let mut state = Self {
      firewall: Value::Null,
      accounts: Value::Null,
      config: Value::Null,
      user: Value::Null,
      group: Value::Null,
      passwords: Default::default(),
      editor: None,
      pending: None,
      queued: Vec::new(),
      user_button: 0,
      user_buttons_focused: false,
      delete_confirm: false,
      create_admin: false,
      creating_password: false,
      lock_draft: None,
      expire_draft: false,
      worker: None,
      operation: String::new(),
      completed: None,
    };
    if is_page(page) {
      state.load(page == Page::Firewall);
    }
    state
  }

  /// Executes the `busy` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn busy(&self) -> bool {
    self.worker.is_some()
  }

  /// Reports whether the snapshot needed to render `page` has already been loaded.
  fn has_loaded_data(&self, page: Page) -> bool {
    match page {
      Page::Firewall => !self.firewall.is_null(),
      _ => self.is_loaded(),
    }
  }
  /// Executes the `cancel_pending` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn cancel_pending(&mut self) {
    self.pending = None;
    self.queued.clear();
  }

  /// Loads the snapshot of an administration page when it is opened and its
  /// data is still missing. Pages opened after the settings app was built
  /// (from Home, search or the CLI) would otherwise show only their Reload row.
  pub fn ensure_loaded(&mut self, page: Page) {
    if is_page(page) && !self.busy() && !self.has_loaded_data(page) {
      self.load(page == Page::Firewall);
    }
  }

  /// Retrieves data for `load` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn load(&mut self, firewall: bool) {
    if self.busy() {
      return;
    }
    self.start(firewall, json!({"action":"snapshot"}), false);
  }

  /// Executes the `start` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn start(&mut self, firewall: bool, value: Value, privileged: bool) {
    self.operation = text(&value, "action");
    let (sender, receiver) = mpsc::channel();
    self.worker = Some(receiver);
    std::thread::spawn(move || {
      let result = request(firewall, value, privileged)
        .and_then(|value| {
          if privileged {
            request(firewall, json!({"action":"snapshot"}), false)
          } else {
            Ok(value)
          }
        })
        .map(|value| (firewall, value));
      let _ = sender.send(result);
    });
  }

  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> Option<Result<(), String>> {
    let result = match self.worker.as_ref()?.try_recv() {
      Ok(result) => result,
      Err(mpsc::TryRecvError::Empty) => return None,
      Err(mpsc::TryRecvError::Disconnected) => Err("backend worker disconnected".into()),
    };
    self.worker = None;
    self.delete_confirm = false;
    let mut queued = std::mem::take(&mut self.queued);
    let done = result.map(|(firewall, value)| {
      self.completed = Some(self.operation.clone());
      if firewall {
        if matches!(self.operation.as_str(), "snapshot" | "save-config") {
          self.config = value["config"].clone();
        }
        self.firewall = value;
      } else {
        // A reload never replaces a draft with unsaved changes (D15); a
        // completed write (edit, create...) always refreshes it.
        let reload = self.operation == "snapshot";
        let keep_user = reload && self.user_draft_changed();
        let keep_group = reload && self.group_draft_changed();
        if let Some(users) = value["users"].as_array()
          && let Some(user) = users.iter().find(|user| user["user"] == self.user["user"])
          && matches!(
            self.operation.as_str(),
            "snapshot" | "edit" | "create" | "avatar" | "password"
          )
          && !keep_user
        {
          self.user = user.clone();
        }
        if !keep_group
          && let Some(groups) = value["group_details"].as_array()
          && let Some(group) = groups.iter().find(|group| {
            group["name"] == self.group["original"] || group["gid"] == self.group["gid"]
          })
        {
          self.group = group.clone();
          self.group["original"] = self.group["name"].clone();
        }
        self.accounts = value;
        if self.operation == "delete" {
          self.user = Value::Null;
        }
        if self.operation == "delete-group" {
          self.group = Value::Null;
        }
      }
    });
    // The requests of one Save run in order (edit, avatar, password). A failed
    // step stops the chain and keeps the drafts, so Save can be tried again.
    if done.is_ok() {
      // The password draft is cleared only once the password change succeeded.
      match self.operation.as_str() {
        "password" => self.passwords = Default::default(),
        "lock" => self.lock_draft = None,
        "expire-password" => self.expire_draft = false,
        "create" => self.create_admin = false,
        _ => {}
      }
      if !queued.is_empty() {
        let next = queued.remove(0);
        self.queued = queued;
        self.start(false, next, true);
      }
    }
    Some(done)
  }

  /// Executes the `submit` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn submit(&mut self) {
    if let Some((firewall, value)) = self.pending.take() {
      self.start(firewall, value, true);
    }
  }

  /// Checks the condition represented by `is_loaded` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn is_loaded(&self) -> bool {
    self.accounts["users"].is_array()
  }

  /// Executes the `users` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub(crate) fn users(&self, include_system: bool) -> Vec<Value> {
    self.accounts["users"]
      .as_array()
      .map(|users| {
        users
          .iter()
          .filter(|user| {
            let uid = user["uid"].as_u64().unwrap_or(0);
            include_system
              || (1000..65534).contains(&uid)
              || user["uid"] == self.accounts["actor_uid"]
          })
          .cloned()
          .collect()
      })
      .unwrap_or_default()
  }

  /// Executes the `groups` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub(crate) fn groups(&self, include_system: bool) -> Vec<Value> {
    self.accounts["group_details"]
      .as_array()
      .map(|groups| {
        groups
          .iter()
          .filter(|group| include_system || group["gid"].as_u64().unwrap_or(0) >= 1000)
          .cloned()
          .collect()
      })
      .unwrap_or_else(|| {
        strings(&self.accounts["groups"])
          .into_iter()
          .filter(|name| include_system || !matches!(name.as_str(), "root" | "bin" | "daemon"))
          .map(|name| json!({"name":name,"gid":"","members":[]}))
          .collect()
      })
  }

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
  fn user_rows(&self, lang: Lang) -> Vec<Row<Item>> {
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
        strings(&self.user["groups"])
          .iter()
          .any(|group| group == "sudo"),
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
  fn create_rows(&self, lang: Lang) -> Vec<Row<Item>> {
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
  fn username_rows(&self, lang: Lang) -> Vec<Row<Item>> {
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
  fn avatar_rows(&self, lang: Lang) -> Vec<Row<Item>> {
    let has_avatar = !text(&self.user, "avatar").is_empty();
    vec![
      Row::action(Item::EditAvatar, tr(lang, "control_center.edit_avatar")).icon(icons::AVATAR),
      Row::action(Item::RemoveAvatar, tr(lang, "control_center.remove_avatar"))
        .icon(icons::IMAGE_REMOVE)
        .enabled(has_avatar),
    ]
  }

  /// The full name page: the hint says Esc cancels the edit.
  fn full_name_rows(&self, lang: Lang) -> Vec<Row<Item>> {
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
  fn delete_rows(&self, lang: Lang) -> Vec<Row<Item>> {
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
  fn firewall_rows(&self, lang: Lang) -> Vec<Row<Item>> {
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
  fn password_row(&self, lang: Lang, index: usize) -> Row<Item> {
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
  fn create_account_label(&self, lang: Lang) -> &'static str {
    if self.passwords[1].is_empty() {
      tr(lang, "control_center.create_account_password_locked")
    } else {
      tr(lang, "control_center.create_account_with_password")
    }
  }

  /// The Save/Create row that commits a draft: dimmed and skipped while
  /// nothing changed, marked "changed" otherwise.
  fn draft_rows(&self, item: Item, label: &str, changed: bool, lang: Lang) -> Vec<Row<Item>> {
    let mut rows = draft_actions(item, label, changed);
    if changed && let Some(row) = rows.pop() {
      rows.push(row.detail(tr(lang, "control_center.draft_changed")));
    }
    rows
  }

  /// The loaded account the user page edits.
  fn loaded_user(&self) -> Option<&Value> {
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
  fn loaded_group(&self) -> Option<&Value> {
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

impl App {
  /// Executes the `admin_paste` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn admin_paste(&mut self, text: &str) {
    if let Some(editor) = self.admin.editor.as_mut() {
      for character in text.replace("\r\n", "\n").chars() {
        if editor.value.len() >= 262144 {
          break;
        }
        if !character.is_control() || (editor.multiline && matches!(character, '\n' | '\t')) {
          editor.value.insert(editor.cursor, character);
          editor.cursor += 1;
        }
      }
    }
  }
  /// Executes the `admin_confirm` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn admin_confirm(&mut self, firewall: bool, value: Value, message: String) {
    self.open_admin_confirm(firewall, value, message, false);
  }

  /// Confirmation of a deletion: drawn with the danger style.
  fn admin_confirm_danger(&mut self, value: Value, message: String) {
    self.open_admin_confirm(false, value, message, true);
  }

  fn open_admin_confirm(&mut self, firewall: bool, value: Value, message: String, danger: bool) {
    self.admin.pending = Some((firewall, value));
    self.confirm = Some(PendingAction::Administration { message, danger });
    self.confirm_focus = argvus_tui::confirm::ConfirmState::new();
  }

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

  /// Index of the focused action button, or `None` while the info list has focus.
  pub(crate) fn user_button_cursor(&self) -> Option<usize> {
    self
      .admin
      .user_buttons_focused
      .then_some(self.admin.user_button)
  }

  /// Left/Right on the user page: moves across the enabled buttons, like
  /// cfdisk does on its bar. The first press only moves the focus there.
  pub(crate) fn move_user_button(&mut self, backwards: bool) {
    let enabled: Vec<usize> = self
      .user_action_rows()
      .iter()
      .enumerate()
      .filter(|(_, row)| row.is_selectable())
      .map(|(index, _)| index)
      .collect();
    let Some(&first) = enabled.first() else {
      return;
    };
    if !self.admin.user_buttons_focused {
      self.admin.user_buttons_focused = true;
      self.admin.user_button = first;
      return;
    }
    let current = self.admin.user_button;
    let next = if backwards {
      enabled
        .iter()
        .rev()
        .find(|&&index| index < current)
        .or(enabled.last())
    } else {
      enabled
        .iter()
        .find(|&&index| index > current)
        .or(enabled.first())
    };
    if let Some(&index) = next {
      self.admin.user_button = index;
    }
  }

  /// Up/Down/Home/End and similar: give the focus back to the info list.
  pub(crate) fn focus_user_info(&mut self) {
    self.admin.user_buttons_focused = false;
  }

  /// Tab: switches focus between the info list and the buttons.
  pub(crate) fn toggle_user_focus(&mut self) {
    if self.admin.user_buttons_focused {
      self.focus_user_info();
    } else {
      self.move_user_button(false);
    }
  }

  /// Leaves the user pages for the user list: after a delete, or when the
  /// delete confirmation is answered no. The page's edits go with them.
  pub(crate) fn leave_user_pages(&mut self) {
    self.admin.delete_confirm = false;
    self.admin.discard_draft(Page::User);
    while matches!(
      self.page(),
      Page::User
        | Page::UserDelete
        | Page::UserAvatar
        | Page::UserFullName
        | Page::UserPassword
        | Page::UserShell
        | Page::UserPrimaryGroup
        | Page::UserGroups
    ) && self.navigation.back()
    {}
    self.normalize_selection();
  }

  /// Enter on a focused button: runs its row, as Enter on the row would.
  pub(crate) fn activate_user_button(&mut self) {
    let rows = self.user_action_rows();
    let Some(row) = rows.get(self.admin.user_button) else {
      return;
    };
    if !row.is_selectable() {
      return;
    }
    if let Some(item) = row.id().copied() {
      self.activate(item);
    }
  }

  /// Runs the row `item` of an account or firewall page. Toggles, actions,
  /// submenus and destructive rows all land here; the effect and its timing
  /// (draft edit, confirmation, page change) are the same as before rows
  /// replaced the button bar.
  pub(crate) fn admin_activate(&mut self, item: Item) {
    if self.admin.busy() {
      return;
    }
    let page = self.page();
    let lang = self.lang;
    let user = text(&self.admin.user, "user");
    let confirm_user = |key: &str| format!("{}: {user}?", tr(lang, key));
    match item {
      // Firewall.
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

      // Users.
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
        if self.admin.create_admin && !groups.iter().any(|group| group == "sudo") {
          groups.push("sudo".into());
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
      // The sudo membership is part of the groups draft, so Save applies it.
      Item::UserAdmin => {
        let mut groups = strings(&self.admin.user["groups"]);
        if groups.iter().any(|group| group == "sudo") {
          groups.retain(|group| group != "sudo");
        } else {
          groups.push("sudo".into());
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

      // Groups.
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
  fn cycle_protection_level(&mut self, delta: i32) {
    let levels = ["low", "medium", "high", "paranoid"];
    let value = text(&self.admin.config, "PROTECTION_LEVEL");
    let index = levels.iter().position(|level| *level == value).unwrap_or(0) as i32;
    let next = (index + delta.signum()).rem_euclid(levels.len() as i32) as usize;
    self.admin.config["PROTECTION_LEVEL"] = json!(levels[next]);
  }

  /// Rejects an empty or unconfirmed new password before any confirmation.
  fn reject_password(&mut self) {
    self.error_modal = Some(
      tr(
        self.lang,
        "control_center.the_new_password_must_be_nonempty_and_match_its_confirmation",
      )
      .into(),
    );
  }

  /// Executes the `admin_input` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn admin_input(&mut self, key: KeyEvent) {
    let Some(mut editor) = self.admin.editor.take() else {
      return;
    };
    if key.code == KeyCode::Esc {
      return;
    }
    let save = (key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL))
      || (key.code == KeyCode::Enter && !editor.multiline);
    if save {
      let value: String = editor.value.iter().collect();
      match editor.target {
        EditTarget::Config(key) => self.admin.config[&key] = json!(value),
        EditTarget::User(key) => self.admin.user[&key] = json!(value),
        EditTarget::Password(index) => self.admin.passwords[index] = value,
        EditTarget::Rules => self.admin_confirm(
          true,
          json!({"action":"save-rules", "original":self.admin.firewall["rules"], "rules":value}),
          tr(
            self.lang,
            "control_center.save_rules_fw_this_script_will_run_as_root_when_applying_the_firewall",
          )
          .into(),
        ),
        EditTarget::Group => self.admin.group["name"] = json!(value),
        EditTarget::FontSize => match value.trim().parse::<u16>() {
          Ok(size) if (8..=32).contains(&size) => self.pending_size = size,
          _ => self.fail(format!(
            "{}: 8–32",
            tr(self.lang, "control_center.font_size")
          )),
        },
        // Window rule fields are drafts: Apply on the rule saves them.
        EditTarget::WindowRuleName(name) => {
          if let Some(loaded) = self
            .window_rules
            .iter()
            .find(|rule| rule.name == name)
            .cloned()
          {
            let draft = self.window_rule_drafts.entry(name).or_insert(loaded);
            draft.name = value.trim().to_owned();
          }
        }
        EditTarget::WindowRuleClasses(name) => {
          let classes: Vec<String> = value
            .split(',')
            .map(str::trim)
            .filter(|class| !class.is_empty())
            .map(str::to_owned)
            .collect();
          if let Some(loaded) = self
            .window_rules
            .iter()
            .find(|rule| rule.name == name)
            .cloned()
          {
            let draft = self.window_rule_drafts.entry(name).or_insert(loaded);
            draft.classes = classes;
          }
        }
        EditTarget::NewWindowRuleName => self.new_window_rule_name = value,
        EditTarget::ProjectPath(root) => match crate::system::projects::add(&value, root) {
          Ok(()) => {
            self.projects = crate::system::projects::load();
            self.success(tr(self.lang, "control_center.projects_added").to_string());
          }
          Err(error) => self.fail(error),
        },
        EditTarget::NewSnippetName => self.new_snippet_name = value,
        EditTarget::NewSnippetContent => self.new_snippet_content = value,
        EditTarget::NewSshKeyName => self.new_ssh_key_name = value,
        EditTarget::GpgFullName => self.gpg_form.full_name = value,
        EditTarget::GpgEmail => self.gpg_form.email = value,
        EditTarget::GpgComment => self.gpg_form.comment = value,
        EditTarget::GpgExpiry => self.gpg_form.expiry = value.trim().to_owned(),
        // A passphrase is stored as typed: trimming would change the secret.
        EditTarget::GpgPassphrase => self.gpg_form.passphrase = value,
        EditTarget::GpgPassphraseConfirm => self.gpg_form.passphrase_confirm = value,
        EditTarget::GpgExportPrivatePath(fingerprint) => {
          let home = std::env::var("HOME").unwrap_or_default();
          let path = crate::system::gpg_keys::expand_home(value.trim(), &home);
          match crate::system::gpg_keys::export_private(&fingerprint, &path) {
            Ok(()) => {
              self.success(tr(self.lang, "control_center.gpg_keys_private_exported").to_string())
            }
            Err(error) => self.fail(error),
          }
        }
        EditTarget::GpgImportPrivatePath => {
          let home = std::env::var("HOME").unwrap_or_default();
          let path = crate::system::gpg_keys::expand_home(value.trim(), &home);
          match crate::system::gpg_keys::import_private(&path) {
            Ok(()) => {
              self.gpg_keys = crate::system::gpg_keys::load();
              self.success(tr(self.lang, "control_center.gpg_keys_private_imported").to_string());
            }
            Err(error) => self.fail(error),
          }
        }
        EditTarget::GpgEditExpiry => self.gpg_edit_expiry = value.trim().to_owned(),
        EditTarget::GpgExportPath(fingerprint) => {
          let home = std::env::var("HOME").unwrap_or_default();
          let path = crate::system::gpg_keys::expand_home(value.trim(), &home);
          match crate::system::gpg_keys::export_public(&fingerprint, &path) {
            Ok(()) => self.success(tr(self.lang, "control_center.gpg_keys_exported").to_string()),
            Err(error) => self.fail(error),
          }
        }
        EditTarget::GpgImportPath => {
          let home = std::env::var("HOME").unwrap_or_default();
          let path = crate::system::gpg_keys::expand_home(value.trim(), &home);
          match crate::system::gpg_keys::import(&path) {
            Ok(()) => {
              self.gpg_keys = crate::system::gpg_keys::load();
              self.success(tr(self.lang, "control_center.gpg_keys_imported").to_string());
            }
            Err(error) => self.fail(error),
          }
        }
        EditTarget::SnippetContent(name) => match crate::system::snippets::add(&name, &value) {
          Ok(()) => {
            self.snippets = crate::system::snippets::load();
            self.success(tr(self.lang, "control_center.snippets_saved").to_string());
          }
          Err(error) => self.fail(error),
        },
        EditTarget::DateTime => match crate::system::time::set_local_time(&value) {
          Ok(()) => self.refresh_time(),
          Err(error) => self.fail(error),
        },
        // The avatar is a draft like the other fields: Save applies it.
        EditTarget::Avatar => self.admin.user["avatar"] = json!(value),
      }
    } else {
      editor.input(key);
      self.admin.editor = Some(editor);
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
/// Defines `EditTarget`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub(crate) enum EditTarget {
  Config(String),
  User(String),
  Password(usize),
  Rules,
  Group,
  Avatar,
  DateTime,
  /// Size used when a font is applied on the font selector (8–32).
  FontSize,
  /// Draft name of a saved rule, keyed by its loaded name.
  WindowRuleName(String),
  WindowRuleClasses(String),
  /// The name typed for the next rule.
  NewWindowRuleName,
  /// A new project path, or a new root when true.
  ProjectPath(bool),
  /// The name and content typed for the next snippet.
  NewSnippetName,
  NewSnippetContent,
  /// Replacement content of a saved snippet, keyed by its name.
  SnippetContent(String),
  /// The name typed for the next SSH key pair.
  NewSshKeyName,
  /// The name, e-mail and comment typed for the next GPG key.
  GpgFullName,
  GpgEmail,
  GpgComment,
  /// The expiry date typed for the next GPG key.
  GpgExpiry,
  /// The passphrase and its confirmation for the next GPG key. Both are masked.
  GpgPassphrase,
  GpgPassphraseConfirm,
  /// Export path of one key, keyed by its fingerprint.
  GpgExportPath(String),
  /// File to import public keys from.
  GpgImportPath,
  /// File to write the private key of one fingerprint to.
  GpgExportPrivatePath(String),
  /// File to import a private key from.
  GpgImportPrivatePath,
  /// The expiry date typed on the edit page of a key.
  GpgEditExpiry,
}

/// Represents `Editor`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Editor {
  title: String,
  value: Vec<char>,
  cursor: usize,
  target: EditTarget,
  multiline: bool,
}

impl Editor {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub(crate) fn new(title: String, value: String, target: EditTarget, multiline: bool) -> Self {
    let value: Vec<char> = value.chars().collect();
    Self {
      title,
      cursor: value.len(),
      value,
      target,
      multiline,
    }
  }

  /// The text currently typed in the editor.
  #[cfg(test)]
  pub(crate) fn text(&self) -> String {
    self.value.iter().collect()
  }

  /// Executes the `input` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn input(&mut self, key: KeyEvent) {
    match key.code {
      KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
      KeyCode::Right => self.cursor = (self.cursor + 1).min(self.value.len()),
      KeyCode::Home => {
        self.cursor = self.value[..self.cursor]
          .iter()
          .rposition(|c| *c == '\n')
          .map(|i| i + 1)
          .unwrap_or(0)
      }
      KeyCode::End => {
        self.cursor += self.value[self.cursor..]
          .iter()
          .position(|c| *c == '\n')
          .unwrap_or(self.value.len() - self.cursor)
      }
      KeyCode::Up | KeyCode::Down => {
        let start = self.value[..self.cursor]
          .iter()
          .rposition(|c| *c == '\n')
          .map(|i| i + 1)
          .unwrap_or(0);
        let column = self.cursor - start;
        if key.code == KeyCode::Up && start > 0 {
          let previous = self.value[..start - 1]
            .iter()
            .rposition(|c| *c == '\n')
            .map(|i| i + 1)
            .unwrap_or(0);
          self.cursor = (previous + column).min(start - 1);
        } else if key.code == KeyCode::Down
          && let Some(end) = self.value[self.cursor..].iter().position(|c| *c == '\n')
        {
          let next = self.cursor + end + 1;
          let length = self.value[next..]
            .iter()
            .position(|c| *c == '\n')
            .unwrap_or(self.value.len() - next);
          self.cursor = next + column.min(length);
        }
      }
      KeyCode::Backspace if self.cursor > 0 => {
        self.cursor -= 1;
        self.value.remove(self.cursor);
      }
      KeyCode::Delete if self.cursor < self.value.len() => {
        self.value.remove(self.cursor);
      }
      KeyCode::Enter if self.multiline => {
        self.value.insert(self.cursor, '\n');
        self.cursor += 1;
      }
      KeyCode::Char(character)
        if !key
          .modifiers
          .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
          && !character.is_control()
          && self.value.len() < 262144 =>
      {
        self.value.insert(self.cursor, character);
        self.cursor += 1;
      }
      _ => {}
    }
  }

  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&self, frame: &mut Frame, area: Rect, app: &App) {
    let area = crate::ui::layout::centered(
      area,
      area.width.saturating_sub(6),
      if self.multiline {
        area.height.saturating_sub(4)
      } else {
        5
      },
    );
    frame.render_widget(Clear, area);
    let block = Block::bordered()
      .title(self.title.as_str())
      .title_bottom(tr(app.lang, "control_center.enter_ctrl_s_save_esc_cancel"))
      .style(
        Style::new()
          .fg(app.theme.foreground)
          .bg(app.theme.background),
      );
    let inner = block.inner(area);
    let secret = matches!(
      self.target,
      EditTarget::Password(_) | EditTarget::GpgPassphrase | EditTarget::GpgPassphraseConfirm
    );
    let text: String = self
      .value
      .iter()
      .map(|c| if secret { '*' } else { *c })
      .collect();
    let before: String = self.value[..self.cursor].iter().collect();
    let line = before.chars().filter(|c| *c == '\n').count();
    let current_line = before.rsplit('\n').next().unwrap_or("");
    let column = if secret {
      current_line.chars().count()
    } else {
      ratatui::text::Line::from(current_line).width()
    };
    let vertical = line.saturating_sub(inner.height.saturating_sub(1) as usize);
    let horizontal = column.saturating_sub(inner.width.saturating_sub(1) as usize);
    frame.render_widget(block, area);
    frame.render_widget(
      Paragraph::new(text).scroll((vertical as u16, horizontal as u16)),
      inner,
    );
    frame.set_cursor_position((
      inner.x + (column - horizontal) as u16,
      inner.y + (line - vertical) as u16,
    ));
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use argvus_tui::menu::RowKind;
  use crossterm::event::Event;

  /// Executes the `app` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn app(page: Page) -> App {
    let mut app = App::with_context(
      Page::Main,
      Lang::for_locale("en-US"),
      crate::theme::Theme::load(),
    );
    app.error_modal = None;
    app.navigation.push(page);
    app.admin.accounts = json!({"actor_uid":1000, "actor_is_admin":true, "shells":["/bin/bash", "/bin/zsh"], "groups":["users", "wheel", "audio"], "group_details":[
      {"name":"audio","gid":986,"members":["alice"]},
      {"name":"users","gid":1000,"members":[]},
      {"name":"wheel","gid":998,"members":["alice"]}
    ], "users":[
      {"user":"root", "uid":0, "gid":0, "name":"root", "shell":"/bin/bash", "primary_group":"root", "groups":[]},
      {"user":"alice", "uid":1000, "gid":1000, "name":"Alice", "shell":"/bin/bash", "primary_group":"users", "groups":["wheel"]}
    ]});
    app.admin.user = app.admin.accounts["users"][1].clone();
    app
  }

  fn press(app: &mut App, code: KeyCode) {
    crate::event::handle(app, Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
  }

  /// Moves the cursor to `item` and asserts that it is selectable.
  fn select(app: &mut App, item: Item) {
    let rows = app.rows();
    assert!(
      app.navigation.current_mut().menu.select(&rows, &item),
      "{item:?} is not selectable"
    );
  }

  fn items(app: &App) -> Vec<Option<Item>> {
    app.rows().iter().map(|row| row.id().copied()).collect()
  }

  #[test]
  /// Executes the `every_firewall_field_is_editable_without_writing` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn every_firewall_field_is_editable_without_writing() {
    let mut app = app(Page::Firewall);
    app.admin.firewall = json!({"active":false,"enabled":false});
    app.admin.config = json!({});
    for (index, (key, _)) in FIREWALL_FIELDS.iter().enumerate() {
      app.admin.config[*key] = json!(if *key == "PROTECTION_LEVEL" {
        "high"
      } else {
        ""
      });
      app.admin_activate(Item::FirewallField(index));
      if *key == "PROTECTION_LEVEL" {
        assert_eq!(app.admin.config[*key], "paranoid");
      } else {
        assert!(app.admin.editor.take().is_some());
      }
    }
    app.admin.config["ALLOW_SSH"] = json!("n");
    let ssh = FIREWALL_FIELDS
      .iter()
      .position(|(key, _)| *key == "ALLOW_SSH")
      .unwrap();
    app.admin_activate(Item::FirewallField(ssh));
    assert_eq!(app.admin.config["ALLOW_SSH"], "y");
    assert!(!app.admin.busy());
  }

  #[test]
  fn firewall_rows_are_toggles_values_and_a_protection_level_step() {
    let mut app = app(Page::Firewall);
    app.admin.firewall = json!({"active":true,"enabled":true});
    app.admin.config = json!({"ALLOW_SSH":"y","PROTECTION_LEVEL":"low","SSH_PORT":"22"});
    let rows = app.rows();
    let kind = |key: &str| {
      let index = FIREWALL_FIELDS.iter().position(|(k, _)| *k == key).unwrap();
      rows
        .iter()
        .find(|row| row.id() == Some(&Item::FirewallField(index)))
        .map(|row| row.kind())
        .unwrap()
    };
    assert_eq!(kind("ALLOW_SSH"), RowKind::Toggle { on: true });
    assert_eq!(kind("PROTECTION_LEVEL"), RowKind::Value { step: Some(1) });
    assert_eq!(kind("SSH_PORT"), RowKind::Value { step: None });
    let level = FIREWALL_FIELDS
      .iter()
      .position(|(key, _)| *key == "PROTECTION_LEVEL")
      .unwrap();
    select(&mut app, Item::FirewallField(level));
    press(&mut app, KeyCode::Left);
    assert_eq!(
      app.admin.config["PROTECTION_LEVEL"], "paranoid",
      "wraps around"
    );
    press(&mut app, KeyCode::Right);
    assert_eq!(app.admin.config["PROTECTION_LEVEL"], "low");
    assert_eq!(
      app.page(),
      Page::Firewall,
      "← adjusts instead of going back"
    );
  }

  #[test]
  /// Executes the `user_group_selection_preserves_primary_group` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn user_group_selection_preserves_primary_group() {
    let mut app = app(Page::UserGroups);
    let rows = app.rows();
    assert!(!rows[0].is_selectable(), "primary group row is disabled");
    assert_eq!(rows[0].kind(), RowKind::Toggle { on: true });
    app.admin_activate(Item::GroupOption(0));
    assert_eq!(strings(&app.admin.user["groups"]), vec!["wheel"]);
    app.normalize_selection();
    select(&mut app, Item::GroupOption(2));
    press(&mut app, KeyCode::Enter);
    assert_eq!(strings(&app.admin.user["groups"]), vec!["wheel", "audio"]);
    press(&mut app, KeyCode::Char(' '));
    assert_eq!(strings(&app.admin.user["groups"]), vec!["wheel"]);
  }

  #[test]
  /// Executes the `destructive_actions_default_to_cancel_and_never_run_on_selection` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn destructive_actions_default_to_cancel_and_never_run_on_selection() {
    let mut app = app(Page::User);
    // Save, Cancel, then Delete user.
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    assert!(
      app.confirm.is_none(),
      "moving the focus does not run anything"
    );
    press(&mut app, KeyCode::Enter);
    assert_eq!(
      app.page(),
      Page::UserDelete,
      "Delete asks Keep or Delete home first"
    );
    select(&mut app, Item::DeleteUserAndHome);
    press(&mut app, KeyCode::Enter);
    assert!(app.confirm.is_some());
    assert!(!app.confirm_focus.is_confirm_focused());
    assert!(!app.admin.busy());
    press(&mut app, KeyCode::Enter);
    assert!(app.confirm.is_none(), "Enter right after opening cancels");
    assert!(app.admin.pending.is_none());
    assert!(!app.admin.busy());
  }

  #[test]
  /// Executes the `password_mismatch_stays_local_and_passwords_are_masked` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn password_mismatch_stays_local_and_passwords_are_masked() {
    let mut app = app(Page::User);
    app.admin.passwords = ["old secret".into(), "new secret".into(), "different".into()];
    app.admin_activate(Item::SaveUser);
    assert!(app.error_modal.is_some());
    assert!(app.admin.pending.is_none());
    for row in app.rows() {
      assert!(!row.detail_text().unwrap_or_default().contains("secret"));
    }
  }

  #[test]
  /// Executes the `multiline_editor_moves_between_lines_and_accepts_paste` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn multiline_editor_moves_between_lines_and_accepts_paste() {
    let mut app = app(Page::Firewall);
    app.admin.editor = Some(Editor::new(
      "Rules".into(),
      "abc\nxy".into(),
      EditTarget::Rules,
      true,
    ));
    app.admin_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(app.admin.editor.as_ref().unwrap().cursor, 2);
    app.admin_paste("q?\niptables -A INPUT -j ACCEPT");
    assert!(
      app
        .admin
        .editor
        .as_ref()
        .unwrap()
        .value
        .iter()
        .collect::<String>()
        .contains("q?\niptables")
    );
    app.admin_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(app.admin.editor.is_none());
    assert!(app.admin.pending.is_none());
  }

  #[test]
  /// Executes the `system_users_are_optional_and_own_user_is_visible` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn system_users_are_optional_and_own_user_is_visible() {
    let app = app(Page::Users);
    assert_eq!(app.admin.users(false).len(), 1);
    assert_eq!(app.admin.users(true).len(), 2);
  }

  #[test]
  /// Executes the `users_overview_is_separate_from_account_list` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn users_overview_is_separate_from_account_list() {
    let mut app = app(Page::Users);
    let labels = app
      .rows()
      .iter()
      .map(|row| row.label().to_string())
      .collect::<Vec<_>>();
    assert_eq!(labels, vec!["Create", "List", "System accounts"]);
    select(&mut app, Item::UserList);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.page(), Page::UserList);
    assert_eq!(items(&app), vec![Some(Item::UserEntry(0))]);
    assert_eq!(app.rows()[0].label(), "alice");
    assert_eq!(app.selected_item(), Some(Item::UserEntry(0)));
  }

  #[test]
  fn r_reloads_the_account_lists() {
    let mut app = app(Page::SystemUsers);
    press(&mut app, KeyCode::Char('r'));
    assert!(app.admin.busy());
  }

  #[test]
  fn user_search_keeps_the_original_index() {
    let mut app = app(Page::SystemUsers);
    app.search = "ali".into();
    assert_eq!(items(&app), vec![Some(Item::UserEntry(1))]);
    app.admin_activate(Item::UserEntry(1));
    assert_eq!(app.page(), Page::User);
    assert_eq!(text(&app.admin.user, "user"), "alice");
  }

  #[test]
  /// Executes the `create_user_screen_has_masked_password_rows_and_routes_to_editor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn create_user_screen_has_masked_password_rows_and_routes_to_editor() {
    let mut app = app(Page::CreateUser);
    let rows = app.rows();
    let labels = rows.iter().map(|row| row.label()).collect::<Vec<_>>();
    // Same two blocks as the user page: User Data, then Actions.
    assert_eq!(
      labels,
      vec![
        tr(app.lang, "control_center.avatar_image"),
        tr(app.lang, "control_center.username"),
        tr(app.lang, "control_center.full_name"),
        tr(app.lang, "control_center.shell"),
        tr(app.lang, "control_center.home_directory"),
        tr(app.lang, "control_center.password"),
        tr(app.lang, "control_center.user_groups"),
        tr(app.lang, "control_center.make_admin"),
        tr(app.lang, "control_center.actions"),
        tr(app.lang, "control_center.create"),
        tr(app.lang, "control_center.cancel"),
      ]
    );
    assert!(rows[8].is_section());
    assert_eq!(app.selected_item(), Some(Item::AvatarImage));
    // The username has its own page; the password page only asks the new one.
    select(&mut app, Item::Username);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.page(), Page::UserUsername);
    press(&mut app, KeyCode::Esc);
    select(&mut app, Item::ChangePassword);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.page(), Page::UserPassword);
    select(&mut app, Item::Password(1));
    press(&mut app, KeyCode::Enter);
    let editor = app.admin.editor.take().expect("password row opens editor");
    assert_eq!(editor.target, EditTarget::Password(1));
    assert!(editor.value.is_empty());
  }

  #[test]
  /// Executes the `create_user_button_is_dynamic_and_validates_passwords` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn create_user_button_is_dynamic_and_validates_passwords() {
    let mut app = app(Page::CreateUser);
    app.admin.user["user"] = json!("carol");
    let pt = Lang::for_locale("pt-BR");
    let create_label = |app: &App| {
      app
        .admin
        .rows(Page::CreateUser, pt, "")
        .iter()
        .find(|row| row.id() == Some(&Item::CreateAccount))
        .map(|row| row.label().to_string())
        .unwrap()
    };
    assert_eq!(create_label(&app), tr(pt, "control_center.create"));
    app.admin.passwords[1] = "segredo".into();
    app.admin.passwords[2] = "segredo".into();
    assert_eq!(
      create_label(&app),
      tr(pt, "control_center.create"),
      "the label does not change"
    );
    app.admin_activate(Item::CreateAccount);
    assert!(
      app.admin.pending.is_some(),
      "valid passwords must reach the confirm flow"
    );
    let payload = app.admin.pending.as_ref().unwrap().1.clone();
    assert_eq!(payload["password"].as_str(), Some("segredo"));
    app.admin.pending = None;

    app.admin.passwords[1] = "a".into();
    app.admin.passwords[2] = "b".into();
    app.admin_activate(Item::CreateAccount);
    assert!(
      app.error_modal.is_some(),
      "mismatched passwords must be rejected"
    );
    assert!(app.admin.pending.is_none());
    app.error_modal = None;

    app.admin.passwords[1].clear();
    app.admin.passwords[2].clear();
    app.admin_activate(Item::CreateAccount);
    assert!(
      app.admin.pending.is_some(),
      "empty passwords keep the locked-account confirm flow"
    );
  }

  #[test]
  /// The user page follows the approved layout: titled sections, read-only
  /// identity as Info, the draft fields and their Save row, the password and
  /// avatar actions, and the deletions in the Danger zone at the end.
  fn user_page_layout_has_info_block_and_actions_block() {
    let mut app = app(Page::User);
    app.admin.user["name"] = json!("Alice Liddell");
    let rows = app.rows();
    let sections: Vec<&str> = rows
      .iter()
      .filter(|row| row.is_section())
      .map(|row| row.label())
      .collect();
    assert_eq!(sections, vec![tr(app.lang, "control_center.actions")]);
    assert!(rows.iter().all(|row| !row.label().starts_with("--")));
    let infos: Vec<&str> = rows
      .iter()
      .filter(|row| row.kind() == RowKind::Info)
      .map(|row| row.label())
      .collect();
    assert_eq!(
      infos,
      vec![
        "Username",
        tr(app.lang, "control_center.uid_gid"),
        tr(app.lang, "control_center.home_directory"),
      ]
    );
    let selectable: Vec<Item> = rows
      .iter()
      .filter(|row| row.is_selectable())
      .filter_map(|row| row.id().copied())
      .collect();
    assert_eq!(
      selectable,
      vec![
        Item::AvatarImage,
        Item::FullName,
        Item::Shell,
        Item::PrimaryGroup,
        Item::SupplementaryGroups,
        Item::UserAdmin,
        Item::ToggleAutoLogin,
        Item::ChangePassword,
        Item::SaveUser,
        Item::CancelUser,
        Item::DeleteUserMenu,
      ]
    );
    let kind = |item: Item| {
      rows
        .iter()
        .find(|row| row.id() == Some(&item))
        .map(|row| row.kind())
        .unwrap()
    };
    assert_eq!(kind(Item::AvatarImage), RowKind::Submenu);
    assert_eq!(kind(Item::FullName), RowKind::Submenu);
    assert_eq!(kind(Item::Shell), RowKind::Submenu);
    assert_eq!(kind(Item::CancelUser), RowKind::Action);
    assert_eq!(kind(Item::DeleteUserMenu), RowKind::Destructive);
    let save = rows
      .iter()
      .position(|row| row.id() == Some(&Item::SaveUser))
      .unwrap();
    assert_eq!(rows[save - 1].kind(), RowKind::Separator);
    assert!(rows[save - 2].is_section(), "Save starts the Actions block");
  }

  #[test]
  /// Retrieves data for `readonly_user_and_group_rows_are_not_selectable` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn readonly_user_and_group_rows_are_not_selectable() {
    let mut user_app = app(Page::User);
    assert_eq!(user_app.selected_item(), Some(Item::AvatarImage));
    press(&mut user_app, KeyCode::Up);
    assert_eq!(
      user_app.selected_item(),
      Some(Item::AvatarImage),
      "Info rows above are skipped"
    );

    let group_app = app(Page::Group);
    let rows = group_app.rows();
    assert!(!rows[0].is_selectable(), "section title");
    assert_eq!(rows[1].id(), Some(&Item::GroupName));
    assert_eq!(rows[2].kind(), RowKind::Info, "GID");
  }

  #[test]
  fn group_page_has_save_after_fields_and_delete_in_the_danger_zone() {
    let app = app(Page::Group);
    let rows = app.admin.rows(Page::Group, Lang::for_locale("pt-BR"), "");
    let ids: Vec<Option<Item>> = rows.iter().map(|row| row.id().copied()).collect();
    assert_eq!(
      ids,
      vec![
        None,
        Some(Item::GroupName),
        None,
        Some(Item::Members),
        None,
        Some(Item::SaveGroup),
        None,
        Some(Item::DeleteGroup),
      ]
    );
    assert_eq!(rows[5].label(), "Salvar alterações");
    assert_eq!(rows[7].kind(), RowKind::Destructive);
    assert!(rows[6].is_section());
  }

  #[test]
  /// Executes the `group_members_picker_toggles_users_in_list` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn group_members_picker_toggles_users_in_list() {
    let mut app = app(Page::GroupMembers);
    app.admin.accounts = json!({
      "actor_is_admin": true,
      "users": [
        {"user":"alice","uid":1000,"shell":"/bin/bash","groups":["users"],"password":"x","locked":false},
        {"user":"bob","uid":1001,"shell":"/bin/bash","groups":["users"],"password":"x","locked":false}
      ],
      "groups": [],
      "group_details": []
    });
    app.admin.group = json!({"name":"dev","gid":1000,"members":["alice"]});
    assert_eq!(strings(&app.admin.group["members"]), vec!["alice"]);
    assert_eq!(app.rows()[0].kind(), RowKind::Toggle { on: true });
    select(&mut app, Item::Member(1));
    press(&mut app, KeyCode::Enter);
    assert_eq!(strings(&app.admin.group["members"]), vec!["alice", "bob"]);
    press(&mut app, KeyCode::Enter);
    assert_eq!(strings(&app.admin.group["members"]), vec!["alice"]);
  }

  #[test]
  /// Executes the `group_open_keeps_original_name_for_rename_and_delete` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn group_open_keeps_original_name_for_rename_and_delete() {
    let mut app = app(Page::GroupList);
    app.admin.accounts = json!({
      "users": [],
      "groups": [{"name":"dev","gid":1000,"members":["alice"]}],
      "group_details": [{"name":"dev","gid":1000,"members":["alice"]}]
    });
    select(&mut app, Item::GroupEntry(0));
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.page(), Page::Group);
    assert_eq!(text(&app.admin.group, "original"), "dev");
  }

  #[test]
  fn firewall_actions_are_rows_in_their_sections() {
    let mut app = app(Page::Firewall);
    app.admin.firewall = json!({"active":false,"enabled":false});
    app.admin.config = json!({});
    let rows = app
      .admin
      .rows(Page::Firewall, Lang::for_locale("pt-BR"), "");
    let sections = rows.iter().filter(|row| row.is_section()).count();
    assert_eq!(sections, 3, "Service, Configuration and Rules");
    let position = |item: Item| rows.iter().position(|row| row.id() == Some(&item)).unwrap();
    assert!(
      position(Item::FirewallField(FIREWALL_FIELDS.len() - 1)) < position(Item::SaveFirewall)
    );
    assert_eq!(
      position(Item::SaveFirewall) + 1,
      position(Item::DiscardFirewall)
    );
    assert!(position(Item::DiscardFirewall) < position(Item::AddRules));
    assert_eq!(position(Item::AddRules) + 1, position(Item::ApplyRules));
    assert_eq!(
      rows[position(Item::SaveFirewall)].label(),
      "Salvar Configuração"
    );
    assert_eq!(
      rows[position(Item::FirewallBoot)].kind(),
      RowKind::Toggle { on: false }
    );
    assert!(rows.iter().all(|row| !row.label().contains("[ ")));
  }

  #[test]
  /// Executes the `firewall_buttons_route_to_confirm_editor_reload_and_cancel_like_before` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn firewall_rows_route_to_confirm_editor_reload_and_cancel_like_before() {
    let mut app = app(Page::Firewall);
    app.admin.firewall = json!({
      "active":false,
      "enabled":false,
      "config_text":"",
      "rules":"iptables -A INPUT -j DROP"
    });
    app.admin.config = json!({"ALLOW_SSH":"n"});
    app.admin_activate(Item::SaveFirewall);
    assert!(app.admin.pending.is_some());
    app.cancel_modal();
    app.admin_activate(Item::AddRules);
    assert_eq!(app.admin.editor.take().unwrap().target, EditTarget::Rules);
    app.admin_activate(Item::ApplyRules);
    assert!(app.admin.pending.is_some());
    let Some(PendingAction::Administration { message, .. }) = app.confirm.clone() else {
      panic!("Apply saved rules must ask first");
    };
    assert!(
      message.contains(tr(app.lang, "control_center.apply_saved_rules_description")),
      "the confirmation warns that the firewall restarts: {message}"
    );
    app.cancel_modal();
    app.admin_activate(Item::FirewallService);
    assert!(
      app.admin.pending.is_some(),
      "service state change is confirmed"
    );
    app.cancel_modal();
    app.admin_activate(Item::DiscardFirewall);
    assert!(
      app.admin.busy(),
      "Cancel changes reloads the saved configuration"
    );
  }

  #[test]
  /// Applies the `save_and_delete_buttons_confirm_without_running` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn save_and_delete_rows_confirm_without_running() {
    for (page, item) in [
      (Page::User, Item::SaveUser),
      (Page::User, Item::ToggleAutoLogin),
      (Page::UserDelete, Item::DeleteUser),
      (Page::UserDelete, Item::DeleteUserAndHome),
    ] {
      let mut app = app(page);
      app.admin.user["name"] = json!("Alice Liddell");
      app.admin_activate(item);
      assert!(app.confirm.is_some(), "{item:?} asks first");
      assert!(!app.admin.busy());
    }
  }

  #[test]
  fn toggle_autologin_confirms_the_opposite_of_the_current_state() {
    let mut app = app(Page::User);
    assert!(!app.admin.is_autologin_user("alice"));
    app.admin_activate(Item::ToggleAutoLogin);
    assert_eq!(
      app.admin.pending,
      Some((
        false,
        json!({"action":"auto-login", "user":"alice", "enabled":true})
      ))
    );
    app.cancel_modal();

    app.admin.accounts["autologin_user"] = json!("alice");
    assert!(app.admin.is_autologin_user("alice"));
    app.admin_activate(Item::ToggleAutoLogin);
    assert_eq!(
      app.admin.pending,
      Some((
        false,
        json!({"action":"auto-login", "user":"alice", "enabled":false})
      ))
    );
  }

  #[test]
  fn up_down_walk_the_info_list_and_left_right_walk_the_buttons() {
    let mut app = app(Page::User);
    assert_eq!(app.selected_item(), Some(Item::AvatarImage));
    for expected in [
      Item::FullName,
      Item::Shell,
      Item::PrimaryGroup,
      Item::SupplementaryGroups,
      Item::UserAdmin,
      Item::ToggleAutoLogin,
      Item::ChangePassword,
    ] {
      press(&mut app, KeyCode::Down);
      assert_eq!(app.selected_item(), Some(expected));
    }
    press(&mut app, KeyCode::Down);
    assert_eq!(
      app.selected_item(),
      Some(Item::ChangePassword),
      "the list ends at the info block"
    );
    assert_eq!(app.user_button_cursor(), None);

    // The bar is Save, Cancel, Delete: the first Right focuses Save.
    press(&mut app, KeyCode::Right);
    assert_eq!(app.user_button_cursor(), Some(0));
    press(&mut app, KeyCode::Right);
    assert_eq!(app.user_button_cursor(), Some(1), "Cancel");
    press(&mut app, KeyCode::Right);
    assert_eq!(app.user_button_cursor(), Some(2), "Delete user");
    press(&mut app, KeyCode::Right);
    assert_eq!(
      app.user_button_cursor(),
      Some(0),
      "Right wraps to the first button"
    );
    press(&mut app, KeyCode::Left);
    assert_eq!(app.user_button_cursor(), Some(2), "Left wraps back");

    // Up gives the focus back to the list and moves it one option up.
    press(&mut app, KeyCode::Up);
    assert_eq!(
      app.user_button_cursor(),
      None,
      "Up gives the focus back to the list"
    );
    assert_eq!(app.selected_item(), Some(Item::ToggleAutoLogin));
    press(&mut app, KeyCode::Tab);
    assert_eq!(
      app.user_button_cursor(),
      Some(0),
      "Tab moves to the buttons"
    );
    press(&mut app, KeyCode::Tab);
    assert_eq!(
      app.user_button_cursor(),
      None,
      "Tab again returns to the list"
    );
  }

  #[test]
  /// Cancel on the focused button discards the edits and stays on the user page.
  fn focused_cancel_button_discards_the_edits_and_stays() {
    let mut app = app(Page::User);
    app.admin.user["name"] = json!("Alice Liddell");
    // Save, then Cancel.
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.page(), Page::User, "Cancel keeps the user page open");
    assert!(!app.admin.user_draft_changed(), "the edits were discarded");
    assert!(app.confirm.is_none(), "Cancel does not ask");
  }

  #[test]
  /// Lock and Unlock are exclusive drafts and Expire is a checkbox; none run
  /// until Save, and Esc on the page asks before dropping them.
  fn password_page_security_options_are_drafts() {
    let mut app = app(Page::UserPassword);
    select(&mut app, Item::LockPassword);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.admin.lock_draft, Some(true));
    select(&mut app, Item::UnlockPassword);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.admin.lock_draft, Some(false), "Unlock replaces Lock");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.admin.lock_draft, None, "choosing it again clears it");
    select(&mut app, Item::ExpirePassword);
    press(&mut app, KeyCode::Enter);
    assert!(app.admin.expire_draft);
    assert!(
      app.confirm.is_none(),
      "nothing is confirmed or saved while editing"
    );
    assert!(
      app.admin.any_user_draft_changed(),
      "the footer warns about it"
    );
    press(&mut app, KeyCode::Esc);
    assert!(
      app.confirm.is_some(),
      "Esc asks before dropping the options"
    );
  }

  #[test]
  /// Edits made on the subpages are drafts: nothing runs until Save.
  fn subpage_edits_are_drafts_until_save() {
    let mut app = app(Page::UserAvatar);
    app.admin.editor = Some(Editor::new(
      "Avatar".into(),
      "/tmp/face.png".into(),
      EditTarget::Avatar,
      false,
    ));
    press(&mut app, KeyCode::Enter);
    assert!(
      app.confirm.is_none(),
      "the avatar is not saved while editing"
    );
    assert_eq!(
      text(&app.admin.user, "avatar"),
      "/tmp/face.png",
      "held in the draft"
    );
    assert!(!app.admin.busy());
  }

  #[test]
  /// Without administration rights the group pages are read-only: no row can run.
  fn group_writes_are_disabled_for_non_administrators() {
    let mut app = app(Page::Group);
    app.admin.accounts["actor_is_admin"] = json!(false);
    assert!(
      app.rows().iter().all(|row| !row.is_selectable()),
      "no group row is selectable"
    );
  }

  #[test]
  /// Covers the subpages of the user page: Full name and Password open their
  /// own page, and Ok of the password page goes back, keeping the change for Save.
  fn user_subpages_open_their_pages_and_ok_keeps_the_password_draft() {
    let mut app = app(Page::User);
    select(&mut app, Item::FullName);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.page(), Page::UserFullName);
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.page(), Page::User, "Esc cancels the Full name page");

    select(&mut app, Item::ChangePassword);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.page(), Page::UserPassword);
    app.admin.passwords = ["old".into(), "new".into(), "new".into()];
    select(&mut app, Item::OkPassword);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.page(), Page::User, "Ok returns to the user page");
    assert!(app.admin.passwords_changed(), "the password waits for Save");
  }

  #[test]
  fn save_changes_is_always_selectable_and_notes_the_draft() {
    let mut app = app(Page::User);
    let save = |app: &App| {
      app
        .rows()
        .into_iter()
        .find(|row| row.id() == Some(&Item::SaveUser))
        .unwrap()
    };
    assert!(save(&app).is_selectable(), "Save is always reachable");
    assert_eq!(save(&app).detail_text(), None, "no draft note while clean");
    app.admin.user["groups"] = json!(["wheel"]);
    assert!(!app.admin.user_draft_changed(), "same groups");
    app.admin.user["name"] = json!("Alice Liddell");
    assert!(save(&app).is_selectable());
    assert_eq!(
      save(&app).detail_text(),
      Some(tr(app.lang, "control_center.draft_changed"))
    );
  }

  #[test]
  fn esc_with_a_changed_user_draft_asks_and_discard_restores_it() {
    let mut app = app(Page::UserList);
    app.admin_activate(Item::UserEntry(0));
    assert_eq!(app.page(), Page::User);
    press(&mut app, KeyCode::Esc);
    assert_eq!(
      app.page(),
      Page::UserList,
      "clean draft leaves without asking"
    );

    app.admin_activate(Item::UserEntry(0));
    app.admin.user["name"] = json!("Changed");
    press(&mut app, KeyCode::Esc);
    assert!(matches!(app.confirm, Some(PendingAction::DiscardDraft)));
    assert!(
      !app.confirm_focus.is_confirm_focused(),
      "focus starts on Cancel"
    );
    press(&mut app, KeyCode::Enter);
    assert_eq!(
      app.page(),
      Page::User,
      "Cancel keeps the draft and the page"
    );
    assert_eq!(text(&app.admin.user, "name"), "Changed");

    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('y'));
    assert_eq!(app.page(), Page::UserList);
    assert_eq!(text(&app.admin.user, "name"), "Alice");
  }

  #[test]
  fn subpages_of_the_same_draft_do_not_ask() {
    let mut app = app(Page::User);
    app.admin_activate(Item::SupplementaryGroups);
    app.admin_activate(Item::GroupOption(2));
    assert!(app.admin.user_draft_changed());
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.page(), Page::User);
    assert!(app.confirm.is_none());
    app.admin_activate(Item::Shell);
    app.admin_activate(Item::ShellOption(1));
    assert_eq!(app.page(), Page::User, "choosing a shell goes back");
    assert_eq!(text(&app.admin.user, "shell"), "/bin/zsh");
  }

  #[test]
  fn reload_keeps_a_changed_user_draft() {
    let mut app = app(Page::User);
    app.admin.user["name"] = json!("Changed");
    let (sender, receiver) = mpsc::channel();
    app.admin.operation = "snapshot".into();
    app.admin.worker = Some(receiver);
    sender
      .send(Ok((false, app.admin.accounts.clone())))
      .unwrap();
    assert!(app.admin.poll().is_some_and(|result| result.is_ok()));
    assert_eq!(text(&app.admin.user, "name"), "Changed");

    app.admin.user["name"] = json!("Alice");
    let mut accounts = app.admin.accounts.clone();
    accounts["users"][1]["shell"] = json!("/bin/zsh");
    let (sender, receiver) = mpsc::channel();
    app.admin.worker = Some(receiver);
    sender.send(Ok((false, accounts))).unwrap();
    app.admin.poll();
    assert_eq!(
      text(&app.admin.user, "shell"),
      "/bin/zsh",
      "a clean draft follows the reload"
    );
  }

  #[test]
  fn group_firewall_create_and_password_drafts_ask_before_leaving() {
    let mut group = app(Page::GroupList);
    group.admin_activate(Item::GroupEntry(0));
    group.admin_activate(Item::Members);
    group.admin_activate(Item::Member(0));
    press(&mut group, KeyCode::Esc);
    assert_eq!(group.page(), Page::Group, "members share the group draft");
    assert!(group.confirm.is_none());
    press(&mut group, KeyCode::Esc);
    assert!(matches!(group.confirm, Some(PendingAction::DiscardDraft)));

    let mut firewall = app(Page::Firewall);
    firewall.admin.firewall = json!({"active":true,"enabled":true,"config":{"ALLOW_SSH":"y"}});
    firewall.admin.config = json!({"ALLOW_SSH":"y"});
    let save = |app: &App| {
      app
        .rows()
        .into_iter()
        .find(|row| row.id() == Some(&Item::SaveFirewall))
        .unwrap()
        .is_selectable()
    };
    assert!(!save(&firewall));
    let ssh = FIREWALL_FIELDS
      .iter()
      .position(|(key, _)| *key == "ALLOW_SSH")
      .unwrap();
    firewall.admin_activate(Item::FirewallField(ssh));
    assert!(save(&firewall));
    press(&mut firewall, KeyCode::Esc);
    assert!(matches!(
      firewall.confirm,
      Some(PendingAction::DiscardDraft)
    ));
    press(&mut firewall, KeyCode::Char('y'));
    assert_eq!(firewall.admin.config["ALLOW_SSH"], "y", "discard restores");

    let mut create = app(Page::Users);
    create.admin_activate(Item::CreateUser);
    let create_row = |app: &App| {
      app
        .rows()
        .into_iter()
        .find(|row| row.id() == Some(&Item::CreateAccount))
        .unwrap()
        .is_selectable()
    };
    assert!(
      create_row(&create),
      "Create is always reachable; an empty name is refused"
    );
    press(&mut create, KeyCode::Esc);
    assert_eq!(create.page(), Page::Users);
    create.admin_activate(Item::CreateUser);
    create.admin.user["user"] = json!("bob");
    assert!(create_row(&create));
    press(&mut create, KeyCode::Esc);
    assert!(matches!(create.confirm, Some(PendingAction::DiscardDraft)));

    let mut password = app(Page::User);
    password.admin_activate(Item::ChangePassword);
    password.admin.passwords[1] = "secret".into();
    press(&mut password, KeyCode::Esc);
    assert!(matches!(
      password.confirm,
      Some(PendingAction::DiscardDraft)
    ));
    press(&mut password, KeyCode::Char('y'));
    assert_eq!(password.page(), Page::User);
    assert!(!password.admin.passwords_changed());
  }

  #[test]
  fn confirmation_uses_the_single_component_keys() {
    let mut app = app(Page::User);
    app.admin.user["name"] = json!("Changed");
    press(&mut app, KeyCode::Esc);
    assert!(app.confirm.is_some());
    press(&mut app, KeyCode::Char('n'));
    assert!(app.confirm.is_none(), "n cancels");
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Down);
    assert!(
      !app.confirm_focus.is_confirm_focused(),
      "Cancel is the lower row"
    );
    press(&mut app, KeyCode::Char('k'));
    assert!(app.confirm_focus.is_confirm_focused());
    press(&mut app, KeyCode::Tab);
    assert!(!app.confirm_focus.is_confirm_focused());
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Enter);
    assert!(app.confirm.is_none());
    assert_eq!(
      app.page(),
      Page::Main,
      "Enter on Confirm discards and leaves"
    );
    assert_eq!(text(&app.admin.user, "name"), "Alice");
  }

  #[test]
  fn deletions_use_the_danger_style_and_other_actions_do_not() {
    let mut user = app(Page::User);
    for (item, danger) in [(Item::DeleteUser, true), (Item::DeleteUserAndHome, true)] {
      user.admin_activate(item);
      let (_, message, confirm, is_danger) = user.confirm_dialog().unwrap();
      assert_eq!(is_danger, danger, "{item:?}");
      if danger {
        assert!(message.contains("irreversible"), "{message}");
        assert_eq!(confirm, tr(user.lang, "control_center.yes"));
      } else {
        assert!(message.ends_with("alice?"), "{message}");
      }
      user.cancel_modal();
    }
    let mut group = app(Page::Group);
    group.admin.group = json!({"name":"wheel","original":"wheel","gid":998,"members":[]});
    group.admin_activate(Item::DeleteGroup);
    assert!(group.confirm_dialog().unwrap().3);
  }

  #[test]
  fn confirmation_renders_vertical_rows_without_button_labels() {
    use ratatui::{Terminal, backend::TestBackend};
    let mut app = app(Page::User);
    app.admin_activate(Item::DeleteUser);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
      .draw(|frame| crate::ui::draw(&mut app, frame))
      .unwrap();
    let rendered: String = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect();
    assert!(rendered.contains(tr(app.lang, "control_center.yes")));
    assert!(rendered.contains(tr(app.lang, "control_center.no")));
    assert!(rendered.contains("irreversible"));
    assert!(!rendered.contains(tr(app.lang, "control_center.cancel_f8378f")));
  }

  #[test]
  /// Executes the `pages_and_password_editor_render_at_minimum_and_large_sizes` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn pages_and_password_editor_render_at_minimum_and_large_sizes() {
    use ratatui::{Terminal, backend::TestBackend};
    for (width, height) in [(60, 15), (120, 40)] {
      for page in [
        Page::Users,
        Page::UserList,
        Page::SystemUsers,
        Page::User,
        Page::CreateUser,
        Page::UserGroups,
        Page::UserShell,
        Page::UserPrimaryGroup,
        Page::UserPassword,
        Page::UserAvatar,
        Page::UserFullName,
        Page::UserDelete,
        Page::Groups,
        Page::GroupList,
        Page::SystemGroups,
        Page::Group,
        Page::CreateGroup,
      ] {
        let mut app = app(page);
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
          .draw(|frame| crate::ui::draw(&mut app, frame))
          .unwrap();
        let rendered: String = terminal
          .backend()
          .buffer()
          .content
          .iter()
          .map(|cell| cell.symbol())
          .collect();
        assert!(rendered.chars().any(|cell| cell != ' '));
        // The user page draws its action buttons, so only the other pages
        // must not show bracketed button labels.
        if !matches!(page, Page::User | Page::CreateUser) {
          assert!(
            !rendered.replace("[ ]", "").contains("[ "),
            "{page:?} has no button labels"
          );
        }
        if page == Page::User && height == 40 {
          assert!(rendered.contains("Delete"));
          assert!(rendered.contains("Actions"));
          assert!(!rendered.contains("--"));
        }
        app.admin.editor = Some(Editor::new(
          "Password".into(),
          "private-password".into(),
          EditTarget::Password(1),
          false,
        ));
        terminal
          .draw(|frame| crate::ui::draw(&mut app, frame))
          .unwrap();
        let rendered: String = terminal
          .backend()
          .buffer()
          .content
          .iter()
          .map(|cell| cell.symbol())
          .collect();
        assert!(!rendered.contains("private-password"));
        assert!(rendered.contains("****************"));
      }
    }
  }
}
