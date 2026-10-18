//! The edit of the Waybar JSONC. Each block is wrapped by `BEGIN`/`END` marker
//! comments; showing a block removes the `//` of its lines, hiding it adds one.
//! The `modules-left` array is then rebuilt in the block order, with commas that
//! keep the JSONC valid. These functions reproduce the sed and awk edits the
//! script ran, line by line, so the same file gives the same result.

use crate::blocks::BLOCK_NAMES;

/// The marker name of a block: `ARGVUS_TELEMETRY_` and the upper-case name.
pub fn marker(block: &str) -> String {
  format!("ARGVUS_TELEMETRY_{}", block.to_uppercase())
}

/// Splits text into lines that keep their line ending, as sed sees them.
fn split_lines(text: &str) -> Vec<&str> {
  text.split_inclusive('\n').collect()
}

/// The text of a line without its line ending.
fn body(line: &str) -> &str {
  line.strip_suffix('\n').unwrap_or(line)
}

/// Joins lines, adding a line ending to each as awk's `print` does.
fn join(lines: &[String]) -> String {
  let mut text = String::new();
  for line in lines {
    text.push_str(line);
    text.push('\n');
  }
  text
}

/// The sed range `/BEGIN/,/END/`: the lines strictly between the markers, and
/// each range start and end line, as the sed script saw them.
fn in_marker_range(lines: &[&str], begin: &str, end: &str) -> Vec<bool> {
  let mut inside = false;
  lines
    .iter()
    .map(|line| {
      if !inside {
        if body(line).contains(begin) {
          inside = true;
        }
        false
      } else if body(line).contains(end) {
        inside = false;
        false
      } else {
        true
      }
    })
    .collect()
}

/// Shows (`enabled`) or hides (`disabled`) the lines of one block, leaving the
/// marker lines alone. Showing removes a leading `//`; hiding adds one to every
/// line that does not already start with a comment after its indentation.
pub fn apply_block(text: &str, block: &str, enabled: bool) -> String {
  let marker = marker(block);
  let begin = format!("{marker}_BEGIN");
  let end = format!("{marker}_END");
  let lines = split_lines(text);
  let inside = in_marker_range(&lines, &begin, &end);
  let mut output = String::new();
  for (line, inside) in lines.iter().zip(inside) {
    let content = body(line);
    let ending = &line[content.len()..];
    if !inside {
      output.push_str(line);
      continue;
    }
    let edited = if enabled {
      content.strip_prefix("//").unwrap_or(content).to_string()
    } else if content.trim_start().starts_with("//") {
      content.to_string()
    } else {
      format!("//{content}")
    };
    output.push_str(&edited);
    output.push_str(ending);
  }
  output
}

/// The `modules-left` key of the array, when the line has one. The script
/// searched the whole line, so the key does not need to start it.
fn is_modules_left(line: &str) -> bool {
  line
    .match_indices("\"modules-left\"")
    .any(|(index, key)| line[index + key.len()..].trim_start().starts_with(':'))
}

/// Whether a line closes an object at the top level of a module definition.
fn is_module_close(line: &str) -> bool {
  let content = line.trim_end();
  let without_comma = content.strip_suffix(',').unwrap_or(content);
  without_comma == "    }"
}

/// Strips a trailing comma and the blanks after it, as
/// `sub(/,[[:space:]]*$/, "")` did. A line without that comma is unchanged.
fn strip_comma(line: &str) -> String {
  let trimmed = line.trim_end();
  match trimmed.strip_suffix(',') {
    Some(stripped) => stripped.to_string(),
    None => line.to_string(),
  }
}

/// Makes every module definition close with a comma, except the last one.
pub fn normalize_module_properties(text: &str) -> String {
  let lines: Vec<String> = text.lines().map(str::to_string).collect();
  let closings: Vec<usize> = lines
    .iter()
    .enumerate()
    .filter(|(_, line)| is_module_close(line))
    .map(|(index, _)| index)
    .collect();
  let last = closings.last().copied();
  let mut output = lines.clone();
  for &index in &closings {
    output[index] = strip_comma(&lines[index]);
    if Some(index) != last {
      output[index].push(',');
    }
  }
  join(&output)
}

