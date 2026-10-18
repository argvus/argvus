//! The Bluetooth state from bluetoothctl: the adapter, its power, the connected
//! devices, and the Waybar module built from them.

/// The state of the adapter as the command reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
  pub available: bool,
  /// `yes` or `no`.
  pub powered: &'static str,
  pub connected: usize,
  pub adapter: String,
  /// The connected devices, separated by commas.
  pub devices: String,
  pub status: &'static str,
}

impl Summary {
  /// The status of a system without bluetoothctl, or without a controller.
  pub fn unavailable(status: &'static str) -> Self {
    Self {
      available: false,
      powered: "no",
      connected: 0,
      adapter: String::new(),
      devices: String::new(),
      status,
    }
  }

  /// The `key=value` lines, without the manager line.
  pub fn render(&self) -> String {
    format!(
      "available={}\npowered={}\nconnected={}\nadapter={}\ndevices={}\nstatus={}\n",
      if self.available { "yes" } else { "no" },
      self.powered,
      self.connected,
      self.adapter,
      self.devices,
      self.status,
    )
  }
}

/// Whether `show` describes a controller: a block that names one and does not
/// say that there is none.
pub fn adapter_available(show: &str) -> bool {
  !show.is_empty()
    && !show.to_ascii_lowercase().contains("no default controller")
    && show.contains("Controller ")
}

/// The value of a field of `bluetoothctl show`, read as the text after the first
/// `: ` on the first line whose name contains `key`.
pub fn adapter_value(show: &str, key: &str) -> String {
  show
    .lines()
    .find_map(|line| {
      let (name, rest) = line.split_once(": ").unwrap_or((line, ""));
      name.contains(key).then(|| {
        rest
          .split(": ")
          .next()
          .unwrap_or_default()
          .trim_start()
          .to_string()
      })
    })
    .unwrap_or_default()
}

/// The names of the connected devices, from `bluetoothctl devices Connected`:
/// each line is `Device ADDRESS NAME`, and only the name is kept.
pub fn connected_devices(listing: &str) -> String {
  listing
    .lines()
    .filter_map(|line| {
      let name = line
        .split_whitespace()
        .skip(2)
        .collect::<Vec<_>>()
        .join(" ");
      (!name.is_empty()).then_some(name)
    })
    .collect::<Vec<_>>()
    .join(", ")
}

/// The number of connected devices: the lines of the listing that are not blank.
pub fn connected_count(listing: &str) -> usize {
  listing
    .lines()
    .filter(|line| !line.trim().is_empty())
    .count()
}

/// The state of the adapter, from the output of `show` and of the connected
/// devices listing.
pub fn summarize(show: &str, listing: &str) -> Summary {
  if !adapter_available(show) {
    return Summary::unavailable("no-controller");
  }
  let connected = connected_count(listing);
  let (powered, status) = match adapter_value(show, "Powered").as_str() {
    "yes" | "Yes" | "true" | "True" => {
      if connected > 0 {
        ("yes", "connected")
      } else {
        ("yes", "on")
      }
    }
    _ => ("no", "off"),
  };
  Summary {
    available: true,
    powered,
    connected,
    adapter: adapter_value(show, "Alias"),
    devices: connected_devices(listing),
    status,
  }
}

/// The power of the adapter in lower case, or `unknown` without a controller.
pub fn power_state(show: &str) -> String {
  if !adapter_available(show) {
    return "unknown".into();
  }
  adapter_value(show, "Powered").to_ascii_lowercase()
}

/// The labels the Waybar module shows when the adapter is off or has no name.
pub struct Labels {
  pub off: String,
  pub name: String,
}

/// The Waybar JSON of the Bluetooth module, or `None` when there is no adapter
/// to show.
pub fn waybar(summary: &Summary, labels: &Labels) -> Option<String> {
  if !summary.available {
    return None;
  }
  let mut class = "bluetooth-off";
  let mut text = "\u{f00af}".to_string();
  let mut tooltip = labels.off.clone();
  if summary.powered == "yes" {
    class = "bluetooth-on";
    tooltip = non_empty_or(&summary.adapter, &labels.name);
    if summary.connected > 0 {
      class = "bluetooth-connected";
      text = format!("\u{f00af} {}", summary.connected);
      tooltip = non_empty_or(&summary.devices, &summary.adapter);
    }
  }
  Some(format!(
    "{{\"text\":\"{}\",\"tooltip\":\"{}\",\"class\":\"{}\"}}",
    json_escape(&text),
    json_escape(&tooltip),
    json_escape(class),
  ))
}

fn non_empty_or(value: &str, fallback: &str) -> String {
  if value.is_empty() {
    fallback.to_string()
  } else {
    value.to_string()
  }
}

/// Escapes what a JSON string cannot carry bare: the backslash and the quote.
pub fn json_escape(text: &str) -> String {
  text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The value of a `key=value` line of a summary, as the widget reads it.
pub fn status_field(output: &str, key: &str) -> Option<String> {
  output.lines().find_map(|line| {
    let (name, value) = line.split_once('=').unwrap_or((line, ""));
    (name == key).then(|| value.to_string())
  })
}

#[cfg(test)]
mod tests {
  use super::{Labels, adapter_available, adapter_value, connected_devices, summarize, waybar};

  const SHOW: &str = "Controller 00:11:22:33:44:55 (public)\n\tPowered: yes\n\tAlias: laptop\n";

  #[test]
  fn controller_is_recognised() {
    assert!(adapter_available(SHOW));
    assert!(!adapter_available("No default controller available\n"));
    assert!(!adapter_available(""));
  }

  #[test]
  fn fields_are_read_after_the_separator() {
    assert_eq!(adapter_value(SHOW, "Powered"), "yes");
    assert_eq!(adapter_value(SHOW, "Alias"), "laptop");
    assert_eq!(adapter_value(SHOW, "Missing"), "");
  }

  #[test]
  fn devices_keep_their_names() {
    let listing = "Device AA:BB:CC:DD:EE:FF Headset Pro\nDevice 11:22:33:44:55:66 Mouse\n";
    assert_eq!(connected_devices(listing), "Headset Pro, Mouse");
  }

  #[test]
  fn summary_reports_power_and_connections() {
    let summary = summarize(SHOW, "Device AA:BB:CC:DD:EE:FF Mouse\n");
    assert_eq!(summary.powered, "yes");
    assert_eq!(summary.connected, 1);
    assert_eq!(summary.status, "connected");
    assert_eq!(summarize("", "").status, "no-controller");
  }

  #[test]
  fn waybar_shows_the_connection_count() {
    let labels = Labels {
      off: "Bluetooth off".into(),
      name: "Bluetooth".into(),
    };
    let summary = summarize(SHOW, "Device AA:BB:CC:DD:EE:FF Mouse\n");
    let json = waybar(&summary, &labels).unwrap();
    assert_eq!(
      json,
      "{\"text\":\"\u{f00af} 1\",\"tooltip\":\"Mouse\",\"class\":\"bluetooth-connected\"}"
    );
    assert_eq!(waybar(&summarize("", ""), &labels), None);
  }
}
