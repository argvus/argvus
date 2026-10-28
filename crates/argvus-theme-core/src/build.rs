//! Rendering a theme into a build, and switching the active build.
//!
//! A build is a directory named after its content (`<id>-<variant>-<hash>`),
//! written next to the others and renamed into place in one step, so the files
//! the consumers read never change under them. The active build is the target
//! of the `current` symlink, which is replaced with one `rename(2)`.

use crate::color::Color;
use crate::error::{ThemeError, read_text};
use crate::expr::{Scope, Value};
use crate::layout::{Layout, Variant};
use crate::manifest::{ResolvedPalette, ThemeManifest};
use crate::registry::{Consumer, Reload};
use crate::template;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The description each build carries, for the switch that activates it.
pub const BUILD_MANIFEST: &str = "build.toml";

/// Unfinished builds older than this are left over from a crash.
const STALE_TEMPORARY_AGE: Duration = Duration::from_secs(600);

/// What to render: a theme, its layout and the user's highlight colour.
pub struct RenderRequest<'a> {
  pub manifest: &'a ThemeManifest,
  pub variant: Variant,
  pub accent: Option<Color>,
  pub layout: Option<&'a Layout>,
  pub consumers: &'a [Consumer],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedFile {
  pub consumer: String,
  pub output: PathBuf,
  pub contents: String,
}

/// A rendered theme, still in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
  pub theme: String,
  pub variant: Variant,
  pub files: Vec<RenderedFile>,
  description: String,
}

#[derive(Serialize)]
struct BuildDescription<'a> {
  theme: &'a str,
  variant: &'a str,
  category: &'a str,
  gtk_scheme: &'a str,
  accent: String,
  /// `palette.bg`, for the readers that paint a plain background.
  background: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  wallpaper: Option<String>,
  consumers: Vec<ConsumerDescription<'a>>,
}

#[derive(Serialize)]
struct ConsumerDescription<'a> {
  id: &'a str,
  output: String,
  reload: &'a Reload,
}

/// The names a template can read.
struct RenderScope<'a> {
  manifest: &'a ThemeManifest,
  variant: Variant,
  palette: &'a ResolvedPalette,
  layout: Option<&'a Layout>,
}

impl Scope for RenderScope<'_> {
  fn lookup(&self, name: &str) -> Option<Value> {
    let (namespace, key) = name.split_once('.')?;
    match namespace {
      "palette" => self.palette.tokens.get(key).copied().map(Value::Color),
      "ansi" => self.palette.ansi.get(key).copied().map(Value::Color),
      "layout" => self.layout?.values.get(key).cloned(),
      "theme" => {
        let text = match key {
          "id" => self.manifest.id.clone(),
          "name" => self.manifest.name.clone(),
          "category" => self.manifest.category.as_str().to_owned(),
          "variant" => self.variant.as_str().to_owned(),
          "gtk_scheme" => self.manifest.gtk_scheme.clone(),
          "wallpaper" => self.manifest.wallpaper.as_ref()?.display().to_string(),
          _ => return None,
        };
        Some(Value::Text(text))
      }
      _ => None,
    }
  }
}

/// Renders every consumer. All of them are tried, so one run reports every
/// broken template; any error means there is no build.
pub fn render(request: &RenderRequest<'_>) -> Result<Rendered, Vec<ThemeError>> {
  let manifest = request
    .manifest
    .with_palette_defaults(&crate::registry::palette_defaults(request.consumers))
    .map_err(|error| vec![error])?;
  let palette = manifest
    .resolve_palette(request.accent)
    .map_err(|error| vec![error])?;
  let scope = RenderScope {
    manifest: &manifest,
    variant: request.variant,
    palette: &palette,
    layout: request.layout,
  };
  let mut files = Vec::with_capacity(request.consumers.len());
  let mut errors = Vec::new();
  for consumer in request.consumers {
    match render_consumer(request.manifest, consumer, &scope) {
      Ok(file) => files.push(file),
      Err(error) => errors.push(error),
    }
  }
  if !errors.is_empty() {
    return Err(errors);
  }
  let description = describe(request, &palette).map_err(|error| vec![error])?;
  Ok(Rendered {
    theme: request.manifest.id.clone(),
    variant: request.variant,
    files,
    description,
  })
}

