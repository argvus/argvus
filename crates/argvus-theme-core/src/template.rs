//! Text templates: everything is copied as is, except `{{ expression }}`, which is
//! replaced by the value of the [`Expression`]. `\{{` writes a literal `{{`.
//!
//! `{{#if expression}}…{{/if}}` keeps its text only when the value is set: a
//! non-zero number, non-empty text or any colour. Sections do not nest; the
//! expression is still checked when the section is left out.

use crate::expr::{Expression, Scope, Value};

const OPEN: &str = "{{";
const CLOSE: &str = "}}";

/// A template error with the 1-based line where the expression starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateError {
  pub line: usize,
  pub message: String,
}

impl std::fmt::Display for TemplateError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "line {}: {}", self.line, self.message)
  }
}

/// Renders `template` with the names of `scope`. Every expression is checked,
/// and the first error stops the render.
pub fn render(template: &str, scope: &dyn Scope) -> Result<String, TemplateError> {
  let mut output = String::with_capacity(template.len());
  let mut rest = template;
  let mut line = 1;
  // The line of the open `{{#if}}` and whether its text is kept.
  let mut section: Option<(usize, bool)> = None;
  while let Some(start) = rest.find(OPEN) {
    let (before, from_open) = rest.split_at(start);
    let keep = section.is_none_or(|(_, kept)| kept);
    if let Some(literal) = before.strip_suffix('\\') {
      if keep {
        output.push_str(literal);
        output.push_str(OPEN);
      }
      line += count_lines(before);
      rest = &from_open[OPEN.len()..];
      continue;
    }
    if keep {
      output.push_str(before);
    }
    line += count_lines(before);
    let inner_and_rest = &from_open[OPEN.len()..];
    let end = inner_and_rest.find(CLOSE).ok_or_else(|| TemplateError {
      line,
      message: "`{{` is not closed".into(),
    })?;
    let inner = &inner_and_rest[..end];
    let error = |message: String| TemplateError { line, message };
    let tag = inner.trim();
    if let Some(condition) = tag.strip_prefix("#if ") {
      if section.is_some() {
        return Err(error("`{{#if}}` sections cannot be nested".into()));
      }
      let value = evaluate(condition, scope).map_err(error)?;
      section = Some((line, value.is_set()));
    } else if tag == "/if" {
      if section.take().is_none() {
        return Err(error("`{{/if}}` has no open `{{#if}}`".into()));
      }
    } else {
      let value = evaluate(inner, scope).map_err(error)?;
      if keep {
        output.push_str(&value.render());
      }
    }
    line += count_lines(inner);
    rest = &inner_and_rest[end + CLOSE.len()..];
  }
  if let Some((opened, _)) = section {
    return Err(TemplateError {
      line: opened,
      message: "`{{#if}}` is not closed by `{{/if}}`".into(),
    });
  }
  output.push_str(rest);
  Ok(output)
}

fn evaluate(text: &str, scope: &dyn Scope) -> Result<Value, String> {
  Expression::parse(text).and_then(|expression| expression.evaluate(scope))
}

fn count_lines(text: &str) -> usize {
  text.bytes().filter(|&byte| byte == b'\n').count()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::color::Color;

  struct Accent;

  impl Scope for Accent {
    fn lookup(&self, name: &str) -> Option<Value> {
      match name {
        "palette.accent" => Some(Value::Color(Color::parse("#3590BD").unwrap())),
        "layout.rounded" => Some(Value::Number(1.0)),
        "layout.square" => Some(Value::Number(0.0)),
        _ => None,
      }
    }
  }

  #[test]
  fn replaces_expressions_and_keeps_the_rest() {
    let template = "@define-color th-accent {{ @accent }};\nborder: {{@accent|hypr}};\n";
    assert_eq!(
      render(template, &Accent).unwrap(),
      "@define-color th-accent #3590BD;\nborder: rgba(3590bdff);\n"
    );
  }

  #[test]
  fn escaped_braces_are_literal() {
    assert_eq!(
      render("a \\{{ b }} {{ @accent }}", &Accent).unwrap(),
      "a {{ b }} #3590BD"
    );
  }

  #[test]
  fn text_without_expressions_is_unchanged() {
    let text = "return { gaps = 4 }\n";
    assert_eq!(render(text, &Accent).unwrap(), text);
  }

  #[test]
  fn errors_carry_the_line_of_the_expression() {
    let error = render("one\ntwo\n{{ @missing }}\n", &Accent).unwrap_err();
    assert_eq!(error.line, 3);
    assert!(error.message.contains("unknown name"));

    let error = render("one\n{{ @accent\n", &Accent).unwrap_err();
    assert_eq!(error.line, 2);
    assert!(error.message.contains("not closed"));
  }

  #[test]
  fn sections_keep_their_text_only_when_set() {
    let template = "a\n{{#if layout.rounded}}round {{ @accent }}\n{{/if}}{{#if layout.square}}square\n{{/if}}b\n";
    assert_eq!(render(template, &Accent).unwrap(), "a\nround #3590BD\nb\n");
  }

  #[test]
  fn sections_must_be_balanced_and_flat() {
    for (template, reason, line) in [
      (
        "{{#if layout.rounded}}\n{{#if layout.rounded}}",
        "nested",
        2,
      ),
      ("x\n{{/if}}", "no open", 2),
      ("\n{{#if layout.rounded}}never closed", "not closed", 2),
      ("{{#if layout.missing}}{{/if}}", "unknown name", 1),
      (
        "{{#if layout.square}}{{ @missing }}{{/if}}",
        "unknown name",
        1,
      ),
    ] {
      let error = render(template, &Accent).unwrap_err();
      assert!(error.message.contains(reason), "{template}: {error}");
      assert_eq!(error.line, line, "{template}");
    }
  }
}
