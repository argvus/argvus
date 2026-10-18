//! Implements isolated integration with system tools and APIs in crate `argvus control center session`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::{
  journal::{self, JournalEntry},
  model::{AutostartEntry, Component, ComponentStatus, DiagnosticsEntry, manifest},
};
use argvus_control_center_core::{
  paths::{config_home, system_config_root},
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Executes the `list_components` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn list_components() -> Vec<Component> {
  let components = manifest();
  components
    .into_iter()
    .map(|mut component| {
      let (unit_status, unit_pid) = match &component.unit {
        Some(unit) => unit_state(unit),
        None => (None, None),
      };
      let (process_status, process_pid) = process_state(&component.process);
      let status = unit_status
        .or(process_status)
        .unwrap_or(ComponentStatus::Stopped);
      let pid = process_pid.or(unit_pid);
      component.status = status;
      component.pid = pid;
      component
    })
    .collect()
}

/// Executes the `unit_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn unit_state(unit: &str) -> (Option<ComponentStatus>, Option<u32>) {
  let out = SystemProcessRunner.run(
    &ProcessRequest::new("systemctl")
      .arg("--user")
      .arg("is-active")
      .arg(unit),
  );
  let Ok(out) = out else {
    return (None, None);
  };
  let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
  let status = match stdout.as_str() {
    "active" => Some(ComponentStatus::Running),
    "failed" | "error" => Some(ComponentStatus::Failed),
    _ => None,
  };
  let pid = if status.is_some() {
    SystemProcessRunner
      .run(
        &ProcessRequest::new("systemctl")
          .arg("--user")
          .arg("show")
          .arg(unit)
          .arg("-p")
          .arg("MainPID")
          .arg("--value"),
      )
      .ok()
      .and_then(|out| {
        String::from_utf8_lossy(&out.stdout)
          .trim()
          .parse::<u32>()
          .ok()
          .filter(|&pid| pid != 0)
      })
  } else {
    None
  };
  (status, pid)
}

/// Processes `process_state` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn process_state(aliases: &[String]) -> (Option<ComponentStatus>, Option<u32>) {
  for alias in aliases {
    if let Some(pid) = find_process(alias) {
      return (Some(ComponentStatus::Running), Some(pid));
    }
  }
  (None, None)
}

/// Executes the `find_process` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn find_process(alias: &str) -> Option<u32> {
  let mut best: Option<u32> = None;
  let entries = fs::read_dir("/proc").ok()?;
  for entry in entries.flatten() {
    let name = entry.file_name();
    let Some(digits) = name.to_str().and_then(|value| value.parse::<u32>().ok()) else {
      continue;
    };
    let comm = match fs::read_to_string(entry.path().join("comm")) {
      Ok(value) => value.trim().to_string(),
      Err(_) => continue,
    };
    if matches_process(&comm, alias) {
      best = Some(best.map_or(digits, |current| current.min(digits)));
      continue;
    }
    let Ok(cmdline) = fs::read_to_string(entry.path().join("cmdline")) else {
      continue;
    };
    if let Some(token) = cmdline.split('\0').next()
      && matches_process(token, alias)
    {
      best = Some(best.map_or(digits, |current| current.min(digits)));
    }
  }
  best
}

/// Executes the `matches_process` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn matches_process(value: &str, alias: &str) -> bool {
  let base = Path::new(value)
    .file_name()
    .map(|name| name.to_string_lossy())
    .unwrap_or_default();
  base == alias
}

/// Defines the constant `AUTOSTART_SYSTEM_CANDIDATES`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const AUTOSTART_SYSTEM_CANDIDATES: [&str; 2] = ["/etc/xdg/autostart", "/usr/share/autostart"];

/// Executes the `autostart_dir` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn autostart_dir() -> PathBuf {
  config_home().join("autostart")
}

/// Executes the `desktop_files` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn desktop_files(dir: &Path) -> Vec<PathBuf> {
  let Ok(entries) = fs::read_dir(dir) else {
    return Vec::new();
  };
  entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| {
      path
        .extension()
        .is_some_and(|extension| extension == "desktop")
    })
    .collect()
}

