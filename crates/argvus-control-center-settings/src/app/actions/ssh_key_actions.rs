//! SSH keys page: generating, copying and loading keys into the agent.

use super::*;

impl App {
  pub(super) fn open_new_ssh_key_name(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.ssh_keys_name").into(),
      self.new_ssh_key_name.clone(),
      EditTarget::NewSshKeyName,
      false,
    ));
  }

  /// Changes only the form's algorithm; nothing is written until Generate runs.
  pub(super) fn toggle_ssh_key_type(&mut self) {
    self.new_ssh_key_type = self.new_ssh_key_type.toggled();
  }

  pub(super) fn generate_ssh_key(&mut self) {
    let name = self.new_ssh_key_name.trim().to_owned();
    if name.is_empty() {
      self.fail(tr(self.lang, "control_center.ssh_keys_name_required").to_string());
      return;
    }
    match ssh_keys::generate(&name, self.new_ssh_key_type) {
      Ok(()) => {
        self.new_ssh_key_name.clear();
        self.ssh_keys = ssh_keys::load();
        self.success(tr(self.lang, "control_center.ssh_keys_generated").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  /// Opens a terminal for ssh-add, which asks for the passphrase there.
  /// The key is not in the agent yet when this returns.
  pub(super) fn copy_ssh_public_key(&mut self, index: usize) {
    let Some(name) = self
      .ssh_keys
      .entries
      .get(index)
      .map(|entry| entry.name.clone())
    else {
      return;
    };
    match ssh_keys::copy_public_key(&name) {
      Ok(()) => {
        self.success(tr(self.lang, "control_center.ssh_keys_public_key_copied").to_string())
      }
      Err(ssh_keys::CopyError::Unreadable) => {
        self.fail(tr(self.lang, "control_center.ssh_keys_public_key_unreadable").to_string());
      }
      Err(ssh_keys::CopyError::Clipboard) => {
        self.fail(tr(self.lang, "control_center.ssh_keys_clipboard_unavailable").to_string());
      }
    }
  }

  pub(super) fn load_ssh_key(&mut self, index: usize) {
    let Some(name) = self
      .ssh_keys
      .entries
      .get(index)
      .map(|entry| entry.name.clone())
    else {
      return;
    };
    match ssh_keys::load_key(&name) {
      Ok(()) => self.success(tr(self.lang, "control_center.ssh_keys_loading").to_string()),
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn unload_ssh_key(&mut self, index: usize) {
    let Some(name) = self
      .ssh_keys
      .entries
      .get(index)
      .map(|entry| entry.name.clone())
    else {
      return;
    };
    match ssh_keys::unload(&name) {
      Ok(()) => {
        self.ssh_keys = ssh_keys::load();
        self.success(tr(self.lang, "control_center.ssh_keys_unloaded").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn unload_all_ssh_keys(&mut self) {
    match ssh_keys::unload_all() {
      Ok(()) => {
        self.ssh_keys = ssh_keys::load();
        self.success(tr(self.lang, "control_center.ssh_keys_unloaded_all").to_string());
      }
      Err(error) => self.fail(error),
    }
  }
}
