use super::*;
use argvus_control_center_core::process::{ProcessOutput, ProcessRequest};
use std::sync::Mutex;

#[derive(Default)]
struct HostShapeRunner(Mutex<Vec<ProcessRequest>>);
impl ProcessRunner for HostShapeRunner {
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    self.0.lock().unwrap().push(request.clone());
    let joined = request.args.join(" ");
    let stdout = if joined.contains("device status") {
      "br0:bridge:connected:br0\neno1:ethernet:connected:br0-ethernet\n"
    } else if joined.contains("device show") {
      "GENERAL.DEVICE:br0\nGENERAL.MTU:1500\nIP4.ADDRESS[1]:192.168.0.2/24\nIP4.GATEWAY:192.168.0.1\nIP4.DNS[1]:8.8.8.8\n"
    } else if joined == "radio wifi" {
      "enabled\n"
    } else if joined.contains("device wifi list") {
      "*:Home:82:WPA2:2412:yes\n"
    } else if joined.contains("connection show") {
      "work-vpn:vpn:\n"
    } else {
      ""
    };
    Ok(ProcessOutput {
      stdout: stdout.as_bytes().to_vec(),
      stderr: Vec::new(),
      status: Some(0),
      timed_out: false,
    })
  }
}
#[test]
fn nmcli_escaping() {
  assert_eq!(
    split_nmcli("wlan0:wifi:connected:Home\\:5G"),
    ["wlan0", "wifi", "connected", "Home:5G"]
  )
}
#[test]
fn wifi_deduplicates() {
  let x = parse_wifi("*:Home:80:WPA2:2412:yes\n:Home:40:WPA2:5180:no");
  assert_eq!(x.len(), 1);
  assert_eq!(x[0].signal, Some(80));
}
#[test]
fn real_nmcli_status_and_details_are_separate_and_optional() {
  let mut devices = parse_devices(
    "br0:bridge:connected:br0\neno1:ethernet:connected:br0-ethernet\nlo:loopback:connected (externally):lo",
  );
  assert_eq!(devices.len(), 3);
  apply_device_details(
    &mut devices[0],
    "GENERAL.DEVICE:br0\nGENERAL.HWADDR:D2:E5:28:7A:2A:6E\nGENERAL.MTU:1500\nIP4.ADDRESS[1]:192.168.0.2/24\nIP4.GATEWAY:192.168.0.1\nIP4.DNS[1]:8.8.8.8\nIP6.ADDRESS[1]:fe80\\::1/64\nIP6.GATEWAY:",
  );
  assert_eq!(devices[0].ipv4, ["192.168.0.2/24"]);
  assert_eq!(devices[0].ipv6, ["fe80::1/64"]);
  assert_eq!(devices[0].gateway.as_deref(), Some("192.168.0.1"));
  assert_eq!(devices[0].dns, ["8.8.8.8"]);
  assert!(devices[1].ipv4.is_empty());
}
#[test]
fn snapshot_uses_valid_status_fields_and_per_device_details() {
  let capabilities = Capabilities {
    has_nmcli: true,
    has_bluetoothctl: true,
    ..Default::default()
  };
  let backend = NetworkBackend::new(HostShapeRunner::default(), capabilities);
  let snapshot = backend.snapshot(false).unwrap();
  assert_eq!(snapshot.interfaces.len(), 2);
  assert_eq!(snapshot.interfaces[0].ipv4, ["192.168.0.2/24"]);
  assert_eq!(snapshot.wifi.len(), 1);
  assert_eq!(snapshot.vpn.len(), 1);
  assert!(!snapshot.bluetooth_present);
  let requests = backend.runner.0.lock().unwrap();
  let status = requests
    .iter()
    .find(|request| request.args.ends_with(&["device".into(), "status".into()]))
    .unwrap();
  assert_eq!(status.args[2], DEVICE_STATUS_FIELDS);
  assert!(!status.args[2].contains("IP4."));
  assert!(requests.iter().any(|request| {
    request
      .args
      .iter()
      .any(|argument| argument == DEVICE_DETAIL_FIELDS)
      && request.args.iter().any(|argument| argument == "br0")
  }));
  assert!(requests.iter().any(|request| {
    request
      .args
      .iter()
      .any(|argument| argument == DEVICE_DETAIL_FIELDS)
      && request.args.iter().any(|argument| argument == "eno1")
  }));
}

