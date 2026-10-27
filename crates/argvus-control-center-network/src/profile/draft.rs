//! The connection being edited: its values as loaded, and as changed by the
//! user until Save.

use super::{
  ConnectionKind,
  schema::{self, ALL_USERS, Condition, Property, SecretRule, Section, TextRule, Widget},
  validate::{self, FieldError},
};
use std::collections::{BTreeMap, BTreeSet};

/// One WireGuard peer. The preshared key is a secret: `Debug` hides it.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct WireGuardPeer {
  pub public_key: String,
  pub endpoint: String,
  /// Comma-separated `address/prefix` entries.
  pub allowed_ips: String,
  pub preshared_key: String,
  /// Seconds, or empty for none.
  pub keepalive: String,
}

impl std::fmt::Debug for WireGuardPeer {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("WireGuardPeer")
      .field("public_key", &self.public_key)
      .field("endpoint", &self.endpoint)
      .field("allowed_ips", &self.allowed_ips)
      .field("preshared_key", &(!self.preshared_key.is_empty()))
      .field("keepalive", &self.keepalive)
      .finish()
  }
}

/// A field of a WireGuard peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeerField {
  PublicKey,
  Endpoint,
  AllowedIps,
  PresharedKey,
  Keepalive,
}

impl PeerField {
  pub const ALL: [Self; 5] = [
    Self::PublicKey,
    Self::Endpoint,
    Self::AllowedIps,
    Self::PresharedKey,
    Self::Keepalive,
  ];

  pub fn label_key(self) -> &'static str {
    match self {
      Self::PublicKey => "control_center.net_f_public_key",
      Self::Endpoint => "control_center.net_f_endpoint",
      Self::AllowedIps => "control_center.net_f_allowed_ips",
      Self::PresharedKey => "control_center.net_f_preshared_key",
      Self::Keepalive => "control_center.net_f_keepalive",
    }
  }

  /// How the field is edited and checked.
  pub fn widget(self) -> Widget {
    match self {
      Self::PublicKey => Widget::Secret(SecretRule::WireGuardKey),
      Self::Endpoint => Widget::Text(TextRule::Endpoint { required: false }),
      Self::AllowedIps => Widget::List(schema::ListRule::Cidr(schema::Family::Any)),
      Self::PresharedKey => Widget::Secret(SecretRule::WireGuardKey),
      Self::Keepalive => Widget::Text(TextRule::OptionalInteger { min: 0, max: 65535 }),
    }
  }

  pub fn is_secret(self) -> bool {
    self == Self::PresharedKey
  }
}

impl WireGuardPeer {
  pub fn get(&self, field: PeerField) -> &str {
    match field {
      PeerField::PublicKey => &self.public_key,
      PeerField::Endpoint => &self.endpoint,
      PeerField::AllowedIps => &self.allowed_ips,
      PeerField::PresharedKey => &self.preshared_key,
      PeerField::Keepalive => &self.keepalive,
    }
  }

  pub fn set(&mut self, field: PeerField, value: String) {
    let slot = match field {
      PeerField::PublicKey => &mut self.public_key,
      PeerField::Endpoint => &mut self.endpoint,
      PeerField::AllowedIps => &mut self.allowed_ips,
      PeerField::PresharedKey => &mut self.preshared_key,
      PeerField::Keepalive => &mut self.keepalive,
    };
    *slot = value.trim().to_owned();
  }

  /// The first field that does not validate. The public key is required; a
  /// peer without it would be dropped by NetworkManager.
  pub fn first_error(&self) -> Option<(PeerField, FieldError)> {
    if self.public_key.is_empty() {
      return Some((PeerField::PublicKey, FieldError::Required));
    }
    PeerField::ALL.into_iter().find_map(|field| {
      let value = self.get(field);
      if value.is_empty() {
        return None;
      }
      validate::check(field.widget(), value)
        .err()
        .map(|error| (field, error))
    })
  }
}

/// What is known about a secret field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretState {
  /// Typed in this editor; written on Save.
  Changed,
  /// Kept as NetworkManager has it (secrets are never read back).
  Unchanged,
}

