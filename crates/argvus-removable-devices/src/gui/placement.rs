//! Where the menu opens: the monitor under the click and the side of the ARGVUS
//! Taskbar that faces the screen.
//!
//! The rules mirror `argvus-taskbar-calendar` (`ui/app/popup.rs`): the nearest
//! horizontal taskbar layer on the clicked monitor decides the side. A taskbar
//! in the upper half opens the menu below it; one in the lower half opens the
//! menu above it, so the popup never covers the bar.

use super::*;

/// Layer namespaces of the taskbar. `argvus-taskbar` is the current Waybar
/// instance name; `waybar` covers sessions still running an unnamed bar.
const TASKBAR_LAYER_NAMESPACES: [&str; 2] = ["argvus-taskbar", "waybar"];

/// The clicked monitor in desktop (layout) coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MonitorArea {
  pub(super) x: i32,
  pub(super) y: i32,
  pub(super) width: i32,
  pub(super) height: i32,
}

/// Vertical edges of the taskbar layer, in desktop coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TaskbarGeometry {
  pub(super) top: i32,
  pub(super) bottom: i32,
}

/// The edge the menu is attached to, in monitor-local coordinates. `Above`
/// keeps the bottom edge fixed so the menu grows upwards when rows are added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuAnchor {
  Below { top: i32 },
  Above { bottom: i32 },
}

pub(super) struct PopupPosition {
  pub(super) monitor: Option<gdk::Monitor>,
  pub(super) area: MonitorArea,
  /// Monitor-local X of the main menu.
  pub(super) menu_x: i32,
  pub(super) anchor: MenuAnchor,
}

/// Resolves the monitor and the menu anchor for a click at `click` (desktop
/// coordinates). `bar_bottom` is the launcher's hint, used only when the
/// taskbar layer cannot be read from Hyprland.
pub(super) fn popup_position(click: (i32, i32), bar_bottom: Option<i32>) -> PopupPosition {
  let (click_x, click_y) = click;
  let monitor =
    gdk::Display::default().and_then(|display| monitor_at_point(&display, click_x, click_y));
  let area = monitor.as_ref().map_or(FALLBACK_AREA, |monitor| {
    let geometry = monitor.geometry();
    MonitorArea {
      x: geometry.x(),
      y: geometry.y(),
      width: geometry.width(),
      height: geometry.height(),
    }
  });
  let taskbar = current_taskbar(click_x, click_y, area);

  PopupPosition {
    monitor,
    area,
    menu_x: menu_left(click_x, area),
    anchor: menu_anchor(taskbar, bar_bottom, click_y, area),
  }
}

/// Used when GDK reports no monitor for the click (no display yet).
const FALLBACK_AREA: MonitorArea = MonitorArea {
  x: 0,
  y: 0,
  width: 1920,
  height: 1080,
};

pub(super) fn menu_anchor(
  taskbar: Option<TaskbarGeometry>,
  bar_bottom: Option<i32>,
  click_y: i32,
  area: MonitorArea,
) -> MenuAnchor {
  match taskbar {
    Some(bar) if bar.top - area.y > area.height / 2 => MenuAnchor::Above {
      bottom: bar.top - area.y - POPUP_GAP_Y,
    },
    Some(bar) => MenuAnchor::Below {
      top: bar.bottom - area.y + POPUP_GAP_Y,
    },
    None => match bar_bottom {
      Some(bottom) => MenuAnchor::Below {
        top: bottom - area.y + POPUP_GAP_Y,
      },
      // No taskbar information at all: open towards the larger free side of
      // the click so the menu does not cover the clicked module.
      None if click_y - area.y > area.height / 2 => MenuAnchor::Above {
        bottom: click_y - area.y - POPUP_GAP_Y,
      },
      None => MenuAnchor::Below {
        top: click_y - area.y + POPUP_GAP_Y,
      },
    },
  }
}

