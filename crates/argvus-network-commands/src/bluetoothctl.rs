//! `argvus-bluetoothctl`: the Bluetooth adapter, its power and its connected
//! devices, through bluetoothctl and BlueZ. The public route is
//! `argvus bluetoothctl`; the taskbar and the control panel read it.

use argvus_network_core::bluetooth::{self, Labels, Summary};
use argvus_runtime::messages::text_in;
use argvus_runtime::process::{Finished, notify, program_exists, run_quiet, run_with_limit};
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::Duration;

/// How long one bluetoothctl call may run.
const BTCTL_LIMIT: Duration = Duration::from_secs(3);
/// Where the kernel lists the Bluetooth controllers.
const SYSFS_BLUETOOTH: &str = "/sys/class/bluetooth";
/// The usage error of a command line the command does not know.
const USAGE_STATUS: u8 = 64;
const DOMAIN: &str = "network";

/// Runs this command. `program` is the name shown in the usage message.
pub fn run_command(program: &str, arguments: Vec<String>) -> ExitCode {
  match arguments.first().map(String::as_str).unwrap_or("status") {
    "status" => print_status(),
    "available" => print_available(),
    "waybar" => print_waybar(),
    "enable" | "on" => set_power(true),
    "disable" | "off" => set_power(false),
    "toggle" => toggle(),
    "manager" => open_manager(),
    _ => usage(program),
  }
}

fn message(key: &str, arguments: &[String]) -> String {
  text_in(DOMAIN, key, arguments)
}

fn notify_bluetooth(key: &str, arguments: &[String]) {
  notify(&message("bluetooth.title", &[]), &message(key, arguments));
}

fn usage(program: &str) -> ExitCode {
  let command = Path::new(program)
    .file_name()
    .map(|name| name.to_string_lossy().into_owned())
    .unwrap_or_default();
  eprintln!(
    "{}",
    message("bluetooth_usage", &[format!("command={command}")])
  );
  ExitCode::from(USAGE_STATUS)
}

fn bluetoothctl_available() -> bool {
  program_exists("bluetoothctl")
}

/// Whether the kernel lists a controller. Without one, `bluetoothctl show`
/// waits for BlueZ until [`BTCTL_LIMIT`] when the service is stopped, so the
/// status would take seconds to say what sysfs already says. A listing that
/// cannot be read for another reason does not decide: bluetoothctl is asked.
fn controller_listed() -> bool {
  match std::fs::read_dir(SYSFS_BLUETOOTH) {
    Ok(entries) => bluetooth::lists_controller(
      entries
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned()),
    ),
    Err(error) => error.kind() != io::ErrorKind::NotFound,
  }
}

/// bluetoothctl under the time limit, with its output if it ended in time.
fn btctl(arguments: &[&str]) -> Option<Finished> {
  run_with_limit("bluetoothctl", arguments, BTCTL_LIMIT)
}

/// The output of a bluetoothctl query, empty when it does not answer.
fn btctl_output(arguments: &[&str]) -> String {
  btctl(arguments)
    .map(|finished| finished.stdout)
    .unwrap_or_default()
}

fn manager_line() -> String {
  if program_exists("blueman-manager") {
    "manager=yes".to_string()
  } else {
    "manager=no".to_string()
  }
}

/// The adapter state: the `show` block, and the connected devices when there is
/// an adapter to ask about.
fn summary() -> Summary {
  if !bluetoothctl_available() {
    return Summary::unavailable("unavailable");
  }
  if !controller_listed() {
    return Summary::unavailable("no-controller");
  }
  let show = btctl_output(&["show"]);
  if !bluetooth::adapter_available(&show) {
    return Summary::unavailable("no-controller");
  }
  let listing = btctl_output(&["devices", "Connected"]);
  bluetooth::summarize(&show, &listing)
}

fn print_status() -> ExitCode {
  print!("{}", summary().render());
  println!("{}", manager_line());
  ExitCode::SUCCESS
}

fn print_available() -> ExitCode {
  if summary().available {
    ExitCode::SUCCESS
  } else {
    ExitCode::from(1)
  }
}

/// The Waybar module, or a failure when there is no adapter to show.
fn print_waybar() -> ExitCode {
  let state = summary();
  let labels = Labels {
    off: message("bluetooth.off", &[]),
    name: message("bluetooth.name", &[]),
  };
  match bluetooth::waybar(&state, &labels) {
    Some(json) => {
      println!("{json}");
      ExitCode::SUCCESS
    }
    None => ExitCode::from(1),
  }
}

/// Starts the BlueZ service when it is not running.
fn start_service() {
  if !program_exists("systemctl")
    || run_quiet("systemctl", &["is-active", "--quiet", "bluetooth.service"])
  {
    return;
  }
  let _ = run_with_limit("systemctl", &["start", "bluetooth.service"], BTCTL_LIMIT);
}

/// Stops the optional Blueman applet, which follows the adapter.
fn stop_blueman() {
  if program_exists("systemctl") {
    let _ = run_quiet(
      "systemctl",
      &["--user", "stop", "argvus-blueman-applet.service"],
    );
  }
}

fn set_power(enabled: bool) -> ExitCode {
  let target = if enabled { "on" } else { "off" };
  if !bluetoothctl_available() {
    notify_bluetooth("bluetoothctl_missing", &[]);
    return print_status();
  }
  if enabled {
    start_service();
  }
  if btctl(&["power", target]).is_some_and(|finished| finished.status.success()) {
    if !enabled {
      stop_blueman();
    }
  } else {
    notify_bluetooth("bluetooth.power_failed", &[format!("target={target}")]);
  }
  print_status()
}

fn toggle() -> ExitCode {
  let show = btctl_output(&["show"]);
  match bluetooth::power_state(&show).as_str() {
    "yes" | "true" => set_power(false),
    "no" | "false" => set_power(true),
    _ => print_status(),
  }
}

/// Opens Blueman's manager, replacing this process.
fn open_manager() -> ExitCode {
  if program_exists("blueman-manager") {
    let error = Command::new("blueman-manager").exec();
    eprintln!("argvus-bluetoothctl: cannot run blueman-manager: {error}");
    return ExitCode::from(1);
  }
  notify_bluetooth("blueman_missing", &[]);
  ExitCode::SUCCESS
}
