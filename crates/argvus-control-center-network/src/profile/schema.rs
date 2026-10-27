//! The settings the connection editor shows, one table per section.
//!
//! Each [`Property`] names its `nmcli` property, its label, how it is edited
//! and validated, and when it applies. The editor rows, the validation and the
//! `nmcli` arguments are all derived from these tables, so a field is declared
//! once. Keys that are not `nmcli` properties are virtual:
//!
//! - `bond.options.<option>` and `vpn.data.<key>` are entries of the
//!   `bond.options` and `vpn.data` dictionaries, written back as a whole;
//! - `vpn.secrets.<key>` is an entry of the `vpn.secrets` dictionary;
//! - [`ALL_USERS`] is `connection.permissions`: empty for everyone, or the
//!   current user only.

use super::ConnectionKind;

/// Virtual key: whether every user may use the profile.
pub const ALL_USERS: &str = "argvus.all-users";
/// Prefix of the virtual keys stored in `bond.options`.
pub const BOND_OPTION_PREFIX: &str = "bond.options.";
/// Prefix of the virtual keys stored in `vpn.data`.
pub const VPN_DATA_PREFIX: &str = "vpn.data.";
/// Prefix of the virtual keys stored in `vpn.secrets`.
pub const VPN_SECRET_PREFIX: &str = "vpn.secrets.";
/// `vpn.service-type` of the OpenVPN plugin, which has its own form.
pub const OPENVPN_SERVICE: &str = "org.freedesktop.NetworkManager.openvpn";

/// Address family of an address field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
  V4,
  V6,
  Any,
}

/// How a typed value is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRule {
  /// Anything without control characters, possibly empty.
  Free,
  /// Anything without control characters, never empty.
  Required,
  /// A kernel interface name (at most 15 bytes, no `/`, `:` or space).
  InterfaceName { required: bool },
  /// A MAC address, or one of NetworkManager's `preserve`, `permanent`,
  /// `random`, `stable`; empty keeps the default.
  ClonedMac,
  /// `auto` or a size from 68 to 65535.
  Mtu,
  /// A host address of `family`.
  Address { family: Family, required: bool },
  /// 1 to 32 bytes.
  Ssid,
  /// An integer in the range, or empty for the default.
  OptionalInteger { min: i64, max: i64 },
  /// `host:port`.
  Endpoint { required: bool },
}

/// How a secret is checked. Secrets are never read back from NetworkManager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretRule {
  Free,
  /// A WPA passphrase: 8 to 63 ASCII characters, or 64 hexadecimal digits.
  WpaPassphrase,
  /// A WireGuard key: 32 bytes in base64.
  WireGuardKey,
}

/// The entries of a list field, edited one per row on a sub-page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListRule {
  /// `address/prefix`.
  Cidr(Family),
  /// A host address.
  Address(Family),
  /// A DNS search domain.
  Domain,
  /// `destination/prefix [next-hop] [metric]`.
  Route(Family),
  /// `key=value`.
  KeyValue,
}

/// One option of a choice field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice {
  pub value: &'static str,
  /// Translation key of the label.
  pub label: &'static str,
}

/// How a field is edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Widget {
  /// Enter opens a text editor.
  Text(TextRule),
  /// Enter opens a masked editor.
  Secret(SecretRule),
  /// Enter or Space switches between `yes` and `no`.
  Toggle,
  /// ←/→ (and Enter) cycle the options.
  Choice(&'static [Choice]),
  /// ←/→ change the number by `step`; Enter types it.
  Number { min: i64, max: i64, step: i64 },
  /// Enter opens the list.
  List(ListRule),
  /// ←/→ cycle the host devices that fit; Enter types a name.
  Device { required: bool },
}

