//! Implements domain state and models consumed by the UI in crate `argvus control center network`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPage {
  Home,
  Status,
  Interfaces,
  Wifi,
  Ethernet,
  Vpn,
  Dns,
  Proxy,
  /// Details of the interface at this index of `NetworkSnapshot::interfaces`.
  Detail(usize),
  /// Details of the Wi-Fi network at this index of `NetworkSnapshot::wifi`.
  WifiDetail(usize),
  /// Details of the VPN connection at this index of `NetworkSnapshot::vpn`.
  VpnDetail(usize),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `NetworkSnapshot`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct NetworkSnapshot {
  pub available: bool,
  pub wifi_enabled: Option<bool>,
  pub connectivity: String,
  pub interfaces: Vec<InterfaceInfo>,
  pub wifi: Vec<WifiNetwork>,
  pub vpn: Vec<VpnConnection>,
  pub bluetooth_present: bool,
  pub dns: DnsInfo,
  pub proxy: ProxyInfo,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `InterfaceInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct InterfaceInfo {
  pub name: String,
  pub kind: String,
  pub state: String,
  pub operstate: String,
  pub mac: Option<String>,
  pub ipv4: Vec<String>,
  pub ipv6: Vec<String>,
  pub mtu: Option<u32>,
  pub speed: Option<String>,
  pub driver: Option<String>,
  pub connection: Option<String>,
  pub gateway: Option<String>,
  pub dns: Vec<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `WifiNetwork`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct WifiNetwork {
  pub ssid: String,
  pub signal: Option<u8>,
  pub security: String,
  pub frequency: Option<String>,
  pub connected: bool,
  pub known: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `VpnConnection`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct VpnConnection {
  pub name: String,
  pub kind: String,
  pub active: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `DnsInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct DnsInfo {
  pub source: String,
  pub servers: Vec<String>,
  pub search_domains: Vec<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `ProxyInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct ProxyInfo {
  pub http: Option<String>,
  pub https: Option<String>,
  pub all: Option<String>,
  pub no_proxy: Option<String>,
}
