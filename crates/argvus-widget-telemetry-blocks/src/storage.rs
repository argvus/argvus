//! The storage block: the use of the root and home mounts.

use argvus_runtime::messages;
use argvus_runtime::process::output_any;
use argvus_widget_telemetry_core::output;
use argvus_widget_telemetry_core::storage::{self, Mount};

/// The table of the block, one line per mount, with the header first.
pub fn table() -> String {
  let header = format!(
    "{:<7} {:<14} {:<7} {}",
    messages::text("label.mount", &[]),
    messages::text("label.used", &[]),
    messages::text("label.percent", &[]),
    ""
  );
  [header, mount_row("/"), mount_row("/home")].join("\n")
}

fn mount_row(mount: &str) -> String {
  let figures = mount_figures(mount);
  format!(
    "{:<7} {:<14} {:<7} {}",
    mount,
    format!("{}/{}G", figures.used_gib, figures.total_gib),
    format!("{}%", figures.percent),
    output::bar(figures.percent),
  )
}

/// The figures of a mount, read from `df`. A mount that cannot be read shows
/// zeros, as the shell did.
fn mount_figures(mount: &str) -> Mount {
  storage::parse_df(&output_any("df", &["-P", "-B1", mount])).unwrap_or(Mount::UNKNOWN)
}
