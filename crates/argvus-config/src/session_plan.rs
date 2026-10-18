//! Reload decision for a configuration transaction.
//!
//! The projection manifest records which runtime components must reload
//! after a configuration change. `argvus-sessionctl` used to parse that file
//! with `grep`; this module makes the decision once, in Rust, and the session
//! scripts only execute it.

use serde_json::Value;

/// Plan used when the manifest is missing or does not say what to reload.
pub const FULL_PLAN: &str = "full";

/// What a configuration transaction must reload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReloadDecision {
  /// The manifest names the components to restart or reload, sorted.
  Targets(Vec<String>),
  /// Older manifests only carry a named plan (`full`, `hyprland`, ...).
  Plan(String),
}

/// Decides the reload from a parsed projection manifest.
///
/// When `reload_targets` is present it is authoritative, even if empty.
/// Otherwise the `reload_plan` string decides, and a missing or empty plan
/// falls back to [`FULL_PLAN`].
pub fn decide_reload(manifest: &Value) -> ReloadDecision {
  if let Some(targets) = manifest.get("reload_targets").and_then(Value::as_object) {
    // Sort explicitly: the output must not depend on the map's iteration order.
    let mut enabled: Vec<String> = targets
      .iter()
      .filter(|(_, enabled)| enabled.as_bool() == Some(true))
      .map(|(name, _)| name.clone())
      .collect();
    enabled.sort();
    return ReloadDecision::Targets(enabled);
  }
  let plan = manifest
    .get("reload_plan")
    .and_then(Value::as_str)
    .filter(|plan| !plan.is_empty())
    .unwrap_or(FULL_PLAN);
  ReloadDecision::Plan(plan.to_string())
}

/// Renders the decision for the shell: the first line names the mode
/// (`targets` or `plan`), and the following lines carry the names or the plan.
pub fn render(decision: &ReloadDecision) -> String {
  match decision {
    ReloadDecision::Targets(names) => {
      let mut output = String::from("targets\n");
      for name in names {
        output.push_str(name);
        output.push('\n');
      }
      output
    }
    ReloadDecision::Plan(plan) => format!("plan\n{plan}\n"),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn targets_are_the_enabled_entries_sorted() {
    let manifest = json!({
        "reload_targets": {
            "waybar": true,
            "hyprland": true,
            "polkit": false,
            "wallpaper": true
        },
        "reload_plan": "full"
    });
    assert_eq!(
      decide_reload(&manifest),
      ReloadDecision::Targets(vec!["hyprland".into(), "wallpaper".into(), "waybar".into()])
    );
  }

  #[test]
  fn empty_targets_are_authoritative_over_the_plan() {
    let manifest = json!({ "reload_targets": {}, "reload_plan": "full" });
    assert_eq!(decide_reload(&manifest), ReloadDecision::Targets(vec![]));
  }

  #[test]
  fn plan_is_used_when_targets_are_absent() {
    let manifest = json!({ "reload_plan": "hyprland" });
    assert_eq!(
      decide_reload(&manifest),
      ReloadDecision::Plan("hyprland".into())
    );
  }

  #[test]
  fn missing_or_empty_plan_falls_back_to_full() {
    assert_eq!(
      decide_reload(&json!({})),
      ReloadDecision::Plan(FULL_PLAN.into())
    );
    assert_eq!(
      decide_reload(&json!({ "reload_plan": "" })),
      ReloadDecision::Plan(FULL_PLAN.into())
    );
  }

  #[test]
  fn render_names_the_mode_then_the_values() {
    assert_eq!(
      render(&ReloadDecision::Targets(vec!["hyprland".into()])),
      "targets\nhyprland\n"
    );
    assert_eq!(render(&ReloadDecision::Targets(vec![])), "targets\n");
    assert_eq!(render(&ReloadDecision::Plan("none".into())), "plan\nnone\n");
  }
}
