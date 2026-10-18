//! Implements kitty OSC 66 text scaling for Material Design icons in crate `argvus tui`.
//!
//! This module wraps the crossterm backend to apply the kitty text sizing protocol
//! (OSC 66) to glyphs in the Material Design icon range (U+F0001..=U+F1AF0),
//! reducing their displayed size while maintaining layout.
//!
//! The scaling is applied only in kitty, and only if not inside a terminal multiplexer.
//! It can be overridden with the `ARGVUS_ICON_SCALE` environment variable.

use std::io::{self, Write};

use ratatui::backend::{Backend, ClearType, CrosstermBackend, WindowSize};
use ratatui::buffer::Cell;
use ratatui::prelude::Position;
use ratatui::style::Color;

/// Material Design icon range in Nerd Fonts.
const ICON_MIN: u32 = 0xF0001;
const ICON_MAX: u32 = 0xF1AF0;

/// OSC 66 text sizing: n/d means scale to n/d of normal size.
/// v=2 and h=2 center the scaled text vertically and horizontally.
fn osc_66_escape(glyph: char, numerator: u32, denominator: u32) -> String {
  format!(
    "\x1b]66;w=1:n={}:d={}:v=2:h=2;{}\x07",
    numerator, denominator, glyph
  )
}

/// Detects whether to apply icon scaling based on terminal and environment.
fn should_scale_icons() -> bool {
  // Check environment override first.
  match std::env::var("ARGVUS_ICON_SCALE").as_deref() {
    Ok("on") => return true,
    Ok("off") => return false,
    _ => {}
  }

  // Only in kitty, and not in multiplexers.
  let is_kitty = std::env::var("KITTY_WINDOW_ID").is_ok()
    || std::env::var("TERM").is_ok_and(|t| t == "xterm-kitty");
  let in_multiplexer = std::env::var("TMUX").is_ok() || std::env::var("STY").is_ok();

  is_kitty && !in_multiplexer
}

/// Parses numerator and denominator from ARGVUS_ICON_SCALE=<n>/<d>, with defaults.
fn parse_scale_ratio() -> (u32, u32) {
  match std::env::var("ARGVUS_ICON_SCALE") {
    Ok(s) if s.contains('/') => {
      let parts: Vec<&str> = s.split('/').collect();
      if let (Some(n_str), Some(d_str)) = (parts.first().copied(), parts.get(1).copied())
        && let (Ok(n), Ok(d)) = (n_str.parse::<u32>(), d_str.parse::<u32>())
        && n > 0
        && d > n
        && d <= 15
      {
        return (n, d);
      }
      (3, 4) // Default: 75%
    }
    _ => (3, 4), // Default: 75%
  }
}

/// Wraps a crossterm backend to apply OSC 66 scaling to Material Design icons.
pub struct IconScaleBackend {
  inner: CrosstermBackend<std::io::Stdout>,
  scale_enabled: bool,
  scale_numerator: u32,
  scale_denominator: u32,
}

impl IconScaleBackend {
  /// Constructs `new` with icon scaling detection and configuration.
  pub fn new(inner: CrosstermBackend<std::io::Stdout>) -> Self {
    let scale_enabled = should_scale_icons();
    let (n, d) = if scale_enabled {
      parse_scale_ratio()
    } else {
      (0, 0)
    };
    Self {
      inner,
      scale_enabled,
      scale_numerator: n,
      scale_denominator: d,
    }
  }

  /// Checks if a character is a Material Design icon.
  fn is_md_icon(&self, ch: char) -> bool {
    if !self.scale_enabled {
      return false;
    }
    let code = ch as u32;
    (ICON_MIN..=ICON_MAX).contains(&code)
  }

