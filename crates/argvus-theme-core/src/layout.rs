//! Layout geometry: the Sticky and Float values (rounding, gaps, margins) that
//! templates read as `layout.<name>`. Geometry is independent of the theme, so
//! a theme no longer ships a `-float` copy of each file.
//!
//! `<system>/theme/layouts/float.toml`:
//!
//! ```toml
//! rounding = 8
//! taskbar_margin = 16
//! ```

use crate::color::Color;
use crate::error::{ThemeError, read_text};
use crate::expr::Value;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
  Sticky,
  Float,
}

impl Variant {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Sticky => "sticky",
      Self::Float => "float",
    }
  }

  pub fn parse(text: &str) -> Option<Self> {
    match text {
      "sticky" => Some(Self::Sticky),
      "float" => Some(Self::Float),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
  pub variant: Variant,
  pub values: BTreeMap<String, Value>,
}

impl Layout {
  /// Loads `<layouts>/<variant>.toml`. A missing file is `None`: templates
  /// that read `layout.*` then fail with an unknown name.
  pub fn load(layouts: &Path, variant: Variant) -> Result<Option<Self>, ThemeError> {
    let path = layouts.join(format!("{}.toml", variant.as_str()));
    if !path.is_file() {
      return Ok(None);
    }
    Self::parse(&path, variant, &read_text(&path)?).map(Some)
  }

  /// Numbers stay numbers, `#…` strings are colours and other strings are text.
  pub fn parse(path: &Path, variant: Variant, text: &str) -> Result<Self, ThemeError> {
    let table: toml::Table =
      toml::from_str(text).map_err(|error| ThemeError::invalid(path, error.to_string()))?;
    let values = table
      .into_iter()
      .map(|(name, value)| {
        let value = match value {
          toml::Value::Integer(number) => Value::Number(number as f64),
          toml::Value::Float(number) => Value::Number(number),
          toml::Value::String(text) if text.starts_with('#') => Color::parse(&text)
            .map(Value::Color)
            .ok_or_else(|| ThemeError::invalid(path, format!("{name}: invalid colour `{text}`")))?,
          toml::Value::String(text) => Value::Text(text),
          other => {
            return Err(ThemeError::invalid(
              path,
              format!("{name}: a {} is not a layout value", other.type_str()),
            ));
          }
        };
        Ok((name, value))
      })
      .collect::<Result<_, _>>()?;
    Ok(Self { variant, values })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_numbers_colours_and_text() {
    let layout = Layout::parse(
      Path::new("float.toml"),
      Variant::Float,
      "rounding = 8\nopacity = 0.9\nshade = \"#000000\"\nstyle = \"round\"\n",
    )
    .unwrap();
    assert_eq!(layout.values["rounding"], Value::Number(8.0));
    assert_eq!(layout.values["opacity"], Value::Number(0.9));
    assert_eq!(layout.values["shade"], Value::Color(Color::opaque(0, 0, 0)));
    assert_eq!(layout.values["style"], Value::Text("round".into()));
  }

  #[test]
  fn rejects_tables_and_bad_colours() {
    for text in ["nested = { a = 1 }", "shade = \"#00\""] {
      assert!(
        Layout::parse(Path::new("x.toml"), Variant::Sticky, text).is_err(),
        "{text}"
      );
    }
  }

  #[test]
  fn variants_round_trip() {
    for variant in [Variant::Sticky, Variant::Float] {
      assert_eq!(Variant::parse(variant.as_str()), Some(variant));
    }
    assert_eq!(Variant::parse("Float"), None);
  }
}
