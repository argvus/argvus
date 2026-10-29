//! Popup placement: pure functions plus the `hyprctl` queries that feed them.

use std::process::Command;

use serde_json::Value;

/// Gap between the bar and the popup, in pixels.
pub const POPUP_GAP_Y: i32 = 4;
/// Layer namespaces that identify the horizontal bar. The ARGVUS taskbar runs
/// Waybar with the `argvus-taskbar` namespace; plain Waybar uses `waybar`.
pub const WAYBAR_LAYER_NAMESPACES: [&str; 2] = ["argvus-taskbar", "waybar"];

/// `(x, y, width, height)` in desktop coordinates.
pub type PopupBounds = (i32, i32, i32, i32);
/// `(x, y, width, height, reserved_top)` of a monitor.
pub type MonitorGeometry = (i32, i32, i32, i32, i32);

/// Where the user clicked, as captured by the Waybar launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopupAnchor {
  pub x: i32,
  pub y: i32,
  pub bar_bottom: Option<i32>,
  pub bar_right: Option<i32>,
}

impl PopupAnchor {
  /// An anchor at a position with no bar geometry.
  pub fn at(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      bar_bottom: None,
      bar_right: None,
    }
  }

  /// The pointer position, when `hyprctl` can tell it.
  pub fn at_pointer() -> Option<Self> {
    current_pointer_position().map(|(x, y)| Self::at(x, y))
  }
}

