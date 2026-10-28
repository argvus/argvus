//! Native Wayland tray-style menu for removable devices.
//!
//! The transparent layer-shell surface provides popup semantics (positioning,
//! Escape and click-outside dismissal). The visible widgets form a compact,
//! hover-driven menu and an adjacent action submenu.
//!
//! GTK4: rows are plain boxes with a click gesture. GTK4 windows are transparent
//! by default and take their WM class from the application id, so the GTK3
//! visual and role calls have no equivalent and are not needed.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;
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
mod menu_widgets;
mod placement;
mod styling;

use menu_widgets::{rebuild_menu, update_hover};
use placement::{current_pointer_position, popup_position};
use styling::install_css;

const MENU_WIDTH: i32 = 284;
const ACTION_MENU_WIDTH: i32 = 144;
const MENU_ROW_HEIGHT: i32 = 29;
const MENU_GAP_Y: i32 = 12;
const SCREEN_MARGIN: i32 = 4;

enum UiUpdate {
  Devices(Vec<Device>),
  Close,
}

#[derive(Clone)]
struct MenuCtx {
  cfg: Config,
  handle: tokio::runtime::Handle,
  update_tx: mpsc::Sender<UiUpdate>,
  busy: Rc<Cell<bool>>,
  overlay: gtk::Window,
  fixed: gtk::Fixed,
  submenu: gtk::Box,
  submenu_x: i32,
  menu_y: i32,
  submenu_y: Rc<Cell<i32>>,
  active_row: Rc<RefCell<Option<gtk::Box>>>,
  hovered_action_row: Rc<RefCell<Option<gtk::Box>>>,
  active_index: Rc<Cell<i32>>,
  devices: Rc<RefCell<Vec<Device>>>,
  main_loop: glib::MainLoop,
}

