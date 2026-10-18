//! Implements isolated integration with system tools and APIs in crate `argvus control center network`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessError, ProcessRequest, ProcessRunner},
  sanitize::terminal_text,
};
use std::{
  net::{Ipv4Addr, Ipv6Addr},
  path::Path,
  time::Duration,
};
use thiserror::Error;

#[derive(Debug, Error)]
/// Defines `NetworkError`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum NetworkError {
  #[error("network backend unavailable")]
  BackendUnavailable,
  #[error("invalid network input")]
  InvalidInput,
  #[error("process failed: {0}")]
  Process(#[from] ProcessError),
  #[error("operation timed out")]
  Timeout,
  #[error("network command failed: {0}")]
  Command(String),
}

/// Represents `NetworkBackend`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct NetworkBackend<R> {
  pub runner: R,
  pub capabilities: Capabilities,
}
impl<R: ProcessRunner> NetworkBackend<R> {
  /// Publish inventory before slower page-specific queries complete.
  pub fn load_page(
    &self,
    page: NetworkPage,
    scan: bool,
    mut state: NetworkSnapshot,
    publish: impl Fn(NetworkSnapshot),
  ) -> Result<NetworkSnapshot, NetworkError> {
    state.proxy = proxy_from_environment();
    if page == NetworkPage::Proxy {
      return Ok(state);
    }
    if !self.capabilities.has_nmcli {
      return Err(NetworkError::BackendUnavailable);
    }
    let devices = self.run(
      "nmcli",
      &["-t", "-f", DEVICE_STATUS_FIELDS, "device", "status"],
      Duration::from_secs(3),
    )?;
    let mut interfaces = parse_devices(&devices);
    for interface in &mut interfaces {
      if let Some(old) = state.interfaces.iter().find(|old| {
        old.name == interface.name
          && old.connection == interface.connection
          && old.state == interface.state
      }) {
        let mut preserved = old.clone();
        preserved.kind = interface.kind.clone();
        *interface = preserved;
      }
    }
    state.interfaces = interfaces;
    state.available = true;
    state.connectivity = if state.interfaces.iter().any(|i| i.state == "connected") {
      "online"
    } else {
      "offline"
    }
    .into();
    publish(state.clone());
    if matches!(
      page,
      NetworkPage::Home | NetworkPage::Status | NetworkPage::Wifi
    ) {
      state.wifi_enabled = self.radio_wifi();
      publish(state.clone());
    }
    if matches!(page, NetworkPage::Home | NetworkPage::Vpn) {
      let value = self.run(
        "nmcli",
        &["-t", "-f", "NAME,TYPE,DEVICE", "connection", "show"],
        Duration::from_secs(3),
      )?;
      state.vpn = parse_vpn(&value);
    }
    if page == NetworkPage::Wifi
      && state
        .interfaces
        .iter()
        .any(|i| matches!(i.kind.as_str(), "wifi" | "802-11-wireless" | "wireless"))
    {
      let value = self.run(
        "nmcli",
        &[
          "-t",
          "-f",
          "IN-USE,SSID,SIGNAL,SECURITY,FREQ,ACTIVE",
          "device",
          "wifi",
          "list",
          "--rescan",
          if scan { "yes" } else { "no" },
        ],
        Duration::from_secs(if scan { 20 } else { 3 }),
      )?;
      state.wifi = parse_wifi(&value);
    }
    if matches!(
      page,
      NetworkPage::Interfaces
        | NetworkPage::Ethernet
        | NetworkPage::Detail(_)
        | NetworkPage::Dns
        | NetworkPage::Status
    ) {
      let names = state
        .interfaces
        .iter()
        .enumerate()
        .filter(|(index, _)| !matches!(page, NetworkPage::Detail(selected) if selected != *index))
        .map(|(index, interface)| (index, interface.name.clone()))
        .collect::<Vec<_>>();
      for batch in names.chunks(4) {
        let results = std::thread::scope(|scope| {
          batch
            .iter()
            .map(|(index, name)| {
              (
                *index,
                scope.spawn(move || {
                  self.run(
                    "nmcli",
                    &["-t", "-f", DEVICE_DETAIL_FIELDS, "device", "show", name],
                    Duration::from_secs(3),
                  )
                }),
              )
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(index, handle)| (index, handle.join().unwrap_or(Err(NetworkError::Timeout))))
            .collect::<Vec<_>>()
        });
        for (index, result) in results {
          if let Ok(details) = result {
            apply_device_details(&mut state.interfaces[index], &details);
          }
        }
        publish(state.clone());
      }
      state.dns = dns_from_interfaces(&state.interfaces).unwrap_or_else(|| read_dns(self));
    }
    Ok(state)
  }

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(runner: R, capabilities: Capabilities) -> Self {
    Self {
      runner,
      capabilities,
    }
  }
  /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run(&self, program: &str, args: &[&str], timeout: Duration) -> Result<String, NetworkError> {
    let request = args
      .iter()
      .fold(ProcessRequest::new(program), |r, a| r.arg(*a))
      .timeout(timeout);
    let output = self.runner.run(&request)?;
    if output.timed_out {
      return Err(NetworkError::Timeout);
    }
    if output.status != Some(0) {
      return Err(NetworkError::Command(terminal_text(
        &String::from_utf8_lossy(&output.stderr),
      )));
    }
    Ok(terminal_text(&String::from_utf8_lossy(&output.stdout)))
  }
  /// Retrieves data for `snapshot` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn snapshot(&self, wifi_scan: bool) -> Result<NetworkSnapshot, NetworkError> {
    if !self.capabilities.has_nmcli {
      return Err(NetworkError::BackendUnavailable);
    }
    let devices = self.run(
      "nmcli",
      &["-t", "-f", DEVICE_STATUS_FIELDS, "device", "status"],
      Duration::from_secs(5),
    )?;
    let mut snapshot = NetworkSnapshot {
      available: true,
      wifi_enabled: self.radio_wifi(),
      connectivity: "unknown".into(),
      ..Default::default()
    };
    snapshot.interfaces = parse_devices(&devices);
    let names = snapshot
      .interfaces
      .iter()
      .map(|interface| interface.name.clone())
      .collect::<Vec<_>>();
    let wifi_args = if wifi_scan {
      vec![
        "-t",
        "-f",
        "IN-USE,SSID,SIGNAL,SECURITY,FREQ,ACTIVE",
        "device",
        "wifi",
        "list",
        "--rescan",
        "yes",
      ]
    } else {
      vec![
        "-t",
        "-f",
        "IN-USE,SSID,SIGNAL,SECURITY,FREQ,ACTIVE",
        "device",
        "wifi",
        "list",
        "--rescan",
        "no",
      ]
    };
    std::thread::scope(|scope| {
      // Per-interface detail lookup, one thread per interface so the
      // `nmcli device show` calls run in parallel, not one after another.
      let mut device_details = Vec::with_capacity(names.len());
      for (index, name) in names.iter().enumerate() {
        let handle = scope.spawn(|| {
          self.run(
            "nmcli",
            &["-t", "-f", DEVICE_DETAIL_FIELDS, "device", "show", name],
            Duration::from_secs(5),
          )
        });
        device_details.push((index, handle));
      }
      // Independent global queries run in parallel with the device details.
      let wifi = scope.spawn(|| self.run("nmcli", &wifi_args, Duration::from_secs(20)));
      let connections = scope.spawn(|| {
        self.run(
          "nmcli",
          &["-t", "-f", "NAME,TYPE,DEVICE", "connection", "show"],
          Duration::from_secs(5),
        )
      });
      let bluetooth = scope.spawn(|| {
        if !self.capabilities.has_bluetoothctl {
          false
        } else {
          self
            .run("bluetoothctl", &["list"], Duration::from_secs(5))
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        }
      });
      // Join everything and apply results in the original interface order.
      for (index, handle) in device_details {
        let details = handle.join().unwrap_or(Err(NetworkError::Timeout));
        if let Ok(details) = details
          && let Some(interface) = snapshot.interfaces.get_mut(index)
        {
          apply_device_details(interface, &details);
        }
      }
      if let Ok(value) = wifi.join().unwrap_or(Err(NetworkError::Timeout)) {
        snapshot.wifi = parse_wifi(&value);
      }
      if let Ok(value) = connections.join().unwrap_or(Err(NetworkError::Timeout)) {
        snapshot.vpn = parse_vpn(&value);
      }
      snapshot.bluetooth_present = bluetooth.join().unwrap_or(false);
    });
    snapshot.dns = dns_from_interfaces(&snapshot.interfaces).unwrap_or_else(|| read_dns(self));
    snapshot.proxy = proxy_from_environment();
    snapshot.connectivity = if snapshot.interfaces.iter().any(|i| i.state == "connected") {
      "online"
    } else {
      "offline"
    }
    .into();
    Ok(snapshot)
  }
  /// Executes the `connect_wifi` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn connect_wifi(&self, ssid: &str, password: Option<&str>) -> Result<(), NetworkError> {
    validate_name(ssid)?;
    let mut request = ProcessRequest::new("nmcli")
      .arg("device")
      .arg("wifi")
      .arg("connect")
      .arg(ssid);
    if let Some(password) = password {
      request = request.arg("password").arg(password);
    }
    let output = self.runner.run(&request.timeout(Duration::from_secs(30)))?;
    if output.timed_out {
      return Err(NetworkError::Timeout);
    }
    if output.status != Some(0) {
      return Err(NetworkError::Command(terminal_text(
        &String::from_utf8_lossy(&output.stderr),
      )));
    }
    Ok(())
  }
  /// Executes the `connection_action` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn connection_action(&self, name: &str, action: &str) -> Result<(), NetworkError> {
    validate_name(name)?;
    if !matches!(action, "up" | "down" | "delete") {
      return Err(NetworkError::InvalidInput);
    }
    self
      .run(
        "nmcli",
        &["connection", action, "id", name],
        Duration::from_secs(30),
      )
      .map(|_| ())
  }
  /// Executes the `device_action` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn device_action(&self, device: &str, action: &str) -> Result<(), NetworkError> {
    validate_name(device)?;
    if !matches!(action, "connect" | "disconnect") {
      return Err(NetworkError::InvalidInput);
    }
    self
      .run(
        "nmcli",
        &["device", action, device],
        Duration::from_secs(30),
      )
      .map(|_| ())
  }
  /// Applies the `set_wifi_enabled` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn set_wifi_enabled(&self, enabled: bool) -> Result<(), NetworkError> {
    self
      .run(
        "nmcli",
        &["radio", "wifi", if enabled { "on" } else { "off" }],
        Duration::from_secs(10),
      )
      .map(|_| ())
  }
  /// Applies the `set_dns` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn set_dns(
    &self,
    connection: &str,
    servers: &[String],
    automatic: bool,
  ) -> Result<(), NetworkError> {
    validate_name(connection)?;
    let value = if automatic { "no" } else { "yes" };
    let ipv4 = servers
      .iter()
      .filter(|v| valid_ipv4(v))
      .cloned()
      .collect::<Vec<_>>()
      .join(",");
    let ipv6 = servers
      .iter()
      .filter(|v| valid_ipv6(v))
      .cloned()
      .collect::<Vec<_>>()
      .join(",");
    let args = [
      "connection",
      "modify",
      "id",
      connection,
      "ipv4.ignore-auto-dns",
      value,
      "ipv6.ignore-auto-dns",
      value,
      "ipv4.dns",
      &ipv4,
      "ipv6.dns",
      &ipv6,
    ];
    self
      .run("nmcli", &args, Duration::from_secs(10))
      .map(|_| ())?;
    self.connection_action(connection, "up")
  }
  /// Executes the `radio_wifi` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn radio_wifi(&self) -> Option<bool> {
    self
      .run("nmcli", &["radio", "wifi"], Duration::from_secs(5))
      .ok()
      .and_then(|v| match v.trim() {
        "enabled" => Some(true),
        "disabled" => Some(false),
        _ => None,
      })
  }
}

