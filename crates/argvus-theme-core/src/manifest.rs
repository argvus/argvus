//! The theme manifest, `theme.toml` with `schema = 2`.
//!
//! A theme is data: metadata, a palette of named colours, the 16 terminal
//! colours and optional per-consumer overrides. Every consumer file is rendered
//! from it by the templates the components ship.
//!
//! ```toml
//! schema = 2
//! id = "sunset"
//! name = "Sunset"
//! category = "dark"
//! wallpaper = "wallpapers/sunset-abstract-dark.jxl"
//!
//! [palette]
//! bg = "#0F0F0F"
//! fg = "#EADCCC"
//! accent = "#E2BE8A"
//! accent_dim = "@accent | alpha(0.13)"
//! ```
//!
//! A palette entry is a colour or an expression over other palette entries, so
//! a highlight colour chosen by the user also changes the entries derived from it.

use crate::color::Color;
use crate::error::{ThemeError, read_text};
use crate::expr::{Expression, Scope, Value};
use crate::paths::relative_inside;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: u32 = 2;
pub const MANIFEST_FILE: &str = "theme.toml";

/// The palette entries every theme must define.
pub const REQUIRED_TOKENS: [&str; 3] = ["bg", "fg", "accent"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
  Dark,
  Light,
}

impl Category {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Dark => "dark",
      Self::Light => "light",
    }
  }

  fn default_gtk_scheme(self) -> &'static str {
    match self {
      Self::Dark => "prefer-dark",
      Self::Light => "prefer-light",
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
  schema: u32,
  id: String,
  name: String,
  category: Category,
  wallpaper: Option<String>,
  preview: Option<String>,
  #[serde(default)]
  name_i18n: BTreeMap<String, String>,
  palette: BTreeMap<String, String>,
  #[serde(default)]
  ansi: BTreeMap<String, String>,
  #[serde(default)]
  gtk: RawGtk,
  #[serde(default)]
  overrides: BTreeMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGtk {
  scheme: Option<String>,
}

/// A validated theme manifest.
#[derive(Debug, Clone)]
pub struct ThemeManifest {
  /// The directory that holds `theme.toml`; relative paths start here.
  pub dir: PathBuf,
  pub id: String,
  pub name: String,
  pub category: Category,
  pub wallpaper: Option<PathBuf>,
  pub preview: Option<PathBuf>,
  pub name_i18n: BTreeMap<String, String>,
  pub gtk_scheme: String,
  palette: BTreeMap<String, Expression>,
  ansi: BTreeMap<String, Expression>,
  overrides: BTreeMap<String, PathBuf>,
}

/// The colours of a theme after the expressions and the user's choices are applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPalette {
  pub tokens: BTreeMap<String, Color>,
  pub ansi: BTreeMap<String, Color>,
}

impl ThemeManifest {
  /// Loads `<dir>/theme.toml`.
  pub fn load(dir: &Path) -> Result<Self, ThemeError> {
    let path = dir.join(MANIFEST_FILE);
    Self::parse(dir, &read_text(&path)?)
  }

