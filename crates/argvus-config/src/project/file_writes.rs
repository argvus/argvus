//! Atomic writes of projected files; Lua output is checked with `luac -p` before it replaces the previous file.

use super::*;

pub(super) fn atomic_write(path: &Path, contents: &str) -> ConfigResult<()> {
  let temporary = write_temporary(path, contents)?;
  commit_temporary(&temporary, path)
}

/// Writes a generated Lua file only after `luac -p` accepts the new contents.
///
/// Hyprland loads these files on every reload, so a syntax error here would
/// break the whole configuration. A rejected candidate is removed and the
/// previous file is left in place.
pub(super) fn atomic_write_lua(path: &Path, contents: &str) -> ConfigResult<()> {
  let temporary = write_temporary(path, contents)?;
  if let Err(error) = validate_lua(&temporary, path) {
    let _ = fs::remove_file(&temporary);
    return Err(error);
  }
  commit_temporary(&temporary, path)
}

/// Writes `contents` to a sibling temporary file with owner-only permissions.
///
/// The temporary file is removed again when any write step fails, so a
/// failed projection never leaves a partial candidate behind.
pub(super) fn write_temporary(path: &Path, contents: &str) -> ConfigResult<PathBuf> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
  let written = (|| -> std::io::Result<()> {
    let mut file = OpenOptions::new()
      .create_new(true)
      .write(true)
      .open(&temporary)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
  })();
  if let Err(error) = written {
    let _ = fs::remove_file(&temporary);
    return Err(error.into());
  }
  Ok(temporary)
}

pub(super) fn commit_temporary(temporary: &Path, path: &Path) -> ConfigResult<()> {
  fs::rename(temporary, path)?;
  Ok(())
}

/// Runs `luac -p` on a candidate Lua file.
///
/// `luac` is optional at runtime: without it the projection still writes the
/// file and reports a warning, so a missing Lua package does not block
/// unrelated settings from being projected.
pub(super) fn validate_lua(candidate: &Path, target: &Path) -> ConfigResult<()> {
  match Command::new("luac").arg("-p").arg(candidate).output() {
    Ok(output) if output.status.success() => Ok(()),
    Ok(output) => Err(ConfigError::Invalid(format!(
      "generated Lua for {} is invalid: {}",
      target.display(),
      String::from_utf8_lossy(&output.stderr).trim()
    ))),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
      eprintln!(
        "argvus-config: luac not found; {} was written without a Lua syntax check",
        target.display()
      );
      Ok(())
    }
    Err(error) => Err(error.into()),
  }
}