/// When a field applies, from the value of another field of the same profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
  Always,
  /// The value of the key is one of the values.
  In(&'static str, &'static [&'static str]),
  /// The value of the key is none of the values.
  NotIn(&'static str, &'static [&'static str]),
}

/// One editable setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Property {
  /// The `nmcli` property, or a virtual key (see the module documentation).
  pub key: &'static str,
  /// Translation key of the label.
  pub label: &'static str,
  pub widget: Widget,
  /// NetworkManager's default, as `nmcli` shows it. A new profile only sends
  /// the fields that differ from it.
  pub default: &'static str,
  /// The row is left out when this does not hold.
  pub shown: Condition,
  /// The row is shown disabled when this does not hold.
  pub enabled: Condition,
}

impl Property {
  const fn new(key: &'static str, label: &'static str, widget: Widget) -> Self {
    Self {
      key,
      label,
      widget,
      default: "",
      shown: Condition::Always,
      enabled: Condition::Always,
    }
  }

  const fn default_value(mut self, value: &'static str) -> Self {
    self.default = value;
    self
  }

  const fn shown_when(mut self, condition: Condition) -> Self {
    self.shown = condition;
    self
  }

  const fn enabled_when(mut self, condition: Condition) -> Self {
    self.enabled = condition;
    self
  }

  /// Whether the field holds a secret.
  pub fn is_secret(&self) -> bool {
    matches!(self.widget, Widget::Secret(_))
  }
}

/// Identity of an editor section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SectionId {
  General,
  Ethernet,
  Wifi,
  WifiSecurity,
  Bridge,
  Bond,
  Team,
  Vlan,
  IpTunnel,
  Loopback,
  Pppoe,
  WireGuard,
  OpenVpn,
  Vpn,
  Ipv4,
  Ipv6,
}

/// A titled group of fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
  pub id: SectionId,
  /// Translation key of the title.
  pub title: &'static str,
  pub properties: &'static [Property],
}

const fn text(key: &'static str, label: &'static str, rule: TextRule) -> Property {
  Property::new(key, label, Widget::Text(rule))
}

const fn secret(key: &'static str, label: &'static str, rule: SecretRule) -> Property {
  Property::new(key, label, Widget::Secret(rule))
}

const fn toggle(key: &'static str, label: &'static str, default: &'static str) -> Property {
  Property::new(key, label, Widget::Toggle).default_value(default)
}

const fn choice(
  key: &'static str,
  label: &'static str,
  options: &'static [Choice],
  default: &'static str,
) -> Property {
  Property::new(key, label, Widget::Choice(options)).default_value(default)
}

const fn number(
  key: &'static str,
  label: &'static str,
  (min, max, step): (i64, i64, i64),
  default: &'static str,
) -> Property {
  Property::new(key, label, Widget::Number { min, max, step }).default_value(default)
}

const fn list(key: &'static str, label: &'static str, rule: ListRule) -> Property {
  Property::new(key, label, Widget::List(rule))
}

const fn option(value: &'static str, label: &'static str) -> Choice {
  Choice { value, label }
}

const MTU: TextRule = TextRule::Mtu;

const GENERAL: Section = Section {
  id: SectionId::General,
  title: "control_center.net_section_general",
  properties: &[
    text(
      "connection.id",
      "control_center.net_f_name",
      TextRule::Required,
    ),
    Property::new(
      "connection.interface-name",
      "control_center.net_f_interface",
      Widget::Device { required: false },
    ),
    toggle(
      "connection.autoconnect",
      "control_center.net_f_autoconnect",
      "yes",
    ),
    number(
      "connection.autoconnect-priority",
      "control_center.net_f_priority",
      (-999, 999, 1),
      "0",
    ),
    toggle(ALL_USERS, "control_center.net_f_all_users", "yes"),
    choice(
      "connection.metered",
      "control_center.net_f_metered",
      &[
        option("unknown", "control_center.net_o_automatic"),
        option("yes", "control_center.yes"),
        option("no", "control_center.no"),
      ],
      "unknown",
    ),
  ],
};

/// General settings of a port: it has no addresses of its own.
const PORT_GENERAL: Section = Section {
  id: SectionId::General,
  title: "control_center.net_section_general",
  properties: &[
    text(
      "connection.id",
      "control_center.net_f_name",
      TextRule::Required,
    ),
    Property::new(
      "connection.interface-name",
      "control_center.net_f_interface",
      Widget::Device { required: true },
    ),
    toggle(
      "connection.autoconnect",
      "control_center.net_f_autoconnect",
      "yes",
    ),
  ],
};

const ETHERNET: Section = Section {
  id: SectionId::Ethernet,
  title: "control_center.ethernet",
  properties: &[
    text(
      "802-3-ethernet.cloned-mac-address",
      "control_center.net_f_cloned_mac",
      TextRule::ClonedMac,
    ),
    text("802-3-ethernet.mtu", "control_center.net_f_mtu", MTU).default_value("auto"),
  ],
};

