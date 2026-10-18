//! Implements default-application detection in crate `argvus control center apps`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! Three complementary signals are combined, as described in the project
//! spec:
//!   1. `.desktop` files under `/usr/share/applications` and the per-user dir;
//!   2. `command -v <binary>` (a direct `$PATH` scan, no subprocess needed);
//!   3. the bundled catalog of known binaries per category.
//!
//! A catalog app counts as *installed* when its binary is on `$PATH` or in the
//! private ARGVUS commands directory (`/usr/lib/argvus/commands`, where
//! ARGVUS-owned entries such as `argvus-terminal` are installed), or when a
//! `.desktop` file references it (covers flatpak/snap-style launchers whose
//! wrapper isn't on `$PATH`).

use std::fs;
use std::path::Path;

use crate::catalog::{Category, spec};
use argvus_control_center_core::paths;

/// A parsed `.desktop` entry.
#[derive(Debug, Clone)]
pub struct DesktopFile {
  /// Desktop file id, e.g. `firefox.desktop`.
  pub id: String,
  /// `Name=` from the entry.
  pub name: String,
  /// First command token of `Exec=`, when parseable.
  pub exec_binary: Option<String>,
}

/// An installed application detected for a category.
#[derive(Debug, Clone)]
pub struct InstalledApp {
  pub binary: String,
  pub display: String,
  pub desktop_id: Option<String>,
  pub tui: bool,
}

impl InstalledApp {
  /// True when this app is the currently effective default.
  pub fn is_current(&self, current: Option<&str>) -> bool {
    current.is_some_and(|c| c == self.binary)
  }
}

/// `command -v binary`: true when `binary` resolves inside `$PATH`, inside the
/// private ARGVUS commands directory, or is an absolute path to an existing
/// executable. ARGVUS-owned commands are not always on `$PATH`, so the private
/// directory is checked as a fallback.
pub fn command_exists(binary: &str) -> bool {
  if binary.is_empty() {
    return false;
  }
  if binary.contains('/') {
    return is_executable(Path::new(binary));
  }
  let on_path = std::env::var("PATH").is_ok_and(|path_var| {
    std::env::split_paths(&path_var).any(|dir| {
      if dir.as_os_str().is_empty() {
        return false;
      }
      is_executable(&dir.join(binary))
    })
  });
  on_path || is_executable(&private_commands_dir().join(binary))
}

/// Directory holding the private ARGVUS executables. `ARGVUS_COMMANDS_DIR`
/// overrides it, matching the `argvus` dispatcher and the launcher scripts.
fn private_commands_dir() -> std::path::PathBuf {
  std::env::var_os("ARGVUS_COMMANDS_DIR")
    .filter(|dir| !dir.is_empty())
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|| std::path::PathBuf::from("/usr/lib/argvus/commands"))
}

/// Checks the condition represented by `is_executable` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn is_executable(path: &Path) -> bool {
  use std::os::unix::fs::PermissionsExt;
  fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// Scan all known `.desktop` locations (user dir first; `XDG_DATA_DIRS` after).
pub fn scan_desktop_files() -> Vec<DesktopFile> {
  let mut out: Vec<DesktopFile> = Vec::new();
  let mut dirs = vec![paths::user_applications_dir()];
  dirs.extend(paths::system_applications_dirs());

  let mut seen: Vec<String> = Vec::new();
  for dir in dirs {
    let Ok(entries) = fs::read_dir(&dir) else {
      continue;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      let Some(name) = path.file_name().and_then(|n| n.to_str()).map(String::from) else {
        continue;
      };
      if !name.ends_with(".desktop") || seen.contains(&name) {
        continue;
      }
      if let Some(df) = parse_desktop_file(&name, &path) {
        seen.push(name);
        out.push(df);
      }
    }
  }
  out
}

/// Converts input data into `parse_desktop_file` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_desktop_file(id: &str, path: &Path) -> Option<DesktopFile> {
  let content = fs::read_to_string(path).ok()?;
  let mut in_entry = false;
  let mut name: Option<String> = None;
  let mut exec: Option<String> = None;
  let mut is_application = false;

  for raw in content.lines() {
    let line = raw.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    if line.starts_with('[') && line.ends_with(']') {
      in_entry = line == "[Desktop Entry]";
    }
    if !in_entry {
      continue;
    }
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    let key = key.trim();
    let value = value.trim();
    match key {
      "Type" => is_application = value == "Application",
      "Name" | "Name[en]" => {
        name.get_or_insert_with(|| value.to_string());
      }
      "Exec" => exec = Some(value.to_string()),
      _ => {}
    }
  }

  if !is_application {
    return None;
  }
  Some(DesktopFile {
    id: id.to_string(),
    name: name.unwrap_or_else(|| id.to_string()),
    exec_binary: exec.as_deref().and_then(exec_binary),
  })
}

