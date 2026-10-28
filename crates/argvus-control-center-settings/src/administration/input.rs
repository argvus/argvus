//! Keys and pasted text on the administration pages: the user action buttons, page focus and the inline editor.

use super::editor::EditTarget;
use super::*;

impl App {
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
