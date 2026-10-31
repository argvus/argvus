//! Keyboard-layout discovery and application of the Settings pages.
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::error::SettingsError;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
  pub code: String,
  pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
  pub code: String,
  pub layout: String,
  pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct KeyboardInfo {
  /// The layout in use right now, which the taskbar and shortcuts change.
  pub x11_layout: String,
  /// The first configured layout: the one Hyprland starts with and the one the
  /// variant belongs to. It only changes when the layout list is applied.
  pub default_layout: String,
  pub x11_variant: String,
  pub x11_model: String,
  pub x11_options: String,
  pub console_keymap: String,
  pub hypr_layout: String,
  pub hypr_variant: String,
  pub hypr_options: String,
}

pub fn info() -> KeyboardInfo {
  let mut info =
    parse_localectl_status(&command_stdout("localectl", &["status"]).unwrap_or_default());
  let hypr = parse_hypr_keyboard_config();
  info.hypr_layout = hypr.0;
  info.hypr_variant = hypr.1;
  info.hypr_options = hypr.2;
  // Keyboard changes are session-scoped and intentionally do not update the
  // privileged localectl database.  The generated Hyprland input is therefore
  // the authoritative state shown by this page.
  if !info.hypr_layout.is_empty() {
    info.default_layout = first_csv_value(&info.hypr_layout);
    info.x11_layout = active_layout(&info.hypr_layout);
  }
  if !info.hypr_variant.is_empty() {
    info.x11_variant = first_csv_value(&info.hypr_variant);
  } else if !info.hypr_layout.is_empty() {
    info.x11_variant.clear();
  }
  info
}

/// The layout in use right now. The Control Center polls it so the page follows
/// changes made elsewhere, such as the taskbar or a keyboard shortcut.
pub fn current_layout() -> String {
  let (configured, _, _) = parse_hypr_keyboard_config();
  active_layout(&configured)
}

pub fn layouts() -> Vec<Layout> {
  parse_xkb_layouts(&read_xkb_rules().unwrap_or_default())
}

pub fn variants(layout: &str) -> Vec<Variant> {
  parse_xkb_variants(&read_xkb_rules().unwrap_or_default(), layout)
}

pub fn console_keymaps() -> Vec<String> {
  command_stdout("localectl", &["list-keymaps"])
    .unwrap_or_default()
    .lines()
    .map(str::trim)
    .filter(|line| is_keymap_name(line))
    .map(str::to_string)
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect()
}

pub fn set_x11_layout(layout: &str, valid: &[Layout]) -> Result<(), SettingsError> {
  if !valid.iter().any(|candidate| candidate.code == layout) {
    return Err(SettingsError::System(format!(
      "invalid keyboard layout: {layout}"
    )));
  }
  let layouts = merged_layout_list(&parse_hypr_keyboard_config().0, layout);
  let layouts = layouts.split(',').map(str::to_string).collect::<Vec<_>>();
  set_x11_layouts(layout, &layouts, valid)
}

pub fn set_x11_layouts(
  default_layout: &str,
  layouts: &[String],
  valid: &[Layout],
) -> Result<(), SettingsError> {
  if layouts.is_empty() || !layouts.iter().any(|layout| layout == default_layout) {
    return Err(SettingsError::System(
      "at least one keyboard layout must be selected".to_string(),
    ));
  }
  if layouts
    .iter()
    .any(|layout| !valid.iter().any(|candidate| candidate.code == *layout))
  {
    return Err(SettingsError::System(
      "invalid selected keyboard layout".to_string(),
    ));
  }
  let mut ordered = vec![default_layout.to_string()];
  ordered.extend(
    layouts
      .iter()
      .filter(|layout| layout.as_str() != default_layout)
      .cloned(),
  );
  let layouts = ordered.join(",");
  // The variant belongs to the default layout. Reordering or adding layouts
  // keeps it; changing the default drops it, because br/abnt2 selected as us
  // makes Hyprland reject the complete config.
  let (current_layouts, current_variant, _) = parse_hypr_keyboard_config();
  let variant = resolved_variant(
    default_layout,
    &variant_after_default_change(&current_layouts, &current_variant, default_layout),
  );
  // The canonical write also reloads argvus-config, which projects the layout
  // and reloads Hyprland. `hyprctl keyword` is rejected by Hyprland 0.56.
  write_generated_hypr_input(Some(&layouts), Some(&variant))?;
  switch_active_layout(default_layout, &layouts)?;
  Ok(())
}

