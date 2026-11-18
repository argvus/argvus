//! Official and effective desktop keyboard shortcuts.
use crate::custom_shortcuts::{CustomShortcut, ShortcutAction};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Binding {
  pub id: String,
  #[serde(skip)]
  pub custom: Option<CustomShortcut>,
  pub category: String,
  pub description_key: String,
  pub keys: String,
  pub action: String,
  #[serde(default)]
  pub context: String,
  #[serde(default)]
  pub input: String,
  #[serde(default = "default_true")]
  pub configurable: bool,
  #[serde(default)]
  pub flags: Vec<String>,
  #[serde(default)]
  pub enabled: bool,
}
fn default_true() -> bool {
  true
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Manifest {
  version: u32,
  bindings: Vec<Binding>,
}
pub fn manifest_path() -> PathBuf {
  std::env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
    .join("hyprland/keybindings.json")
}
pub fn custom_binding(id: String, mut shortcut: CustomShortcut) -> Binding {
  shortcut.keys = normalize_keys(&shortcut.keys).unwrap_or(shortcut.keys);
  Binding {
    id,
    category: "custom".into(),
    description_key: shortcut.description.clone(),
    keys: shortcut.keys.clone(),
    enabled: shortcut.enabled,
    action: match &shortcut.action {
      ShortcutAction::Command { command } => command.clone(),
      ShortcutAction::Dispatcher {
        dispatcher,
        arguments,
      } => format!(
        "{dispatcher} {}",
        arguments
          .as_ref()
          .map(ToString::to_string)
          .unwrap_or_default()
      ),
    },
    context: String::new(),
    input: String::new(),
    configurable: true,
    flags: Vec::new(),
    custom: Some(shortcut),
  }
}

pub fn normalize_keys(value: &str) -> Result<String, String> {
  let mut parts: Vec<String> = value
    .split('+')
    .map(str::trim)
    .filter(|v| !v.is_empty())
    .map(|v| match v.to_ascii_lowercase().as_str() {
      "super" | "win" | "meta" => "SUPER".into(),
      "ctrl" | "control" => "CTRL".into(),
      "alt" => "ALT".into(),
      "shift" => "SHIFT".into(),
      "enter" | "return" => "Return".into(),
      "esc" | "escape" => "Escape".into(),
      "space" => "Space".into(),
      "tab" => "Tab".into(),
      "left" | "leftarrow" => "left".into(),
      "right" | "rightarrow" => "right".into(),
      "up" | "uparrow" => "up".into(),
      "down" | "downarrow" => "down".into(),
      "pageup" | "page_up" => "Page_Up".into(),
      "pagedown" | "page_down" => "Page_Down".into(),
      "backspace" => "BackSpace".into(),
      "delete" => "Delete".into(),
      "insert" => "Insert".into(),
      "home" => "Home".into(),
      "end" => "End".into(),
      "/" | "slash" => "slash".into(),
      "?" | "question" => "question".into(),
      "-" | "minus" => "minus".into(),
      "." | "period" => "period".into(),
      "," | "comma" => "comma".into(),
      ";" | "semicolon" => "semicolon".into(),
      "'" | "apostrophe" => "apostrophe".into(),
      "[" | "bracketleft" => "bracketleft".into(),
      "]" | "bracketright" => "bracketright".into(),
      "\\" | "backslash" => "backslash".into(),
      "`" | "grave" => "grave".into(),
      "=" | "equal" => "equal".into(),
      "print" => "Print".into(),
      "xf86audioraisevolume" => "XF86AudioRaiseVolume".into(),
      "xf86audiolowervolume" => "XF86AudioLowerVolume".into(),
      "xf86audiomute" => "XF86AudioMute".into(),
      "xf86audionext" => "XF86AudioNext".into(),
      "xf86audioplay" => "XF86AudioPlay".into(),
      "xf86audioprev" => "XF86AudioPrev".into(),
      "xf86audiostop" => "XF86AudioStop".into(),
      "xf86monbrightnessdown" => "XF86MonBrightnessDown".into(),
      "xf86monbrightnessup" => "XF86MonBrightnessUp".into(),
      v => {
        if v.starts_with("mouse:") || v.starts_with("XF86") || v == "Print" {
          v.to_string()
        } else if v.len() == 1
          || (v.len() >= 2
            && v.starts_with('f')
            && v[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)))
        {
          v.to_ascii_uppercase()
        } else {
          v.to_string()
        }
      }
    })
    .collect();
  let key = parts.pop().ok_or("shortcut needs a key")?;
  if ["CTRL", "ALT", "SHIFT", "SUPER"].contains(&key.as_str()) {
    return Err("shortcut needs a non-modifier key".into());
  }
  let mut mods = parts;
  mods.sort_by_key(|v| {
    ["SUPER", "CTRL", "ALT", "SHIFT"]
      .iter()
      .position(|m| m == v)
      .unwrap_or(99)
  });
  mods.dedup();
  Ok(if mods.is_empty() {
    key
  } else {
    format!("{} + {key}", mods.join(" + "))
  })
}

