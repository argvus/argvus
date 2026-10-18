//! The ports block: the listening TCP ports, from `ss -tln`.

/// The count of listening ports and the first few of them, as the block shows.
pub const SHOWN_PORTS: usize = 5;

/// The unique listening ports of `ss -tln`, in ascending order. The port is the
/// part of the local address after its last colon.
pub fn parse_ss(text: &str) -> Vec<u64> {
  let mut ports: Vec<u64> = text
    .lines()
    .skip(1)
    .filter_map(|line| line.split_whitespace().nth(3))
    .map(|address| {
      address
        .rsplit(':')
        .next()
        .and_then(|port| port.parse().ok())
        .unwrap_or(0)
    })
    .collect();
  ports.sort_unstable();
  ports.dedup();
  ports
}

/// The first ports, comma separated, and `+N` for the ports that are not shown.
pub fn shown(ports: &[u64]) -> String {
  let mut text = ports
    .iter()
    .take(SHOWN_PORTS)
    .map(u64::to_string)
    .collect::<Vec<_>>()
    .join(",");
  if ports.len() > SHOWN_PORTS {
    text.push_str(&format!(",+{}", ports.len() - SHOWN_PORTS));
  }
  text
}

#[cfg(test)]
mod tests {
  use super::{parse_ss, shown};

  #[test]
  fn ports_are_unique_and_sorted() {
    let text = "State Recv-Q Send-Q Local Address:Port Peer Address:Port\n\
                    LISTEN 0 4096 127.0.0.1:631 0.0.0.0:*\n\
                    LISTEN 0 128 [::]:22 [::]:*\n\
                    LISTEN 0 128 0.0.0.0:22 0.0.0.0:*\n";
    assert_eq!(parse_ss(text), [22, 631]);
  }

  #[test]
  fn shown_lists_five_and_counts_the_rest() {
    assert_eq!(shown(&[1, 2, 3]), "1,2,3");
    assert_eq!(shown(&[1, 2, 3, 4, 5, 6, 7]), "1,2,3,4,5,+2");
    assert_eq!(shown(&[]), "");
  }
}