const WIFI_MODES: &[Choice] = &[
  option("infrastructure", "control_center.net_o_client"),
  option("ap", "control_center.net_kind_hotspot"),
  option("adhoc", "control_center.net_o_adhoc"),
];

const WIFI_BANDS: &[Choice] = &[
  option("", "control_center.net_o_automatic"),
  option("a", "control_center.net_o_band_5"),
  option("bg", "control_center.net_o_band_2_4"),
];

const WIFI: Section = Section {
  id: SectionId::Wifi,
  title: "control_center.wi_fi",
  properties: &[
    text(
      "802-11-wireless.ssid",
      "control_center.net_f_ssid",
      TextRule::Ssid,
    ),
    choice(
      "802-11-wireless.mode",
      "control_center.net_f_mode",
      WIFI_MODES,
      "infrastructure",
    ),
    choice(
      "802-11-wireless.band",
      "control_center.net_f_band",
      WIFI_BANDS,
      "",
    ),
    number(
      "802-11-wireless.channel",
      "control_center.net_f_channel",
      (0, 196, 1),
      "0",
    )
    .enabled_when(Condition::NotIn("802-11-wireless.band", &[""])),
    toggle(
      "802-11-wireless.hidden",
      "control_center.net_f_hidden",
      "no",
    ),
    text(
      "802-11-wireless.cloned-mac-address",
      "control_center.net_f_cloned_mac",
      TextRule::ClonedMac,
    ),
    text("802-11-wireless.mtu", "control_center.net_f_mtu", MTU).default_value("auto"),
  ],
};

/// `802-11-wireless-security.key-mgmt` of an open network: the setting is
/// removed instead.
pub const OPEN_NETWORK: &str = "";
const KEY_MGMT: &str = "802-11-wireless-security.key-mgmt";
const PERSONAL: &[&str] = &["wpa-psk", "sae"];
const ENTERPRISE: &[&str] = &["wpa-eap"];

const WIFI_SECURITY: Section = Section {
  id: SectionId::WifiSecurity,
  title: "control_center.net_section_security",
  properties: &[
    choice(
      KEY_MGMT,
      "control_center.net_f_security",
      &[
        option(OPEN_NETWORK, "control_center.net_o_none"),
        option("wpa-psk", "control_center.net_o_wpa_personal"),
        option("sae", "control_center.net_o_wpa3_personal"),
        option("owe", "control_center.net_o_enhanced_open"),
        option("wpa-eap", "control_center.net_o_wpa_enterprise"),
      ],
      OPEN_NETWORK,
    ),
    secret(
      "802-11-wireless-security.psk",
      "control_center.net_f_password",
      SecretRule::WpaPassphrase,
    )
    .shown_when(Condition::In(KEY_MGMT, PERSONAL)),
    choice(
      "802-1x.eap",
      "control_center.net_f_eap",
      &[
        option("peap", "control_center.net_o_peap"),
        option("ttls", "control_center.net_o_ttls"),
      ],
      "",
    )
    .shown_when(Condition::In(KEY_MGMT, ENTERPRISE)),
    text(
      "802-1x.identity",
      "control_center.net_f_identity",
      TextRule::Required,
    )
    .shown_when(Condition::In(KEY_MGMT, ENTERPRISE)),
    text(
      "802-1x.anonymous-identity",
      "control_center.net_f_anonymous_identity",
      TextRule::Free,
    )
    .shown_when(Condition::In(KEY_MGMT, ENTERPRISE)),
    choice(
      "802-1x.phase2-auth",
      "control_center.net_f_phase2",
      &[
        option("mschapv2", "control_center.net_o_mschapv2"),
        option("pap", "control_center.net_o_pap"),
        option("gtc", "control_center.net_o_gtc"),
        option("md5", "control_center.net_o_md5"),
      ],
      "",
    )
    .shown_when(Condition::In(KEY_MGMT, ENTERPRISE)),
    secret(
      "802-1x.password",
      "control_center.net_f_password",
      SecretRule::Free,
    )
    .shown_when(Condition::In(KEY_MGMT, ENTERPRISE)),
    text(
      "802-1x.ca-cert",
      "control_center.net_f_ca_certificate",
      TextRule::Free,
    )
    .shown_when(Condition::In(KEY_MGMT, ENTERPRISE)),
    text(
      "802-1x.domain-suffix-match",
      "control_center.net_f_domain_match",
      TextRule::Free,
    )
    .shown_when(Condition::In(KEY_MGMT, ENTERPRISE)),
  ],
};

