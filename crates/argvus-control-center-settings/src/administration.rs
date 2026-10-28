//! User and group administration of the Settings pages.
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

mod activation;
mod drafts;
mod editor;
mod input;
mod rows;

pub(crate) use editor::EditTarget;
pub use editor::Editor;

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

fn text(value: &Value, key: &str) -> String {
  value[key].as_str().unwrap_or_default().into()
}

/// Groups that grant administration, as in `argvus-accounts-core` (`ADMIN_GROUPS`).
const ADMIN_GROUPS: [&str; 2] = ["wheel", "sudo"];

/// Whether `groups` contains an administrator group.
fn has_admin_group(groups: &[String]) -> bool {
  groups
    .iter()
    .any(|group| ADMIN_GROUPS.contains(&group.as_str()))
}

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

fn request(firewall: bool, value: Value, privileged: bool) -> Result<Value, String> {
  let binary = if firewall {
    "/usr/lib/argvus/commands/firewall"
  } else {
    "/usr/lib/argvus/commands/accounts"
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

  pub fn load(&mut self, firewall: bool) {
    if self.busy() {
      return;
    }
    self.start(firewall, json!({"action":"snapshot"}), false);
  }

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

  pub fn submit(&mut self) {
    if let Some((firewall, value)) = self.pending.take() {
      self.start(firewall, value, true);
    }
  }

  pub fn is_loaded(&self) -> bool {
    self.accounts["users"].is_array()
  }

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
}

#[cfg(test)]
mod tests;
