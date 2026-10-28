//! Popup window placement: layer-shell setup, the monitor under the pointer and the position next to Waybar.

use super::drafts::cancel_editor;
use super::*;

pub(super) fn hide_popup(view: &mut ViewState, window: &gtk::Window, popup_state: &PopupState) {
  popup_state
    .presentation_id
    .set(popup_state.presentation_id.get().wrapping_add(1));
  popup_state.dismiss_armed.set(false);
  popup_state.bounds.set(None);
  cancel_editor(view);
  window.set_visible(false);
  relm4::main_application().quit();
}

pub(super) fn present_popup(
  window: &gtk::Window,
  popup: &gtk::Box,
  popup_state: &Rc<PopupState>,
  fixed_position: Option<PopupAnchor>,
) {
  let presentation_id = popup_state.presentation_id.get().wrapping_add(1);
  popup_state.presentation_id.set(presentation_id);
  popup_state.dismiss_armed.set(false);
  popup_state.focused_once.set(false);
  set_monitor_at_pointer(window, fixed_position);
  popup_state
    .bounds
    .set(Some(position_popup(window, fixed_position)));
  window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
  popup.queue_resize();
  window.queue_resize();
  window.present();
  let measured_state = popup_state.clone();
  window.add_tick_callback(move |window, _| {
    if !window.is_visible() || measured_state.presentation_id.get() != presentation_id {
      return gtk::glib::ControlFlow::Break;
    }
    let width = window.width();
    let height = window.height();
    if width > 0
      && height > 0
      && let Some((x, y, old_width, old_height)) = measured_state.bounds.get()
    {
      let y = if window.is_anchor(gtk4_layer_shell::Edge::Bottom) {
        y + old_height - height
      } else {
        y
      };
      measured_state
        .bounds
        .set(Some((x + old_width - width, y, width, height)));
    }
    gtk::glib::ControlFlow::Continue
  });
  window.grab_focus();
  window.queue_draw();
  let click_outside = window.clone();
  let popup_state = popup_state.clone();
  gtk::glib::timeout_add_local(Duration::from_millis(800), move || {
    if click_outside.is_visible() && popup_state.presentation_id.get() == presentation_id {
      click_outside.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
      popup_state.dismiss_armed.set(true);
    }
    gtk::glib::ControlFlow::Break
  });
}

pub(super) fn set_monitor_at_pointer(window: &gtk::Window, fixed_position: Option<PopupAnchor>) {
  let Some(anchor) = fixed_position.or_else(|| {
    current_pointer_position().map(|(x, y)| PopupAnchor {
      x,
      y,
      bar_bottom: None,
      bar_right: None,
    })
  }) else {
    return;
  };
  let (pointer_x, pointer_y) = (anchor.x, anchor.y);
  let Some(display) = gtk::gdk::Display::default() else {
    return;
  };
  let monitors = display.monitors();
  for index in 0..monitors.n_items() {
    let Some(item) = monitors.item(index) else {
      continue;
    };
    let Ok(monitor) = item.downcast::<gtk::gdk::Monitor>() else {
      continue;
    };
    let geometry = monitor.geometry();
    let contains = pointer_x >= geometry.x()
      && pointer_x < geometry.x() + geometry.width()
      && pointer_y >= geometry.y()
      && pointer_y < geometry.y() + geometry.height();
    if contains {
      window.set_monitor(Some(&monitor));
      return;
    }
  }
}

pub(super) fn apply_layer_shell(window: &gtk::Window) {
  window.init_layer_shell();
  window.set_namespace(Some("argvus-taskbar-calendar"));
  window.set_layer(gtk4_layer_shell::Layer::Overlay);
  window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
  // Margins below are relative to the full monitor. With zone 0 the
  // compositor adds the panels' reserved area again, pushing us down.
  window.set_exclusive_zone(-1);
  for edge in [
    gtk4_layer_shell::Edge::Top,
    gtk4_layer_shell::Edge::Right,
    gtk4_layer_shell::Edge::Bottom,
    gtk4_layer_shell::Edge::Left,
  ] {
    window.set_anchor(edge, false);
    window.set_margin(edge, 0);
  }
  window.set_anchor(gtk4_layer_shell::Edge::Top, true);
  window.set_anchor(gtk4_layer_shell::Edge::Right, true);
}

pub(super) fn position_popup(
  window: &gtk::Window,
  fixed_position: Option<PopupAnchor>,
) -> PopupBounds {
  let position = popup_position(fixed_position);
  window.set_anchor(gtk4_layer_shell::Edge::Top, !position.above);
  window.set_anchor(gtk4_layer_shell::Edge::Bottom, position.above);
  window.set_margin(gtk4_layer_shell::Edge::Bottom, position.margin_bottom);
  window.set_margin(gtk4_layer_shell::Edge::Left, 0);
  window.set_margin(gtk4_layer_shell::Edge::Top, position.margin_top);
  window.set_margin(gtk4_layer_shell::Edge::Right, position.margin_right);
  position.bounds
}

pub(super) struct PopupPosition {
  pub(super) above: bool,
  pub(super) margin_bottom: i32,
  pub(super) margin_top: i32,
  pub(super) margin_right: i32,
  pub(super) bounds: PopupBounds,
}