/// A connection profile being created or edited.
#[derive(Clone)]
pub struct ConnectionDraft {
  pub kind: ConnectionKind,
  /// `None` until NetworkManager has the profile.
  pub uuid: Option<String>,
  /// `vpn.service-type` of a VPN profile.
  pub service_type: Option<String>,
  /// `connection.port-type` when the profile is a port of a controller.
  pub port_type: Option<String>,
  /// The user that owns the profile when it is not for all users.
  pub owner: String,
  sections: Vec<&'static Section>,
  values: BTreeMap<&'static str, String>,
  original: BTreeMap<&'static str, String>,
  secrets: BTreeMap<&'static str, String>,
  /// Entries of `bond.options` and `vpn.data` the editor has no field for,
  /// kept as they are when the dictionary is written back.
  pub(crate) extras: BTreeMap<&'static str, Vec<(String, String)>>,
  /// Devices that are ports of this controller (bridge, bond, team).
  pub ports: BTreeSet<String>,
  original_ports: BTreeSet<String>,
  pub peers: Vec<WireGuardPeer>,
  original_peers: Vec<WireGuardPeer>,
}

impl std::fmt::Debug for ConnectionDraft {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("ConnectionDraft")
      .field("kind", &self.kind)
      .field("uuid", &self.uuid)
      .field("values", &self.values)
      .field("secrets", &self.secrets.keys().collect::<Vec<_>>())
      .field("ports", &self.ports)
      .field("peers", &self.peers)
      .finish_non_exhaustive()
  }
}

impl ConnectionDraft {
  /// A new profile of `kind` named `name`, with NetworkManager's defaults
  /// and the kind's presets.
  pub fn new(kind: ConnectionKind, name: &str, service_type: Option<String>, owner: &str) -> Self {
    let sections = schema::sections(kind, service_type.as_deref(), false);
    let mut values = schema::properties(&sections)
      .into_iter()
      .map(|property| (property.key, property.default.to_owned()))
      .collect::<BTreeMap<_, _>>();
    values.insert("connection.id", name.to_owned());
    for (key, value) in schema::presets(kind) {
      values.insert(key, (*value).to_owned());
    }
    Self {
      kind,
      uuid: None,
      service_type,
      port_type: None,
      owner: owner.to_owned(),
      sections,
      original: BTreeMap::new(),
      values,
      secrets: BTreeMap::new(),
      extras: BTreeMap::new(),
      ports: BTreeSet::new(),
      original_ports: BTreeSet::new(),
      peers: Vec::new(),
      original_peers: Vec::new(),
    }
  }

  /// A profile NetworkManager already has, from its values (keys of the
  /// sections; missing ones take the default). The loaded state is the
  /// baseline of [`Self::is_dirty`].
  pub(crate) fn loaded(
    kind: ConnectionKind,
    uuid: String,
    service_type: Option<String>,
    port_type: Option<String>,
    owner: &str,
    mut lookup: impl FnMut(&'static Property) -> Option<String>,
  ) -> Self {
    let sections = schema::sections(kind, service_type.as_deref(), port_type.is_some());
    let values = schema::properties(&sections)
      .into_iter()
      .map(|property| {
        let value = lookup(property).unwrap_or_else(|| property.default.to_owned());
        (property.key, value)
      })
      .collect::<BTreeMap<_, _>>();
    Self {
      kind,
      uuid: Some(uuid),
      service_type,
      port_type,
      owner: owner.to_owned(),
      sections,
      original: values.clone(),
      values,
      secrets: BTreeMap::new(),
      extras: BTreeMap::new(),
      ports: BTreeSet::new(),
      original_ports: BTreeSet::new(),
      peers: Vec::new(),
      original_peers: Vec::new(),
    }
  }

  /// Sets the ports and peers loaded with the profile, as their baseline.
  pub(crate) fn set_loaded_members(&mut self, ports: BTreeSet<String>, peers: Vec<WireGuardPeer>) {
    self.original_ports = ports.clone();
    self.ports = ports;
    self.original_peers = peers.clone();
    self.peers = peers;
  }

  pub fn is_new(&self) -> bool {
    self.uuid.is_none()
  }

  pub fn is_port(&self) -> bool {
    self.port_type.is_some()
  }

  pub fn sections(&self) -> &[&'static Section] {
    &self.sections
  }

  /// Every property of the profile, in editor order.
  pub fn properties(&self) -> Vec<&'static Property> {
    schema::properties(&self.sections)
  }

