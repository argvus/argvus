//! The session-loading spinner: a long-running service that attaches the
//! spinner surface to the user compositor as soon as it publishes a socket,
//! and keeps retrying until one of them accepts it.
//!
//! The surface is the Wayland mode of the greeter's handoff helper: a plain
//! shared-memory client that maps its first frame within milliseconds of
//! connecting. The GTK splash needed about 300 ms, during which the compositor
//! already showed its background without the spinner.

use crate::accent::DEFAULT_THEME;
use crate::session_paths::SessionPaths;
use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

const DEFAULT_SPLASH_BIN: &str = "/usr/lib/argvus/loading-theme/splash";
const DEFAULT_SPINNER_BIN: &str = "/usr/lib/argvus/greeter/handoff";
/// Pause after a round that found sockets but no accepting compositor. A round
/// starts the spinner process, which only connects and exits on failure.
const RETRY_WITH_SOCKETS: Duration = Duration::from_millis(10);
/// Pause after a round that found no socket at all. A round is only a
/// directory read, and every millisecond here is time without a spinner on
/// screen once Hyprland shows its first frame.
const RETRY_WITHOUT_SOCKETS: Duration = Duration::from_millis(10);

/// Runs the spinner until the splash accepts a socket. Never returns on
/// failure: the service is meant to keep trying while the compositor starts.
pub fn run(paths: &SessionPaths, runtime_dir: &Path, splash: &Path) -> i32 {
  let mut preferred = std::env::var("WAYLAND_DISPLAY")
    .ok()
    .filter(|name| !name.is_empty());
  loop {
    let sockets = socket_candidates(runtime_dir, preferred.as_deref());
    let found = !sockets.is_empty();
    for socket in sockets {
      let display = socket
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
      // Like the shell, the chosen display becomes the preferred one for
      // the following rounds.
      preferred = Some(display.clone());
      if attach(paths, splash, &display) {
        return 0;
      }
    }
    sleep(if found {
      RETRY_WITH_SOCKETS
    } else {
      RETRY_WITHOUT_SOCKETS
    });
  }
}

/// Runs the spinner surface on one display. Returns `true` when it exits
/// successfully, which ends the service.
fn attach(paths: &SessionPaths, splash: &Path, display: &str) -> bool {
  let theme = theme_name(paths);
  let mut command = Command::new(splash);
  // `--retire-handoff` lets the greeter's DRM handoff surface remove the
  // overlay that hides the compositor's first frames once this spinner is on
  // screen. Without a greeter there is no surface and nothing to retire.
  command
    .arg("--wayland")
    .arg("--retire-handoff")
    .arg("--theme")
    .arg(&theme)
    .env("WAYLAND_DISPLAY", display);
  if let Some(accent) = accent_color(paths) {
    command.arg("--accent").arg(accent);
  }
  matches!(command.status(), Ok(status) if status.success())
}

/// The Wayland sockets to try, preferred display first, then the others in
/// name order, as the shell glob listed them.
pub fn socket_candidates(runtime_dir: &Path, preferred: Option<&str>) -> Vec<PathBuf> {
  let mut candidates = Vec::new();
  let mut preferred_path = None;
  if let Some(display) = preferred {
    let path = if display.starts_with('/') {
      PathBuf::from(display)
    } else {
      runtime_dir.join(display)
    };
    if is_socket(&path) {
      preferred_path = Some(path.clone());
      candidates.push(path);
    }
  }
  let Ok(entries) = fs::read_dir(runtime_dir) else {
    return candidates;
  };
  let mut others: Vec<PathBuf> = entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| {
      path
        .file_name()
        .map(|name| name.to_string_lossy().starts_with("wayland-"))
        .unwrap_or(false)
    })
    .filter(|path| is_socket(path) && Some(path) != preferred_path.as_ref())
    .collect();
  others.sort();
  candidates.extend(others);
  candidates
}

fn is_socket(path: &Path) -> bool {
  fs::metadata(path)
    .map(|metadata| metadata.file_type().is_socket())
    .unwrap_or(false)
}

/// The theme name from the first line of the marker, or the ARGVUS default.
/// The data copy is preferred; the root copy is read for profiles that have
/// not migrated yet.
pub fn theme_name(paths: &SessionPaths) -> String {
  let file = marker(paths, ".active-theme");
  first_line_trimmed(&file)
    .filter(|theme| !theme.is_empty())
    .unwrap_or_else(|| DEFAULT_THEME.to_string())
}

/// The accent colour as `#RRGGBB`, or `None` when the marker is missing or is
/// not six hexadecimal digits (with or without `#`).
pub fn accent_color(paths: &SessionPaths) -> Option<String> {
  let file = marker(paths, ".accent-color");
  first_line_trimmed(&file).and_then(|raw| normalize_accent(&raw))
}

/// Normalizes an accent value: drops one leading `#`, then requires exactly six
/// hexadecimal digits.
pub fn normalize_accent(raw: &str) -> Option<String> {
  let digits = raw.strip_prefix('#').unwrap_or(raw);
  if digits.chars().count() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit()) {
    Some(format!("#{digits}"))
  } else {
    None
  }
}

