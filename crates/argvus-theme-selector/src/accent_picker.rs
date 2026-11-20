//! Layer-shell editor for a single opaque accent color.

use std::{cell::Cell, rc::Rc};

use argvus_i18n::{Lang, TextDirection, tr};
use argvus_theme_core::color::Color;
use gtk::{cairo, gdk, glib, prelude::*};

#[derive(Clone, Copy)]
struct Hsv {
  hue: f64,
  saturation: f64,
  value: f64,
}

impl Hsv {
  fn from_color(color: Color, previous_hue: f64) -> Self {
    let [red, green, blue] =
      [color.red, color.green, color.blue].map(|byte| f64::from(byte) / 255.0);
    let value = red.max(green).max(blue);
    let spread = value - red.min(green).min(blue);
    let hue = if spread == 0.0 {
      previous_hue
    } else if value == red {
      ((green - blue) / spread).rem_euclid(6.0) / 6.0
    } else if value == green {
      ((blue - red) / spread + 2.0) / 6.0
    } else {
      ((red - green) / spread + 4.0) / 6.0
    };
    let saturation = if value == 0.0 { 0.0 } else { spread / value };
    Self {
      hue,
      saturation,
      value,
    }
  }

  fn rgb(self) -> (f64, f64, f64) {
    let sector = self.hue.rem_euclid(1.0) * 6.0;
    let chroma = self.value * self.saturation;
    let mid = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match sector as u8 {
      0 => (chroma, mid, 0.0),
      1 => (mid, chroma, 0.0),
      2 => (0.0, chroma, mid),
      3 => (0.0, mid, chroma),
      4 => (mid, 0.0, chroma),
      _ => (chroma, 0.0, mid),
    };
    let floor = self.value - chroma;
    (red + floor, green + floor, blue + floor)
  }

  fn hex(self) -> String {
    let (red, green, blue) = self.rgb();
    let byte = |channel: f64| (channel * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", byte(red), byte(green), byte(blue))
  }
}

fn parse(text: &str) -> Option<Color> {
  let text = text.trim();
  let digits = text.strip_prefix('#').unwrap_or(text);
  (digits.len() == 6)
    .then(|| Color::parse(&format!("#{digits}")))
    .flatten()
}

#[derive(Clone, Copy)]
enum Control {
  Hue,
  Plane,
}

impl Control {
  fn pointer(self, mut hsv: Hsv, x: f64, y: f64) -> Hsv {
    match self {
      Self::Hue => hsv.hue = y.clamp(0.0, 1.0),
      Self::Plane => {
        hsv.value = x.clamp(0.0, 1.0);
        hsv.saturation = 1.0 - y.clamp(0.0, 1.0);
      }
    }
    hsv
  }

  fn key(self, hsv: Hsv, key: gdk::Key) -> Option<Hsv> {
    const STEP: f64 = 0.01;
    let x = hsv.value;
    let y = match self {
      Self::Hue => hsv.hue,
      Self::Plane => 1.0 - hsv.saturation,
    };
    match key {
      gdk::Key::Up => Some(self.pointer(hsv, x, y - STEP)),
      gdk::Key::Down => Some(self.pointer(hsv, x, y + STEP)),
      gdk::Key::Left if matches!(self, Self::Plane) => Some(self.pointer(hsv, x - STEP, y)),
      gdk::Key::Right if matches!(self, Self::Plane) => Some(self.pointer(hsv, x + STEP, y)),
      _ => None,
    }
  }

