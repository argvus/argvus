//! Theme-derived illustration and card geometry; never projects a theme build.
use argvus_theme_core::{color::Color, manifest::ThemeManifest};
use gtk::cairo;

pub fn paint(
  context: &cairo::Context,
  theme: &ThemeManifest,
  image: Option<&cairo::ImageSurface>,
  illustrative: bool,
  spaced: bool,
) {
  if spaced {
    rounded(context, 0.0, 0.0, 960.0, 540.0, 12.0);
  } else {
    context.rectangle(0.0, 0.0, 960.0, 540.0);
  }
  context.clip();
  let background = theme.token("bg").unwrap_or(Color::opaque(20, 22, 28));
  let foreground = theme.token("fg").unwrap_or(Color::opaque(235, 235, 235));
  let accent = theme.token("accent").unwrap_or(foreground);
  fill(context, background, 0.0, 0.0, 960.0, 540.0);
  if let Some(image) = image
    && context.save().is_ok()
  {
    let scale = (960.0 / f64::from(image.width())).max(540.0 / f64::from(image.height()));
    context.translate(
      (960.0 - f64::from(image.width()) * scale) / 2.0,
      (540.0 - f64::from(image.height()) * scale) / 2.0,
    );
    context.scale(scale, scale);
    if context.set_source_surface(image, 0.0, 0.0).is_ok() {
      let _ = context.paint();
    }
    let _ = context.restore();
  }
  if illustrative {
    fill(context, background, 16.0, 12.0, 928.0, 30.0);
    for index in 0..5 {
      fill(
        context,
        if index == 0 {
          accent
        } else {
          foreground.with_opacity(0.4)
        },
        30.0 + f64::from(index) * 24.0,
        22.0,
        14.0,
        10.0,
      );
    }
    fill(context, background, 40.0, 72.0, 520.0, 410.0);
    fill(context, accent, 40.0, 72.0, 520.0, 3.0);
    for row in 0..21 {
      fill(
        context,
        if row % 4 == 0 {
          accent
        } else {
          foreground.with_opacity(0.55)
        },
        62.0 + f64::from(row % 3) * 12.0,
        100.0 + f64::from(row) * 16.0,
        100.0 + f64::from((row * 53) % 340),
        5.0,
      );
    }
    fill(context, background, 588.0, 150.0, 330.0, 320.0);
    fill(
      context,
      foreground.with_opacity(0.15),
      588.0,
      150.0,
      330.0,
      32.0,
    );
    for row in 0..3 {
      for column in 0..4 {
        fill(
          context,
          accent.with_opacity(0.85),
          612.0 + f64::from(column) * 72.0,
          212.0 + f64::from(row) * 76.0,
          42.0,
          32.0,
        );
      }
    }
  }
  context.set_source_rgb(
    f64::from(accent.red) / 255.0,
    f64::from(accent.green) / 255.0,
    f64::from(accent.blue) / 255.0,
  );
  context.set_line_width(4.0);
  if spaced {
    rounded(context, 2.0, 2.0, 956.0, 536.0, 10.0);
  } else {
    context.rectangle(2.0, 2.0, 956.0, 536.0);
  }
  let _ = context.stroke();
}
fn fill(context: &cairo::Context, color: Color, x: f64, y: f64, width: f64, height: f64) {
  context.set_source_rgba(
    f64::from(color.red) / 255.0,
    f64::from(color.green) / 255.0,
    f64::from(color.blue) / 255.0,
    f64::from(color.alpha) / 255.0,
  );
  context.rectangle(x, y, width, height);
  let _ = context.fill();
}

fn rounded(context: &cairo::Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
  use std::f64::consts::{FRAC_PI_2, PI};
  context.new_sub_path();
  context.arc(x + width - radius, y + radius, radius, -FRAC_PI_2, 0.0);
  context.arc(
    x + width - radius,
    y + height - radius,
    radius,
    0.0,
    FRAC_PI_2,
  );
  context.arc(x + radius, y + height - radius, radius, FRAC_PI_2, PI);
  context.arc(x + radius, y + radius, radius, PI, PI + FRAC_PI_2);
  context.close_path();
}

