//! The storage block: the size and use of a mount, as `df` reports it. The
//! sizes are rounded up to whole GiB and the percentage is rounded up, which is
//! how `df -BG` shows them.

const GIBIBYTE: u64 = 1024 * 1024 * 1024;

/// The figures of one mount, in whole GiB, with its use in percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mount {
  pub used_gib: u64,
  pub total_gib: u64,
  pub percent: u64,
}

impl Mount {
  /// The figures of a mount that cannot be read: an empty bar of zeros.
  pub const UNKNOWN: Mount = Mount {
    used_gib: 0,
    total_gib: 0,
    percent: 0,
  };
}

/// Reads the second line of `df -P -B1 MOUNT`: the total and used bytes, and the
/// capacity column, which is the percentage df rounds up.
pub fn parse_df(text: &str) -> Option<Mount> {
  let line = text.lines().nth(1)?;
  let fields: Vec<&str> = line.split_whitespace().collect();
  let total_bytes: u64 = fields.get(1)?.parse().ok()?;
  let used_bytes: u64 = fields.get(2)?.parse().ok()?;
  let percent: u64 = fields.get(4)?.trim_end_matches('%').parse().ok()?;
  Some(Mount {
    used_gib: ceiling_gib(used_bytes),
    total_gib: ceiling_gib(total_bytes),
    percent,
  })
}

fn ceiling_gib(bytes: u64) -> u64 {
  bytes.div_ceil(GIBIBYTE)
}

#[cfg(test)]
mod tests {
  use super::{Mount, parse_df};

  #[test]
  fn sizes_round_up_to_whole_gibibytes() {
    let text = "Filesystem 1-blocks Used Available Capacity Mounted on\n\
                    /dev/sda2 500000000000 120000000000 380000000000 25% /\n";
    assert_eq!(
      parse_df(text),
      Some(Mount {
        used_gib: 112,
        total_gib: 466,
        percent: 25,
      })
    );
  }

  #[test]
  fn missing_mount_reads_as_none() {
    assert_eq!(
      parse_df("Filesystem 1-blocks Used Available Capacity Mounted on\n"),
      None
    );
    assert_eq!(parse_df(""), None);
  }
}