  pub fn parse(dir: &Path, text: &str) -> Result<Self, ThemeError> {
    let path = dir.join(MANIFEST_FILE);
    let invalid = |message: String| ThemeError::invalid(&path, message);
    let raw: RawManifest = toml::from_str(text).map_err(|error| invalid(error.to_string()))?;
    if raw.schema != SCHEMA_VERSION {
      return Err(invalid(format!(
        "schema {} is not supported (expected {SCHEMA_VERSION})",
        raw.schema
      )));
    }
    validate_id(&raw.id).map_err(invalid)?;
    if raw.name.trim().is_empty() {
      return Err(invalid("name must not be empty".into()));
    }
    let palette = parse_entries(&raw.palette, "palette", is_token_name).map_err(invalid)?;
    if let Some(missing) = REQUIRED_TOKENS
      .iter()
      .find(|token| !palette.contains_key(**token))
    {
      return Err(invalid(format!("palette.{missing} is required")));
    }
    let ansi = parse_entries(&raw.ansi, "ansi", is_ansi_name).map_err(invalid)?;
    let file = |field: &str, value: Option<String>| -> Result<Option<PathBuf>, ThemeError> {
      value
        .map(|value| {
          theme_path(dir, &value).map_err(|message| invalid(format!("{field}: {message}")))
        })
        .transpose()
    };
    let wallpaper = file("wallpaper", raw.wallpaper)?;
    let preview = file("preview", raw.preview)?;
    let overrides = raw
      .overrides
      .into_iter()
      .map(|(consumer, value)| {
        relative_inside(&value)
          .map(|relative| (consumer.clone(), dir.join(relative)))
          .map_err(|message| invalid(format!("overrides.{consumer}: {message}")))
      })
      .collect::<Result<_, _>>()?;
    let gtk_scheme = match raw.gtk.scheme {
      Some(scheme) if matches!(scheme.as_str(), "prefer-dark" | "prefer-light" | "default") => {
        scheme
      }
      Some(scheme) => {
        return Err(invalid(format!(
          "gtk.scheme must be prefer-dark, prefer-light or default (got {scheme})"
        )));
      }
      None => raw.category.default_gtk_scheme().to_owned(),
    };
    Ok(Self {
      dir: dir.to_path_buf(),
      id: raw.id,
      name: raw.name,
      category: raw.category,
      wallpaper,
      preview,
      name_i18n: raw.name_i18n,
      gtk_scheme,
      palette,
      ansi,
      overrides,
    })
  }

  /// This manifest with `defaults` (palette expressions from the consumer
  /// registry) for the tokens it does not define itself.
  pub fn with_palette_defaults(
    &self,
    defaults: &BTreeMap<String, String>,
  ) -> Result<Self, ThemeError> {
    let path = self.dir.join(MANIFEST_FILE);
    let parsed = parse_entries(defaults, "palette", is_token_name)
      .map_err(|message| ThemeError::invalid(&path, message))?;
    let mut manifest = self.clone();
    for (token, expression) in parsed {
      manifest.palette.entry(token).or_insert(expression);
    }
    Ok(manifest)
  }

  /// One palette entry, evaluated with only the entries it depends on.
  pub fn token(&self, name: &str) -> Result<Color, ThemeError> {
    let mut resolver = Resolver {
      entries: &self.palette,
      accent: None,
      resolved: BTreeMap::new(),
      visiting: Vec::new(),
    };
    resolver
      .resolve(name)
      .map_err(|message| ThemeError::invalid(&self.dir.join(MANIFEST_FILE), message))
  }

  /// The template a theme ships for one consumer instead of the component's.
  pub fn override_for(&self, consumer: &str) -> Option<&Path> {
    self.overrides.get(consumer).map(PathBuf::as_path)
  }

  /// Evaluates the palette. `accent` replaces `palette.accent`, so the entries
  /// derived from it follow the user's highlight colour.
  pub fn resolve_palette(&self, accent: Option<Color>) -> Result<ResolvedPalette, ThemeError> {
    let path = self.dir.join(MANIFEST_FILE);
    let mut resolver = Resolver {
      entries: &self.palette,
      accent,
      resolved: BTreeMap::new(),
      visiting: Vec::new(),
    };
    for name in self.palette.keys() {
      resolver
        .resolve(name)
        .map_err(|message| ThemeError::invalid(&path, message))?;
    }
    let tokens = resolver.resolved;
    let scope = PaletteScope(&tokens);
    let ansi = self
      .ansi
      .iter()
      .map(|(name, expression)| {
        let color = expect_color(expression.evaluate(&scope), &format!("ansi.{name}"))
          .map_err(|message| ThemeError::invalid(&path, message))?;
        Ok((name.clone(), color))
      })
      .collect::<Result<_, ThemeError>>()?;
    Ok(ResolvedPalette { tokens, ansi })
  }
}

/// Checks a theme id: lowercase letters, digits and inner hyphens, without the
/// `-float` suffix, which is a layout and not a theme.
pub fn validate_id(id: &str) -> Result<(), String> {
  if id.is_empty() {
    return Err("id must not be empty".into());
  }
  if !id
    .bytes()
    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
  {
    return Err(format!("id `{id}` may only use a-z, 0-9 and `-`"));
  }
  if id.starts_with('-') || id.ends_with('-') || id.contains("--") {
    return Err(format!("id `{id}` has a misplaced `-`"));
  }
  if id.ends_with("-float") {
    return Err(format!("id `{id}` must not end in -float"));
  }
  Ok(())
}