  fn draw(self, cr: &cairo::Context, width: f64, height: f64, hsv: Hsv) {
    let vertical = cairo::LinearGradient::new(0.0, 0.0, 0.0, height);
    match self {
      Self::Hue => {
        for sector in 0..=6 {
          let hue = f64::from(sector) / 6.0;
          let (red, green, blue) = Hsv {
            hue,
            saturation: 1.0,
            value: 1.0,
          }
          .rgb();
          vertical.add_color_stop_rgb(hue, red, green, blue);
        }
        let _ = cr.set_source(&vertical);
        let _ = cr.paint();
        marker(cr, width / 2.0, hsv.hue * height, width, height);
      }
      Self::Plane => {
        let (red, green, blue) = Hsv {
          saturation: 1.0,
          value: 1.0,
          ..hsv
        }
        .rgb();
        vertical.add_color_stop_rgb(0.0, red, green, blue);
        vertical.add_color_stop_rgb(1.0, 1.0, 1.0, 1.0);
        let _ = cr.set_source(&vertical);
        let _ = cr.paint();
        let horizontal = cairo::LinearGradient::new(0.0, 0.0, width, 0.0);
        horizontal.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 1.0);
        horizontal.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 0.0);
        let _ = cr.set_source(&horizontal);
        let _ = cr.paint();
        marker(
          cr,
          hsv.value * width,
          (1.0 - hsv.saturation) * height,
          width,
          height,
        );
      }
    }
  }
}

fn marker(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64) {
  cr.arc(
    x.clamp(0.0, width),
    y.clamp(0.0, height),
    6.0,
    0.0,
    std::f64::consts::TAU,
  );
  cr.set_source_rgb(0.0, 0.0, 0.0);
  cr.set_line_width(4.0);
  let _ = cr.stroke_preserve();
  cr.set_source_rgb(1.0, 1.0, 1.0);
  cr.set_line_width(2.0);
  let _ = cr.stroke();
}

fn connect_control(
  area: &gtk::DrawingArea,
  control: Control,
  state: &Rc<Cell<Hsv>>,
  editor: &gtk::Entry,
) {
  let draw_state = state.clone();
  area.set_draw_func(move |_, cr, width, height| {
    control.draw(cr, f64::from(width), f64::from(height), draw_state.get());
  });
  let update = {
    let state = state.clone();
    let editor = editor.downgrade();
    move |color: Hsv| {
      if let Some(editor) = editor.upgrade() {
        state.set(color);
        let hex = color.hex();
        if editor.text() == hex {
          editor.emit_by_name::<()>("changed", &[]);
        } else {
          editor.set_text(&hex);
        }
        state.set(color);
      }
    }
  };
  let drag = gtk::GestureDrag::builder().button(1).build();
  let drag_state = state.clone();
  let drag_update = update.clone();
  let pointer = move |gesture: &gtk::GestureDrag, x: f64, y: f64| {
    if let Some(widget) = gesture.widget() {
      widget.grab_focus();
      let color = control.pointer(
        drag_state.get(),
        x / f64::from(widget.width().max(1)),
        y / f64::from(widget.height().max(1)),
      );
      drag_update(color);
    }
  };
  let drag_pointer = pointer.clone();
  drag.connect_drag_begin(move |gesture, x, y| drag_pointer(gesture, x, y));
  drag.connect_drag_update(move |gesture, dx, dy| {
    if let Some((x, y)) = gesture.start_point() {
      pointer(gesture, x + dx, y + dy);
    }
  });
  area.add_controller(drag);
  let keys = gtk::EventControllerKey::new();
  let state = state.clone();
  keys.connect_key_pressed(move |_, key, _, _| {
    if let Some(color) = control.key(state.get(), key) {
      update(color);
      glib::Propagation::Stop
    } else {
      glib::Propagation::Proceed
    }
  });
  area.add_controller(keys);
}