/// Monitor-local X of the main menu: the click column, kept on screen.
pub(super) fn menu_left(click_x: i32, area: MonitorArea) -> i32 {
  let max_x = (area.width - MENU_WIDTH - SCREEN_MARGIN).max(SCREEN_MARGIN);
  (click_x - area.x).clamp(SCREEN_MARGIN, max_x)
}

/// Monitor-local Y of the main menu once its height is known.
pub(super) fn menu_top(anchor: MenuAnchor, menu_height: i32, area_height: i32) -> i32 {
  let top = match anchor {
    MenuAnchor::Below { top } => top,
    MenuAnchor::Above { bottom } => bottom - menu_height,
  };
  let max_y = (area_height - menu_height - SCREEN_MARGIN).max(SCREEN_MARGIN);
  top.clamp(SCREEN_MARGIN, max_y)
}

/// The action submenu opens to the right of the menu, or to its left when it
/// would leave the monitor.
pub(super) fn submenu_left(menu_x: i32, area_width: i32) -> i32 {
  if menu_x + MENU_WIDTH + ACTION_MENU_WIDTH <= area_width - SCREEN_MARGIN {
    menu_x + MENU_WIDTH
  } else {
    (menu_x - ACTION_MENU_WIDTH).max(SCREEN_MARGIN)
  }
}

/// The submenu starts level with its device row and is pushed up when it
/// would leave the bottom of the monitor (taskbar at the bottom, long lists).
pub(super) fn submenu_top(row_y: i32, submenu_height: i32, area_height: i32) -> i32 {
  let max_y = (area_height - submenu_height - SCREEN_MARGIN).max(SCREEN_MARGIN);
  row_y.clamp(SCREEN_MARGIN, max_y)
}

