//! Where the menu opens: the monitor under the pointer and the bottom edge of Waybar.

use super::*;

pub(super) struct PopupPosition {
  pub(super) monitor: Option<gdk::Monitor>,
  pub(super) x: i32,
  pub(super) y: i32,
  pub(super) width: i32,
  pub(super) height: i32,
  pub(super) origin_x: i32,
  pub(super) origin_y: i32,
}

pub(super) fn popup_position(
  device_count: usize,
  fixed_pos: Option<(i32, i32)>,
  bar_bottom: Option<i32>,
) -> PopupPosition {
  let (pointer_x, pointer_y) =
    fixed_pos.unwrap_or_else(|| current_pointer_position().unwrap_or((0, 0)));
  let monitor =
    gdk::Display::default().and_then(|display| monitor_at_point(&display, pointer_x, pointer_y));

  let Some(monitor_ref) = monitor.as_ref() else {
    return PopupPosition {
      monitor,
      x: SCREEN_MARGIN,
      y: 32,
      width: 1920,
      height: 1080,
      origin_x: 0,
      origin_y: 0,
    };
  };

  let geometry = monitor_ref.geometry();
  let local_x = pointer_x - geometry.x();
  let estimated_height = ((device_count as i32 + 3) * 32).max(128);
  let max_x = (geometry.width() - MENU_WIDTH - SCREEN_MARGIN).max(SCREEN_MARGIN);
  let max_y = (geometry.height() - estimated_height - SCREEN_MARGIN).max(SCREEN_MARGIN);
  let bar_bottom = bar_bottom
    .or_else(|| current_waybar_bottom(pointer_x, pointer_y))
    .unwrap_or(pointer_y);
  let popup_y = bar_bottom - geometry.y() + MENU_GAP_Y;

  PopupPosition {
    monitor,
    x: local_x.clamp(SCREEN_MARGIN, max_x),
    y: popup_y.clamp(SCREEN_MARGIN, max_y),
    width: geometry.width(),
    height: geometry.height(),
    origin_x: geometry.x(),
    origin_y: geometry.y(),
  }
}

/// The monitor whose geometry contains the point. GTK4 has no point lookup on
/// the display, so the monitor list is searched.
pub(super) fn monitor_at_point(display: &gdk::Display, x: i32, y: i32) -> Option<gdk::Monitor> {
  let monitors = display.monitors();
  (0..monitors.n_items()).find_map(|index| {
    let monitor = monitors.item(index)?.downcast::<gdk::Monitor>().ok()?;
    let geometry = monitor.geometry();
    let inside = x >= geometry.x()
      && x < geometry.x() + geometry.width()
      && y >= geometry.y()
      && y < geometry.y() + geometry.height();
    inside.then_some(monitor)
  })
}

pub(super) fn current_waybar_bottom(pointer_x: i32, pointer_y: i32) -> Option<i32> {
  let output = std::process::Command::new("hyprctl")
    .args(["layers", "-j"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let outputs = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
  outputs.as_object()?.values().find_map(|output| {
    output
      .get("levels")?
      .as_object()?
      .values()
      .flat_map(serde_json::Value::as_array)
      .flatten()
      .find_map(|layer| {
        let namespace = layer.get("namespace")?.as_str()?;
        let x = layer.get("x")?.as_i64()? as i32;
        let y = layer.get("y")?.as_i64()? as i32;
        let w = layer.get("w")?.as_i64()? as i32;
        let h = layer.get("h")?.as_i64()? as i32;
        let horizontal = namespace == "waybar" && w > h.saturating_mul(4);
        let contains = pointer_x >= x && pointer_x < x + w && pointer_y >= y && pointer_y < y + h;
        (horizontal && contains).then_some(y + h)
      })
  })
}

pub(super) fn current_pointer_position() -> Option<(i32, i32)> {
  let output = std::process::Command::new("hyprctl")
    .args(["cursorpos", "-j"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let value = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
  let x = value.get("x")?.as_i64()? as i32;
  let y = value.get("y")?.as_i64()? as i32;
  Some((x, y))
}
