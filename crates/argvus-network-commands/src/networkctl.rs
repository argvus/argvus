//! `argvus-networkctl`: the network status and its switches, through
//! NetworkManager's nmcli. The public route is `argvus networkctl`; the widget and
//! the control panel read `status`.

use argvus_network_core::nmcli;
use argvus_network_core::status::{self, Status};
use argvus_runtime::messages::text_in;
use argvus_runtime::process::{
  Finished, notify, output_any, program_exists, read_text, run_with_limit,
};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Duration;

/// How long one nmcli call may run before the status goes on without it.
const NMCLI_LIMIT: Duration = Duration::from_secs(5);
/// The usage error of a command line the command does not know.
const USAGE_STATUS: u8 = 64;
const DOMAIN: &str = "network";

/// Runs this command. `program` is the name shown in the usage message.
pub fn run_command(program: &str, arguments: Vec<String>) -> ExitCode {
  let rest = arguments.get(1).map(String::as_str);
  match arguments.first().map(String::as_str).unwrap_or("status") {
    "status" => print_status(),
    "toggle" => toggle_networking(),
    "enable" | "on" => set_networking(true),
    "disable" | "off" => set_networking(false),
    "manager" | "menu" => open_manager(),
    "wifi" => wifi(program, rest.unwrap_or("status")),
    _ => usage(program),
  }
}

fn message(key: &str, arguments: &[String]) -> String {
  text_in(DOMAIN, key, arguments)
}

fn notify_network(key: &str) {
  notify(&message("notification.title", &[]), &message(key, &[]));
}

fn usage(program: &str) -> ExitCode {
  let command = Path::new(program)
    .file_name()
    .map(|name| name.to_string_lossy().into_owned())
    .unwrap_or_default();
  eprintln!("{}", message("cli.usage", &[format!("command={command}")]));
  ExitCode::from(USAGE_STATUS)
}

fn nmcli_available() -> bool {
  program_exists("nmcli")
}

/// nmcli under the time limit, with its output if it ended in time.
fn nmcli(arguments: &[&str]) -> Option<Finished> {
  run_with_limit("nmcli", arguments, NMCLI_LIMIT)
}

/// The output of an nmcli query, or `unknown` when it fails.
fn nmcli_state(arguments: &[&str]) -> String {
  match nmcli(arguments) {
    Some(finished) if finished.status.success() => finished.stdout.trim_end().to_string(),
    _ => "unknown".to_string(),
  }
}

fn networking_state() -> String {
  nmcli_state(&["-t", "networking"])
}

fn wifi_state() -> String {
  nmcli_state(&["-t", "radio", "wifi"])
}

fn connectivity_state() -> String {
  nmcli_state(&["-t", "-g", "CONNECTIVITY", "general"])
}

/// The device that carries the connection. NetworkManager's device state is the
/// authority; a stale default route must not report a connection.
fn active_device() -> Option<String> {
  if nmcli_available() {
    let listing = nmcli(&["-t", "-f", "DEVICE,STATE", "device", "status"])
      .map(|finished| finished.stdout)
      .unwrap_or_default();
    return nmcli::first_connected_device(&listing);
  }
  nmcli::default_route_device(&output_any("ip", &["route", "show", "default"]))
}

fn device_type(device: &str) -> String {
  match nmcli(&["-g", "GENERAL.TYPE", "device", "show", device]) {
    Some(finished) => nmcli::device_type(&finished.stdout),
    None => String::new(),
  }
}

fn device_address(device: &str) -> String {
  if !program_exists("ip") {
    return String::new();
  }
  let addresses = output_any("ip", &["-4", "addr", "show", device]);
  nmcli::ipv4_address(&addresses).unwrap_or_default()
}

fn device_ssid(device: &str) -> String {
  if program_exists("iwgetid") {
    return output_any("iwgetid", &["-r", device])
      .trim_end()
      .to_string();
  }
  if !nmcli_available() {
    return String::new();
  }
  let listing = nmcli(&[
    "-t",
    "-f",
    "ACTIVE,SSID",
    "device",
    "wifi",
    "list",
    "ifname",
    device,
  ])
  .map(|finished| finished.stdout)
  .unwrap_or_default();
  nmcli::active_ssid(&listing).unwrap_or_default()
}

