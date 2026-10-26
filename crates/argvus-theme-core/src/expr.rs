//! Expressions of templates and palette entries: a value followed by filters.
//!
//! ```text
//! palette.accent | alpha(0.13) | argb
//! @accent | mix(@bg, 0.5)
//! layout.rounding
//! ```
//!
//! `@name` is short for `palette.name`. A value is a name looked up in a
//! [`Scope`], a colour literal (`#RRGGBB`) or a number.

use crate::color::{Color, format_decimal};

/// A value produced by an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Color(Color),
  Number(f64),
  Text(String),
}

impl Value {
  /// The text written into a rendered file. An opaque colour is `#RRGGBB`, a
  /// translucent one `#RRGGBBAA`; templates choose other forms with filters.
  pub fn render(&self) -> String {
    match self {
      Self::Color(color) if color.alpha == 255 => color.hex(),
      Self::Color(color) => color.hex_with_alpha(),
      Self::Number(number) => format_decimal(*number),
      Self::Text(text) => text.clone(),
    }
  }

  /// Whether a `{{#if}}` section keeps its text: a non-zero number, non-empty
  /// text or any colour.
  pub fn is_set(&self) -> bool {
    match self {
      Self::Color(_) => true,
      Self::Number(number) => *number != 0.0,
      Self::Text(text) => !text.is_empty(),
    }
  }

  fn kind(&self) -> &'static str {
    match self {
      Self::Color(_) => "a colour",
      Self::Number(_) => "a number",
      Self::Text(_) => "text",
    }
  }
}

/// The names an expression can read.
pub trait Scope {
  fn lookup(&self, name: &str) -> Option<Value>;
}

/// A parsed expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
  operand: Operand,
  filters: Vec<Filter>,
}

#[derive(Debug, Clone, PartialEq)]
enum Operand {
  Name(String),
  Literal(Value),
}

#[derive(Debug, Clone, PartialEq)]
struct Filter {
  name: String,
  arguments: Vec<Operand>,
}

impl Expression {
  pub fn parse(text: &str) -> Result<Self, String> {
    let mut parts = text.split('|');
    let operand = parse_operand(parts.next().unwrap_or_default())?;
    let filters = parts.map(parse_filter).collect::<Result<_, _>>()?;
    Ok(Self { operand, filters })
  }

  /// Every name the expression reads, in the order they appear.
  pub fn names(&self) -> impl Iterator<Item = &str> {
    std::iter::once(&self.operand)
      .chain(self.filters.iter().flat_map(|filter| &filter.arguments))
      .filter_map(|operand| match operand {
        Operand::Name(name) => Some(name.as_str()),
        Operand::Literal(_) => None,
      })
  }

  pub fn evaluate(&self, scope: &dyn Scope) -> Result<Value, String> {
    let mut value = resolve(&self.operand, scope)?;
    for filter in &self.filters {
      let arguments = filter
        .arguments
        .iter()
        .map(|argument| resolve(argument, scope))
        .collect::<Result<Vec<_>, _>>()?;
      value = apply_filter(&filter.name, value, &arguments)?;
    }
    Ok(value)
  }
}

fn resolve(operand: &Operand, scope: &dyn Scope) -> Result<Value, String> {
  match operand {
    Operand::Literal(value) => Ok(value.clone()),
    Operand::Name(name) => scope
      .lookup(name)
      .ok_or_else(|| format!("unknown name `{name}`")),
  }
}

fn parse_operand(text: &str) -> Result<Operand, String> {
  let text = text.trim();
  if text.is_empty() {
    return Err("missing value".into());
  }
  if text.starts_with('#') {
    return Color::parse(text)
      .map(|color| Operand::Literal(Value::Color(color)))
      .ok_or_else(|| format!("invalid colour `{text}`"));
  }
  if text.starts_with(|first: char| first.is_ascii_digit() || first == '-' || first == '.') {
    return text
      .parse::<f64>()
      .ok()
      .filter(|number| number.is_finite())
      .map(|number| Operand::Literal(Value::Number(number)))
      .ok_or_else(|| format!("invalid number `{text}`"));
  }
  let name = match text.strip_prefix('@') {
    Some(token) => format!("palette.{token}"),
    None => text.to_owned(),
  };
  if !is_name(&name) {
    return Err(format!("invalid name `{text}`"));
  }
  Ok(Operand::Name(name))
}

