//! The network block: the interface, its address and the traffic rate, read
//! from `argvus networkctl status` over one second.

use argvus_runtime::messages;
use argvus_runtime::process::output_any;
use argvus_widget_telemetry_core::network::{field, human_rate};
use std::thread;
use std::time::Duration;

/// The text of the block: a header and the figures, or the disconnected state.
pub fn text() -> String {
  let first = status();
  let connected = field(&first, "connected").as_deref() == Some("yes");
  let interface = field(&first, "iface").filter(|interface| !interface.is_empty());
  let (Some(interface), true) = (interface, connected) else {
    return messages::text("network.disconnected", &[]);
  };
  let address = field(&first, "ip").filter(|address| !address.is_empty());

  // Two readings one second apart give the rate of each direction.
  let received_before = counter(&first, "rx");
  let sent_before = counter(&first, "tx");
  thread::sleep(Duration::from_secs(1));
  let second = status();
  let received_rate =
    counter(&second, "rx").unwrap_or(received_before.unwrap_or(0)) - received_before.unwrap_or(0);
  let sent_rate =
    counter(&second, "tx").unwrap_or(sent_before.unwrap_or(0)) - sent_before.unwrap_or(0);

  let header = format!(
    "{}  {}              {}",
    messages::text("label.interface", &[]),
    messages::text("label.ip", &[]),
    messages::text("label.traffic", &[]),
  );
  let figures = format!(
    "{}  {}  ↓ {} ↑ {}",
    interface,
    address.unwrap_or_else(|| messages::text("network.none", &[])),
    human_rate(received_rate),
    human_rate(sent_rate),
  );
  format!("{header}\n{figures}")
}

/// The `argvus networkctl status` lines, whatever the status of the command.
fn status() -> String {
  output_any("argvus", &["networkctl", "status"])
}

/// A byte counter of the status, or `None` when the status does not give it.
fn counter(status: &str, key: &str) -> Option<i64> {
  field(status, key)?.parse().ok()
}
