//! The development block: the active project, its git state, the listening
//! ports and the running containers.

use argvus_runtime::messages;
use argvus_runtime::process::{output, output_any, program_exists, read_text, run_quiet};
use argvus_widget_telemetry_core::ports::{parse_ss, shown};
use std::env;
use std::path::{Path, PathBuf};

/// The width of the label column.
const LABEL_WIDTH: usize = 11;

/// The text of the block, one line per fact.
pub fn text() -> String {
  let mut lines = Vec::new();
  if let Some(project) = active_project() {
    lines.push(row(
      &messages::text("label.project", &[]),
      &project.file_name().unwrap_or_default().to_string_lossy(),
    ));
    lines.push(git_row(&project));
  } else {
    lines.push(messages::text("dev_dashboard.no_project", &[]));
  }
  lines.push(ports_row());
  lines.push(containers_row());
  lines.join("\n")
}

/// The project the desktop has active, when its directory exists.
fn active_project() -> Option<PathBuf> {
  let runtime_directory = env::var("XDG_RUNTIME_DIR")
    .ok()
    .filter(|value| !value.is_empty())
    .unwrap_or_else(|| "/tmp".to_string());
  let file = Path::new(&runtime_directory).join("argvus/active-project");
  let first_line = read_text(&file)?.lines().next()?.to_string();
  let project = PathBuf::from(first_line);
  project.is_dir().then_some(project)
}

fn git_row(project: &Path) -> String {
  let directory = project.to_string_lossy();
  if !run_quiet(
    "git",
    &["-C", &directory, "rev-parse", "--is-inside-work-tree"],
  ) {
    return row(
      &messages::text("label.branch", &[]),
      &messages::text("dev_dashboard.not_git", &[]),
    );
  }
  let branch = output(
    "git",
    &["-C", &directory, "rev-parse", "--abbrev-ref", "HEAD"],
  )
  .unwrap_or_default();
  let dirty = !output_any("git", &["-C", &directory, "status", "--porcelain"]).is_empty();
  let state = if dirty {
    messages::text("label.dirty", &[])
  } else {
    messages::text("label.clean", &[])
  };
  format!(
    "{:<LABEL_WIDTH$} {} ({})",
    messages::text("label.branch", &[]),
    branch.trim(),
    state,
  )
}

fn ports_row() -> String {
  let ports = parse_ss(&output_any("ss", &["-tln"]));
  format!(
    "{:<LABEL_WIDTH$} {} {}",
    messages::text("label.ports", &[]),
    ports.len(),
    shown(&ports),
  )
}

fn containers_row() -> String {
  let label = messages::text("label.containers", &[]);
  let engines: Vec<&str> = ["podman", "docker"]
    .into_iter()
    .filter(|engine| program_exists(engine))
    .collect();
  if engines.is_empty() {
    return row(
      &label,
      &messages::text("dev_dashboard.containers_unavailable", &[]),
    );
  }
  let running: usize = engines
    .iter()
    .map(|engine| output_any(engine, &["ps", "-q"]).lines().count())
    .sum();
  row(&label, &running.to_string())
}

fn row(label: &str, value: &str) -> String {
  format!("{label:<LABEL_WIDTH$} {value}")
}