pub fn display_key(key: &str) -> String {
  match key {
    "slash" => "/".into(),
    "question" => "?".into(),
    "minus" => "-".into(),
    "period" => ".".into(),
    "comma" => ",".into(),
    "semicolon" => ";".into(),
    "apostrophe" => "'".into(),
    "bracketleft" => "[".into(),
    "bracketright" => "]".into(),
    "backslash" => "\\".into(),
    "grave" => "`".into(),
    "equal" => "=".into(),
    "left" => "Left Arrow".into(),
    "right" => "Right Arrow".into(),
    "up" => "Up Arrow".into(),
    "down" => "Down Arrow".into(),
    "Return" => "Enter".into(),
    "Escape" => "Escape".into(),
    "Page_Up" => "Page Up".into(),
    "Page_Down" => "Page Down".into(),
    "BackSpace" => "Backspace".into(),
    "XF86AudioLowerVolume" => "Volume Down".into(),
    "XF86AudioMute" => "Mute".into(),
    "XF86AudioNext" => "Next Track".into(),
    "XF86AudioPlay" => "Play/Pause".into(),
    "XF86AudioPrev" => "Previous Track".into(),
    "XF86AudioStop" => "Stop".into(),
    "XF86MonBrightnessDown" => "Brightness Down".into(),
    "XF86MonBrightnessUp" => "Brightness Up".into(),
    other => other.into(),
  }
}
pub fn display_keys(keys: &str) -> String {
  let mut modifiers = Vec::new();
  let mut key = "";
  for part in keys.split(" + ") {
    match part.to_ascii_uppercase().as_str() {
      "SUPER" | "CTRL" | "ALT" | "SHIFT" => modifiers.push(part.to_ascii_uppercase()),
      _ => key = part,
    }
  }
  modifiers.sort_by_key(|part| {
    ["SUPER", "CTRL", "ALT", "SHIFT"]
      .iter()
      .position(|m| m == part)
      .unwrap_or(99)
  });
  let mut output = modifiers
    .into_iter()
    .map(|part| {
      if part == "SHIFT" {
        "Shift".into()
      } else {
        part
      }
    })
    .collect::<Vec<String>>();
  if !key.is_empty() {
    output.push(display_key(key));
  }
  output.join(" + ")
}
pub fn defaults() -> Result<Vec<Binding>, String> {
  let path = manifest_path();
  let raw = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
  serde_json::from_str::<Manifest>(&raw)
    .map(|manifest| manifest.bindings)
    .map_err(|error| format!("{}: {error}", path.display()))
}

pub fn load() -> Result<Vec<Binding>, String> {
  let document = crate::ConfigStore::from_environment()
    .and_then(|store| store.load_effective())
    .map_err(|error| error.to_string())?;
  let canonical = document.get("/keyboard_shortcuts");
  let mut bindings = defaults()?;
  for binding in &mut bindings {
    binding.keys = normalize_keys(&binding.keys).unwrap_or_else(|_| binding.keys.clone());
    match canonical.and_then(|value| value.get(crate::config_key_for_binding(&binding.id))) {
      Some(serde_json::Value::String(keys)) => {
        binding.keys = normalize_keys(keys).unwrap_or_else(|_| binding.keys.clone());
        binding.enabled = true;
      }
      Some(serde_json::Value::Null) => binding.enabled = false,
      _ => binding.enabled = true,
    }
  }
  if let Some(custom) = document.get("/hyprland/custom_shortcuts") {
    let shortcuts =
      serde_json::from_value::<std::collections::BTreeMap<String, CustomShortcut>>(custom.clone())
        .map_err(|error| error.to_string())?;
    bindings.extend(
      shortcuts
        .into_iter()
        .map(|(id, shortcut)| custom_binding(id, shortcut)),
    );
  }
  Ok(bindings)
}
