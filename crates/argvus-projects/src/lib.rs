//! `argvus-projects`: project switcher of ARGVUS. The public command is
//! `argvus projects`; the binary is a private command of the ARGVUS dispatcher.
//!
//! Usage:
//!   argvus-projects                    pick a project (SUPER + O)
//!   argvus-projects add [--root] DIR   add a project, or a root holding projects
//!   argvus-projects remove DIR         remove a project or root
//!   argvus-projects list               print configured roots and paths
//!   argvus-projects pick [PROMPT]      print the path picked from the list, for other tools
//!   argvus-projects open SELECTOR      open a project by position or name

mod config;
mod messages;

use argvus_projects_core::projects::{self, MAX_PROJECTS};
use messages::{Stream, print_launcher, report_error};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

/// Runs one command of this crate. `arguments` are the words after the command
/// name. The `argvus` dispatcher and the binary both call this function.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  run(&arguments)
}

fn run(arguments: &[String]) -> ExitCode {
  match arguments.first().map(String::as_str) {
    Some("open") => open(&arguments[1..]),
    Some("add") => add(&arguments[1..]),
    Some("remove") => remove(&arguments[1..]),
    Some("list") => list(),
    Some("pick") => pick_command(&arguments[1..]),
    Some("-V" | "--version") => {
      println!("argvus-projects {}", env!("CARGO_PKG_VERSION"));
      ExitCode::SUCCESS
    }
    // Any other word, like no word at all, opens the picker: the script
    // ignored its arguments the same way.
    _ => switch(),
  }
}

fn home() -> String {
  env::var("HOME").unwrap_or_default()
}

/// The configured roots and paths, expanded and existing, merged in order.
fn collect_projects() -> Vec<String> {
  let mut projects = Vec::new();
  for root in config::list("/projects/roots") {
    let root = projects::expand_home(&root, &home());
    projects.extend(projects::scan_root(&PathBuf::from(root)));
  }
  for path in config::list("/projects/paths") {
    let path = projects::expand_home(&path, &home());
    if PathBuf::from(&path).is_dir() {
      projects.push(path.trim_end_matches('/').to_string());
    }
  }
  projects::unique_in_order(projects)
}

/// Prints a canonical directory, or reports why it cannot be a project.
fn normalize_dir(value: &str) -> Result<String, ExitCode> {
  let expanded = projects::expand_home(value, &home());
  match fs::canonicalize(&expanded) {
    Ok(path) if path.is_dir() => Ok(path.to_string_lossy().into_owned()),
    _ => {
      print_launcher(
        "error.not_directory",
        &[format!("path={value}")],
        Stream::Stderr,
      );
      Err(ExitCode::from(1))
    }
  }
}

fn add(arguments: &[String]) -> ExitCode {
  let (pointer, value) = match arguments {
    [flag, value] if flag == "--root" => ("/projects/roots", value),
    [value] if value != "--root" => ("/projects/paths", value),
    _ => {
      print_launcher("error.usage", &[], Stream::Stderr);
      return ExitCode::from(1);
    }
  };
  let dir = match normalize_dir(value) {
    Ok(dir) => dir,
    Err(code) => return code,
  };
  // The count the list would have with the new entry, checked before writing.
  let mut candidates = collect_projects();
  if pointer == "/projects/roots" {
    candidates.extend(projects::scan_root(&PathBuf::from(&dir)));
  } else {
    candidates.push(dir.clone());
  }
  let total = projects::unique_in_order(candidates).len();
  if total > MAX_PROJECTS {
    print_launcher(
      "error.project_limit_reached",
      &[format!("limit={MAX_PROJECTS}")],
      Stream::Stderr,
    );
    return ExitCode::from(1);
  }
  let mut entries = config::array(pointer);
  entries.push(dir.clone());
  // `jq unique` sorted the array, so the stored order is sorted too.
  entries.sort();
  entries.dedup();
  if !config::set_array(pointer, &entries) {
    return ExitCode::from(1);
  }
  print_launcher("message.added", &[format!("path={dir}")], Stream::Stdout);
  ExitCode::SUCCESS
}

fn remove(arguments: &[String]) -> ExitCode {
  let [value] = arguments else {
    print_launcher("error.usage", &[], Stream::Stderr);
    return ExitCode::from(1);
  };
  let mut dir = projects::expand_home(value, &home());
  if let Ok(path) = fs::canonicalize(&dir) {
    dir = path.to_string_lossy().into_owned();
  }
  for pointer in ["/projects/roots", "/projects/paths"] {
    let kept: Vec<String> = config::array(pointer)
      .into_iter()
      .filter(|entry| *entry != dir && entry != value)
      .collect();
    if !config::set_array(pointer, &kept) {
      return ExitCode::from(1);
    }
  }
  print_launcher(
    "message.removed",
    &[format!("path={value}")],
    Stream::Stdout,
  );
  ExitCode::SUCCESS
}

fn list() -> ExitCode {
  for root in config::list("/projects/roots") {
    println!("root\t{root}");
  }
  for path in config::list("/projects/paths") {
    println!("path\t{path}");
  }
  ExitCode::SUCCESS
}

