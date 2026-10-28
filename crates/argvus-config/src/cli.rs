//! The `argvus-config` command set. The `argvus` dispatcher and the `argvus-config`
//! binary both call [`run_command`].

use crate::json_pointers;
use crate::{
  ConfigDocument, ConfigScope, ConfigStore, appearance_runtime, css_edit, session_loading,
  session_paths, session_prepare, session_reload, session_splash, session_units,
  text_edit::{self, replace_file},
  theme_apply, theme_resolve, theme_transition, user_copy,
};
use crate::{ConfigError, ConfigResult};
use crate::{accent, accent_switch, display, effects, monitor_state, waybar};
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod monitor_commands;
mod path_commands;
mod session_commands;
mod store_commands;
mod taskbar_commands;
mod theme_commands;

use monitor_commands::{
  monitor_import, monitor_primary, monitor_remove, monitor_set, monitor_sync, monitor_workspaces,
};
use path_commands::{mutable_path, read_path};
use session_commands::{
  session_loading, session_prepare, session_reload_config, session_restart, session_sync_config,
};
use store_commands::run;
use taskbar_commands::{
  css_block, jsonc_margin, setting, taskbar_border, taskbar_height, taskbar_shape, waybar_font,
  waybar_layout, waybar_refresh,
};
use theme_commands::{
  gtk_files, gtk_runtime, gtk_theme_name, theme_apply_command, theme_command, theme_file,
  theme_runtime_command, transition_overlay, wallpaper_apply,
};

/// Runs one `argvus-config` command. `arguments` are the words after the program name.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  let Some(command) = arguments.first().map(String::as_str) else {
    return run_and_report(arguments);
  };
  let args = &arguments[1..];
  match command {
    "session-restart" => session_restart(args),
    "session-reload-config" => session_reload_config(args),
    "session-sync-config" => session_sync_config(),
    "effects" => ExitCode::from(effects::run(args) as u8),
    "gtk-theme-name" => gtk_theme_name(args),
    "waybar-refresh" => waybar_refresh(args),
    "waybar-layout" => waybar_layout(args),
    "waybar-font" => waybar_font(args),
    "accent" => accent_switch::run(args),
    "display" => ExitCode::from(display::run(args) as u8),
    "monitor-set" => monitor_set(args),
    "monitor-import" => monitor_import(args),
    "monitor-primary" => monitor_primary(args),
    "monitor-workspaces" => monitor_workspaces(args),
    "monitor-remove" => monitor_remove(args),
    "monitor-sync" => monitor_sync(args),
    "read-path" => read_path(args),
    "mutable-path" => mutable_path(args),
    "gtk-files" => gtk_files(args),
    "gtk-runtime" => gtk_runtime(args),
    "wallpaper-apply" => wallpaper_apply(args),
    "transition-overlay" => transition_overlay(args),
    "theme-apply" => theme_apply_command(args),
    "theme-runtime" => theme_runtime_command(args),
    "theme-file" => theme_file(args),
    // After the exact `theme-*` names above: the prefix routes theme queries.
    theme_query if theme_query.starts_with("theme-") => theme_command(theme_query, args),
    "jsonc-margin" => jsonc_margin(args),
    "taskbar-height" => taskbar_height(args),
    "taskbar-border" => taskbar_border(args),
    "taskbar-shape" => taskbar_shape(args),
    "setting" => setting(args),
    "css-block" => css_block(args),
    "session-prepare" => session_prepare(),
    "session-loading" => session_loading(),
    _ => run_and_report(arguments),
  }
}

/// Runs the store subcommands (`get`, `set`, `project`, ...) and reports an
/// error on stderr with exit status 1.
fn run_and_report(arguments: Vec<String>) -> ExitCode {
  match run(arguments) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      ExitCode::from(1)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::theme_commands::hold_overlay;
  use super::*;
  use session_splash::StartOutcome;
  use std::cell::RefCell;

  #[test]
  fn hold_waits_and_stops_only_an_overlay_it_started() {
    let steps = RefCell::new(Vec::new());
    let mut out = Vec::new();
    let outcome = hold_overlay(
      || StartOutcome::Started,
      || steps.borrow_mut().push("wait"),
      || steps.borrow_mut().push("stop"),
      &mut out,
    );
    assert_eq!(outcome, StartOutcome::Started);
    assert_eq!(out, b"started\n");
    assert_eq!(*steps.borrow(), ["wait", "stop"]);
  }

  #[test]
  fn hold_leaves_an_overlay_of_another_owner_alone() {
    let steps = RefCell::new(Vec::new());
    let mut out = Vec::new();
    let outcome = hold_overlay(
      || StartOutcome::AlreadyRunning,
      || steps.borrow_mut().push("wait"),
      || steps.borrow_mut().push("stop"),
      &mut out,
    );
    assert_eq!(outcome, StartOutcome::AlreadyRunning);
    assert!(
      out.is_empty(),
      "nothing is printed when the call did not start it"
    );
    assert!(steps.borrow().is_empty(), "no wait and no stop");
  }
}
