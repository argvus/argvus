//! Draft form and background persistence for user-created keyboard shortcuts.
use super::*;
use crate::administration::{EditTarget, Editor};
use crate::item::Item;
use argvus_tui::menu::Row;

#[derive(Default)]
pub(super) struct ShortcutDraft {
  pub id: String,
  pub existing: bool,
  pub description: String,
  pub keys: String,
  pub dispatcher_action: bool,
  pub command: String,
  pub dispatcher: String,
  pub arguments: String,
  pub enabled: bool,
  pub accepted_conflicts: Vec<String>,
}

pub(super) struct ShortcutJob {
  pub handle: JobHandle<(Vec<keybindings::Binding>, Option<String>)>,
  pub id: String,
  pub leave: bool,
  pub deleted: bool,
}

impl App {
  pub(crate) fn shortcut_mutation_pending(&self) -> bool {
    self.shortcut_job.is_some()
  }

  pub(crate) fn custom_shortcut_is_existing(&self) -> bool {
    self.custom_shortcut.existing
  }

  pub(super) fn open_custom_shortcut(&mut self, binding: Option<&keybindings::Binding>) {
    self.keybinding_capturing = false;
    self.keybinding_conflict = None;
    self.search.clear();
    let mut draft = ShortcutDraft {
      id: format!(
        "custom_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap_or_default()
          .as_nanos()
      ),
      enabled: true,
      dispatcher: "window.close".into(),
      ..ShortcutDraft::default()
    };
    if let Some(binding) = binding
      && let Some(shortcut) = &binding.custom
    {
      draft.id = binding.id.clone();
      draft.existing = true;
      draft.description = shortcut.description.clone();
      draft.keys = shortcut.keys.clone();
      draft.enabled = shortcut.enabled;
      match &shortcut.action {
        keybindings::ShortcutAction::Command { command } => draft.command = command.clone(),
        keybindings::ShortcutAction::Dispatcher {
          dispatcher,
          arguments,
        } => {
          draft.dispatcher_action = true;
          draft.dispatcher = dispatcher.clone();
          draft.arguments = arguments
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        }
      }
    }
    self.custom_shortcut = draft;
  }

  pub(super) fn custom_shortcut_rows(&self) -> Vec<Row<Item>> {
    let draft = &self.custom_shortcut;
    let field =
      |item, label, value: &str| Row::value(item, tr(self.lang, label), value.to_owned(), None);
    let mut rows = vec![
      Row::section(tr(
        self.lang,
        if draft.existing {
          "control_center.edit_shortcut"
        } else {
          "control_center.shortcut_create_title"
        },
      )),
      field(
        Item::CustomDescription,
        "control_center.shortcut_description",
        &draft.description,
      ),
      Row::action(
        Item::CustomKeys,
        tr(self.lang, "control_center.keybindings_current_shortcut"),
      )
      .detail(keybindings::display_keys(&draft.keys)),
      Row::action(
        Item::CaptureCustomShortcut,
        tr(self.lang, "control_center.change_shortcut"),
      ),
      field(
        Item::CustomActionType,
        "control_center.shortcut_action_type",
        tr(
          self.lang,
          if draft.dispatcher_action {
            "control_center.shortcut_dispatcher"
          } else {
            "control_center.shortcut_command"
          },
        ),
      ),
    ];
    if draft.dispatcher_action {
      rows.push(field(
        Item::CustomDispatcher,
        "control_center.shortcut_dispatcher",
        &draft.dispatcher,
      ));
      rows.push(field(
        Item::CustomArguments,
        "control_center.shortcut_arguments",
        &draft.arguments,
      ));
      rows.push(Row::info(
        tr(self.lang, "control_center.shortcut_arguments_help"),
        "",
      ));
    } else {
      rows.push(field(
        Item::CustomCommand,
        "control_center.shortcut_command",
        &draft.command,
      ));
    }
    if self.keybinding_capturing {
      rows.push(Row::info(
        tr(self.lang, "control_center.press_new_shortcut"),
        "",
      ));
    }
    if draft.existing {
      self.danger_zone(
        &mut rows,
        vec![Row::destructive(
          Item::DeleteCustomShortcut,
          tr(self.lang, "control_center.delete"),
        )],
      );
    }
    rows
  }

  pub(super) fn custom_shortcut_action_rows(&self) -> Vec<Row<Item>> {
    let busy = self.shortcut_job.is_some();
    vec![
      Row::action(
        Item::SaveCustomShortcut,
        tr(
          self.lang,
          if self.custom_shortcut.existing {
            "control_center.save"
          } else {
            "control_center.create"
          },
        ),
      )
      .enabled(!busy),
      Row::action(
        Item::CancelCustomShortcut,
        tr(self.lang, "control_center.cancel"),
      )
      .enabled(!busy),
    ]
  }

