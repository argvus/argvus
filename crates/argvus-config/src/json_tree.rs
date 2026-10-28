//! JSON pointer navigation and edits on `serde_json::Value` trees, and key-sorted copies for stable output.

use super::*;

/// Resolves one JSON Pointer segment, using numeric segments as array indexes.
pub(super) fn child_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
  match value {
    Value::Array(items) => key.parse::<usize>().ok().and_then(|index| items.get(index)),
    other => other.get(key),
  }
}

pub(super) fn pointer_components(pointer: &str) -> ConfigResult<Vec<String>> {
  if !pointer.starts_with('/') {
    return Err(ConfigError::Invalid(format!(
      "JSON pointer must start with '/': {pointer}"
    )));
  }
  let components = pointer
    .trim_start_matches('/')
    .split('/')
    .filter(|component| !component.is_empty())
    .map(|component| component.replace("~1", "/").replace("~0", "~"))
    .collect::<Vec<_>>();
  if components.is_empty() {
    return Err(ConfigError::Invalid("empty JSON pointer".into()));
  }
  Ok(components)
}

pub(super) fn set_nested(
  root: &mut Value,
  components: &[String],
  value: Value,
) -> ConfigResult<()> {
  let mut current = root;
  for component in &components[..components.len().saturating_sub(1)] {
    let object = current
      .as_object_mut()
      .ok_or_else(|| ConfigError::Invalid("intermediate value must be an object".into()))?;
    current = object
      .entry(component)
      .or_insert_with(|| Value::Object(Map::new()));
  }
  current
    .as_object_mut()
    .ok_or_else(|| ConfigError::Invalid("target parent must be an object".into()))?
    .insert(components.last().cloned().unwrap_or_default(), value);
  Ok(())
}

pub(super) fn unset_nested(root: &mut Value, components: &[String]) {
  if components.is_empty() {
    return;
  }
  if components.len() == 1 {
    if let Some(object) = root.as_object_mut() {
      object.remove(&components[0]);
    }
    return;
  }
  if let Some(next) = root.get_mut(&components[0]) {
    unset_nested(next, &components[1..]);
  }
}

/// Returns `value` with every object's keys in sorted order, recursively.
///
/// `serde_json` keeps insertion order when its `preserve_order` feature is enabled,
/// and the Control Center enables it. Cargo unifies features across a workspace
/// build, so the configuration writers would otherwise change their output with
/// the set of crates built together. The configuration files have always been
/// written with sorted keys, so every writer goes through this function.
pub fn sorted_value(value: &Value) -> Value {
  match value {
    Value::Object(map) => {
      let mut entries: Vec<(&String, &Value)> = map.iter().collect();
      entries.sort_by(|left, right| left.0.cmp(right.0));
      Value::Object(
        entries
          .into_iter()
          .map(|(key, item)| (key.clone(), sorted_value(item)))
          .collect(),
      )
    }
    Value::Array(items) => Value::Array(items.iter().map(sorted_value).collect()),
    other => other.clone(),
  }
}
