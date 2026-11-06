//! User-created global keyboard shortcuts, shared by configuration and settings.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ConfigDocument, ConfigError, ConfigResult};

pub const POINTER: &str = "/hyprland/custom_shortcuts";

/// Lua dispatcher factories supported by the ARGVUS Hyprland configuration API.
/// Arguments use the same JSON object fields as the corresponding Lua options table.
pub const DISPATCHERS: &[&str] = &[
  "dpms",
  "exit",
  "focus",
  "force_idle",
  "global",
  "layout",
  "no_op",
  "pass",
  "release_input_capture",
  "reload_config",
  "send_key_state",
  "send_shortcut",
  "cursor.move",
  "cursor.move_to_corner",
  "group.active",
  "group.lock",
  "group.lock_active",
  "group.move_window",
  "group.next",
  "group.prev",
  "group.toggle",
  "window.alter_zorder",
  "window.bring_to_top",
  "window.center",
  "window.clear_tags",
  "window.close",
  "window.cycle_next",
  "window.deny_from_group",
  "window.float",
  "window.fullscreen",
  "window.fullscreen_state",
  "window.kill",
  "window.move",
  "window.pin",
  "window.pseudo",
  "window.resize",
  "window.set_prop",
  "window.signal",
  "window.swap",
  "window.tag",
  "window.toggle_swallow",
  "workspace.change_id",
  "workspace.move",
  "workspace.rename",
  "workspace.swap_monitors",
  "workspace.toggle_special",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShortcutAction {
  Command {
    command: String,
  },
  Dispatcher {
    dispatcher: String,
    arguments: Option<Value>,
  },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomShortcut {
  pub description: String,
  pub keys: String,
  pub enabled: bool,
  pub action: ShortcutAction,
}

impl CustomShortcut {
  pub fn validate(&self) -> ConfigResult<()> {
    let invalid = |message: &str| ConfigError::Invalid(message.into());
    if self.description.trim().is_empty() || self.description.chars().any(char::is_control) {
      return Err(invalid(
        "shortcut description must be nonempty and contain no control characters",
      ));
    }
    validate_keys(&self.keys)?;
    match &self.action {
      ShortcutAction::Command { command } => {
        if command.trim().is_empty() || command.chars().any(char::is_control) {
          return Err(invalid(
            "shortcut command must be nonempty and contain no control characters",
          ));
        }
      }
      ShortcutAction::Dispatcher {
        dispatcher,
        arguments,
      } => {
        if !DISPATCHERS.contains(&dispatcher.as_str()) {
          return Err(invalid("unsupported shortcut dispatcher"));
        }
        if arguments
          .as_ref()
          .is_some_and(|value| !value.is_object() && !value.is_null())
        {
          return Err(invalid(
            "dispatcher arguments must be a JSON object or null",
          ));
        }
      }
    }
    Ok(())
  }
}

/// Custom shortcuts accept modifiers and one keyboard keysym, never Lua or mouse bindings.
pub fn validate_keys(keys: &str) -> ConfigResult<()> {
  let parts: Vec<_> = keys.split('+').map(str::trim).collect();
  let Some((key, modifiers)) = parts.split_last() else {
    return Err(ConfigError::Invalid("shortcut needs a key".into()));
  };
  let is_modifier = |part: &str| ["SUPER", "CTRL", "ALT", "SHIFT"].contains(&part);
  if key.is_empty()
    || is_modifier(key)
    || !key
      .chars()
      .all(|character| character.is_ascii_alphanumeric() || character == '_')
    || modifiers.iter().any(|modifier| !is_modifier(modifier))
  {
    return Err(ConfigError::Invalid(
      "shortcut needs valid modifiers and one keyboard keysym".into(),
    ));
  }
  Ok(())
}

pub fn from_document(document: &ConfigDocument) -> ConfigResult<BTreeMap<String, CustomShortcut>> {
  let Some(value) = document.get(POINTER) else {
    return Ok(BTreeMap::new());
  };
  let shortcuts: BTreeMap<String, CustomShortcut> = serde_json::from_value(value.clone())?;
  for (id, shortcut) in &shortcuts {
    if !id.starts_with("custom_")
      || id.len() <= 7
      || !id.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
      })
    {
      return Err(ConfigError::Invalid(
        "custom shortcut ID must use the custom_ namespace".into(),
      ));
    }
    shortcut.validate()?;
  }
  Ok(shortcuts)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn validates_custom_shortcuts_without_changing_official_overrides() {
    let mut document = ConfigDocument::default();
    document
      .set("/keyboard_shortcuts/open_browser", json!("SUPER + B"))
      .unwrap();
    assert!(from_document(&document).unwrap().is_empty());
    document
      .set(
        POINTER,
        json!({"custom_example": {
          "description": "Open editor", "keys": "SUPER + E", "enabled": true,
          "action": {"type": "command", "command": "editor 'a b'"}
        }}),
      )
      .unwrap();
    assert_eq!(from_document(&document).unwrap().len(), 1);
    assert_eq!(
      document.get("/keyboard_shortcuts/open_browser"),
      Some(&json!("SUPER + B"))
    );
    document
      .set(
        "/hyprland/custom_shortcuts/custom_example/action",
        json!({
          "type": "dispatcher", "dispatcher": "window.float", "arguments": {"action": "toggle"}
        }),
      )
      .unwrap();
    assert!(from_document(&document).is_ok());
  }

  #[test]
  fn rejects_invalid_fields_keys_and_dispatchers() {
    let mut shortcut = CustomShortcut {
      description: "Example".into(),
      keys: "SUPER + E".into(),
      enabled: true,
      action: ShortcutAction::Command {
        command: "editor".into(),
      },
    };
    for invalid in [
      "",
      "SUPER",
      "UNKNOWN + E",
      "SUPER + mouse:272",
      "E\nerror()",
    ] {
      shortcut.keys = invalid.into();
      assert!(shortcut.validate().is_err(), "{invalid}");
    }
    shortcut.keys = "SUPER + E".into();
    shortcut.action = ShortcutAction::Dispatcher {
      dispatcher: "arbitrary_code()".into(),
      arguments: None,
    };
    assert!(shortcut.validate().is_err());
    shortcut.action = ShortcutAction::Dispatcher {
      dispatcher: "window.close".into(),
      arguments: Some(json!("not an object")),
    };
    assert!(shortcut.validate().is_err());
  }
}

