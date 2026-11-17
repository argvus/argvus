//! Exact, bounded word arithmetic. Values are bit patterns, never floats.
use crate::engine::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Base {
  Binary,
  Octal,
  #[default]
  Decimal,
  Hexadecimal,
}
impl Base {
  pub fn radix(self) -> u32 {
    match self {
      Self::Binary => 2,
      Self::Octal => 8,
      Self::Decimal => 10,
      Self::Hexadecimal => 16,
    }
  }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
  pub base: Base,
  pub bits: u8,
  pub signed: bool,
}
impl Default for Options {
  fn default() -> Self {
    Self {
      base: Base::Decimal,
      bits: 64,
      signed: true,
    }
  }
}
impl Options {
  pub fn validate(self) -> Result<Self, Error> {
    if [8, 16, 32, 64].contains(&self.bits) {
      Ok(self)
    } else {
      Err(Error::Domain)
    }
  }
  fn mask(self) -> u64 {
    u64::MAX >> (64 - self.bits)
  }
  fn signed_value(self, value: u64) -> i128 {
    let value = value & self.mask();
    if self.signed && value & (1u64 << (self.bits - 1)) != 0 {
      i128::from(value) - (1i128 << self.bits)
    } else {
      i128::from(value)
    }
  }
  pub fn format(self, value: u64) -> String {
    let value = value & self.mask();
    match self.base {
      Base::Binary => format!("{value:b}"),
      Base::Octal => format!("{value:o}"),
      Base::Hexadecimal => format!("{value:X}"),
      Base::Decimal => self.signed_value(value).to_string(),
    }
  }
}
/// Evaluates up to 4096 bytes and 64 recursive levels with modular overflow.
pub fn evaluate(input: &str, options: Options) -> Result<String, Error> {
  Ok(options.format(word(input, options)?))
}
pub fn word(input: &str, options: Options) -> Result<u64, Error> {
  let options = options.validate()?;
  if input.len() > 4096 {
    return Err(Error::Limit);
  }
  let mut parser = Parser {
    input: input.as_bytes(),
    position: 0,
    depth: 0,
    options,
  };
  let value = parser.expression(0)?;
  parser.space();
  if parser.position != parser.input.len() {
    return Err(Error::Syntax);
  }
  Ok(value)
}