/// The fixed width and the estimated height of a popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopupSize {
  pub width: i32,
  pub height_estimate: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaybarGeometry {
  pub top: i32,
  pub right: i32,
  pub bottom: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopupPosition {
  /// Open above the bar (bottom bar) instead of below it.
  pub above: bool,
  pub margin_bottom: i32,
  pub margin_top: i32,
  pub margin_right: i32,
  pub bounds: PopupBounds,
}

/// The popup's top-left corner, relative to its monitor.
pub fn popup_origin(
  bar_right: i32,
  bar_bottom: i32,
  monitor: MonitorGeometry,
  size: PopupSize,
) -> (i32, i32) {
  let (origin_x, origin_y, width, height, _) = monitor;
  let max_x = (width - size.width).max(0);
  let max_y = (height - size.height_estimate).max(0);
  let popup_x = (bar_right - origin_x - size.width).clamp(0, max_x);
  let popup_y = (bar_bottom - origin_y + POPUP_GAP_Y).clamp(0, max_y);
  (popup_x, popup_y)
}

/// Resolves the popup position for an anchor, asking `hyprctl` for the
/// monitor and the bar.
pub fn popup_position(anchor: Option<PopupAnchor>, size: PopupSize) -> PopupPosition {
  let anchor = anchor
    .or_else(PopupAnchor::at_pointer)
    .unwrap_or(PopupAnchor::at(0, 0));
  let monitor = current_monitor_geometry(anchor.x, anchor.y).unwrap_or((0, 0, 1920, 1080, 0));
  let waybar = current_waybar_geometry(anchor.x, anchor.y, monitor);
  resolve_position(anchor, monitor, waybar, size)
}

/// The pure part of [`popup_position`].
pub fn resolve_position(
  anchor: PopupAnchor,
  monitor: MonitorGeometry,
  waybar: Option<WaybarGeometry>,
  size: PopupSize,
) -> PopupPosition {
  let (origin_x, origin_y, width, height, reserved_top) = monitor;
  let bar_bottom = waybar
    .map(|geometry| geometry.bottom)
    .or(anchor.bar_bottom)
    .unwrap_or(origin_y + reserved_top);
  let bar_right = waybar
    .map(|geometry| geometry.right)
    .or(anchor.bar_right)
    .unwrap_or(origin_x + width);
  let (popup_x, popup_y) = popup_origin(bar_right, bar_bottom, monitor, size);
  let above = waybar.is_some_and(|bar| bar.top - origin_y > height / 2);
  let margin_bottom = waybar
    .filter(|_| above)
    .map_or(0, |bar| (origin_y + height - bar.top + POPUP_GAP_Y).max(0));

  PopupPosition {
    above,
    margin_bottom,
    margin_top: if above { 0 } else { popup_y.max(0) },
    margin_right: (width - popup_x - size.width).max(0),
    bounds: (
      origin_x + popup_x,
      origin_y
        + if above {
          height - margin_bottom - size.height_estimate
        } else {
          popup_y
        },
      size.width,
      size.height_estimate,
    ),
  }
}

pub fn current_waybar_geometry(
  pointer_x: i32,
  pointer_y: i32,
  monitor: MonitorGeometry,
) -> Option<WaybarGeometry> {
  let outputs = hyprctl_json(&["layers", "-j"])?;
  select_waybar(&outputs, pointer_x, pointer_y, monitor)
}

/// The horizontal bar closest to the click on the click's monitor.
pub fn select_waybar(
  outputs: &Value,
  pointer_x: i32,
  pointer_y: i32,
  monitor: MonitorGeometry,
) -> Option<WaybarGeometry> {
  let (mx, my, mw, mh, _) = monitor;
  outputs
    .as_object()?
    .values()
    .filter_map(|output| output.get("levels")?.as_object())
    .flat_map(|levels| levels.values())
    .filter_map(Value::as_array)
    .flatten()
    .filter_map(|layer| {
      let namespace = layer.get("namespace")?.as_str()?;
      let x = layer.get("x")?.as_i64()? as i32;
      let y = layer.get("y")?.as_i64()? as i32;
      let w = layer.get("w")?.as_i64()? as i32;
      let h = layer.get("h")?.as_i64()? as i32;
      let horizontal = WAYBAR_LAYER_NAMESPACES.contains(&namespace) && w > h.saturating_mul(4);
      let same_monitor = x >= mx && x < mx + mw && y >= my && y < my + mh;
      let distance = i64::from(pointer_x.clamp(x, x + w) - pointer_x).abs()
        + i64::from(pointer_y.clamp(y, y + h) - pointer_y).abs();
      (horizontal && same_monitor).then_some((
        distance,
        WaybarGeometry {
          top: y,
          right: x.checked_add(w)?,
          bottom: y.checked_add(h)?,
        },
      ))
    })
    .min_by_key(|(distance, _)| *distance)
    .map(|(_, bar)| bar)
}

pub fn pointer_is_outside(bounds: Option<PopupBounds>) -> bool {
  let Some((x, y, width, height)) = bounds else {
    return true;
  };
  let Some((pointer_x, pointer_y)) = current_pointer_position() else {
    return true;
  };
  pointer_x < x || pointer_x >= x + width || pointer_y < y || pointer_y >= y + height
}

pub fn current_pointer_position() -> Option<(i32, i32)> {
  let value = hyprctl_json(&["cursorpos", "-j"])?;
  Some((
    value.get("x")?.as_i64()? as i32,
    value.get("y")?.as_i64()? as i32,
  ))
}

pub fn current_monitor_geometry(pointer_x: i32, pointer_y: i32) -> Option<MonitorGeometry> {
  let monitors = hyprctl_json(&["monitors", "-j"])?;
  monitors.as_array()?.iter().find_map(|monitor| {
    let x = monitor.get("x")?.as_i64()? as i32;
    let y = monitor.get("y")?.as_i64()? as i32;
    let width = monitor.get("width")?.as_i64()? as i32;
    let height = monitor.get("height")?.as_i64()? as i32;
    let reserved_top = monitor
      .get("reserved")
      .and_then(|reserved| reserved.get(1))
      .and_then(Value::as_i64)
      .unwrap_or(0) as i32;
    let contains =
      pointer_x >= x && pointer_y >= y && pointer_x < x + width && pointer_y < y + height;
    contains.then_some((x, y, width, height, reserved_top))
  })
}

fn hyprctl_json(arguments: &[&str]) -> Option<Value> {
  let output = Command::new("hyprctl").args(arguments).output().ok()?;
  if !output.status.success() {
    return None;
  }
  serde_json::from_slice(&output.stdout).ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The size of the taskbar calendar popup.
  const CALENDAR: PopupSize = PopupSize {
    width: 398,
    height_estimate: 520,
  };
  const MONITOR: MonitorGeometry = (0, 0, 1920, 1080, 64);

  #[test]
  fn popup_opens_below_the_fixed_click_position() {
    assert_eq!(
      popup_origin(1902, 46, (0, 0, 1920, 1080, 0), CALENDAR),
      (1504, 50)
    );
    assert_eq!(
      popup_origin(1920, 1060, (0, 0, 1920, 1080, 0), CALENDAR),
      (1522, 560)
    );
  }

  #[test]
  fn popup_right_edge_follows_waybar_not_click_position() {
    let layers = serde_json::json!({"HDMI-A-1": {"levels": {"1": [
      {"namespace":"waybar", "x":18, "y":18, "w":1884, "h":28},
      {"namespace":"waybar", "x":18, "y":64, "w":384, "h":998}
    ]}}});
    for y in [0, 7, 17, 18, 30, 45, 46] {
      let bar = select_waybar(&layers, 1600, y, MONITOR).unwrap();
      assert_eq!(
        bar,
        WaybarGeometry {
          top: 18,
          right: 1902,
          bottom: 46
        }
      );
      assert_eq!(
        popup_origin(bar.right, bar.bottom, MONITOR, CALENDAR),
        (1504, 50)
      );
    }
  }

  #[test]
  fn argvus_taskbar_layer_is_recognized_at_top_and_bottom() {
    let top = serde_json::json!({"HDMI-A-1": {"levels": {"1": [
      {"namespace":"argvus-taskbar", "x":18, "y":18, "w":1884, "h":28},
      {"namespace":"argvus-widget-telemetry", "x":18, "y":64, "w":384, "h":998}
    ]}}});
    let bar = select_waybar(&top, 1600, 30, MONITOR).unwrap();
    assert_eq!(
      bar,
      WaybarGeometry {
        top: 18,
        right: 1902,
        bottom: 46
      }
    );

    let bottom = serde_json::json!({"HDMI-A-1": {"levels": {"1": [
      {"namespace":"argvus-taskbar", "x":18, "y":1034, "w":1884, "h":28}
    ]}}});
    let bar = select_waybar(&bottom, 1600, 1040, (0, 0, 1920, 1080, 0)).unwrap();
    assert_eq!((bar.top, bar.right, bar.bottom), (1034, 1902, 1062));
  }

  #[test]
  fn bottom_bar_is_found_even_with_surface_local_click_coordinates() {
    let layers = serde_json::json!({"HDMI-A-1": {"levels": {"1": [
      {"namespace":"waybar", "x":0, "y":1050, "w":1920, "h":28},
      {"namespace":"waybar", "x":0, "y":0, "w":384, "h":1050}
    ]}}});
    for y in [7, 1050, 1077] {
      let bar = select_waybar(&layers, 1600, y, (0, 0, 1920, 1080, 0)).unwrap();
      assert_eq!((bar.top, bar.right), (1050, 1920));
    }
  }

  #[test]
  fn top_bar_gives_a_popup_below_it_and_a_bottom_bar_one_above_it() {
    let anchor = PopupAnchor::at(1600, 30);
    let top = WaybarGeometry {
      top: 18,
      right: 1902,
      bottom: 46,
    };
    let below = resolve_position(anchor, MONITOR, Some(top), CALENDAR);
    assert!(!below.above);
    assert_eq!(
      (below.margin_top, below.margin_right),
      (50, 1920 - 1504 - 398)
    );
    assert_eq!(below.bounds, (1504, 50, 398, 520));

    let bottom = WaybarGeometry {
      top: 1034,
      right: 1902,
      bottom: 1062,
    };
    let above = resolve_position(anchor, (0, 0, 1920, 1080, 0), Some(bottom), CALENDAR);
    assert!(above.above);
    assert_eq!(above.margin_bottom, 1080 - 1034 + POPUP_GAP_Y);
    assert_eq!(above.margin_top, 0);
  }

  #[test]
  fn click_geometry_is_used_when_no_bar_is_found() {
    let anchor = PopupAnchor {
      x: 1600,
      y: 30,
      bar_bottom: Some(46),
      bar_right: Some(1902),
    };
    let position = resolve_position(anchor, (0, 0, 1920, 1080, 0), None, CALENDAR);
    assert_eq!(position.bounds, (1504, 50, 398, 520));
  }

  #[test]
  fn a_missing_pointer_counts_as_outside() {
    assert!(pointer_is_outside(None));
  }
}