  pub fn property(&self, key: &str) -> Option<&'static Property> {
    self
      .properties()
      .into_iter()
      .find(|property| property.key == key)
  }

  /// The current value of a (non-secret) field.
  pub fn value(&self, key: &str) -> &str {
    self.values.get(key).map_or("", String::as_str)
  }

  /// The value of a field as loaded, or `None` for a new profile.
  pub(crate) fn original_value(&self, key: &str) -> Option<&str> {
    self.original.get(key).map(String::as_str)
  }

  pub fn set(&mut self, key: &'static str, value: String) {
    let value = value.trim().to_owned();
    self.values.insert(key, value);
  }

  /// Switches a toggle between `yes` and `no`.
  pub fn toggle(&mut self, key: &'static str) {
    let next = if self.value(key) == "yes" {
      "no"
    } else {
      "yes"
    };
    self.values.insert(key, next.to_owned());
  }

  /// Moves a choice to the next (`step > 0`) or previous option, or a
  /// number by `step` times its own step, within its range.
  pub fn adjust(&mut self, key: &'static str, step: i32) {
    let Some(property) = self.property(key) else {
      return;
    };
    match property.widget {
      Widget::Choice(options) => {
        let current = options
          .iter()
          .position(|choice| choice.value == self.value(key))
          .unwrap_or(0);
        let next = cycled(current, options.len(), step);
        self.values.insert(key, options[next].value.to_owned());
      }
      Widget::Number {
        min,
        max,
        step: size,
      } => {
        let current = self.value(key).parse::<i64>().unwrap_or(min);
        let next = (current + i64::from(step) * size).clamp(min, max);
        self.values.insert(key, next.to_string());
      }
      _ => {}
    }
  }

  /// Moves a device field to the next or previous of `devices` (with an
  /// empty name, "any device", first when the field is optional).
  pub fn adjust_device(&mut self, key: &'static str, devices: &[String], step: i32) {
    let Some(Widget::Device { required }) = self.property(key).map(|property| property.widget)
    else {
      return;
    };
    let mut options = Vec::with_capacity(devices.len() + 1);
    if !required {
      options.push(String::new());
    }
    options.extend(devices.iter().cloned());
    if options.is_empty() {
      return;
    }
    let current = options
      .iter()
      .position(|device| device == self.value(key))
      .unwrap_or(0);
    let next = cycled(current, options.len(), step);
    self.values.insert(key, options[next].clone());
  }

  /// Records a secret typed by the user.
  pub fn set_secret(&mut self, key: &'static str, value: String) {
    self.secrets.insert(key, value);
  }

  pub fn secret(&self, key: &str) -> Option<&str> {
    self.secrets.get(key).map(String::as_str)
  }

  pub fn secret_state(&self, key: &str) -> SecretState {
    if self.secrets.contains_key(key) {
      SecretState::Changed
    } else {
      SecretState::Unchanged
    }
  }

  /// The secrets typed in this editor, by key.
  pub(crate) fn changed_secrets(&self) -> impl Iterator<Item = (&'static str, &str)> {
    self
      .secrets
      .iter()
      .map(|(key, value)| (*key, value.as_str()))
  }

  /// Whether the condition holds for the current values.
  pub fn holds(&self, condition: Condition) -> bool {
    match condition {
      Condition::Always => true,
      Condition::In(key, values) => values.contains(&self.value(key)),
      Condition::NotIn(key, values) => !values.contains(&self.value(key)),
    }
  }

  /// Whether a field is part of the profile with the current choices.
  pub fn is_shown(&self, property: &Property) -> bool {
    self.holds(property.shown)
  }

  /// Whether a shown field applies with the current choices.
  pub fn is_enabled(&self, property: &Property) -> bool {
    self.holds(property.enabled)
  }

  /// Whether a field changed since the profile was loaded.
  pub fn is_changed(&self, key: &str) -> bool {
    self.original.get(key).map(String::as_str) != Some(self.value(key))
  }

  /// Whether the profile has anything to save.
  pub fn is_dirty(&self) -> bool {
    self.is_new()
      || self.values != self.original
      || !self.secrets.is_empty()
      || self.ports != self.original_ports
      || self.peers != self.original_peers
  }

  pub fn ports_changed(&self) -> bool {
    self.ports != self.original_ports
  }

  pub fn peers_changed(&self) -> bool {
    self.peers != self.original_peers
  }

  /// Ports joined since the profile was loaded, and ports dropped.
  pub fn port_changes(&self) -> (Vec<String>, Vec<String>) {
    let added = self
      .ports
      .difference(&self.original_ports)
      .cloned()
      .collect();
    let removed = self
      .original_ports
      .difference(&self.ports)
      .cloned()
      .collect();
    (added, removed)
  }

  /// The first shown field that does not validate, or the first invalid peer.
  pub fn first_error(&self) -> Option<DraftError> {
    for property in self.properties() {
      if !self.is_shown(property) || !self.is_enabled(property) {
        continue;
      }
      let result = match property.widget {
        Widget::Secret(_) => self
          .secret(property.key)
          .filter(|secret| !secret.is_empty())
          .map_or(Ok(()), |secret| validate::check(property.widget, secret)),
        widget => validate::check(widget, self.value(property.key)),
      };
      if let Err(error) = result {
        return Some(DraftError::Field(property.key, error));
      }
    }
    if let Some(key) = self.missing_manual_address() {
      return Some(DraftError::Field(key, FieldError::AddressRequired));
    }
    if let Some(key) = self.missing_hotspot_password() {
      return Some(DraftError::Field(key, FieldError::Required));
    }
    self.peers.iter().enumerate().find_map(|(index, peer)| {
      peer
        .first_error()
        .map(|(field, error)| DraftError::Peer(index, field, error))
    })
  }

  /// The password field of a new protected hotspot without a password:
  /// NetworkManager has nobody to ask for the secret of an access point it
  /// runs itself, so the hotspot would never start.
  fn missing_hotspot_password(&self) -> Option<&'static str> {
    const PSK: &str = "802-11-wireless-security.psk";
    let protected = matches!(
      self.value("802-11-wireless-security.key-mgmt"),
      "wpa-psk" | "sae"
    );
    let typed = self.secret(PSK).is_some_and(|secret| !secret.is_empty());
    (self.kind == ConnectionKind::Hotspot && self.is_new() && protected && !typed).then_some(PSK)
  }

  /// The address list of a family whose method is manual but which has no
  /// address. Ports have no IP settings, so they never get here.
  fn missing_manual_address(&self) -> Option<&'static str> {
    [
      ("ipv4.method", "ipv4.addresses"),
      ("ipv6.method", "ipv6.addresses"),
    ]
    .into_iter()
    .find(|(method, addresses)| {
      self.values.contains_key(method)
        && self.value(method) == "manual"
        && validate::split_list(self.value(addresses)).is_empty()
    })
    .map(|(_, addresses)| addresses)
  }

  /// Whether the profile is for every user.
  pub fn for_all_users(&self) -> bool {
    self.value(ALL_USERS) != "no"
  }
}