  pub(crate) fn edit_custom_shortcut_field(&mut self, item: Item) {
    let draft = &self.custom_shortcut;
    let (label, value) = match item {
      Item::CustomDescription => ("control_center.shortcut_description", &draft.description),
      Item::CustomCommand => ("control_center.shortcut_command", &draft.command),
      Item::CustomDispatcher => ("control_center.shortcut_dispatcher", &draft.dispatcher),
      Item::CustomArguments => ("control_center.shortcut_arguments", &draft.arguments),
      _ => return,
    };
    self.admin.editor = Some(Editor::new(
      tr(self.lang, label).into(),
      value.clone(),
      EditTarget::CustomShortcutField(item),
      false,
    ));
  }

  pub(crate) fn set_custom_shortcut_field(&mut self, item: Item, value: String) {
    let draft = &mut self.custom_shortcut;
    match item {
      Item::CustomDescription => draft.description = value,
      Item::CustomCommand => draft.command = value,
      Item::CustomDispatcher => draft.dispatcher = value,
      Item::CustomArguments => draft.arguments = value,
      _ => {}
    }
  }

  pub(crate) fn begin_custom_shortcut_capture(&mut self) {
    self.keybinding_editor_key.clear();
    self.keybinding_editor_modifiers = [false; 4];
    self.keybinding_super_required = false;
    self.keybinding_detected = false;
    self.keybinding_capturing = true;
  }

  pub(crate) fn capture_custom_shortcut(&mut self, key: crossterm::event::KeyEvent) {
    self.capture_keybinding(key);
    if self.keybinding_detected && !self.keybinding_capturing {
      let mut parts = Vec::new();
      for (enabled, modifier) in self
        .keybinding_editor_modifiers
        .iter()
        .zip(["CTRL", "ALT", "SHIFT", "SUPER"])
      {
        if *enabled {
          parts.push(modifier);
        }
      }
      parts.push(&self.keybinding_editor_key);
      if let Ok(keys) = keybindings::normalize_keys(&parts.join(" + ")) {
        self.accept_captured_custom_keys(keys);
      }
    }
  }

  fn accept_captured_custom_keys(&mut self, keys: String) {
    if let Err(error) = argvus_config_core::custom_shortcuts::validate_keys(&keys) {
      argvus_control_center_core::diagnostics::error(&format!(
        "invalid captured shortcut: {error}"
      ));
      self.fail(tr(self.lang, "control_center.shortcut_invalid_keys"));
      return;
    }
    let conflicts = keybindings::conflicts(&self.keybindings, &self.custom_shortcut.id, &keys);
    if self
      .keybindings
      .iter()
      .any(|binding| conflicts.contains(&binding.id) && !binding.configurable)
    {
      self.fail(tr(self.lang, "control_center.shortcut_protected_conflict"));
      return;
    }
    if conflicts.is_empty() {
      self.custom_shortcut.keys = keys;
      self.custom_shortcut.accepted_conflicts.clear();
    } else {
      self.keybinding_conflict = Some((self.custom_shortcut.id.clone(), keys, conflicts));
      self.confirm_focus = ConfirmState::new();
    }
  }

  /// Acknowledging a captured conflict changes only the draft; Save performs the replacement.
  pub(super) fn accept_custom_shortcut_conflict(&mut self) {
    let Some((_, keys, conflicts)) = self.keybinding_conflict.take() else {
      return;
    };
    self.custom_shortcut.keys = keys;
    self.custom_shortcut.accepted_conflicts = conflicts;
    self.success(tr(self.lang, "control_center.shortcut_conflict_selected").into());
  }

