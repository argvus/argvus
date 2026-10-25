//! Reads the uptime and the system locale.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::{env, fs};

/// Seconds since boot, from the first field of `/proc/uptime`.
pub fn uptime_seconds() -> Option<u64> {
  parse_uptime(&fs::read_to_string("/proc/uptime").ok()?)
}

/// Parses `/proc/uptime` (`12345.67 54321.00`), dropping the fraction of a second.
pub fn parse_uptime(input: &str) -> Option<u64> {
  let seconds = input.split_whitespace().next()?.parse::<f64>().ok()?;
  (seconds.is_finite() && seconds >= 0.0).then_some(seconds as u64)
}

/// Returns the system locale from `/etc/locale.conf`, falling back to the session `LANG`.
pub fn system_locale() -> Option<String> {
  fs::read_to_string("/etc/locale.conf")
    .ok()
    .and_then(|contents| parse_locale_conf(&contents))
    .or_else(|| env::var("LANG").ok().filter(|value| !value.is_empty()))
}

/// Reads `LANG=` from a `locale.conf` file, ignoring comments and quotes.
pub fn parse_locale_conf(contents: &str) -> Option<String> {
  contents
    .lines()
    .map(str::trim)
    .filter(|line| !line.starts_with('#'))
    .find_map(|line| line.strip_prefix("LANG="))
    .map(|value| {
      value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
    })
    .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_whole_seconds_of_uptime() {
    assert_eq!(parse_uptime("356821.47 2793012.12\n"), Some(356_821));
    assert_eq!(parse_uptime(""), None);
    assert_eq!(parse_uptime("abc 1"), None);
  }

  #[test]
  fn reads_lang_from_locale_conf() {
    let contents = "# generated\nLC_TIME=en_GB.UTF-8\nLANG=\"pt_BR.UTF-8\"\n";
    assert_eq!(parse_locale_conf(contents).as_deref(), Some("pt_BR.UTF-8"));
    assert_eq!(parse_locale_conf("#LANG=C\n"), None);
    assert_eq!(parse_locale_conf("LANG=\n"), None);
  }
}