pub fn set_x11_variant(
  layout: &str,
  variant: &str,
  valid: &[Variant],
) -> Result<(), SettingsError> {
  if !variant.is_empty() && !valid.iter().any(|candidate| candidate.code == variant) {
    return Err(SettingsError::System(format!(
      "invalid keyboard variant: {variant}"
    )));
  }
  // Preserve the complete cycle list while changing only the variant of the
  // currently selected layout.
  let layouts = parse_hypr_keyboard_config().0;
  write_generated_hypr_input(None, Some(&resolved_variant(layout, variant)))?;
  switch_active_layout(layout, &layouts)?;
  Ok(())
}

pub fn set_console_keymap(keymap: &str, valid: &[String]) -> Result<(), SettingsError> {
  if !valid.iter().any(|candidate| candidate == keymap) {
    return Err(SettingsError::System(format!(
      "invalid console keymap: {keymap}"
    )));
  }
  super::privileged::run(&["keyboard", "console-keymap", keymap]).map(|_| ())
}

pub fn parse_localectl_status(contents: &str) -> KeyboardInfo {
  let mut info = KeyboardInfo::default();
  for line in contents.lines().map(str::trim) {
    let Some((key, value)) = line.split_once(':') else {
      continue;
    };
    let value = value.trim().to_string();
    match key {
      "VC Keymap" => info.console_keymap = value,
      "X11 Layout" => info.x11_layout = value,
      "X11 Model" => info.x11_model = value,
      "X11 Variant" => info.x11_variant = value,
      "X11 Options" => info.x11_options = value,
      _ => {}
    }
  }
  info
}

pub fn parse_xkb_layouts(contents: &str) -> Vec<Layout> {
  let mut in_layout = false;
  let mut out = Vec::new();
  for line in contents.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with('!') {
      in_layout = trimmed == "! layout";
      continue;
    }
    if !in_layout || trimmed.is_empty() {
      continue;
    }
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let Some(code) = parts.next() else {
      continue;
    };
    let description = parts.next().unwrap_or("").trim();
    if is_xkb_name(code) {
      out.push(Layout {
        code: code.to_string(),
        description: description.to_string(),
      });
    }
  }
  out
}

pub fn parse_xkb_variants(contents: &str, layout: &str) -> Vec<Variant> {
  let mut in_variant = false;
  let mut out = vec![Variant {
    code: String::new(),
    layout: layout.to_string(),
    description: "Default".to_string(),
  }];
  // XKB omits the default symbols section from the variants list. Expose
  // Brazil's ABNT2 explicitly so selecting it targets the session keyboard.
  if layout == "br" {
    out.push(Variant {
      code: "abnt2".to_string(),
      layout: layout.to_string(),
      description: "ABNT2".to_string(),
    });
  }
  for line in contents.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with('!') {
      in_variant = trimmed == "! variant";
      continue;
    }
    if !in_variant || trimmed.is_empty() {
      continue;
    }
    let Some((left, description)) = trimmed.split_once(':') else {
      continue;
    };
    let mut parts = left.split_whitespace();
    let Some(code) = parts.next() else {
      continue;
    };
    let Some(entry_layout) = parts.next() else {
      continue;
    };
    if entry_layout == layout && is_xkb_name(code) {
      out.push(Variant {
        code: code.to_string(),
        layout: entry_layout.to_string(),
        description: description.trim().to_string(),
      });
    }
  }
  out
}

