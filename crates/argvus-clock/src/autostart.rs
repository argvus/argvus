//! "Start with the session": the `argvus-clock.service` user unit, which runs
//! `argvus clock --background` after the graphical session starts.
//!
//! `systemctl` runs asynchronously through `GSubprocess`, so the window never
//! waits for it. The unit is enabled or disabled without `--now`: the running
//! instance already keeps the alarms.

use std::ffi::OsStr;

use gtk::gio;
use gtk::glib::GString;

pub const UNIT: &str = "argvus-clock.service";

/// Reads whether the unit is enabled. `None` means unknown: the unit is not
/// installed or systemd cannot be reached.
pub fn query(done: impl FnOnce(Option<bool>) + 'static) {
  run(&["is-enabled", UNIT], move |result| {
    done(result.ok().and_then(|output| parse_is_enabled(&output)));
  });
}

/// Enables or disables the unit.
pub fn set(enabled: bool, done: impl FnOnce(Result<(), String>) + 'static) {
  let verb = if enabled { "enable" } else { "disable" };
  run(&[verb, UNIT], move |result| done(result.map(|_| ())));
}

fn parse_is_enabled(output: &str) -> Option<bool> {
  match output.trim() {
    "enabled" | "enabled-runtime" => Some(true),
    "disabled" => Some(false),
    _ => None,
  }
}

/// Runs `systemctl --user <arguments>` and passes its standard output, or
/// its error output when it fails. `is-enabled` exits with 1 for a disabled
/// unit, so the output is returned whenever the command printed one.
fn run(arguments: &[&str], done: impl FnOnce(Result<String, String>) + 'static) {
  let mut argv: Vec<&OsStr> = vec![OsStr::new("systemctl"), OsStr::new("--user")];
  argv.extend(arguments.iter().map(OsStr::new));
  let flags = gio::SubprocessFlags::STDOUT_PIPE | gio::SubprocessFlags::STDERR_PIPE;
  let process = match gio::Subprocess::newv(&argv, flags) {
    Ok(process) => process,
    Err(error) => return done(Err(error.message().to_owned())),
  };
  let finished = process.clone();
  process.communicate_utf8_async(None, None::<&gio::Cancellable>, move |result| {
    let text = |output: Option<GString>| {
      output
        .map(|text| text.trim().to_owned())
        .unwrap_or_default()
    };
    done(match result {
      Ok((stdout, stderr)) => {
        let (stdout, stderr) = (text(stdout), text(stderr));
        if finished.is_successful() || !stdout.is_empty() {
          Ok(stdout)
        } else {
          Err(stderr)
        }
      }
      Err(error) => Err(error.message().to_owned()),
    });
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn is_enabled_output_is_parsed() {
    assert_eq!(parse_is_enabled("enabled\n"), Some(true));
    assert_eq!(parse_is_enabled("disabled"), Some(false));
    assert_eq!(parse_is_enabled("not-found"), None);
    assert_eq!(parse_is_enabled(""), None);
  }
}
