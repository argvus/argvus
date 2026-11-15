//! Bounded, disposable image cache. Loading and conversion run off the GTK thread.
use argvus_theme_core::manifest::ThemeManifest;
use std::{
  collections::hash_map::DefaultHasher,
  fs,
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
  process::{Command, Stdio},
};

pub struct Image {
  pub pixels: Vec<u8>,
  pub width: i32,
  pub height: i32,
  pub illustrative: bool,
}

pub fn image(theme: &ThemeManifest) -> Option<Image> {
  let root = cache_root()?;
  image_at(theme, &root)
}

fn image_at(theme: &ThemeManifest, root: &Path) -> Option<Image> {
  for (path, illustrative) in [
    (theme.preview.as_deref(), false),
    (theme.wallpaper.as_deref(), true),
  ] {
    if let Some(png) = path.and_then(|path| read_image(path, root)) {
      let decoded = image::load_from_memory(&png).ok()?.into_rgba8();
      let (width, height) = decoded.dimensions();
      let pixels = decoded
        .pixels()
        .flat_map(|pixel| {
          let [red, green, blue, alpha] = pixel.0.map(u32::from);
          ((alpha << 24)
            | ((red * alpha / 255) << 16)
            | ((green * alpha / 255) << 8)
            | (blue * alpha / 255))
            .to_ne_bytes()
        })
        .collect();
      return Some(Image {
        pixels,
        width: width as i32,
        height: height as i32,
        illustrative,
      });
    }
  }
  None
}

fn thumbnail(bytes: &[u8]) -> Option<Vec<u8>> {
  let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
    .with_guessed_format()
    .ok()?;
  let mut limits = image::Limits::default();
  limits.max_image_width = Some(16384);
  limits.max_image_height = Some(16384);
  limits.max_alloc = Some(256 * 1024 * 1024);
  reader.limits(limits);
  let decoded = reader.decode().ok()?.thumbnail(960, 540);
  let mut output = std::io::Cursor::new(Vec::new());
  decoded
    .write_to(&mut output, image::ImageFormat::Png)
    .ok()?;
  Some(output.into_inner())
}

fn read_image(path: &Path, root: &Path) -> Option<Vec<u8>> {
  let metadata = fs::metadata(path).ok()?;
  if metadata.len() > 64 * 1024 * 1024 {
    return None;
  }
  fs::create_dir_all(root).ok()?;
  let mut hash = DefaultHasher::new();
  path.hash(&mut hash);
  metadata.len().hash(&mut hash);
  metadata.modified().ok()?.hash(&mut hash);
  let cached = root.join(format!("{:016x}.png", hash.finish()));
  if let Ok(bytes) = fs::read(&cached)
    && let Some(png) = thumbnail(&bytes)
  {
    return Some(png);
  }
  let temporary = tempfile::Builder::new()
    .suffix(".png")
    .tempfile_in(root)
    .ok()?;
  let bytes = if path
    .extension()
    .is_some_and(|extension| extension.eq_ignore_ascii_case("jxl"))
  {
    let mut child = Command::new("djxl")
      .arg(path)
      .arg(temporary.path())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .ok()?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
      match child.try_wait() {
        Ok(Some(status)) if status.success() => break,
        Ok(Some(_)) => return None,
        Err(_) => {
          let _ = child.kill();
          let _ = child.wait();
          return None;
        }
        Ok(None) if std::time::Instant::now() < deadline => {
          std::thread::sleep(std::time::Duration::from_millis(25))
        }
        Ok(None) => {
          let _ = child.kill();
          let _ = child.wait();
          return None;
        }
      }
    }
    thumbnail(&fs::read(temporary.path()).ok()?)?
  } else {
    thumbnail(&fs::read(path).ok()?)?
  };
  fs::write(temporary.path(), &bytes).ok()?;
  if let Err(error) = temporary.persist(&cached) {
    eprintln!("argvus-theme-selector: cache: {error}");
  }
  prune(root);
  Some(bytes)
}

fn prune(root: &Path) {
  let Ok(entries) = fs::read_dir(root) else {
    return;
  };
  let mut files: Vec<_> = entries
    .flatten()
    .filter_map(|entry| {
      let path = entry.path();
      if path.extension().is_some_and(|extension| extension == "png") {
        Some((entry.metadata().ok()?.modified().ok()?, path))
      } else {
        None
      }
    })
    .collect();
  files.sort_by_key(|(modified, _)| *modified);
  let excess = files.len().saturating_sub(128);
  for (_, path) in files.into_iter().take(excess) {
    if let Err(error) = fs::remove_file(path) {
      eprintln!("argvus-theme-selector: pruning cache: {error}");
    }
  }
}

fn cache_root() -> Option<PathBuf> {
  Some(
    std::env::var_os("XDG_CACHE_HOME")
      .map(PathBuf::from)
      .filter(|path| path.is_absolute())
      .or_else(|| {
        std::env::var_os("HOME")
          .map(|home| PathBuf::from(home).join(".cache"))
          .filter(|path| path.is_absolute())
      })?
      .join("argvus/theme-selector/v2"),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn broken_preview_falls_back_to_wallpaper_and_cache_tracks_source() {
    fs::create_dir_all("/tmp/argvus").unwrap();
    let temp = tempfile::tempdir_in("/tmp/argvus").unwrap();
    let dir = temp.path();
    fs::write(dir.join("preview.png"), b"invalid image").unwrap();
    let mut pixels = image::RgbaImage::from_pixel(2000, 1000, image::Rgba([53, 144, 189, 255]));
    let wallpaper = dir.join("wallpaper.png");
    pixels.save(&wallpaper).unwrap();
    fs::write(dir.join("theme.toml"), "schema = 2\nid = \"demo\"\nname = \"Demo\"\ncategory = \"dark\"\npreview = \"preview.png\"\nwallpaper = \"wallpaper.png\"\n[palette]\nbg = \"#000000\"\nfg = \"#eeeeee\"\naccent = \"#3590bd\"\n").unwrap();
    let theme = ThemeManifest::load(dir).unwrap();
    let cache = dir.join("cache");
    let image = image_at(&theme, &cache).unwrap();
    assert!(image.illustrative);
    assert_eq!(image.pixels, image_at(&theme, &cache).unwrap().pixels);
    assert!(image.width <= 960);
    for pixel in pixels.pixels_mut() {
      *pixel = image::Rgba([255, 0, 0, 255]);
    }
    pixels.save(&wallpaper).unwrap();
    assert_ne!(image.pixels, image_at(&theme, &cache).unwrap().pixels);
    // A valid author-provided preview takes priority over the illustrated wallpaper.
    fs::copy(&wallpaper, dir.join("preview.png")).unwrap();
    assert!(!image_at(&theme, &cache).unwrap().illustrative);
    fs::write(dir.join("preview.png"), b"broken again").unwrap();
    fs::remove_file(&wallpaper).unwrap();
    assert!(image_at(&theme, &cache).is_none());
  }
}