/// Defines the constant `DEVICE_STATUS_FIELDS`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const DEVICE_STATUS_FIELDS: &str = "DEVICE,TYPE,STATE,CONNECTION";
/// Defines the constant `DEVICE_DETAIL_FIELDS`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const DEVICE_DETAIL_FIELDS: &str = "GENERAL.DEVICE,GENERAL.TYPE,GENERAL.STATE,GENERAL.CONNECTION,GENERAL.HWADDR,GENERAL.MTU,IP4.ADDRESS,IP4.GATEWAY,IP4.DNS,IP6.ADDRESS,IP6.GATEWAY,IP6.DNS";

/// Converts input data into `parse_field` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_field(value: &str) -> String {
  terminal_text(value).trim().to_string()
}
/// Executes the `split_nmcli` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn split_nmcli(line: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut cur = String::new();
  let mut escaped = false;
  for c in line.chars() {
    if escaped {
      cur.push(c);
      escaped = false;
    } else if c == '\\' {
      escaped = true;
    } else if c == ':' {
      out.push(cur);
      cur = String::new();
    } else {
      cur.push(c);
    }
  }
  if escaped {
    cur.push('\\');
  }
  out.push(cur);
  out.into_iter().map(|s| parse_field(&s)).collect()
}
/// Converts input data into `parse_devices` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_devices(input: &str) -> Vec<InterfaceInfo> {
  input
    .lines()
    .filter_map(|line| {
      let f = split_nmcli(line);
      if f.len() < 4 || f[0].is_empty() {
        return None;
      }
      let driver = Path::new(&format!("/sys/class/net/{}/device/driver", f[0]))
        .read_link()
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().into_owned()));
      Some(InterfaceInfo {
        name: f[0].clone(),
        kind: f[1].clone(),
        state: f[2].clone(),
        connection: (!f[3].is_empty()).then(|| f[3].clone()),
        operstate: read_text(&format!("/sys/class/net/{}/operstate", f[0])).unwrap_or_default(),
        mac: read_text(&format!("/sys/class/net/{}/address", f[0])),
        mtu: read_text(&format!("/sys/class/net/{}/mtu", f[0])).and_then(|v| v.parse().ok()),
        speed: read_text(&format!("/sys/class/net/{}/speed", f[0])),
        driver,
        ..Default::default()
      })
    })
    .collect()
}

