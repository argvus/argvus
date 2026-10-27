//! The connection editor: one draft of a profile, shown as a single list of
//! sections (General, the kind's own settings, IPv4, IPv6), with list
//! fields, ports and WireGuard peers on sub-pages that share the draft.
//!
//! Nothing reaches NetworkManager until Save. Esc with pending changes asks
//! before dropping them. Saving a profile that is active asks first, since it
//! is brought up again with the new settings.

use super::{Item, NetworkApp};
use crate::{
  backend::{NetworkBackend, NetworkError, SaveOutcome},
  model::{ConnectionProfile, NetworkPage},
  profile::{
    ConnectionDraft, ConnectionKind, DraftError, PeerField, SecretState, WireGuardPeer,
    schema::{Choice, ListRule, Property, SectionId, Widget},
    validate::{self, split_list},
  },
};
use argvus_control_center_core::process::SystemProcessRunner;
use argvus_i18n::{Lang, tr};
use argvus_tui::{
  components::{StatusKind, StatusMessage},
  confirm::ConfirmState,
  icons,
  menu::{Row, draft_actions},
  text_editor::TextEditor,
};
use std::collections::BTreeSet;

/// Glyphs shown for a secret typed in this editor.
const MASKED: &str = "••••••••";

/// The profile being edited and how its editor is laid out.
pub(super) struct EditorState {
  pub(super) draft: ConnectionDraft,
  /// Sections folded by the user.
  pub(super) collapsed: BTreeSet<SectionId>,
  /// Public key of the WireGuard private key, when known.
  pub(super) public_key: Option<String>,
}

impl EditorState {
  fn new(draft: ConnectionDraft) -> Self {
    Self {
      draft,
      collapsed: BTreeSet::new(),
      public_key: None,
    }
  }
}

/// What a text editor opened from the connection pages writes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditTarget {
  Field(&'static str),
  /// An entry of a list field; `None` adds one.
  ListEntry(&'static str, Option<usize>),
  PeerField(usize, PeerField),
  /// The file of a VPN import: a plugin index, or `None` for WireGuard.
  ImportPath(Option<usize>),
}

/// A confirmation the editor is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditorConfirm {
  Discard,
  SaveActive,
  Delete,
}

/// How the profile to open is found.
pub(super) enum ProfileSelector {
  Uuid(String),
  /// A profile by name, preferring the one on `device`.
  Name {
    name: String,
    device: Option<String>,
  },
}

/// Result of a background editor task.
pub(super) enum EditorResult {
  Loaded {
    draft: Box<ConnectionDraft>,
    profiles: Vec<ConnectionProfile>,
    message: Option<String>,
  },
  Saved {
    draft: Box<ConnectionDraft>,
    profiles: Vec<ConnectionProfile>,
    outcome: SaveOutcome,
  },
  Deleted,
  KeyPair {
    private: String,
    public: String,
  },
  PublicKey(String),
}

/// The text of a backend error, in the user's language when it has one.
pub(super) fn error_message(lang: Lang, error: &NetworkError) -> String {
  match error {
    NetworkError::NotAuthorized => tr(lang, "control_center.net_e_not_authorized").to_owned(),
    NetworkError::BackendUnavailable => {
      tr(lang, "control_center.net_e_no_networkmanager").to_owned()
    }
    other => other.to_string(),
  }
}

/// The label of a choice's current value.
fn choice_label(lang: Lang, options: &[Choice], value: &str) -> String {
  options
    .iter()
    .find(|choice| choice.value == value)
    .map_or_else(|| dash(value), |choice| tr(lang, choice.label).to_owned())
}

fn dash(value: &str) -> String {
  if value.is_empty() {
    "—".to_owned()
  } else {
    value.to_owned()
  }
}

/// A list value in one line: its first entry and how many follow.
fn list_summary(value: &str) -> String {
  let entries = split_list(value);
  match entries.as_slice() {
    [] => "—".to_owned(),
    [only] => only.clone(),
    [first, rest @ ..] => format!("{first}  +{}", rest.len()),
  }
}

/// An example entry of a list, shown under it.
fn list_example(rule: ListRule) -> &'static str {
  use crate::profile::schema::Family;
  match rule {
    ListRule::Cidr(Family::V6) => "2001:db8::10/64",
    ListRule::Cidr(_) => "192.168.1.10/24",
    ListRule::Address(Family::V6) => "2606:4700:4700::1111",
    ListRule::Address(_) => "1.1.1.1",
    ListRule::Domain => "example.org",
    ListRule::Route(Family::V6) => "2001:db8:1::/48 2001:db8::1 100",
    ListRule::Route(_) => "10.0.0.0/8 192.168.1.1 100",
    ListRule::KeyValue => "key=value",
  }
}

