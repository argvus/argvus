use image::{DynamicImage, RgbaImage, imageops};
use ratatui::{
  Frame,
  layout::{Rect, Size},
  style::{Color, Style},
  text::{Line, Span},
  widgets::Paragraph,
};
use ratatui_image::{
  Image, Resize,
  picker::{Picker, ProtocolType},
  protocol::Protocol,
};
use resvg::usvg::{Options, Tree};
use std::path::Path;

pub struct ImageSurface {
  source: RgbaImage,
  backend: Option<Backend>,
  graphics_attempted: bool,
}

struct Backend {
  protocol: Protocol,
  target: Size,
}

impl ImageSurface {
  pub fn from_path(path: &Path) -> Option<Self> {
    let source = load_path(path)?;
    Some(Self::from_source(source))
  }

  pub fn from_bytes(bytes: &[u8], svg: bool) -> Option<Self> {
    let source = if svg {
      load_svg(bytes)?
    } else {
      image::load_from_memory(bytes).ok()?.into_rgba8()
    };
    Some(Self::from_source(source))
  }

  fn from_source(source: RgbaImage) -> Self {
    Self {
      source,
      backend: None,
      graphics_attempted: false,
    }
  }

  pub fn render(&mut self, frame: &mut Frame, area: Rect, background: Color, accent: Color) {
    if area.width == 0 || area.height == 0 {
      return;
    }
    let target = Size::new(area.width, area.height);
    if self
      .backend
      .as_ref()
      .is_some_and(|backend| backend.target != target)
    {
      self.backend = None;
      self.graphics_attempted = false;
    }
    if self.backend.is_none() && !self.graphics_attempted {
      self.graphics_attempted = true;
      self.backend = graphics_backend(&self.source, target);
    }
    if let Some(backend) = self.backend.as_ref() {
      frame.render_widget(Image::new(&backend.protocol).allow_clipping(true), area);
    } else {
      frame.render_widget(
        Paragraph::new(fallback_lines(&self.source, area, background, accent)),
        area,
      );
    }
  }
}

fn graphics_backend(source: &RgbaImage, target: Size) -> Option<Backend> {
  let picker = Picker::from_query_stdio().ok()?;
  if !matches!(
    picker.protocol_type(),
    ProtocolType::Kitty | ProtocolType::Iterm2 | ProtocolType::Sixel
  ) {
    return None;
  }
  let protocol = picker
    .new_protocol(
      DynamicImage::ImageRgba8(source.clone()),
      target,
      Resize::Fit(None),
    )
    .ok()?;
  Some(Backend { protocol, target })
}

fn load_path(path: &Path) -> Option<RgbaImage> {
  if path
    .extension()
    .and_then(|value| value.to_str())
    .is_some_and(|value| value.eq_ignore_ascii_case("svg"))
  {
    return load_svg(&std::fs::read(path).ok()?);
  }
  image::ImageReader::open(path)
    .ok()?
    .with_guessed_format()
    .ok()?
    .decode()
    .ok()
    .map(DynamicImage::into_rgba8)
}

fn load_svg(data: &[u8]) -> Option<RgbaImage> {
  let tree = Tree::from_data(data, &Options::default()).ok()?;
  let size = tree.size();
  if size.width() <= 0.0 || size.height() <= 0.0 {
    return None;
  }
  let scale = (512.0 / size.width().max(size.height())).min(1.0);
  let width = (size.width() * scale).round().max(1.0) as u32;
  let height = (size.height() * scale).round().max(1.0) as u32;
  let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
  resvg::render(
    &tree,
    resvg::tiny_skia::Transform::from_scale(scale, scale),
    &mut pixmap.as_mut(),
  );
  let mut image = RgbaImage::new(width, height);
  for (index, pixel) in pixmap.data().chunks_exact(4).enumerate() {
    image.put_pixel(
      (index % width as usize) as u32,
      (index / width as usize) as u32,
      image::Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]),
    );
  }
  Some(image)
}

fn fallback_lines(
  source: &RgbaImage,
  area: Rect,
  background: Color,
  accent: Color,
) -> Vec<Line<'static>> {
  let width = usize::from(area.width.max(1));
  let height = usize::from(area.height.max(1));
  let resized = imageops::resize(
    source,
    width as u32,
    (height * 2) as u32,
    imageops::FilterType::Triangle,
  );
  (0..height)
    .map(|row| {
      let spans = (0..width)
        .map(|column| {
          let top = resized.get_pixel(column as u32, (row * 2) as u32).0;
          let bottom = resized.get_pixel(column as u32, (row * 2 + 1) as u32).0;
          let alpha = u16::from(top[3]).max(u16::from(bottom[3]));
          if alpha < 16 {
            Span::styled(" ", Style::new().bg(background))
          } else {
            Span::styled("▀", Style::new().fg(accent).bg(background))
          }
        })
        .collect::<Vec<_>>();
      Line::from(spans)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
  use std::io::Cursor;

  #[test]
  fn decodes_extensionless_png_files() {
    let mut bytes = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255])))
      .write_to(&mut bytes, ImageFormat::Png)
      .expect("encode test PNG");

    let path = std::env::temp_dir().join(format!(
      "argvus-tui-avatar-{}-{}",
      std::process::id(),
      std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&path, bytes.into_inner()).expect("write test PNG");

    let image = ImageSurface::from_path(&path).expect("decode extensionless PNG");
    assert_eq!(image.source.dimensions(), (2, 2));
    std::fs::remove_file(path).expect("remove test PNG");
  }
}
