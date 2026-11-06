//! Rules of the project switcher: the project list, the workspace names, the
//! quoting for the Hyprland dispatch layer, and the picker labels. Only the
//! scan of a root directory touches the filesystem.

use std::fs;
use std::path::{Path, PathBuf};

/// SUPER + ALT + 1..9 only cover nine slots, so the list is capped at this size.
pub const MAX_PROJECTS: usize = 9;

/// Expands a leading `~` to the home directory.
pub fn expand_home(path: &str, home: &str) -> String {
  if path == "~" {
    return home.to_string();
  }
  match path.strip_prefix("~/") {
    Some(rest) => format!("{home}/{rest}"),
    None => path.to_string(),
  }
}

/// The immediate subdirectories of a root, in name order.
pub fn scan_root(root: &Path) -> Vec<String> {
  let Ok(entries) = fs::read_dir(root) else {
    return Vec::new();
  };
  let mut directories: Vec<PathBuf> = entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| path.is_dir())
    .collect();
  directories.sort();
  directories
    .into_iter()
    .map(|path| path.to_string_lossy().trim_end_matches('/').to_string())
    .collect()
}

/// Keeps the first occurrence of every entry.
pub fn unique_in_order<I: IntoIterator<Item = String>>(entries: I) -> Vec<String> {
  let mut seen = Vec::new();
  for entry in entries {
    if !seen.contains(&entry) {
      seen.push(entry);
    }
  }
  seen
}

/// Readable prefix; identity is provided by the full-path hash in `workspace_name`.
fn workspace_prefix(path: &str) -> String {
  let name = Path::new(path)
    .file_name()
    .map(|name| name.to_string_lossy().into_owned())
    .unwrap_or_default();
  let slug: String = name
    .chars()
    .map(|character| {
      if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
        character
      } else {
        '-'
      }
    })
    .collect();
  format!("proj-{slug}")
}

/// A stable workspace identity derived from the full, canonical project path.
/// FNV-1a is fixed here so identities do not change with Rust toolchain versions.
pub fn workspace_name(path: &str) -> String {
  let hash = path
    .as_bytes()
    .iter()
    .fold(0xcbf29ce484222325_u64, |hash, byte| {
      (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
  format!("{}-{hash:016x}", workspace_prefix(path))
}

/// Double-quotes a path for the shell that Hyprland runs exec commands with.
/// Double quotes, not single quotes, because the dispatch argument is Lua; the
/// escapes of the shell layer are added here.
pub fn shell_quote(value: &str) -> String {
  let escaped = value
    .replace('\\', "\\\\")
    .replace('"', "\\\"")
    .replace('$', "\\$")
    .replace('`', "\\`");
  format!("\"{escaped}\"")
}

/// Quotes a value as a Lua string literal for `hl.dsp.*` calls.
pub fn lua_string(value: &str) -> String {
  let mut quoted = String::from("\"");
  for character in value.chars() {
    match character {
      '\\' => quoted.push_str("\\\\"),
      '"' => quoted.push_str("\\\""),
      character if character.is_control() => {
        use std::fmt::Write;
        let _ = write!(quoted, "\\{:03}", character as u32);
      }
      character => quoted.push(character),
    }
  }
  quoted.push('"');
  quoted
}

/// Resolves a project by position, full path, or an unambiguous directory name.
/// A number always means the 1-based position in the picker.
pub fn resolve(projects: &[String], selector: &str) -> Option<String> {
  if selector.is_empty() {
    return None;
  }
  if selector.bytes().all(|byte| byte.is_ascii_digit()) {
    let position: usize = selector.parse().ok()?;
    return position
      .checked_sub(1)
      .and_then(|index| projects.get(index))
      .cloned();
  }
  if let Some(path) = projects.iter().find(|path| path.as_str() == selector) {
    return Some(path.clone());
  }
  let mut matches = projects
    .iter()
    .filter(|project| project.rsplit('/').next() == Some(selector));
  let path = matches.next()?;
  matches.next().is_none().then(|| path.clone())
}

/// The picker line of a project: its position, its name and its path.
pub fn picker_label(position: usize, path: &str) -> String {
  let name = path.rsplit('/').next().unwrap_or(path);
  format!("{position}  {name}  —  {path}")
}

/// The file where the active project is recorded for the telemetry block. It
/// is session state, so it lives under the runtime directory.
pub fn active_project_file(runtime_dir: Option<&str>) -> PathBuf {
  PathBuf::from(
    runtime_dir
      .filter(|value| !value.is_empty())
      .unwrap_or("/tmp"),
  )
  .join("argvus")
  .join("active-project")
}

#[cfg(test)]
mod tests {
  use super::{
    active_project_file, expand_home, lua_string, picker_label, resolve, shell_quote,
    unique_in_order, workspace_name, workspace_prefix,
  };

  #[test]
  fn home_is_expanded_only_at_the_start() {
    assert_eq!(expand_home("~/code", "/home/u"), "/home/u/code");
    assert_eq!(expand_home("~", "/home/u"), "/home/u");
    assert_eq!(expand_home("/srv/~x", "/home/u"), "/srv/~x");
  }

  #[test]
  fn workspace_identity_distinguishes_paths_and_slug_collisions() {
    assert_eq!(
      workspace_prefix("/home/u/code/argvus-keys"),
      "proj-argvus-keys"
    );
    assert_eq!(workspace_prefix("/home/u/my project"), "proj-my-project");
    assert_ne!(workspace_name("/a/app"), workspace_name("/b/app"));
    assert_ne!(workspace_name("/a/my app"), workspace_name("/a/my-app"));
    assert_eq!(workspace_name("/a/app"), "proj-app-c812e46e42a0356d");
  }

  #[test]
  fn duplicates_keep_their_first_position() {
    let merged = unique_in_order(["/a", "/b", "/a"].map(String::from));
    assert_eq!(merged, vec!["/a", "/b"]);
  }

  #[test]
  fn numbers_are_positions_and_names_match_the_last_component() {
    let projects = vec!["/x/one".to_string(), "/y/two".to_string()];
    assert_eq!(resolve(&projects, "2").as_deref(), Some("/y/two"));
    assert_eq!(resolve(&projects, "0"), None);
    assert_eq!(resolve(&projects, "9"), None);
    assert_eq!(resolve(&projects, "one").as_deref(), Some("/x/one"));
    assert_eq!(resolve(&projects, ""), None);
    let duplicates = vec!["/x/app".to_string(), "/y/app".to_string()];
    assert_eq!(resolve(&duplicates, "app"), None);
    assert_eq!(resolve(&duplicates, "/y/app").as_deref(), Some("/y/app"));
  }

  #[test]
  fn quoting_escapes_the_shell_and_the_lua_layers() {
    assert_eq!(shell_quote(r#"a"$b"#), r#""a\"\$b""#);
    assert_eq!(lua_string(r#"x"y\"#), r#""x\"y\\""#);
    assert_eq!(lua_string("a\nb"), "\"a\\010b\"");
  }

  #[test]
  fn picker_label_shows_position_name_and_path() {
    assert_eq!(picker_label(1, "/x/one"), "1  one  —  /x/one");
  }

  #[test]
  fn active_project_lives_under_the_runtime_directory() {
    assert_eq!(
      active_project_file(Some("/run/user/1000")),
      std::path::PathBuf::from("/run/user/1000/argvus/active-project")
    );
    assert_eq!(
      active_project_file(None),
      std::path::PathBuf::from("/tmp/argvus/active-project")
    );
  }
}