/// The start of a WireGuard key, enough to tell peers apart.
fn short_key(key: &str) -> String {
  if key.chars().count() > 12 {
    format!("{}…", key.chars().take(12).collect::<String>())
  } else {
    key.to_owned()
  }
}

impl NetworkApp {
  /// Opens the editor on a new profile of `kind`.
  pub(super) fn open_new_profile(&mut self, kind: ConnectionKind, service: Option<String>) {
    let name = self.unused_profile_name(kind, service.as_deref());
    let draft = ConnectionDraft::new(kind, &name, service, &self.owner);
    self.editor_return = self.page;
    self.editor = Some(EditorState::new(draft));
    self.go_to(NetworkPage::Editor);
  }

  /// `<kind> N` with the first N no profile uses.
  fn unused_profile_name(&self, kind: ConnectionKind, service: Option<&str>) -> String {
    let base = match service.and_then(|service| self.vpn_plugin_label(service)) {
      Some(label) => label,
      None => tr(self.lang, kind.label_key()).to_owned(),
    };
    (1..)
      .map(|number| format!("{base} {number}"))
      .find(|name| {
        !self
          .snapshot
          .profiles
          .iter()
          .any(|profile| &profile.name == name)
      })
      .unwrap_or(base)
  }

  /// Opens the editor on a saved profile, loaded in the background.
  pub(super) fn open_profile(&mut self, selector: ProfileSelector) {
    if self.editor_job.is_some() {
      return;
    }
    self.editor_return = self.page;
    self.editor = None;
    self.go_to(NetworkPage::Editor);
    let owner = self.owner.clone();
    self.spawn_editor_job("control_center.loading", move |backend| {
      let profiles = backend.list_profiles()?;
      let uuid = match selector {
        ProfileSelector::Uuid(uuid) => uuid,
        ProfileSelector::Name { name, device } => profiles
          .iter()
          .filter(|profile| profile.name == name)
          .max_by_key(|profile| device.is_some() && profile.device == device)
          .map(|profile| profile.uuid.clone())
          .ok_or(NetworkError::InvalidInput)?,
      };
      let draft = backend.load_profile(&uuid, &profiles, &owner)?;
      Ok(EditorResult::Loaded {
        draft: Box::new(draft),
        profiles,
        message: None,
      })
    });
  }

  /// Moves to `page` with the cursor on its first row.
  pub(super) fn go_to(&mut self, page: NetworkPage) {
    self.page = page;
    self.menu = argvus_tui::menu::MenuState::default();
  }

