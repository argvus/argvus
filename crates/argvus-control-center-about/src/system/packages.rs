//! Counts the installed pacman packages.
use super::command_stdout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Installed packages: all, explicitly installed, and foreign (AUR or built locally).
pub struct PackageCounts {
  pub total: usize,
  pub explicit: usize,
  pub foreign: usize,
}

/// Queries `pacman -Qq`, `-Qqe` and `-Qqm` in parallel; `None` when pacman lists nothing.
///
/// `-Qqm` exits with status 1 when there are no foreign packages, so stdout is read
/// whatever the exit status.
pub fn package_counts() -> Option<PackageCounts> {
  let count = |flag: &'static str| move || count_names(&command_stdout("pacman", &[flag]));
  let (total, explicit, foreign) = std::thread::scope(|scope| {
    let explicit = scope.spawn(count("-Qqe"));
    let foreign = scope.spawn(count("-Qqm"));
    (
      count("-Qq")(),
      explicit.join().unwrap_or_default(),
      foreign.join().unwrap_or_default(),
    )
  });
  (total > 0).then_some(PackageCounts {
    total,
    explicit,
    foreign,
  })
}

/// Counts the package names printed one per line.
pub fn count_names(output: &str) -> usize {
  output
    .lines()
    .filter(|line| !line.trim().is_empty())
    .count()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn counts_one_package_per_line() {
    assert_eq!(count_names("argvus\nhyprland\nwayland\n"), 3);
    assert_eq!(count_names(""), 0);
    assert_eq!(count_names("\n\n"), 0);
  }
}