/// Checks palette entries outside a manifest (the registry defaults).
pub fn validate_palette_entries(entries: &BTreeMap<String, String>) -> Result<(), String> {
  parse_entries(entries, "palette", is_token_name).map(|_| ())
}

fn is_token_name(name: &str) -> bool {
  !name.is_empty()
    && name.starts_with(|first: char| first.is_ascii_lowercase())
    && name
      .bytes()
      .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn is_ansi_name(name: &str) -> bool {
  name
    .strip_prefix("color")
    .and_then(|index| index.parse::<u8>().ok())
    .is_some_and(|index| index < 16 && name == format!("color{index}"))
}

fn parse_entries(
  raw: &BTreeMap<String, String>,
  table: &str,
  valid_name: fn(&str) -> bool,
) -> Result<BTreeMap<String, Expression>, String> {
  raw
    .iter()
    .map(|(name, text)| {
      if !valid_name(name) {
        return Err(format!("{table}.{name} is not a valid entry name"));
      }
      let expression =
        Expression::parse(text).map_err(|message| format!("{table}.{name}: {message}"))?;
      Ok((name.clone(), expression))
    })
    .collect()
}

/// A wallpaper or preview path: absolute, or relative inside the theme directory.
fn theme_path(dir: &Path, value: &str) -> Result<PathBuf, String> {
  let path = Path::new(value);
  if path.is_absolute() {
    return Ok(path.to_path_buf());
  }
  relative_inside(value).map(|relative| dir.join(relative))
}

struct Resolver<'a> {
  entries: &'a BTreeMap<String, Expression>,
  accent: Option<Color>,
  resolved: BTreeMap<String, Color>,
  visiting: Vec<String>,
}

impl Resolver<'_> {
  fn resolve(&mut self, name: &str) -> Result<Color, String> {
    if let Some(color) = self.resolved.get(name) {
      return Ok(*color);
    }
    if name == "accent"
      && let Some(accent) = self.accent
    {
      self.resolved.insert(name.to_owned(), accent);
      return Ok(accent);
    }
    if self.visiting.iter().any(|visiting| visiting == name) {
      self.visiting.push(name.to_owned());
      return Err(format!(
        "palette entries refer to each other in a cycle: {}",
        self.visiting.join(" -> ")
      ));
    }
    let expression = self
      .entries
      .get(name)
      .ok_or_else(|| format!("palette.{name} is not defined"))?;
    self.visiting.push(name.to_owned());
    for dependency in expression.names() {
      let token = dependency.strip_prefix("palette.").ok_or_else(|| {
        format!("palette.{name} may only refer to palette entries, not `{dependency}`")
      })?;
      self.resolve(token)?;
    }
    self.visiting.pop();
    let color = expect_color(
      expression.evaluate(&PaletteScope(&self.resolved)),
      &format!("palette.{name}"),
    )?;
    self.resolved.insert(name.to_owned(), color);
    Ok(color)
  }
}

fn expect_color(value: Result<Value, String>, entry: &str) -> Result<Color, String> {
  match value {
    Ok(Value::Color(color)) => Ok(color),
    Ok(_) => Err(format!("{entry} must be a colour")),
    Err(message) => Err(format!("{entry}: {message}")),
  }
}

struct PaletteScope<'a>(&'a BTreeMap<String, Color>);

