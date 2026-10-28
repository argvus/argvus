//! Window rules page: editing, adding and applying rules.

use super::*;

impl App {
  /// Reloads the saved rules and keeps the drafts whose rule still exists.
  pub(crate) fn reload_window_rules(&mut self) {
    self.window_rules = window_rules::load();
    let rules = &self.window_rules;
    self
      .window_rule_drafts
      .retain(|name, _| rules.iter().any(|rule| &rule.name == name));
  }

  /// Whether a rule differs from its saved state. Only drafts can differ.
  pub(crate) fn window_rule_changed(&self, rule: &window_rules::WindowRule) -> bool {
    self
      .window_rule_drafts
      .get(&rule.name)
      .is_some_and(|draft| draft != rule)
  }

  /// Changes the draft of a saved rule; nothing reaches the config until Apply.
  pub(super) fn edit_window_rule(
    &mut self,
    index: usize,
    edit: impl FnOnce(&mut window_rules::WindowRule),
  ) {
    let Some(loaded) = self.window_rules.get(index).cloned() else {
      return;
    };
    let draft = self
      .window_rule_drafts
      .entry(loaded.name.clone())
      .or_insert_with(|| loaded.clone());
    edit(draft);
  }

  pub(super) fn adjust_window_rule(&mut self, index: usize, delta: i32) {
    self.edit_window_rule(index, |rule| {
      rule.workspace = window_rules::cycle_workspace(rule.workspace, delta);
    });
  }

  /// Opens the name editor of a saved rule, showing its draft name.
  pub(super) fn open_window_rule_name(&mut self, index: usize) {
    let Some(loaded) = self.window_rules.get(index) else {
      return;
    };
    let draft = self.window_rule_drafts.get(&loaded.name).unwrap_or(loaded);
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.window_rules_name").into(),
      draft.name.clone(),
      EditTarget::WindowRuleName(loaded.name.clone()),
      false,
    ));
  }

  pub(super) fn open_window_rule_classes(&mut self, index: usize) {
    let Some(loaded) = self.window_rules.get(index) else {
      return;
    };
    let draft = self.window_rule_drafts.get(&loaded.name).unwrap_or(loaded);
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.window_rules_classes").into(),
      draft.classes.join(", "),
      EditTarget::WindowRuleClasses(loaded.name.clone()),
      false,
    ));
  }

  pub(super) fn open_new_window_rule_name(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.window_rules_name").into(),
      self.new_window_rule_name.clone(),
      EditTarget::NewWindowRuleName,
      false,
    ));
  }

  /// Saves the draft of one rule. A new name renames the rule.
  pub(super) fn apply_window_rule(&mut self, index: usize) {
    let Some(loaded) = self.window_rules.get(index).cloned() else {
      return;
    };
    let Some(draft) = self.window_rule_drafts.get(&loaded.name).cloned() else {
      return;
    };
    if let Err(message) = self.check_window_rule_name(&draft.name, Some(loaded.name.as_str())) {
      self.fail(message);
      return;
    }
    match window_rules::save(Some(loaded.name.as_str()), &draft) {
      Ok(()) => {
        self.window_rule_drafts.remove(&loaded.name);
        self.reload_window_rules();
        self.success(tr(self.lang, "control_center.window_rules_applied").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  /// Adds a rule named by the typed name, or by the next free `rule-N` when empty.
  pub(super) fn add_window_rule(&mut self) {
    let typed = self.new_window_rule_name.trim().to_owned();
    let name = if typed.is_empty() {
      window_rules::next_rule_name(&self.window_rules)
    } else {
      typed
    };
    if let Err(message) = self.check_window_rule_name(&name, None) {
      self.fail(message);
      return;
    }
    let rule = window_rules::WindowRule {
      name,
      workspace: 1,
      classes: Vec::new(),
    };
    match window_rules::save(None, &rule) {
      Ok(()) => {
        self.new_window_rule_name.clear();
        self.reload_window_rules();
        self.success(tr(self.lang, "control_center.window_rules_saved").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  /// Validates a rule name. `current` is the saved name of the rule being
  /// changed, which may keep its own name.
  pub(crate) fn check_window_rule_name(
    &self,
    name: &str,
    current: Option<&str>,
  ) -> Result<(), String> {
    if !window_rules::is_valid_name(name) {
      return Err(tr(self.lang, "control_center.window_rules_invalid_name").into());
    }
    let taken = self
      .window_rules
      .iter()
      .any(|rule| rule.name == name && Some(rule.name.as_str()) != current);
    if taken {
      return Err(tr(self.lang, "control_center.window_rules_name_in_use").into());
    }
    Ok(())
  }
}
