// Native Wayland tray-style menu for removable devices.
//
// The transparent layer-shell surface provides popup semantics (positioning,
// Escape and click-outside dismissal). The visible widgets form a compact,
// hover-driven menu and an adjacent action submenu.
//
// GTK4: rows are plain boxes with a click gesture. GTK4 windows are transparent
// by default and take their WM class from the application id, so the GTK3
// visual and role calls have no equivalent and are not needed.

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
use crate::snapshot;
use crate::util;

const MENU_WIDTH: i32 = 284;
const ACTION_MENU_WIDTH: i32 = 144;
const MENU_ROW_HEIGHT: i32 = 29;
const MENU_GAP_Y: i32 = 12;
const SCREEN_MARGIN: i32 = 4;

struct DeviceEntry {
  kind: ActionKind,
  device: Device,
}

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

struct PopupPosition {
  monitor: Option<gdk::Monitor>,
  x: i32,
  y: i32,
  width: i32,
  height: i32,
  origin_x: i32,
  origin_y: i32,
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
    util::notify(
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

fn rebuild_menu(main_menu: &gtk::Box, devices: &[Device], ctx: &MenuCtx) {
  clear_menu(main_menu);
  clear_menu(&ctx.submenu);
  ctx.submenu.set_visible(false);
  *ctx.devices.borrow_mut() = devices.to_vec();
  ctx.active_index.set(-1);
  *ctx.active_row.borrow_mut() = None;
  *ctx.hovered_action_row.borrow_mut() = None;

  if devices.is_empty() {
    let empty = menu_row(
      i18n::tr("No removable devices", "Nenhum dispositivo removível"),
      false,
    );
    empty.set_sensitive(false);
    main_menu.append(&empty);
  } else {
    for device in devices {
      let row = menu_row(&device_title(device), true);
      main_menu.append(&row);
    }
  }

  let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
  separator.set_margin_top(2);
  separator.set_margin_bottom(2);
  main_menu.append(&separator);

  let refresh = menu_row(i18n::tr("Refresh", "Atualizar"), false);
  connect_refresh(&refresh, ctx);
  main_menu.append(&refresh);

  let quit = menu_row(i18n::tr("Quit", "Sair"), false);
  let quit_loop = ctx.main_loop.clone();
  on_release(&quit, move || quit_loop.quit());
  main_menu.append(&quit);
}

/// Builds one menu row. The row is a box with a label and an optional arrow.
fn menu_row(text: &str, has_submenu: bool) -> gtk::Box {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  row.set_size_request(-1, MENU_ROW_HEIGHT);
  row.set_margin_start(10);
  row.set_margin_end(8);
  row.add_css_class("storage-menu-row");

  let label = gtk::Label::new(Some(text));
  label.set_xalign(0.0);
  label.set_hexpand(true);
  label.set_ellipsize(gtk::pango::EllipsizeMode::End);
  row.append(&label);

  if has_submenu {
    let arrow = gtk::Image::from_icon_name("pan-end-symbolic");
    row.append(&arrow);
  }
  row
}

/// Runs `action` when a pointer button is released over `row`.
fn on_release(row: &gtk::Box, action: impl Fn() + 'static) {
  let gesture = gtk::GestureClick::new();
  gesture.set_button(0);
  gesture.connect_released(move |_, _, _, _| action());
  row.add_controller(gesture);
}

fn show_device_submenu(row: &gtk::Box, device: &Device, index: usize, ctx: &MenuCtx) {
  set_active_row(ctx, Some(row));
  rebuild_action_menu(device, ctx);
  let submenu_y = ctx.menu_y + index as i32 * MENU_ROW_HEIGHT;
  ctx.submenu_y.set(submenu_y);
  ctx
    .fixed
    .move_(&ctx.submenu, f64::from(ctx.submenu_x), f64::from(submenu_y));
  ctx.submenu.set_visible(true);
}

fn set_active_row(ctx: &MenuCtx, row: Option<&gtk::Box>) {
  if ctx
    .active_row
    .borrow()
    .as_ref()
    .is_some_and(|current| Some(current) == row)
  {
    return;
  }
  if let Some(previous) = ctx.active_row.borrow_mut().take() {
    previous.remove_css_class("submenu-open");
    previous.queue_draw();
  }
  if let Some(row) = row {
    row.add_css_class("submenu-open");
    row.queue_draw();
    *ctx.active_row.borrow_mut() = Some(row.clone());
  }
}

fn set_hovered_action_row(ctx: &MenuCtx, row: Option<&gtk::Box>) {
  if ctx
    .hovered_action_row
    .borrow()
    .as_ref()
    .is_some_and(|current| Some(current) == row)
  {
    return;
  }
  if let Some(previous) = ctx.hovered_action_row.borrow_mut().take() {
    previous.remove_css_class("pointer-over");
    previous.queue_draw();
  }
  if let Some(row) = row {
    row.add_css_class("pointer-over");
    row.queue_draw();
    *ctx.hovered_action_row.borrow_mut() = Some(row.clone());
  }
}

fn update_hover(main_menu: &gtk::Box, x: i32, y: i32, ctx: &MenuCtx) {
  if ctx.submenu.is_visible() {
    for child in children(&ctx.submenu) {
      let Ok(row) = child.downcast::<gtk::Box>() else {
        continue;
      };
      if widget_contains_pointer(&row, x, y, &ctx.overlay) {
        set_hovered_action_row(ctx, Some(&row));
        return;
      }
    }
  }
  set_hovered_action_row(ctx, None);

  let device_count = ctx.devices.borrow().len();
  let rows: Vec<gtk::Box> = children(main_menu)
    .into_iter()
    .filter_map(|child| child.downcast::<gtk::Box>().ok())
    .collect();

  for (index, row) in rows.iter().enumerate() {
    if !widget_contains_pointer(row, x, y, &ctx.overlay) {
      continue;
    }
    if index < device_count {
      if ctx.active_index.replace(index as i32) != index as i32 {
        let device = ctx.devices.borrow()[index].clone();
        show_device_submenu(row, &device, index, ctx);
      }
    } else {
      ctx.active_index.set(-1);
      set_active_row(ctx, Some(row));
      ctx.submenu.set_visible(false);
    }
    return;
  }

  ctx.active_index.set(-1);
  set_active_row(ctx, None);
  ctx.submenu.set_visible(false);
}

fn widget_contains_pointer(widget: &gtk::Box, x: i32, y: i32, overlay: &gtk::Window) -> bool {
  if !widget.is_visible() {
    return false;
  }
  let Some(origin) = widget.compute_point(overlay, &gtk::graphene::Point::new(0.0, 0.0)) else {
    return false;
  };
  let widget_x = origin.x() as i32;
  let widget_y = origin.y() as i32;
  x >= widget_x && x < widget_x + widget.width() && y >= widget_y && y < widget_y + widget.height()
}

fn rebuild_action_menu(device: &Device, ctx: &MenuCtx) {
  clear_menu(&ctx.submenu);
  for entry in device_entries(device) {
    let action = menu_row(action_label(entry.kind), false);
    action.set_tooltip_text(Some(action_tooltip(entry.kind)));
    connect_action(&action, entry, ctx);
    ctx.submenu.append(&action);
  }
}

/// The direct children of a container, in display order.
fn children(container: &impl IsA<gtk::Widget>) -> Vec<gtk::Widget> {
  let mut children = Vec::new();
  let mut child = container.as_ref().first_child();
  while let Some(widget) = child {
    child = widget.next_sibling();
    children.push(widget);
  }
  children
}

fn clear_menu(container: &gtk::Box) {
  for child in children(container) {
    container.remove(&child);
  }
}

fn connect_refresh(item: &gtk::Box, ctx: &MenuCtx) {
  let cfg = ctx.cfg.clone();
  let handle = ctx.handle.clone();
  let update_tx = ctx.update_tx.clone();
  let busy = ctx.busy.clone();
  let item_state = item.clone();
  on_release(item, move || {
    if busy.replace(true) {
      return;
    }
    item_state.set_sensitive(false);
    let cfg = cfg.clone();
    let update_tx = update_tx.clone();
    handle.spawn(async move {
      let devices = snapshot(&cfg).await;
      let _ = update_tx.send(UiUpdate::Devices(devices));
    });
  });
}

fn connect_action(item: &gtk::Box, entry: DeviceEntry, ctx: &MenuCtx) {
  let cfg = ctx.cfg.clone();
  let handle = ctx.handle.clone();
  let update_tx = ctx.update_tx.clone();
  let busy = ctx.busy.clone();
  let overlay = ctx.overlay.clone();

  on_release(item, move || {
    if busy.replace(true) {
      return;
    }

    overlay.set_visible(false);
    let kind = entry.kind;
    let device = entry.device.clone();
    let cfg = cfg.clone();
    let update_tx = update_tx.clone();
    handle.spawn(async move {
      let _ = Actions::new(&cfg).perform(kind, &device).await;
      let _ = update_tx.send(UiUpdate::Close);
    });
  });
}

fn popup_position(
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

fn current_waybar_bottom(pointer_x: i32, pointer_y: i32) -> Option<i32> {
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

fn current_pointer_position() -> Option<(i32, i32)> {
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

// Loads theme.css from disk instead of embedding CSS in the binary.
// Candidates are tried in the same precedence order as config.json (system
// defaults, then user override, then a theme.css next to an explicit
// --config file); every one that exists is applied, in that order, so a
// later file's rules win over an earlier file's for the same selector.
// Missing files are silently skipped — if none exist, the menu falls back
// to unstyled GTK widgets rather than failing to open. GTK4 reports invalid
// CSS through its own log, so a bad file no longer aborts this step.
fn install_css(config_path: &str) {
  let Some(display) = gdk::Display::default() else {
    return;
  };
  for path in Config::theme_css_paths(config_path) {
    if !std::path::Path::new(&path).is_file() {
      continue;
    }
    let provider = gtk::CssProvider::new();
    provider.load_from_path(&path);
    gtk::style_context_add_provider_for_display(
      &display,
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
  }
  let accent = accent_runtime_css();
  if !accent.is_empty() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&accent);
    gtk::style_context_add_provider_for_display(
      &display,
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
    );
  }
}

fn accent_runtime_css() -> String {
  let Some(config_home) = std::env::var_os("ARGVUS_CONFIG_HOME")
    .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
    .or_else(|| {
      std::env::var_os("HOME").map(|home| {
        let mut path = std::path::PathBuf::from(home);
        path.push(".config");
        path.into_os_string()
      })
    })
  else {
    return String::new();
  };
  let path = std::path::PathBuf::from(config_home)
    .join("argvus")
    .join(".accent-color");
  let Ok(value) = std::fs::read_to_string(path) else {
    return String::new();
  };
  let value = value.trim();
  let hex = value.strip_prefix('#').unwrap_or(value);
  if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
    return String::new();
  }
  let Ok(red) = u8::from_str_radix(&hex[0..2], 16) else {
    return String::new();
  };
  let Ok(green) = u8::from_str_radix(&hex[2..4], 16) else {
    return String::new();
  };
  let Ok(blue) = u8::from_str_radix(&hex[4..6], 16) else {
    return String::new();
  };
  let luminance = (299 * u32::from(red) + 587 * u32::from(green) + 114 * u32::from(blue)) / 1000;
  let text = if luminance >= 128 {
    "#000000"
  } else {
    "#FFFFFF"
  };
  format!(
    ".storage-popup-menu {{ border-color: rgba({red}, {green}, {blue}, 0.54); }}\n\
     .storage-menu-row:hover, .storage-menu-row.submenu-open, .storage-menu-row.pointer-over {{ background-color: {value}; color: {text}; }}"
  )
}

// Mirror the action set offered by the rofi mode, filtered by device state.
fn device_entries(d: &Device) -> Vec<DeviceEntry> {
  let can_mount = !(d.mounted || (d.encrypted && d.locked));
  let mut out = Vec::new();
  if !d.encrypted || !d.locked {
    out.push(DeviceEntry {
      kind: ActionKind::Open,
      device: d.clone(),
    });
  }
  if can_mount {
    out.push(DeviceEntry {
      kind: ActionKind::Mount,
      device: d.clone(),
    });
  }
  if d.mounted {
    out.push(DeviceEntry {
      kind: ActionKind::Unmount,
      device: d.clone(),
    });
  }
  if d.encrypted && d.locked {
    out.push(DeviceEntry {
      kind: ActionKind::Unlock,
      device: d.clone(),
    });
  }
  if d.encrypted && !d.locked && d.mounted {
    out.push(DeviceEntry {
      kind: ActionKind::Lock,
      device: d.clone(),
    });
  }
  if d.ejectable {
    out.push(DeviceEntry {
      kind: ActionKind::Eject,
      device: d.clone(),
    });
  }
  if d.can_power_off {
    out.push(DeviceEntry {
      kind: ActionKind::PowerOff,
      device: d.clone(),
    });
  }
  if d.mounted {
    out.push(DeviceEntry {
      kind: ActionKind::Copy,
      device: d.clone(),
    });
  }
  out
}

fn action_label(kind: ActionKind) -> &'static str {
  let (en, pt) = match kind {
    ActionKind::Open => ("Open", "Abrir"),
    ActionKind::Mount => ("Mount", "Montar"),
    ActionKind::Unmount => ("Unmount", "Desmontar"),
    ActionKind::Eject => ("Eject", "Ejetar"),
    ActionKind::PowerOff => ("Power Off", "Desligar"),
    ActionKind::Lock => ("Lock", "Bloquear"),
    ActionKind::Unlock => ("Unlock", "Desbloquear"),
    ActionKind::Copy => ("Copy path", "Copiar caminho"),
  };
  i18n::tr(en, pt)
}

fn action_tooltip(kind: ActionKind) -> &'static str {
  let (en, pt) = match kind {
    ActionKind::Open => (
      "Open in the configured file manager",
      "Abrir no gerenciador de arquivos configurado",
    ),
    ActionKind::Mount => ("Mount this device", "Montar este dispositivo"),
    ActionKind::Unmount => ("Unmount this device", "Desmontar este dispositivo"),
    ActionKind::Eject => (
      "Unmount and eject this device",
      "Desmontar e ejetar este dispositivo",
    ),
    ActionKind::PowerOff => (
      "Safely power off this drive",
      "Desligar esta unidade com segurança",
    ),
    ActionKind::Lock => (
      "Lock this encrypted volume",
      "Bloquear este volume criptografado",
    ),
    ActionKind::Unlock => (
      "Unlock this encrypted volume",
      "Desbloquear este volume criptografado",
    ),
    ActionKind::Copy => ("Copy the mount path", "Copiar o ponto de montagem"),
  };
  i18n::tr(en, pt)
}

fn device_title(d: &Device) -> String {
  let (en, pt) = state_words(d.encrypted && d.locked, d.mounted);
  let state = i18n::tr(en, pt);
  if d.capacity > 0 {
    format!("{} · {} · {}", d.name, util::human_bytes(d.capacity), state)
  } else {
    format!("{} · {}", d.name, state)
  }
}

fn state_words(locked: bool, mounted: bool) -> (&'static str, &'static str) {
  if locked {
    ("locked", "bloqueado")
  } else if mounted {
    ("mounted", "montado")
  } else {
    ("available", "disponível")
  }
}

#[cfg(test)]
mod tests {
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