/// The marker file the shell chose: the data copy when it has content,
/// otherwise the root copy.
fn marker(paths: &SessionPaths, name: &str) -> PathBuf {
  let data = paths.config_home.join("argvus/data").join(name);
  if non_empty(&data) {
    return data;
  }
  paths.config_home.join("argvus").join(name)
}

fn non_empty(path: &Path) -> bool {
  fs::metadata(path)
    .map(|metadata| metadata.len() > 0)
    .unwrap_or(false)
}

/// The first line with a trailing CR and surrounding blanks removed. `None`
/// when the file is missing or empty.
fn first_line_trimmed(path: &Path) -> Option<String> {
  if !non_empty(path) {
    return None;
  }
  let text = fs::read_to_string(path).ok()?;
  let first = text.lines().next()?;
  Some(first.trim_end_matches('\r').trim().to_string())
}

/// The splash binary: `ARGVUS_LOADING_THEME_BIN` when set, otherwise the packaged one.
pub fn splash_binary() -> PathBuf {
  std::env::var_os("ARGVUS_LOADING_THEME_BIN")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(DEFAULT_SPLASH_BIN))
}

/// The session spinner binary: `ARGVUS_SESSION_SPINNER_BIN` when set,
/// otherwise the packaged handoff helper.
pub fn spinner_binary() -> PathBuf {
  std::env::var_os("ARGVUS_SESSION_SPINNER_BIN")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(DEFAULT_SPINNER_BIN))
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::net::UnixListener;

  fn paths_under(root: &Path) -> SessionPaths {
    SessionPaths {
      config_home: root.join("config"),
      data_home: root.join("config/argvus/data"),
      system_config: root.join("system"),
    }
  }

  #[test]
  fn accent_accepts_six_hex_digits_with_or_without_hash() {
    assert_eq!(normalize_accent("#AABBCC").as_deref(), Some("#AABBCC"));
    assert_eq!(normalize_accent("aabbcc").as_deref(), Some("#aabbcc"));
    assert_eq!(normalize_accent("#aabbc"), None, "five digits");
    assert_eq!(normalize_accent("#aabbcg"), None, "not hex");
    assert_eq!(normalize_accent("##aabbcc"), None, "two hashes");
    assert_eq!(normalize_accent(""), None);
  }

  #[test]
  fn theme_prefers_the_data_copy_and_falls_back_to_the_root_copy() {
    let root = std::env::temp_dir().join(format!("argvus-loading-theme-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_under(&root);
    fs::create_dir_all(paths.config_home.join("argvus/data")).expect("data dir");
    assert_eq!(theme_name(&paths), DEFAULT_THEME);

    fs::write(
      paths.config_home.join("argvus/.active-theme"),
      "root-theme\n",
    )
    .expect("root");
    assert_eq!(
      theme_name(&paths),
      "root-theme",
      "root copy when data is absent"
    );

    fs::write(
      paths.config_home.join("argvus/data/.active-theme"),
      "  data-theme\r\n",
    )
    .expect("data");
    assert_eq!(
      theme_name(&paths),
      "data-theme",
      "data copy wins and is trimmed"
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn socket_candidates_put_the_preferred_display_first() {
    let root = std::env::temp_dir().join(format!("argvus-loading-sockets-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("runtime dir");
    let _b = UnixListener::bind(root.join("wayland-2")).expect("socket");
    let _a = UnixListener::bind(root.join("wayland-1")).expect("socket");
    fs::write(root.join("wayland-0"), "").expect("regular file is not a socket");

    let names = |list: Vec<PathBuf>| -> Vec<String> {
      list
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
    };
    assert_eq!(
      names(socket_candidates(&root, None)),
      vec!["wayland-1", "wayland-2"]
    );
    assert_eq!(
      names(socket_candidates(&root, Some("wayland-2"))),
      vec!["wayland-2", "wayland-1"]
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn attach_passes_theme_accent_and_display_to_the_splash() {
    let root = std::env::temp_dir().join(format!("argvus-loading-attach-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_under(&root);
    fs::create_dir_all(paths.config_home.join("argvus/data")).expect("data dir");
    fs::write(
      paths.config_home.join("argvus/data/.active-theme"),
      "github-light\n",
    )
    .expect("theme");
    fs::write(
      paths.config_home.join("argvus/data/.accent-color"),
      "#112233\n",
    )
    .expect("accent");
    let log = root.join("args.log");
    let splash = root.join("splash");
    fs::write(
      &splash,
      format!(
        "#!/bin/sh\nprintf '%s %s\\n' \"$WAYLAND_DISPLAY\" \"$*\" > \"{}\"\n",
        log.display()
      ),
    )
    .expect("fake splash");
    let mut permissions = fs::metadata(&splash).expect("meta").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(&splash, permissions).expect("chmod");

    // Another test may fork while the fake splash is still open for writing,
    // which makes its exec fail with ETXTBSY for a moment. Retry briefly.
    let attached = (0..50).any(|_| {
      attach(&paths, &splash, "wayland-7") || {
        sleep(Duration::from_millis(10));
        false
      }
    });
    assert!(attached, "the splash accepted the display");
    assert_eq!(
      fs::read_to_string(&log).expect("log"),
      "wayland-7 --wayland --retire-handoff --theme github-light --accent #112233\n"
    );
    let _ = fs::remove_dir_all(root);
  }
}