fn render_consumer(
  manifest: &ThemeManifest,
  consumer: &Consumer,
  scope: &RenderScope<'_>,
) -> Result<RenderedFile, ThemeError> {
  let template_path = manifest
    .override_for(&consumer.id)
    .unwrap_or(&consumer.template);
  let template_text = read_text(template_path)?;
  let contents = template::render(&template_text, scope).map_err(|error| ThemeError::Render {
    consumer: consumer.id.clone(),
    template: template_path.to_path_buf(),
    error,
  })?;
  Ok(RenderedFile {
    consumer: consumer.id.clone(),
    output: consumer.output.clone(),
    contents,
  })
}

fn describe(request: &RenderRequest<'_>, palette: &ResolvedPalette) -> Result<String, ThemeError> {
  let manifest = request.manifest;
  let description = BuildDescription {
    theme: &manifest.id,
    variant: request.variant.as_str(),
    category: manifest.category.as_str(),
    gtk_scheme: &manifest.gtk_scheme,
    accent: palette.tokens["accent"].hex(),
    background: palette.tokens["bg"].hex(),
    wallpaper: manifest
      .wallpaper
      .as_ref()
      .map(|path| path.display().to_string()),
    consumers: request
      .consumers
      .iter()
      .map(|consumer| ConsumerDescription {
        id: &consumer.id,
        output: consumer.output.display().to_string(),
        reload: &consumer.reload,
      })
      .collect(),
  };
  toml::to_string(&description).map_err(|error| {
    ThemeError::invalid(&manifest.dir, format!("cannot describe the build: {error}"))
  })
}

impl Rendered {
  /// The directory name: theme, variant and a hash of everything written.
  pub fn build_name(&self) -> String {
    let mut hash = Fnv1a::new();
    hash.write(self.description.as_bytes());
    for file in &self.files {
      hash.write(file.output.as_os_str().as_encoded_bytes());
      hash.write(&[0]);
      hash.write(file.contents.as_bytes());
      hash.write(&[0]);
    }
    format!(
      "{}-{}-{:016x}",
      self.theme,
      self.variant.as_str(),
      hash.finish()
    )
  }

  /// Writes the files under `directory`, creating parents as needed.
  pub fn write_files(&self, directory: &Path) -> Result<(), ThemeError> {
    for file in &self.files {
      write_file(&directory.join(&file.output), &file.contents)?;
    }
    write_file(&directory.join(BUILD_MANIFEST), &self.description)
  }

  /// Writes the build under `builds` and returns its directory. A build with
  /// the same content is reused instead of written again.
  pub fn write_build(&self, builds: &Path) -> Result<PathBuf, ThemeError> {
    fs::create_dir_all(builds).map_err(|error| ThemeError::io(builds, error))?;
    let name = self.build_name();
    let destination = builds.join(&name);
    if destination.is_dir() {
      // Reused builds count as recent for the garbage collector.
      let _ = fs::File::open(&destination)
        .and_then(|directory| directory.set_modified(SystemTime::now()));
      return Ok(destination);
    }
    let temporary = builds.join(format!(".tmp-{name}-{}", std::process::id()));
    remove_tree(&temporary)?;
    let written = self.write_files(&temporary).and_then(|()| {
      fs::rename(&temporary, &destination).map_err(|error| ThemeError::io(&destination, error))
    });
    if let Err(error) = written {
      let _ = remove_tree(&temporary);
      // Another writer finished the same build first; its content is identical.
      if destination.is_dir() {
        return Ok(destination);
      }
      return Err(error);
    }
    Ok(destination)
  }
}

fn write_file(path: &Path, contents: &str) -> Result<(), ThemeError> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|error| ThemeError::io(parent, error))?;
  }
  fs::write(path, contents).map_err(|error| ThemeError::io(path, error))
}

fn remove_tree(path: &Path) -> Result<(), ThemeError> {
  match fs::remove_dir_all(path) {
    Err(error) if error.kind() != io::ErrorKind::NotFound => Err(ThemeError::io(path, error)),
    _ => Ok(()),
  }
}