  /// Writes SGR attributes (colors and modifiers) and OSC 66 escape sequence for a cell.
  fn write_sgr_and_osc(&mut self, cell: &Cell, ch: char, n: u32, d: u32) -> io::Result<()> {
    // Write foreground color.
    match cell.fg {
      Color::Reset => self.write_all(b"\x1b[39m")?,
      Color::Black => self.write_all(b"\x1b[30m")?,
      Color::Red => self.write_all(b"\x1b[31m")?,
      Color::Green => self.write_all(b"\x1b[32m")?,
      Color::Yellow => self.write_all(b"\x1b[33m")?,
      Color::Blue => self.write_all(b"\x1b[34m")?,
      Color::Magenta => self.write_all(b"\x1b[35m")?,
      Color::Cyan => self.write_all(b"\x1b[36m")?,
      Color::White => self.write_all(b"\x1b[37m")?,
      Color::Gray => self.write_all(b"\x1b[90m")?,
      Color::DarkGray => self.write_all(b"\x1b[90m")?,
      Color::LightRed => self.write_all(b"\x1b[91m")?,
      Color::LightGreen => self.write_all(b"\x1b[92m")?,
      Color::LightYellow => self.write_all(b"\x1b[93m")?,
      Color::LightBlue => self.write_all(b"\x1b[94m")?,
      Color::LightMagenta => self.write_all(b"\x1b[95m")?,
      Color::LightCyan => self.write_all(b"\x1b[96m")?,
      Color::Rgb(r, g, b) => write!(self, "\x1b[38;2;{};{};{}m", r, g, b)?,
      Color::Indexed(idx) => write!(self, "\x1b[38;5;{}m", idx)?,
    }

    // Write background color.
    match cell.bg {
      Color::Reset => self.write_all(b"\x1b[49m")?,
      Color::Black => self.write_all(b"\x1b[40m")?,
      Color::Red => self.write_all(b"\x1b[41m")?,
      Color::Green => self.write_all(b"\x1b[42m")?,
      Color::Yellow => self.write_all(b"\x1b[43m")?,
      Color::Blue => self.write_all(b"\x1b[44m")?,
      Color::Magenta => self.write_all(b"\x1b[45m")?,
      Color::Cyan => self.write_all(b"\x1b[46m")?,
      Color::White => self.write_all(b"\x1b[47m")?,
      Color::Gray => self.write_all(b"\x1b[100m")?,
      Color::DarkGray => self.write_all(b"\x1b[100m")?,
      Color::LightRed => self.write_all(b"\x1b[101m")?,
      Color::LightGreen => self.write_all(b"\x1b[102m")?,
      Color::LightYellow => self.write_all(b"\x1b[103m")?,
      Color::LightBlue => self.write_all(b"\x1b[104m")?,
      Color::LightMagenta => self.write_all(b"\x1b[105m")?,
      Color::LightCyan => self.write_all(b"\x1b[106m")?,
      Color::Rgb(r, g, b) => write!(self, "\x1b[48;2;{};{};{}m", r, g, b)?,
      Color::Indexed(idx) => write!(self, "\x1b[48;5;{}m", idx)?,
    }

    // Write modifiers.
    if cell.modifier.contains(ratatui::style::Modifier::BOLD) {
      self.write_all(b"\x1b[1m")?;
    }
    if cell.modifier.contains(ratatui::style::Modifier::DIM) {
      self.write_all(b"\x1b[2m")?;
    }
    if cell.modifier.contains(ratatui::style::Modifier::ITALIC) {
      self.write_all(b"\x1b[3m")?;
    }
    if cell.modifier.contains(ratatui::style::Modifier::UNDERLINED) {
      self.write_all(b"\x1b[4m")?;
    }
    if cell.modifier.contains(ratatui::style::Modifier::SLOW_BLINK) {
      self.write_all(b"\x1b[5m")?;
    }
    if cell
      .modifier
      .contains(ratatui::style::Modifier::RAPID_BLINK)
    {
      self.write_all(b"\x1b[6m")?;
    }
    if cell.modifier.contains(ratatui::style::Modifier::REVERSED) {
      self.write_all(b"\x1b[7m")?;
    }
    if cell.modifier.contains(ratatui::style::Modifier::HIDDEN) {
      self.write_all(b"\x1b[8m")?;
    }
    if cell
      .modifier
      .contains(ratatui::style::Modifier::CROSSED_OUT)
    {
      self.write_all(b"\x1b[9m")?;
    }

    // Write OSC 66 scaling sequence.
    write!(self, "{}", osc_66_escape(ch, n, d))?;

    Ok(())
  }
}

