//! Running a row of the profile editor: choices, value steps, text fields and the confirmation texts.

use super::*;

impl NetworkApp {
  /// Runs an editor row. Returns `false` when the item is not the editor's.
  pub(in super::super) fn editor_activate(&mut self, item: Item) -> bool {
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
  pub(in super::super) fn editor_adjust(&mut self, item: Item, step: i32) {
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
  pub(in super::super) fn open_text(
    &mut self,
    target: EditTarget,
    label: &str,
    value: &str,
    masked: bool,
  ) {
    let editor = TextEditor::single_line(tr(self.lang, label), value);
    let editor = if masked { editor.masked() } else { editor };
    self.text_edit = Some((target, editor));
  }

  /// Writes what the text editor saved to its target.
  pub(in super::super) fn apply_text(&mut self, target: EditTarget, value: String) {
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

  /// The message of the pending editor confirmation.
  pub(in super::super) fn editor_confirm_text(
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
}