pub fn run(
  cfg: &Config,
  devices: &[Device],
  config_path: &str,
  fixed_pos: Option<(i32, i32)>,
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

  let handle = tokio::runtime::Handle::current();
  let (update_tx, update_rx) = mpsc::channel::<UiUpdate>();
  let position = popup_position(devices.len(), fixed_pos, bar_bottom);
  let main_loop = glib::MainLoop::new(None, false);

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
  overlay.set_anchor(Edge::Top, true);
  overlay.set_anchor(Edge::Bottom, true);
  overlay.set_anchor(Edge::Left, true);
  overlay.set_anchor(Edge::Right, true);
  overlay.set_exclusive_zone(-1);
  overlay.set_keyboard_mode(KeyboardMode::Exclusive);
  if let Some(monitor) = &position.monitor {
    overlay.set_monitor(Some(monitor));
  }

  // The fixed container fills the surface, so its coordinates are the window's.
  // GTK sizes a window from its children's natural size, and the Fixed only
  // reports the menu's extent, so the monitor size is requested explicitly.
  // Without it the surface stays at the menu's height while the compositor
  // paints the menu column down to the bottom of the output.
  overlay.set_default_size(position.width, position.height);
  let fixed = gtk::Fixed::new();
  fixed.set_size_request(position.width, position.height);
  fixed.set_widget_name("storage-click-catcher");
  overlay.set_child(Some(&fixed));

  let main_menu = gtk::Box::new(gtk::Orientation::Vertical, 0);
  main_menu.set_size_request(MENU_WIDTH, -1);
  main_menu.add_css_class("storage-popup-menu");
  let menu_x = position.x;
  let menu_y = position.y;
  fixed.put(&main_menu, f64::from(menu_x), f64::from(menu_y));

  let submenu_x = if menu_x + MENU_WIDTH + ACTION_MENU_WIDTH <= position.width - SCREEN_MARGIN {
    menu_x + MENU_WIDTH
  } else {
    (menu_x - ACTION_MENU_WIDTH).max(SCREEN_MARGIN)
  };
  let submenu_y = Rc::new(Cell::new(menu_y));
  let submenu = gtk::Box::new(gtk::Orientation::Vertical, 0);
  submenu.set_size_request(ACTION_MENU_WIDTH, -1);
  submenu.add_css_class("storage-popup-menu");
  // Hidden until a device row is hovered. GTK4 shows widgets by default, so the
  // visibility is set explicitly instead of relying on `show_all`.
  submenu.set_visible(false);
  fixed.put(&submenu, f64::from(submenu_x), f64::from(menu_y));

  let ctx = MenuCtx {
    cfg: cfg.clone(),
    handle,
    update_tx,
    busy: Rc::new(Cell::new(false)),
    overlay: overlay.clone(),
    fixed: fixed.clone(),
    submenu: submenu.clone(),
    submenu_x,
    menu_y,
    submenu_y: submenu_y.clone(),
    active_row: Rc::new(RefCell::new(None)),
    hovered_action_row: Rc::new(RefCell::new(None)),
    active_index: Rc::new(Cell::new(-1)),
    devices: Rc::new(RefCell::new(devices.to_vec())),
    main_loop: main_loop.clone(),
  };

  rebuild_menu(&main_menu, devices, &ctx);

  // A press outside both menus dismisses the popup.
  let outside_main = main_menu.clone();
  let outside_submenu = submenu.clone();
  let outside_submenu_y = submenu_y.clone();
  let outside_loop = main_loop.clone();
  let outside_click = gtk::GestureClick::new();
  outside_click.set_button(0);
  outside_click.connect_pressed(move |_, _, x, y| {
    let main_allocation_width = f64::from(outside_main.width());
    let main_allocation_height = f64::from(outside_main.height());
    let inside_main = x >= f64::from(menu_x)
      && x < f64::from(menu_x) + main_allocation_width
      && y >= f64::from(menu_y)
      && y < f64::from(menu_y) + main_allocation_height;
    let submenu_y = outside_submenu_y.get();
    let inside_submenu = outside_submenu.is_visible()
      && x >= f64::from(submenu_x)
      && x < f64::from(submenu_x) + f64::from(outside_submenu.width())
      && y >= f64::from(submenu_y)
      && y < f64::from(submenu_y) + f64::from(outside_submenu.height());
    if !inside_main && !inside_submenu {
      outside_loop.quit();
    }
  });
  fixed.add_controller(outside_click);

  let close_loop = main_loop.clone();
  overlay.connect_close_request(move |_| {
    close_loop.quit();
    glib::Propagation::Proceed
  });
  let key_loop = main_loop.clone();
  let key_controller = gtk::EventControllerKey::new();
  key_controller.connect_key_pressed(move |_, keyval, _, _| {
    if keyval == gdk::Key::Escape {
      key_loop.quit();
      return glib::Propagation::Stop;
    }
    glib::Propagation::Proceed
  });
  overlay.add_controller(key_controller);

  let poll_menu = main_menu.clone();
  let poll_ctx = ctx.clone();
  let origin_x = position.origin_x;
  let origin_y = position.origin_y;
  let poll_loop = main_loop.clone();
  glib::timeout_add_local(Duration::from_millis(50), move || {
    while let Ok(update) = update_rx.try_recv() {
      poll_ctx.busy.set(false);
      match update {
        UiUpdate::Devices(devices) => rebuild_menu(&poll_menu, &devices, &poll_ctx),
        UiUpdate::Close => {
          poll_loop.quit();
          return glib::ControlFlow::Break;
        }
      }
    }
    if let Some((pointer_x, pointer_y)) = current_pointer_position() {
      update_hover(
        &poll_menu,
        pointer_x - origin_x,
        pointer_y - origin_y,
        &poll_ctx,
      );
    }
    glib::ControlFlow::Continue
  });

  overlay.present();
  main_loop.run();
  0
}

#[cfg(test)]
mod tests {
  use super::entries::{device_entries, device_title, state_words};
  use super::*;

  #[test]
  fn mounted_device_has_open_unmount_and_copy_actions() {
    let device = Device {
      mounted: true,
      ejectable: true,
      ..Device::default()
    };
    let kinds: Vec<ActionKind> = device_entries(&device)
      .into_iter()
      .map(|entry| entry.kind)
      .collect();

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
    let kinds: Vec<ActionKind> = device_entries(&device)
      .into_iter()
      .map(|entry| entry.kind)
      .collect();

    assert!(kinds.contains(&ActionKind::Unlock));
    assert!(kinds.contains(&ActionKind::Eject));
    assert!(kinds.contains(&ActionKind::PowerOff));
    assert!(!kinds.contains(&ActionKind::Open));
    assert!(!kinds.contains(&ActionKind::Mount));
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
