//! Implements default-application persistence in crate `argvus control center apps`. This separation keeps external effects from contaminating models, routes, or rendering.
//! nudge the running Argvus session to re-read the state.

use std::fs;
use std::process::Stdio;

use crate::catalog::{Category, find_app};
use crate::detect::{self, DesktopFile};
use argvus_control_center_core::{i18n::label, paths};

/// What `apply` did, so the CLI/GUI can report useful feedback.
#[derive(Debug, Default)]
pub struct ApplyReport {
  /// `.desktop` id used for XDG/MIME registration (if any).
  pub desktop_id: Option<String>,
  /// `mimeapps.list` was updated (path).
  pub mimeapps_updated: Option<std::path::PathBuf>,
  /// `xdg-settings set default-web-browser` attempted (browser only).
  pub xdg_settings: bool,
  /// `hyprctl reload` requested so Argvus re-reads the state.
  pub hyprctl_reloaded: bool,
  /// Notification posted to the session.
  pub notified: bool,
}

/// Entry point: persist + apply a category value.
///
/// `desktops` is the desktop-file index (used to resolve a `.desktop` id when
/// the catalog does not ship one for the picked binary).
pub fn apply(cat: Category, binary: &str, desktops: &[DesktopFile]) -> Result<ApplyReport, String> {
  let mut report = ApplyReport::default();

  let desktop_id = resolve_desktop_id(cat, binary, desktops);

  if cat.uses_xdg() {
    let Some(id) = &desktop_id else {
      return Err(format!(
        "cannot apply {}: no .desktop id known for '{binary}'",
        cat.key()
      ));
    };
    report.desktop_id = Some(id.clone());

    let associations: Vec<(String, String)> = cat
      .mimes()
      .iter()
      .map(|m| (m.to_string(), id.clone()))
      .collect();

    if cat == Category::Browser {
      report.xdg_settings = run_xdg_settings(id);
    }
    report.mimeapps_updated = update_mimeapps(&associations);
  }

  report.hyprctl_reloaded = refresh_argvus();
  let category_label = match cat {
    Category::Terminal => "control_center.terminal",
    Category::FileManager => "control_center.file_manager",
    Category::TextEditor => "control_center.text_editor",
    Category::TerminalEditor => "control_center.terminal_editor",
    Category::Browser => "control_center.browser",
    Category::ImageViewer => "control_center.image_viewer",
    Category::PdfViewer => "control_center.pdf_viewer",
    Category::VideoPlayer => "control_center.video_player",
    Category::AudioPlayer => "control_center.audio_player",
    Category::Archive => "control_center.archive",
    Category::Launcher => "control_center.launcher",
  };
  report.notified = notify_user(&format!("{} → {binary}", label(category_label)));

  Ok(report)
}

/// `xdg-settings set default-web-browser <desktop-id>`.
fn run_xdg_settings(desktop_id: &str) -> bool {
  if !detect::command_exists("xdg-settings") {
    return false;
  }
  argvus_control_center_core::process::command("xdg-settings")
    .args(["set", "default-web-browser", desktop_id])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|s| s.success())
    .unwrap_or(false)
}

/// Executes the `resolve_desktop_id` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn resolve_desktop_id(cat: Category, binary: &str, desktops: &[DesktopFile]) -> Option<String> {
  // Prefer the desktop file actually installed on this system, then fall back
  // to the catalog. Distros may ship a different id than upstream examples.
  let detected_id = detect::desktop_id_for_binary(binary, desktops);
  let catalog_id = find_app(cat, binary)
    .filter(|a| !a.desktop_id.is_empty())
    .map(|a| a.desktop_id.to_string());

  detected_id.or(catalog_id)
}

/// Merge `(mime, desktop_id)` associations into the user `mimeapps.list`.
fn update_mimeapps(associations: &[(String, String)]) -> Option<std::path::PathBuf> {
  let path = paths::mimeapps_list();
  let existing = fs::read_to_string(&path).unwrap_or_default();
  let updated = update_default_applications(&existing, associations);
  if updated == existing {
    return None;
  }
  if let Some(parent) = path.parent() {
    let _ = fs::create_dir_all(parent);
  }
  fs::write(&path, updated.as_bytes()).ok()?;
  Some(path)
}