fn parse_hypr_keyboard_config() -> (String, String, String) {
  let generated = generated_hypr_input_path();
  let generated_contents = fs::read_to_string(generated).unwrap_or_default();
  let packaged = fs::read_to_string(hypr_config_path()).unwrap_or_default();
  let layout = extract_lua_string(&generated_contents, "kb_layout")
    .or_else(|| extract_lua_string(&packaged, "kb_layout"))
    .unwrap_or_default();
  let variant = extract_lua_string(&generated_contents, "kb_variant")
    .or_else(|| extract_lua_string(&packaged, "kb_variant"))
    .unwrap_or_default();
  let options = extract_lua_string(&generated_contents, "kb_options")
    .or_else(|| extract_lua_string(&packaged, "kb_options"))
    .unwrap_or_default();
  (layout, variant, options)
}

fn hypr_config_path() -> PathBuf {
  let user = argvus_control_center_core::paths::config_home().join("hypr/hyprland.lua");
  if user.exists() {
    user
  } else {
    packaged_hypr_config_path()
  }
}

/// The Hyprland configuration shipped by the package. It holds the ARGVUS
/// default keyboard layout and variant.
fn packaged_hypr_config_path() -> PathBuf {
  argvus_control_center_core::paths::system_config_root().join("hyprland/config/hyprland.lua")
}

fn generated_hypr_input_path() -> PathBuf {
  argvus_control_center_core::paths::argvus_data_home().join("generated/hypr/input.lua")
}

/// Extracts the quoted string assigned to `key` (`key = "value"`).
///
/// The generated `hypr/input.lua` is a single-line table
/// (`return { kb_layout = "br,us", ... }`) while the packaged config uses one
/// assignment per line, so the key may appear anywhere in a line. A match must
/// start at a word boundary so `_argvus_input.kb_layout` (a Lua reference, not a
/// literal) is not mistaken for the literal assignment.
fn extract_lua_string(contents: &str, key: &str) -> Option<String> {
  contents.lines().find_map(|line| {
    let mut search_from = 0;
    while let Some(offset) = line[search_from..].find(key) {
      let start = search_from + offset;
      let end = start + key.len();
      search_from = end;
      let preceded_by_identifier = line[..start]
        .chars()
        .next_back()
        .is_some_and(|character| character.is_alphanumeric() || matches!(character, '_' | '.'));
      if preceded_by_identifier {
        continue;
      }
      let Some(value) = line[end..]
        .trim_start()
        .strip_prefix('=')
        .and_then(|rest| rest.trim_start().strip_prefix('"'))
      else {
        continue;
      };
      // The closing quote ends the literal; Lua strings in this config never escape quotes.
      if let Some(close) = value.find('"') {
        return Some(value[..close].to_string());
      }
    }
    None
  })
}

fn first_csv_value(value: &str) -> String {
  value
    .split(',')
    .map(str::trim)
    .find(|part| !part.is_empty())
    .unwrap_or_default()
    .to_string()
}

fn active_layout(layouts: &str) -> String {
  let fallback = first_csv_value(layouts);
  let Some(output) = command_stdout("hyprctl", &["devices", "-j"]) else {
    return fallback;
  };
  let Ok(root) = serde_json::from_str::<Value>(&output) else {
    return fallback;
  };
  let Some(index) = active_layout_index(&root) else {
    return fallback;
  };
  layouts
    .split(',')
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .nth(index as usize)
    .unwrap_or(&fallback)
    .to_string()
}

