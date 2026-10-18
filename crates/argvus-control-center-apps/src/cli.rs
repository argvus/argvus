//! Implements argument parsing and startup route selection in crate `argvus control center apps`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::process::ExitCode;

use argvus_control_center_core::i18n::label;
use argvus_control_center_core::paths;

use crate::apply;
use crate::catalog::{Category, effective_values, find_app};
use crate::detect;
use crate::state::AppState;

/// Defines the constant `PROG`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const PROG: &str = "argvus-control-center";
/// Defines the constant `CATEGORY_KEYS`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const CATEGORY_KEYS: &str = "terminal, file_manager, text_editor, terminal_editor, browser, image_viewer, pdf_viewer, video_player, audio_player, archive, launcher";

/// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn run(args: &[String]) -> ExitCode {
  let Some(cmd) = args.first().map(String::as_str) else {
    usage(false);
    return ExitCode::from(2);
  };

  match cmd {
    "help" | "--help" | "-h" => {
      usage(true);
      ExitCode::SUCCESS
    }
    "version" | "--version" | "-V" => {
      println!("{PROG} {}", env!("CARGO_PKG_VERSION"));
      ExitCode::SUCCESS
    }
    "list" => cmd_list(&args[1..]),
    "get" => cmd_get(&args[1..]),
    "set" => cmd_set(&args[1..]),
    "categories" => {
      for c in Category::ORDER {
        println!("{}", c.key());
      }
      ExitCode::SUCCESS
    }
    unknown => {
      eprintln!(
        "{PROG}: {} '{unknown}'",
        label("control_center.apps_unknown_command")
      );
      usage(false);
      ExitCode::from(2)
    }
  }
}

/// Executes the `bad_category` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn bad_category(key: &str) -> ExitCode {
  eprintln!(
    "{PROG}: {} '{key}' ({}: {CATEGORY_KEYS})",
    label("control_center.apps_unknown_category"),
    label("control_center.apps_valid")
  );
  ExitCode::from(2)
}

/// Executes the `cmd_list` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn cmd_list(args: &[String]) -> ExitCode {
  let desktops = detect::scan_desktop_files();
  let cats: Vec<Category> = match args.first() {
    None => Category::ORDER.to_vec(),
    Some(key) => match Category::from_key(key) {
      Some(c) => vec![c],
      None => return bad_category(key),
    },
  };

  let state = AppState::load();
  for cat in &cats {
    let current = state.effective(*cat);
    let apps = detect::installed_apps(*cat, &desktops);
    let title = if !current.is_empty() {
      format!("{} ({})", cat.key(), label("control_center.apps_current"))
    } else {
      format!(
        "{} ({})",
        cat.key(),
        label("control_center.apps_no_default")
      )
    };
    println!("[{title}]");
    if apps.is_empty() {
      println!("  ({})", label("control_center.apps_none_detected"));
      continue;
    }
    for app in &apps {
      let marker = if app.is_current(Some(&current)) {
        " *"
      } else {
        "  "
      };
      let id = app.desktop_id.as_deref().unwrap_or("");
      let tui = if app.tui { " [tui]" } else { "" };
      let mut line = format!("{marker} {:<20} {}{tui}", app.binary, app.display);
      if !id.is_empty() {
        line.push_str(&format!(" ({id})"));
      }
      println!("{line}");
    }
    println!();
  }
  ExitCode::SUCCESS
}

/// Executes the `cmd_get` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn cmd_get(args: &[String]) -> ExitCode {
  let state = AppState::load();
  match args.first() {
    None => {
      for (cat, value) in effective_values(&as_list(&state)) {
        println!("{}={value}", cat.key());
      }
      ExitCode::SUCCESS
    }
    Some(key) => match Category::from_key(key) {
      Some(cat) => {
        let value = state.effective(cat);
        if value.is_empty() {
          eprintln!(
            "{PROG}: {} '{key}' ({})",
            label("control_center.apps_no_default_set_for"),
            label("control_center.apps_falls_back_to_xdg")
          );
          ExitCode::from(1)
        } else {
          println!("{value}");
          ExitCode::SUCCESS
        }
      }
      None => bad_category(key),
    },
  }
}