/// The byte counter of a device direction, read as the kernel writes it. A device
/// without the counter reports `0`.
fn byte_counter(device: &str, direction: &str) -> String {
  let path: PathBuf = ["/sys/class/net", device, "statistics"]
    .iter()
    .collect::<PathBuf>()
    .join(format!("{direction}_bytes"));
  read_text(&path)
    .map(|text| text.lines().next().unwrap_or_default().to_string())
    .unwrap_or_else(|| "0".to_string())
}

fn gather_status() -> Status {
  let networking = networking_state();
  let wifi = wifi_state();
  let connectivity = connectivity_state();
  let iface = active_device().unwrap_or_default();
  let kind = if iface.is_empty() || !nmcli_available() {
    String::new()
  } else {
    device_type(&iface)
  };
  let ip = if iface.is_empty() {
    String::new()
  } else {
    device_address(&iface)
  };
  let ssid = if iface.is_empty() {
    String::new()
  } else {
    device_ssid(&iface)
  };
  let (connected, word) = status::classify(&networking, &iface, &connectivity);
  Status {
    available: true,
    networking,
    wifi,
    rx: byte_counter(&iface, "rx"),
    tx: byte_counter(&iface, "tx"),
    iface,
    kind,
    ip,
    ssid,
    connected,
    connectivity,
    status: word,
  }
}

fn print_status() -> ExitCode {
  let status = if nmcli_available() {
    gather_status()
  } else {
    Status::unavailable()
  };
  print!("{}", status.render());
  ExitCode::SUCCESS
}

/// Switches networking on or off, then shows the status.
fn set_networking(enabled: bool) -> ExitCode {
  let target = if enabled { "on" } else { "off" };
  if !nmcli_available() {
    notify_network("message.available_no");
  } else if nmcli(&["networking", target]).is_some_and(|finished| finished.status.success()) {
    notify_network(if enabled {
      "message.reconnecting"
    } else {
      "message.disconnected"
    });
  } else {
    notify_network("message.available_no");
  }
  print_status()
}

/// Switches Wi-Fi on or off, then shows the status.
fn set_wifi(enabled: bool) -> ExitCode {
  let target = if enabled { "on" } else { "off" };
  if !nmcli_available() {
    notify_network("message.available_no");
  } else if nmcli(&["radio", "wifi", target]).is_some_and(|finished| finished.status.success()) {
    notify_network(if enabled {
      "message.wifi_enabled"
    } else {
      "message.wifi_disabled"
    });
  } else {
    notify_network("message.available_no");
  }
  print_status()
}

fn toggle_networking() -> ExitCode {
  match networking_state().as_str() {
    "enabled" => set_networking(false),
    "disabled" => set_networking(true),
    _ => print_status(),
  }
}

fn wifi(program: &str, action: &str) -> ExitCode {
  match action {
    "status" => {
      println!("{}", wifi_state());
      ExitCode::SUCCESS
    }
    "toggle" => match wifi_state().as_str() {
      "enabled" => set_wifi(false),
      "disabled" => set_wifi(true),
      _ => print_status(),
    },
    "enable" | "on" => set_wifi(true),
    "disable" | "off" => set_wifi(false),
    _ => usage(program),
  }
}

/// Opens the first connection manager installed, replacing this process.
fn open_manager() -> ExitCode {
  let managers: [(&str, &[&str]); 3] = [
    ("argvus", &["--network-interfaces"]),
    ("nm-connection-editor", &[]),
    ("nmtui", &[]),
  ];
  for (program, arguments) in managers {
    if program_exists(program) {
      let error = Command::new(program).args(arguments).exec();
      eprintln!("argvus-networkctl: cannot run {program}: {error}");
      return ExitCode::from(1);
    }
  }
  notify_network("message.manager_missing");
  ExitCode::SUCCESS
}