#[derive(Default)]
struct FailingDetailRunner(Mutex<Vec<ProcessRequest>>);

#[test]
fn inventory_survives_detail_failure() {
  let backend = NetworkBackend::new(
    FailingDetailRunner::default(),
    Capabilities {
      has_nmcli: true,
      ..Default::default()
    },
  );
  let updates = Mutex::new(Vec::new());
  backend
    .load_page(
      NetworkPage::Interfaces,
      false,
      NetworkSnapshot::default(),
      |state| updates.lock().unwrap().push(state),
    )
    .unwrap();
  assert_eq!(updates.lock().unwrap()[0].interfaces.len(), 2);
}

/// Requests whose arguments contain every one of `parts`.
fn requests_with(runner: &Mutex<Vec<ProcessRequest>>, parts: &[&str]) -> usize {
  runner
    .lock()
    .unwrap()
    .iter()
    .filter(|request| {
      parts
        .iter()
        .all(|part| request.args.iter().any(|argument| argument == part))
    })
    .count()
}

#[test]
fn home_reads_its_summary_without_scanning_or_probing_bluetooth() {
  let backend = NetworkBackend::new(
    HostShapeRunner::default(),
    Capabilities {
      has_nmcli: true,
      has_bluetoothctl: true,
      ..Default::default()
    },
  );
  let updates = Mutex::new(Vec::new());
  let snapshot = backend
    .load_page(
      NetworkPage::Home,
      false,
      NetworkSnapshot::default(),
      |state| updates.lock().unwrap().push(state),
    )
    .unwrap();
  // The inventory is published before any slower query has answered.
  let first = updates.lock().unwrap()[0].clone();
  assert_eq!(first.interfaces.len(), 2);
  assert!(first.interfaces[0].ipv4.is_empty());
  // What the Home shows: the connected interfaces' addresses and DNS.
  assert_eq!(snapshot.interfaces[0].ipv4, ["192.168.0.2/24"]);
  assert_eq!(snapshot.dns.servers, ["8.8.8.8"]);
  assert_eq!(snapshot.wifi_enabled, Some(true));
  assert_eq!(snapshot.vpn.len(), 1);
  let requests = backend.runner.0.lock().unwrap();
  assert!(
    requests
      .iter()
      .all(|request| request.program != "bluetoothctl")
  );
  // No Wi-Fi card in this inventory: no Wi-Fi list at all.
  assert!(
    requests
      .iter()
      .all(|request| !request.args.iter().any(|argument| argument == "--rescan"))
  );
}

/// A laptop: one Wi-Fi card, connected, and an unplugged Ethernet port.
#[derive(Default)]
struct LaptopRunner(Mutex<Vec<ProcessRequest>>);
impl ProcessRunner for LaptopRunner {
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    self.0.lock().unwrap().push(request.clone());
    let joined = request.args.join(" ");
    let stdout = if joined.contains("device status") {
      "wlan0:wifi:connected:Home\neno1:ethernet:unavailable:\n"
    } else if joined.contains("device show wlan0") {
      "GENERAL.DEVICE:wlan0\nIP4.ADDRESS[1]:10.0.0.5/24\nIP4.DNS[1]:10.0.0.1\n"
    } else if joined == "radio wifi" {
      "enabled\n"
    } else if joined.contains("device wifi list") {
      "*:Home:82:WPA2:2412:yes\n"
    } else {
      ""
    };
    Ok(ProcessOutput {
      stdout: stdout.as_bytes().to_vec(),
      stderr: Vec::new(),
      status: Some(0),
      timed_out: false,
    })
  }
}