/// Executes the `cmd_set` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn cmd_set(args: &[String]) -> ExitCode {
  if args.len() < 2 {
    eprintln!(
      "{}: {PROG} set <{}> <{}>",
      label("control_center.apps_usage"),
      label("control_center.apps_category"),
      label("control_center.apps_app")
    );
    return ExitCode::from(2);
  }
  let cat = match Category::from_key(&args[0]) {
    Some(c) => c,
    None => return bad_category(&args[0]),
  };
  let app_query = &args[1];

  let desktops = detect::scan_desktop_files();
  let binary = find_app(cat, app_query)
    .map(|a| a.binary.to_string())
    .unwrap_or_else(|| app_query.to_string());

  if binary == "default" {
    let mut state = AppState::load();
    state.set(cat, "");
    return match state.save() {
      Ok(()) => {
        let reloaded = apply::refresh_argvus();
        println!("{}={}", cat.key(), label("control_center.apps_default"));
        if reloaded {
          println!(
            "  ARGVUS: {}",
            label("control_center.apps_hyprctl_reload_requested")
          );
        }
        ExitCode::SUCCESS
      }
      Err(e) => {
        eprintln!(
          "{PROG}: {} {}: {e}",
          label("control_center.apps_could_not_write"),
          paths::defaults_file().display()
        );
        ExitCode::from(1)
      }
    };
  }

  if !detect::binary_available(&binary, &desktops) {
    eprintln!(
      "{PROG}: '{}' {} '{}'",
      binary,
      label("control_center.apps_is_not_installed_in_category"),
      cat.key()
    );
    eprintln!("{}:", label("control_center.apps_installed_options"));
    for app in detect::installed_apps(cat, &desktops) {
      eprintln!("  {} ({})", app.binary, app.display);
    }
    return ExitCode::from(1);
  }

  let mut state = AppState::load();
  state.set(cat, binary.clone());
  if let Err(e) = state.save() {
    eprintln!(
      "{PROG}: {} {}: {e}",
      label("control_center.apps_could_not_write"),
      paths::defaults_file().display()
    );
    return ExitCode::from(1);
  }

  match apply::apply(cat, &binary, &desktops) {
    Ok(report) => {
      println!("{}={}", cat.key(), binary);
      if let Some(id) = report.desktop_id {
        println!(
          "  {}: {id}",
          label("control_center.apps_registered_desktop")
        );
      }
      if let Some(path) = report.mimeapps_updated {
        println!(
          "  {}: {}",
          label("control_center.apps_updated_mimeapps"),
          path.display()
        );
      }
      if report.xdg_settings {
        println!(
          "  xdg-settings: {}",
          label("control_center.apps_default_web_browser_set")
        );
      }
      if report.hyprctl_reloaded {
        println!(
          "  ARGVUS: {}",
          label("control_center.apps_hyprctl_reload_requested")
        );
      }
      ExitCode::SUCCESS
    }
    Err(err) => {
      eprintln!("{PROG}: {err}");
      ExitCode::from(1)
    }
  }
}

/// Executes the `as_list` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn as_list(state: &AppState) -> Vec<(Category, Option<String>)> {
  Category::ORDER
    .iter()
    .copied()
    .map(|c| (c, state.get(c)))
    .collect()
}

/// Executes the `usage` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn usage(verbose: bool) {
  println!("{}", label("control_center.apps_usage"));

  if verbose {
    println!(
      "\n{}:\n  {CATEGORY_KEYS}\n\n{}: {}",
      label("control_center.apps_categories"),
      label("control_center.apps_state"),
      paths::defaults_file().display()
    );
  }
}
