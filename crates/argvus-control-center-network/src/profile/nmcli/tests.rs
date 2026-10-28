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
  let settings =
    parse_settings("connection.type:802-11-wireless\nconnection.uuid:u\n802-11-wireless.mode:ap\n");
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
