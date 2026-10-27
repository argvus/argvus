//! Checks of the values typed in the connection editor, before anything
//! reaches `nmcli`. Each failure names what is wrong, so the editor can show
//! it next to the field.

use super::schema::{Family, ListRule, SecretRule, TextRule, Widget};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Why a value was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldError {
  Required,
  ControlCharacter,
  InterfaceName,
  Mac,
  Mtu,
  Address,
  Cidr,
  Route,
  Domain,
  KeyValue,
  Ssid,
  Integer,
  OutOfRange,
  Endpoint,
  WpaPassphrase,
  WireGuardKey,
  /// A manual method without any address.
  AddressRequired,
}

impl FieldError {
  /// Translation key of the message.
  pub fn message_key(self) -> &'static str {
    match self {
      Self::Required => "control_center.net_e_required",
      Self::ControlCharacter => "control_center.net_e_control_character",
      Self::InterfaceName => "control_center.net_e_interface_name",
      Self::Mac => "control_center.net_e_mac",
      Self::Mtu => "control_center.net_e_mtu",
      Self::Address => "control_center.net_e_address",
      Self::Cidr => "control_center.net_e_cidr",
      Self::Route => "control_center.net_e_route",
      Self::Domain => "control_center.net_e_domain",
      Self::KeyValue => "control_center.net_e_key_value",
      Self::Ssid => "control_center.net_e_ssid",
      Self::Integer => "control_center.net_e_integer",
      Self::OutOfRange => "control_center.net_e_out_of_range",
      Self::Endpoint => "control_center.net_e_endpoint",
      Self::WpaPassphrase => "control_center.net_e_wpa_passphrase",
      Self::WireGuardKey => "control_center.net_e_wireguard_key",
      Self::AddressRequired => "control_center.net_e_address_required",
    }
  }
}

/// MAC policies NetworkManager accepts in place of an address.
const MAC_POLICIES: [&str; 4] = ["preserve", "permanent", "random", "stable"];
/// Longest kernel interface name (`IFNAMSIZ - 1`).
const INTERFACE_NAME_MAX: usize = 15;

/// Checks the value of a field. Lists are checked entry by entry with
/// [`check_list_entry`]; their values here are the comma-joined entries.
pub fn check(widget: Widget, value: &str) -> Result<(), FieldError> {
  if value.chars().any(char::is_control) {
    return Err(FieldError::ControlCharacter);
  }
  match widget {
    Widget::Text(rule) => check_text(rule, value),
    Widget::Secret(rule) => check_secret(rule, value),
    Widget::Number { min, max, .. } => check_integer(value, min, max),
    Widget::Device { required } => check_text(TextRule::InterfaceName { required }, value),
    Widget::List(rule) => split_list(value)
      .iter()
      .try_for_each(|entry| check_list_entry(rule, entry)),
    Widget::Toggle | Widget::Choice(_) => Ok(()),
  }
}

fn check_text(rule: TextRule, value: &str) -> Result<(), FieldError> {
  let value = value.trim();
  match rule {
    TextRule::Free => Ok(()),
    TextRule::Required if value.is_empty() => Err(FieldError::Required),
    TextRule::Required => Ok(()),
    TextRule::InterfaceName { required } => {
      if value.is_empty() {
        return if required {
          Err(FieldError::Required)
        } else {
          Ok(())
        };
      }
      let valid = value.len() <= INTERFACE_NAME_MAX
        && value != "."
        && value != ".."
        && !value
          .chars()
          .any(|character| matches!(character, '/' | ':') || character.is_whitespace());
      valid.then_some(()).ok_or(FieldError::InterfaceName)
    }
    TextRule::ClonedMac if value.is_empty() || MAC_POLICIES.contains(&value) => Ok(()),
    TextRule::ClonedMac => is_mac(value).then_some(()).ok_or(FieldError::Mac),
    TextRule::Mtu if value.is_empty() || value == "auto" || value == "0" => Ok(()),
    TextRule::Mtu => match value.parse::<u32>() {
      Ok(68..=65535) => Ok(()),
      _ => Err(FieldError::Mtu),
    },
    TextRule::Address { required, .. } if value.is_empty() => {
      if required {
        Err(FieldError::Required)
      } else {
        Ok(())
      }
    }
    TextRule::Address { family, .. } => is_address(value, family)
      .then_some(())
      .ok_or(FieldError::Address),
    TextRule::Ssid if value.is_empty() => Err(FieldError::Required),
    TextRule::Ssid => (value.len() <= 32).then_some(()).ok_or(FieldError::Ssid),
    TextRule::OptionalInteger { .. } if value.is_empty() => Ok(()),
    TextRule::OptionalInteger { min, max } => check_integer(value, min, max),
    TextRule::Endpoint { required } if value.is_empty() => {
      if required {
        Err(FieldError::Required)
      } else {
        Ok(())
      }
    }
    TextRule::Endpoint { .. } => is_endpoint(value).then_some(()).ok_or(FieldError::Endpoint),
  }
}