/// Pure function updating the `[Default Applications]` section of a
/// `mimeapps.list` content string. Existing entries for the same mime types are
/// replaced in place; other sections are preserved verbatim.
pub fn update_default_applications(content: &str, associations: &[(String, String)]) -> String {
  /// Defines the constant `HEADER`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const HEADER: &str = "[Default Applications]";

  let mut lines: Vec<String> = content.lines().map(str::to_string).collect();

  // Locate the header and the end of its section.
  let mut header_idx: Option<usize> = None;
  let mut section_end = lines.len();
  let mut cur = String::new();
  for (i, l) in lines.iter().enumerate() {
    let t = l.trim();
    if t.starts_with('[') {
      if cur == HEADER {
        section_end = i;
        break;
      }
      cur = t.to_string();
      if t == HEADER {
        header_idx = Some(i);
      }
    }
  }

  // Keep pre-existing lines inside the section that we do not override.
  let mut keep: Vec<String> = Vec::new();
  if let Some(h) = header_idx {
    for l in lines.iter().take(section_end).skip(h + 1) {
      let replaced = l
        .split_once('=')
        .is_some_and(|(m, _)| associations.iter().any(|(a, _)| a == m.trim()));
      if !replaced {
        keep.push(l.clone());
      }
    }
  }

  // Assemble the new section block.
  let mut block: Vec<String> = Vec::new();
  block.push(HEADER.to_string());
  block.extend(keep);
  for (mime, id) in associations {
    let already = block.iter().any(|l| {
      l.split_once('=')
        .is_some_and(|(k, _)| k.trim() == mime.as_str())
    });
    if !already {
      block.push(format!("{mime}={id};"));
    }
  }

  match header_idx {
    Some(h) => {
      lines.splice(h..section_end, block);
    }
    None => {
      lines.push(HEADER.to_string());
      block.iter().skip(1).for_each(|l| lines.push(l.clone()));
    }
  }

  lines.join("\n") + "\n"
}

/// Tell the running Argvus session to re-read the defaults state.
///
/// Hyprland reloads `hyprland.lua` (terminal/file manager/launcher/browser
/// bindings). Consumer shell scripts read the state lazily on each run, so
/// they always see the newest values without an explicit callback.
pub fn refresh_argvus() -> bool {
  if !detect::command_exists("hyprctl") {
    return false;
  }
  argvus_control_center_core::process::command("hyprctl")
    .args(["reload"])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|s| s.success())
    .unwrap_or(false)
}

/// A user-facing notification about the change (best effort).
fn notify_user(body: &str) -> bool {
  if !detect::command_exists("notify-send") {
    return false;
  }
  argvus_control_center_core::process::command("notify-send")
    .args(["Default Programs", body])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|s| s.success())
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Executes the `assoc` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn assoc(mime: &str, id: &str) -> (String, String) {
    (mime.to_string(), id.to_string())
  }

  #[test]
  /// Executes the `adds_section_when_missing` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn adds_section_when_missing() {
    let input = "# comment\nsome=thing\n";
    let result = update_default_applications(input, &[assoc("application/pdf", "x.desktop")]);
    assert!(result.contains("[Default Applications]"));
    assert!(result.contains("application/pdf=x.desktop;"));
    assert!(result.contains("some=thing"));
  }

  #[test]
  /// Executes the `replaces_existing_entry` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn replaces_existing_entry() {
    let input =
      "[Default Applications]\napplication/pdf=old.desktop;\n\n[Added Associations]\nfoo=bar\n";
    let result = update_default_applications(input, &[assoc("application/pdf", "new.desktop")]);
    assert!(result.contains("application/pdf=new.desktop;"));
    assert!(!result.contains("old.desktop"));
    assert!(result.contains("[Added Associations]"));
    assert!(result.contains("foo=bar"));
  }

  #[test]
  /// Executes the `one_section_per_mime` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn one_section_per_mime() {
    let input = "[Default Applications]\n";
    let result = update_default_applications(
      input,
      &[
        assoc("video/mp4", "player.desktop"),
        assoc("audio/mpeg", "player.desktop"),
      ],
    );
    assert_eq!(result.matches("player.desktop").count(), 2);
  }

  #[test]
  /// Executes the `preserves_other_sections` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn preserves_other_sections() {
    let input =
      "[Default Applications]\naudio/mpeg=old.desktop;\n\n[Removed Associations]\nx=kill;\n";
    let result = update_default_applications(input, &[assoc("audio/mpeg", "new.desktop")]);
    assert!(result.contains("audio/mpeg=new.desktop;"));
    assert!(result.contains("[Removed Associations]\nx=kill;"));
  }

  #[test]
  /// Executes the `noop_when_unchanged` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn noop_when_unchanged() {
    let input = "[Default Applications]\ntext/plain=e.desktop;\n";
    let result = update_default_applications(input, &[assoc("text/plain", "e.desktop")]);
    assert_eq!(trimmable(&result), trimmable(input));
  }

  #[test]
  /// Executes the `installed_desktop_id_wins_over_catalog` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn installed_desktop_id_wins_over_catalog() {
    let desktops = vec![DesktopFile {
      id: "org.example.Ristretto.desktop".to_string(),
      name: "Ristretto".to_string(),
      exec_binary: Some("ristretto".to_string()),
    }];

    assert_eq!(
      resolve_desktop_id(Category::ImageViewer, "ristretto", &desktops).as_deref(),
      Some("org.example.Ristretto.desktop")
    );
  }

  /// Executes the `trimmable` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn trimmable(s: &str) -> String {
    s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n")
  }
}