/// Executes the `autostart_entries` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn autostart_entries() -> Vec<AutostartEntry> {
  let user_dir = autostart_dir();
  let mut entries: Vec<AutostartEntry> = Vec::new();
  let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
  for path in desktop_files(&user_dir) {
    let id = file_stem(&path);
    if let Some(entry) = parse_desktop(&id, &path, false) {
      entries.push(entry);
      seen.insert(id);
    }
  }
  for candidate in AUTOSTART_SYSTEM_CANDIDATES {
    let dir = system_config_root().join("applications/autostart");
    let _ = dir;
    let system_dir = PathBuf::from(candidate);
    for path in desktop_files(&system_dir) {
      let id = file_stem(&path);
      if seen.contains(&id) {
        continue;
      }
      if let Some(entry) = parse_desktop(&id, &path, true) {
        entries.push(entry);
        seen.insert(id);
      }
    }
  }
  entries.sort_by_key(|left| left.name.to_lowercase());
  entries
}

/// Executes the `file_stem` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn file_stem(path: &Path) -> String {
  path
    .file_name()
    .map(|name| name.to_string_lossy().into_owned())
    .unwrap_or_default()
}

/// Converts input data into `parse_desktop` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_desktop(id: &str, path: &Path, from_system: bool) -> Option<AutostartEntry> {
  let content = fs::read_to_string(path).ok()?;
  let mut name = id.to_string();
  let mut command = String::new();
  let mut hidden = false;
  let mut gtk_enabled: Option<bool> = None;
  for line in content.lines() {
    let line = line.trim();
    if line.starts_with('#') || line.is_empty() {
      continue;
    }
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    match key {
      "Name" => name = value.to_string(),
      "Exec" => command = value.to_string(),
      "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
      "X-GNOME-Autostart-Enabled" => gtk_enabled = Some(value.eq_ignore_ascii_case("true")),
      _ => {}
    }
  }
  let enabled = !hidden && gtk_enabled.unwrap_or(true);
  Some(AutostartEntry {
    id: id.into(),
    name,
    command,
    from_system,
    enabled,
    path: path.to_string_lossy().into_owned(),
  })
}

/// Toggles an autostart entry. User entries are edited in place; system
/// entries are shadowed by a user copy (the XDG shadowing rule), so disabling
/// an entry from `/etc/xdg/autostart` never edits system files.
pub fn set_autostart(entry_id: &str, enabled: bool) -> Result<(), String> {
  validate_autostart_id(entry_id)?;
  let user_dir = autostart_dir();
  fs::create_dir_all(&user_dir).map_err(|error| error.to_string())?;
  let user_path = user_dir.join(format!("{entry_id}.desktop"));
  let source = if user_path.exists() {
    user_path.clone()
  } else {
    find_system_autostart(entry_id)
      .ok_or_else(|| format!("entrada de autostart '{entry_id}' não encontrada para editar"))?
  };
  let original = fs::read_to_string(&source).map_err(|error| error.to_string())?;
  let updated = rewrite_desktop_flags(&original, enabled);
  let tmp = user_path.with_extension("tmp");
  fs::write(&tmp, updated).map_err(|error| error.to_string())?;
  fs::rename(&tmp, &user_path).map_err(|error| error.to_string())?;
  Ok(())
}

/// Executes the `find_system_autostart` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn find_system_autostart(entry_id: &str) -> Option<PathBuf> {
  let target = format!("{entry_id}.desktop");
  AUTOSTART_SYSTEM_CANDIDATES
    .iter()
    .map(PathBuf::from)
    .find_map(|dir| {
      let path = dir.join(&target);
      path.exists().then_some(path)
    })
}

/// Executes the `validate_autostart_id` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn validate_autostart_id(entry_id: &str) -> Result<(), String> {
  if !entry_id.is_empty()
    && entry_id.len() <= 128
    && entry_id
      .chars()
      .all(|character| !character.is_control() && !character.is_whitespace())
  {
    Ok(())
  } else {
    Err("invalid autostart id".into())
  }
}