/// The active layout index reported for the main keyboard. Hyprland also lists
/// virtual keyboards (power keys, consumer control) whose index can differ, so
/// the physical keyboard wins and the first keyboard is only a fallback.
fn active_layout_index(devices: &Value) -> Option<u64> {
  let keyboards = devices.get("keyboards")?.as_array()?;
  let indexes: Vec<(bool, u64)> = keyboards
    .iter()
    .filter_map(|keyboard| {
      let index = keyboard.get("active_layout_index")?.as_u64()?;
      let is_main = keyboard
        .get("main")
        .and_then(Value::as_bool)
        .unwrap_or(false);
      Some((is_main, index))
    })
    .collect();
  indexes
    .iter()
    .find(|(is_main, _)| *is_main)
    .or_else(|| indexes.first())
    .map(|(_, index)| *index)
}

fn write_generated_hypr_input(
  layout: Option<&str>,
  variant: Option<&str>,
) -> Result<(), SettingsError> {
  let current = parse_hypr_keyboard_config();
  let layout = layout.unwrap_or(&current.0);
  let variant = variant.unwrap_or(&current.1);
  let options = current.2;
  if !layout.split(',').all(is_xkb_name) {
    return Err(SettingsError::System(format!(
      "invalid Hyprland keyboard layout: {layout}"
    )));
  }
  if !variant.is_empty() && !variant.split(',').all(is_xkb_name) {
    return Err(SettingsError::System(format!(
      "invalid Hyprland keyboard variant: {variant}"
    )));
  }
  let patch = serde_json::json!({
    "/hyprland/keyboard/layout": layout,
    "/hyprland/keyboard/variant": variant,
    "/hyprland/keyboard/options": options,
  });
  let patch =
    serde_json::to_string(&patch).map_err(|error| SettingsError::System(error.to_string()))?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| SettingsError::System(error.to_string()))?;
  if !status.success() {
    return Err(SettingsError::System(
      "argvus config rejected the keyboard configuration".into(),
    ));
  }
  argvus_control_center_core::config::reload_argvus_config_service().map_err(SettingsError::System)
}

fn read_xkb_rules() -> Option<String> {
  [
    "/usr/share/X11/xkb/rules/base.lst",
    "/usr/share/X11/xkb/rules/evdev.lst",
  ]
  .into_iter()
  .map(Path::new)
  .find(|path| path.is_file())
  .and_then(|path| fs::read_to_string(path).ok())
}

pub fn is_xkb_name(value: &str) -> bool {
  !value.is_empty()
    && value
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

pub fn is_keymap_name(value: &str) -> bool {
  is_xkb_name(value)
    || value
      .bytes()
      .all(|b| matches!(b, b'/' | b'.') || b.is_ascii_alphanumeric() || b == b'-')
}

fn command_stdout(command: &str, args: &[&str]) -> Option<String> {
  let output = argvus_control_center_core::process::command(command)
    .args(args)
    .stdin(Stdio::null())
    .output()
    .ok()?;
  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).to_string())
}

/// The variant to store for `layout`. An empty request means "Default", which is
/// the ARGVUS default for that layout (abnt2 for the default Brazilian layout).
fn resolved_variant(layout: &str, requested: &str) -> String {
  if requested.is_empty() {
    default_variant_for(layout)
  } else {
    requested.to_string()
  }
}

/// The packaged ARGVUS default variant for `layout`, or empty when the layout
/// is not the packaged default layout.
fn default_variant_for(layout: &str) -> String {
  let packaged = fs::read_to_string(packaged_hypr_config_path()).unwrap_or_default();
  default_variant_from_packaged(&packaged, layout)
}

/// Reads the packaged default variant. The packaged variant belongs to the
/// packaged default layout only, so other layouts resolve to no variant.
fn default_variant_from_packaged(packaged: &str, layout: &str) -> String {
  if layout.is_empty() {
    return String::new();
  }
  let layouts = extract_lua_string(packaged, "kb_layout").unwrap_or_default();
  let packaged_default = layouts.split(',').next().unwrap_or_default();
  if layout != packaged_default {
    return String::new();
  }
  extract_lua_string(packaged, "kb_variant").unwrap_or_default()
}