const BRIDGE: Section = Section {
  id: SectionId::Bridge,
  title: "control_center.net_kind_bridge",
  properties: &[
    toggle("bridge.stp", "control_center.net_f_stp", "yes"),
    number(
      "bridge.priority",
      "control_center.net_f_bridge_priority",
      (0, 65535, 4096),
      "32768",
    )
    .enabled_when(Condition::In("bridge.stp", &["yes"])),
    number(
      "bridge.forward-delay",
      "control_center.net_f_forward_delay",
      (2, 30, 1),
      "15",
    )
    .enabled_when(Condition::In("bridge.stp", &["yes"])),
    number(
      "bridge.hello-time",
      "control_center.net_f_hello_time",
      (1, 10, 1),
      "2",
    )
    .enabled_when(Condition::In("bridge.stp", &["yes"])),
    number(
      "bridge.max-age",
      "control_center.net_f_max_age",
      (6, 40, 1),
      "20",
    )
    .enabled_when(Condition::In("bridge.stp", &["yes"])),
    number(
      "bridge.ageing-time",
      "control_center.net_f_ageing_time",
      (0, 1_000_000, 10),
      "300",
    ),
    toggle(
      "bridge.multicast-snooping",
      "control_center.net_f_multicast_snooping",
      "yes",
    ),
    toggle(
      "bridge.vlan-filtering",
      "control_center.net_f_vlan_filtering",
      "no",
    ),
    text(
      "bridge.mac-address",
      "control_center.net_f_mac",
      TextRule::ClonedMac,
    ),
  ],
};

const BOND_MODE: &str = "bond.options.mode";

const BOND: Section = Section {
  id: SectionId::Bond,
  title: "control_center.net_kind_bond",
  properties: &[
    choice(
      BOND_MODE,
      "control_center.net_f_mode",
      &[
        option("balance-rr", "control_center.net_o_balance_rr"),
        option("active-backup", "control_center.net_o_active_backup"),
        option("balance-xor", "control_center.net_o_balance_xor"),
        option("broadcast", "control_center.net_o_broadcast"),
        option("802.3ad", "control_center.net_o_8023ad"),
        option("balance-tlb", "control_center.net_o_balance_tlb"),
        option("balance-alb", "control_center.net_o_balance_alb"),
      ],
      "balance-rr",
    ),
    text(
      "bond.options.primary",
      "control_center.net_f_primary",
      TextRule::InterfaceName { required: false },
    )
    .shown_when(Condition::In(
      BOND_MODE,
      &["active-backup", "balance-tlb", "balance-alb"],
    )),
    text(
      "bond.options.miimon",
      "control_center.net_f_miimon",
      TextRule::OptionalInteger {
        min: 0,
        max: 1_000_000,
      },
    ),
    text(
      "bond.options.updelay",
      "control_center.net_f_updelay",
      TextRule::OptionalInteger {
        min: 0,
        max: 1_000_000,
      },
    ),
    text(
      "bond.options.downdelay",
      "control_center.net_f_downdelay",
      TextRule::OptionalInteger {
        min: 0,
        max: 1_000_000,
      },
    ),
    choice(
      "bond.options.xmit_hash_policy",
      "control_center.net_f_hash_policy",
      &[
        option("", "control_center.net_o_automatic"),
        option("layer2", "control_center.net_o_layer2"),
        option("layer2+3", "control_center.net_o_layer23"),
        option("layer3+4", "control_center.net_o_layer34"),
      ],
      "",
    )
    .shown_when(Condition::In(
      BOND_MODE,
      &["balance-xor", "802.3ad", "balance-tlb"],
    )),
    choice(
      "bond.options.lacp_rate",
      "control_center.net_f_lacp_rate",
      &[
        option("", "control_center.net_o_automatic"),
        option("slow", "control_center.net_o_slow"),
        option("fast", "control_center.net_o_fast"),
      ],
      "",
    )
    .shown_when(Condition::In(BOND_MODE, &["802.3ad"])),
  ],
};

const TEAM: Section = Section {
  id: SectionId::Team,
  title: "control_center.net_kind_team",
  properties: &[choice(
    "team.runner",
    "control_center.net_f_runner",
    &[
      option("roundrobin", "control_center.net_o_round_robin"),
      option("broadcast", "control_center.net_o_broadcast"),
      option("activebackup", "control_center.net_o_active_backup"),
      option("loadbalance", "control_center.net_o_load_balance"),
      option("lacp", "control_center.net_o_lacp"),
    ],
    "",
  )],
};

