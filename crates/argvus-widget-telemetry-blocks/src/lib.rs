//! `argvus-widget-telemetry-block`: prints the text of one part of the Waybar
//! telemetry widget. The widget's modules run it, and the taskbar runs its
//! probes. Nothing is printed for a block that is outside the scroll window,
//! except the JSON blocks, which print an empty text so Waybar clears them.
//!
//! Usage:
//!   argvus-widget-telemetry-block header BLOCK        the title of a block
//!   argvus-widget-telemetry-block keys                the shortcut line
//!   argvus-widget-telemetry-block BLOCK               a block's JSON text
//!                                                      (system, cpu_gpu, memory,
//!                                                      storage, processes, network,
//!                                                      dev_dashboard)
//!   argvus-widget-telemetry-block cpu-temp [--available]
//!   argvus-widget-telemetry-block gpu --temp | --available-temp

mod cpu_gpu;
mod dev_dashboard;
mod gpu;
mod hardware;
mod memory;
mod network;
mod probes;
mod processes;
mod storage;
mod system;
mod window;

use argvus_runtime::messages;
use argvus_widget_telemetry_core::blocks::is_block;
use argvus_widget_telemetry_core::output;
use std::process::ExitCode;

/// The name the usage message shows for this command.
const PROGRAM: &str = "argvus-widget-telemetry-block";

/// Runs one command of this crate. `arguments` are the words after the command
/// name. The `argvus` dispatcher and the binary both call this function.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  let rest = &arguments[arguments.len().min(1)..];
  match arguments.first().map(String::as_str) {
    Some("header") => header(rest.first().map(String::as_str).unwrap_or("")),
    Some("keys") => keys(),
    Some("cpu-temp") => probes::cpu_temp(rest).unwrap_or_else(usage),
    Some("gpu") => probes::gpu_temp(rest).unwrap_or_else(usage),
    Some("system") => json_block("system", system::text),
    Some("cpu_gpu") => json_block("cpu_gpu", cpu_gpu::table),
    Some("memory") => json_block("memory", memory::table),
    Some("storage") => json_block("storage", storage::table),
    Some("processes") => json_block("processes", processes::table),
    Some("network") => json_block("network", network::text),
    Some("dev_dashboard") => json_block("dev_dashboard", dev_dashboard::text),
    _ => usage(),
  }
}

fn usage() -> ExitCode {
  let text = messages::text(
    "cli.usage",
    &[
      format!("command={PROGRAM}"),
      "options={header BLOCK|keys|BLOCK|cpu-temp|gpu}".to_string(),
    ],
  );
  eprintln!("{text}");
  ExitCode::from(2)
}

/// The title of a block. A header is hidden together with its block, and the
/// arrows mark content scrolled out of view above or below the window.
fn header(block: &str) -> ExitCode {
  if !is_block(block) || !window::shows(block) {
    return ExitCode::SUCCESS;
  }
  let scroll = window::current();
  let mut label = messages::text(&format!("header.{block}"), &[]);
  if scroll.first.as_deref() == Some(block) && scroll.above {
    label.push_str(" ▲");
  }
  if scroll.last.as_deref() == Some(block) && scroll.below {
    label.push_str(" ▼");
  }
  println!("{label}");
  ExitCode::SUCCESS
}

/// The shortcut line, shown only while the keys block is inside the window.
fn keys() -> ExitCode {
  if window::shows("keys") {
    println!("{}", messages::text("keys.format", &[]));
  }
  ExitCode::SUCCESS
}

/// The JSON text of a block for Waybar: the rendered text inside the window, and
/// an empty text outside it.
fn json_block(block: &str, render: fn() -> String) -> ExitCode {
  let text = if window::shows(block) {
    render()
  } else {
    String::new()
  };
  println!("{}", output::json_text(&text));
  ExitCode::SUCCESS
}