/// Points `current` at `build` with one atomic rename. `build` must be a
/// directory directly inside the `builds` directory next to `current`.
pub fn commit(current: &Path, build: &Path) -> Result<(), ThemeError> {
  let parent = current
    .parent()
    .ok_or_else(|| ThemeError::invalid(current, "has no parent directory"))?;
  let name = build
    .file_name()
    .filter(|_| build.is_dir())
    .ok_or_else(|| ThemeError::invalid(build, "is not a build directory"))?;
  let target = Path::new("builds").join(name);
  if parent.join(&target) != build {
    return Err(ThemeError::invalid(
      build,
      format!("is not inside {}", parent.join("builds").display()),
    ));
  }
  let temporary = parent.join(format!(".current.tmp-{}", std::process::id()));
  match fs::remove_file(&temporary) {
    Err(error) if error.kind() != io::ErrorKind::NotFound => {
      return Err(ThemeError::io(&temporary, error));
    }
    _ => {}
  }
  symlink(&target, &temporary).map_err(|error| ThemeError::io(&temporary, error))?;
  fs::rename(&temporary, current).map_err(|error| {
    let _ = fs::remove_file(&temporary);
    ThemeError::io(current, error)
  })
}

#[derive(Deserialize)]
struct BuildIdentity {
  theme: String,
  variant: String,
}

/// The theme id (`<id>` or `<id>-float`) of the build behind `current`.
pub fn current_theme(current: &Path) -> Option<String> {
  let text = fs::read_to_string(current.join(BUILD_MANIFEST)).ok()?;
  let identity: BuildIdentity = toml::from_str(&text).ok()?;
  Some(match identity.variant.as_str() {
    "float" => format!("{}-float", identity.theme),
    _ => identity.theme,
  })
}

/// A file of the build behind `current`, only when that build was rendered
/// for `theme`: a consumer asked for another theme reads its own files.
pub fn current_file_for(current: &Path, theme: &str, relative: &str) -> Option<PathBuf> {
  if current_theme(current)? != theme {
    return None;
  }
  Some(current.join(relative)).filter(|path| path.is_file())
}

/// The build `current` points at, when it points at one.
pub fn active_build(current: &Path) -> Option<PathBuf> {
  let target = fs::read_link(current).ok()?;
  let parent = current.parent()?;
  Some(parent.join(target))
}

/// Removes old builds: the active one and the `keep` most recent others stay.
/// Unfinished builds are removed once they are stale. Returns what was removed.
pub fn collect_garbage(
  builds: &Path,
  current: &Path,
  keep: usize,
) -> Result<Vec<PathBuf>, ThemeError> {
  let entries = match fs::read_dir(builds) {
    Ok(entries) => entries,
    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
    Err(error) => return Err(ThemeError::io(builds, error)),
  };
  let active = active_build(current);
  let now = SystemTime::now();
  let mut finished = Vec::new();
  let mut removed = Vec::new();
  for entry in entries {
    let entry = entry.map_err(|error| ThemeError::io(builds, error))?;
    let path = entry.path();
    let modified = entry
      .metadata()
      .and_then(|metadata| metadata.modified())
      .unwrap_or(now);
    if entry.file_name().to_string_lossy().starts_with(".tmp-") {
      if now.duration_since(modified).unwrap_or_default() > STALE_TEMPORARY_AGE {
        remove_tree(&path)?;
        removed.push(path);
      }
      continue;
    }
    if path.is_dir() && Some(&path) != active.as_ref() {
      finished.push((modified, path));
    }
  }
  finished.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
  for (_, path) in finished.into_iter().skip(keep) {
    remove_tree(&path)?;
    removed.push(path);
  }
  Ok(removed)
}

/// FNV-1a, 64 bits: stable across Rust releases, unlike `DefaultHasher`, so a
/// build keeps its name after an upgrade.
struct Fnv1a(u64);

impl Fnv1a {
  const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
  const PRIME: u64 = 0x0000_0100_0000_01b3;

  fn new() -> Self {
    Self(Self::OFFSET)
  }

  fn write(&mut self, bytes: &[u8]) {
    for byte in bytes {
      self.0 ^= u64::from(*byte);
      self.0 = self.0.wrapping_mul(Self::PRIME);
    }
  }

  fn finish(&self) -> u64 {
    self.0
  }
}

#[cfg(test)]
mod tests;