/// Returns the variant to keep when the layout list changes. The variant is
/// kept while the default layout is the same, and cleared when the default
/// changes, because a variant belongs to one layout.
fn variant_after_default_change(
  current_layouts: &str,
  current_variant: &str,
  default_layout: &str,
) -> String {
  let current_default = current_layouts.split(',').next().unwrap_or_default();
  if current_default == default_layout {
    current_variant.to_string()
  } else {
    String::new()
  }
}

fn merged_layout_list(current: &str, layout: &str) -> String {
  let mut parts: Vec<String> = current
    .split(',')
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .map(str::to_string)
    .collect();
  let mut merged: Vec<String> = Vec::with_capacity(parts.len() + 1);
  for part in parts.drain(..) {
    if part != layout && !merged.contains(&part) {
      merged.push(part);
    }
  }
  merged.insert(0, layout.to_string());
  merged.join(",")
}

fn switch_active_layout(layout: &str, layouts: &str) -> Result<(), SettingsError> {
  let index = layouts
    .split(',')
    .position(|part| part == layout)
    .unwrap_or(0);
  let status = argvus_control_center_core::process::command("hyprctl")
    .args(["switchxkblayout", "all", &index.to_string()])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .status()
    .map_err(|error| {
      SettingsError::System(format!("could not activate keyboard layout: {error}"))
    })?;
  if !status.success() {
    return Err(SettingsError::System(
      "could not activate keyboard layout".into(),
    ));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn brazilian_default_variant_is_explicitly_selectable() {
    let variants = parse_xkb_variants(
      "! variant\n nodeadkeys br: Portuguese (Brazil, no dead keys)\n",
      "br",
    );
    assert!(
      variants
        .iter()
        .any(|variant| variant.code == "abnt2" && variant.layout == "br")
    );
    assert!(
      !parse_xkb_variants("! variant\n", "us")
        .iter()
        .any(|variant| variant.code == "abnt2")
    );
  }

  #[test]
  fn variant_survives_reordering_and_added_layouts() {
    // Same default, different list: the variant still belongs to br.
    assert_eq!(
      variant_after_default_change("br,us", "abnt2", "br"),
      "abnt2"
    );
    assert_eq!(
      variant_after_default_change("br,latam", "abnt2", "br"),
      "abnt2"
    );
  }

  #[test]
  fn variant_is_cleared_when_the_default_layout_changes() {
    assert_eq!(variant_after_default_change("br,us", "abnt2", "us"), "");
  }

  #[test]
  fn reads_keyboard_values_from_the_one_line_projection() {
    // argvus-config writes the whole table on one line.
    let projected = "-- Generated by argvus-config. Do not edit.\nreturn { kb_layout = \"br,us\", kb_variant = \"abnt2\", kb_options = \"grp:alt_shift_toggle\", }\n";
    assert_eq!(
      extract_lua_string(projected, "kb_layout").as_deref(),
      Some("br,us")
    );
    assert_eq!(
      extract_lua_string(projected, "kb_variant").as_deref(),
      Some("abnt2")
    );
  }

  #[test]
  fn default_variant_comes_from_the_packaged_default_layout_only() {
    let packaged =
      "local _argvus_input = {\n  kb_layout = \"br,us\",\n  kb_variant = \"abnt2\",\n}\n";
    assert_eq!(default_variant_from_packaged(packaged, "br"), "abnt2");
    assert_eq!(default_variant_from_packaged(packaged, "us"), "");
    assert_eq!(default_variant_from_packaged(packaged, ""), "");
  }

  #[test]
  fn resolved_variant_keeps_an_explicit_choice() {
    assert_eq!(resolved_variant("us", "intl"), "intl");
  }

  #[test]
  fn variant_is_empty_when_none_was_configured() {
    assert_eq!(variant_after_default_change("", "", "us"), "");
  }

  #[test]
  fn parses_localectl_keyboard_status() {
    let parsed =
      parse_localectl_status("VC Keymap: br-abnt2\nX11 Layout: br\nX11 Variant: abnt2\n");
    assert_eq!(parsed.console_keymap, "br-abnt2");
    assert_eq!(parsed.x11_layout, "br");
    assert_eq!(parsed.x11_variant, "abnt2");
  }

  #[test]
  fn parses_xkb_layouts_and_variants() {
    let data = "! layout\n  br Portuguese (Brazil)\n  us English (US)\n! variant\n  abnt2 br: ABNT2\n  intl us: English intl\n";
    let layouts = parse_xkb_layouts(data);
    assert_eq!(layouts[0].code, "br");
    let variants = parse_xkb_variants(data, "br");
    assert!(variants.iter().any(|variant| variant.code == "abnt2"));
    assert!(!variants.iter().any(|variant| variant.code == "intl"));
  }

  #[test]
  fn rejects_invalid_xkb_names() {
    assert!(is_xkb_name("br"));
    assert!(is_xkb_name("br_abnt2"));
    assert!(!is_xkb_name("br;reboot"));
  }

  #[test]
  fn merged_layout_list_moves_the_chosen_layout_first() {
    assert_eq!(merged_layout_list("", "br"), "br");
    assert_eq!(merged_layout_list("br", "br"), "br");
    assert_eq!(merged_layout_list("br,us", "us"), "us,br");
    assert_eq!(merged_layout_list("us,br,de", "br"), "br,us,de");
    assert_eq!(merged_layout_list("us,br", "us"), "us,br");
  }

  #[test]
  fn first_csv_value_reads_the_active_layout() {
    assert_eq!(first_csv_value("us,br"), "us");
    assert_eq!(first_csv_value("  br  "), "br");
    assert_eq!(first_csv_value(""), "");
  }

  #[test]
  fn active_layout_falls_back_to_the_first_configured_layout() {
    assert_eq!(active_layout("br,us"), "br");
  }

  #[test]
  /// Reads the values from the single-line table written by `argvus-config project`.
  fn extracts_layout_from_single_line_generated_table() {
    let generated = "-- Generated by argvus-config. Do not edit.\nreturn { kb_layout = \"us,br\", kb_variant = \"\", kb_options = \"\", }\n";
    assert_eq!(
      extract_lua_string(generated, "kb_layout").as_deref(),
      Some("us,br")
    );
    assert_eq!(
      extract_lua_string(generated, "kb_variant").as_deref(),
      Some("")
    );
  }

  #[test]
  /// Reads the per-line assignments of the packaged Hyprland config and ignores Lua references.
  fn extracts_layout_from_packaged_config_and_ignores_references() {
    let packaged = "  kb_layout = \"br,us\",\n  kb_variant = _argvus_input.kb_variant,\n";
    assert_eq!(
      extract_lua_string(packaged, "kb_layout").as_deref(),
      Some("br,us")
    );
    assert_eq!(extract_lua_string(packaged, "kb_variant"), None);
  }

  #[test]
  fn active_layout_index_prefers_the_main_keyboard() {
    let devices: Value = serde_json::from_str(
      r#"{"keyboards":[
        {"name":"power-button","main":false,"active_layout_index":0},
        {"name":"keyboard","main":true,"active_layout_index":1}
      ]}"#,
    )
    .unwrap();
    assert_eq!(active_layout_index(&devices), Some(1));
  }

  #[test]
  fn active_layout_index_falls_back_to_the_first_keyboard_without_a_main_one() {
    let devices: Value =
      serde_json::from_str(r#"{"keyboards":[{"name":"keyboard","active_layout_index":1}]}"#)
        .unwrap();
    assert_eq!(active_layout_index(&devices), Some(1));
    assert_eq!(active_layout_index(&Value::Null), None);
  }
}