const VLAN: Section = Section {
  id: SectionId::Vlan,
  title: "control_center.net_kind_vlan",
  properties: &[
    Property::new(
      "vlan.parent",
      "control_center.net_f_parent",
      Widget::Device { required: true },
    ),
    number("vlan.id", "control_center.net_f_vlan_id", (0, 4094, 1), "0"),
  ],
};

const TUNNEL_MODE: &str = "ip-tunnel.mode";
const KEYED_TUNNELS: &[&str] = &["gre", "ip6gre", "gretap", "ip6gretap", "vti", "vti6"];

const IP_TUNNEL: Section = Section {
  id: SectionId::IpTunnel,
  title: "control_center.net_kind_ip_tunnel",
  properties: &[
    choice(
      TUNNEL_MODE,
      "control_center.net_f_mode",
      &[
        option("ipip", "control_center.net_o_ipip"),
        option("gre", "control_center.net_o_gre"),
        option("sit", "control_center.net_o_sit"),
        option("isatap", "control_center.net_o_isatap"),
        option("vti", "control_center.net_o_vti"),
        option("ip6ip6", "control_center.net_o_ip6ip6"),
        option("ipip6", "control_center.net_o_ipip6"),
        option("ip6gre", "control_center.net_o_ip6gre"),
        option("vti6", "control_center.net_o_vti6"),
        option("gretap", "control_center.net_o_gretap"),
        option("ip6gretap", "control_center.net_o_ip6gretap"),
      ],
      "",
    ),
    Property::new(
      "ip-tunnel.parent",
      "control_center.net_f_parent",
      Widget::Device { required: false },
    ),
    text(
      "ip-tunnel.local",
      "control_center.net_f_local_address",
      TextRule::Address {
        family: Family::Any,
        required: false,
      },
    ),
    text(
      "ip-tunnel.remote",
      "control_center.net_f_remote_address",
      TextRule::Address {
        family: Family::Any,
        required: true,
      },
    ),
    number(
      "ip-tunnel.ttl",
      "control_center.net_f_ttl",
      (0, 255, 1),
      "0",
    ),
    text(
      "ip-tunnel.input-key",
      "control_center.net_f_input_key",
      TextRule::Free,
    )
    .shown_when(Condition::In(TUNNEL_MODE, KEYED_TUNNELS)),
    text(
      "ip-tunnel.output-key",
      "control_center.net_f_output_key",
      TextRule::Free,
    )
    .shown_when(Condition::In(TUNNEL_MODE, KEYED_TUNNELS)),
    text("ip-tunnel.mtu", "control_center.net_f_mtu", MTU).default_value("auto"),
  ],
};

const LOOPBACK: Section = Section {
  id: SectionId::Loopback,
  title: "control_center.net_kind_loopback",
  properties: &[text("loopback.mtu", "control_center.net_f_mtu", MTU).default_value("auto")],
};

const PPPOE: Section = Section {
  id: SectionId::Pppoe,
  title: "control_center.net_kind_dsl",
  properties: &[
    Property::new(
      "pppoe.parent",
      "control_center.net_f_parent",
      Widget::Device { required: true },
    ),
    text(
      "pppoe.username",
      "control_center.net_f_username",
      TextRule::Required,
    ),
    secret(
      "pppoe.password",
      "control_center.net_f_password",
      SecretRule::Free,
    ),
    text(
      "pppoe.service",
      "control_center.net_f_service",
      TextRule::Free,
    ),
  ],
};

const WIREGUARD: Section = Section {
  id: SectionId::WireGuard,
  title: "control_center.net_kind_wireguard",
  properties: &[
    secret(
      "wireguard.private-key",
      "control_center.net_f_private_key",
      SecretRule::WireGuardKey,
    ),
    number(
      "wireguard.listen-port",
      "control_center.net_f_listen_port",
      (0, 65535, 1),
      "0",
    ),
    toggle(
      "wireguard.peer-routes",
      "control_center.net_f_peer_routes",
      "yes",
    ),
    text("wireguard.mtu", "control_center.net_f_mtu", MTU).default_value("auto"),
  ],
};

