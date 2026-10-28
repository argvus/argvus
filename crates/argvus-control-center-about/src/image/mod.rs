//! Module declarations for the `image` subsystem of the About page.
use std::env;

use image::RgbaImage;
use ratatui::Frame;
use ratatui::layout::{Rect, Size};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};
use ratatui_image::Image;
use ratatui_image::picker::ProtocolType;

use crate::theme::Theme;

pub mod fallback;
pub mod loader;
pub mod protocol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
  Graphics(ProtocolType),
  HalfBlocks,
  Ascii,
  Unavailable,
}

pub struct Logo {
  pub kind: BackendKind,
  source: Option<RgbaImage>,
  backend: Option<protocol::Backend>,
}

impl Logo {
  pub fn new() -> Self {
    Self::detect()
  }

  pub fn detect() -> Self {
    let source = loader::find_logo_path().and_then(|path| loader::load_svg(&path));

    let mut logo = Self {
      kind: BackendKind::Unavailable,
      source: source.clone(),
      backend: None,
    };

    if source.is_none() {
      return logo;
    }

    logo.kind = match env::var("ARGVUS_ABOUT_IMAGE").as_deref() {
      Ok("graphics") => {
        logo.backend = Self::try_graphics(160);
        logo.backend.as_ref().map_or_else(
          || {
            if color_supported() {
              BackendKind::HalfBlocks
            } else {
              BackendKind::Ascii
            }
          },
          |backend| BackendKind::Graphics(backend.protocol_type),
        )
      }
      Ok("halfblocks") | Ok("halfblocks-fallback") => BackendKind::HalfBlocks,
      Ok("ascii") => BackendKind::Ascii,
      _ => {
        if let Some(backend) = Self::try_graphics(160) {
          let kind = BackendKind::Graphics(backend.protocol_type);
          logo.backend = Some(backend);
          kind
        } else if color_supported() {
          BackendKind::HalfBlocks
        } else {
          BackendKind::Ascii
        }
      }
    };

    logo
  }

  fn try_graphics(pixel_size: u32) -> Option<protocol::Backend> {
    let picker = ratatui_image::picker::Picker::from_query_stdio().ok()?;
    let font = picker.font_size();
    let cols = (pixel_size / u32::from(font.width.max(1))).max(8) as u16;
    let rows = (pixel_size / u32::from(font.height.max(1))).max(8) as u16;
    let target = Size::new(cols, rows);
    let source = loader::find_logo_path().and_then(|path| loader::load_svg(&path))?;
    protocol::build(picker, &source, target)
  }

  pub fn is_visible(&self) -> bool {
    self.kind != BackendKind::Unavailable
  }

  pub fn is_graphics(&self) -> bool {
    matches!(self.kind, BackendKind::Graphics(_))
  }

  pub fn sync_target(&mut self, target: Size) {
    if !self.is_graphics() {
      return;
    }
    if let (Some(backend), Some(source)) = (self.backend.as_mut(), self.source.as_ref()) {
      let _ = protocol::rebuild_target(backend, source, target);
    }
  }

  pub fn render_graphics(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
    let Some(backend) = self.backend.as_ref() else {
      return;
    };
    let widget = Image::new(&backend.protocol).allow_clipping(true);
    if let Some(placeholder_area) = backend.protocol.needs_placeholder(area) {
      Block::default()
        .bg(theme.background)
        .render(placeholder_area, frame.buffer_mut());
    }
    frame.render_widget(widget, area);
  }

  pub fn lines(&self, cols: usize, rows: usize, theme: &Theme) -> Vec<Line<'static>> {
    let Some(source) = self.source.as_ref() else {
      return Vec::new();
    };
    match self.kind {
      BackendKind::HalfBlocks => fallback::half_blocks(source, cols, rows, theme.background),
      BackendKind::Ascii => fallback::ascii(source, cols, rows, theme.accent),
      _ => Vec::new(),
    }
  }
}

impl Default for Logo {
  fn default() -> Self {
    Self::new()
  }
}

fn color_supported() -> bool {
  if env::var("COLORTERM").as_deref() == Ok("truecolor") {
    return true;
  }
  env::var("TERM")
    .map(|term| term.contains("256color") || term.starts_with("xterm") || term.contains("kitty"))
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_color_support() {
    let _ = color_supported();
  }

  #[test]
  fn unavailable_logo_has_no_lines() {
    let logo = Logo {
      kind: BackendKind::Unavailable,
      source: None,
      backend: None,
    };
    assert!(!logo.is_visible());
    assert!(logo.lines(10, 10, &Theme::load()).is_empty());
  }
}