impl Scope for PaletteScope<'_> {
  fn lookup(&self, name: &str) -> Option<Value> {
    let token = name.strip_prefix("palette.")?;
    self.0.get(token).copied().map(Value::Color)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const MINIMAL: &str = r##"
schema = 2
id = "sunset"
name = "Sunset"
category = "dark"
wallpaper = "wallpapers/sunset.jxl"

[name_i18n]
pt-BR = "Pôr do Sol"

[palette]
bg = "#0F0F0F"
fg = "#EADCCC"
accent = "#E2BE8A"
accent_dim = "@accent | alpha(0.13)"
title = "@accent"

[ansi]
color1 = "#E25D6C"
color4 = "@accent"
"##;

  fn manifest(text: &str) -> Result<ThemeManifest, ThemeError> {
    ThemeManifest::parse(Path::new("/themes/sunset"), text)
  }

  #[test]
  fn parses_metadata_and_resolves_relative_paths() {
    let theme = manifest(MINIMAL).unwrap();
    assert_eq!(theme.id, "sunset");
    assert_eq!(theme.category, Category::Dark);
    assert_eq!(theme.gtk_scheme, "prefer-dark");
    assert_eq!(
      theme.wallpaper.as_deref(),
      Some(Path::new("/themes/sunset/wallpapers/sunset.jxl"))
    );
    assert_eq!(theme.name_i18n["pt-BR"], "Pôr do Sol");
  }

  #[test]
  fn derived_entries_follow_the_users_accent() {
    let theme = manifest(MINIMAL).unwrap();
    let palette = theme.resolve_palette(None).unwrap();
    assert_eq!(palette.tokens["title"].hex(), "#E2BE8A");
    assert_eq!(palette.tokens["accent_dim"].hex_with_alpha(), "#E2BE8A21");
    assert_eq!(palette.ansi["color4"].hex(), "#E2BE8A");

    let chosen = Color::parse("#3590BD");
    let palette = theme.resolve_palette(chosen).unwrap();
    assert_eq!(palette.tokens["accent"].hex(), "#3590BD");
    assert_eq!(palette.tokens["title"].hex(), "#3590BD");
    assert_eq!(palette.tokens["accent_dim"].hex_with_alpha(), "#3590BD21");
    assert_eq!(palette.ansi["color4"].hex(), "#3590BD");
  }

  #[test]
  fn cycles_and_dangling_references_are_errors() {
    let cycle = MINIMAL.replace(
      "title = \"@accent\"",
      "title = \"@other\"\nother = \"@title\"",
    );
    let error = manifest(&cycle).unwrap().resolve_palette(None).unwrap_err();
    assert!(error.to_string().contains("cycle"), "{error}");

    let dangling = MINIMAL.replace("title = \"@accent\"", "title = \"@missing\"");
    let error = manifest(&dangling)
      .unwrap()
      .resolve_palette(None)
      .unwrap_err();
    assert!(error.to_string().contains("palette.missing"), "{error}");

    let outside = MINIMAL.replace("title = \"@accent\"", "title = \"layout.rounding\"");
    let error = manifest(&outside)
      .unwrap()
      .resolve_palette(None)
      .unwrap_err();
    assert!(
      error.to_string().contains("only refer to palette"),
      "{error}"
    );
  }

  #[test]
  fn rejects_invalid_manifests() {
    for (text, reason) in [
      (MINIMAL.replace("schema = 2", "schema = 1"), "schema 1"),
      (MINIMAL.replace("\"sunset\"", "\"sunset-float\""), "-float"),
      (MINIMAL.replace("\"sunset\"", "\"Sunset\""), "a-z"),
      (
        MINIMAL.replace("fg = \"#EADCCC\"\n", ""),
        "palette.fg is required",
      ),
      (
        MINIMAL.replace("\"#0F0F0F\"", "\"#0F0F0\""),
        "invalid colour",
      ),
      (MINIMAL.replace("color1", "color16"), "ansi.color16"),
      (MINIMAL.replace("wallpapers/", "../"), "wallpaper"),
      (format!("unknown = 1\n{MINIMAL}"), "unknown field"),
      (
        format!("{MINIMAL}\n[gtk]\nscheme = \"dark\"\n"),
        "gtk.scheme",
      ),
    ] {
      let error = manifest(&text).unwrap_err().to_string();
      assert!(error.contains(reason), "{reason}: {error}");
    }
  }

  #[test]
  fn overrides_stay_inside_the_theme() {
    let text = format!("{MINIMAL}\n[overrides]\nlauncher = \"overrides/theme.rasi\"\n");
    let theme = manifest(&text).unwrap();
    assert_eq!(
      theme.override_for("launcher"),
      Some(Path::new("/themes/sunset/overrides/theme.rasi"))
    );
    let escaping = format!("{MINIMAL}\n[overrides]\nlauncher = \"/etc/passwd\"\n");
    assert!(manifest(&escaping).is_err());
  }
}