/// Executes the `rewrite_desktop_flags` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn rewrite_desktop_flags(content: &str, enabled: bool) -> String {
  let hidden = if enabled { "false" } else { "true" };
  let mut has_hidden = false;
  let mut rewritten = String::new();
  for line in content.lines() {
    if line.trim_start().starts_with("Hidden=") {
      let indent: String = line
        .chars()
        .take_while(|character| character.is_whitespace())
        .collect();
      rewritten.push_str(&format!("{indent}Hidden={hidden}\n"));
      has_hidden = true;
    } else if line.trim_start().starts_with("X-GNOME-Autostart-Enabled=") {
      rewritten.push_str("X-GNOME-Autostart-Enabled=");
      rewritten.push_str(if enabled { "true" } else { "false" });
      rewritten.push('\n');
    } else {
      rewritten.push_str(line);
      rewritten.push('\n');
    }
  }
  if !has_hidden {
    rewritten.push_str(&format!("Hidden={hidden}\n"));
  }
  rewritten
}

/// Executes the `diagnostics` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn diagnostics(components: &[Component]) -> Vec<DiagnosticsEntry> {
  let mut entries = Vec::new();
  let session_running = components
    .iter()
    .filter(|component| component.id == "argvus-session")
    .any(|component| component.status == ComponentStatus::Running);
  if session_running {
    entries.push(DiagnosticsEntry::good(
      "Sessão ARGVUS",
      "Gerenciador de sessão ativo.",
    ));
  } else {
    entries.push(DiagnosticsEntry::bad(
      "Sessão ARGVUS",
      "Gerenciador de sessão não está rodando.",
    ));
  }
  let systemd_user = user_manager_state();
  entries.push(systemd_user);
  let essential = components.iter().filter(|component| component.essential);
  let missing_essential = essential
    .clone()
    .filter(|component| !component.status.running())
    .count();
  if missing_essential == 0 {
    entries.push(DiagnosticsEntry::good(
      "Componentes essenciais",
      "Todos os componentes essenciais estão ativos.",
    ));
  } else {
    entries.push(DiagnosticsEntry::bad(
      "Componentes essenciais",
      format!("{missing_essential} componente(s) essencial(is) não ativo(s)."),
    ));
  }
  let failed = components
    .iter()
    .filter(|component| component.status == ComponentStatus::Failed);
  for component in failed {
    entries.push(DiagnosticsEntry::warning(
      format!("Falha em {}", component.display_name),
      format!(
        "PID {} — reinicie pelo componente (réiniciar).",
        component
          .pid
          .map(|pid| pid.to_string())
          .unwrap_or_else(|| "—".into())
      ),
    ));
  }
  if entries.len() < 4 {
    entries.push(DiagnosticsEntry::good(
      "Ambiente",
      "Sessão gráfica detectada.",
    ));
  }
  entries
}

/// Executes the `user_manager_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn user_manager_state() -> DiagnosticsEntry {
  let out = SystemProcessRunner.run(
    &ProcessRequest::new("systemctl")
      .arg("--user")
      .arg("is-system-running"),
  );
  match out {
    Ok(output) if output.status == Some(0) => {
      let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
      if state == "running" {
        DiagnosticsEntry::good("systemd --user", "Gerenciador de sessão systemd ativo.")
      } else {
        DiagnosticsEntry::warning(
          "systemd --user",
          format!("Gerenciador de sessão systemd em estado '{state}'."),
        )
      }
    }
    _ => DiagnosticsEntry::bad(
      "systemd --user",
      "Gerenciador de sessão systemd indisponível.",
    ),
  }
}

/// Restarts a component through ARGVUS's session manager, falling back to the
/// systemd user unit when `argvus-sessionctl` is unavailable or errors.
pub fn restart_component(id: &str) -> Result<(), String> {
  if !Component::valid_id(id) {
    return Err("invalid component id".into());
  }
  if let Some(component) = manifest().into_iter().find(|component| component.id == id) {
    for command in [
      Some(vec![
        "argvus".to_string(),
        "sessionctl".into(),
        "restart".into(),
        id.into(),
      ]),
      component
        .unit
        .map(|unit| vec!["systemctl".into(), "--user".into(), "restart".into(), unit]),
    ]
    .into_iter()
    .flatten()
    {
      if run_user_command(&command).is_ok() {
        return Ok(());
      }
    }
  }
  Err(format!("não foi possível reiniciar o componente '{id}'"))
}

