//! The network status: the fields `argvus-networkctl status` prints, and the
//! status word that a networking state, a device and a connectivity mean.

/// The network status as the command reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
  pub available: bool,
  pub networking: String,
  pub wifi: String,
  pub iface: String,
  pub kind: String,
  pub ip: String,
  pub ssid: String,
  pub rx: String,
  pub tx: String,
  pub connected: bool,
  pub connectivity: String,
  pub status: &'static str,
}

impl Status {
  /// The status of a system without NetworkManager.
  pub fn unavailable() -> Self {
    Self {
      available: false,
      networking: "unknown".into(),
      wifi: "unknown".into(),
      iface: String::new(),
      kind: String::new(),
      ip: String::new(),
      ssid: String::new(),
      rx: "0".into(),
      tx: "0".into(),
      connected: false,
      connectivity: "unknown".into(),
      status: "unavailable",
    }
  }

  /// The `key=value` lines, in the order the widget reads them.
  pub fn render(&self) -> String {
    let lines = [
      ("available", yes_no(self.available).to_string()),
      ("networking", self.networking.clone()),
      ("wifi", self.wifi.clone()),
      ("iface", self.iface.clone()),
      ("type", self.kind.clone()),
      ("ip", self.ip.clone()),
      ("ssid", self.ssid.clone()),
      ("rx", self.rx.clone()),
      ("tx", self.tx.clone()),
      ("connected", yes_no(self.connected).to_string()),
      ("connectivity", self.connectivity.clone()),
      ("status", self.status.to_string()),
    ];
    lines
      .iter()
      .map(|(key, value)| format!("{key}={value}\n"))
      .collect()
  }
}

fn yes_no(value: bool) -> &'static str {
  if value { "yes" } else { "no" }
}

/// Whether the device counts as connected, and the status word. Networking that
/// is switched off wins; a device with a full connectivity is connected, and any
/// other device is limited.
pub fn classify(networking: &str, iface: &str, connectivity: &str) -> (bool, &'static str) {
  if networking == "disabled" {
    return (false, "disabled");
  }
  if iface.is_empty() {
    return (false, "disconnected");
  }
  match connectivity {
    "full" => (true, "connected"),
    _ => (true, "limited"),
  }
}

#[cfg(test)]
mod tests {
  use super::{Status, classify};

  #[test]
  fn status_words_follow_the_state() {
    assert_eq!(classify("disabled", "br0", "full"), (false, "disabled"));
    assert_eq!(classify("enabled", "", "full"), (false, "disconnected"));
    assert_eq!(classify("enabled", "br0", "full"), (true, "connected"));
    assert_eq!(classify("enabled", "br0", "portal"), (true, "limited"));
  }

  #[test]
  fn unavailable_status_lists_every_field() {
    let rendered = Status::unavailable().render();
    assert!(rendered.starts_with("available=no\nnetworking=unknown\n"));
    assert!(rendered.ends_with("connectivity=unknown\nstatus=unavailable\n"));
  }
}