/// Applies the `apply_device_details` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn apply_device_details(interface: &mut InterfaceInfo, input: &str) {
  for line in input.lines() {
    let Some((key, raw)) = line.split_once(':') else {
      continue;
    };
    let value = parse_field(&raw.replace("\\:", ":"));
    if value.is_empty() || value == "--" {
      continue;
    }
    match key {
      "GENERAL.TYPE" => interface.kind = value,
      "GENERAL.CONNECTION" => interface.connection = Some(value),
      "GENERAL.HWADDR" => interface.mac = Some(value),
      "GENERAL.MTU" => interface.mtu = value.parse().ok(),
      key if key.starts_with("IP4.ADDRESS") => interface.ipv4.push(value),
      "IP4.GATEWAY" => interface.gateway = Some(value),
      key if key.starts_with("IP4.DNS") => interface.dns.push(value),
      key if key.starts_with("IP6.ADDRESS") => interface.ipv6.push(value),
      "IP6.GATEWAY" if interface.gateway.is_none() => interface.gateway = Some(value),
      key if key.starts_with("IP6.DNS") => interface.dns.push(value),
      _ => {}
    }
  }
}

/// Executes the `dns_from_interfaces` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn dns_from_interfaces(interfaces: &[InterfaceInfo]) -> Option<DnsInfo> {
  let mut servers = interfaces
    .iter()
    .flat_map(|interface| interface.dns.iter().cloned())
    .collect::<Vec<_>>();
  servers.sort();
  servers.dedup();
  (!servers.is_empty()).then(|| DnsInfo {
    source: "NetworkManager".into(),
    servers,
    ..Default::default()
  })
}
/// Converts input data into `parse_wifi` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_wifi(input: &str) -> Vec<WifiNetwork> {
  let mut out = Vec::new();
  for line in input.lines() {
    let f = split_nmcli(line);
    if f.len() < 6 || f[1].is_empty() {
      continue;
    }
    let item = WifiNetwork {
      connected: f[0] == "*" || f[5] == "yes",
      ssid: f[1].clone(),
      signal: f[2].parse().ok(),
      security: f[3].clone(),
      frequency: (!f[4].is_empty()).then(|| f[4].clone()),
      known: f[5] == "yes",
    };
    if let Some(old) = out
      .iter_mut()
      .find(|v: &&mut WifiNetwork| v.ssid == item.ssid)
    {
      if item.signal.unwrap_or(0) > old.signal.unwrap_or(0) {
        *old = item;
      }
    } else {
      out.push(item);
    }
  }
  out
}
/// Converts input data into `parse_vpn` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_vpn(input: &str) -> Vec<VpnConnection> {
  input
    .lines()
    .filter_map(|line| {
      let f = split_nmcli(line);
      if f.len() < 3 || !f[1].to_ascii_lowercase().contains("vpn") {
        return None;
      }
      Some(VpnConnection {
        name: f[0].clone(),
        kind: f[1].clone(),
        active: !f[2].is_empty() && f[2] != "--",
      })
    })
    .collect()
}
/// Executes the `valid_ipv4` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn valid_ipv4(value: &str) -> bool {
  value.parse::<Ipv4Addr>().is_ok()
}
/// Executes the `valid_ipv6` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn valid_ipv6(value: &str) -> bool {
  value.parse::<Ipv6Addr>().is_ok()
}
/// Executes the `validate_name` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn validate_name(value: &str) -> Result<(), NetworkError> {
  if value.is_empty() || value.chars().any(|c| c.is_control()) {
    Err(NetworkError::InvalidInput)
  } else {
    Ok(())
  }
}
/// Retrieves data for `read_text` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_text(path: &str) -> Option<String> {
  std::fs::read_to_string(path)
    .ok()
    .map(|v| terminal_text(&v).trim().to_string())
    .filter(|v| !v.is_empty())
}
/// Retrieves data for `read_dns` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_dns<R: ProcessRunner>(backend: &NetworkBackend<R>) -> DnsInfo {
  if backend.capabilities.has_resolvectl
    && let Ok(text) = backend.run("resolvectl", &["dns"], Duration::from_secs(5))
  {
    let servers = text
      .split_whitespace()
      .filter(|v| v.parse::<Ipv4Addr>().is_ok() || v.parse::<Ipv6Addr>().is_ok())
      .map(str::to_owned)
      .collect();
    return DnsInfo {
      source: "systemd-resolved".into(),
      servers,
      ..Default::default()
    };
  }
  let servers = read_text("/etc/resolv.conf")
    .unwrap_or_default()
    .lines()
    .filter_map(|v| v.strip_prefix("nameserver "))
    .map(str::to_owned)
    .collect();
  DnsInfo {
    source: "/etc/resolv.conf".into(),
    servers,
    ..Default::default()
  }
}
/// Executes the `proxy_from_environment` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn proxy_from_environment() -> ProxyInfo {
  /// Executes the `val` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn val(names: &[&str]) -> Option<String> {
    names
      .iter()
      .find_map(|n| std::env::var(n).ok())
      .map(|v| redact_proxy(&v))
  }
  ProxyInfo {
    http: val(&["HTTP_PROXY", "http_proxy"]),
    https: val(&["HTTPS_PROXY", "https_proxy"]),
    all: val(&["ALL_PROXY", "all_proxy"]),
    no_proxy: val(&["NO_PROXY", "no_proxy"]),
  }
}
/// Executes the `redact_proxy` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn redact_proxy(value: &str) -> String {
  if let Some(at) = value.rfind('@')
    && let Some(scheme) = value.find("://")
  {
    return format!("{}://***@{}", &value[..scheme], &value[at + 1..]);
  }
  value.to_string()
}
#[cfg(test)]
mod tests {
  use super::*;
  use argvus_control_center_core::process::{ProcessOutput, ProcessRequest};
  use std::sync::Mutex;

