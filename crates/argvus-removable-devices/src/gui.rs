//! Native Wayland tray-style menu for removable devices.
//!
//! The transparent layer-shell surface covers the clicked monitor and provides
//! popup semantics (Escape and click-outside dismissal). The visible widgets
//! form a compact device menu and an adjacent action submenu, placed next to
//! the ARGVUS Taskbar like the calendar popup.
//!
//! Pointer and keyboard input come from GTK event controllers on the surface,
//! so hover tracking needs no polling. Device work (refresh and actions) runs
//! on the tokio runtime and its result is awaited on the GTK main context.
//!
//! GTK4: rows are plain boxes with a click gesture. GTK4 windows are transparent
//! by default and take their WM class from the application id, so the GTK3
//! visual and role calls have no equivalent and are not needed.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::actions::{ActionKind, Actions};
use crate::config::Config;
use crate::device::Device;
use crate::i18n;
use crate::process;
use crate::snapshot;
use crate::text;

mod entries;
mod input;
mod menu_widgets;
mod placement;
mod styling;

use entries::DeviceEntry;
use placement::{MenuAnchor, MonitorArea, popup_position, submenu_left};
use styling::install_css;

const MENU_WIDTH: i32 = 284;
const ACTION_MENU_WIDTH: i32 = 144;
const MENU_ROW_HEIGHT: i32 = 29;
/// Space between the taskbar and the menu, the same as the calendar popup.
const POPUP_GAP_Y: i32 = 4;
const SCREEN_MARGIN: i32 = 4;
/// How long the pointer must rest on another row before the open submenu
/// follows it. Crossing other rows on the way to the submenu is then harmless.
const SUBMENU_SWITCH_DELAY: Duration = Duration::from_millis(220);

/// What a row of the main menu does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainRowKind {
  /// Opens the action submenu of `devices[index]`.
  Device(usize),
  Refresh,
  Quit,
}

struct MainRow {
  widget: gtk::Box,
  kind: MainRowKind,
}

struct ActionRow {
  widget: gtk::Box,
  entry: DeviceEntry,
}

/// A submenu change waiting for the hover delay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingHover {
  Open(usize),
  Close,
}

/// State shared by the widgets and the event controllers of one popup.
struct Menu {
  cfg: Config,
  handle: tokio::runtime::Handle,
  main_loop: glib::MainLoop,
  overlay: gtk::Window,
  fixed: gtk::Fixed,
  main_menu: gtk::Box,
  submenu: gtk::Box,
  area: MonitorArea,
  anchor: MenuAnchor,
  menu_x: i32,
  submenu_x: i32,
  /// A refresh or an action is running; further activations are ignored.
  busy: Cell<bool>,
  devices: RefCell<Vec<Device>>,
  main_rows: RefCell<Vec<MainRow>>,
  action_rows: RefCell<Vec<ActionRow>>,
  /// Device whose action submenu is shown.
  open_device: Cell<Option<usize>>,
  /// Highlighted row of the main menu (pointer or keyboard).
  main_selection: Cell<Option<usize>>,
  /// Highlighted action; `Some` also means the keyboard is in the submenu.
  action_selection: Cell<Option<usize>>,
  pending_hover: RefCell<Option<(PendingHover, glib::SourceId)>>,
}

pub fn run(
  cfg: &Config,
  devices: &[Device],
  config_path: &str,
  click: (i32, i32),
  bar_bottom: Option<i32>,
) -> i32 {
  if let Err(e) = gtk::init() {
    eprintln!("argvus-removable-devices: gtk init failed: {e}");
    process::notify(
      "argvus-removable-devices",
      i18n::tr(
        "cannot open the storage menu",
        "não foi possível abrir o menu de armazenamento",
      ),
      true,
    );
    return 1;
  }

  install_css(config_path);

  let position = popup_position(click, bar_bottom);
  let main_loop = glib::MainLoop::new(None, false);
  let overlay = build_overlay(position.monitor.as_ref(), position.area);

  // The fixed container fills the surface, so its coordinates are the
  // monitor-local coordinates used by the placement module.
  let fixed = gtk::Fixed::new();
  fixed.set_size_request(position.area.width, position.area.height);
  fixed.set_widget_name("storage-click-catcher");
  overlay.set_child(Some(&fixed));

  let main_menu = gtk::Box::new(gtk::Orientation::Vertical, 0);
  main_menu.set_size_request(MENU_WIDTH, -1);
  main_menu.add_css_class("storage-popup-menu");
  fixed.put(&main_menu, f64::from(position.menu_x), 0.0);

  let submenu_x = submenu_left(position.menu_x, position.area.width);
  let submenu = gtk::Box::new(gtk::Orientation::Vertical, 0);
  submenu.set_size_request(ACTION_MENU_WIDTH, -1);
  submenu.add_css_class("storage-popup-menu");
  // Hidden until a device row is hovered. GTK4 shows widgets by default, so the
  // visibility is set explicitly instead of relying on `show_all`.
  submenu.set_visible(false);
  fixed.put(&submenu, f64::from(submenu_x), 0.0);

  let menu = Rc::new(Menu {
    cfg: cfg.clone(),
    handle: tokio::runtime::Handle::current(),
    main_loop: main_loop.clone(),
    overlay: overlay.clone(),
    fixed,
    main_menu,
    submenu,
    area: position.area,
    anchor: position.anchor,
    menu_x: position.menu_x,
    submenu_x,
    busy: Cell::new(false),
    devices: RefCell::new(Vec::new()),
    main_rows: RefCell::new(Vec::new()),
    action_rows: RefCell::new(Vec::new()),
    open_device: Cell::new(None),
    main_selection: Cell::new(None),
    action_selection: Cell::new(None),
    pending_hover: RefCell::new(None),
  });

  menu.rebuild(devices);
  input::connect(&menu);

  let close_loop = main_loop.clone();
  overlay.connect_close_request(move |_| {
    close_loop.quit();
    glib::Propagation::Proceed
  });

  overlay.present();
  // The first measurement can run before the theme CSS reaches the widgets;
  // place the menu again once the first frame is laid out.
  let placed = Rc::downgrade(&menu);
  glib::idle_add_local_once(move || {
    if let Some(menu) = placed.upgrade() {
      menu.place_main_menu();
    }
  });
  main_loop.run();
  0
}