  /// Runs an editor task in the background. `progress` is the translation
  /// key shown while it runs.
  pub(super) fn spawn_editor_job<F>(&mut self, progress: &'static str, task: F)
  where
    F: FnOnce(NetworkBackend<SystemProcessRunner>) -> Result<EditorResult, NetworkError>
      + Send
      + 'static,
  {
    if self.editor_job.is_some() {
      return;
    }
    self.cancel_refresh_for_action();
    let capabilities = self.capabilities.clone();
    let lang = self.lang;
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: format!("{}...", tr(lang, progress).trim_end_matches('.')),
    });
    self.editor_job = Some(self.jobs.spawn(move |_| {
      Ok(
        task(NetworkBackend::new(SystemProcessRunner, capabilities))
          .map_err(|error| error_message(lang, &error)),
      )
    }));
  }

  /// Applies a finished editor task. Returns whether anything changed.
  pub(super) fn poll_editor(&mut self) -> bool {
    use argvus_control_center_core::jobs::JobState;
    let Some(job) = &self.editor_job else {
      return false;
    };
    let JobState::Finished(result) = job.try_state() else {
      return false;
    };
    self.editor_job = None;
    let result = match result {
      Ok(result) => result,
      Err(error) => Err(error),
    };
    match result {
      Ok(EditorResult::Loaded {
        draft,
        profiles,
        message,
      }) => {
        self.snapshot.profiles = profiles;
        if self.page.is_editor() {
          self.editor = Some(EditorState::new(*draft));
        }
        self.status = message.map(|text| StatusMessage {
          kind: StatusKind::Success,
          text,
        });
      }
      Ok(EditorResult::Saved {
        draft,
        profiles,
        outcome,
      }) => {
        self.snapshot.profiles = profiles;
        let mut state = EditorState::new(*draft);
        if let Some(previous) = self.editor.take() {
          state.collapsed = previous.collapsed;
          state.public_key = previous.public_key;
        }
        self.editor = Some(state);
        let key = if outcome.reactivated {
          "control_center.net_saved_reapplied"
        } else {
          "control_center.net_saved"
        };
        self.status = Some(StatusMessage {
          kind: StatusKind::Success,
          text: tr(self.lang, key).into(),
        });
      }
      Ok(EditorResult::Deleted) => {
        self.close_editor();
        self.status = Some(StatusMessage {
          kind: StatusKind::Success,
          text: tr(self.lang, "control_center.net_deleted").into(),
        });
      }
      Ok(EditorResult::KeyPair { private, public }) => {
        if let Some(state) = &mut self.editor {
          state.draft.set_secret("wireguard.private-key", private);
          state.public_key = Some(public);
        }
        self.status = None;
      }
      Ok(EditorResult::PublicKey(public)) => {
        if let Some(state) = &mut self.editor {
          state.public_key = Some(public);
        }
        self.status = None;
      }
      Err(error) => {
        // A profile that never loaded leaves nothing to edit.
        if self.editor.is_none() && self.page.is_editor() {
          self.go_to(self.editor_return);
        }
        self.status = Some(StatusMessage {
          kind: StatusKind::Error,
          text: error,
        });
      }
    }
    true
  }

  /// Leaves the editor for the page that opened it.
  pub(super) fn close_editor(&mut self) {
    if let Some(job) = self.editor_job.take() {
      job.cancel();
    }
    self.editor = None;
    self.text_edit = None;
    self.editor_confirm = None;
    let back_to = if self.editor_return.is_editor() {
      NetworkPage::Connections
    } else {
      self.editor_return
    };
    self.go_to(back_to);
    let rows = self.rows();
    self.menu.normalize(&rows);
    self.start_refresh(false);
  }

  /// Esc on an editor page: sub-pages return to their parent; the editor
  /// itself asks before dropping pending changes.
  pub(super) fn editor_back(&mut self) {
    let (parent, opener) = match self.page {
      NetworkPage::EditorList(key) => (NetworkPage::Editor, Item::Field(key)),
      NetworkPage::EditorPorts => (NetworkPage::Editor, Item::OpenPorts),
      NetworkPage::EditorPeers => (NetworkPage::Editor, Item::OpenPeers),
      NetworkPage::EditorPeer(index) => (NetworkPage::EditorPeers, Item::Peer(index)),
      _ => {
        let dirty = self
          .editor
          .as_ref()
          .is_some_and(|state| state.draft.is_dirty() && self.editor_job.is_none());
        if dirty {
          self.editor_confirm = Some((EditorConfirm::Discard, ConfirmState::new()));
        } else {
          self.close_editor();
        }
        return;
      }
    };
    self.go_to(parent);
    let rows = self.rows();
    self.menu.select(&rows, &opener);
  }

  /// The devices a device field can cycle through.
  pub(super) fn device_candidates(&self, key: &str) -> Vec<String> {
    let Some(state) = &self.editor else {
      return Vec::new();
    };
    let draft = &state.draft;
    let types: &[&str] = match key {
      "connection.interface-name" if draft.is_port() => &["ethernet"],
      "connection.interface-name" => draft.kind.device_types(),
      _ => &[],
    };
    self
      .snapshot
      .interfaces
      .iter()
      .filter(|interface| {
        let kind = interface.kind.to_ascii_lowercase();
        if key == "connection.interface-name" {
          types.iter().any(|wanted| kind == *wanted)
        } else {
          // A parent: any real or virtual link but the loopback.
          !matches!(kind.as_str(), "loopback" | "wifi-p2p")
        }
      })
      .map(|interface| interface.name.clone())
      .collect()
  }

  /// Ethernet devices that can join the controller being edited, plus the
  /// ports it already has.
  pub(super) fn port_candidates(&self) -> Vec<String> {
    let Some(state) = &self.editor else {
      return Vec::new();
    };
    let own = state.draft.value("connection.interface-name");
    let mut devices = self
      .snapshot
      .interfaces
      .iter()
      .filter(|interface| interface.kind.eq_ignore_ascii_case("ethernet") && interface.name != own)
      .map(|interface| interface.name.clone())
      .collect::<Vec<_>>();
    for port in &state.draft.ports {
      if !devices.contains(port) {
        devices.push(port.clone());
      }
    }
    devices
  }

  /// The listed profile the editor shows, when it is saved.
  fn edited_profile(&self) -> Option<&ConnectionProfile> {
    let uuid = self.editor.as_ref()?.draft.uuid.as_deref()?;
    self
      .snapshot
      .profiles
      .iter()
      .find(|profile| profile.uuid == uuid)
  }

  /// Rows of the editor page.
  pub(super) fn editor_rows(&self) -> Vec<Row<Item>> {
    let Some(state) = &self.editor else {
      return vec![Row::info(tr(self.lang, "control_center.loading"), "")];
    };
    let draft = &state.draft;
    let mut rows = Vec::new();
    for section in draft.sections() {
      let collapsed = state.collapsed.contains(&section.id);
      rows.push(Row::collapsible_section(
        Item::Section(section.id),
        tr(self.lang, section.title),
        collapsed,
      ));
      if collapsed {
        continue;
      }
      for property in section.properties {
        if draft.is_shown(property) {
          rows.push(self.field_row(draft, property));
        }
      }
      rows.extend(self.section_extras(state, section.id));
    }
    if draft.kind == ConnectionKind::Other {
      rows.push(Row::info(
        tr(self.lang, "control_center.net_other_kind_note"),
        "",
      ));
    }
    let read_only = self
      .edited_profile()
      .is_some_and(|profile| profile.read_only);
    let error = draft.first_error();
    let dirty = draft.is_dirty();
    let mut save = draft_actions(
      Item::Save,
      tr(self.lang, "control_center.net_save"),
      dirty && error.is_none() && !read_only,
    );
    let save_detail = match error {
      _ if read_only => tr(self.lang, "control_center.net_read_only").to_owned(),
      Some(error) => self.draft_error_text(draft, error),
      None if dirty => tr(self.lang, "control_center.draft_changed").to_owned(),
      None => String::new(),
    };
    if let Some(row) = save.pop() {
      save.push(row.detail(save_detail));
    }
    rows.extend(save);
    rows
      .push(Row::action(Item::Cancel, tr(self.lang, "control_center.cancel")).icon(icons::CANCEL));
    if let Some(profile) = self.edited_profile() {
      let (label, icon) = if profile.active {
        ("control_center.disconnect", icons::LINK_OFF)
      } else {
        ("control_center.connect", icons::LINK_ON)
      };
      rows.push(Row::section(tr(
        self.lang,
        "control_center.net_section_connection",
      )));
      rows.push(
        Row::action(Item::ToggleActive, tr(self.lang, label))
          .icon(icon)
          .detail(self.profile_status(profile)),
      );
      rows.push(Row::section(tr(self.lang, "control_center.danger_zone")));
      rows.push(
        Row::destructive(
          Item::Delete,
          tr(self.lang, "control_center.net_delete_connection"),
        )
        .icon(icons::DELETE)
        .enabled(!profile.read_only),
      );
    }
    rows
  }

  /// `★ Active · eno1`, `Inactive`, or the port it is.
  pub(super) fn profile_status(&self, profile: &ConnectionProfile) -> String {
    if profile.active {
      let device = profile
        .device
        .as_deref()
        .map(|device| format!(" · {device}"))
        .unwrap_or_default();
      format!(
        "★ {}{device}",
        tr(self.lang, "control_center.active_c7cc67")
      )
    } else {
      tr(self.lang, "control_center.inactive_6f56ef").to_owned()
    }
  }

  /// What keeps the draft from being saved, with the field it is about.
  fn draft_error_text(&self, draft: &ConnectionDraft, error: DraftError) -> String {
    let message = tr(self.lang, error.error().message_key());
    let field = match error {
      DraftError::Field(key, _) => draft
        .property(key)
        .map(|property| tr(self.lang, property.label).to_owned())
        .unwrap_or_default(),
      DraftError::Peer(index, field, _) => format!(
        "{} {} · {}",
        tr(self.lang, "control_center.net_peer"),
        index + 1,
        tr(self.lang, field.label_key())
      ),
    };
    format!("{field}: {message}")
  }

  /// One row for a field, edited as its widget says.
  fn field_row(&self, draft: &ConnectionDraft, property: &'static Property) -> Row<Item> {
    let key = property.key;
    let label = tr(self.lang, property.label);
    let value = draft.value(key);
    let item = Item::Field(key);
    let row = match property.widget {
      Widget::Toggle => Row::toggle(item, label, value == "yes"),
      Widget::Choice(options) => Row::value(
        item,
        label,
        choice_label(self.lang, options, value),
        Some(1),
      ),
      Widget::Number { .. } => Row::value(item, label, dash(value), Some(1)),
      Widget::Device { .. } => {
        let shown = if value.is_empty() {
          tr(self.lang, "control_center.net_any_device").to_owned()
        } else {
          value.to_owned()
        };
        let step = (!self.device_candidates(key).is_empty()).then_some(1);
        Row::value(item, label, shown, step)
      }
      Widget::Text(_) => Row::value(item, label, dash(value), None),
      Widget::Secret(_) => Row::value(item, label, self.secret_text(draft, key), None),
      Widget::List(_) => Row::submenu(item, label).detail(list_summary(value)),
    };
    row.enabled(draft.is_enabled(property))
  }

  /// How a secret reads: never its value.
  fn secret_text(&self, draft: &ConnectionDraft, key: &str) -> String {
    match (draft.secret_state(key), draft.secret(key)) {
      (SecretState::Changed, Some("")) => tr(self.lang, "control_center.net_secret_cleared").into(),
      (SecretState::Changed, _) => MASKED.to_owned(),
      (SecretState::Unchanged, _) if draft.is_new() => "—".to_owned(),
      (SecretState::Unchanged, _) => tr(self.lang, "control_center.net_secret_unchanged").into(),
    }
  }

  /// Rows a section adds after its fields: ports, peers, key generation.
  fn section_extras(&self, state: &EditorState, section: SectionId) -> Vec<Row<Item>> {
    let draft = &state.draft;
    match section {
      SectionId::Bridge | SectionId::Bond | SectionId::Team if !draft.is_port() => {
        let ports = draft.ports.iter().cloned().collect::<Vec<_>>();
        vec![
          Row::submenu(Item::OpenPorts, tr(self.lang, "control_center.net_f_ports"))
            .icon(icons::PORT)
            .detail(if ports.is_empty() {
              "—".to_owned()
            } else {
              ports.join(", ")
            }),
        ]
      }
      SectionId::WireGuard => {
        let mut rows = vec![
          Row::submenu(Item::OpenPeers, tr(self.lang, "control_center.net_f_peers"))
            .icon(icons::WIREGUARD)
            .detail(draft.peers.len().to_string()),
        ];
        let generate = Row::action(
          Item::GenerateKey,
          tr(self.lang, "control_center.net_generate_key"),
        )
        .icon(icons::KEY_GENERATE);
        rows.push(if self.capabilities.has_wireguard_tools {
          generate
        } else {
          generate
            .enabled(false)
            .detail(tr(self.lang, "control_center.net_requires_wireguard_tools"))
        });
        if let Some(public) = &state.public_key {
          rows.push(Row::info(
            tr(self.lang, "control_center.net_f_public_key"),
            public.clone(),
          ));
        }
        rows
      }
      _ => Vec::new(),
    }
  }

  /// Rows of a list field's page.
  pub(super) fn editor_list_rows(&self, key: &'static str) -> Vec<Row<Item>> {
    let Some(state) = &self.editor else {
      return Vec::new();
    };
    let Some(property) = state.draft.property(key) else {
      return Vec::new();
    };
    let Widget::List(rule) = property.widget else {
      return Vec::new();
    };
    let mut rows = vec![Row::section(tr(self.lang, property.label))];
    rows.extend(
      split_list(state.draft.value(key))
        .into_iter()
        .enumerate()
        .map(|(index, entry)| Row::value(Item::ListEntry(index), entry, String::new(), None)),
    );
    rows.push(
      Row::action(
        Item::AddListEntry,
        tr(self.lang, "control_center.net_add_entry"),
      )
      .icon(icons::ADD),
    );
    rows.push(Row::section(tr(
      self.lang,
      "control_center.section_summary",
    )));
    rows.push(Row::info(
      tr(self.lang, "control_center.net_format"),
      list_example(rule),
    ));
    rows.push(Row::info(tr(self.lang, "control_center.net_list_hint"), ""));
    rows
  }

  /// Rows of the ports page: one switch per Ethernet device.
  pub(super) fn editor_port_rows(&self) -> Vec<Row<Item>> {
    let Some(state) = &self.editor else {
      return Vec::new();
    };
    let mut rows = vec![Row::section(tr(self.lang, "control_center.net_f_ports"))];
    let candidates = self.port_candidates();
    if candidates.is_empty() {
      rows.push(Row::info(
        tr(self.lang, "control_center.net_no_ethernet_devices"),
        "",
      ));
    }
    rows.extend(candidates.iter().enumerate().map(|(index, device)| {
      let connection = self
        .snapshot
        .interfaces
        .iter()
        .find(|interface| &interface.name == device)
        .and_then(|interface| interface.connection.clone())
        .unwrap_or_default();
      Row::toggle(
        Item::Port(index),
        device.clone(),
        state.draft.ports.contains(device),
      )
      .detail(connection)
    }));
    rows
  }

  /// Rows of the WireGuard peers page.
  pub(super) fn editor_peer_rows(&self) -> Vec<Row<Item>> {
    let Some(state) = &self.editor else {
      return Vec::new();
    };
    let mut rows = vec![Row::section(tr(self.lang, "control_center.net_f_peers"))];
    rows.extend(state.draft.peers.iter().enumerate().map(|(index, peer)| {
      let label = if peer.public_key.is_empty() {
        format!("{} {}", tr(self.lang, "control_center.net_peer"), index + 1)
      } else {
        short_key(&peer.public_key)
      };
      Row::submenu(Item::Peer(index), label).detail(dash(&peer.endpoint))
    }));
    rows.push(
      Row::action(Item::AddPeer, tr(self.lang, "control_center.net_add_peer")).icon(icons::ADD),
    );
    rows
  }

  /// Rows of one peer's page.
  pub(super) fn editor_single_peer_rows(&self, index: usize) -> Vec<Row<Item>> {
    let Some(peer) = self
      .editor
      .as_ref()
      .and_then(|state| state.draft.peers.get(index))
    else {
      return Vec::new();
    };
    let mut rows = vec![Row::section(format!(
      "{} {}",
      tr(self.lang, "control_center.net_peer"),
      index + 1
    ))];
    rows.extend(PeerField::ALL.into_iter().map(|field| {
      let value = peer.get(field);
      let shown = if field.is_secret() && !value.is_empty() {
        MASKED.to_owned()
      } else {
        dash(value)
      };
      Row::value(
        Item::PeerField(field),
        tr(self.lang, field.label_key()),
        shown,
        None,
      )
    }));
    rows.push(Row::separator());
    rows.push(
      Row::action(
        Item::RemovePeer,
        tr(self.lang, "control_center.net_remove_peer"),
      )
      .icon(icons::DELETE),
    );
    rows
  }

  /// Runs an editor row. Returns `false` when the item is not the editor's.
  pub(super) fn editor_activate(&mut self, item: Item) -> bool {
    if self.editor_job.is_some() && !matches!(item, Item::Cancel) {
      // Saving or loading: the draft is about to be replaced.
      return matches!(
        item,
        Item::Field(_)
          | Item::Section(_)
          | Item::OpenPorts
          | Item::Port(_)
          | Item::OpenPeers
          | Item::Peer(_)
          | Item::AddPeer
          | Item::PeerField(_)
          | Item::RemovePeer
          | Item::ListEntry(_)
          | Item::AddListEntry
          | Item::GenerateKey
          | Item::Save
          | Item::ToggleActive
          | Item::Delete
      );
    }
    let page = self.page;
    let Some(state) = &mut self.editor else {
      return matches!(item, Item::Cancel) && {
        self.close_editor();
        true
      };
    };
    match item {
      Item::Field(key) if page == NetworkPage::Editor => {
        let Some(property) = state.draft.property(key) else {
          return true;
        };
        match property.widget {
          Widget::Toggle => state.draft.toggle(key),
          Widget::Choice(_) => state.draft.adjust(key, 1),
          Widget::List(_) => self.go_to(NetworkPage::EditorList(key)),
          Widget::Secret(_) => {
            let current = state.draft.secret(key).unwrap_or_default().to_owned();
            self.open_text(EditTarget::Field(key), property.label, &current, true);
          }
          Widget::Text(_) | Widget::Number { .. } | Widget::Device { .. } => {
            let current = state.draft.value(key).to_owned();
            self.open_text(EditTarget::Field(key), property.label, &current, false);
          }
        }
      }
      Item::Section(section) => {
        if !state.collapsed.remove(&section) {
          state.collapsed.insert(section);
        }
      }
      Item::OpenPorts => self.go_to(NetworkPage::EditorPorts),
      Item::Port(index) => {
        if let Some(device) = self.port_candidates().get(index).cloned()
          && let Some(state) = &mut self.editor
          && !state.draft.ports.remove(&device)
        {
          state.draft.ports.insert(device);
        }
      }
      Item::OpenPeers => self.go_to(NetworkPage::EditorPeers),
      Item::Peer(index) => self.go_to(NetworkPage::EditorPeer(index)),
      Item::AddPeer => {
        state.draft.peers.push(WireGuardPeer::default());
        let index = state.draft.peers.len() - 1;
        self.go_to(NetworkPage::EditorPeer(index));
      }
      Item::PeerField(field) => {
        if let NetworkPage::EditorPeer(index) = page
          && let Some(peer) = state.draft.peers.get(index)
        {
          let current = peer.get(field).to_owned();
          self.open_text(
            EditTarget::PeerField(index, field),
            field.label_key(),
            &current,
            field.is_secret(),
          );
        }
      }
      Item::RemovePeer => {
        if let NetworkPage::EditorPeer(index) = page
          && index < state.draft.peers.len()
        {
          state.draft.peers.remove(index);
          self.go_to(NetworkPage::EditorPeers);
        }
      }
      Item::ListEntry(index) => {
        if let NetworkPage::EditorList(key) = page {
          let current = split_list(state.draft.value(key))
            .get(index)
            .cloned()
            .unwrap_or_default();
          let label = state
            .draft
            .property(key)
            .map_or("control_center.net_add_entry", |property| property.label);
          self.open_text(
            EditTarget::ListEntry(key, Some(index)),
            label,
            &current,
            false,
          );
        }
      }
      Item::AddListEntry => {
        if let NetworkPage::EditorList(key) = page {
          self.open_text(
            EditTarget::ListEntry(key, None),
            "control_center.net_add_entry",
            "",
            false,
          );
        }
      }
      Item::GenerateKey => self.generate_wireguard_key(),
      Item::Save => self.request_save(),
      Item::Cancel => self.editor_back(),
      Item::ToggleActive => self.toggle_edited_profile(),
      Item::Delete => {
        if self
          .edited_profile()
          .is_some_and(|profile| !profile.read_only)
        {
          self.editor_confirm = Some((EditorConfirm::Delete, ConfirmState::new()));
        }
      }
      _ => return false,
    }
    true
  }

  /// ←/→ on a choice, number or device row of the editor.
  pub(super) fn editor_adjust(&mut self, item: Item, step: i32) {
    if self.editor_job.is_some() {
      return;
    }
    let Item::Field(key) = item else {
      return;
    };
    let devices = self.device_candidates(key);
    let Some(state) = &mut self.editor else {
      return;
    };
    match state.draft.property(key).map(|property| property.widget) {
      Some(Widget::Device { .. }) => state.draft.adjust_device(key, &devices, step),
      Some(_) => state.draft.adjust(key, step),
      None => {}
    }
  }

  /// Opens the text editor for `target`, titled by the translation `label`.
  pub(super) fn open_text(&mut self, target: EditTarget, label: &str, value: &str, masked: bool) {
    let editor = TextEditor::single_line(tr(self.lang, label), value);
    let editor = if masked { editor.masked() } else { editor };
    self.text_edit = Some((target, editor));
  }

  /// Writes what the text editor saved to its target.
  pub(super) fn apply_text(&mut self, target: EditTarget, value: String) {
    let value = value.trim().to_owned();
    if let EditTarget::ImportPath(plugin) = target {
      self.import_vpn(plugin, value);
      return;
    }
    let lang = self.lang;
    let Some(state) = &mut self.editor else {
      return;
    };
    let mut problem = None;
    match target {
      EditTarget::Field(key) => {
        let Some(property) = state.draft.property(key) else {
          return;
        };
        problem = validate::check(property.widget, &value).err();
        if property.is_secret() {
          let derive = key == "wireguard.private-key" && problem.is_none() && !value.is_empty();
          state.draft.set_secret(key, value.clone());
          state.public_key = None;
          if derive {
            self.derive_public_key(value);
          }
        } else {
          state.draft.set(key, value);
        }
      }
      EditTarget::ListEntry(key, index) => {
        let Some(Widget::List(rule)) = state.draft.property(key).map(|property| property.widget)
        else {
          return;
        };
        let mut entries = split_list(state.draft.value(key));
        match index {
          Some(index) if value.is_empty() => {
            if index < entries.len() {
              entries.remove(index);
            }
          }
          _ if value.is_empty() => return,
          _ => match validate::check_list_entry(rule, &value) {
            Err(error) => problem = Some(error),
            Ok(()) => match index {
              Some(index) if index < entries.len() => entries[index] = value,
              _ => entries.push(value),
            },
          },
        }
        state.draft.set(key, entries.join(", "));
      }
      EditTarget::PeerField(index, field) => {
        if let Some(peer) = state.draft.peers.get_mut(index) {
          if !value.is_empty() {
            problem = validate::check(field.widget(), &value).err();
          }
          peer.set(field, value);
        }
      }
      EditTarget::ImportPath(_) => {}
    }
    if let Some(error) = problem {
      self.status = Some(StatusMessage {
        kind: StatusKind::Error,
        text: tr(lang, error.message_key()).into(),
      });
    }
  }

  /// Saves the draft, asking first when the profile is active.
  fn request_save(&mut self) {
    let Some(state) = &self.editor else {
      return;
    };
    if let Some(error) = state.draft.first_error() {
      self.status = Some(StatusMessage {
        kind: StatusKind::Error,
        text: self.draft_error_text(&state.draft, error),
      });
      return;
    }
    if !state.draft.is_dirty() {
      return;
    }
    if self.edited_profile().is_some_and(|profile| profile.active) {
      self.editor_confirm = Some((EditorConfirm::SaveActive, ConfirmState::new()));
    } else {
      self.perform_save();
    }
  }

  /// Saves the draft and reloads it as NetworkManager now has it.
  pub(super) fn perform_save(&mut self) {
    let Some(state) = &self.editor else {
      return;
    };
    let draft = state.draft.clone();
    let profiles = self.snapshot.profiles.clone();
    let owner = self.owner.clone();
    self.spawn_editor_job("control_center.net_saving", move |backend| {
      let outcome = backend.save_profile(&draft, &profiles)?;
      let profiles = backend.list_profiles()?;
      let draft = backend.load_profile(&outcome.uuid, &profiles, &owner)?;
      Ok(EditorResult::Saved {
        draft: Box::new(draft),
        profiles,
        outcome,
      })
    });
  }

  /// Deletes the profile being edited (and its ports).
  pub(super) fn perform_delete(&mut self) {
    let Some(state) = &self.editor else {
      return;
    };
    let draft = state.draft.clone();
    let profiles = self.snapshot.profiles.clone();
    self.spawn_editor_job("control_center.net_deleting", move |backend| {
      backend.delete_profile(&draft, &profiles)?;
      Ok(EditorResult::Deleted)
    });
  }

  /// Brings the edited profile up or down, as it is saved.
  fn toggle_edited_profile(&mut self) {
    if let Some(profile) = self.edited_profile().cloned() {
      self.toggle_profile(&profile);
    }
  }

  /// Brings a profile up when it is down, and down when it is up.
  pub(super) fn toggle_profile(&mut self, profile: &ConnectionProfile) {
    let uuid = profile.uuid.clone();
    if profile.active {
      self.action("Desconectando", move |backend| backend.deactivate(&uuid));
    } else {
      self.action("Conectando", move |backend| backend.activate(&uuid));
    }
  }

  fn generate_wireguard_key(&mut self) {
    if !self.capabilities.has_wireguard_tools {
      return;
    }
    self.spawn_editor_job("control_center.net_generating_key", |backend| {
      let private = backend.wireguard_private_key()?;
      let public = backend.wireguard_public_key(&private)?;
      Ok(EditorResult::KeyPair { private, public })
    });
  }

  fn derive_public_key(&mut self, private: String) {
    if !self.capabilities.has_wireguard_tools {
      return;
    }
    self.spawn_editor_job("control_center.net_generating_key", move |backend| {
      backend
        .wireguard_public_key(&private)
        .map(EditorResult::PublicKey)
    });
  }

  /// The message of the pending editor confirmation.
  pub(super) fn editor_confirm_text(
    &self,
    confirm: EditorConfirm,
  ) -> (String, String, String, bool) {
    let name = self
      .editor
      .as_ref()
      .map(|state| state.draft.value("connection.id").to_owned())
      .unwrap_or_default();
    let lang = self.lang;
    match confirm {
      EditorConfirm::Discard => (
        tr(lang, "control_center.net_discard_title").into(),
        tr(lang, "control_center.net_discard_message").into(),
        tr(lang, "control_center.net_discard").into(),
        true,
      ),
      EditorConfirm::SaveActive => (
        tr(lang, "control_center.net_save_active_title").into(),
        lang.tr_args(
          "control_center.net_save_active_message",
          [("name", name.as_str())],
        ),
        tr(lang, "control_center.net_save").into(),
        false,
      ),
      EditorConfirm::Delete => {
        let ports = self
          .editor
          .as_ref()
          .map(|state| {
            state
              .draft
              .ports
              .iter()
              .cloned()
              .collect::<Vec<_>>()
              .join(", ")
          })
          .unwrap_or_default();
        let mut message = lang.tr_args(
          "control_center.net_delete_message",
          [("name", name.as_str())],
        );
        if !ports.is_empty() {
          message.push('\n');
          message.push_str(&lang.tr_args(
            "control_center.net_delete_ports_message",
            [("ports", ports.as_str())],
          ));
        }
        if self.edited_profile().is_some_and(|profile| profile.active) {
          message.push('\n');
          message.push_str(tr(lang, "control_center.net_delete_active_message"));
        }
        (
          tr(lang, "control_center.net_delete_connection").into(),
          message,
          tr(lang, "control_center.delete").into(),
          true,
        )
      }
    }
  }

  /// Breadcrumb of an editor page, after `Connections`.
  pub(super) fn editor_breadcrumb(&self) -> Vec<String> {
    let mut parts = Vec::new();
    let Some(state) = &self.editor else {
      parts.push(
        tr(self.lang, "control_center.loading")
          .trim_end_matches('.')
          .to_owned(),
      );
      return parts;
    };
    let draft = &state.draft;
    if draft.is_new() {
      parts.push(tr(self.lang, "control_center.net_add_connection").to_owned());
      parts.push(tr(self.lang, draft.kind.label_key()).to_owned());
    } else {
      parts.push(argvus_control_center_core::sanitize::terminal_text(
        draft.value("connection.id"),
      ));
    }
    match self.page {
      NetworkPage::EditorList(key) => {
        if let Some(property) = draft.property(key) {
          parts.push(tr(self.lang, property.label).to_owned());
        }
      }
      NetworkPage::EditorPorts => {
        parts.push(tr(self.lang, "control_center.net_f_ports").to_owned())
      }
      NetworkPage::EditorPeers => {
        parts.push(tr(self.lang, "control_center.net_f_peers").to_owned())
      }
      NetworkPage::EditorPeer(index) => {
        parts.push(tr(self.lang, "control_center.net_f_peers").to_owned());
        parts.push(format!(
          "{} {}",
          tr(self.lang, "control_center.net_peer"),
          index + 1
        ));
      }
      _ => {}
    }
    parts
  }
}