const OPENVPN_TYPE: &str = "vpn.data.connection-type";
const OPENVPN_CERTIFICATES: &[&str] = &["tls", "password-tls"];
const OPENVPN_PASSWORDS: &[&str] = &["password", "password-tls"];

const OPENVPN: Section = Section {
  id: SectionId::OpenVpn,
  title: "control_center.net_section_openvpn",
  properties: &[
    text(
      "vpn.data.remote",
      "control_center.net_f_gateway_host",
      TextRule::Required,
    ),
    choice(
      OPENVPN_TYPE,
      "control_center.net_f_authentication",
      &[
        option("tls", "control_center.net_o_certificates"),
        option("password", "control_center.net_f_password"),
        option("password-tls", "control_center.net_o_password_certificates"),
        option("static-key", "control_center.net_o_static_key"),
      ],
      "",
    ),
    text(
      "vpn.data.ca",
      "control_center.net_f_ca_certificate",
      TextRule::Free,
    )
    .shown_when(Condition::NotIn(OPENVPN_TYPE, &["static-key"])),
    text(
      "vpn.data.cert",
      "control_center.net_f_user_certificate",
      TextRule::Free,
    )
    .shown_when(Condition::In(OPENVPN_TYPE, OPENVPN_CERTIFICATES)),
    text(
      "vpn.data.key",
      "control_center.net_f_user_key",
      TextRule::Free,
    )
    .shown_when(Condition::In(OPENVPN_TYPE, OPENVPN_CERTIFICATES)),
    text(
      "vpn.data.static-key",
      "control_center.net_f_static_key_file",
      TextRule::Free,
    )
    .shown_when(Condition::In(OPENVPN_TYPE, &["static-key"])),
    text(
      "vpn.data.username",
      "control_center.net_f_username",
      TextRule::Free,
    )
    .shown_when(Condition::In(OPENVPN_TYPE, OPENVPN_PASSWORDS)),
    secret(
      "vpn.secrets.password",
      "control_center.net_f_password",
      SecretRule::Free,
    )
    .shown_when(Condition::In(OPENVPN_TYPE, OPENVPN_PASSWORDS)),
  ],
};

/// Any other VPN plugin: its data as `key=value` entries.
const VPN: Section = Section {
  id: SectionId::Vpn,
  title: "control_center.vpn",
  properties: &[
    text(
      "vpn.user-name",
      "control_center.net_f_username",
      TextRule::Free,
    ),
    list(
      "vpn.data",
      "control_center.net_f_plugin_data",
      ListRule::KeyValue,
    ),
    secret(
      "vpn.secrets.password",
      "control_center.net_f_password",
      SecretRule::Free,
    ),
  ],
};

const IPV4_METHOD: &str = "ipv4.method";
const IPV6_METHOD: &str = "ipv6.method";

const IPV4: Section = Section {
  id: SectionId::Ipv4,
  title: "control_center.net_section_ipv4",
  properties: &[
    choice(
      IPV4_METHOD,
      "control_center.net_f_method",
      &[
        option("auto", "control_center.net_o_automatic"),
        option("manual", "control_center.net_o_manual"),
        option("link-local", "control_center.net_o_link_local"),
        option("shared", "control_center.net_o_shared"),
        option("disabled", "control_center.disabled"),
      ],
      "auto",
    ),
    list(
      "ipv4.addresses",
      "control_center.net_f_addresses",
      ListRule::Cidr(Family::V4),
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["auto", "manual", "shared"])),
    text(
      "ipv4.gateway",
      "control_center.net_f_gateway",
      TextRule::Address {
        family: Family::V4,
        required: false,
      },
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["manual"])),
    list(
      "ipv4.dns",
      "control_center.net_f_dns_servers",
      ListRule::Address(Family::V4),
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["auto", "manual", "shared"])),
    list(
      "ipv4.dns-search",
      "control_center.net_f_search_domains",
      ListRule::Domain,
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["auto", "manual", "shared"])),
    list(
      "ipv4.routes",
      "control_center.net_f_routes",
      ListRule::Route(Family::V4),
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["auto", "manual"])),
    toggle(
      "ipv4.ignore-auto-dns",
      "control_center.net_f_ignore_auto_dns",
      "no",
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["auto"])),
    toggle(
      "ipv4.ignore-auto-routes",
      "control_center.net_f_ignore_auto_routes",
      "no",
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["auto"])),
    toggle(
      "ipv4.never-default",
      "control_center.net_f_never_default",
      "no",
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["auto", "manual"])),
    toggle("ipv4.may-fail", "control_center.net_f_may_fail", "yes")
      .enabled_when(Condition::NotIn(IPV4_METHOD, &["disabled"])),
    number(
      "ipv4.route-metric",
      "control_center.net_f_route_metric",
      (-1, 4_294_967, 1),
      "-1",
    )
    .enabled_when(Condition::In(IPV4_METHOD, &["auto", "manual"])),
  ],
};