/// The full-monitor layer surface that hosts both menus.
fn build_overlay(monitor: Option<&gdk::Monitor>, area: MonitorArea) -> gtk::Window {
  let overlay = gtk::Window::new();
  overlay.set_title(Some(i18n::tr(
    "Removable devices",
    "Dispositivos removíveis",
  )));
  overlay.set_widget_name("storage-menu-overlay");
  overlay.set_decorated(false);
  overlay.set_resizable(false);

  overlay.init_layer_shell();
  overlay.set_namespace(Some("argvus-removable-devices-menu"));
  overlay.set_layer(Layer::Overlay);
  for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
    overlay.set_anchor(edge, true);
  }
  // Cover the taskbar too, so the monitor-local coordinates start at the
  // output origin and a click on the bar dismisses the menu.
  overlay.set_exclusive_zone(-1);
  overlay.set_keyboard_mode(KeyboardMode::Exclusive);
  if let Some(monitor) = monitor {
    overlay.set_monitor(Some(monitor));
  }

  // GTK sizes a window from its children's natural size, and the Fixed only
  // reports the menu's extent, so the monitor size is requested explicitly.
  // Without it the surface stays at the menu's height while the compositor
  // paints the menu column down to the bottom of the output.
  overlay.set_default_size(area.width, area.height);
  overlay
}

impl Menu {
  /// Runs the action of `self.action_rows[index]` and closes the popup when
  /// it finishes. The popup is hidden first so a launched application or the
  /// unlock terminal is not covered by it.
  fn run_action(&self, index: usize) {
    let Some((kind, device)) = self
      .action_rows
      .borrow()
      .get(index)
      .map(|row| (row.entry.kind, row.entry.device.clone()))
    else {
      return;
    };
    if self.busy.replace(true) {
      return;
    }

    self.overlay.set_visible(false);
    let cfg = self.cfg.clone();
    let task = self
      .handle
      .spawn(async move { Actions::new(&cfg).perform(kind, &device).await });
    let main_loop = self.main_loop.clone();
    glib::spawn_future_local(async move {
      // `perform` reports its own failures through a notification.
      let _ = task.await;
      main_loop.quit();
    });
  }

  /// Enumerates the devices again and rebuilds both menus.
  fn refresh(self: &Rc<Self>) {
    if self.busy.replace(true) {
      return;
    }
    self.close_submenu();
    for row in self.main_rows.borrow().iter() {
      if row.kind == MainRowKind::Refresh {
        row.widget.set_sensitive(false);
      }
    }

    let cfg = self.cfg.clone();
    let task = self.handle.spawn(async move { snapshot(&cfg).await });
    let menu = Rc::downgrade(self);
    glib::spawn_future_local(async move {
      let devices = task.await;
      let Some(menu) = menu.upgrade() else {
        return;
      };
      menu.busy.set(false);
      match devices {
        Ok(devices) => menu.rebuild(&devices),
        // The enumeration task panicked: keep the current list usable.
        Err(_) => {
          let devices = menu.devices.borrow().clone();
          menu.rebuild(&devices);
        }
      }
    });
  }

  fn quit(&self) {
    self.main_loop.quit();
  }
}

#[cfg(test)]
mod tests {
  use super::entries::{device_entries, device_title, state_words};
  use super::*;

  fn kinds(device: &Device) -> Vec<ActionKind> {
    device_entries(device)
      .into_iter()
      .map(|entry| entry.kind)
      .collect()
  }

  #[test]
  fn mounted_device_has_open_unmount_and_copy_actions() {
    let device = Device {
      mounted: true,
      ejectable: true,
      ..Device::default()
    };
    let kinds = kinds(&device);

    assert!(kinds.contains(&ActionKind::Open));
    assert!(kinds.contains(&ActionKind::Unmount));
    assert!(kinds.contains(&ActionKind::Eject));
    assert!(kinds.contains(&ActionKind::Copy));
    assert!(!kinds.contains(&ActionKind::Mount));
  }

  #[test]
  fn locked_device_only_offers_unlock_and_safe_removal() {
    let device = Device {
      encrypted: true,
      locked: true,
      ejectable: true,
      can_power_off: true,
      ..Device::default()
    };
    let kinds = kinds(&device);

    assert!(kinds.contains(&ActionKind::Unlock));
    assert!(kinds.contains(&ActionKind::Eject));
    assert!(kinds.contains(&ActionKind::PowerOff));
    assert!(!kinds.contains(&ActionKind::Open));
    assert!(!kinds.contains(&ActionKind::Mount));
  }

  #[test]
  fn unlocked_volume_can_be_locked_mounted_or_not() {
    for mounted in [false, true] {
      let device = Device {
        encrypted: true,
        locked: false,
        mounted,
        ..Device::default()
      };
      assert!(
        kinds(&device).contains(&ActionKind::Lock),
        "mounted={mounted}"
      );
    }
  }

  #[test]
  fn device_title_matches_tray_menu_format() {
    let device = Device {
      name: "Ventoy".to_string(),
      capacity: 43_700_000_000,
      ..Device::default()
    };
    let (en, _) = state_words(false, false);

    assert_eq!(device_title(&device), format!("Ventoy · 43.7 GB · {}", en));
  }
}