  #[derive(Default)]
  /// Represents `HostShapeRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  struct HostShapeRunner(Mutex<Vec<ProcessRequest>>);
  impl ProcessRunner for HostShapeRunner {
    /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `nmcli_escaping` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn nmcli_escaping() {
    assert_eq!(
      split_nmcli("wlan0:wifi:connected:Home\\:5G"),
      ["wlan0", "wifi", "connected", "Home:5G"]
    )
  }
  #[test]
  /// Executes the `wifi_deduplicates` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn wifi_deduplicates() {
    let x = parse_wifi("*:Home:80:WPA2:2412:yes\n:Home:40:WPA2:5180:no");
    assert_eq!(x.len(), 1);
    assert_eq!(x[0].signal, Some(80));
  }
  #[test]
  /// Executes the `real_nmcli_status_and_details_are_separate_and_optional` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Retrieves data for `snapshot_uses_valid_status_fields_and_per_device_details` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Represents `FailingDetailRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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

  #[test]
  fn home_does_not_scan_wifi_probe_bluetooth_or_fetch_details() {
    let backend = NetworkBackend::new(
      HostShapeRunner::default(),
      Capabilities {
        has_nmcli: true,
        has_bluetoothctl: true,
        ..Default::default()
      },
    );
    backend
      .load_page(NetworkPage::Home, false, NetworkSnapshot::default(), |_| {})
      .unwrap();
    let requests = backend.runner.0.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|request| {
      request.program != "bluetoothctl"
        && !request
          .args
          .iter()
          .any(|arg| arg == DEVICE_DETAIL_FIELDS || arg == "--rescan")
    }));
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
    /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Retrieves data for `snapshot_ignores_failed_per_device_queries_and_stays_complete` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `addresses_validate` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn addresses_validate() {
    assert!(valid_ipv4("192.0.2.1"));
    assert!(!valid_ipv4("no"));
    assert!(valid_ipv6("2001:db8::1"));
  }
  #[test]
  /// Executes the `proxy_redacts_credentials` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn proxy_redacts_credentials() {
    assert_eq!(
      redact_proxy("http://user:pass@example.test:8080"),
      "http://***@example.test:8080"
    );
  }
}
