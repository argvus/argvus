//! Bounded expression evaluator, independent of GTK and external processes.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Angle {
  Degrees,
  Radians,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
  Syntax,
  Domain,
  Limit,
}

/// Evaluates explicit arithmetic expressions with at most 4096 bytes and 64 levels.
pub fn evaluate(input: &str, angle: Angle) -> Result<f64, Error> {
  if input.len() > 4096 {
    return Err(Error::Limit);
  }
  let normalized = input.replace(',', ".").replace('π', "pi");
  let mut parser = Parser {
    input: normalized.as_bytes(),
    position: 0,
    depth: 0,
    angle,
  };
  let value = parser.expression(0)?;
  parser.space();
  if parser.position != parser.input.len() {
    return Err(Error::Syntax);
  }
  finite(value)
}
fn finite(value: f64) -> Result<f64, Error> {
  if value.is_finite() {
    Ok(value)
  } else {
    Err(Error::Domain)
  }
}
struct Parser<'a> {
  input: &'a [u8],
  position: usize,
  depth: usize,
  angle: Angle,
}
impl Parser<'_> {
  fn space(&mut self) {
    while self
      .input
      .get(self.position)
      .is_some_and(u8::is_ascii_whitespace)
    {
      self.position += 1;
    }
  }
  fn take(&mut self, token: u8) -> bool {
    self.space();
    if self.input.get(self.position) == Some(&token) {
      self.position += 1;
      true
    } else {
      false
    }
  }
  fn expression(&mut self, minimum: u8) -> Result<f64, Error> {
    self.depth += 1;
    if self.depth > 64 {
      return Err(Error::Limit);
    }
    let mut left = if self.take(b'-') {
      -self.expression(3)?
    } else if self.take(b'+') {
      self.expression(3)?
    } else {
      self.atom()?
    };
    loop {
      self.space();
      if self.input.get(self.position) == Some(&b'%') {
        if minimum > 5 {
          break;
        }
        self.position += 1;
        left /= 100.0;
        continue;
      }
      let Some(&operator) = self.input.get(self.position) else {
        break;
      };
      let (binding, right_binding) = match operator {
        b'+' | b'-' => (1, 2),
        b'*' | b'/' => (2, 3),
        b'^' => (4, 4),
        _ => break,
      };
      if binding < minimum {
        break;
      }
      self.position += 1;
      let right = self.expression(right_binding)?;
      left = finite(match operator {
        b'+' => left + right,
        b'-' => left - right,
        b'*' => left * right,
        b'/' => left / right,
        _ => left.powf(right),
      })?;
    }
    self.depth -= 1;
    finite(left)
  }
  fn atom(&mut self) -> Result<f64, Error> {
    if self.take(b'(') {
      let value = self.expression(0)?;
      return if self.take(b')') {
        Ok(value)
      } else {
        Err(Error::Syntax)
      };
    }
    self.space();
    let start = self.position;
    while self
      .input
      .get(self.position)
      .is_some_and(u8::is_ascii_alphabetic)
    {
      self.position += 1;
    }
    if self.position > start {
      let name =
        std::str::from_utf8(&self.input[start..self.position]).map_err(|_| Error::Syntax)?;
      match name {
        "pi" => return Ok(std::f64::consts::PI),
        "e" => return Ok(std::f64::consts::E),
        _ => {}
      }
      if !self.take(b'(') {
        return Err(Error::Syntax);
      }
      let value = self.expression(0)?;
      if !self.take(b')') {
        return Err(Error::Syntax);
      }
      let radians = if self.angle == Angle::Degrees {
        value.to_radians()
      } else {
        value
      };
      let inverse = |value: f64| {
        if self.angle == Angle::Degrees {
          value.to_degrees()
        } else {
          value
        }
      };
      return finite(match name {
        "sqrt" => value.sqrt(),
        "ln" => value.ln(),
        "log" => value.log10(),
        "exp" => value.exp(),
        "sin" => radians.sin(),
        "cos" => radians.cos(),
        "tan" => {
          if radians.cos().abs() < 1e-15 {
            return Err(Error::Domain);
          }
          radians.tan()
        }
        "asin" => inverse(value.asin()),
        "acos" => inverse(value.acos()),
        "atan" => inverse(value.atan()),
        _ => return Err(Error::Syntax),
      });
    }
    while self
      .input
      .get(self.position)
      .is_some_and(|byte| byte.is_ascii_digit() || *byte == b'.')
    {
      self.position += 1;
    }
    // An exponent is part of a number only after a numeric mantissa.
    if self.position > start
      && self
        .input
        .get(self.position)
        .is_some_and(|byte| *byte == b'e' || *byte == b'E')
    {
      self.position += 1;
      if self
        .input
        .get(self.position)
        .is_some_and(|byte| *byte == b'+' || *byte == b'-')
      {
        self.position += 1;
      }
      while self
        .input
        .get(self.position)
        .is_some_and(u8::is_ascii_digit)
      {
        self.position += 1;
      }
    }
    let number =
      std::str::from_utf8(&self.input[start..self.position]).map_err(|_| Error::Syntax)?;
    finite(number.parse().map_err(|_| Error::Syntax)?)
  }
}
/// Formats fifteen significant digits, retaining scientific notation for extremes.
pub fn format_result(value: f64) -> String {
  if value == 0.0 {
    return "0".into();
  }
  let scientific = format!("{value:.14e}");
  let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
  let exponent: i32 = exponent.parse().unwrap_or(0);
  if !(-6..15).contains(&exponent) {
    return format!(
      "{}e{exponent}",
      mantissa.trim_end_matches('0').trim_end_matches('.')
    );
  }
  let decimals = usize::try_from((14 - exponent).max(0)).unwrap_or(0);
  let fixed = format!("{value:.decimals$}");
  if fixed.contains('.') {
    fixed.trim_end_matches('0').trim_end_matches('.').into()
  } else {
    fixed
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn precedence_and_functions() {
    for (input, expected) in [
      ("2+3*4", 14.0),
      ("-2^2", -4.0),
      ("2^3^2", 512.0),
      ("2^-2", 0.25),
      ("(2+3)*4", 20.0),
      ("50%", 0.5),
      ("1,5+2.5", 4.0),
      ("sqrt(9)+log(100)", 5.0),
      ("sin(30)", 0.5),
      ("asin(1)", 90.0),
      ("1e3", 1000.0),
    ] {
      assert!(
        (evaluate(input, Angle::Degrees).unwrap() - expected).abs() < 1e-10,
        "{input}"
      );
    }
    assert!((evaluate("sin(pi/2)", Angle::Radians).unwrap() - 1.0).abs() < 1e-10);
  }
  #[test]
  fn rejects_invalid_and_unbounded_input() {
    for input in [
      "",
      "2pi",
      "2(3)",
      "1..2",
      "sin(",
      "abc(1)",
      "1/0",
      "sqrt(-1)",
      "log(0)",
      "asin(2)",
      "exp(1000)",
      "tan(90)",
      "1e",
    ] {
      assert!(evaluate(input, Angle::Degrees).is_err(), "{input}");
    }
    assert_eq!(evaluate(&"(".repeat(70), Angle::Degrees), Err(Error::Limit));
    assert_eq!(
      evaluate(&"1".repeat(4097), Angle::Degrees),
      Err(Error::Limit)
    );
  }
  #[test]
  fn formatting_is_reusable() {
    for value in [0.0, -0.0, 1.0, 1e20, 1e-15, std::f64::consts::PI] {
      let text = format_result(value);
      assert!(evaluate(&text, Angle::Radians).is_ok());
      assert!(text.len() < 24);
    }
    assert_eq!(format_result(1000.0), "1000");
  }
}
