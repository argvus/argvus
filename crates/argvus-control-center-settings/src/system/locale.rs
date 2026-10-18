//! Implements locale discovery and configuration in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Stdio;

use crate::error::SettingsError;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `Distro`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Distro {
  pub id: String,
  pub pretty_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `LocaleEntry`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct LocaleEntry {
  pub locale: String,
  pub encoding: String,
  pub enabled: bool,
  pub line_index: usize,
}

/// Executes the `distro` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn distro() -> Distro {
  parse_os_release(&fs::read_to_string("/etc/os-release").unwrap_or_default())
}

/// Converts input data into `parse_os_release` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_os_release(contents: &str) -> Distro {
  let mut id = String::new();
  let mut pretty_name = String::new();
  for line in contents.lines() {
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    let value = value.trim_matches('"').to_string();
    match key {
      "ID" => id = value,
      "PRETTY_NAME" => pretty_name = value,
      _ => {}
    }
  }
  Distro { id, pretty_name }
}

/// Executes the `current_lang` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn current_lang() -> String {
  command_stdout("localectl", &["status"])
    .and_then(|output| {
      output.lines().find_map(|line| {
        line
          .trim()
          .strip_prefix("System Locale:")
          .and_then(|rest| {
            rest
              .split_whitespace()
              .find_map(|part| part.strip_prefix("LANG="))
          })
          .map(str::to_string)
      })
    })
    .or_else(|| {
      fs::read_to_string("/etc/locale.conf")
        .ok()
        .and_then(|contents| {
          contents.lines().find_map(|line| {
            line
              .trim()
              .strip_prefix("LANG=")
              .map(|value| value.trim_matches('"').to_string())
          })
        })
    })
    .unwrap_or_default()
}

/// Executes the `generated_locales` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn generated_locales() -> Vec<String> {
  command_stdout("locale", &["-a"])
    .unwrap_or_default()
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty() && *line != "C" && *line != "POSIX")
    .map(normalize_locale)
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect()
}

/// Executes the `locale_gen_entries` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn locale_gen_entries(path: &Path) -> Result<Vec<LocaleEntry>, SettingsError> {
  let contents = fs::read_to_string(path)
    .map_err(|error| SettingsError::System(format!("{}: {error}", path.display())))?;
  Ok(parse_locale_gen(&contents))
}

/// Converts input data into `parse_locale_gen` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_locale_gen(contents: &str) -> Vec<LocaleEntry> {
  contents
    .lines()
    .enumerate()
    .filter_map(|(line_index, line)| parse_locale_gen_line(line, line_index))
    .collect()
}

/// Converts input data into `parse_locale_gen_line` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_locale_gen_line(line: &str, line_index: usize) -> Option<LocaleEntry> {
  let trimmed = line.trim_start();
  let (enabled, body) = if let Some(rest) = trimmed.strip_prefix('#') {
    (false, rest.trim_start())
  } else {
    (true, trimmed)
  };
  if body.is_empty() || body.starts_with('#') {
    return None;
  }
  let mut parts = body.split_whitespace();
  let locale = parts.next()?;
  let encoding = parts.next()?;
  if parts.next().is_some() || !is_locale_name(locale) || !is_encoding_name(encoding) {
    return None;
  }
  Some(LocaleEntry {
    locale: locale.to_string(),
    encoding: encoding.to_string(),
    enabled,
    line_index,
  })
}

/// Applies the `apply_locale_gen` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn apply_locale_gen(contents: &str, selected: &BTreeSet<String>) -> String {
  let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
  for entry in parse_locale_gen(contents) {
    let should_enable = selected.contains(&format!("{} {}", entry.locale, entry.encoding));
    if should_enable == entry.enabled {
      continue;
    }
    let line = &mut lines[entry.line_index];
    if should_enable {
      if let Some(pos) = line.find('#') {
        let after_hash = &line[pos + 1..];
        if after_hash.trim_start().starts_with(&entry.locale) {
          let spaces = after_hash.len() - after_hash.trim_start().len();
          line.replace_range(pos..pos + 1 + spaces, "");
        }
      }
    } else if let Some(pos) = line.find(&entry.locale) {
      line.insert(pos, '#');
    }
  }
  let mut output = lines.join("\n");
  if contents.ends_with('\n') {
    output.push('\n');
  }
  output
}

/// Applies the `set_lang` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set_lang(locale: &str, generated: &[String]) -> Result<(), SettingsError> {
  if !generated.iter().any(|candidate| candidate == locale) {
    return Err(SettingsError::System(format!(
      "locale is not generated: {locale}"
    )));
  }
  super::privileged::run(&["locale", "set-lang", locale]).map(|_| ())
}

/// Applies the `set_system_locales` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set_system_locales(
  entries: &[LocaleEntry],
  selected: &BTreeSet<String>,
) -> Result<(), SettingsError> {
  let valid: BTreeSet<String> = entries
    .iter()
    .map(|entry| format!("{} {}", entry.locale, entry.encoding))
    .collect();
  if selected.iter().any(|value| !valid.contains(value)) {
    return Err(SettingsError::System(
      "selected locale is not present in /etc/locale.gen".to_string(),
    ));
  }
  let mut args = vec!["locale".to_string(), "set-generated".to_string()];
  args.extend(selected.iter().cloned());
  let refs: Vec<&str> = args.iter().map(String::as_str).collect();
  super::privileged::run(&refs).map(|_| ())
}

/// Executes the `encoding_from_locale` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn encoding_from_locale(locale: &str) -> String {
  locale
    .rsplit_once('.')
    .map(|(_, encoding)| encoding.to_string())
    .unwrap_or_else(|| "UTF-8".to_string())
}

/// Executes the `normalize_locale` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn normalize_locale(value: &str) -> String {
  if let Some((base, encoding)) = value.rsplit_once('.') {
    format!(
      "{base}.{}",
      encoding.to_ascii_uppercase().replace("UTF8", "UTF-8")
    )
  } else {
    value.to_string()
  }
}

/// Checks the condition represented by `is_locale_name` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn is_locale_name(value: &str) -> bool {
  !value.is_empty()
    && value
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'@' | b'-'))
}

/// Checks the condition represented by `is_encoding_name` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn is_encoding_name(value: &str) -> bool {
  !value.is_empty()
    && value
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

/// Executes the `command_stdout` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Converts input data into `parses_os_release` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_os_release() {
    let distro = parse_os_release("ID=arch\nPRETTY_NAME=\"Arch Linux\"\n");
    assert_eq!(distro.id, "arch");
    assert_eq!(distro.pretty_name, "Arch Linux");
  }

  #[test]
  /// Converts input data into `parses_locale_gen_and_preserves_comments` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_locale_gen_and_preserves_comments() {
    let entries = parse_locale_gen("# comment\n#en_US.UTF-8 UTF-8\nde_DE.UTF-8 UTF-8\n\n");
    assert_eq!(entries.len(), 2);
    assert!(!entries[0].enabled);
    assert!(entries[1].enabled);
  }

  #[test]
  /// Applies the `toggles_only_locale_lines` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn toggles_only_locale_lines() {
    let input = "# Header\n#en_US.UTF-8 UTF-8\nde_DE.UTF-8 UTF-8\n";
    let selected = BTreeSet::from(["en_US.UTF-8 UTF-8".to_string()]);
    let output = apply_locale_gen(input, &selected);
    assert!(output.contains("# Header"));
    assert!(output.contains("en_US.UTF-8 UTF-8"));
    assert!(output.contains("#de_DE.UTF-8 UTF-8"));
  }
}