pub fn css(theme: Option<&ThemeManifest>, spaced: bool) -> String {
  let token = |name: &str, fallback| {
    theme
      .and_then(|theme| theme.token(name).ok())
      .unwrap_or(fallback)
  };
  let bg = token("bg", Color::opaque(17, 19, 22))
    .with_opacity(0.94)
    .hex_with_alpha();
  let fg = token("fg", Color::opaque(238, 238, 238)).hex();
  let accent = token("accent", Color::opaque(53, 144, 189));
  let radius = if spaced { "6px" } else { "0" };
  format!(
    "window.theme-selector {{ background: {bg}; color: {fg}; font-family: monospace; font-size: 13px; }} .theme-selector * {{ border-radius: {radius}; box-shadow: none; text-shadow: none; }} .theme-selector button, .theme-selector entry, .theme-selector dropdown {{ min-height: 38px; padding: 0 12px; border: 1px solid alpha({fg}, 0.22); background: alpha({fg}, 0.045); color: {fg}; }} .theme-selector button:hover, .theme-selector entry:focus, .theme-selector dropdown:focus {{ border-color: {}; }} .theme-selector button:focus-visible {{ outline: 2px solid {}; outline-offset: 2px; }} .theme-selector button:hover {{ background: alpha({fg}, 0.10); }} .theme-selector .title {{ font-size: 20px; font-weight: bold; }} .theme-selector .muted {{ opacity: 0.72; }} .theme-selector .suggested-action {{ background: {}; color: {}; border-color: {}; font-weight: bold; }} .theme-selector .suggested-action:hover {{ background: {}; }} .theme-selector popover, .theme-selector popover contents {{ background: {bg}; color: {fg}; border: 1px solid alpha({fg}, 0.3); }} .theme-selector separator {{ background: alpha({fg}, 0.16); }}",
    accent.hex(),
    accent.hex(),
    accent.hex(),
    accent.contrast().hex(),
    accent.hex(),
    accent.hex()
  )
}

#[derive(Debug)]
pub struct Card {
  pub offset: isize,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

/// Paint order is back to front; pointer selection uses the reverse order.
pub fn cards(width: f64, height: f64, count: usize) -> Vec<Card> {
  if count == 0 || width <= 0.0 || height <= 0.0 {
    return Vec::new();
  }
  let center_width = center_width(width, height);
  let center_height = center_width * 9.0 / 16.0;
  let center_x = (width - center_width) / 2.0;
  let mut positions = Vec::new();
  if count > 2 {
    positions.push((-1, center_x - center_width * 0.62, 0.76));
  }
  if count > 1 {
    positions.push((1, center_x + center_width * 0.86, 0.76));
  }
  positions.push((0, center_x, 1.0));
  positions
    .into_iter()
    .map(|(offset, x, scale)| Card {
      offset,
      x,
      y: (height - center_height * scale) / 2.0,
      width: center_width * scale,
      height: center_height * scale,
    })
    .collect()
}

pub fn center_width(width: f64, height: f64) -> f64 {
  (width * 0.62).min(840.0).min(height * 16.0 / 9.0)
}

pub fn hit_test(width: f64, height: f64, count: usize, x: f64, y: f64) -> Option<isize> {
  cards(width, height, count)
    .into_iter()
    .rev()
    .find(|card| {
      x >= card.x && x <= card.x + card.width && y >= card.y && y <= card.y + card.height
    })
    .map(|card| card.offset)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn central_card_wins_over_neighbors_and_empty_space_cancels() {
    assert!(cards(1920.0, 480.0, 0).is_empty());
    for count in [1, 2, 3, 50] {
      assert_eq!(cards(1920.0, 480.0, count).len(), count.min(3));
      assert_eq!(hit_test(1920.0, 480.0, count, 960.0, 240.0), Some(0));
    }
    assert_eq!(hit_test(1920.0, 480.0, 3, 300.0, 240.0), Some(-1));
    assert_eq!(hit_test(1920.0, 480.0, 2, 300.0, 240.0), None);
    assert_eq!(hit_test(1920.0, 480.0, 3, 1600.0, 240.0), Some(1));
    assert_eq!(hit_test(1920.0, 480.0, 3, 960.0, 0.0), None);
    assert_eq!(cards(640.0, 160.0, 3).last().unwrap().height, 160.0);
    for (width, height) in [(1920.0, 480.0), (1280.0, 720.0), (720.0, 320.0)] {
      assert_eq!(
        cards(width, height, 3).last().unwrap().width,
        center_width(width, height)
      );
    }
  }

  #[test]
  fn sticky_and_spaced_styles_have_distinct_corner_geometry() {
    assert!(css(None, false).contains("border-radius: 0"));
    assert!(css(None, true).contains("border-radius: 6px"));
  }
}