/// Extract the command binary from a `.desktop` `Exec=` line.
///
/// Handles quoting, `env KEY=VALUE ...` prefixes and trailing field codes.
pub fn exec_binary(exec: &str) -> Option<String> {
  let exec = exec.trim();
  if exec.is_empty() {
    return None;
  }
  let tokens = tokenize_exec(exec);
  let iter = tokens.into_iter();
  let mut cmd: Option<String> = None;
  for tok in iter {
    if tok == "env" || tok.ends_with("/env") {
      continue;
    }
    if tok.contains('=') && !tok.eq_ignore_ascii_case("x-scheme-handler") {
      continue;
    }
    cmd = Some(tok);
    break;
  }
  match cmd {
    Some(c) if c == "sh" || c == "/bin/sh" || c.is_empty() => None,
    Some(c) => Some(c),
    None => None,
  }
}

/// Split an `Exec=` line into arguments, honoring single/double quotes and
/// backslash escapes. Trailing `%` field codes (%U, %F, ...) are discarded.
fn tokenize_exec(line: &str) -> Vec<String> {
  let mut args: Vec<String> = Vec::new();
  let mut buf = String::new();
  let mut iter = line.chars().peekable();
  let mut active_quote: Option<char> = None;

  while let Some(c) = iter.next() {
    if c == '%' {
      break; // drop %-field codes and anything after them
    }
    match active_quote {
      Some(_) if c == '\\' => {
        if let Some(c2) = iter.next() {
          buf.push(c2);
        }
      }
      Some(q) if c == q => active_quote = None,
      Some(_) => buf.push(c),
      None => match c {
        '\'' | '"' => active_quote = Some(c),
        c if c.is_whitespace() => {
          if !buf.is_empty() {
            args.push(std::mem::take(&mut buf));
          }
        }
        _ => buf.push(c),
      },
    }
  }
  if !buf.is_empty() {
    args.push(buf);
  }
  args
}

/// Leaf name of an `Exec=` token without any leading path.
fn entry_basename(token: &str) -> &str {
  Path::new(token)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or(token)
}

/// Resolve the display name for a known app from a detected desktop entry.
///
/// `Exec=` tokens are often absolute paths (`/usr/bin/kitty`) while the catalog
/// uses plain binaries (`kitty`); compare both the full token and its basename.
fn desktop_for<'a>(
  binaries: &[&str],
  preferred_id: &str,
  desktops: &'a [DesktopFile],
) -> Option<&'a DesktopFile> {
  desktops
    .iter()
    .find(|desktop| !preferred_id.is_empty() && desktop.id.eq_ignore_ascii_case(preferred_id))
    .or_else(|| {
      binaries.iter().find_map(|binary| {
        let expected = format!("{}.desktop", entry_basename(binary));
        desktops
          .iter()
          .find(|desktop| desktop.id.eq_ignore_ascii_case(&expected))
      })
    })
    .or_else(|| {
      desktops.iter().find(|d| {
        d.exec_binary
          .as_deref()
          .is_some_and(|b| binaries.contains(&b) || binaries.contains(&entry_basename(b)))
      })
    })
}

/// Installed apps for a category.
///
/// `current` is the currently effective default (used only for highlighting).
pub fn installed_apps(category: Category, desktops: &[DesktopFile]) -> Vec<InstalledApp> {
  let mut apps = Vec::new();
  for known in spec(category).apps {
    let found = desktop_for(&[known.binary], known.desktop_id, desktops);
    let present = command_exists(known.binary) || found.is_some();
    if !present {
      continue;
    }
    let desktop_id = found.map(|d| d.id.clone()).or_else(|| {
      if !known.desktop_id.is_empty() {
        Some(known.desktop_id.to_string())
      } else {
        None
      }
    });
    let display = found
      .map(|d| d.name.clone())
      .unwrap_or_else(|| known.display.to_string());
    apps.push(InstalledApp {
      binary: known.binary.to_string(),
      display,
      desktop_id,
      tui: known.tui,
    });
  }
  apps.sort_by(|a, b| a.display.cmp(&b.display));
  apps
}

/// Resolve the best desktop id for a binary across the whole catalog.
pub fn desktop_id_for_binary(binary: &str, desktops: &[DesktopFile]) -> Option<String> {
  for cat in Category::ORDER {
    for known in spec(cat).apps {
      if known.binary == binary {
        if let Some(d) = desktop_for(&[binary], known.desktop_id, desktops) {
          return Some(d.id.clone());
        }
        if !known.desktop_id.is_empty() {
          return Some(known.desktop_id.to_string());
        }
      }
    }
  }
  None
}

