//! The output of NetworkManager's nmcli, of `ip` and of `iwgetid`, read as the
//! network status needs it.

/// The first connected device other than the loopback, from
/// `nmcli -t -f DEVICE,STATE device status`.
pub fn first_connected_device(listing: &str) -> Option<String> {
  listing.lines().find_map(|line| {
    let mut fields = line.split(':');
    let device = fields.next()?;
    let state = fields.next()?;
    (state == "connected" && device != "lo" && !device.is_empty()).then(|| device.to_string())
  })
}

/// The device of the first default route, from `ip route show default`.
pub fn default_route_device(routes: &str) -> Option<String> {
  routes
    .lines()
    .next()?
    .split_whitespace()
    .nth(4)
    .map(String::from)
}

/// The type of a device: the first line of `nmcli -g GENERAL.TYPE device show`.
pub fn device_type(output: &str) -> String {
  output.lines().next().unwrap_or_default().to_string()
}

/// The IPv4 address of a device, without its prefix length, from
/// `ip -4 addr show DEVICE`.
pub fn ipv4_address(addresses: &str) -> Option<String> {
  let line = addresses.lines().find(|line| line.contains("inet "))?;
  let address = line.split_whitespace().nth(1)?;
  Some(address.split('/').next().unwrap_or_default().to_string())
}

/// The SSID of the active Wi-Fi network, from
/// `nmcli -t -f ACTIVE,SSID device wifi list ifname DEVICE`.
pub fn active_ssid(listing: &str) -> Option<String> {
  listing.lines().find_map(|line| {
    let mut fields = line.split(':');
    (fields.next()? == "yes").then(|| fields.next().unwrap_or_default().to_string())
  })
}

#[cfg(test)]
mod tests {
  use super::{
    active_ssid, default_route_device, device_type, first_connected_device, ipv4_address,
  };

  #[test]
  fn connected_device_skips_loopback() {
    let listing = "lo:connected\neth0:unavailable\nbr0:connected\n";
    assert_eq!(first_connected_device(listing).as_deref(), Some("br0"));
    assert_eq!(first_connected_device("lo:connected\n"), None);
  }

  #[test]
  fn default_route_and_type_read_the_first_line() {
    assert_eq!(
      default_route_device("default via 192.168.0.1 dev br0 proto dhcp metric 425\n").as_deref(),
      Some("br0")
    );
    assert_eq!(device_type("wifi\nignored\n"), "wifi");
  }

  #[test]
  fn address_drops_the_prefix() {
    let output =
      "2: br0: <BROADCAST>\n    inet 192.168.0.2/24 brd 192.168.0.255 scope global br0\n";
    assert_eq!(ipv4_address(output).as_deref(), Some("192.168.0.2"));
    assert_eq!(ipv4_address("2: br0: <NO-CARRIER>\n"), None);
  }

  #[test]
  fn ssid_is_the_active_network() {
    assert_eq!(
      active_ssid("no:Cafe\nyes:Home net\n").as_deref(),
      Some("Home net")
    );
    assert_eq!(active_ssid("no:Cafe\n"), None);
  }
}
