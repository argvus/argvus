//! The kinds of NetworkManager connection the editor creates and edits, and
//! what each one needs from the host.

use argvus_control_center_core::capabilities::Capabilities;
use argvus_tui::icons;

/// A connection profile kind, as the Add page offers it.
///
/// A Hotspot is a Wi-Fi profile in access-point mode: NetworkManager stores
/// both as `802-11-wireless`, so [`ConnectionKind::from_nmcli`] tells them
/// apart by the Wi-Fi mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectionKind {
  Ethernet,
  Wifi,
  Hotspot,
  Pppoe,
  Bridge,
  Bond,
  Team,
  Vlan,
  IpTunnel,
  Loopback,
  WireGuard,
  Vpn,
  /// A type the editor does not know (InfiniBand, OVS, MACsec...): it can be
  /// renamed, activated and deleted, but its own settings are not shown.
  Other,
}

/// Where a kind is listed on the Add page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindGroup {
  Hardware,
  Virtual,
  Vpn,
}

impl ConnectionKind {
  /// The kinds the Add page offers, in its order.
  pub const ADDABLE: [Self; 12] = [
    Self::Ethernet,
    Self::Wifi,
    Self::Hotspot,
    Self::Pppoe,
    Self::Bridge,
    Self::Bond,
    Self::Team,
    Self::Vlan,
    Self::IpTunnel,
    Self::Loopback,
    Self::WireGuard,
    Self::Vpn,
  ];

  /// The `type` argument of `nmcli connection add`.
  pub fn nmcli_type(self) -> &'static str {
    match self {
      Self::Ethernet | Self::Other => "ethernet",
      Self::Wifi | Self::Hotspot => "wifi",
      Self::Pppoe => "pppoe",
      Self::Bridge => "bridge",
      Self::Bond => "bond",
      Self::Team => "team",
      Self::Vlan => "vlan",
      Self::IpTunnel => "ip-tunnel",
      Self::Loopback => "loopback",
      Self::WireGuard => "wireguard",
      Self::Vpn => "vpn",
    }
  }

  /// The kind of an `nmcli` connection type (long or short name). `wifi_mode`
  /// is `802-11-wireless.mode`, which turns a Wi-Fi profile into a Hotspot.
  pub fn from_nmcli(type_name: &str, wifi_mode: Option<&str>) -> Self {
    match type_name.trim() {
      "802-3-ethernet" | "ethernet" => Self::Ethernet,
      "802-11-wireless" | "wifi" if wifi_mode == Some("ap") => Self::Hotspot,
      "802-11-wireless" | "wifi" => Self::Wifi,
      "pppoe" => Self::Pppoe,
      "bridge" => Self::Bridge,
      "bond" => Self::Bond,
      "team" => Self::Team,
      "vlan" => Self::Vlan,
      "ip-tunnel" => Self::IpTunnel,
      "loopback" => Self::Loopback,
      "wireguard" => Self::WireGuard,
      "vpn" => Self::Vpn,
      _ => Self::Other,
    }
  }

  /// Translation key of the kind's name.
  pub fn label_key(self) -> &'static str {
    match self {
      Self::Ethernet => "control_center.ethernet",
      Self::Wifi => "control_center.wi_fi",
      Self::Hotspot => "control_center.net_kind_hotspot",
      Self::Pppoe => "control_center.net_kind_dsl",
      Self::Bridge => "control_center.net_kind_bridge",
      Self::Bond => "control_center.net_kind_bond",
      Self::Team => "control_center.net_kind_team",
      Self::Vlan => "control_center.net_kind_vlan",
      Self::IpTunnel => "control_center.net_kind_ip_tunnel",
      Self::Loopback => "control_center.net_kind_loopback",
      Self::WireGuard => "control_center.net_kind_wireguard",
      Self::Vpn => "control_center.vpn",
      Self::Other => "control_center.net_kind_other",
    }
  }

  pub fn icon(self) -> &'static str {
    match self {
      Self::Ethernet => icons::ETHERNET,
      Self::Wifi => icons::WIFI,
      Self::Hotspot => icons::HOTSPOT,
      Self::Pppoe => icons::DSL,
      Self::Bridge => icons::BRIDGE,
      Self::Bond => icons::BOND,
      Self::Team => icons::TEAM,
      Self::Vlan => icons::VLAN,
      Self::IpTunnel => icons::TUNNEL,
      Self::Loopback => icons::LOOPBACK,
      Self::WireGuard => icons::WIREGUARD,
      Self::Vpn => icons::VPN,
      Self::Other => icons::NETWORK,
    }
  }

  pub fn group(self) -> KindGroup {
    match self {
      Self::Ethernet | Self::Wifi | Self::Hotspot | Self::Pppoe | Self::Other => {
        KindGroup::Hardware
      }
      Self::Bridge
      | Self::Bond
      | Self::Team
      | Self::Vlan
      | Self::IpTunnel
      | Self::Loopback
      | Self::WireGuard => KindGroup::Virtual,
      Self::Vpn => KindGroup::Vpn,
    }
  }

  /// The `port-type` of the profiles that join this controller, for the kinds
  /// that take ports.
  pub fn port_type(self) -> Option<&'static str> {
    match self {
      Self::Bridge => Some("bridge"),
      Self::Bond => Some("bond"),
      Self::Team => Some("team"),
      _ => None,
    }
  }

  /// Whether the profile names a device that already exists (a card), as
  /// opposed to a virtual interface it creates.
  pub fn binds_existing_device(self) -> bool {
    matches!(self, Self::Ethernet | Self::Wifi | Self::Hotspot)
  }

  /// The device kinds (`nmcli device` TYPE) this profile can be bound to.
  pub fn device_types(self) -> &'static [&'static str] {
    match self {
      Self::Ethernet => &["ethernet"],
      Self::Wifi | Self::Hotspot => &["wifi"],
      _ => &[],
    }
  }

  /// Why this kind cannot be created on this host, as a translation key, or
  /// `None` when nothing is missing.
  pub fn missing_requirement(self, capabilities: &Capabilities) -> Option<&'static str> {
    match self {
      Self::Team if !capabilities.has_teamd => Some("control_center.net_requires_libteam"),
      Self::Pppoe if !capabilities.has_pppd => Some("control_center.net_requires_ppp"),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn nmcli_types_round_trip_and_hotspot_is_wifi_in_ap_mode() {
    for kind in ConnectionKind::ADDABLE {
      let mode = (kind == ConnectionKind::Hotspot).then_some("ap");
      assert_eq!(ConnectionKind::from_nmcli(kind.nmcli_type(), mode), kind);
    }
    assert_eq!(
      ConnectionKind::from_nmcli("802-11-wireless", Some("infrastructure")),
      ConnectionKind::Wifi
    );
    assert_eq!(
      ConnectionKind::from_nmcli("802-3-ethernet", None),
      ConnectionKind::Ethernet
    );
    assert_eq!(
      ConnectionKind::from_nmcli("infiniband", None),
      ConnectionKind::Other
    );
  }

  #[test]
  fn missing_daemons_disable_team_and_dsl() {
    let none = Capabilities::default();
    assert!(ConnectionKind::Team.missing_requirement(&none).is_some());
    assert!(ConnectionKind::Pppoe.missing_requirement(&none).is_some());
    assert!(ConnectionKind::Bridge.missing_requirement(&none).is_none());
    let full = Capabilities {
      has_teamd: true,
      has_pppd: true,
      ..Default::default()
    };
    assert!(ConnectionKind::Team.missing_requirement(&full).is_none());
  }
}