/// Displays the editor and returns an opaque hex color after confirmation.
pub fn choose(locale: &str, lang: Lang, initial: &str) -> Result<Option<String>, glib::BoolError> {
  gtk::init()?;
  gtk::Widget::set_default_direction(match lang.direction() {
    TextDirection::RightToLeft => gtk::TextDirection::Rtl,
    TextDirection::LeftToRight => gtk::TextDirection::Ltr,
  });
  let title = tr(lang, "accent.title");
  let window = crate::picker::window("argvus-accent", &format!("ARGVUS {title}"), locale);
  window.set_default_size(
    window.default_width().min(460),
    window.default_height().min(455),
  );
  let panel = crate::picker::panel(&window, title);
  panel.add_css_class("accent-editor");

  let color = parse(initial).unwrap_or_else(|| Color::opaque(53, 144, 189));
  let state = Rc::new(Cell::new(Hsv::from_color(color, 0.0)));
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  let sample = gtk::DrawingArea::builder()
    .content_width(65)
    .content_height(28)
    .build();
  let sample_state = state.clone();
  sample.set_draw_func(move |_, cr, _, _| {
    let (red, green, blue) = sample_state.get().rgb();
    cr.set_source_rgb(red, green, blue);
    let _ = cr.paint();
  });
  let editor = gtk::Entry::builder()
    .text(state.get().hex())
    .hexpand(true)
    .build();
  editor.set_direction(gtk::TextDirection::Ltr);
  editor.update_property(&[gtk::accessible::Property::Label(tr(lang, "accent.hex"))]);
  row.append(&sample);
  row.append(&editor);
  panel.append(&row);

  let controls = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  let hue = gtk::DrawingArea::builder()
    .content_width(24)
    .focusable(true)
    .build();
  let plane = gtk::DrawingArea::builder()
    .hexpand(true)
    .vexpand(true)
    .focusable(true)
    .build();
  for (area, control, key) in [
    (&hue, Control::Hue, "accent.hue"),
    (&plane, Control::Plane, "accent.saturation_value"),
  ] {
    area.update_property(&[gtk::accessible::Property::Label(tr(lang, key))]);
    connect_control(area, control, &state, &editor);
    controls.append(area);
  }
  panel.append(&controls);

  let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  footer.add_css_class("footer");
  let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  spacer.set_hexpand(true);
  footer.append(&spacer);
  let cancel = gtk::Button::with_label(tr(lang, "accent.cancel"));
  let select = gtk::Button::with_label(tr(lang, "accent.select"));
  select.add_css_class("suggested-action");
  footer.append(&cancel);
  footer.append(&select);
  panel.append(&footer);

  let selected = Rc::new(Cell::new(None));
  let running = glib::MainLoop::new(None, false);
  let stop = running.clone();
  cancel.connect_clicked(move |_| stop.quit());
  let stop = running.clone();
  window.connect_close_request(move |_| {
    stop.quit();
    glib::Propagation::Stop
  });
  let stop = running.clone();
  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  keys.connect_key_pressed(move |_, key, _, _| {
    if key != gdk::Key::Escape {
      return glib::Propagation::Proceed;
    }
    stop.quit();
    glib::Propagation::Stop
  });
  window.add_controller(keys);
  let stop = running.clone();
  let result = selected.clone();
  let input = editor.downgrade();
  select.connect_clicked(move |_| {
    if let Some(color) = input.upgrade().and_then(|entry| parse(&entry.text())) {
      result.set(Some(color));
      stop.quit();
    }
  });
  let accept = select.clone();
  editor.connect_activate(move |_| {
    if accept.is_sensitive() {
      accept.emit_clicked();
    }
  });
  editor.connect_changed(move |entry| {
    if let Some(color) = parse(&entry.text()) {
      entry.remove_css_class("error");
      select.set_sensitive(true);
      state.set(Hsv::from_color(color, state.get().hue));
      for widget in [&sample, &hue, &plane] {
        widget.queue_draw();
      }
    } else {
      entry.add_css_class("error");
      select.set_sensitive(false);
    }
  });

  window.present();
  editor.grab_focus();
  running.run();
  window.destroy();
  Ok(
    selected
      .take()
      .map(|color| color.hex().to_ascii_uppercase()),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hex_round_trips_through_hsv() {
    for text in [
      "#BF4A40", "#FF0000", "#00FF00", "#0000FF", "#FFFFFF", "#000000", "#129F7C",
    ] {
      let color = parse(text).unwrap();
      assert_eq!(Hsv::from_color(color, 0.0).hex(), text);
    }
    assert!(parse("#123ABCFF").is_none());
    assert!(parse("#abc").is_none());
    assert!(parse("#123agc").is_none());
    assert_eq!(parse(" 123ABC "), parse("#123ABC"));
  }
}