/// The picker: a list of the projects; prints nothing when it is cancelled.
fn pick_chosen(prompt: &str) -> Result<Option<String>, ExitCode> {
  let list = collect_projects();
  if list.is_empty() {
    report_error(&messages::text("error.no_projects", &[]));
    return Err(ExitCode::from(1));
  }
  let labels: Vec<String> = list
    .iter()
    .enumerate()
    .map(|(index, path)| projects::picker_label(index + 1, path))
    .collect();
  let config = config::read_path("launcher/config/config.rasi");
  let mut child = match Command::new("rofi")
    .arg("-config")
    .arg(config)
    .args(["-dmenu", "-i", "-format", "i", "-p", prompt])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
  {
    Ok(child) => child,
    Err(_) => return Ok(None),
  };
  if let Some(mut stdin) = child.stdin.take() {
    let _ = stdin.write_all(labels.join("\n").as_bytes());
  }
  let Ok(output) = child.wait_with_output() else {
    return Ok(None);
  };
  if !output.status.success() {
    return Ok(None);
  }
  let index: Option<usize> = String::from_utf8_lossy(&output.stdout).trim().parse().ok();
  Ok(index.and_then(|index| list.get(index).cloned()))
}

fn pick_command(arguments: &[String]) -> ExitCode {
  let prompt = match arguments.first() {
    Some(prompt) => prompt.clone(),
    None => messages::text("search.project", &[]),
  };
  match pick_chosen(&prompt) {
    Ok(Some(path)) => {
      println!("{path}");
      ExitCode::SUCCESS
    }
    Ok(None) => ExitCode::SUCCESS,
    Err(code) => code,
  }
}

/// The shortcut: pick a project and open it.
fn switch() -> ExitCode {
  let prompt = messages::text("search.project", &[]);
  match pick_chosen(&prompt) {
    Ok(Some(path)) => open_path(&path),
    Ok(None) => ExitCode::SUCCESS,
    Err(code) => code,
  }
}

fn open(arguments: &[String]) -> ExitCode {
  let selector = arguments.first().map(String::as_str).unwrap_or("");
  let list = collect_projects();
  if list.is_empty() {
    report_error(&messages::text("error.no_projects", &[]));
    return ExitCode::from(1);
  }
  let Some(path) = projects::resolve(&list, selector) else {
    report_error(&messages::text(
      "error.project_not_found",
      &[format!("name={selector}")],
    ));
    return ExitCode::from(1);
  };
  open_path(&path)
}

/// Focuses the workspace of a project, and on its first opening starts the
/// terminal and the editor in it. The active project is recorded for telemetry.
fn open_path(path: &str) -> ExitCode {
  let workspace = projects::workspace_name(path);
  let created = !workspace_exists(&workspace);
  if dispatch(&format!(
    "hl.dsp.focus({{ workspace = {} }})",
    projects::lua_string(&format!("name:{workspace}"))
  ))
  .is_err()
  {
    return ExitCode::from(1);
  }
  write_active_project(path);
  if !created {
    return ExitCode::SUCCESS;
  }
  let terminal = format!(
    "argvus terminal --directory {}",
    projects::shell_quote(path)
  );
  if dispatch(&format!(
    "hl.dsp.exec_cmd({})",
    projects::lua_string(&terminal)
  ))
  .is_err()
  {
    return ExitCode::from(1);
  }
  let editor = config::get("/projects/editor");
  if !editor.is_empty() && editor != "null" {
    let command = format!("{editor} {}", projects::shell_quote(path));
    if dispatch(&format!(
      "hl.dsp.exec_cmd({})",
      projects::lua_string(&command)
    ))
    .is_err()
    {
      return ExitCode::from(1);
    }
  }
  ExitCode::SUCCESS
}

/// Whether Hyprland already has the workspace. Without a reply it does not.
fn workspace_exists(name: &str) -> bool {
  let Ok(output) = Command::new("hyprctl").args(["-j", "workspaces"]).output() else {
    return false;
  };
  serde_json::from_slice::<serde_json::Value>(&output.stdout)
    .ok()
    .and_then(|value| value.as_array().cloned())
    .is_some_and(|workspaces| {
      workspaces
        .iter()
        .any(|workspace| workspace.get("name").and_then(|value| value.as_str()) == Some(name))
    })
}

/// Runs a Hyprland dispatch. A rejected dispatch prints its reason, which the
/// shortcut shows as a notification.
fn dispatch(lua: &str) -> Result<(), ()> {
  match Command::new("hyprctl").args(["dispatch", lua]).output() {
    Ok(output) if output.status.success() => Ok(()),
    Ok(output) => {
      let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
      text.push_str(&String::from_utf8_lossy(&output.stderr));
      report_error(text.trim_end());
      Err(())
    }
    Err(error) => {
      report_error(&error.to_string());
      Err(())
    }
  }
}

fn write_active_project(path: &str) {
  let file = projects::active_project_file(env::var("XDG_RUNTIME_DIR").ok().as_deref());
  if let Some(parent) = file.parent() {
    let _ = fs::create_dir_all(parent);
  }
  let _ = fs::write(file, format!("{path}\n"));
}
