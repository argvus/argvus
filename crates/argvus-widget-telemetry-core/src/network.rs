//! The network block: the interface and the traffic rate. The figures come from
//! the `key=value` lines of `argvus networkctl status`.

/// The value of the first `key=value` line that names `key`.
pub fn field(status: &str, key: &str) -> Option<String> {
  status.lines().find_map(|line| {
    let (name, value) = line.split_once('=')?;
    (name == key).then(|| value.to_string())
  })
}

/// A byte rate with the largest binary unit it reaches; below a KiB it is whole
/// bytes.
pub fn human_rate(bytes_per_second: i64) -> String {
  const KIB: f64 = 1024.0;
  const MIB: f64 = KIB * 1024.0;
  const GIB: f64 = MIB * 1024.0;
  let bytes = bytes_per_second as f64;
  if bytes >= GIB {
    format!("{:.1}GiB/s", bytes / GIB)
  } else if bytes >= MIB {
    format!("{:.1}MiB/s", bytes / MIB)
  } else if bytes >= KIB {
    format!("{:.1}KiB/s", bytes / KIB)
  } else {
    format!("{bytes_per_second}B/s")
  }
}

#[cfg(test)]
mod tests {
  use super::{field, human_rate};

  #[test]
  fn reads_the_first_value_of_a_key() {
    let status = "iface=br0\nip=192.168.0.2\nrx=10\nrx=20\n";
    assert_eq!(field(status, "iface").as_deref(), Some("br0"));
    assert_eq!(field(status, "rx").as_deref(), Some("10"));
    assert_eq!(field(status, "ssid"), None);
  }

  #[test]
  fn rates_pick_the_largest_unit() {
    assert_eq!(human_rate(512), "512B/s");
    assert_eq!(human_rate(2048), "2.0KiB/s");
    assert_eq!(human_rate(3 * 1024 * 1024 / 2), "1.5MiB/s");
    assert_eq!(human_rate(2 * 1024 * 1024 * 1024), "2.0GiB/s");
  }
}