/// Keeps the `modules-left` array free of a trailing comma after its last custom
/// module, and every custom module but the last ends with a comma.
pub fn normalize_modules_left(text: &str) -> String {
  let lines: Vec<&str> = text.lines().collect();
  let mut output: Vec<String> = Vec::new();
  let mut index = 0;
  while index < lines.len() {
    let line = lines[index];
    output.push(line.to_string());
    if !is_modules_left(line) {
      index += 1;
      continue;
    }
    index += 1;
    let mut buffered: Vec<String> = Vec::new();
    let mut closed = false;
    while index < lines.len() {
      let inner = lines[index];
      index += 1;
      if inner.trim_start().starts_with("],") {
        closed = true;
        break;
      }
      buffered.push(inner.to_string());
    }
    if !closed {
      // The script never printed the buffered lines when the array was not
      // closed, so they are dropped the same way.
      break;
    }
    let last_custom = buffered
      .iter()
      .rposition(|item| item.trim_start().starts_with("\"custom/"));
    for (position, item) in buffered.iter().enumerate() {
      if item.trim_start().starts_with("\"custom/") {
        let mut edited = strip_comma(item);
        if Some(position) != last_custom {
          edited.push(',');
        }
        output.push(edited);
      } else {
        output.push(item.clone());
      }
    }
    output.push(lines[index - 1].to_string());
  }
  join(&output)
}

/// The lines of one block as sed printed them: from its `BEGIN` marker to the
/// first `END` marker after it, both included.
fn block_chunk(lines: &[&str], block: &str) -> Vec<String> {
  let marker = marker(block);
  let begin = format!("{marker}_BEGIN");
  let end = format!("{marker}_END");
  let mut chunk = Vec::new();
  let mut started = false;
  for line in lines {
    if !started {
      if body(line).contains(&begin) {
        started = true;
      } else {
        continue;
      }
    }
    chunk.push(body(line).to_string());
    if body(line).contains(&end) {
      break;
    }
  }
  chunk
}

/// Rebuilds `modules-left` with each block's chunk in the given order. The lines
/// of a block move with its markers and keep their comment state.
pub fn reorder_modules_left(text: &str, order: &[String]) -> String {
  let lines = split_lines(text);
  let chunks: Vec<(String, Vec<String>)> = BLOCK_NAMES
    .iter()
    .map(|block| (block.to_string(), block_chunk(&lines, block)))
    .collect();
  let mut output: Vec<String> = Vec::new();
  let mut inside = false;
  for line in lines {
    let content = body(line).to_string();
    if is_modules_left(&content) {
      output.push(content);
      for block in order {
        if let Some((_, chunk)) = chunks.iter().find(|(name, _)| name == block) {
          output.extend(chunk.iter().cloned());
        }
      }
      inside = true;
      continue;
    }
    if inside && content.trim_start().starts_with("],") {
      inside = false;
      output.push(content);
      continue;
    }
    if inside {
      continue;
    }
    output.push(content);
  }
  join(&output)
}

/// Every block's state applied, the array reordered, and the commas fixed. This
/// is the whole edit the script ran on a configuration file.
pub fn apply_all(text: &str, states: &[(&str, bool)], order: &[String]) -> String {
  let mut edited = text.to_string();
  for block in BLOCK_NAMES {
    let enabled = states
      .iter()
      .find(|(name, _)| *name == block)
      .map(|(_, enabled)| *enabled)
      .unwrap_or(true);
    edited = apply_block(&edited, block, enabled);
  }
  let reordered = reorder_modules_left(&edited, order);
  let fixed = normalize_modules_left(&reordered);
  normalize_module_properties(&fixed)
}

#[cfg(test)]
mod tests {
  use super::{apply_block, normalize_module_properties};

  #[test]
  fn hiding_a_block_comments_its_lines_and_showing_uncomments_them() {
    let text = "// ARGVUS_TELEMETRY_KEYS_BEGIN\n    \"a\": 1\n// ARGVUS_TELEMETRY_KEYS_END\n";
    let hidden = apply_block(text, "keys", false);
    assert_eq!(
      hidden,
      "// ARGVUS_TELEMETRY_KEYS_BEGIN\n//    \"a\": 1\n// ARGVUS_TELEMETRY_KEYS_END\n"
    );
    assert_eq!(apply_block(&hidden, "keys", true), text);
  }

  #[test]
  fn module_objects_close_with_commas_except_the_last() {
    let text = "    },\n    }\n";
    assert_eq!(normalize_module_properties(text), "    },\n    }\n");
    assert_eq!(
      normalize_module_properties("    }\n    },\n"),
      "    },\n    }\n"
    );
  }
}