const IPV6: Section = Section {
  id: SectionId::Ipv6,
  title: "control_center.net_section_ipv6",
  properties: &[
    choice(
      IPV6_METHOD,
      "control_center.net_f_method",
      &[
        option("auto", "control_center.net_o_automatic"),
        option("dhcp", "control_center.net_o_dhcp_only"),
        option("manual", "control_center.net_o_manual"),
        option("link-local", "control_center.net_o_link_local"),
        option("shared", "control_center.net_o_shared"),
        option("ignore", "control_center.net_o_ignore"),
        option("disabled", "control_center.disabled"),
      ],
      "auto",
    ),
    list(
      "ipv6.addresses",
      "control_center.net_f_addresses",
      ListRule::Cidr(Family::V6),
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp", "manual"])),
    text(
      "ipv6.gateway",
      "control_center.net_f_gateway",
      TextRule::Address {
        family: Family::V6,
        required: false,
      },
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["manual"])),
    list(
      "ipv6.dns",
      "control_center.net_f_dns_servers",
      ListRule::Address(Family::V6),
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp", "manual"])),
    list(
      "ipv6.dns-search",
      "control_center.net_f_search_domains",
      ListRule::Domain,
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp", "manual"])),
    list(
      "ipv6.routes",
      "control_center.net_f_routes",
      ListRule::Route(Family::V6),
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp", "manual"])),
    toggle(
      "ipv6.ignore-auto-dns",
      "control_center.net_f_ignore_auto_dns",
      "no",
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp"])),
    toggle(
      "ipv6.ignore-auto-routes",
      "control_center.net_f_ignore_auto_routes",
      "no",
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp"])),
    toggle(
      "ipv6.never-default",
      "control_center.net_f_never_default",
      "no",
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp", "manual"])),
    toggle("ipv6.may-fail", "control_center.net_f_may_fail", "yes")
      .enabled_when(Condition::NotIn(IPV6_METHOD, &["ignore", "disabled"])),
    choice(
      "ipv6.ip6-privacy",
      "control_center.net_f_ip6_privacy",
      &[
        option("-1", "control_center.net_o_default"),
        option("0", "control_center.disabled"),
        option("1", "control_center.net_o_privacy_public"),
        option("2", "control_center.net_o_privacy_temporary"),
      ],
      "-1",
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp"])),
    choice(
      "ipv6.addr-gen-mode",
      "control_center.net_f_addr_gen_mode",
      &[
        option("default", "control_center.net_o_default"),
        option("eui64", "control_center.net_o_eui64"),
        option("stable-privacy", "control_center.net_o_stable_privacy"),
        option("default-or-eui64", "control_center.net_o_default_or_eui64"),
      ],
      "default",
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp", "manual"])),
    number(
      "ipv6.route-metric",
      "control_center.net_f_route_metric",
      (-1, 4_294_967, 1),
      "-1",
    )
    .enabled_when(Condition::In(IPV6_METHOD, &["auto", "dhcp", "manual"])),
  ],
};

/// The sections of a profile, in editor order. A port only has its General
/// section; a VPN shows the OpenVPN form or the generic plugin data.
pub fn sections(kind: ConnectionKind, service: Option<&str>, port: bool) -> Vec<&'static Section> {
  if port {
    return vec![&PORT_GENERAL];
  }
  let own: &[&'static Section] = match kind {
    ConnectionKind::Ethernet => &[&ETHERNET],
    ConnectionKind::Wifi | ConnectionKind::Hotspot => &[&WIFI, &WIFI_SECURITY],
    ConnectionKind::Pppoe => &[&PPPOE],
    ConnectionKind::Bridge => &[&BRIDGE],
    ConnectionKind::Bond => &[&BOND],
    ConnectionKind::Team => &[&TEAM],
    ConnectionKind::Vlan => &[&VLAN],
    ConnectionKind::IpTunnel => &[&IP_TUNNEL],
    ConnectionKind::Loopback => &[&LOOPBACK],
    ConnectionKind::WireGuard => &[&WIREGUARD],
    ConnectionKind::Vpn if service == Some(OPENVPN_SERVICE) => &[&OPENVPN],
    ConnectionKind::Vpn => &[&VPN],
    ConnectionKind::Other => &[],
  };
  let mut sections = vec![&GENERAL];
  sections.extend_from_slice(own);
  sections.extend([&IPV4, &IPV6]);
  sections
}

/// Every property of the sections, in editor order.
pub fn properties(sections: &[&'static Section]) -> Vec<&'static Property> {
  sections
    .iter()
    .flat_map(|section| section.properties.iter())
    .collect()
}

/// The initial values of a new profile of `kind`, beyond NetworkManager's
/// defaults: what makes a Hotspot a hotspot, the addresses a loopback needs,
/// the most common choice of each mode.
pub fn presets(kind: ConnectionKind) -> &'static [(&'static str, &'static str)] {
  match kind {
    ConnectionKind::Wifi => &[("802-1x.eap", "peap"), ("802-1x.phase2-auth", "mschapv2")],
    ConnectionKind::Hotspot => &[
      ("802-11-wireless.mode", "ap"),
      ("802-11-wireless.band", "bg"),
      (KEY_MGMT, "wpa-psk"),
      (IPV4_METHOD, "shared"),
      (IPV6_METHOD, "ignore"),
    ],
    ConnectionKind::Team => &[("team.runner", "roundrobin")],
    ConnectionKind::Bond => &[("bond.options.miimon", "100")],
    ConnectionKind::IpTunnel => &[(TUNNEL_MODE, "gre")],
    ConnectionKind::Loopback => &[
      ("connection.interface-name", "lo"),
      (IPV4_METHOD, "manual"),
      ("ipv4.addresses", "127.0.0.1/8"),
      (IPV6_METHOD, "manual"),
      ("ipv6.addresses", "::1/128"),
    ],
    ConnectionKind::WireGuard => &[(IPV4_METHOD, "manual"), (IPV6_METHOD, "disabled")],
    ConnectionKind::Vpn => &[(OPENVPN_TYPE, "tls")],
    _ => &[],
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  #[test]
  fn keys_are_unique_within_each_kind() {
    for kind in ConnectionKind::ADDABLE {
      for service in [None, Some(OPENVPN_SERVICE)] {
        let mut seen = HashSet::new();
        for property in properties(&sections(kind, service, false)) {
          assert!(seen.insert(property.key), "{kind:?}: {}", property.key);
        }
      }
    }
  }

  #[test]
  fn choice_defaults_and_presets_are_real_options() {
    for kind in ConnectionKind::ADDABLE {
      let all = properties(&sections(kind, Some(OPENVPN_SERVICE), false));
      for property in &all {
        if let Widget::Choice(options) = property.widget
          && !property.default.is_empty()
        {
          assert!(
            options
              .iter()
              .any(|choice| choice.value == property.default),
            "{}",
            property.key
          );
        }
      }
      for (key, value) in presets(kind) {
        let property = all
          .iter()
          .find(|property| property.key == *key)
          .unwrap_or_else(|| panic!("{kind:?} preset {key} has no field"));
        if let Widget::Choice(options) = property.widget {
          assert!(options.iter().any(|choice| choice.value == *value), "{key}");
        }
      }
    }
  }

  #[test]
  fn conditions_name_fields_of_the_same_kind() {
    for kind in ConnectionKind::ADDABLE {
      let all = properties(&sections(kind, Some(OPENVPN_SERVICE), false))
        .into_iter()
        .map(|property| property.key)
        .collect::<HashSet<_>>();
      for property in properties(&sections(kind, Some(OPENVPN_SERVICE), false)) {
        for condition in [property.shown, property.enabled] {
          if let Condition::In(key, _) | Condition::NotIn(key, _) = condition {
            assert!(all.contains(key), "{}: {key}", property.key);
          }
        }
      }
    }
  }

  #[test]
  fn ports_only_have_general_settings() {
    let port = sections(ConnectionKind::Ethernet, None, true);
    assert_eq!(port.len(), 1);
    assert!(
      properties(&port)
        .iter()
        .all(|property| !property.key.starts_with("ipv"))
    );
  }
}