/// The monitor whose geometry contains the point. GTK4 has no point lookup on
/// the display, so the monitor list is searched.
fn monitor_at_point(display: &gdk::Display, x: i32, y: i32) -> Option<gdk::Monitor> {
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

fn current_taskbar(click_x: i32, click_y: i32, area: MonitorArea) -> Option<TaskbarGeometry> {
  let output = std::process::Command::new("hyprctl")
    .args(["layers", "-j"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  let outputs = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
  select_taskbar(&outputs, click_x, click_y, area)
}

/// Picks, from `hyprctl layers -j`, the horizontal taskbar layer on `area`
/// closest to the click.
pub(super) fn select_taskbar(
  outputs: &serde_json::Value,
  click_x: i32,
  click_y: i32,
  area: MonitorArea,
) -> Option<TaskbarGeometry> {
  outputs
    .as_object()?
    .values()
    .filter_map(|output| output.get("levels")?.as_object())
    .flat_map(|levels| levels.values())
    .filter_map(serde_json::Value::as_array)
    .flatten()
    .filter_map(|layer| {
      let namespace = layer.get("namespace")?.as_str()?;
      let x = i32::try_from(layer.get("x")?.as_i64()?).ok()?;
      let y = i32::try_from(layer.get("y")?.as_i64()?).ok()?;
      let width = i32::try_from(layer.get("w")?.as_i64()?).ok()?;
      let height = i32::try_from(layer.get("h")?.as_i64()?).ok()?;
      let horizontal =
        TASKBAR_LAYER_NAMESPACES.contains(&namespace) && width > height.saturating_mul(4);
      let same_monitor =
        x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height;
      let distance = i64::from(click_x.clamp(x, x + width) - click_x).abs()
        + i64::from(click_y.clamp(y, y + height) - click_y).abs();
      (horizontal && same_monitor).then_some((
        distance,
        TaskbarGeometry {
          top: y,
          bottom: y.checked_add(height)?,
        },
      ))
    })
    .min_by_key(|(distance, _)| *distance)
    .map(|(_, bar)| bar)
}

#[cfg(test)]
mod tests {
  use super::*;

  const AREA: MonitorArea = MonitorArea {
    x: 0,
    y: 0,
    width: 1920,
    height: 1080,
  };

  fn layers(namespace: &str, y: i32) -> serde_json::Value {
    serde_json::json!({
      "DP-1": { "levels": { "2": [
        { "namespace": "hyprpaper", "x": 0, "y": 0, "w": 1920, "h": 1080 },
        { "namespace": namespace, "x": 18, "y": y, "w": 1884, "h": 28 }
      ] } }
    })
  }

  #[test]
  fn finds_the_argvus_taskbar_layer() {
    let bar = select_taskbar(&layers("argvus-taskbar", 18), 1500, 30, AREA);
    assert_eq!(
      bar,
      Some(TaskbarGeometry {
        top: 18,
        bottom: 46
      })
    );
  }

  #[test]
  fn still_finds_an_unnamed_waybar_layer() {
    let bar = select_taskbar(&layers("waybar", 18), 1500, 30, AREA);
    assert_eq!(
      bar,
      Some(TaskbarGeometry {
        top: 18,
        bottom: 46
      })
    );
  }

  #[test]
  fn ignores_taskbars_on_other_monitors() {
    let second_monitor = MonitorArea { x: 1920, ..AREA };
    assert_eq!(
      select_taskbar(&layers("argvus-taskbar", 18), 2500, 30, second_monitor),
      None
    );
  }

  #[test]
  fn top_taskbar_opens_the_menu_below_it() {
    let bar = TaskbarGeometry {
      top: 18,
      bottom: 46,
    };
    let anchor = menu_anchor(Some(bar), None, 30, AREA);

    assert_eq!(
      anchor,
      MenuAnchor::Below {
        top: 46 + POPUP_GAP_Y
      }
    );
    assert_eq!(menu_top(anchor, 200, AREA.height), 46 + POPUP_GAP_Y);
  }

  #[test]
  fn bottom_taskbar_opens_the_menu_above_it() {
    let bar = TaskbarGeometry {
      top: 1034,
      bottom: 1062,
    };
    let anchor = menu_anchor(Some(bar), None, 1050, AREA);

    assert_eq!(
      anchor,
      MenuAnchor::Above {
        bottom: 1034 - POPUP_GAP_Y
      }
    );
    assert_eq!(menu_top(anchor, 200, AREA.height), 1034 - POPUP_GAP_Y - 200);
  }

  #[test]
  fn anchor_is_relative_to_the_clicked_monitor() {
    let lower_monitor = MonitorArea { y: 1080, ..AREA };
    let bar = TaskbarGeometry {
      top: 1098,
      bottom: 1126,
    };

    assert_eq!(
      menu_anchor(Some(bar), None, 1110, lower_monitor),
      MenuAnchor::Below {
        top: 46 + POPUP_GAP_Y
      }
    );
  }

  #[test]
  fn launcher_hint_is_used_without_hyprland() {
    assert_eq!(
      menu_anchor(None, Some(46), 30, AREA),
      MenuAnchor::Below {
        top: 46 + POPUP_GAP_Y
      }
    );
    assert_eq!(
      menu_anchor(None, None, 1050, AREA),
      MenuAnchor::Above {
        bottom: 1050 - POPUP_GAP_Y
      }
    );
  }

  #[test]
  fn menu_and_submenu_stay_on_screen() {
    assert_eq!(menu_left(1910, AREA), 1920 - MENU_WIDTH - SCREEN_MARGIN);
    assert_eq!(menu_top(MenuAnchor::Below { top: 1000 }, 200, 1080), 876);
    assert_eq!(submenu_left(100, 1920), 100 + MENU_WIDTH);
    assert_eq!(submenu_left(1600, 1920), 1600 - ACTION_MENU_WIDTH);
    assert_eq!(submenu_top(1000, 150, 1080), 926);
    assert_eq!(submenu_top(60, 150, 1080), 60);
  }
}