#[test]
fn summary_pages_read_the_cached_wifi_list_and_only_connected_details() {
  for page in [NetworkPage::Home, NetworkPage::Status] {
    let backend = NetworkBackend::new(
      LaptopRunner::default(),
      Capabilities {
        has_nmcli: true,
        ..Default::default()
      },
    );
    let snapshot = backend
      .load_page(page, true, NetworkSnapshot::default(), |_| {})
      .unwrap();
    assert!(snapshot.wifi.iter().any(|network| network.connected));
    assert_eq!(snapshot.interfaces[0].ipv4, ["10.0.0.5/24"]);
    // Never a rescan outside the Wi-Fi page, even when asked for one.
    assert_eq!(requests_with(&backend.runner.0, &["--rescan", "no"]), 1);
    assert_eq!(requests_with(&backend.runner.0, &["--rescan", "yes"]), 0);
    assert_eq!(
      requests_with(&backend.runner.0, &["device", "show", "wlan0"]),
      1
    );
    assert_eq!(
      requests_with(&backend.runner.0, &["device", "show", "eno1"]),
      0
    );
  }
}

#[test]
fn wifi_page_rescans_only_when_asked() {
  for (scan, rescan) in [(false, "no"), (true, "yes")] {
    let backend = NetworkBackend::new(
      LaptopRunner::default(),
      Capabilities {
        has_nmcli: true,
        ..Default::default()
      },
    );
    backend
      .load_page(NetworkPage::Wifi, scan, NetworkSnapshot::default(), |_| {})
      .unwrap();
    assert_eq!(requests_with(&backend.runner.0, &["--rescan", rescan]), 1);
    assert_eq!(requests_with(&backend.runner.0, &[DEVICE_DETAIL_FIELDS]), 0);
  }
}

#[test]
fn proxy_requires_no_subprocess_or_network_manager() {
  let backend = NetworkBackend::new(HostShapeRunner::default(), Capabilities::default());
  backend
    .load_page(
      NetworkPage::Proxy,
      false,
      NetworkSnapshot::default(),
      |_| {},
    )
    .unwrap();
  assert!(backend.runner.0.lock().unwrap().is_empty());
}
impl ProcessRunner for FailingDetailRunner {
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    self.0.lock().unwrap().push(request.clone());
    let joined = request.args.join(" ");
    let stdout = if joined.contains("device status") {
      "br0:bridge:connected:br0\neno1:ethernet:connected:br0-ethernet\n"
    } else if joined.contains("device show br0") {
      "GENERAL.DEVICE:br0\nIP4.DNS[1]:10.0.0.1\n"
    } else if joined == "radio wifi" {
      "enabled\n"
    } else if joined.contains("device wifi list") {
      "*:Home:82:WPA2:2412:yes\n"
    } else if joined.contains("connection show") {
      "work-vpn:vpn:\n"
    } else {
      ""
    };
    Ok(ProcessOutput {
      stdout: stdout.as_bytes().to_vec(),
      stderr: Vec::new(),
      status: if joined.contains("device show eno1") {
        Some(1)
      } else {
        Some(0)
      },
      timed_out: false,
    })
  }
}

#[test]
fn snapshot_ignores_failed_per_device_queries_and_stays_complete() {
  let capabilities = Capabilities {
    has_nmcli: true,
    ..Default::default()
  };
  let backend = NetworkBackend::new(FailingDetailRunner::default(), capabilities);
  let snapshot = backend.snapshot(false).unwrap();
  assert_eq!(snapshot.interfaces.len(), 2);
  assert!(snapshot.interfaces[0].ipv4.is_empty());
  assert_eq!(snapshot.interfaces[0].dns, ["10.0.0.1"]);
  assert!(snapshot.interfaces[1].ipv4.is_empty());
  assert_eq!(snapshot.wifi.len(), 1);
  assert_eq!(snapshot.vpn.len(), 1);
  assert_eq!(snapshot.dns.servers, ["10.0.0.1"]);
  assert_eq!(snapshot.dns.source, "NetworkManager");
}
#[test]
fn addresses_validate() {
  assert!(valid_ipv4("192.0.2.1"));
  assert!(!valid_ipv4("no"));
  assert!(valid_ipv6("2001:db8::1"));
}
#[test]
fn proxy_redacts_credentials() {
  assert_eq!(
    redact_proxy("http://user:pass@example.test:8080"),
    "http://***@example.test:8080"
  );
}
