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

mod activation;
mod jobs;
mod rows;

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

  /// The listed profile the editor shows, when it is saved.
  fn edited_profile(&self) -> Option<&ConnectionProfile> {
    let uuid = self.editor.as_ref()?.draft.uuid.as_deref()?;
    self
      .snapshot
      .profiles
      .iter()
      .find(|profile| profile.uuid == uuid)
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
