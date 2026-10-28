//! CPU model, cores, threads and maximum frequency shown in About.
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Returns the CPU model name from `/proc/cpuinfo` content.
pub fn cpu_model(input: &str) -> Option<String> {
  cpuinfo_values(input, "model name")
    .next()
    .map(str::to_string)
}

/// Counts the logical processors (hardware threads) listed in `/proc/cpuinfo`.
pub fn threads(input: &str) -> Option<usize> {
  let count = cpuinfo_values(input, "processor").count();
  (count > 0).then_some(count)
}

/// Counts the physical cores as unique `(physical id, core id)` pairs of `/proc/cpuinfo`.
///
/// Architectures that omit those fields (such as most ARM kernels) fall back to the
/// sysfs CPU topology.
pub fn cores(input: &str) -> Option<usize> {
  cores_from_cpuinfo(input).or_else(cores_from_sysfs)
}

/// Pairs each `core id` with the `physical id` that precedes it in the same processor block.
fn cores_from_cpuinfo(input: &str) -> Option<usize> {
  let mut package = None;
  let mut pairs = HashSet::new();
  for (key, value) in input.lines().filter_map(|line| line.split_once(':')) {
    match key.trim() {
      "physical id" => package = Some(value.trim()),
      "core id" => {
        pairs.insert((package, value.trim()));
      }
      _ => {}
    }
  }
  (!pairs.is_empty()).then_some(pairs.len())
}

/// Counts unique `(physical_package_id, core_id)` pairs under `/sys/devices/system/cpu`.
fn cores_from_sysfs() -> Option<usize> {
  let root = Path::new("/sys/devices/system/cpu");
  let pairs = fs::read_dir(root)
    .ok()?
    .flatten()
    .filter(|entry| {
      let name = entry.file_name();
      let name = name.to_string_lossy();
      name
        .strip_prefix("cpu")
        .is_some_and(|index| !index.is_empty() && index.bytes().all(|b| b.is_ascii_digit()))
    })
    .filter_map(|entry| {
      let topology = entry.path().join("topology");
      let package = fs::read_to_string(topology.join("physical_package_id")).ok()?;
      let core = fs::read_to_string(topology.join("core_id")).ok()?;
      Some((package.trim().to_string(), core.trim().to_string()))
    })
    .collect::<HashSet<_>>();
  (!pairs.is_empty()).then_some(pairs.len())
}

/// Returns the highest maximum frequency among all CPUs, in MHz.
///
/// Hybrid designs (Intel P/E cores, ARM big.LITTLE) have different limits per core, so
/// the highest one is the processor's rated maximum. `None` where cpufreq is absent,
/// as in most virtual machines.
pub fn max_frequency_mhz() -> Option<u64> {
  let entries = fs::read_dir("/sys/devices/system/cpu").ok()?;
  let limits = entries
    .flatten()
    .filter_map(|entry| fs::read_to_string(entry.path().join("cpufreq/cpuinfo_max_freq")).ok())
    .collect::<Vec<_>>();
  highest_mhz(limits.iter().map(String::as_str))
}

/// Picks the highest value of `cpuinfo_max_freq` (kHz) and converts it to MHz.
pub fn highest_mhz<'a>(limits_khz: impl IntoIterator<Item = &'a str>) -> Option<u64> {
  limits_khz
    .into_iter()
    .filter_map(|value| value.trim().parse::<u64>().ok())
    .max()
    .filter(|khz| *khz > 0)
    .map(|khz| khz / 1000)
}

/// Iterates the trimmed values of one `/proc/cpuinfo` key.
fn cpuinfo_values<'a>(input: &'a str, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
  input.lines().filter_map(move |line| {
    line
      .split_once(':')
      .filter(|(name, _)| name.trim() == key)
      .map(|(_, value)| value.trim())
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Two physical cores with two threads each, as printed on x86.
  const CPUINFO: &str = "processor\t: 0\nmodel name\t: Example CPU\nphysical id\t: 0\ncore id\t\t: 0\n\n\
processor\t: 1\nmodel name\t: Example CPU\nphysical id\t: 0\ncore id\t\t: 1\n\n\
processor\t: 2\nmodel name\t: Example CPU\nphysical id\t: 0\ncore id\t\t: 0\n\n\
processor\t: 3\nmodel name\t: Example CPU\nphysical id\t: 0\ncore id\t\t: 1\n";

  #[test]
  fn reads_cpu_model_name() {
    assert_eq!(cpu_model(CPUINFO).as_deref(), Some("Example CPU"));
    assert_eq!(cpu_model("processor\t: 0\n"), None);
  }

  #[test]
  fn counts_logical_processors_as_threads() {
    assert_eq!(threads(CPUINFO), Some(4));
    assert_eq!(threads(""), None);
  }

  #[test]
  fn counts_unique_physical_cores() {
    assert_eq!(cores_from_cpuinfo(CPUINFO), Some(2));
  }

  #[test]
  fn separates_cores_of_different_sockets() {
    let two_sockets = "physical id\t: 0\ncore id\t: 0\nphysical id\t: 1\ncore id\t: 0\n";
    assert_eq!(cores_from_cpuinfo(two_sockets), Some(2));
  }

  #[test]
  fn uses_the_highest_limit_of_hybrid_cores() {
    let limits = ["4700000\n", "3600000\n", "garbage"];
    assert_eq!(highest_mhz(limits), Some(4700));
    assert_eq!(highest_mhz(["0"]), None);
    assert_eq!(highest_mhz([]), None);
  }

  #[test]
  fn reports_no_cores_without_topology_fields() {
    assert_eq!(cores_from_cpuinfo("processor\t: 0\n"), None);
  }
}
