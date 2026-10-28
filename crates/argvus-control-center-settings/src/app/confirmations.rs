//! Confirmation dialogs: the pending action, its text and running it when accepted.

use super::labels::{category_label, enabled_label, font_target_label, setting_label};
use super::*;

impl App {
  /// A key while the confirmation is open: Enter runs the focused row
  /// (Cancel first), `y` confirms, `n`/Esc cancel, arrows and Tab move.
  pub fn confirm_key(&mut self, key: crossterm::event::KeyCode) {
    match self.confirm_focus.handle(key) {
      ConfirmOutcome::Confirmed => self.confirm_accept(),
      ConfirmOutcome::Cancelled => self.cancel_modal(),
      ConfirmOutcome::Pending => {}
    }
  }

  /// Runs the confirmed pending action.
  pub fn confirm_accept(&mut self) {
    self.confirm_focus = ConfirmState::new();
    let Some(action) = self.confirm.take() else {
      return;
    };
    match action {
      PendingAction::ResetApps => match self.apps.reset_all() {
        Ok(()) => {
          self.success(tr(self.lang, "control_center.default_applications_restored").to_string())
        }
        Err(error) => self.fail(error),
      },
      PendingAction::ResetApp(category) => match self.apps.reset_default(category) {
        Ok(()) => self.success(format!(
          "{} → {}",
          category_label(self.lang, category),
          tr(self.lang, "control_center.argvus_default")
        )),
        Err(error) => self.fail(error),
      },
      PendingAction::ResetFonts => match self.fonts.reset_all() {
        Ok(()) => self.success(tr(self.lang, "control_center.font_settings_restored").to_string()),
        Err(error) => self.fail(error),
      },
      PendingAction::ResetFont(target) => match self.fonts.reset_font(target) {
        Ok(()) => self.success(format!(
          "{} → {}",
          font_target_label(self.lang, target),
          tr(self.lang, "control_center.argvus_default")
        )),
        Err(error) => self.fail(error),
      },
      PendingAction::ResetFontSetting(setting) => match self.fonts.reset_setting(setting) {
        Ok(()) => self.success(format!(
          "{} → {}",
          setting_label(self.lang, setting),
          tr(self.lang, "control_center.argvus_default")
        )),
        Err(error) => self.fail(error),
      },
      PendingAction::RemoveWindowRule(name) => match window_rules::remove(&name) {
        Ok(()) => {
          self.reload_window_rules();
          self.success(tr(self.lang, "control_center.window_rules_removed").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::RemoveProject(path) => match projects::remove(&path) {
        Ok(()) => {
          self.projects = projects::load();
          self.success(tr(self.lang, "control_center.projects_removed").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::RemoveSnippet(name) => match snippets::remove(&name) {
        Ok(()) => {
          self.snippets = snippets::load();
          self.success(tr(self.lang, "control_center.snippets_removed").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::DeleteSshKey(name) => match ssh_keys::delete(&name) {
        Ok(()) => {
          self.ssh_keys = ssh_keys::load();
          self.success(tr(self.lang, "control_center.ssh_keys_deleted").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::GenerateGpgNoPassphrase => self.start_gpg_generation(),
      PendingAction::DeleteGpgKey(fingerprint) => self
        .start_gpg_action("control_center.gpg_keys_deleted", move || {
          gpg_keys::delete(&fingerprint)
        }),
      PendingAction::ExportGpgPrivate(fingerprint) => self.open_gpg_export_private(fingerprint),
      PendingAction::ImportGpgPrivate => {
        self.start_gpg_import_pick(gpg_keys::GpgImportKind::Private)
      }
      PendingAction::RevokeGpgKey(fingerprint, reason) => self
        .start_gpg_action("control_center.gpg_keys_revoked", move || {
          gpg_keys::revoke(&fingerprint, reason)
        }),
      PendingAction::SignGpgKey(fingerprint, signer) => self
        .start_gpg_action("control_center.gpg_keys_signed", move || {
          gpg_keys::sign_key(&fingerprint, &signer)
        }),
      PendingAction::SendGpgKey(fingerprint) => self
        .start_gpg_action("control_center.gpg_keys_sent", move || {
          gpg_keys::keyserver_send(&fingerprint)
        }),
      PendingAction::ResetKeybindings => match keybindings::restore_all(&self.keybindings) {
        Ok(()) => {
          self.keybindings = keybindings::load();
          self.success(tr(self.lang, "control_center.keybindings_restored_all").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::RestoreKeybinding { id, leave } => {
        self.restore_keybinding(&id);
        if leave {
          self.keybinding_capturing = false;
          self.navigation.back();
        }
      }
      PendingAction::ApplySystemLocales => self.apply_system_locales(),
      PendingAction::DiscardDraft => self.discard_and_leave(),
      PendingAction::Administration { .. } => self.admin.submit(),
      PendingAction::SetNtp(enabled) => match time::set_ntp(enabled) {
        Ok(()) => {
          self.refresh_time();
          self.success(tr(self.lang, "control_center.ntp_setting_applied").to_string());
        }
        Err(error) => self.fail(error),
      },
    }
  }

  pub fn cancel_modal(&mut self) {
    let delete_asked = self.admin.delete_confirm;
    self.admin.cancel_pending();
    self.confirm = None;
    self.confirm_focus = ConfirmState::new();
    // No on the delete confirmation returns to the user list.
    if delete_asked {
      self.leave_user_pages();
    }
  }

  /// Texts of the open confirmation: title, message, confirm label and
  /// whether it uses the danger style.
  pub fn confirm_dialog(&self) -> Option<(String, String, &'static str, bool)> {
    let label = |key: &str| tr(self.lang, key);
    if let Some((_, keys, conflicts)) = self.keybinding_conflict_state() {
      let mut message = format!(
        "{}\n{keys}\n\n{}",
        label("control_center.keybindings_conflict"),
        label("control_center.keybindings_used_by")
      );
      for id in conflicts {
        message.push_str(&format!("\n• {}", self.keybinding_label_for(&id)));
      }
      return Some((
        label("control_center.keybindings_conflict_title").into(),
        message,
        label("control_center.replace"),
        false,
      ));
    }
    let action = self.confirm.as_ref()?;
    let (title, message) = pending_action_text(self.lang, action);
    let (confirm, danger) = match action {
      // The user delete confirmation answers Yes / No.
      PendingAction::Administration { danger: true, .. } if self.admin.delete_confirm => {
        (label("control_center.yes"), true)
      }
      PendingAction::Administration { danger: true, .. } => (label("control_center.delete"), true),
      PendingAction::DiscardDraft => (label("control_center.discard"), false),
      PendingAction::ResetApps
      | PendingAction::ResetApp(_)
      | PendingAction::ResetFonts
      | PendingAction::ResetFont(_)
      | PendingAction::ResetFontSetting(_)
      | PendingAction::ResetKeybindings
      | PendingAction::RemoveWindowRule(_)
      | PendingAction::RemoveProject(_)
      | PendingAction::RemoveSnippet(_)
      | PendingAction::DeleteSshKey(_)
      | PendingAction::DeleteGpgKey(_)
      | PendingAction::ExportGpgPrivate(_)
      | PendingAction::RevokeGpgKey(..)
      | PendingAction::GenerateGpgNoPassphrase => (label("control_center.confirm"), true),
      _ => (label("control_center.confirm"), false),
    };
    Some((title, message, confirm, danger))
  }

  pub(super) fn open_confirm(&mut self, action: PendingAction) {
    self.confirm = Some(action);
    self.confirm_focus = ConfirmState::new();
  }
}

pub fn pending_action_text(lang: Lang, action: &PendingAction) -> (String, String) {
  match action {
    PendingAction::Administration { message, .. } => (
      tr(lang, "control_center.confirm_administrative_operation").into(),
      message.clone(),
    ),
    PendingAction::ResetApps => (
      tr(lang, "control_center.reset_default_applications").to_string(),
      tr(
        lang,
        "control_center.this_will_restore_the_application_choices_defined_by_argvus",
      )
      .to_string(),
    ),
    PendingAction::ResetApp(category) => (
      format!("{}?", tr(lang, "control_center.reset_default_application")),
      category_label(lang, *category).to_string(),
    ),
    PendingAction::ResetFonts => (
      tr(lang, "control_center.reset_font_settings").to_string(),
      tr(
        lang,
        "control_center.this_will_restore_the_font_settings_defined_by_argvus",
      )
      .to_string(),
    ),
    PendingAction::ResetFont(target) => (
      format!("{}?", tr(lang, "control_center.reset_font")),
      font_target_label(lang, *target).to_string(),
    ),
    PendingAction::ResetFontSetting(setting) => (
      format!("{}?", tr(lang, "control_center.reset_setting")),
      setting_label(lang, *setting).to_string(),
    ),
    PendingAction::ResetKeybindings => (
      tr(lang, "control_center.restore_all_shortcuts").to_string(),
      tr(lang, "control_center.restore_all_shortcuts_description").to_string(),
    ),
    PendingAction::RemoveWindowRule(name) => (
      tr(lang, "control_center.window_rules_remove").to_string(),
      name.clone(),
    ),
    PendingAction::RemoveProject(path) => (
      tr(lang, "control_center.projects_remove_title").to_string(),
      path.clone(),
    ),
    PendingAction::RemoveSnippet(name) => (
      tr(lang, "control_center.snippets_remove_title").to_string(),
      name.clone(),
    ),
    PendingAction::DeleteSshKey(name) => (
      tr(lang, "control_center.ssh_keys_delete_title").to_string(),
      format!(
        "{name}. {}",
        tr(lang, "control_center.ssh_keys_delete_description")
      ),
    ),
    PendingAction::GenerateGpgNoPassphrase => (
      tr(lang, "control_center.gpg_keys_no_passphrase_title").to_string(),
      tr(lang, "control_center.gpg_keys_no_passphrase_description").to_string(),
    ),
    PendingAction::DeleteGpgKey(fingerprint) => (
      tr(lang, "control_center.gpg_keys_delete_title").to_string(),
      format!(
        "{fingerprint}. {}",
        tr(lang, "control_center.gpg_keys_delete_description")
      ),
    ),
    PendingAction::ExportGpgPrivate(fingerprint) => (
      tr(lang, "control_center.gpg_keys_export_private_confirm_title").to_string(),
      format!(
        "{fingerprint}. {}",
        tr(lang, "control_center.gpg_keys_export_private_description")
      ),
    ),
    PendingAction::ImportGpgPrivate => (
      tr(lang, "control_center.gpg_keys_import_private_confirm_title").to_string(),
      tr(lang, "control_center.gpg_keys_import_private_description").to_string(),
    ),
    PendingAction::RevokeGpgKey(fingerprint, _) => (
      tr(lang, "control_center.gpg_keys_revoke_title").to_string(),
      format!(
        "{fingerprint}. {}",
        tr(lang, "control_center.gpg_keys_revoke_description")
      ),
    ),
    PendingAction::SignGpgKey(fingerprint, signer) => (
      tr(lang, "control_center.gpg_keys_sign_title").to_string(),
      format!(
        "{fingerprint} → {signer}. {}",
        tr(lang, "control_center.gpg_keys_sign_description")
      ),
    ),
    PendingAction::SendGpgKey(fingerprint) => (
      tr(lang, "control_center.gpg_keys_send_title").to_string(),
      format!(
        "{fingerprint}. {}",
        tr(lang, "control_center.gpg_keys_send_description")
      ),
    ),
    PendingAction::RestoreKeybinding { .. } => (
      tr(lang, "control_center.restore_shortcut_title").to_string(),
      tr(lang, "control_center.restore_shortcut_description").to_string(),
    ),
    PendingAction::ApplySystemLocales => (
      tr(lang, "control_center.apply_locale_changes").to_string(),
      tr(
        lang,
        "control_center.this_will_update_etc_locale_gen_and_run_locale_gen",
      )
      .to_string(),
    ),
    PendingAction::DiscardDraft => (
      tr(lang, "control_center.discard_changes_title").to_string(),
      tr(lang, "control_center.discard_changes_description").to_string(),
    ),
    PendingAction::SetNtp(enabled) => (
      tr(lang, "control_center.change_automatic_date_time").to_string(),
      enabled_label(lang, *enabled).to_string(),
    ),
  }
}
