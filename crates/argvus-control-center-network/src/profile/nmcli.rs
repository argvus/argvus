//! Translation between a [`ConnectionDraft`] and `nmcli`: the settings it
//! prints, the arguments that create or change a profile, and the commands
//! that set its secrets.
//!
//! Secrets never become arguments, which every local user can read in
//! `/proc`. They go through the standard input of
//! `nmcli connection edit`, as `set <property> <value>` lines followed by
//! `save persistent`.

use super::{
  ConnectionDraft, ConnectionKind, WireGuardPeer,
  schema::{
    ALL_USERS, BOND_OPTION_PREFIX, Property, TextRule, VPN_DATA_PREFIX, VPN_SECRET_PREFIX, Widget,
  },
  validate::split_list,
};
use std::collections::{BTreeMap, BTreeSet};

/// The settings of one profile, as `nmcli -t connection show uuid <uuid>`
/// prints them (`key:value` per line, `--` for unset).
pub fn parse_settings(output: &str) -> BTreeMap<String, String> {
  output
    .lines()
    .filter_map(|line| line.split_once(':'))
    .map(|(key, value)| {
      let value = value.trim();
      let value = if value == "--" { "" } else { value };
      (key.trim().to_owned(), value.to_owned())
    })
    .collect()
}

/// Removes the escaping `nmcli -g` adds to `:` and `\`.
pub fn unescape(value: &str) -> String {
  let mut out = String::with_capacity(value.len());
  let mut characters = value.chars();
  while let Some(character) = characters.next() {
    if character == '\\'
      && let Some(next) = characters.next()
    {
      out.push(next);
    } else {
      out.push(character);
    }
  }
  out
}

/// A `key=value` dictionary (`bond.options`, `vpn.data`), in the order
/// `nmcli` prints it. Both `k=v,k2=v2` and `k = v, k2 = v2` are read.
pub fn parse_dictionary(value: &str) -> Vec<(String, String)> {
  value
    .split(',')
    .filter_map(|entry| entry.split_once('='))
    .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
    .filter(|(key, _)| !key.is_empty())
    .collect()
}

fn format_dictionary<'a>(entries: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
  entries
    .into_iter()
    .filter(|(_, value)| !value.is_empty())
    .map(|(key, value)| format!("{key}={value}"))
    .collect::<Vec<_>>()
    .join(",")
}

/// The value of a choice as stored: `nmcli` may print a number with its
/// name (`-1 (unknown)`, `2 (gre)`), so either part that names an option
/// is taken.
fn normalize_choice(raw: &str, property: &Property) -> String {
  let Widget::Choice(options) = property.widget else {
    return raw.to_owned();
  };
  let is_option = |candidate: &str| options.iter().any(|choice| choice.value == candidate);
  if is_option(raw) {
    return raw.to_owned();
  }
  let named = raw
    .split_once('(')
    .and_then(|(_, rest)| rest.strip_suffix(')'))
    .map(str::trim);
  let first = raw.split_whitespace().next();
  named
    .into_iter()
    .chain(first)
    .find(|candidate| is_option(candidate))
    .unwrap_or(raw)
    .to_owned()
}

/// The value of a field as the editor keeps it, from what `nmcli` printed.
fn normalize(raw: &str, property: &Property) -> String {
  match property.widget {
    Widget::Choice(_) => normalize_choice(raw, property),
    Widget::Text(TextRule::Mtu) if raw.is_empty() || raw == "0" => "auto".to_owned(),
    Widget::List(_) => split_list(raw).join(", "),
    _ => raw.to_owned(),
  }
}

