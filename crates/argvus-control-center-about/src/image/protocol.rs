//! Image-backend protocol of the About page.
use image::DynamicImage;
use ratatui::layout::Size;
use ratatui_image::Resize;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::Protocol;

pub struct Backend {
  pub picker: Picker,
  pub protocol: Protocol,
  pub protocol_type: ProtocolType,
  pub target: Size,
}

pub fn build(picker: Picker, source: &image::RgbaImage, target: Size) -> Option<Backend> {
  let protocol_type = picker.protocol_type();
  if matches!(
    protocol_type,
    ProtocolType::Sixel | ProtocolType::Kitty | ProtocolType::Iterm2
  ) {
    let protocol = picker
      .new_protocol(
        DynamicImage::ImageRgba8(source.clone()),
        target,
        Resize::Fit(None),
      )
      .ok()?;
    Some(Backend {
      picker,
      protocol,
      protocol_type,
      target,
    })
  } else {
    None
  }
}

pub fn rebuild_target(backend: &mut Backend, source: &image::RgbaImage, target: Size) -> bool {
  if target == backend.target {
    return false;
  }
  let Ok(protocol) = backend.picker.new_protocol(
    DynamicImage::ImageRgba8(source.clone()),
    target,
    Resize::Fit(None),
  ) else {
    return false;
  };
  backend.protocol = protocol;
  backend.target = target;
  true
}
