//! Readers for the legacy file formats: JSON with comments, the Lua tables and the TOML subsets older versions wrote.

use super::*;

pub(super) fn strip_json_comments(contents: &str) -> String {
  let mut output = String::with_capacity(contents.len());
  let mut chars = contents.chars().peekable();
  let mut in_string = false;
  let mut escaped = false;
  while let Some(character) = chars.next() {
    if in_string {
      output.push(character);
      if escaped {
        escaped = false;
      } else if character == '\\' {
        escaped = true;
      } else if character == '"' {
        in_string = false;
      }
      continue;
    }
    if character == '"' {
      in_string = true;
      output.push(character);
    } else if character == '/' && chars.peek() == Some(&'/') {
      chars.next();
      for comment_character in chars.by_ref() {
        if comment_character == '\n' {
          output.push('\n');
          break;
        }
      }
    } else {
      output.push(character);
    }
  }
  output
}

pub(super) fn lua_call_bodies(contents: &str, call: &str) -> Vec<String> {
  let mut bodies = Vec::new();
  let mut offset = 0;
  while let Some(relative) = contents[offset..].find(&format!("{call}({{")) {
    let start = offset + relative + call.len() + 2;
    let Some(end) = contents[start..].find("})") else {
      break;
    };
    bodies.push(contents[start..start + end].to_string());
    offset = start + end + 2;
  }
  bodies
}

pub(super) fn lua_fields(body: &str) -> Map<String, Value> {
  body
    .split(',')
    .filter_map(|field| {
      let (key, raw) = field.split_once('=')?;
      Some((key.trim().to_string(), parse_lua_value(raw.trim())?))
    })
    .collect()
}

pub(super) fn parse_lua_value(raw: &str) -> Option<Value> {
  parse_scalar(raw).or_else(|| {
    raw
      .parse::<f64>()
      .ok()
      .and_then(serde_json::Number::from_f64)
      .map(Value::Number)
  })
}

pub(super) fn parse_keybindings_toml(path: &Path) -> ConfigResult<Option<Value>> {
  let Ok(contents) = fs::read_to_string(path) else {
    return Ok(None);
  };
  let mut bindings = Map::new();
  let mut current = None;
  for line in contents.lines() {
    let line = line.trim();
    if let Some(section) = line.strip_prefix("[keybindings.")
      && let Some(section) = section.strip_suffix(']')
    {
      let section = section.trim_matches('"').to_string();
      bindings.insert(section.clone(), Value::Object(Map::new()));
      current = Some(section);
      continue;
    }
    let Some(id) = current.as_ref() else {
      continue;
    };
    let Some((key, raw)) = line.split_once('=') else {
      continue;
    };
    let Some(value) = parse_scalar(raw.trim()) else {
      continue;
    };
    if let Some(object) = bindings.get_mut(id).and_then(Value::as_object_mut) {
      object.insert(key.trim().to_string(), value);
    }
  }
  Ok((!bindings.is_empty()).then_some(Value::Object(bindings)))
}

pub(super) fn parse_input_toml(path: &Path) -> ConfigResult<Option<Value>> {
  let Ok(contents) = fs::read_to_string(path) else {
    return Ok(None);
  };
  let mut sections = Map::new();
  let mut current = None;
  for line in contents.lines() {
    let line = line.trim();
    if let Some(section) = line
      .strip_prefix('[')
      .and_then(|line| line.strip_suffix(']'))
      && matches!(section, "mouse" | "touchpad")
    {
      current = Some(section.to_string());
      sections.insert(section.to_string(), Value::Object(Map::new()));
      continue;
    }
    let Some(section) = current.as_ref() else {
      continue;
    };
    let Some((key, raw)) = line.split_once('=') else {
      continue;
    };
    let Some(value) = parse_scalar(raw.trim()) else {
      continue;
    };
    if let Some(object) = sections.get_mut(section).and_then(Value::as_object_mut) {
      object.insert(key.trim().to_string(), value);
    }
  }
  Ok((!sections.is_empty()).then_some(Value::Object(sections)))
}

pub(super) fn parse_keyboard_lua(path: &Path) -> ConfigResult<Option<Value>> {
  let Ok(contents) = fs::read_to_string(path) else {
    return Ok(None);
  };
  let mut keyboard = Map::new();
  for key in ["kb_layout", "kb_variant", "kb_options"] {
    let Some(value) = contents.lines().find_map(|line| {
      let value = line.trim().strip_prefix(&format!("{key} ="))?.trim();
      let value = value.strip_suffix(',').unwrap_or(value);
      let value = value.strip_prefix('"')?.strip_suffix('"')?;
      Some(value.to_string())
    }) else {
      continue;
    };
    keyboard.insert(
      match key {
        "kb_layout" => "layout",
        "kb_variant" => "variant",
        _ => "options",
      }
      .into(),
      Value::String(value),
    );
  }
  Ok((!keyboard.is_empty()).then_some(Value::Object(keyboard)))
}

pub(super) fn parse_scalar(raw: &str) -> Option<Value> {
  serde_json::from_str(raw).ok().or_else(|| {
    raw
      .parse::<f64>()
      .ok()
      .and_then(serde_json::Number::from_f64)
      .map(Value::Number)
  })
}
