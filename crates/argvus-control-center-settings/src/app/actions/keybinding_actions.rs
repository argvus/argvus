//! Keyboard shortcut pages: the visible bindings and editing, disabling and restoring one.

use super::*;

impl App {
  /// Indexes of the shortcuts shown on the Keyboard Shortcuts page, in
  /// display order and narrowed by the search.
  pub(in super::super) fn visible_keybindings(&self) -> Vec<usize> {
    let mut bindings: Vec<usize> = self
      .keybindings
      .iter()
      .enumerate()
      .filter(|(_, binding)| keybinding_is_cheatsheet_entry(binding))
      .filter(|(_, binding)| {
        search_matches(
          &self.search,
          &[
            &binding.action,
            &binding.id,
            &binding.category,
            &binding.keys,
            &keybindings::display_keys(&binding.keys),
            &binding.description_key,
            &keybinding_label(self.lang, binding),
            &keybinding_section(self.lang, binding),
          ],
        )
      })
      .map(|(index, _)| index)
      .collect();
    bindings.sort_by_key(|index| {
      let binding = &self.keybindings[*index];
      (
        keybinding_section_order(binding),
        keybinding_category_order(&binding.category),
        binding.id.clone(),
      )
    });
    bindings
  }

  /// The shortcut under the cursor on the Keyboard Shortcuts page.
  pub(in super::super) fn selected_keybinding(&self) -> Option<&keybindings::Binding> {
    match self.selected_item() {
      Some(Item::Keybinding(index)) => self.keybindings.get(index),
      _ => None,
    }
  }

  pub(super) fn open_keybinding_edit(&mut self, index: usize) {
    if let Some(shortcut) = self
      .keybindings
      .get(index)
      .filter(|binding| binding.custom.is_some())
      .cloned()
    {
      self.open_custom_shortcut(Some(&shortcut));
      self.navigation.push(Page::CustomShortcut);
      return;
    }
    if self
      .keybindings
      .get(index)
      .is_some_and(|binding| !binding.configurable)
    {
      self.fail(tr(self.lang, "control_center.shortcut_protected_conflict"));
      return;
    }
    let Some(id) = self
      .keybindings
      .get(index)
      .map(|binding| binding.id.clone())
    else {
      return;
    };
    self.keybinding_edit_id = Some(id);
    self.navigation.push(Page::KeybindingEdit);
    self.begin_keybinding_capture();
    self.keybinding_capturing = false;
    self.status = None;
  }

  /// "Disable" of the edit page: turns the edited shortcut off.
  pub(super) fn disable_edited_keybinding(&mut self) {
    let Some(binding) = self.edited_binding() else {
      return;
    };
    let (id, keys) = (binding.id.clone(), binding.keys.clone());
    if let Err(error) = keybindings::save_override(&id, Some(keys), Some(false), &self.keybindings)
    {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.success(tr(self.lang, "control_center.keybindings_applied").to_string());
  }

  /// Space on the Keyboard Shortcuts list: enables or disables the
  /// shortcut under the cursor.
  pub(crate) fn toggle_selected_keybinding(&mut self) {
    if self.shortcut_job.is_some() {
      return;
    }
    let Some(binding) = self.selected_keybinding().cloned() else {
      return;
    };
    if !binding.configurable {
      return;
    }
    if !binding.enabled {
      let conflicts = keybindings::conflicts_in_context(
        &self.keybindings,
        &binding.id,
        &binding.keys,
        &binding.context,
        &binding.input,
      );
      if self
        .keybindings
        .iter()
        .any(|binding| conflicts.contains(&binding.id) && !binding.configurable)
      {
        self.fail(tr(self.lang, "control_center.shortcut_protected_conflict"));
        return;
      }
      if !conflicts.is_empty() && binding.custom.is_none() {
        self.keybinding_conflict = Some((binding.id.clone(), binding.keys.clone(), conflicts));
        self.confirm_focus = super::super::ConfirmState::new();
        return;
      }
    }
    if let Some(mut shortcut) = binding.custom.clone() {
      shortcut.enabled = !binding.enabled;
      if shortcut.enabled
        && !keybindings::conflicts(&self.keybindings, &binding.id, &binding.keys).is_empty()
      {
        self.open_custom_shortcut(Some(&binding));
        self.custom_shortcut.enabled = true;
        self.navigation.push(Page::CustomShortcut);
        self.save_custom_shortcut();
        return;
      }
      let id = binding.id.clone();
      let save_id = id.clone();
      self.shortcut_job = Some(super::super::custom_shortcut::ShortcutJob {
        id,
        leave: false,
        deleted: false,
        handle: self.jobs.spawn(move |_| {
          keybindings::persist_custom(&save_id, &shortcut, &[])?;
          let apply_error = keybindings::apply_saved().err();
          Ok((keybindings::load(), apply_error))
        }),
      });
      return;
    }
    let (id, keys, enabled) = (binding.id.clone(), binding.keys.clone(), binding.enabled);
    if let Err(error) =
      keybindings::save_override(&id, Some(keys), Some(!enabled), &self.keybindings)
    {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.success(tr(self.lang, "control_center.keybindings_applied").to_string());
  }

  /// "Restore default" of the edit page: immediate, as before.
  pub(super) fn restore_edited_keybinding(&mut self) {
    if let Some(id) = self.keybinding_edit_id.clone() {
      self.restore_keybinding(&id);
    }
  }

  /// Restores one shortcut to its ARGVUS default.
  pub(in super::super) fn restore_keybinding(&mut self, id: &str) {
    if keybindings::restoration_conflicts(&self.keybindings, Some(id)) {
      self.fail(tr(self.lang, "control_center.shortcut_restore_conflict"));
      return;
    }
    if let Err(error) = keybindings::restore(id, &self.keybindings) {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.success(tr(self.lang, "control_center.keybindings_restored").to_string());
  }
}