fn check_secret(rule: SecretRule, value: &str) -> Result<(), FieldError> {
  match rule {
    SecretRule::Free => Ok(()),
    SecretRule::WpaPassphrase => {
      let passphrase = (8..=63).contains(&value.len()) && value.is_ascii();
      let hex_key = value.len() == 64 && value.chars().all(|digit| digit.is_ascii_hexdigit());
      (passphrase || hex_key)
        .then_some(())
        .ok_or(FieldError::WpaPassphrase)
    }
    SecretRule::WireGuardKey => is_wireguard_key(value)
      .then_some(())
      .ok_or(FieldError::WireGuardKey),
  }
}

fn check_integer(value: &str, min: i64, max: i64) -> Result<(), FieldError> {
  let number = value
    .trim()
    .parse::<i64>()
    .map_err(|_| FieldError::Integer)?;
  (min..=max)
    .contains(&number)
    .then_some(())
    .ok_or(FieldError::OutOfRange)
}

/// Checks one entry of a list field.
pub fn check_list_entry(rule: ListRule, entry: &str) -> Result<(), FieldError> {
  let entry = entry.trim();
  if entry.is_empty() {
    return Err(FieldError::Required);
  }
  if entry.chars().any(char::is_control) {
    return Err(FieldError::ControlCharacter);
  }
  let valid = match rule {
    ListRule::Cidr(family) => is_cidr(entry, family),
    ListRule::Address(family) => is_address(entry, family),
    ListRule::Domain => is_domain(entry),
    ListRule::Route(family) => is_route(entry, family),
    ListRule::KeyValue => entry
      .split_once('=')
      .is_some_and(|(key, _)| !key.trim().is_empty() && !entry.contains(',')),
  };
  valid.then_some(()).ok_or(match rule {
    ListRule::Cidr(_) => FieldError::Cidr,
    ListRule::Address(_) => FieldError::Address,
    ListRule::Domain => FieldError::Domain,
    ListRule::Route(_) => FieldError::Route,
    ListRule::KeyValue => FieldError::KeyValue,
  })
}

/// The entries of a comma-separated list value, trimmed, without empty ones.
pub fn split_list(value: &str) -> Vec<String> {
  value
    .split(',')
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .map(str::to_owned)
    .collect()
}

fn is_address(value: &str, family: Family) -> bool {
  match family {
    Family::V4 => value.parse::<Ipv4Addr>().is_ok(),
    Family::V6 => value.parse::<Ipv6Addr>().is_ok(),
    Family::Any => value.parse::<IpAddr>().is_ok(),
  }
}

fn is_cidr(value: &str, family: Family) -> bool {
  let (address, prefix) = value.split_once('/').unwrap_or((value, ""));
  let longest = match address.parse::<IpAddr>() {
    Ok(IpAddr::V4(_)) if family != Family::V6 => 32,
    Ok(IpAddr::V6(_)) if family != Family::V4 => 128,
    _ => return false,
  };
  // A bare address is a host route: NetworkManager takes /32 or /128.
  prefix.is_empty() || prefix.parse::<u8>().is_ok_and(|prefix| prefix <= longest)
}

fn is_route(value: &str, family: Family) -> bool {
  let mut parts = value.split_whitespace();
  let Some(destination) = parts.next() else {
    return false;
  };
  if !is_cidr(destination, family) {
    return false;
  }
  let rest = parts.collect::<Vec<_>>();
  match rest.as_slice() {
    [] => true,
    [next] => is_address(next, family) || next.parse::<u32>().is_ok(),
    [next_hop, metric] => is_address(next_hop, family) && metric.parse::<u32>().is_ok(),
    _ => false,
  }
}

fn is_domain(value: &str) -> bool {
  value.len() <= 253
    && value.trim_start_matches('~').split('.').all(|label| {
      !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
          .chars()
          .all(|character| character.is_ascii_alphanumeric() || character == '-')
    })
}

fn is_mac(value: &str) -> bool {
  let octets = value.split(':').collect::<Vec<_>>();
  octets.len() == 6
    && octets
      .iter()
      .all(|octet| octet.len() == 2 && octet.chars().all(|digit| digit.is_ascii_hexdigit()))
}

fn is_endpoint(value: &str) -> bool {
  let Some((host, port)) = value.rsplit_once(':') else {
    return false;
  };
  let host = host
    .strip_prefix('[')
    .and_then(|inner| inner.strip_suffix(']'))
    .unwrap_or(host);
  let host_valid = host.parse::<IpAddr>().is_ok() || is_domain(host);
  host_valid && port.parse::<u16>().is_ok_and(|port| port > 0)
}

