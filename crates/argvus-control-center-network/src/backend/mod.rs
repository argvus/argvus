//! Isolated integration with system tools and APIs of the Network page.
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

mod connections;
pub use connections::{
  SaveOutcome, VpnPlugin, installed_vpn_plugins, parse_profiles, parse_vpn_plugin,
};

#[derive(Debug, Error)]
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
  /// NetworkManager refused the change (polkit denied or no agent answered).
  #[error("not authorized to change network settings")]
  NotAuthorized,
}

pub struct NetworkBackend<R> {
  pub runner: R,
  pub capabilities: Capabilities,
}
impl<R: ProcessRunner> NetworkBackend<R> {
  /// Loads what `page` shows, publishing each part as soon as it is known.
  ///
  /// The device inventory comes first, since every other query depends on
  /// it. The remaining queries are independent `nmcli` calls, so they run
  /// concurrently and every result is published on arrival instead of after
  /// the slowest one. The Home and Status summaries only read what they show:
  /// the details of the connected interfaces and the cached Wi-Fi list
  /// (never a rescan), so they stay cheap on hosts with many interfaces.
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

    let summary = matches!(page, NetworkPage::Home | NetworkPage::Status);
    let wants_profiles = page.needs_profiles();
    let wants_radio = summary || page == NetworkPage::Wifi;
    let wants_connections = summary || page == NetworkPage::Vpn;
    let wants_wifi = (summary || page == NetworkPage::Wifi)
      && state
        .interfaces
        .iter()
        .any(|interface| is_wifi_kind(&interface.kind));
    let rescan = if scan && page == NetworkPage::Wifi {
      "yes"
    } else {
      "no"
    };
    let wifi_args = [
      "-t",
      "-f",
      "IN-USE,SSID,SIGNAL,SECURITY,FREQ,ACTIVE",
      "device",
      "wifi",
      "list",
      "--rescan",
      rescan,
    ];
    let wifi_timeout = Duration::from_secs(if rescan == "yes" { 20 } else { 3 });
    let details_for = state
      .interfaces
      .iter()
      .enumerate()
      .filter(|(index, interface)| match page {
        NetworkPage::Interfaces | NetworkPage::Ethernet | NetworkPage::Dns => true,
        NetworkPage::Detail(selected) => selected == *index,
        NetworkPage::Home | NetworkPage::Status => interface.state == "connected",
        _ => false,
      })
      .map(|(index, interface)| (index, interface.name.clone()))
      .collect::<Vec<_>>();
    let wants_dns = summary
      || matches!(
        page,
        NetworkPage::Interfaces | NetworkPage::Ethernet | NetworkPage::Detail(_) | NetworkPage::Dns
      );

    let mut failure = None;
    let next_detail = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
      let (sender, receiver) = std::sync::mpsc::channel();
      if wants_radio {
        let sender = sender.clone();
        scope.spawn(move || {
          let _ = sender.send(PageReading::Radio(self.radio_wifi()));
        });
      }
      if wants_connections {
        let sender = sender.clone();
        scope.spawn(move || {
          let connections = self.run(
            "nmcli",
            &["-t", "-f", "NAME,TYPE,DEVICE", "connection", "show"],
            Duration::from_secs(3),
          );
          let _ = sender.send(PageReading::Connections(connections));
        });
      }
      if wants_profiles {
        let sender = sender.clone();
        scope.spawn(move || {
          let _ = sender.send(PageReading::Profiles(self.list_profiles()));
        });
      }
      if wants_wifi {
        let sender = sender.clone();
        let wifi_args = &wifi_args;
        scope.spawn(move || {
          let wifi = self.run("nmcli", wifi_args, wifi_timeout);
          let _ = sender.send(PageReading::Wifi(wifi));
        });
      }
      // At most DETAIL_WORKERS `nmcli device show` at once: each worker takes
      // the next interface until none is left.
      for _ in 0..details_for.len().min(DETAIL_WORKERS) {
        let sender = sender.clone();
        let next_detail = &next_detail;
        let details_for = &details_for;
        scope.spawn(move || {
          while let Some((index, name)) =
            details_for.get(next_detail.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
          {
            let details = self.run(
              "nmcli",
              &["-t", "-f", DEVICE_DETAIL_FIELDS, "device", "show", name],
              Duration::from_secs(3),
            );
            let _ = sender.send(PageReading::Details(*index, details));
          }
        });
      }
      drop(sender);
      // Results are applied on this thread only, in arrival order.
      for reading in receiver {
        match reading {
          PageReading::Radio(enabled) => state.wifi_enabled = enabled,
          PageReading::Connections(Ok(value)) => state.vpn = parse_vpn(&value),
          PageReading::Wifi(Ok(value)) => state.wifi = parse_wifi(&value),
          PageReading::Profiles(Ok(profiles)) => state.profiles = profiles,
          PageReading::Connections(Err(error))
          | PageReading::Wifi(Err(error))
          | PageReading::Profiles(Err(error)) => {
            failure.get_or_insert(error);
            continue;
          }
          PageReading::Details(index, Ok(details)) => {
            apply_device_details(&mut state.interfaces[index], &details);
          }
          // A failed detail query keeps the inventory row as it is.
          PageReading::Details(_, Err(_)) => continue,
        }
        publish(state.clone());
      }
    });
    if wants_dns {
      state.dns = dns_from_interfaces(&state.interfaces).unwrap_or_else(|| read_dns(self));
    }
    match failure {
      Some(error) => {
        publish(state.clone());
        Err(error)
      }
      None => Ok(state),
    }
  }

  pub fn new(runner: R, capabilities: Capabilities) -> Self {
    Self {
      runner,
      capabilities,
    }
  }
  fn run(&self, program: &str, args: &[&str], timeout: Duration) -> Result<String, NetworkError> {
    let request = args
      .iter()
      .fold(ProcessRequest::new(program), |r, a| r.arg(*a))
      .timeout(timeout);
    self.execute(&request)
  }

  /// Runs a request and returns its sanitized output, or the error its
  /// standard error describes.
  fn execute(&self, request: &ProcessRequest) -> Result<String, NetworkError> {
    let output = self.runner.run(request)?;
    if output.timed_out {
      return Err(NetworkError::Timeout);
    }
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr));
    if output.status != Some(0) {
      return Err(command_error(stderr));
    }
    Ok(terminal_text(&String::from_utf8_lossy(&output.stdout)))
  }
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
  pub fn set_wifi_enabled(&self, enabled: bool) -> Result<(), NetworkError> {
    self
      .run(
        "nmcli",
        &["radio", "wifi", if enabled { "on" } else { "off" }],
        Duration::from_secs(10),
      )
      .map(|_| ())
  }
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

