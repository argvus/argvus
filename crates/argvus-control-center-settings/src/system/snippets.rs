//! Snippets typed by `argvus-snippets` (SUPER + ALT + N).
//!
//! The launcher owns the stored data and every write. This module reads the
//! entries from `argvus-config` and changes them only through the
//! `argvus-snippets` CLI, so both the page and the shortcut share one rule set.
use argvus_control_center_core::process::command;
use serde_json::Value;

/// Delay before a typed snippet, so the user can focus the target window
/// after pressing Enter on the page.
const TYPE_DELAY_SECONDS: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnippetEntry {
  pub name: String,
  pub content: String,
}

/// The loaded entries. `available` is false when `argvus-snippets` is not
/// installed, so the page can say so instead of showing an empty list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Snippets {
  pub available: bool,
  pub entries: Vec<SnippetEntry>,
}

/// Loads the entries from `argvus-config`, after checking that the launcher is installed.
pub fn load() -> Snippets {
  let installed = command("argvus")
    .args(["snippets", "list"])
    .output()
    .is_ok_and(|output| output.status.success());
  if !installed {
    return Snippets::default();
  }
  let entries = command("argvus")
    .arg("config")
    .args(["get", "/snippets/items", "--effective", "--raw"])
    .output()
    .ok()
    .filter(|output| output.status.success())
    .map(|output| parse_items(&String::from_utf8_lossy(&output.stdout)))
    .unwrap_or_default();
  Snippets {
    available: true,
    entries,
  }
}

/// Parses the `{name, content}` array stored under `/snippets/items`.
/// Entries without a name or with non-text fields are skipped.
pub fn parse_items(json: &str) -> Vec<SnippetEntry> {
  let Ok(items) = serde_json::from_str::<Vec<Value>>(json) else {
    return Vec::new();
  };
  items
    .iter()
    .filter_map(|item| {
      let name = item.get("name")?.as_str()?.trim();
      let content = item.get("content")?.as_str()?;
      (!name.is_empty()).then(|| SnippetEntry {
        name: name.to_owned(),
        content: content.to_owned(),
      })
    })
    .collect()
}

/// Saves a snippet, replacing the one with the same name.
pub fn add(name: &str, content: &str) -> Result<(), String> {
  let name = name.trim();
  if name.is_empty() {
    return Err("empty snippet name".into());
  }
  if content.is_empty() {
    return Err("empty snippet content".into());
  }
  run(&["add", name, content])
}

/// Removes a snippet by its name.
pub fn remove(name: &str) -> Result<(), String> {
  run(&["remove", name])
}

/// Types the snippet at a 1-based position after a short delay.
pub fn type_position(position: usize) -> Result<(), String> {
  type_later(&["open".to_owned(), position.to_string()])
}

/// Opens the picker after a short delay, so the user can focus the target window.
pub fn open_picker() -> Result<(), String> {
  type_later(&[])
}

/// Starts `argvus-snippets` in the background after the delay. The shell
/// detaches the child and exits at once, so no zombie process stays behind.
fn type_later(args: &[String]) -> Result<(), String> {
  let script = format!("( sleep {TYPE_DELAY_SECONDS}; exec \"$0\" \"$@\" ) >/dev/null 2>&1 &");
  let status = command("sh")
    .arg("-c")
    .arg(script)
    .arg("argvus")
    .arg("snippets")
    .args(args)
    .status()
    .map_err(|error| format!("failed to run argvus-snippets: {error}"))?;
  if status.success() {
    Ok(())
  } else {
    Err("argvus-snippets failed to start".into())
  }
}

fn run(args: &[&str]) -> Result<(), String> {
  let output = command("argvus")
    .arg("snippets")
    .args(args)
    .output()
    .map_err(|error| format!("failed to run argvus-snippets: {error}"))?;
  if output.status.success() {
    return Ok(());
  }
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
  Err(if stderr.is_empty() {
    "argvus-snippets failed".into()
  } else {
    stderr
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_named_items_and_skips_invalid_ones() {
    let entries = parse_items(
      r#"[{"name":"email","content":"you@example.com"},{"name":"","content":"x"},{"content":"no name"},{"name":"bad","content":3}]"#,
    );
    assert_eq!(
      entries,
      vec![SnippetEntry {
        name: "email".into(),
        content: "you@example.com".into(),
      }]
    );
  }

  #[test]
  fn null_or_malformed_config_gives_no_entries() {
    assert!(parse_items("null").is_empty());
    assert!(parse_items("not json").is_empty());
  }

  #[test]
  fn add_rejects_empty_name_or_content_without_running_the_launcher() {
    assert_eq!(add("   ", "text"), Err("empty snippet name".into()));
    assert_eq!(add("name", ""), Err("empty snippet content".into()));
  }
}