fn is_name(text: &str) -> bool {
  !text.is_empty()
    && !text.starts_with('.')
    && !text.ends_with('.')
    && !text.contains("..")
    && text
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

fn parse_filter(text: &str) -> Result<Filter, String> {
  let text = text.trim();
  let (name, arguments) = match text.split_once('(') {
    Some((name, rest)) => {
      let inner = rest
        .strip_suffix(')')
        .ok_or_else(|| format!("filter `{text}` is missing `)`"))?;
      let arguments = if inner.trim().is_empty() {
        Vec::new()
      } else {
        inner
          .split(',')
          .map(parse_operand)
          .collect::<Result<_, _>>()?
      };
      (name.trim(), arguments)
    }
    None => (text, Vec::new()),
  };
  if name.is_empty()
    || !name
      .bytes()
      .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
  {
    return Err(format!("invalid filter `{text}`"));
  }
  Ok(Filter {
    name: name.to_owned(),
    arguments,
  })
}

fn apply_filter(name: &str, value: Value, arguments: &[Value]) -> Result<Value, String> {
  let text_form = |format: fn(Color) -> String| {
    expect_arity(name, arguments, 0)?;
    Ok(Value::Text(format(expect_color(name, &value)?)))
  };
  match name {
    "hex" => text_form(Color::hex),
    "hexa" => text_form(Color::hex_with_alpha),
    "argb" => text_form(Color::argb_hex),
    "bare" => text_form(Color::bare_hex),
    "rgb" => text_form(Color::css_rgb),
    "rgba" => text_form(Color::css_rgba),
    "rgba_percent" => text_form(Color::css_rgba_percent),
    "rgb_hex" => text_form(Color::hex_rgb_function),
    "hypr" => text_form(Color::hyprland),
    "argb_int" => text_form(Color::argb_integer),
    "alpha" => {
      expect_arity(name, arguments, 1)?;
      let opacity = expect_unit(name, &arguments[0])?;
      Ok(Value::Color(
        expect_color(name, &value)?.with_opacity(opacity),
      ))
    }
    "mix" => {
      expect_arity(name, arguments, 2)?;
      let other = expect_color(name, &arguments[0])?;
      let weight = expect_unit(name, &arguments[1])?;
      Ok(Value::Color(expect_color(name, &value)?.mix(other, weight)))
    }
    "lighten" | "darken" => {
      expect_arity(name, arguments, 1)?;
      let amount = expect_unit(name, &arguments[0])?;
      let color = expect_color(name, &value)?;
      Ok(Value::Color(if name == "lighten" {
        color.lighten(amount)
      } else {
        color.darken(amount)
      }))
    }
    "contrast" => {
      expect_arity(name, arguments, 0)?;
      Ok(Value::Color(expect_color(name, &value)?.contrast()))
    }
    "upper" => {
      expect_arity(name, arguments, 0)?;
      Ok(Value::Text(value.render().to_uppercase()))
    }
    "lower" => {
      expect_arity(name, arguments, 0)?;
      Ok(Value::Text(value.render().to_lowercase()))
    }
    _ => Err(format!("unknown filter `{name}`")),
  }
}

fn expect_arity(name: &str, arguments: &[Value], expected: usize) -> Result<(), String> {
  if arguments.len() == expected {
    Ok(())
  } else {
    Err(format!(
      "filter `{name}` takes {expected} argument(s), got {}",
      arguments.len()
    ))
  }
}

fn expect_color(name: &str, value: &Value) -> Result<Color, String> {
  match value {
    Value::Color(color) => Ok(*color),
    other => Err(format!(
      "filter `{name}` needs a colour, got {}",
      other.kind()
    )),
  }
}

fn expect_unit(name: &str, value: &Value) -> Result<f64, String> {
  match value {
    Value::Number(number) if (0.0..=1.0).contains(number) => Ok(*number),
    Value::Number(number) => Err(format!(
      "filter `{name}` needs a number from 0 to 1, got {}",
      format_decimal(*number)
    )),
    other => Err(format!(
      "filter `{name}` needs a number, got {}",
      other.kind()
    )),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  struct Names(HashMap<&'static str, Value>);

  impl Scope for Names {
    fn lookup(&self, name: &str) -> Option<Value> {
      self.0.get(name).cloned()
    }
  }

  fn scope() -> Names {
    Names(HashMap::from([
      (
        "palette.accent",
        Value::Color(Color::parse("#E2BE8A").unwrap()),
      ),
      ("palette.bg", Value::Color(Color::parse("#0F0F0F").unwrap())),
      ("layout.rounding", Value::Number(8.0)),
      ("theme.name", Value::Text("Sunset".into())),
    ]))
  }

  fn evaluate(text: &str) -> Result<String, String> {
    Expression::parse(text)?
      .evaluate(&scope())
      .map(|value| value.render())
  }

  #[test]
  fn renders_names_literals_and_shorthand() {
    assert_eq!(evaluate("palette.accent").unwrap(), "#E2BE8A");
    assert_eq!(evaluate("@accent").unwrap(), "#E2BE8A");
    assert_eq!(evaluate("layout.rounding").unwrap(), "8");
    assert_eq!(evaluate("theme.name").unwrap(), "Sunset");
    assert_eq!(evaluate("#abc").unwrap(), "#AABBCC");
    assert_eq!(evaluate("0.5").unwrap(), "0.5");
  }

  #[test]
  fn chains_filters_left_to_right() {
    assert_eq!(
      evaluate("@accent | alpha(0.13) | argb").unwrap(),
      "#21E2BE8A"
    );
    assert_eq!(evaluate("@accent|hypr").unwrap(), "rgba(e2be8aff)");
    assert_eq!(evaluate("@accent | mix(@bg, 1) | hex").unwrap(), "#0F0F0F");
    assert_eq!(evaluate("@accent | bare | lower").unwrap(), "e2be8a");
    assert_eq!(evaluate("@accent | alpha(0.5)").unwrap(), "#E2BE8A80");
    assert_eq!(evaluate("@accent | contrast").unwrap(), "#000000");
  }

  #[test]
  fn reports_names_for_dependency_tracking() {
    let expression = Expression::parse("@accent | mix(@bg, 0.5)").unwrap();
    assert_eq!(
      expression.names().collect::<Vec<_>>(),
      ["palette.accent", "palette.bg"]
    );
  }

  #[test]
  fn rejects_bad_expressions_with_a_reason() {
    for (text, reason) in [
      ("", "missing value"),
      ("@missing", "unknown name"),
      ("@accent | nope", "unknown filter"),
      ("@accent | alpha", "takes 1 argument"),
      ("@accent | alpha(2)", "from 0 to 1"),
      ("layout.rounding | hex", "needs a colour"),
      ("@accent | alpha(0.5", "missing `)`"),
      ("#12345", "invalid colour"),
      ("../etc", "invalid"),
    ] {
      let error = evaluate(text).unwrap_err();
      assert!(error.contains(reason), "{text}: {error}");
    }
  }
}