/// The editor's view of a profile `nmcli` printed (see [`parse_settings`]).
/// `owner` is the user a private profile is limited to.
pub fn draft_from_settings(settings: &BTreeMap<String, String>, owner: &str) -> ConnectionDraft {
  let get = |key: &str| settings.get(key).map(String::as_str).unwrap_or_default();
  let non_empty = |key: &str| Some(get(key)).filter(|value| !value.is_empty());
  let wifi_mode = non_empty("802-11-wireless.mode");
  let kind = ConnectionKind::from_nmcli(get("connection.type"), wifi_mode);
  let service_type = non_empty("vpn.service-type").map(str::to_owned);
  let port_type = non_empty("connection.port-type")
    .or_else(|| non_empty("connection.slave-type"))
    .map(str::to_owned);
  let bond = parse_dictionary(get("bond.options"));
  let vpn_data = parse_dictionary(get("vpn.data"));
  let lookup_entry = |entries: &[(String, String)], name: &str| {
    entries
      .iter()
      .find(|(key, _)| key == name)
      .map(|(_, value)| value.clone())
  };
  let mut draft = ConnectionDraft::loaded(
    kind,
    get("connection.uuid").to_owned(),
    service_type,
    port_type,
    owner,
    |property| {
      if property.is_secret() {
        return None;
      }
      if property.key == ALL_USERS {
        let everyone = get("connection.permissions").is_empty();
        return Some(if everyone { "yes" } else { "no" }.to_owned());
      }
      if let Some(option) = property.key.strip_prefix(BOND_OPTION_PREFIX) {
        return lookup_entry(&bond, option).map(|value| normalize(&value, property));
      }
      if let Some(entry) = property.key.strip_prefix(VPN_DATA_PREFIX) {
        return lookup_entry(&vpn_data, entry).map(|value| normalize(&value, property));
      }
      if property.key == "vpn.data" {
        let entries = format_dictionary(
          vpn_data
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
        );
        return Some(normalize(&entries, property));
      }
      settings
        .get(property.key)
        .map(|raw| normalize(raw, property))
    },
  );
  // Dictionary entries without a field are kept as they are.
  let known = draft
    .properties()
    .into_iter()
    .map(|property| property.key)
    .collect::<BTreeSet<_>>();
  let unknown = |prefix: &str, entries: Vec<(String, String)>| {
    entries
      .into_iter()
      .filter(|(key, _)| !known.contains(format!("{prefix}{key}").as_str()))
      .collect::<Vec<_>>()
  };
  draft
    .extras
    .insert("bond.options", unknown(BOND_OPTION_PREFIX, bond));
  if !known.contains("vpn.data") {
    draft
      .extras
      .insert("vpn.data", unknown(VPN_DATA_PREFIX, vpn_data));
  }
  draft
}

/// The peers of `wireguard.peers`, as `nmcli -s -g wireguard.peers` prints
/// them (escaped; see [`unescape`]).
pub fn parse_peers(value: &str) -> Vec<WireGuardPeer> {
  unescape(value)
    .split(',')
    .filter_map(|entry| {
      let mut tokens = entry.split_whitespace();
      let public_key = tokens.next()?.to_owned();
      let mut peer = WireGuardPeer {
        public_key,
        ..Default::default()
      };
      for token in tokens {
        let Some((name, value)) = token.split_once('=') else {
          continue;
        };
        match name {
          "endpoint" => peer.endpoint = value.to_owned(),
          "allowed-ips" => peer.allowed_ips = split_allowed_ips(value).join(", "),
          "preshared-key" => peer.preshared_key = value.to_owned(),
          "persistent-keepalive" => peer.keepalive = value.to_owned(),
          _ => {}
        }
      }
      Some(peer)
    })
    .collect()
}

fn split_allowed_ips(value: &str) -> Vec<&str> {
  value
    .split([';', ','])
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .collect()
}

/// `wireguard.peers` for `nmcli`: peers separated by `, `, attributes by
/// spaces, allowed IPs by `;`.
pub fn format_peers(peers: &[WireGuardPeer]) -> String {
  peers
    .iter()
    .map(|peer| {
      let mut parts = vec![peer.public_key.clone()];
      if !peer.endpoint.is_empty() {
        parts.push(format!("endpoint={}", peer.endpoint));
      }
      let allowed = split_allowed_ips(&peer.allowed_ips);
      if !allowed.is_empty() {
        parts.push(format!("allowed-ips={}", allowed.join(";")));
      }
      if !peer.keepalive.is_empty() && peer.keepalive != "0" {
        parts.push(format!("persistent-keepalive={}", peer.keepalive));
      }
      if !peer.preshared_key.is_empty() {
        parts.push(format!("preshared-key={}", peer.preshared_key));
      }
      parts.join(" ")
    })
    .collect::<Vec<_>>()
    .join(", ")
}

