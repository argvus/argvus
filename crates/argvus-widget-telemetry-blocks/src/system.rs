//! The system block: the operating system, the session, the window manager, the
//! display, the CPU and the GPU.

use argvus_runtime::messages;
use argvus_runtime::process::{output_any, output_with_environment, program_exists, read_text};
use argvus_widget_telemetry_core::machine::{
  cpu_model, diagonal_inches, first_monitor, gpu_name, is_external, pretty_name, session_type,
  window_manager,
};
use std::env;

/// The rows of the block, in the order the widget shows them.
pub fn text() -> String {
  let unknown = messages::text("machine.unknown", &[]);
  let os = pretty_name(&read_text("/etc/os-release".as_ref()).unwrap_or_default());
  let desktop = if program_exists("argvus") {
    output_any("argvus", &["--version"]).trim_end().to_string()
  } else {
    unknown.clone()
  };
  let kernel = read_text("/proc/sys/kernel/osrelease".as_ref())
    .unwrap_or_default()
    .trim()
    .to_string();
  let locale = env::var("LANG").unwrap_or_default();
  let uptime = output_any("uptime", &["-p"])
    .trim_end()
    .trim_start_matches("up ")
    .to_string();
  let session = session_label(&unknown);
  let rows = [
    row("machine.os", &os),
    row("machine.desktop", &desktop),
    row("machine.kernel", &kernel),
    row("machine.locale", &locale),
    row("machine.uptime", &uptime),
    row("machine.window_manager", &window_name(&session, &unknown)),
    row("machine.display", &display(&unknown)),
    row("machine.cpu", &cpu()),
    row("machine.gpu", &gpu_label()),
  ];
  rows.join("\n")
}

/// The session type of the environment, or the unknown label.
fn session_label(unknown: &str) -> String {
  let session = session_type(
    env::var("XDG_SESSION_TYPE").ok().as_deref(),
    env::var_os("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty()),
    env::var_os("DISPLAY").is_some_and(|value| !value.is_empty()),
  );
  session
    .map(str::to_string)
    .unwrap_or_else(|| unknown.to_string())
}

/// The window manager with its version, for Hyprland.
fn window_name(session: &str, unknown: &str) -> String {
  let name = env::var("XDG_CURRENT_DESKTOP")
    .ok()
    .filter(|value| !value.is_empty())
    .or_else(|| {
      env::var("XDG_SESSION_DESKTOP")
        .ok()
        .filter(|value| !value.is_empty())
    })
    .unwrap_or_else(|| unknown.to_string());
  let version = (name == "Hyprland")
    .then(|| {
      output_any("hyprctl", &["version"])
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1).map(str::to_string))
    })
    .flatten();
  window_manager(&name, version.as_deref(), session)
}

/// The resolution, the diagonal, the refresh rate and the kind of the first
/// monitor of Hyprland.
fn display(unknown: &str) -> String {
  let on_wayland = env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland");
  if !program_exists("hyprctl") || !on_wayland {
    return unknown.to_string();
  }
  let Some(monitor) = first_monitor(&output_any("hyprctl", &["monitors", "-j"])) else {
    return unknown.to_string();
  };
  let resolution = format!("{}x{}", monitor.width, monitor.height);
  let refresh = monitor.refresh_hz.to_string();
  match diagonal_inches(monitor.physical_width_mm, monitor.physical_height_mm) {
    Some(inches) => {
      let kind = if is_external(inches) {
        messages::text("display.external", &[])
      } else {
        messages::text("display.laptop", &[])
      };
      messages::text(
        "display.resolution",
        &[
          format!("resolution={resolution}"),
          format!("inches={inches}"),
          format!("refresh={refresh}"),
          format!("type={kind}"),
        ],
      )
    }
    None => messages::text(
      "display.resolution_simple",
      &[
        format!("resolution={resolution}"),
        format!("refresh={refresh}"),
      ],
    ),
  }
}

/// The CPU model, read with the C locale so `lscpu` names the field in English.
fn cpu() -> String {
  cpu_model(&output_with_environment("lscpu", &[], &[("LC_ALL", "C")]))
}

/// The GPU name, from the first display controller.
fn gpu_label() -> String {
  gpu_name(&output_any("lspci", &[]))
}

fn row(key: &str, value: &str) -> String {
  let label = messages::text(key, &[]);
  let spacing = match key {
    "machine.kernel" | "machine.locale" | "machine.uptime" => "   ",
    "machine.display" => "  ",
    "machine.cpu" | "machine.gpu" => "      ",
    _ => "       ",
  };
  format!("<span>{label}:</span>{spacing}{value}")
}