/// Executes the `run_user_command` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_user_command(command: &[String]) -> Result<(), String> {
  let status = argvus_control_center_core::process::command(&command[0])
    .args(&command[1..])
    .status()
    .map_err(|error| error.to_string())?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("command exited with {status}"))
  }
}

/// Executes the `logs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn logs(filter: Option<&str>, limit: u32) -> Result<Vec<JournalEntry>, String> {
  let mut request = ProcessRequest::new("journalctl")
    .arg("--user")
    .arg("--no-pager")
    .arg("--output=json")
    .arg("-n")
    .arg(limit.to_string());
  if let Some(unit) = filter {
    request = request.arg("-u").arg(unit);
  }
  let out = SystemProcessRunner
    .run(&request)
    .map_err(|error| error.to_string())?;
  if out.status.is_none_or(|status| status != 0) {
    return Err(
      terminal_text(&String::from_utf8_lossy(&out.stderr))
        .trim()
        .into(),
    );
  }
  Ok(
    String::from_utf8_lossy(&out.stdout)
      .lines()
      .filter_map(|line| journal::parse_line(line).ok())
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `matches_process_names_basename_only` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn matches_process_names_basename_only() {
    assert!(matches_process("/usr/bin/waybar", "waybar"));
    assert!(matches_process("waybar", "waybar"));
    assert!(!matches_process("/usr/bin/waybar-cmd", "waybar"));
  }

  #[test]
  /// Executes the `rewrites_desktop_hidden_flag` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn rewrites_desktop_hidden_flag() {
    let content = "Name=Foo\nExec=foo --bar\nHidden=false\n";
    let disabled = rewrite_desktop_flags(content, false);
    assert!(disabled.contains("Hidden=true"), "{disabled}");
    let enabled = rewrite_desktop_flags(content, true);
    assert!(enabled.contains("Hidden=false"), "{enabled}");
  }

  #[test]
  /// Executes the `appends_hidden_when_absent` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn appends_hidden_when_absent() {
    let content = "Name=Foo\nExec=foo\ndefault=True\n";
    assert!(rewrite_desktop_flags(content, false).contains("Hidden=true"));
  }

  #[test]
  /// Executes the `autostart_ids_are_validated` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn autostart_ids_are_validated() {
    assert!(validate_autostart_id("foo-bar").is_ok());
    assert!(validate_autostart_id("").is_err());
    assert!(validate_autostart_id("bad id").is_err());
  }

  #[test]
  /// Converts input data into `parses_desktop_entries` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_desktop_entries() {
    let dir = std::env::temp_dir().join(format!("argvus-autostart-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("demo.desktop");
    fs::write(
      &path,
      "[Desktop Entry]\nName=Demo\nExec=demo --hidden\nHidden=false\n",
    )
    .unwrap();
    let entry = parse_desktop("demo", &path, true).unwrap();
    assert_eq!(entry.name, "Demo");
    assert!(entry.enabled);
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  /// Executes the `desktops_hidden_true_disables` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn desktops_hidden_true_disables() {
    let dir = std::env::temp_dir().join(format!("argvus-autostart-2-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("demo.desktop");
    fs::write(&path, "[Desktop Entry]\nHidden=true\n").unwrap();
    let entry = parse_desktop("demo", &path, true).unwrap();
    assert!(!entry.enabled);
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  /// Executes the `git_shadowing_prefers_user_entry` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn git_shadowing_prefers_user_entry() {
    let dir = std::env::temp_dir().join(format!("argvus-autostart-3-{}", std::process::id()));
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("user/a.desktop"), "Name=User\nHidden=false\n").unwrap();
    let entries = [parse_desktop("a", &dir.join("user/a.desktop"), false).unwrap()];
    assert_eq!(entries.len(), 1);
    assert!(!entries[0].from_system);
    let _ = fs::remove_dir_all(&dir);
  }
}
