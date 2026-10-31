//! Blur and animation settings table: reading, validation and writes.

use super::commands::{master_switch, table_row};
use super::surfaces::spawn_runtime;
use super::*;

impl Effects {
  pub(super) fn settings_read(&self, table: &str, key: &str) -> Option<String> {
    let row = table_row(table, key)?;
    let value = self
      .store_raw(&format!("/effects/{key}"))
      .unwrap_or_else(|| "null".to_string());
    if settings_entry_valid(key, &row.kind, &row.minimum, &row.maximum, &value) {
      Some(value)
    } else {
      Some(row.default)
    }
  }

  /// `get`: one `name=value` line for every row of the table.
  pub(super) fn settings_get(&self, table: &str, prefix: &str) -> Vec<String> {
    table
      .lines()
      .filter_map(|line| line.split_whitespace().next())
      .map(|key| {
        format!(
          "{}={}",
          key.strip_prefix(prefix).unwrap_or(key),
          self.settings_read(table, key).unwrap_or_default()
        )
      })
      .collect()
  }

  /// `set name=value ...`: validates every pair, then writes them in one patch.
  /// `enabled=enabled|disabled` sets the master switch of the same effect, so
  /// the switch and the values reload the configuration once.
  pub(super) fn settings_set(
    &self,
    table: &str,
    prefix: &str,
    pairs: &[String],
  ) -> Result<Vec<String>, Failure> {
    if pairs.is_empty() || self.store.is_none() {
      return Err(64);
    }
    let mut patch = Map::new();
    let mut switch_written = false;
    for argument in pairs {
      let (name, value) = argument.split_once('=').ok_or(64)?;
      if name == "enabled" {
        if value != "enabled" && value != "disabled" {
          return Err(64);
        }
        patch.insert(
          master_switch(prefix).ok_or(64)?.into(),
          Value::Bool(value == "enabled"),
        );
        switch_written = true;
        continue;
      }
      // Animations owns the Windows draft, including decoration.wobble.
      // Its combined table still validates every key before the atomic patch.
      let key = if prefix == "animation_" && name.starts_with("wobble_") {
        name.to_owned()
      } else {
        format!("{prefix}{name}")
      };
      let row = table_row(table, &key).ok_or(64)?;
      if !settings_entry_valid(&key, &row.kind, &row.minimum, &row.maximum, value) {
        return Err(64);
      }
      let literal = if row.kind.starts_with("enum:") || row.kind == "color" {
        Value::String(value.to_string())
      } else {
        serde_json::from_str(value).map_err(|_| 64)?
      };
      patch.insert(format!("/effects/{key}"), literal);
    }
    self.patch_store(&patch).map_err(|_| 1)?;
    self.request_config_reload().map_err(|_| 1)?;
    if switch_written {
      self
        .apply_surfaces(self.status("transparency"), self.status("animations"))
        .map_err(|_| 1)?;
    }
    let output = self.settings_get(table, prefix);
    spawn_runtime();
    Ok(output)
  }
}

/// Wobble permits an exact zero for damping and intensity. Preserve the legacy
/// settings parser's leading-zero contract for the other effects.
fn settings_entry_valid(key: &str, kind: &str, minimum: &str, maximum: &str, value: &str) -> bool {
  (key.starts_with("wobble_") && kind == "number" && minimum == "0" && value == "0")
    || settings_valid(kind, minimum, maximum, value)
}

/// Checks a value against its row. The rules are the shell's, including its
/// leading-zero test, which also refuses a lone `0`.
pub fn settings_valid(kind: &str, minimum: &str, maximum: &str, value: &str) -> bool {
  match kind {
    "boolean" => return value == "true" || value == "false",
    // An empty color means "follow the theme accent"; anything else is RRGGBBAA.
    "color" => {
      return value.is_empty()
        || (value.len() == 8 && value.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    _ if kind.starts_with("enum:") => {
      return kind["enum:".len()..]
        .split(',')
        .any(|choice| choice == value);
    }
    _ => {}
  }
  if is_leading_zero(value) {
    return false;
  }
  let digits = value.strip_prefix('-').unwrap_or(value);
  let shape_ok = match kind {
    "integer" => !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()),
    "number" => {
      !digits.is_empty()
        && digits
          .bytes()
          .all(|byte| byte.is_ascii_digit() || byte == b'.')
        && !digits.starts_with('.')
        && !digits.ends_with('.')
        && digits.matches('.').count() <= 1
    }
    _ => false,
  };
  if !shape_ok {
    return false;
  }
  match (
    value.parse::<f64>(),
    minimum.parse::<f64>(),
    maximum.parse::<f64>(),
  ) {
    (Ok(number), Ok(low), Ok(high)) => number >= low && number <= high,
    _ => false,
  }
}

/// `0[0-9]*` or `-0[0-9]*`, the shell's pattern: it matches a lone `0` too.
pub(super) fn is_leading_zero(value: &str) -> bool {
  let rest = match value.strip_prefix("-0") {
    Some(rest) => rest,
    None => match value.strip_prefix('0') {
      Some(rest) => rest,
      None => return false,
    },
  };
  rest.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod wobble_tests {
  use super::*;

  #[test]
  fn wobble_zero_is_valid_only_for_parameters_with_a_zero_minimum() {
    assert!(settings_entry_valid(
      "wobble_damping",
      "number",
      "0",
      "1000",
      "0"
    ));
    assert!(settings_entry_valid(
      "wobble_intensity",
      "number",
      "0",
      "2",
      "0"
    ));
    assert!(!settings_entry_valid(
      "wobble_mass",
      "number",
      "0.0001",
      "1000",
      "0"
    ));
    assert!(!settings_entry_valid(
      "wobble_mesh",
      "integer",
      "2",
      "32",
      "0"
    ));
    assert!(!settings_entry_valid("blur_noise", "number", "0", "1", "0"));
  }
}