/// The error of a failed `nmcli`, from its standard error. A polkit refusal
/// gets its own variant, so the page can explain it in the user's language.
fn command_error(stderr: String) -> NetworkError {
  let lowered = stderr.to_ascii_lowercase();
  if lowered.contains("not authorized") || lowered.contains("insufficient privileges") {
    NetworkError::NotAuthorized
  } else {
    NetworkError::Command(stderr.trim().to_owned())
  }
}

/// How many `nmcli device show` calls a page load runs at once.
const DETAIL_WORKERS: usize = 4;

/// One result of the concurrent queries of [`NetworkBackend::load_page`].
enum PageReading {
  Radio(Option<bool>),
  Connections(Result<String, NetworkError>),
  Wifi(Result<String, NetworkError>),
  Profiles(Result<Vec<ConnectionProfile>, NetworkError>),
  Details(usize, Result<String, NetworkError>),
}

/// Whether an nmcli device type is a Wi-Fi card.
pub fn is_wifi_kind(kind: &str) -> bool {
  let kind = kind.to_ascii_lowercase();
  kind.contains("wifi") || kind.contains("wireless")
}

pub const DEVICE_STATUS_FIELDS: &str = "DEVICE,TYPE,STATE,CONNECTION";
pub const DEVICE_DETAIL_FIELDS: &str = "GENERAL.DEVICE,GENERAL.TYPE,GENERAL.STATE,GENERAL.CONNECTION,GENERAL.HWADDR,GENERAL.MTU,IP4.ADDRESS,IP4.GATEWAY,IP4.DNS,IP6.ADDRESS,IP6.GATEWAY,IP6.DNS";

fn parse_field(value: &str) -> String {
  terminal_text(value).trim().to_string()
}
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
pub fn parse_vpn(input: &str) -> Vec<VpnConnection> {
  input
    .lines()
    .filter_map(|line| {
      let f = split_nmcli(line);
      if f.len() < 3 || !is_vpn_kind(&f[1]) {
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
/// Whether an nmcli connection type is a VPN: a VPN plugin or WireGuard.
pub fn is_vpn_kind(kind: &str) -> bool {
  let kind = kind.to_ascii_lowercase();
  kind.contains("vpn") || kind == "wireguard"
}
pub fn valid_ipv4(value: &str) -> bool {
  value.parse::<Ipv4Addr>().is_ok()
}
pub fn valid_ipv6(value: &str) -> bool {
  value.parse::<Ipv6Addr>().is_ok()
}
fn validate_name(value: &str) -> Result<(), NetworkError> {
  if value.is_empty() || value.chars().any(|c| c.is_control()) {
    Err(NetworkError::InvalidInput)
  } else {
    Ok(())
  }
}
fn read_text(path: &str) -> Option<String> {
  std::fs::read_to_string(path)
    .ok()
    .map(|v| terminal_text(&v).trim().to_string())
    .filter(|v| !v.is_empty())
}
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
fn proxy_from_environment() -> ProxyInfo {
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
pub fn redact_proxy(value: &str) -> String {
  if let Some(at) = value.rfind('@')
    && let Some(scheme) = value.find("://")
  {
    return format!("{}://***@{}", &value[..scheme], &value[at + 1..]);
  }
  value.to_string()
}
#[cfg(test)]
mod tests;