impl Backend for IconScaleBackend {
  fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
  where
    I: Iterator<Item = (u16, u16, &'a Cell)>,
  {
    if !self.scale_enabled {
      // No scaling: delegate to inner backend.
      return self.inner.draw(content);
    }

    // Collect and separate MD icon cells from regular cells.
    let mut md_icons: Vec<(u16, u16, Cell)> = Vec::new();
    let mut regular_cells: Vec<(u16, u16, Cell)> = Vec::new();

    for (x, y, cell) in content {
      let symbol = cell.symbol();
      if symbol.chars().count() == 1 {
        let ch = symbol.chars().next().unwrap();
        if self.is_md_icon(ch) {
          md_icons.push((x, y, cell.clone()));
          continue;
        }
      }
      regular_cells.push((x, y, cell.clone()));
    }

    // Draw regular cells via inner backend first.
    if !regular_cells.is_empty() {
      self
        .inner
        .draw(regular_cells.iter().map(|(x, y, c)| (*x, *y, c)))?;
    }

    // Draw MD icons with OSC 66 scaling.
    if !md_icons.is_empty() {
      let n = self.scale_numerator;
      let d = self.scale_denominator;

      for (x, y, cell) in md_icons {
        let ch = cell.symbol().chars().next().unwrap();

        // Set cursor position.
        self.inner.set_cursor_position(Position::new(x, y))?;

        // Apply SGR attributes and write OSC 66 sequence.
        self.write_sgr_and_osc(&cell, ch, n, d)?;
      }
    }

    Ok(())
  }

  fn hide_cursor(&mut self) -> io::Result<()> {
    self.inner.hide_cursor()
  }

  fn show_cursor(&mut self) -> io::Result<()> {
    self.inner.show_cursor()
  }

  fn get_cursor_position(&mut self) -> io::Result<Position> {
    self.inner.get_cursor_position()
  }

  fn set_cursor_position<P>(&mut self, position: P) -> io::Result<()>
  where
    P: Into<Position>,
  {
    self.inner.set_cursor_position(position)
  }

  fn clear(&mut self) -> io::Result<()> {
    self.inner.clear()
  }

  fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
    self.inner.clear_region(clear_type)
  }

  fn append_lines(&mut self, n: u16) -> io::Result<()> {
    self.inner.append_lines(n)
  }

  fn window_size(&mut self) -> io::Result<WindowSize> {
    self.inner.window_size()
  }

  fn size(&self) -> io::Result<ratatui::prelude::Size> {
    self.inner.size()
  }

  fn flush(&mut self) -> io::Result<()> {
    Backend::flush(&mut self.inner)
  }

  type Error = io::Error;
}

impl Write for IconScaleBackend {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    Write::write(&mut self.inner, buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    Write::flush(&mut self.inner)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_scale_ratio_default() {
    unsafe { std::env::remove_var("ARGVUS_ICON_SCALE") };
    let (n, d) = parse_scale_ratio();
    assert_eq!((n, d), (3, 4));
  }

  #[test]
  fn test_parse_scale_ratio_custom() {
    unsafe { std::env::set_var("ARGVUS_ICON_SCALE", "2/3") };
    let (n, d) = parse_scale_ratio();
    assert_eq!((n, d), (2, 3));
    unsafe { std::env::remove_var("ARGVUS_ICON_SCALE") };
  }

  #[test]
  fn test_osc_66_escape() {
    let escape = osc_66_escape('A', 3, 4);
    assert!(escape.contains("66"));
    assert!(escape.contains("w=1"));
    assert!(escape.contains("n=3"));
    assert!(escape.contains("d=4"));
    assert!(escape.contains("v=2"));
    assert!(escape.contains("h=2"));
    assert!(escape.contains("A"));
  }
}