pub(super) fn popup_position(fixed_position: Option<PopupAnchor>) -> PopupPosition {
  let anchor = fixed_position.unwrap_or_else(|| {
    current_pointer_position()
      .map(|(x, y)| PopupAnchor {
        x,
        y,
        bar_bottom: None,
        bar_right: None,
      })
      .unwrap_or(PopupAnchor {
        x: 0,
        y: 0,
        bar_bottom: None,
        bar_right: None,
      })
  });
  let (pointer_x, pointer_y) = (anchor.x, anchor.y);
  let monitor = current_monitor_geometry(pointer_x, pointer_y).unwrap_or((0, 0, 1920, 1080, 0));
  let (origin_x, origin_y, width, height, reserved_top) = monitor;
  let waybar = current_waybar_geometry(pointer_x, pointer_y, monitor);
  let bar_bottom = waybar
    .map(|geometry| geometry.bottom)
    .or(anchor.bar_bottom)
    .unwrap_or(origin_y + reserved_top);
  let bar_right = waybar
    .map(|geometry| geometry.right)
    .or(anchor.bar_right)
    .unwrap_or(origin_x + width);
  let (popup_x, popup_y) = popup_origin(bar_right, bar_bottom, origin_x, origin_y, width, height);
  let above = waybar.is_some_and(|bar| bar.top - origin_y > height / 2);
  let margin_bottom = waybar
    .filter(|_| above)
    .map_or(0, |bar| (origin_y + height - bar.top + POPUP_GAP_Y).max(0));

  PopupPosition {
    above,
    margin_bottom,
    margin_top: if above { 0 } else { popup_y.max(0) },
    margin_right: (width - popup_x - POPUP_WIDTH).max(0),
    bounds: (
      origin_x + popup_x,
      origin_y
        + if above {
          height - margin_bottom - POPUP_HEIGHT_ESTIMATE
        } else {
          popup_y
        },
      POPUP_WIDTH,
      POPUP_HEIGHT_ESTIMATE,
    ),
  }
}

pub(super) fn popup_origin(
  bar_right: i32,
  bar_bottom: i32,
  origin_x: i32,
  origin_y: i32,
  width: i32,
  height: i32,
) -> (i32, i32) {
  let max_x = (width - POPUP_WIDTH).max(0);
  let max_y = (height - POPUP_HEIGHT_ESTIMATE).max(0);
  let popup_x = (bar_right - origin_x - POPUP_WIDTH).clamp(0, max_x);
  let popup_y = (bar_bottom - origin_y + POPUP_GAP_Y).clamp(0, max_y);
  (popup_x, popup_y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WaybarGeometry {
  pub(super) top: i32,
  pub(super) right: i32,
  pub(super) bottom: i32,
}

pub(super) fn current_waybar_geometry(
  pointer_x: i32,
  pointer_y: i32,
  monitor: (i32, i32, i32, i32, i32),
) -> Option<WaybarGeometry> {
  let output = Command::new("hyprctl")
    .args(["layers", "-j"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let outputs = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
  select_waybar(&outputs, pointer_x, pointer_y, monitor)
}

pub(super) fn select_waybar(
  outputs: &serde_json::Value,
  pointer_x: i32,
  pointer_y: i32,
  monitor: (i32, i32, i32, i32, i32),
) -> Option<WaybarGeometry> {
  let (mx, my, mw, mh, _) = monitor;
  outputs
    .as_object()?
    .values()
    .filter_map(|output| output.get("levels")?.as_object())
    .flat_map(|levels| levels.values())
    .filter_map(serde_json::Value::as_array)
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

pub(super) fn pointer_is_outside_popup(bounds: Option<PopupBounds>) -> bool {
  let Some((x, y, width, height)) = bounds else {
    return true;
  };
  let Some((pointer_x, pointer_y)) = current_pointer_position() else {
    return true;
  };
  pointer_x < x || pointer_x >= x + width || pointer_y < y || pointer_y >= y + height
}

pub(crate) fn current_pointer_position() -> Option<(i32, i32)> {
  let output = Command::new("hyprctl")
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

pub(super) fn current_monitor_geometry(
  pointer_x: i32,
  pointer_y: i32,
) -> Option<(i32, i32, i32, i32, i32)> {
  let output = Command::new("hyprctl")
    .args(["monitors", "-j"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let monitors = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
  monitors.as_array()?.iter().find_map(|monitor| {
    let x = monitor.get("x")?.as_i64()? as i32;
    let y = monitor.get("y")?.as_i64()? as i32;
    let width = monitor.get("width")?.as_i64()? as i32;
    let height = monitor.get("height")?.as_i64()? as i32;
    let reserved_top = monitor
      .get("reserved")
      .and_then(|reserved| reserved.get(1))
      .and_then(serde_json::Value::as_i64)
      .unwrap_or(0) as i32;
    let contains =
      pointer_x >= x && pointer_y >= y && pointer_x < x + width && pointer_y < y + height;
    contains.then_some((x, y, width, height, reserved_top))
  })
}