/// True when any desktop entry (or a `$PATH` match) references `binary`.
pub fn binary_available(binary: &str, desktops: &[DesktopFile]) -> bool {
  let base = Path::new(binary)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or(binary);
  command_exists(binary)
    || desktops.iter().any(|d| {
      d.exec_binary.as_deref().is_some_and(|b| {
        b == binary || Path::new(b).file_name().and_then(|n| n.to_str()) == Some(base)
      })
    })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `exec_binary_plain` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn exec_binary_plain() {
    assert_eq!(exec_binary("firefox %U").as_deref(), Some("firefox"));
    assert_eq!(
      exec_binary("nautilus --new-window %U").as_deref(),
      Some("nautilus")
    );
  }

  #[test]
  /// Executes the `exec_binary_env_prefix` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn exec_binary_env_prefix() {
    assert_eq!(
      exec_binary("env GTK_THEME=dark firefox %U").as_deref(),
      Some("firefox")
    );
  }

  #[test]
  /// Executes the `exec_binary_quoted` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn exec_binary_quoted() {
    assert_eq!(
      exec_binary("\"/usr/bin/min\" https:// %U").as_deref(),
      Some("/usr/bin/min")
    );
  }

  #[test]
  /// Executes the `exec_binary_shell_wrappers_ignored` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn exec_binary_shell_wrappers_ignored() {
    assert_eq!(exec_binary("sh -c 'exec kitty %U'"), None);
    assert_eq!(exec_binary(""), None);
  }

  #[test]
  /// Executes the `command_exists_truthy_for_env` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn command_exists_truthy_for_env() {
    assert!(command_exists("sh"));
    assert!(!command_exists("argvus-definitely-not-a-command-xyz"));
  }

  #[test]
  /// Executes the `command_exists_finds_private_commands` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn command_exists_finds_private_commands() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!(
      "argvus-apps-private-commands-{}",
      std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let binary = dir.join("argvus-detect-test-private");
    fs::write(&binary, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();

    unsafe {
      std::env::set_var("ARGVUS_COMMANDS_DIR", &dir);
    }
    assert!(command_exists("argvus-detect-test-private"));
    assert!(!command_exists("argvus-detect-test-missing"));
    unsafe {
      std::env::remove_var("ARGVUS_COMMANDS_DIR");
    }

    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  /// Executes the `desktop_id_resolution_uses_scan` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn desktop_id_resolution_uses_scan() {
    let id = desktop_id_for_binary("kitty", &[]);
    assert_eq!(id.as_deref(), Some("org.kitt.humans.kitty.desktop"));
    let none = desktop_id_for_binary("does-not-exist-app", &[]);
    assert_eq!(none, None);
  }

  #[test]
  /// Executes the `preferred_catalog_desktop_wins_over_wrapper_exec_match` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn preferred_catalog_desktop_wins_over_wrapper_exec_match() {
    let desktops = vec![
      DesktopFile {
        id: "wrapper.desktop".into(),
        name: "Wrapped Tool".into(),
        exec_binary: Some("kitty".into()),
      },
      DesktopFile {
        id: "kitty.desktop".into(),
        name: "Kitty".into(),
        exec_binary: Some("kitty".into()),
      },
    ];
    let app = installed_apps(Category::Terminal, &desktops)
      .into_iter()
      .find(|app| app.binary == "kitty")
      .unwrap();
    assert_eq!(app.display, "Kitty");
    assert_eq!(app.desktop_id.as_deref(), Some("kitty.desktop"));
  }

  #[test]
  /// Converts input data into `parse_desktop_file_requires_application_type` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parse_desktop_file_requires_application_type() {
    let dir = std::env::temp_dir().join("argvus-default-apps-detect-test");
    fs::create_dir_all(&dir).unwrap();
    let app_path = dir.join("app.desktop");
    let link_path = dir.join("link.desktop");

    fs::write(
      &app_path,
      "[Desktop Entry]\nType=Application\nName=Browser\nExec=firefox %U\n",
    )
    .unwrap();
    fs::write(
      &link_path,
      "[Desktop Entry]\nType=Link\nName=Docs\nExec=firefox %U\n",
    )
    .unwrap();

    let app = parse_desktop_file("app.desktop", &app_path).unwrap();
    assert_eq!(app.name, "Browser");
    assert_eq!(app.exec_binary.as_deref(), Some("firefox"));
    assert!(parse_desktop_file("link.desktop", &link_path).is_none());
  }
}