/// The `nmcli` property and value of a field, or `None` for fields written
/// another way (secrets, dictionary entries).
fn property_argument(draft: &ConnectionDraft, property: &Property) -> Option<(String, String)> {
  if property.is_secret()
    || property.key.starts_with(BOND_OPTION_PREFIX)
    || property.key.starts_with(VPN_DATA_PREFIX)
  {
    return None;
  }
  let value = draft.value(property.key);
  if property.key == ALL_USERS {
    let permissions = if value == "no" {
      format!("user:{}", draft.owner)
    } else {
      String::new()
    };
    return Some(("connection.permissions".to_owned(), permissions));
  }
  if property.key == "vpn.data" {
    let entries = split_list(value);
    let pairs = entries
      .iter()
      .filter_map(|entry| entry.split_once('='))
      .map(|(key, value)| (key.trim(), value.trim()));
    return Some((property.key.to_owned(), format_dictionary(pairs)));
  }
  let value = match property.widget {
    Widget::List(_) => split_list(value).join(","),
    _ => value.to_owned(),
  };
  Some((property.key.to_owned(), value))
}

/// The shown fields of the profile with their current value.
fn shown_properties(draft: &ConnectionDraft) -> impl Iterator<Item = &'static Property> + '_ {
  draft
    .properties()
    .into_iter()
    .filter(|property| draft.is_shown(property))
}

/// A dictionary written back whole when any of its fields changed: the
/// shown fields plus the entries the editor has no field for.
fn dictionary_argument(
  draft: &ConnectionDraft,
  name: &'static str,
  prefix: &str,
  only_if_changed: bool,
) -> Option<(String, String)> {
  let fields = shown_properties(draft)
    .filter(|property| property.key.starts_with(prefix))
    .collect::<Vec<_>>();
  if fields.is_empty() {
    return None;
  }
  let changed = fields.iter().any(|property| draft.is_changed(property.key));
  if only_if_changed && !changed {
    return None;
  }
  let extras = draft.extras.get(name).cloned().unwrap_or_default();
  let entries = fields
    .iter()
    .map(|property| {
      (
        property.key.trim_start_matches(prefix),
        draft.value(property.key),
      )
    })
    .chain(
      extras
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str())),
    )
    .collect::<Vec<_>>();
  Some((name.to_owned(), format_dictionary(entries)))
}

/// Whether the Wi-Fi security setting exists with the current choice.
fn has_wifi_security(draft: &ConnectionDraft) -> bool {
  draft.values_contain("802-11-wireless-security.key-mgmt")
    && !draft.value("802-11-wireless-security.key-mgmt").is_empty()
}

/// Whether a property belongs to a setting the current choices remove.
fn in_removed_setting(draft: &ConnectionDraft, key: &str) -> bool {
  let open = draft.values_contain("802-11-wireless-security.key-mgmt") && !has_wifi_security(draft);
  (open && (key.starts_with("802-11-wireless-security.") || key.starts_with("802-1x.")))
    || (key.starts_with("802-1x.") && draft.value("802-11-wireless-security.key-mgmt") != "wpa-eap")
}

/// Fields `nmcli connection add` demands even when they hold the default
/// (a VLAN without `vlan.id` is refused, although 0 is valid).
const REQUIRED_ON_ADD: &[&str] = &["vlan.id"];