  pub(crate) fn save_custom_shortcut(&mut self) {
    if self.shortcut_job.is_some() {
      return;
    }
    let draft = &self.custom_shortcut;
    if draft.description.trim().is_empty() {
      self.fail(tr(
        self.lang,
        "control_center.shortcut_description_required",
      ));
      return;
    }
    if !draft.dispatcher_action && draft.command.trim().is_empty() {
      self.fail(tr(self.lang, "control_center.shortcut_command_required"));
      return;
    }
    let keys = match keybindings::normalize_keys(&draft.keys) {
      Ok(keys) => keys,
      Err(error) => {
        self.shortcut_validation_failed(error);
        return;
      }
    };
    let action = if draft.dispatcher_action {
      let arguments = if draft.arguments.trim().is_empty() {
        None
      } else {
        match serde_json::from_str(&draft.arguments) {
          Ok(arguments) => Some(arguments),
          Err(error) => {
            self.shortcut_validation_failed(error.to_string());
            return;
          }
        }
      };
      keybindings::ShortcutAction::Dispatcher {
        dispatcher: draft.dispatcher.trim().into(),
        arguments,
      }
    } else {
      keybindings::ShortcutAction::Command {
        command: draft.command.clone(),
      }
    };
    let shortcut = keybindings::CustomShortcut {
      description: draft.description.trim().into(),
      keys: keys.clone(),
      enabled: draft.enabled,
      action,
    };
    if let Err(error) = shortcut.validate() {
      self.shortcut_validation_failed(error.to_string());
      return;
    }
    let id = draft.id.clone();
    let conflicts = if shortcut.enabled {
      keybindings::conflicts(&self.keybindings, &id, &keys)
    } else {
      Vec::new()
    };
    let bindings: Vec<_> = self
      .keybindings
      .iter()
      .filter(|binding| conflicts.contains(&binding.id))
      .cloned()
      .collect();
    if bindings.iter().any(|binding| !binding.configurable) {
      self.fail(tr(self.lang, "control_center.shortcut_protected_conflict"));
      return;
    }
    if conflicts
      .iter()
      .any(|id| !draft.accepted_conflicts.contains(id))
    {
      self.keybinding_conflict = Some((id, keys, conflicts));
      self.confirm_focus = ConfirmState::new();
      return;
    }
    self.keybinding_conflict = None;
    let save_id = id.clone();
    self.success(tr(self.lang, "control_center.applying").into());
    self.shortcut_job = Some(ShortcutJob {
      id,
      leave: true,
      deleted: false,
      handle: self.jobs.spawn(move |_| {
        keybindings::persist_custom(&save_id, &shortcut, &bindings)?;
        let apply_error = keybindings::apply_saved().err();
        Ok((keybindings::load(), apply_error))
      }),
    });
  }

  fn shortcut_validation_failed(&mut self, detail: String) {
    argvus_control_center_core::diagnostics::error(&format!("invalid custom shortcut: {detail}"));
    self.fail(tr(self.lang, "control_center.shortcut_invalid"));
  }

  pub(crate) fn cancel_custom_shortcut(&mut self) {
    if self.shortcut_job.is_some() {
      return;
    }
    self.keybinding_capturing = false;
    self.keybinding_conflict = None;
    self.custom_shortcut = ShortcutDraft::default();
    self.navigation.back();
  }

  pub(super) fn delete_custom_shortcut(&mut self, id: String) {
    if self.shortcut_job.is_some() {
      return;
    }
    let delete_id = id.clone();
    self.success(tr(self.lang, "control_center.applying").into());
    self.shortcut_job = Some(ShortcutJob {
      id,
      leave: true,
      deleted: true,
      handle: self.jobs.spawn(move |_| {
        keybindings::delete_custom(&delete_id)?;
        let apply_error = keybindings::apply_saved().err();
        Ok((keybindings::load(), apply_error))
      }),
    });
  }

  pub(super) fn poll_shortcut_job(&mut self) -> bool {
    let Some(job) = self.shortcut_job.take() else {
      return false;
    };
    match job.handle.try_state() {
      JobState::Running => {
        self.shortcut_job = Some(job);
        false
      }
      JobState::Finished(Err(error)) => {
        argvus_control_center_core::diagnostics::error(&format!(
          "shortcut persistence failed: {error}"
        ));
        self.fail(tr(
          self.lang,
          if job.deleted {
            "control_center.shortcut_delete_failed"
          } else {
            "control_center.shortcut_save_failed"
          },
        ));
        true
      }
      JobState::Finished(Ok((bindings, apply_error))) => {
        self.keybindings = bindings;
        if !job.deleted {
          self.custom_shortcut.existing = true;
        }
        if apply_error.is_none() || job.deleted {
          if job.leave && self.page() == Page::CustomShortcut {
            self.navigation.back();
            if self.page() != Page::AllShortcuts {
              self.navigation.push(Page::AllShortcuts);
            }
          }
          self.search.clear();
          self.searching = false;
          let rows = self.rows();
          if let Some(index) = self
            .keybindings
            .iter()
            .position(|binding| binding.id == job.id)
          {
            self
              .navigation
              .current_mut()
              .menu
              .select(&rows, &Item::Keybinding(index));
          } else {
            self.normalize_selection();
          }
        }
        if let Some(error) = apply_error {
          argvus_control_center_core::diagnostics::error(&format!(
            "saved shortcut could not be applied: {error}"
          ));
          self.fail(tr(
            self.lang,
            if job.deleted {
              "control_center.shortcut_delete_apply_failed"
            } else {
              "control_center.shortcut_apply_failed"
            },
          ));
        } else {
          self.success(
            tr(
              self.lang,
              if job.deleted {
                "control_center.shortcut_deleted"
              } else {
                "control_center.keybindings_applied"
              },
            )
            .into(),
          );
        }
        true
      }
    }
  }
}