/// What keeps a draft from being saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftError {
  Field(&'static str, FieldError),
  Peer(usize, PeerField, FieldError),
}

impl DraftError {
  pub fn error(self) -> FieldError {
    match self {
      Self::Field(_, error) | Self::Peer(_, _, error) => error,
    }
  }
}

fn cycled(current: usize, count: usize, step: i32) -> usize {
  let count = i64::try_from(count).unwrap_or(1).max(1);
  let next = (i64::try_from(current).unwrap_or(0) + i64::from(step)).rem_euclid(count);
  usize::try_from(next).unwrap_or(0)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn wifi() -> ConnectionDraft {
    ConnectionDraft::new(ConnectionKind::Wifi, "Home", None, "boss")
  }

  #[test]
  fn new_profiles_start_from_defaults_and_presets() {
    let hotspot = ConnectionDraft::new(ConnectionKind::Hotspot, "Hotspot", None, "boss");
    assert_eq!(hotspot.value("802-11-wireless.mode"), "ap");
    assert_eq!(hotspot.value("ipv4.method"), "shared");
    assert_eq!(hotspot.value("connection.id"), "Hotspot");
    assert_eq!(hotspot.value("connection.autoconnect"), "yes");
    assert!(hotspot.is_new() && hotspot.is_dirty());
  }

  #[test]
  fn choices_cycle_both_ways_and_numbers_stay_in_range() {
    let mut draft = wifi();
    draft.adjust("802-11-wireless-security.key-mgmt", 1);
    assert_eq!(draft.value("802-11-wireless-security.key-mgmt"), "wpa-psk");
    draft.adjust("802-11-wireless-security.key-mgmt", -2);
    assert_eq!(draft.value("802-11-wireless-security.key-mgmt"), "wpa-eap");
    let mut vlan = ConnectionDraft::new(ConnectionKind::Vlan, "VLAN", None, "boss");
    vlan.adjust("vlan.id", -1);
    assert_eq!(vlan.value("vlan.id"), "0");
    vlan.set("vlan.id", "4094".into());
    vlan.adjust("vlan.id", 1);
    assert_eq!(vlan.value("vlan.id"), "4094");
  }

  #[test]
  fn devices_cycle_with_any_first_when_optional() {
    let mut draft = ConnectionDraft::new(ConnectionKind::Ethernet, "Wired", None, "boss");
    let devices = vec!["eno1".to_owned(), "eno2".to_owned()];
    draft.adjust_device("connection.interface-name", &devices, 1);
    assert_eq!(draft.value("connection.interface-name"), "eno1");
    draft.adjust_device("connection.interface-name", &devices, 2);
    assert_eq!(draft.value("connection.interface-name"), "");
  }

  #[test]
  fn shown_fields_follow_the_security_choice() {
    let mut draft = wifi();
    let psk = draft.property("802-11-wireless-security.psk").unwrap();
    assert!(!draft.is_shown(psk));
    draft.set("802-11-wireless-security.key-mgmt", "sae".into());
    assert!(draft.is_shown(psk));
  }

  #[test]
  fn errors_point_at_the_first_invalid_field() {
    let mut draft = wifi();
    assert_eq!(
      draft.first_error(),
      Some(DraftError::Field(
        "802-11-wireless.ssid",
        FieldError::Required
      ))
    );
    draft.set("802-11-wireless.ssid", "Home".into());
    draft.set("802-11-wireless-security.key-mgmt", "wpa-psk".into());
    draft.set_secret("802-11-wireless-security.psk", "short".into());
    assert_eq!(
      draft.first_error(),
      Some(DraftError::Field(
        "802-11-wireless-security.psk",
        FieldError::WpaPassphrase
      ))
    );
    draft.set_secret("802-11-wireless-security.psk", "long enough".into());
    draft.set("ipv4.method", "manual".into());
    assert_eq!(
      draft.first_error(),
      Some(DraftError::Field(
        "ipv4.addresses",
        FieldError::AddressRequired
      ))
    );
    draft.set("ipv4.addresses", "192.168.0.10/24".into());
    assert_eq!(draft.first_error(), None);
  }

  #[test]
  fn a_new_protected_hotspot_needs_its_password() {
    let mut hotspot = ConnectionDraft::new(ConnectionKind::Hotspot, "Hotspot", None, "boss");
    hotspot.set("802-11-wireless.ssid", "argvus".into());
    assert_eq!(
      hotspot.first_error(),
      Some(DraftError::Field(
        "802-11-wireless-security.psk",
        FieldError::Required
      ))
    );
    hotspot.set_secret("802-11-wireless-security.psk", "hotspot-pass".into());
    assert_eq!(hotspot.first_error(), None);
  }

  #[test]
  fn disabled_fields_are_not_validated() {
    let mut draft = ConnectionDraft::new(ConnectionKind::Ethernet, "Wired", None, "boss");
    draft.set("ipv4.gateway", "not an address".into());
    assert_eq!(draft.first_error(), None, "gateway is off with DHCP");
    draft.set("ipv4.method", "manual".into());
    draft.set("ipv4.addresses", "10.0.0.2/24".into());
    assert_eq!(
      draft.first_error(),
      Some(DraftError::Field("ipv4.gateway", FieldError::Address))
    );
  }

  #[test]
  fn loaded_profiles_are_clean_until_changed() {
    let mut draft = ConnectionDraft::loaded(
      ConnectionKind::Ethernet,
      "uuid".into(),
      None,
      None,
      "boss",
      |property| (property.key == "connection.id").then(|| "Wired".to_owned()),
    );
    assert!(!draft.is_dirty());
    draft.toggle("connection.autoconnect");
    assert!(draft.is_dirty() && draft.is_changed("connection.autoconnect"));
    draft.toggle("connection.autoconnect");
    assert!(!draft.is_dirty());
    draft.set_secret("802-1x.password", String::new());
    assert!(draft.is_dirty());
  }

  #[test]
  fn peers_validate_and_hide_their_preshared_key() {
    let peer = WireGuardPeer {
      public_key: "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=".into(),
      endpoint: "vpn.example.org:51820".into(),
      allowed_ips: "0.0.0.0/0, ::/0".into(),
      preshared_key: "yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=".into(),
      keepalive: "25".into(),
    };
    assert_eq!(peer.first_error(), None);
    assert!(!format!("{peer:?}").contains("yAnz5"));
    let missing = WireGuardPeer::default();
    assert_eq!(
      missing.first_error(),
      Some((PeerField::PublicKey, FieldError::Required))
    );
  }
}