/// Arguments of `nmcli` that create the profile. Fields equal to
/// NetworkManager's default are left out, so the new profile only records
/// what the user chose. Secrets, ports and peers follow in other commands.
pub fn add_arguments(draft: &ConnectionDraft) -> Vec<String> {
  let mut args = vec![
    "connection".to_owned(),
    "add".to_owned(),
    "type".to_owned(),
    draft.kind.nmcli_type().to_owned(),
  ];
  if let Some(service) = &draft.service_type {
    args.extend(["vpn.service-type".to_owned(), service.clone()]);
  }
  if let Some(port_type) = &draft.port_type {
    args.extend(["port-type".to_owned(), port_type.clone()]);
  }
  for property in shown_properties(draft) {
    let is_default =
      draft.value(property.key) == property.default && !REQUIRED_ON_ADD.contains(&property.key);
    if in_removed_setting(draft, property.key) || is_default {
      continue;
    }
    if let Some(argument) = property_argument(draft, property) {
      args.extend([argument.0, argument.1]);
    }
  }
  for (name, prefix) in [
    ("bond.options", BOND_OPTION_PREFIX),
    ("vpn.data", VPN_DATA_PREFIX),
  ] {
    if let Some((key, value)) = dictionary_argument(draft, name, prefix, false)
      && !value.is_empty()
    {
      args.extend([key, value]);
    }
  }
  args
}

/// Arguments of `nmcli` that change the profile, with only the fields that
/// changed, or `None` when no field did.
pub fn modify_arguments(draft: &ConnectionDraft) -> Option<Vec<String>> {
  let uuid = draft.uuid.as_ref()?;
  let mut changes = Vec::new();
  for property in shown_properties(draft) {
    if in_removed_setting(draft, property.key) || !draft.is_changed(property.key) {
      continue;
    }
    if let Some(argument) = property_argument(draft, property) {
      changes.extend([argument.0, argument.1]);
    }
  }
  for (name, prefix) in [
    ("bond.options", BOND_OPTION_PREFIX),
    ("vpn.data", VPN_DATA_PREFIX),
  ] {
    if let Some((key, value)) = dictionary_argument(draft, name, prefix, true) {
      changes.extend([key, value]);
    }
  }
  if changes.is_empty() {
    return None;
  }
  let mut args = vec![
    "connection".to_owned(),
    "modify".to_owned(),
    "uuid".to_owned(),
    uuid.clone(),
  ];
  args.extend(changes);
  Some(args)
}

/// Settings the current choices drop from a saved profile: Wi-Fi security
/// for an open network, 802.1X when the network is no longer Enterprise.
pub fn removed_settings(draft: &ConnectionDraft) -> Vec<&'static str> {
  let mut removed = Vec::new();
  let had = |key: &str, values: &[&str]| {
    draft
      .original_value(key)
      .is_some_and(|value| values.contains(&value))
  };
  let key_mgmt = "802-11-wireless-security.key-mgmt";
  if draft.is_new() || !draft.values_contain(key_mgmt) {
    return removed;
  }
  let now = draft.value(key_mgmt);
  if now.is_empty() && !had(key_mgmt, &[""]) {
    removed.push("802-11-wireless-security");
  }
  if now != "wpa-eap" && had(key_mgmt, &["wpa-eap"]) {
    removed.push("802-1x");
  }
  removed
}

/// The standard input of `nmcli connection edit` that writes the secrets
/// typed in the editor and the WireGuard peers (which may carry preshared
/// keys), or `None` when there is nothing to write.
pub fn secret_script(draft: &ConnectionDraft) -> Option<String> {
  let mut lines = Vec::new();
  let mut vpn_secrets = Vec::new();
  for (key, value) in draft.changed_secrets() {
    let Some(property) = draft.property(key) else {
      continue;
    };
    if !draft.is_shown(property) || in_removed_setting(draft, key) {
      continue;
    }
    if let Some(name) = key.strip_prefix(VPN_SECRET_PREFIX) {
      vpn_secrets.push((name, value));
    } else if value.is_empty() {
      lines.push(format!("remove {key}"));
    } else {
      lines.push(format!("set {key} {value}"));
    }
  }
  if !vpn_secrets.is_empty() {
    lines.push(format!(
      "set vpn.secrets {}",
      format_dictionary(vpn_secrets)
    ));
  }
  if draft.kind == ConnectionKind::WireGuard && draft.peers_changed() {
    if draft.peers.is_empty() {
      lines.push("remove wireguard.peers".to_owned());
    } else {
      lines.push(format!(
        "set wireguard.peers {}",
        format_peers(&draft.peers)
      ));
    }
  }
  if lines.is_empty() {
    return None;
  }
  lines.extend(["save persistent".to_owned(), "quit".to_owned()]);
  Some(lines.join("\n") + "\n")
}