struct Parser<'a> {
  input: &'a [u8],
  position: usize,
  depth: u8,
  options: Options,
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
  fn take(&mut self, token: &[u8]) -> bool {
    self.space();
    if self.input[self.position..].starts_with(token) {
      self.position += token.len();
      true
    } else {
      false
    }
  }
  fn expression(&mut self, minimum: u8) -> Result<u64, Error> {
    self.depth += 1;
    if self.depth > 64 {
      return Err(Error::Limit);
    }
    let mut left = if self.take(b"NOT") || self.take(b"~") {
      !self.expression(7)?
    } else if self.take(b"-") {
      self.expression(7)?.wrapping_neg()
    } else if self.take(b"+") {
      self.expression(7)?
    } else if self.take(b"(") {
      let value = self.expression(0)?;
      if !self.take(b")") {
        return Err(Error::Syntax);
      }
      value
    } else {
      self.number()?
    };
    left &= self.options.mask();
    loop {
      self.space();
      let remaining = &self.input[self.position..];
      let operator = [
        ("OR", 1),
        ("|", 1),
        ("XOR", 2),
        ("^", 2),
        ("AND", 3),
        ("&", 3),
        ("<<", 4),
        (">>", 4),
        ("+", 5),
        ("-", 5),
        ("*", 6),
        ("/", 6),
        ("%", 6),
      ]
      .into_iter()
      .find(|(token, _)| remaining.starts_with(token.as_bytes()));
      let Some((operator, binding)) = operator else {
        break;
      };
      if binding < minimum {
        break;
      }
      self.position += operator.len();
      let right = self.expression(binding + 1)?;
      left = match operator {
        "OR" | "|" => left | right,
        "XOR" | "^" => left ^ right,
        "AND" | "&" => left & right,
        "+" => left.wrapping_add(right),
        "-" => left.wrapping_sub(right),
        "*" => left.wrapping_mul(right),
        "/" | "%" => {
          if right == 0 {
            return Err(Error::Domain);
          }
          let left = self.options.signed_value(left);
          let right = self.options.signed_value(right);
          let value = if operator == "/" {
            left / right
          } else {
            left % right
          };
          // Mask the signed result before conversion, including MIN / -1.
          u64::try_from(value & i128::from(self.options.mask())).map_err(|_| Error::Domain)?
        }
        "<<" | ">>" => {
          if right >= u64::from(self.options.bits) {
            return Err(Error::Domain);
          }
          if operator == "<<" {
            left << right
          } else if self.options.signed {
            u64::try_from(
              (self.options.signed_value(left) >> right) & i128::from(self.options.mask()),
            )
            .map_err(|_| Error::Domain)?
          } else {
            left >> right
          }
        }
        _ => return Err(Error::Syntax),
      } & self.options.mask();
    }
    self.depth -= 1;
    Ok(left)
  }
  fn number(&mut self) -> Result<u64, Error> {
    self.space();
    let mut radix = self.options.base.radix();
    for (prefix, base) in [
      (b"0x", 16),
      (b"0X", 16),
      (b"0b", 2),
      (b"0B", 2),
      (b"0o", 8),
      (b"0O", 8),
    ] {
      if self.input[self.position..].starts_with(prefix) {
        radix = base;
        self.position += 2;
        break;
      }
    }
    let start = self.position;
    let mut value = 0u64;
    while let Some(&byte) = self.input.get(self.position) {
      let digit = match byte {
        b'0'..=b'9' => u32::from(byte - b'0'),
        b'a'..=b'f' => u32::from(byte - b'a' + 10),
        b'A'..=b'F' => u32::from(byte - b'A' + 10),
        _ => break,
      };
      if digit >= radix {
        return Err(Error::Syntax);
      }
      value = value
        .wrapping_mul(u64::from(radix))
        .wrapping_add(u64::from(digit))
        & self.options.mask();
      self.position += 1;
    }
    if start == self.position {
      Err(Error::Syntax)
    } else {
      Ok(value)
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn exact_large_values_and_precedence() {
    let options = Options::default();
    for (input, result) in [
      ("9007199254740993+2", "9007199254740995"),
      ("0xFF AND 0b1010", "10"),
      ("2+3*4", "14"),
      ("1<<3+1", "16"),
      ("NOT 0", "-1"),
      ("-7/2", "-3"),
    ] {
      assert_eq!(evaluate(input, options).unwrap(), result, "{input}");
    }
  }
  #[test]
  fn words_wrap_and_sign_extend() {
    for bits in [8, 16, 32, 64] {
      let options = Options {
        bits,
        ..Options::default()
      };
      assert_eq!(evaluate("NOT 0+1", options).unwrap(), "0");
      assert_eq!(evaluate("-2>>1", options).unwrap(), "-1");
    }
    let options = Options {
      base: Base::Hexadecimal,
      bits: 8,
      signed: false,
    };
    assert_eq!(evaluate("FF+1", options).unwrap(), "0");
    assert_eq!(evaluate("-1", options).unwrap(), "FF");
  }
  #[test]
  fn every_word_width_and_base_preserves_the_bit_pattern() {
    let mut value = 0x1234_5678_9abc_def0u64;
    for bits in [8, 16, 32, 64] {
      for base in [Base::Binary, Base::Octal, Base::Decimal, Base::Hexadecimal] {
        for signed in [false, true] {
          let options = Options { base, bits, signed };
          for _ in 0..100 {
            value = value.wrapping_mul(6364136223846793005).wrapping_add(1);
            let formatted = options.format(value);
            assert_eq!(word(&formatted, options).unwrap(), value & options.mask());
          }
        }
      }
    }
    let options = Options::default();
    assert_eq!(
      evaluate("-9223372036854775808/-1", options).unwrap(),
      "-9223372036854775808"
    );
    assert_eq!(evaluate("0xAA XOR 0xFF", options).unwrap(), "85");
    assert_eq!(
      evaluate(
        "18446744073709551615",
        Options {
          signed: false,
          ..options
        }
      )
      .unwrap(),
      "18446744073709551615"
    );
  }
  #[test]
  fn rejects_invalid() {
    for input in ["", "1/0", "1%0", "1<<64", "1<<-1", "2(3)", "0b2", "1.2"] {
      assert!(evaluate(input, Options::default()).is_err(), "{input}");
    }
    assert_eq!(
      evaluate(&"(".repeat(70), Options::default()),
      Err(Error::Limit)
    );
  }
}
