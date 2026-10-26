//! Theme projections for the surfaces around the desktop: the taskbar
//! calendar's stylesheet cache and the theme id the pre-login greeter shows.

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// Writes the calendar stylesheet (`source`, the build's) of `theme` to the
/// calendar cache and tells the running calendar to reload it.
pub fn apply_calendar_theme(source: &Path, cache_file: &Path, theme: &str) -> io::Result<()> {
  let mut text = format!("/* argvus-theme: {theme} */\n").into_bytes();
  text.extend(fs::read(source)?);
  if let Some(parent) = cache_file.parent() {
    fs::create_dir_all(parent)?;
  }
  // Write beside the cache file and rename, so the calendar never reads a
  // half-written stylesheet.
  let temporary = cache_file.with_extension(format!("css.{}", std::process::id()));
  let written = fs::write(&temporary, &text).and_then(|()| fs::rename(&temporary, cache_file));
  if let Err(error) = written {
    let _ = fs::remove_file(&temporary);
    return Err(error);
  }
  let _ = Command::new("argvus")
    .args(["taskbar-calendar", "reload"])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  Ok(())
}

/// Publishes the theme id for the greeter. The greeter runs before login and
/// renders official themes only (`system_themes/<family>/theme.toml`), and the
/// state directory must already exist. The file is named by the user id and
/// replaced atomically.
pub fn publish_greeter_theme(
  system_themes: &Path,
  state_dir: &Path,
  theme: &str,
  user_id: u32,
) -> io::Result<bool> {
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  if argvus_theme_core::manifest::validate_id(family).is_err()
    || !system_themes.join(family).join("theme.toml").is_file()
    || !state_dir.is_dir()
  {
    return Ok(false);
  }
  let target = state_dir.join(user_id.to_string());
  let temporary = state_dir.join(format!("{user_id}.tmp.{}", std::process::id()));
  let written = (|| -> io::Result<()> {
    let mut file = fs::File::create(&temporary)?;
    writeln!(file, "{theme}")?;
    drop(file);
    // The greeter reads the file as another user; the mode is not secret.
    let _ = fs::set_permissions(&temporary, fs::Permissions::from_mode(0o644));
    fs::rename(&temporary, &target)
  })();
  if let Err(error) = written {
    let _ = fs::remove_file(&temporary);
    return Err(error);
  }
  Ok(true)
}

/// The user id of the calling process.
pub fn current_user_id() -> u32 {
  // SAFETY: getuid cannot fail and has no preconditions.
  unsafe { libc::getuid() }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn scratch(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("argvus-surfaces-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("state")).expect("state");
    root
  }

  #[test]
  fn greeter_publishes_official_themes_only() {
    let root = scratch("greeter");
    let themes = root.join("themes");
    fs::create_dir_all(themes.join("sunset")).expect("theme");
    fs::write(themes.join("sunset/theme.toml"), "").expect("manifest");

    assert!(
      publish_greeter_theme(&themes, &root.join("state"), "sunset-float", 1000).expect("publish")
    );
    assert_eq!(
      fs::read_to_string(root.join("state/1000")).expect("published"),
      "sunset-float\n"
    );
    for refused in ["mine", "../sunset", "Sunset"] {
      assert!(
        !publish_greeter_theme(&themes, &root.join("state"), refused, 1001).expect("publish")
      );
    }
    assert!(!root.join("state/1001").exists());
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn the_calendar_cache_names_the_theme() {
    let root = scratch("calendar");
    fs::write(root.join("build.css"), "window {}\n").expect("build");
    let cache = root.join("cache/theme.css");
    apply_calendar_theme(&root.join("build.css"), &cache, "sunset").expect("written");
    assert_eq!(
      fs::read_to_string(cache).expect("cache"),
      "/* argvus-theme: sunset */\nwindow {}\n"
    );
    let _ = fs::remove_dir_all(root);
  }
}