/// Whether `value` is 32 bytes in standard base64 (44 characters, one `=`).
pub fn is_wireguard_key(value: &str) -> bool {
  let bytes = value.as_bytes();
  bytes.len() == 44
    && bytes[43] == b'='
    && bytes[..43]
      .iter()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/'))
    // The last data character carries 4 padding bits, which must be zero.
    && b"AEIMQUYcgkosw048".contains(&bytes[42])
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn interface_names_follow_the_kernel_rules() {
    let rule = Widget::Device { required: true };
    assert!(check(rule, "eno1").is_ok());
    assert!(check(rule, "br0.100").is_ok());
    assert_eq!(check(rule, ""), Err(FieldError::Required));
    assert_eq!(check(rule, "a/b"), Err(FieldError::InterfaceName));
    assert_eq!(
      check(rule, "sixteen-chars-xx"),
      Err(FieldError::InterfaceName)
    );
    assert!(check(Widget::Device { required: false }, "").is_ok());
  }

  #[test]
  fn addresses_cidrs_and_routes() {
    let v4 = ListRule::Cidr(Family::V4);
    assert!(check_list_entry(v4, "192.168.0.2/24").is_ok());
    assert!(check_list_entry(v4, "10.0.0.1").is_ok());
    assert_eq!(
      check_list_entry(v4, "192.168.0.2/33"),
      Err(FieldError::Cidr)
    );
    assert_eq!(
      check_list_entry(v4, "2001:db8::1/64"),
      Err(FieldError::Cidr)
    );
    assert!(check_list_entry(ListRule::Cidr(Family::V6), "2001:db8::1/64").is_ok());
    let route = ListRule::Route(Family::V4);
    assert!(check_list_entry(route, "10.0.0.0/8").is_ok());
    assert!(check_list_entry(route, "10.0.0.0/8 192.168.0.1").is_ok());
    assert!(check_list_entry(route, "10.0.0.0/8 192.168.0.1 100").is_ok());
    assert!(check_list_entry(route, "10.0.0.0/8 100").is_ok());
    assert_eq!(
      check_list_entry(route, "10.0.0.0/8 nowhere"),
      Err(FieldError::Route)
    );
    assert!(check_list_entry(ListRule::Address(Family::V4), "1.1.1.1").is_ok());
    assert_eq!(
      check(Widget::List(ListRule::Address(Family::V4)), "1.1.1.1, ::1"),
      Err(FieldError::Address)
    );
  }

  #[test]
  fn domains_and_key_values() {
    assert!(check_list_entry(ListRule::Domain, "example.org").is_ok());
    assert!(check_list_entry(ListRule::Domain, "~.").is_err());
    assert!(check_list_entry(ListRule::Domain, "~corp.example").is_ok());
    assert_eq!(
      check_list_entry(ListRule::Domain, "bad_domain"),
      Err(FieldError::Domain)
    );
    assert!(check_list_entry(ListRule::KeyValue, "gateway=vpn.example.org").is_ok());
    assert_eq!(
      check_list_entry(ListRule::KeyValue, "=x"),
      Err(FieldError::KeyValue)
    );
  }

  #[test]
  fn mac_mtu_and_numbers() {
    let mac = Widget::Text(TextRule::ClonedMac);
    assert!(check(mac, "D2:E5:28:7A:2A:6E").is_ok());
    assert!(check(mac, "random").is_ok());
    assert_eq!(check(mac, "D2:E5:28"), Err(FieldError::Mac));
    let mtu = Widget::Text(TextRule::Mtu);
    assert!(check(mtu, "auto").is_ok());
    assert!(check(mtu, "1500").is_ok());
    assert_eq!(check(mtu, "20"), Err(FieldError::Mtu));
    let vlan = Widget::Number {
      min: 0,
      max: 4094,
      step: 1,
    };
    assert!(check(vlan, "100").is_ok());
    assert_eq!(check(vlan, "4095"), Err(FieldError::OutOfRange));
    assert_eq!(check(vlan, "ten"), Err(FieldError::Integer));
  }

  #[test]
  fn secrets() {
    let wpa = Widget::Secret(SecretRule::WpaPassphrase);
    assert!(check(wpa, "correct horse").is_ok());
    assert_eq!(check(wpa, "short"), Err(FieldError::WpaPassphrase));
    assert!(check(wpa, &"a".repeat(64)).is_ok());
    let key = Widget::Secret(SecretRule::WireGuardKey);
    assert!(check(key, "yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=").is_ok());
    assert_eq!(check(key, "not-a-key"), Err(FieldError::WireGuardKey));
  }

  #[test]
  fn endpoints() {
    let endpoint = Widget::Text(TextRule::Endpoint { required: true });
    assert!(check(endpoint, "vpn.example.org:51820").is_ok());
    assert!(check(endpoint, "[2001:db8::1]:51820").is_ok());
    assert!(check(endpoint, "192.0.2.1:51820").is_ok());
    assert_eq!(check(endpoint, "192.0.2.1"), Err(FieldError::Endpoint));
    assert_eq!(check(endpoint, ""), Err(FieldError::Required));
  }

  #[test]
  fn control_characters_never_pass() {
    assert_eq!(
      check(Widget::Text(TextRule::Free), "a\nb"),
      Err(FieldError::ControlCharacter)
    );
  }
}
