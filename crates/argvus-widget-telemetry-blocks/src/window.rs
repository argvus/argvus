//! The scroll window of the widget, as the toggle reports it. A block is shown
//! only while it is inside the window.

use argvus_runtime::process::output_any;
use argvus_widget_telemetry_core::report::{self, Report};
use std::env;

/// The `argvus` dispatcher, found on PATH, runs the toggle that owns the window.
const TOGGLE_DEFAULT_ARGUMENTS: [&str; 3] = ["widget-telemetry", "toggle", "layout"];

/// The window the toggle reports. A toggle that cannot run leaves it empty, so
/// every block hides. `ARGVUS_WIDGET_TELEMETRY_TOGGLE` replaces the program; it then
/// receives `layout` as its only argument.
pub fn current() -> Report {
  let output = match env::var("ARGVUS_WIDGET_TELEMETRY_TOGGLE") {
    Ok(program) if !program.is_empty() => output_any(&program, &["layout"]),
    _ => output_any("argvus", &TOGGLE_DEFAULT_ARGUMENTS),
  };
  report::parse(&output)
}

/// Whether a block is inside the window.
pub fn shows(block: &str) -> bool {
  current().visible.iter().any(|visible| visible == block)
}
