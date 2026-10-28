//! GPG keys pages: the key form, import, export, signing, expiry and generation.

use super::*;

impl App {
  /// The date layout follows the language, the same rule the form uses to read
  /// and write dates.
  pub(in super::super) fn gpg_date_order(&self) -> DateOrder {
    DateOrder::for_locale(&self.lang.locale())
  }

  /// Opens the single-line editor for one text field of the generate form.
  pub(super) fn open_gpg_field(&mut self, title_key: &str, value: String, target: EditTarget) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, title_key).into(),
      value,
      target,
      false,
    ));
  }

  pub(super) fn open_gpg_import(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.gpg_keys_import_title").into(),
      String::new(),
      EditTarget::GpgImportPath,
      false,
    ));
  }

  /// Proposes `~/<fingerprint>.asc`, which the user can change before the export
  /// runs. The launcher never overwrites an existing file.
  pub(super) fn open_gpg_export(&mut self, index: usize) {
    let Some(entry) = self.gpg_keys.entries.get(index) else {
      return;
    };
    let home = std::env::var("HOME").unwrap_or_default();
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.gpg_keys_export_title").into(),
      format!("{home}/{}.asc", entry.fingerprint),
      EditTarget::GpgExportPath(entry.fingerprint.clone()),
      false,
    ));
  }

  /// The key on the edit page, looked up by fingerprint every time, so a
  /// refresh that removes the key leaves the page with nothing to act on.
  pub(in super::super) fn gpg_edit_entry(&self) -> Option<&gpg_keys::GpgKeyEntry> {
    let fingerprint = self.gpg_edit_fingerprint.as_deref()?;
    self
      .gpg_keys
      .entries
      .iter()
      .find(|entry| entry.fingerprint == fingerprint)
  }

  /// Opens the edit page of one key. The page follows the fingerprint, not the index.
  pub(super) fn open_gpg_key_edit(&mut self, index: usize) {
    let Some(fingerprint) = self
      .gpg_keys
      .entries
      .get(index)
      .map(|entry| entry.fingerprint.clone())
    else {
      return;
    };
    self.gpg_edit_fingerprint = Some(fingerprint);
    self.navigation.push(Page::GpgKeyEdit);
  }

  /// Starts the expiry page with the key's current expiry. A key that never
  /// expires starts with the switch on and a date two years away.
  pub(super) fn open_gpg_edit_expiration(&mut self) {
    let order = self.gpg_date_order();
    let expires = self
      .gpg_edit_entry()
      .map(|entry| entry.expires.clone())
      .unwrap_or_default();
    if expires.is_empty() || expires == "never" {
      self.gpg_edit_never = true;
      self.gpg_edit_expiry = CalendarDate::today().add_years(2).display(order);
    } else {
      self.gpg_edit_never = false;
      // The launcher prints expiry as ISO, so it is read year first.
      self.gpg_edit_expiry = CalendarDate::parse(&expires, DateOrder::YearFirst)
        .map(|date| date.display(order))
        .unwrap_or_default();
    }
    self.navigation.push(Page::GpgEditExpiration);
  }

  /// Checks the typed expiry, then changes the key in the background.
  pub(super) fn apply_gpg_expiry(&mut self) {
    let Some(fingerprint) = self.gpg_edit_fingerprint.clone() else {
      return;
    };
    let order = self.gpg_date_order();
    match gpg_keys::expiry_argument(self.gpg_edit_never, &self.gpg_edit_expiry, order) {
      Ok(expire) => self.start_gpg_action("control_center.gpg_keys_expiry_changed", move || {
        gpg_keys::set_expiry(&fingerprint, &expire)
      }),
      Err(error) => {
        let message = self.gpg_form_error_text(error);
        self.fail(message);
      }
    }
  }

  /// Runs one key change on a worker thread. Only one runs at a time, because
  /// each may open the pinentry window. The page reloads the keyring when it ends.
  pub(in super::super) fn start_gpg_action<F>(&mut self, success_key: &'static str, task: F)
  where
    F: FnOnce() -> Result<(), String> + Send + 'static,
  {
    if self.gpg_action.is_some() {
      self.fail(tr(self.lang, "control_center.gpg_keys_action_busy").to_string());
      return;
    }
    self.gpg_action = Some((self.jobs.spawn(move |_| Ok(task())), success_key));
    self.success(tr(self.lang, "control_center.gpg_keys_working").to_string());
  }

  pub(super) fn confirm_gpg_delete(&mut self, index: usize) {
    if let Some(entry) = self.gpg_keys.entries.get(index) {
      let fingerprint = entry.fingerprint.clone();
      self.open_confirm(PendingAction::DeleteGpgKey(fingerprint));
    }
  }

  /// Only a key with a private part has something to export.
  pub(super) fn confirm_gpg_export_private(&mut self, index: usize) {
    let Some(entry) = self.gpg_keys.entries.get(index) else {
      return;
    };
    if !entry.secret {
      self.fail(tr(self.lang, "control_center.gpg_keys_no_private").to_string());
      return;
    }
    let fingerprint = entry.fingerprint.clone();
    self.open_confirm(PendingAction::ExportGpgPrivate(fingerprint));
  }

  pub(super) fn confirm_gpg_import_private(&mut self) {
    self.open_confirm(PendingAction::ImportGpgPrivate);
  }

  /// Certifies the key on the edit page with the key at `index`.
  pub(super) fn confirm_gpg_sign(&mut self, index: usize) {
    let Some(fingerprint) = self.gpg_edit_fingerprint.clone() else {
      return;
    };
    let Some(signer) = self
      .gpg_keys
      .entries
      .get(index)
      .map(|entry| entry.fingerprint.clone())
    else {
      return;
    };
    self.open_confirm(PendingAction::SignGpgKey(fingerprint, signer));
  }

  /// Opens the path editor for a private export. The proposed file is
  /// `<fingerprint>-private.asc`, so it never collides with the public export;
  /// it can be changed, and the launcher never overwrites a file.
  pub(in super::super) fn open_gpg_export_private(&mut self, fingerprint: String) {
    let home = std::env::var("HOME").unwrap_or_default();
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.gpg_keys_export_private_title").into(),
      format!("{home}/{fingerprint}-private.asc"),
      EditTarget::GpgExportPrivatePath(fingerprint),
      false,
    ));
  }

  /// Opens the file manager to choose a key file, on a worker thread because it
  /// stays open until the user chooses or closes it.
  pub(in super::super) fn start_gpg_import_pick(&mut self, kind: gpg_keys::GpgImportKind) {
    if self.gpg_pick.is_some() || self.gpg_action.is_some() {
      self.fail(tr(self.lang, "control_center.gpg_keys_action_busy").to_string());
      return;
    }
    self.gpg_pick = Some((
      self.jobs.spawn(move |_| {
        Ok(argvus_control_center_core::picker::pick_file(
          "argvus-gpg-picker",
          "gpg-selection",
        ))
      }),
      kind,
    ));
    self.success(tr(self.lang, "control_center.gpg_keys_working").to_string());
  }

  /// Imports the file at `path` in the background, the same way as the other key changes.
  pub(in super::super) fn import_gpg_file(&mut self, kind: gpg_keys::GpgImportKind, path: &str) {
    let path = path.to_owned();
    match kind {
      gpg_keys::GpgImportKind::Public => self
        .start_gpg_action("control_center.gpg_keys_imported", move || {
          gpg_keys::import(&path)
        }),
      gpg_keys::GpgImportKind::Private => self
        .start_gpg_action("control_center.gpg_keys_private_imported", move || {
          gpg_keys::import_private(&path)
        }),
    }
  }

  /// The typed-path editor, used when the file manager cannot report a choice.
  pub(in super::super) fn open_gpg_import_path(&mut self, kind: gpg_keys::GpgImportKind) {
    match kind {
      gpg_keys::GpgImportKind::Public => self.open_gpg_import(),
      gpg_keys::GpgImportKind::Private => self.open_gpg_import_private(),
    }
  }

  pub(in super::super) fn open_gpg_import_private(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.gpg_keys_import_private_title").into(),
      String::new(),
      EditTarget::GpgImportPrivatePath,
      false,
    ));
  }

  /// Copies the armored public key through the launcher's standard output.
  pub(super) fn copy_gpg_public(&mut self, index: usize) {
    let Some(fingerprint) = self
      .gpg_keys
      .entries
      .get(index)
      .map(|entry| entry.fingerprint.clone())
    else {
      return;
    };
    match gpg_keys::public_key(&fingerprint).and_then(|key| {
      clipboard::copy(&key)
        .map_err(|_| tr(self.lang, "control_center.gpg_keys_copy_failed").to_string())
    }) {
      Ok(()) => self.success(tr(self.lang, "control_center.gpg_keys_public_copied").to_string()),
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn copy_gpg_fingerprint(&mut self, index: usize) {
    let Some(fingerprint) = self
      .gpg_keys
      .entries
      .get(index)
      .map(|entry| entry.fingerprint.clone())
    else {
      return;
    };
    match clipboard::copy(&fingerprint) {
      Ok(()) => {
        self.success(tr(self.lang, "control_center.gpg_keys_fingerprint_copied").to_string())
      }
      Err(_) => self.fail(tr(self.lang, "control_center.gpg_keys_copy_failed").to_string()),
    }
  }

  /// Checks the form first, so a mistake is reported before gpg runs. Without a
  /// passphrase the page asks first.
  pub(super) fn generate_gpg_key(&mut self) {
    let order = self.gpg_date_order();
    if let Err(error) = self.gpg_form.validate(order) {
      let message = self.gpg_form_error_text(error);
      self.fail(message);
      return;
    }
    if self.gpg_form.protect_with_passphrase {
      self.start_gpg_generation();
    } else {
      self.open_confirm(PendingAction::GenerateGpgNoPassphrase);
    }
  }

  /// Runs the launcher on a worker thread. The passphrase is cleared from the
  /// page as soon as the job has its copy, so it lives only in that job. When the
  /// job fails the other fields stay and the passphrase must be typed again.
  pub(in super::super) fn start_gpg_generation(&mut self) {
    let order = self.gpg_date_order();
    let form = self.gpg_form.clone();
    self.gpg_form.passphrase.clear();
    self.gpg_form.passphrase_confirm.clear();
    self.gpg_job = Some(
      self
        .jobs
        .spawn(move |_| Ok(gpg_keys::generate(&form, order))),
    );
    self.success(tr(self.lang, "control_center.gpg_keys_generating").to_string());
  }

  /// The message for one problem with the generate form, in the current language.
  pub(super) fn gpg_form_error_text(&self, error: gpg_keys::FormError) -> String {
    let key = match error {
      gpg_keys::FormError::NameRequired => "control_center.gpg_keys_name_required",
      gpg_keys::FormError::EmailInvalid => "control_center.gpg_keys_email_invalid",
      gpg_keys::FormError::DateInvalid => {
        let layout = self.gpg_date_order().layout();
        return self
          .lang
          .tr_args("control_center.gpg_keys_date_invalid", [("layout", layout)])
          .to_string();
      }
      gpg_keys::FormError::DateNotFuture => "control_center.gpg_keys_date_not_future",
      gpg_keys::FormError::PassphraseEmpty => "control_center.gpg_keys_passphrase_empty",
      gpg_keys::FormError::PassphraseInvalid => "control_center.gpg_keys_passphrase_invalid",
      gpg_keys::FormError::PassphraseMismatch => "control_center.gpg_keys_passphrase_mismatch",
    };
    tr(self.lang, key).to_string()
  }
}
