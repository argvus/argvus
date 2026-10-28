//! Rows of the profile editor: sections, fields, lists, ports and WireGuard peers, and their labels.

use super::*;

/// The label of a choice's current value.
pub(super) fn choice_label(lang: Lang, options: &[Choice], value: &str) -> String {
  options
    .iter()
    .find(|choice| choice.value == value)
    .map_or_else(|| dash(value), |choice| tr(lang, choice.label).to_owned())
}

pub(super) fn dash(value: &str) -> String {
  if value.is_empty() {
    "—".to_owned()
  } else {
    value.to_owned()
  }
}

/// A list value in one line: its first entry and how many follow.
pub(super) fn list_summary(value: &str) -> String {
  let entries = split_list(value);
  match entries.as_slice() {
    [] => "—".to_owned(),
    [only] => only.clone(),
    [first, rest @ ..] => format!("{first}  +{}", rest.len()),
  }
}

/// An example entry of a list, shown under it.
pub(super) fn list_example(rule: ListRule) -> &'static str {
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
pub(super) fn short_key(key: &str) -> String {
  if key.chars().count() > 12 {
    format!("{}…", key.chars().take(12).collect::<String>())
  } else {
    key.to_owned()
  }
}

impl NetworkApp {
  /// The devices a device field can cycle through.
  pub(in super::super) fn device_candidates(&self, key: &str) -> Vec<String> {
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
  pub(in super::super) fn port_candidates(&self) -> Vec<String> {
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

  /// Rows of the editor page.
  pub(in super::super) fn editor_rows(&self) -> Vec<Row<Item>> {
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
  pub(in super::super) fn profile_status(&self, profile: &ConnectionProfile) -> String {
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
  pub(super) fn draft_error_text(&self, draft: &ConnectionDraft, error: DraftError) -> String {
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
  pub(super) fn field_row(
    &self,
    draft: &ConnectionDraft,
    property: &'static Property,
  ) -> Row<Item> {
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
  pub(super) fn secret_text(&self, draft: &ConnectionDraft, key: &str) -> String {
    match (draft.secret_state(key), draft.secret(key)) {
      (SecretState::Changed, Some("")) => tr(self.lang, "control_center.net_secret_cleared").into(),
      (SecretState::Changed, _) => MASKED.to_owned(),
      (SecretState::Unchanged, _) if draft.is_new() => "—".to_owned(),
      (SecretState::Unchanged, _) => tr(self.lang, "control_center.net_secret_unchanged").into(),
    }
  }

  /// Rows a section adds after its fields: ports, peers, key generation.
  pub(super) fn section_extras(&self, state: &EditorState, section: SectionId) -> Vec<Row<Item>> {
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
  pub(in super::super) fn editor_list_rows(&self, key: &'static str) -> Vec<Row<Item>> {
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
  pub(in super::super) fn editor_port_rows(&self) -> Vec<Row<Item>> {
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
  pub(in super::super) fn editor_peer_rows(&self) -> Vec<Row<Item>> {
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
  pub(in super::super) fn editor_single_peer_rows(&self, index: usize) -> Vec<Row<Item>> {
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
}
