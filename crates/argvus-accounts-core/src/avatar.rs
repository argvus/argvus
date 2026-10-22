//! Avatar storage: validation, normalization and atomic deployment.
//!
//! The canonical avatar location is `$HOME/.face`, the long-standing Unix/
//! freedesktop convention consumed by LightDM, SDDM and most desktop
//! environments. Files are always regular, owned by the target user, mode
//! `0644`, PNG-encoded at 256x256 pixels so any consumer (including a greeter
//! running as an unprivileged user) can render them without extra logic.
//!
//! Updates are atomic: the new image is written to a temporary file inside
//! the user's home directory, fsynced, permissioned, chowned and then renamed
//! over `.face`. An interrupted update therefore never destroys the previous
//! avatar.

use std::fs;
use std::io::{Cursor, Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use image::ImageFormat;
use rustix::fs::{AtFlags, Mode, OFlags, openat, renameat, statat, unlinkat};
use rustix::io::Errno;

use crate::error::{Error, Result};

/// Canonical avatar file name inside the user's home directory.
pub const AVATAR_FILENAME: &str = ".face";

/// Maximum accepted size for a source avatar file (20 MiB).
pub const MAX_SOURCE_FILE_BYTES: u64 = 20 * 1024 * 1024;

/// Edge length of the normalized output image.
pub const OUTPUT_SIZE: u32 = 256;

/// Minimum accepted input dimension.
pub const MIN_INPUT_DIMENSION: u32 = 16;

/// Maximum accepted input dimension (guards against decompression bombs).
pub const MAX_INPUT_DIMENSION: u32 = 16384;

/// Hard budget for decoded pixel buffers (128 MiB).
const MAX_DECODED_ALLOC_BYTES: u64 = 128 * 1024 * 1024;

/// Returns the canonical avatar path for a given home directory.
#[must_use]
pub fn avatar_path(home: &Path) -> std::path::PathBuf {
  home.join(AVATAR_FILENAME)
}

/// Returns the avatar path if, and only if, it currently exists as a regular
/// file. Symlinks are deliberately not followed.
#[must_use]
pub fn find_avatar(home: &Path) -> Option<std::path::PathBuf> {
  let path = avatar_path(home);
  let is_regular = path
    .symlink_metadata()
    .map(|meta| meta.is_file())
    .unwrap_or(false);
  is_regular.then_some(path)
}

/// Validates `source` and deploys it atomically as the user's avatar.
///
/// The source file is fully copied/normalized; the original is never touched.
/// `owner` must match the uid/gid of the account that owns `home`.
pub fn set_avatar(home: &Path, source: &Path, owner: (u32, u32)) -> Result<std::path::PathBuf> {
  // O_NONBLOCK keeps open() from waiting on a FIFO that the path was swapped
  // to. The checks below run on the open descriptor, so they cannot be raced
  // by a path replacement and are not fooled by special files reporting len 0.
  let file = fs::OpenOptions::new()
    .read(true)
    .custom_flags(libc::O_NONBLOCK)
    .open(source)
    .map_err(|err| {
      Error::InvalidImage(format!(
        "cannot read source image '{}': {err}",
        source.display()
      ))
    })?;
  let meta = file.metadata()?;
  if !meta.is_file() {
    return Err(Error::InvalidImage(format!(
      "source image '{}' is not a regular file",
      source.display()
    )));
  }
  if meta.len() > MAX_SOURCE_FILE_BYTES {
    return Err(Error::ImageTooLarge {
      actual: meta.len(),
      max: MAX_SOURCE_FILE_BYTES,
    });
  }
  // Read at most one byte past the limit: the file may grow after the stat.
  let mut bytes = Vec::new();
  file
    .take(MAX_SOURCE_FILE_BYTES + 1)
    .read_to_end(&mut bytes)?;
  if bytes.len() as u64 > MAX_SOURCE_FILE_BYTES {
    return Err(Error::ImageTooLarge {
      actual: bytes.len() as u64,
      max: MAX_SOURCE_FILE_BYTES,
    });
  }
  let png = normalize_png(&bytes)?;
  deploy_png(home, &png, owner)
}

/// Removes the avatar of a user whose home directory is `home`.
pub fn remove_avatar(home: &Path) -> Result<()> {
  let home_dir = open_home_dir(home)?;
  // NOFOLLOW: a symlink named `.face` is removed itself, never its target.
  let stat = match statat(&home_dir, AVATAR_FILENAME, AtFlags::SYMLINK_NOFOLLOW) {
    Ok(stat) => stat,
    Err(Errno::NOENT) => {
      return Err(Error::AvatarNotFound(AVATAR_FILENAME.to_string()));
    }
    Err(errno) => return Err(std::io::Error::from(errno).into()),
  };
  if rustix::fs::FileType::from_raw_mode(stat.st_mode) == rustix::fs::FileType::Directory {
    return Err(Error::system(format!(
      "'{}' is a directory; refusing to remove it",
      avatar_path(home).display()
    )));
  }
  unlinkat(&home_dir, AVATAR_FILENAME, AtFlags::empty()).map_err(std::io::Error::from)?;

  // Best effort: durability hint, not a correctness requirement here.
  let _ = home_dir.sync_all();
  Ok(())
}

/// Sniffs the real format, decodes under strict limits and re-encodes as a
/// square 256x256 PNG using a center crop. The format is detected from the
/// byte stream itself; file extensions are never trusted.
fn normalize_png(bytes: &[u8]) -> Result<Vec<u8>> {
  let mut reader = image::ImageReader::new(Cursor::new(bytes));
  let mut limits = image::Limits::default();
  limits.max_image_width = Some(MAX_INPUT_DIMENSION);
  limits.max_image_height = Some(MAX_INPUT_DIMENSION);
  limits.max_alloc = Some(MAX_DECODED_ALLOC_BYTES);
  reader.limits(limits);

  let reader = reader.with_guessed_format()?;
  let format = reader
    .format()
    .ok_or_else(|| Error::UnsupportedImageFormat("unrecognized".to_string()))?;
  match format {
    ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP => {}
    other => {
      return Err(Error::UnsupportedImageFormat(format!("{other:?}")));
    }
  }

  let decoded = reader.decode()?;
  let (width, height) = (decoded.width(), decoded.height());
  if width < MIN_INPUT_DIMENSION || height < MIN_INPUT_DIMENSION {
    return Err(Error::InvalidImage(format!(
      "dimensions {width}x{height} below minimum {MIN_INPUT_DIMENSION}x{MIN_INPUT_DIMENSION}"
    )));
  }

  let squared = decoded.resize_to_fill(
    OUTPUT_SIZE,
    OUTPUT_SIZE,
    image::imageops::FilterType::Lanczos3,
  );
  let mut png = Vec::with_capacity(64 * 1024);
  squared.write_to(&mut Cursor::new(&mut png), ImageFormat::Png)?;
  Ok(png)
}

/// Opens the home directory without following a symlink in its final
/// component. Every later step works relative to this descriptor, so a path
/// swapped for a symlink after the checks cannot redirect the write.
fn open_home_dir(home: &Path) -> Result<fs::File> {
  let home_fd = rustix::fs::open(
    home,
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
    Mode::empty(),
  )
  .map_err(|errno| match errno {
    // NOENT: missing. LOOP: a symlink. NOTDIR: not a directory.
    Errno::NOENT | Errno::LOOP | Errno::NOTDIR => {
      Error::HomeDirUnusable(home.display().to_string())
    }
    other => std::io::Error::from(other).into(),
  })?;
  Ok(fs::File::from(home_fd))
}

fn deploy_png(home: &Path, png: &[u8], owner: (u32, u32)) -> Result<std::path::PathBuf> {
  let home_dir = open_home_dir(home)?;
  let home_uid = home_dir.metadata()?.uid();
  if home_uid != owner.0 {
    return Err(Error::PermissionDenied(format!(
      "home directory '{}' is owned by uid {home_uid}, expected uid {}",
      home.display(),
      owner.0
    )));
  }

  // O_EXCL and O_NOFOLLOW on the temporary name: an existing entry is an error
  // rather than something to write through.
  let temp_name = format!(
    ".face-tmp-{}-{}.png",
    std::process::id(),
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map_or(0, |elapsed| elapsed.subsec_nanos())
  );
  let temp_fd = openat(
    &home_dir,
    temp_name.as_str(),
    OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
    Mode::from_raw_mode(0o600),
  )
  .map_err(std::io::Error::from)?;
  let mut temp = fs::File::from(temp_fd);

  let staged = stage_temp(&mut temp, png, owner).and_then(|()| {
    // renameat(2) replaces any existing entry, including a symlink, atomically.
    renameat(&home_dir, temp_name.as_str(), &home_dir, AVATAR_FILENAME)
      .map_err(std::io::Error::from)?;
    Ok(())
  });
  if staged.is_err() {
    // Best effort: the temporary file is ours and must not be left behind.
    let _ = unlinkat(&home_dir, temp_name.as_str(), AtFlags::empty());
  }
  staged?;

  // Best effort: durability hint, not a correctness requirement here.
  let _ = home_dir.sync_all();
  Ok(avatar_path(home))
}

/// Writes and permissions the temporary avatar before it is renamed into place.
fn stage_temp(temp: &mut fs::File, png: &[u8], owner: (u32, u32)) -> Result<()> {
  temp.write_all(png)?;
  temp.sync_all()?;
  temp.set_permissions(fs::Permissions::from_mode(0o644))?;
  std::os::unix::fs::fchown(&*temp, Some(owner.0), Some(owner.1))?;
  Ok(())
}
