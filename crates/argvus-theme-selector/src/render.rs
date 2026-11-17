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
  let accent = argvus_config_core::ConfigStore::from_environment()
    .and_then(|store| store.load_effective())
    .ok()
    .and_then(|document| {
      document
        .get("/appearance/accent")
        .and_then(|value| value.as_str())
        .and_then(Color::parse)
    })
    .unwrap_or_else(|| token("accent", Color::opaque(53, 144, 189)));
  let selection = accent.hex();
  let radius = if spaced { "6px" } else { "0" };
  format!(
    "window.theme-selector {{ background: {bg}; color: {fg}; font-size: 13px; }} .theme-selector * {{ border-radius: {radius}; box-shadow: none; text-shadow: none; }} .theme-selector button, .theme-selector entry, .theme-selector dropdown {{ min-height: 38px; padding: 0 12px; border: 1px solid alpha({fg}, 0.22); background: alpha({fg}, 0.045); color: {fg}; }} .theme-selector button:hover, .theme-selector entry:focus-within, .theme-selector searchentry:focus-within, .theme-selector dropdown:focus {{ border-color: {}; }} .theme-selector button:focus-visible {{ outline: 2px solid {}; outline-offset: 2px; }} .theme-selector button:checked {{ border-color: {selection}; background: alpha({selection}, 0.18); }} .theme-selector button:hover {{ background: alpha({fg}, 0.10); }} .theme-selector .title {{ font-size: 20px; font-weight: bold; }} .theme-selector .muted {{ opacity: 0.72; }} .theme-selector .suggested-action {{ background: {}; color: {}; border-color: {}; font-weight: bold; }} .theme-selector .suggested-action:hover {{ background: {}; }} .theme-selector popover, .theme-selector popover contents {{ background: {bg}; color: {fg}; border: 1px solid alpha({fg}, 0.3); }} .theme-selector .applying-preview {{ padding: 0; background: alpha(black, 0.72); color: white; border: none; }} .theme-selector dropdown {{ padding: 0; border: none; background: transparent; }} .theme-selector dropdown > button {{ min-height: 38px; }} .theme-selector searchentry:focus-within {{ outline: none; border-color: {}; }} .theme-selector searchentry:focus-within * {{ outline: none; box-shadow: none; }} .theme-selector separator {{ background: alpha({fg}, 0.16); }} .theme-selector button.mode-choice {{ padding: 8px; }} .theme-selector button.mode-choice.mode-selected {{ border-color: {selection}; background: alpha({selection}, 0.18); }} .theme-selector button.mode-choice.mode-cursor {{ outline: 2px solid {}; outline-offset: 2px; }}",
    accent.hex(),
    accent.hex(),
    accent.hex(),
    accent.contrast().hex(),
    accent.hex(),
    accent.hex(),
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

/// A layout comparison uses the same wallpaper and palette, with distinct geometry.
pub fn paint_mode(
  context: &cairo::Context,
  theme: &ThemeManifest,
  image: Option<&cairo::ImageSurface>,
  spaced: bool,
) {
  paint_preset(context, theme, image, spaced, false, false);
}

/// Draw each edge explicitly: moving bars must never flip the wallpaper.
pub fn paint_preset(
  context: &cairo::Context,
  theme: &ThemeManifest,
  image: Option<&cairo::ImageSurface>,
  spaced: bool,
  floating: bool,
  bottom: bool,
) {
  paint(context, theme, image, false, spaced);
  let bg = theme.token("bg").unwrap_or(Color::opaque(20, 22, 28));
  let fg = theme.token("fg").unwrap_or(Color::opaque(235, 235, 235));
  let accent = theme.token("accent").unwrap_or(fg);
  let margin = if spaced { 24.0 } else { 3.0 };
  let gap = if spaced { 18.0 } else { 3.0 };
  let radius = if spaced { 8.0 } else { 0.0 };
  let bar_height = 30.0;
  let main_y = if bottom {
    540.0 - margin - bar_height
  } else {
    margin
  };
  illustration_panel(
    context,
    (margin, main_y, 960.0 - 2.0 * margin, bar_height),
    radius,
    [bg, fg, accent],
    PanelKind::Taskbar,
  );
  if floating {
    let opposite_y = if bottom {
      margin
    } else {
      540.0 - margin - bar_height
    };
    illustration_panel(
      context,
      (margin, opposite_y, 960.0 - 2.0 * margin, bar_height),
      radius,
      [bg, fg, accent],
      PanelKind::ApplicationBar,
    );
    illustration_panel(
      context,
      (90.0, 86.0, 510.0, 328.0),
      radius,
      [bg, fg, accent],
      PanelKind::Window,
    );
    illustration_panel(
      context,
      (400.0, 180.0, 450.0, 275.0),
      radius,
      [bg, fg, accent],
      PanelKind::Window,
    );
  } else {
    let window_y = if bottom {
      margin
    } else {
      margin + bar_height + gap
    };
    let window_height = 540.0 - 2.0 * margin - bar_height - gap;
    let left_width = (960.0 - 2.0 * margin - gap) * 0.6;
    illustration_panel(
      context,
      (margin, window_y, left_width, window_height),
      radius,
      [bg, fg, accent],
      PanelKind::Window,
    );
    illustration_panel(
      context,
      (
        margin + left_width + gap,
        window_y,
        (960.0 - 2.0 * margin - gap) * 0.4,
        window_height,
      ),
      radius,
      [bg, fg, accent],
      PanelKind::Window,
    );
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PanelKind {
  Taskbar,
  ApplicationBar,
  Window,
}

fn illustration_panel(
  context: &cairo::Context,
  bounds: (f64, f64, f64, f64),
  radius: f64,
  colors: [Color; 3],
  kind: PanelKind,
) {
  let (x, y, width, height) = bounds;
  let [bg, fg, accent] = colors;
  rounded(context, x, y, width, height, radius);
  context.set_source_rgb(
    f64::from(bg.red) / 255.0,
    f64::from(bg.green) / 255.0,
    f64::from(bg.blue) / 255.0,
  );
  let _ = context.fill_preserve();
  context.set_source_rgb(
    f64::from(accent.red) / 255.0,
    f64::from(accent.green) / 255.0,
    f64::from(accent.blue) / 255.0,
  );
  context.set_line_width(2.0);
  let _ = context.stroke();
  if kind == PanelKind::Taskbar {
    for index in 0..6 {
      fill(
        context,
        accent,
        x + 16.0 + f64::from(index) * 32.0,
        y + 10.0,
        18.0,
        10.0,
      );
    }
    fill(
      context,
      fg.with_opacity(0.5),
      x + width - 110.0,
      y + 12.0,
      90.0,
      5.0,
    );
  } else if kind == PanelKind::ApplicationBar {
    for index in 0..6 {
      fill(
        context,
        accent,
        x + width / 2.0 - 110.0 + f64::from(index) * 40.0,
        y + 8.0,
        24.0,
        14.0,
      );
    }
  } else {
    // Window controls stay on the top edge for both taskbar positions.
    for index in 0..3 {
      fill(
        context,
        accent,
        x + width - 60.0 + f64::from(index) * 16.0,
        y + 12.0,
        8.0,
        8.0,
      );
    }
    fill(context, accent, x + 16.0, y + 14.0, width * 0.28, 4.0);
    for row in 0..((height - 48.0) / 20.0) as usize {
      fill(
        context,
        if row % 4 == 0 {
          accent
        } else {
          fg.with_opacity(0.5)
        },
        x + 16.0,
        y + 42.0 + row as f64 * 20.0,
        (width - 32.0) * (0.35 + (row % 4) as f64 * 0.12),
        4.0,
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn mode_previews_have_distinct_window_geometry() {
    std::fs::create_dir_all("/tmp/argvus").unwrap();
    let root = tempfile::tempdir_in("/tmp/argvus").unwrap();
    std::fs::write(
      root.path().join("theme.toml"),
      r##"schema = 2
id = "demo"
name = "Demo"
category = "dark"
[palette]
bg = "#112233"
fg = "#ffffff"
accent = "#dd9900"
"##,
    )
    .unwrap();
    let theme = ThemeManifest::load(root.path()).unwrap();
    let wallpaper = cairo::ImageSurface::create(cairo::Format::ARgb32, 960, 540).unwrap();
    {
      let context = cairo::Context::new(&wallpaper).unwrap();
      context.set_source_rgb(1.0, 0.0, 0.0);
      context.rectangle(0.0, 0.0, 960.0, 270.0);
      context.fill().unwrap();
      context.set_source_rgb(0.0, 0.0, 1.0);
      context.rectangle(0.0, 270.0, 960.0, 270.0);
      context.fill().unwrap();
    }
    let pixels = |spaced, floating, bottom| {
      let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 960, 540).unwrap();
      {
        let context = cairo::Context::new(&surface).unwrap();
        paint_preset(&context, &theme, Some(&wallpaper), spaced, floating, bottom);
      }
      surface.data().unwrap().to_vec()
    };
    let sticky = pixels(false, false, false);
    let spaced = pixels(true, false, false);
    let bottom = pixels(true, false, true);
    let wallpaper_sample = (100 * 960 + 10) * 4;
    assert_eq!(
      &spaced[wallpaper_sample..wallpaper_sample + 4],
      &bottom[wallpaper_sample..wallpaper_sample + 4]
    );
    let mut previews = Vec::new();
    for spaced in [false, true] {
      for floating in [false, true] {
        for bottom in [false, true] {
          let preview = pixels(spaced, floating, bottom);
          assert!(!previews.contains(&preview));
          previews.push(preview);
        }
      }
    }
    assert_eq!(previews.len(), 8);
    let left_edge = (100 * 960 + 24) * 4;
    assert_ne!(
      &sticky[left_edge..left_edge + 4],
      &spaced[left_edge..left_edge + 4]
    );
  }

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
