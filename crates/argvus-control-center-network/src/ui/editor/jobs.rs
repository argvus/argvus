//! Background work of the profile editor: saving, deleting and toggling profiles, and generating WireGuard keys.

use super::*;

impl NetworkApp {
  /// Runs an editor task in the background. `progress` is the translation
  /// key shown while it runs.
  pub(in super::super) fn spawn_editor_job<F>(&mut self, progress: &'static str, task: F)
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
  pub(in super::super) fn poll_editor(&mut self) -> bool {
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

  /// Saves the draft, asking first when the profile is active.
  pub(super) fn request_save(&mut self) {
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
  pub(in super::super) fn perform_save(&mut self) {
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
  pub(in super::super) fn perform_delete(&mut self) {
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
  pub(super) fn toggle_edited_profile(&mut self) {
    if let Some(profile) = self.edited_profile().cloned() {
      self.toggle_profile(&profile);
    }
  }

  /// Brings a profile up when it is down, and down when it is up.
  pub(in super::super) fn toggle_profile(&mut self, profile: &ConnectionProfile) {
    let uuid = profile.uuid.clone();
    if profile.active {
      self.action("Desconectando", move |backend| backend.deactivate(&uuid));
    } else {
      self.action("Conectando", move |backend| backend.activate(&uuid));
    }
  }

  pub(super) fn generate_wireguard_key(&mut self) {
    if !self.capabilities.has_wireguard_tools {
      return;
    }
    self.spawn_editor_job("control_center.net_generating_key", |backend| {
      let private = backend.wireguard_private_key()?;
      let public = backend.wireguard_public_key(&private)?;
      Ok(EditorResult::KeyPair { private, public })
    });
  }

  pub(super) fn derive_public_key(&mut self, private: String) {
    if !self.capabilities.has_wireguard_tools {
      return;
    }
    self.spawn_editor_job("control_center.net_generating_key", move |backend| {
      backend
        .wireguard_public_key(&private)
        .map(EditorResult::PublicKey)
    });
  }
}
