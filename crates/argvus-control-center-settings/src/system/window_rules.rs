use argvus_control_center_core::process::command;
use serde_json::Value;

const POINTER_ROOT: &str = "/hyprland/window_rules";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowRule {
  pub name: String,
  pub workspace: u64,
  pub classes: Vec<String>,
}

/// Loads the effective window placement rules from argvus-config.
pub fn load() -> Vec<WindowRule> {
  let Ok(output) = command("argvus")
    .arg("config")
    .args(["get", POINTER_ROOT, "--effective"])
    .output()
  else {
    return Vec::new();
  };
  if !output.status.success() {
    return Vec::new();
  }
  let Some(rules) = serde_json::from_slice::<Value>(&output.stdout)
    .ok()
    .and_then(|value| value.as_object().cloned())
  else {
    return Vec::new();
  };
  rules
    .iter()
    .map(|(name, rule)| WindowRule {
      name: name.clone(),
      workspace: rule.get("workspace").and_then(Value::as_u64).unwrap_or(1),
      classes: rule
        .get("classes")
        .and_then(Value::as_array)
        .map(|items| {
          items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_owned))
            .collect()
        })
        .unwrap_or_default(),
    })
    .collect()
}

/// Steps the workspace of a rule through 1..=10, wrapping around.
pub fn cycle_workspace(current: u64, delta: i32) -> u64 {
  let zero_based = (current as i64 - 1 + i64::from(delta)).rem_euclid(10);
  zero_based as u64 + 1
}

/// Returns the first `rule-N` name that no existing rule uses.
pub fn next_rule_name(rules: &[WindowRule]) -> String {
  let mut number = rules.len() + 1;
  while rules
    .iter()
    .any(|rule| rule.name == format!("rule-{number}"))
  {
    number += 1;
  }
  format!("rule-{number}")
}

/// A rule name becomes a config path segment, so only plain identifiers are accepted.
pub fn is_valid_name(name: &str) -> bool {
  !name.is_empty()
    && name
      .chars()
      .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
}

/// Writes a rule under its name, renaming it when `original` differs, then reloads once.
/// `None` creates a new rule.
pub fn save(original: Option<&str>, rule: &WindowRule) -> Result<(), String> {
  let value = serde_json::json!({ "workspace": rule.workspace, "classes": rule.classes });
  write_value(&format!("{POINTER_ROOT}/{}", rule.name), &value.to_string())?;
  if let Some(original) = original.filter(|original| *original != rule.name) {
    unset_value(&format!("{POINTER_ROOT}/{original}"))?;
  }
  apply()
}

pub fn remove(name: &str) -> Result<(), String> {
  unset_value(&format!("{POINTER_ROOT}/{name}"))?;
  apply()
}

fn write_value(pointer: &str, value: &str) -> Result<(), String> {
  let status = command("argvus")
    .arg("config")
    .args(["set", pointer, value])
    .status()
    .map_err(|error| format!("failed to save window rule: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the window rule".into());
  }
  Ok(())
}

fn unset_value(pointer: &str) -> Result<(), String> {
  let status = command("argvus")
    .arg("config")
    .args(["unset", pointer])
    .status()
    .map_err(|error| format!("failed to remove window rule: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the window rule removal".into());
  }
  Ok(())
}

fn apply() -> Result<(), String> {
  argvus_control_center_core::config::reload_argvus_config_service()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cycles_workspaces_through_one_to_ten() {
    assert_eq!(cycle_workspace(1, -1), 10);
    assert_eq!(cycle_workspace(10, 1), 1);
    assert_eq!(cycle_workspace(2, 1), 3);
  }

  #[test]
  fn picks_unused_rule_names() {
    let rules = vec![WindowRule {
      name: "rule-2".into(),
      workspace: 1,
      classes: Vec::new(),
    }];
    assert_eq!(next_rule_name(&rules), "rule-3");
  }

  #[test]
  fn accepts_only_plain_rule_names() {
    assert!(is_valid_name("browser"));
    assert!(is_valid_name("ide-2_main"));
    assert!(!is_valid_name(""));
    assert!(!is_valid_name("has space"));
    assert!(!is_valid_name("a/b"));
  }
}