/// Arguments of `nmcli` that create the port profile joining `device` to the
/// controller `controller` (its interface name or UUID).
pub fn port_arguments(port_type: &str, controller: &str, device: &str) -> Vec<String> {
  [
    "connection",
    "add",
    "type",
    "ethernet",
    "con-name",
    &format!("{port_type}-port-{device}"),
    "ifname",
    device,
    "port-type",
    port_type,
    "controller",
    controller,
  ]
  .into_iter()
  .map(str::to_owned)
  .collect()
}

impl ConnectionDraft {
  /// Whether the field belongs to the profile's sections.
  pub(crate) fn values_contain(&self, key: &str) -> bool {
    self.property(key).is_some()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// What `nmcli -t connection show` printed for a VLAN profile on a real
  /// host (NetworkManager 1.58), trimmed.
  const VLAN: &str = "connection.id:argvus-test-vlan
connection.uuid:26cd4de0-e33b-4ac6-97e5-f14aa19a6d10
connection.type:vlan
connection.interface-name:
connection.autoconnect:no
connection.autoconnect-priority:0
connection.permissions:
802-3-ethernet.mtu:auto
ipv4.method:manual
ipv4.dns:1.1.1.1,9.9.9.9
ipv4.dns-search:a.example,b.example
ipv4.addresses:10.99.0.2/24, 10.99.1.2/24
ipv4.routes:10.98.0.0/16 10.99.0.1 50, 10.97.0.0/16
ipv6.method:manual
ipv6.addresses:2001:db8::2/64
ipv6.ip6-privacy:-1
ipv6.addr-gen-mode:default
vlan.parent:eno1
vlan.id:999
";

  fn vlan() -> ConnectionDraft {
    draft_from_settings(&parse_settings(VLAN), "boss")
  }

  #[test]
  fn real_output_becomes_a_clean_draft() {
    let draft = vlan();
    assert_eq!(draft.kind, ConnectionKind::Vlan);
    assert_eq!(
      draft.uuid.as_deref(),
      Some("26cd4de0-e33b-4ac6-97e5-f14aa19a6d10")
    );
    assert_eq!(draft.value("vlan.id"), "999");
    assert_eq!(draft.value("ipv4.addresses"), "10.99.0.2/24, 10.99.1.2/24");
    assert_eq!(
      draft.value("ipv4.routes"),
      "10.98.0.0/16 10.99.0.1 50, 10.97.0.0/16"
    );
    assert_eq!(draft.value("ipv6.addresses"), "2001:db8::2/64");
    assert_eq!(draft.value(ALL_USERS), "yes");
    assert!(!draft.is_dirty());
    assert_eq!(draft.first_error(), None);
    assert_eq!(modify_arguments(&draft), None);
    assert_eq!(secret_script(&draft), None);
  }

  #[test]
  fn modify_sends_only_what_changed() {
    let mut draft = vlan();
    draft.set("ipv4.dns", "8.8.8.8, 1.1.1.1".into());
    draft.toggle("connection.autoconnect");
    draft.toggle(ALL_USERS);
    let args = modify_arguments(&draft).unwrap();
    assert_eq!(
      args,
      [
        "connection",
        "modify",
        "uuid",
        "26cd4de0-e33b-4ac6-97e5-f14aa19a6d10",
        "connection.autoconnect",
        "yes",
        "connection.permissions",
        "user:boss",
        "ipv4.dns",
        "8.8.8.8,1.1.1.1",
      ]
    );
  }

  #[test]
  fn add_leaves_defaults_out_and_never_carries_secrets() {
    let mut draft = ConnectionDraft::new(ConnectionKind::Wifi, "Home", None, "boss");
    draft.set("802-11-wireless.ssid", "Home Net".into());
    draft.set("802-11-wireless-security.key-mgmt", "wpa-psk".into());
    draft.set_secret("802-11-wireless-security.psk", "hunter2hunter2".into());
    let args = add_arguments(&draft);
    assert_eq!(
      args,
      [
        "connection",
        "add",
        "type",
        "wifi",
        "connection.id",
        "Home",
        "802-11-wireless.ssid",
        "Home Net",
        "802-11-wireless-security.key-mgmt",
        "wpa-psk",
      ]
    );
    assert!(!args.iter().any(|argument| argument.contains("hunter2")));
    assert_eq!(
      secret_script(&draft).unwrap(),
      "set 802-11-wireless-security.psk hunter2hunter2\nsave persistent\nquit\n"
    );
  }

  #[test]
  fn open_networks_drop_security_and_enterprise_fields_stay_out() {
    let mut draft = ConnectionDraft::new(ConnectionKind::Wifi, "Cafe", None, "boss");
    draft.set("802-11-wireless.ssid", "Cafe".into());
    let args = add_arguments(&draft);
    assert!(!args.iter().any(|argument| argument.starts_with("802-1x")));
    assert!(
      !args
        .iter()
        .any(|argument| argument.starts_with("802-11-wireless-security"))
    );
    let mut settings = parse_settings(
      "connection.id:Office\nconnection.uuid:u1\nconnection.type:802-11-wireless\n802-11-wireless.ssid:Office\n802-11-wireless-security.key-mgmt:wpa-eap\n802-1x.eap:peap\n802-1x.identity:me\n",
    );
    let mut office = draft_from_settings(&settings, "boss");
    office.set("802-11-wireless-security.key-mgmt", "wpa-psk".into());
    assert_eq!(removed_settings(&office), ["802-1x"]);
    settings.insert("802-11-wireless-security.key-mgmt".into(), "wpa-psk".into());
    let mut home = draft_from_settings(&settings, "boss");
    home.set("802-11-wireless-security.key-mgmt", String::new());
    assert_eq!(removed_settings(&home), ["802-11-wireless-security"]);
  }

  #[test]
  fn hotspot_is_detected_and_created_in_ap_mode() {
    let settings = parse_settings(
      "connection.type:802-11-wireless\nconnection.uuid:u\n802-11-wireless.mode:ap\n",
    );
    assert_eq!(
      draft_from_settings(&settings, "boss").kind,
      ConnectionKind::Hotspot
    );
    let mut hotspot = ConnectionDraft::new(ConnectionKind::Hotspot, "Hotspot", None, "boss");
    hotspot.set("802-11-wireless.ssid", "argvus".into());
    let args = add_arguments(&hotspot);
    for expected in ["802-11-wireless.mode", "ap", "ipv4.method", "shared"] {
      assert!(
        args.iter().any(|argument| argument == expected),
        "{expected}"
      );
    }
  }

  #[test]
  fn bond_options_are_written_whole_and_keep_unknown_entries() {
    let settings = parse_settings(
      "connection.type:bond\nconnection.uuid:u\nbond.options:mode=active-backup,miimon=100,arp_validate=all\n",
    );
    let mut bond = draft_from_settings(&settings, "boss");
    assert_eq!(bond.value("bond.options.mode"), "active-backup");
    assert_eq!(modify_arguments(&bond), None);
    bond.set("bond.options.primary", "eno1".into());
    let args = modify_arguments(&bond).unwrap();
    assert_eq!(args[4], "bond.options");
    assert_eq!(
      args[5],
      "mode=active-backup,primary=eno1,miimon=100,arp_validate=all"
    );
    // A field hidden by the mode is dropped from the dictionary.
    bond.set("bond.options.mode", "balance-rr".into());
    let args = modify_arguments(&bond).unwrap();
    assert_eq!(args[5], "mode=balance-rr,miimon=100,arp_validate=all");
  }

  #[test]
  fn choices_printed_with_numbers_are_normalized() {
    let settings = parse_settings(
      "connection.type:ip-tunnel\nconnection.uuid:u\nip-tunnel.mode:2 (gre)\nipv6.ip6-privacy:-1 (unknown)\nip-tunnel.mtu:0\n",
    );
    let tunnel = draft_from_settings(&settings, "boss");
    assert_eq!(tunnel.value("ip-tunnel.mode"), "gre");
    assert_eq!(tunnel.value("ipv6.ip6-privacy"), "-1");
    assert_eq!(tunnel.value("ip-tunnel.mtu"), "auto");
  }

  #[test]
  fn openvpn_keeps_plugin_entries_and_sends_the_password_by_stdin() {
    let settings = parse_settings(
      "connection.type:vpn\nconnection.uuid:u\nvpn.service-type:org.freedesktop.NetworkManager.openvpn\nvpn.data:connection-type = password, remote = vpn.example.org, username = me, cipher = AES-256-GCM\n",
    );
    let mut openvpn = draft_from_settings(&settings, "boss");
    assert_eq!(openvpn.value("vpn.data.remote"), "vpn.example.org");
    openvpn.set("vpn.data.remote", "vpn2.example.org".into());
    openvpn.set_secret("vpn.secrets.password", "s3cret".into());
    let args = modify_arguments(&openvpn).unwrap();
    assert_eq!(
      args[5],
      "remote=vpn2.example.org,connection-type=password,username=me,cipher=AES-256-GCM"
    );
    assert!(!args.iter().any(|argument| argument.contains("s3cret")));
    assert_eq!(
      secret_script(&openvpn).unwrap(),
      "set vpn.secrets password=s3cret\nsave persistent\nquit\n"
    );
  }

  #[test]
  fn generic_vpn_data_is_a_list_of_entries() {
    let settings = parse_settings(
      "connection.type:vpn\nconnection.uuid:u\nvpn.service-type:org.freedesktop.NetworkManager.vpnc\nvpn.data:IPSec gateway = vpn.example.org, IPSec ID = group\n",
    );
    let mut vpnc = draft_from_settings(&settings, "boss");
    assert_eq!(
      vpnc.value("vpn.data"),
      "IPSec gateway=vpn.example.org, IPSec ID=group"
    );
    vpnc.set("vpn.data", "IPSec gateway=vpn2.example.org".into());
    let args = modify_arguments(&vpnc).unwrap();
    assert_eq!(args[4..], ["vpn.data", "IPSec gateway=vpn2.example.org"]);
  }

  #[test]
  fn wireguard_peers_round_trip_through_stdin() {
    let printed = "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg= allowed-ips=0.0.0.0/0;\\:\\:/0 endpoint=1.2.3.4\\:51820 persistent-keepalive=25 preshared-key=yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk= preshared-key-flags=0, AAAAA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg= allowed-ips=10.0.0.0/8";
    let peers = parse_peers(printed);
    assert_eq!(peers.len(), 2);
    assert_eq!(peers[0].endpoint, "1.2.3.4:51820");
    assert_eq!(peers[0].allowed_ips, "0.0.0.0/0, ::/0");
    assert_eq!(peers[0].keepalive, "25");
    assert_eq!(
      format_peers(&peers),
      "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg= endpoint=1.2.3.4:51820 allowed-ips=0.0.0.0/0;::/0 persistent-keepalive=25 preshared-key=yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=, AAAAA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg= allowed-ips=10.0.0.0/8"
    );
    let mut draft = ConnectionDraft::new(ConnectionKind::WireGuard, "wg", None, "boss");
    draft.peers = peers;
    let script = secret_script(&draft).unwrap();
    assert!(script.starts_with("set wireguard.peers xTIBA"));
    assert!(
      !add_arguments(&draft)
        .iter()
        .any(|argument| argument.contains("yAnz5"))
    );
  }

  #[test]
  fn a_new_vlan_always_names_its_id() {
    let mut vlan = ConnectionDraft::new(ConnectionKind::Vlan, "VLAN", None, "boss");
    vlan.set("vlan.parent", "eno1".into());
    let args = add_arguments(&vlan);
    let id = args
      .iter()
      .position(|argument| argument == "vlan.id")
      .unwrap();
    assert_eq!(args[id + 1], "0");
  }

  #[test]
  fn ports_join_their_controller() {
    assert_eq!(
      port_arguments("bridge", "br1", "eno2"),
      [
        "connection",
        "add",
        "type",
        "ethernet",
        "con-name",
        "bridge-port-eno2",
        "ifname",
        "eno2",
        "port-type",
        "bridge",
        "controller",
        "br1"
      ]
    );
  }
}