#[cfg(test)]
mod persistence_tests {
  use super::*;
  use crate::ConfigStore;
  use serde_json::json;

  #[test]
  fn create_edit_disable_and_delete_survive_reopening_the_modular_store() {
    let directory = std::env::temp_dir()
      .join("argvus")
      .join(format!("custom-shortcuts-store-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let store = ConfigStore::at(&directory);
    let pointer = "/hyprland/custom_shortcuts/custom_example";
    let shortcut = json!({"description":"Example", "keys":"SUPER + CTRL + E", "enabled":true,
      "action":{"type":"command", "command":"editor 'a b'"}});
    let mut patch = serde_json::Map::new();
    patch.insert(pointer.into(), shortcut);
    patch.insert("/keyboard_shortcuts/open_browser".into(), Value::Null);
    store.patch(&patch).unwrap();
    let reopened = ConfigStore::at(&directory);
    assert!(from_document(&reopened.load().unwrap()).unwrap()["custom_example"].enabled);
    assert_eq!(
      reopened
        .load()
        .unwrap()
        .get("/keyboard_shortcuts/open_browser"),
      Some(&Value::Null)
    );
    reopened
      .update(&format!("{pointer}/description"), json!("Renamed"))
      .unwrap();
    reopened
      .update(&format!("{pointer}/enabled"), json!(false))
      .unwrap();
    reopened
      .update("/keyboard_shortcuts/open_browser", json!("SUPER + B"))
      .unwrap();
    let saved = from_document(&reopened.load().unwrap()).unwrap();
    assert_eq!(saved["custom_example"].description, "Renamed");
    assert!(!saved["custom_example"].enabled);
    assert_eq!(
      saved.len(),
      1,
      "restoring an official shortcut preserves custom records"
    );
    let before = reopened.load().unwrap().get(POINTER).cloned();
    assert!(
      reopened
        .update(&format!("{pointer}/keys"), json!("SUPER"))
        .is_err()
    );
    assert_eq!(reopened.load().unwrap().get(POINTER), before.as_ref());
    reopened.unset(pointer).unwrap();
    assert!(from_document(&reopened.load().unwrap()).unwrap().is_empty());
    std::fs::remove_dir_all(directory).unwrap();
  }
}
