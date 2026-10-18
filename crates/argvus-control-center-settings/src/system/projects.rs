//! Project roots and paths used by `argvus-projects` (SUPER + O).
//!
//! The launcher owns the config keys and the validation, so this module only
//! calls its CLI and never writes `argvus-config` itself.
use argvus_control_center_core::process::command;

/// Whether an entry is a directory that holds projects or one project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectSource {
  Root,
  Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEntry {
  pub source: ProjectSource,
  /// The value as stored in the config, for example `~/Projects`.
  pub path: String,
}

/// The loaded entries. `available` is false when the launcher is not installed,
/// so the page can say so instead of showing an empty list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Projects {
  pub available: bool,
  pub entries: Vec<ProjectEntry>,
}

/// Loads the entries from `argvus-projects list`.
pub fn load() -> Projects {
  let Ok(output) = command("argvus").args(["projects", "list"]).output() else {
    return Projects::default();
  };
  if !output.status.success() {
    return Projects::default();
  }
  Projects {
    available: true,
    entries: parse_list(&String::from_utf8_lossy(&output.stdout)),
  }
}

/// Parses the `kind<TAB>value` lines printed by `argvus-projects list`.
/// Unknown lines are ignored.
pub fn parse_list(output: &str) -> Vec<ProjectEntry> {
  output
    .lines()
    .filter_map(|line| {
      let (kind, path) = line.split_once('\t')?;
      let source = match kind {
        "root" => ProjectSource::Root,
        "path" => ProjectSource::Path,
        _ => return None,
      };
      let path = path.trim();
      (!path.is_empty()).then(|| ProjectEntry {
        source,
        path: path.to_owned(),
      })
    })
    .collect()
}

/// Adds a project directory, or a root whose subdirectories are projects.
pub fn add(path: &str, root: bool) -> Result<(), String> {
  let path = path.trim();
  if path.is_empty() {
    return Err("empty project path".into());
  }
  let mut args = vec!["add"];
  if root {
    args.push("--root");
  }
  args.push(path);
  run(&args)
}

/// Removes a project or root by the value stored in the config.
pub fn remove(path: &str) -> Result<(), String> {
  run(&["remove", path])
}

fn run(args: &[&str]) -> Result<(), String> {
  let output = command("argvus")
    .arg("projects")
    .args(args)
    .output()
    .map_err(|error| format!("failed to run argvus-projects: {error}"))?;
  if output.status.success() {
    return Ok(());
  }
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
  Err(if stderr.is_empty() {
    "argvus-projects failed".into()
  } else {
    stderr
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_roots_and_paths_and_skips_unknown_lines() {
    let entries = parse_list("root\t~/Projects\npath\t/home/u/Work/api\nnoise\nroot\t\n");
    assert_eq!(
      entries,
      vec![
        ProjectEntry {
          source: ProjectSource::Root,
          path: "~/Projects".into(),
        },
        ProjectEntry {
          source: ProjectSource::Path,
          path: "/home/u/Work/api".into(),
        },
      ]
    );
  }

  #[test]
  fn empty_output_gives_no_entries() {
    assert!(parse_list("").is_empty());
  }

  #[test]
  fn add_rejects_an_empty_path_without_running_the_launcher() {
    assert_eq!(add("   ", false), Err("empty project path".into()));
  }
}
