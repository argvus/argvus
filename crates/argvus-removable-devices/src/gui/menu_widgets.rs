//! Device menu and action submenu widgets: rows, hover tracking and the signals that run the actions.

use super::entries::{DeviceEntry, action_label, action_tooltip, device_entries, device_title};
use super::*;

pub(super) fn rebuild_menu(main_menu: &gtk::Box, devices: &[Device], ctx: &MenuCtx) {
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
pub(super) fn menu_row(text: &str, has_submenu: bool) -> gtk::Box {
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
pub(super) fn on_release(row: &gtk::Box, action: impl Fn() + 'static) {
  let gesture = gtk::GestureClick::new();
  gesture.set_button(0);
  gesture.connect_released(move |_, _, _, _| action());
  row.add_controller(gesture);
}

pub(super) fn show_device_submenu(row: &gtk::Box, device: &Device, index: usize, ctx: &MenuCtx) {
  set_active_row(ctx, Some(row));
  rebuild_action_menu(device, ctx);
  let submenu_y = ctx.menu_y + index as i32 * MENU_ROW_HEIGHT;
  ctx.submenu_y.set(submenu_y);
  ctx
    .fixed
    .move_(&ctx.submenu, f64::from(ctx.submenu_x), f64::from(submenu_y));
  ctx.submenu.set_visible(true);
}

pub(super) fn set_active_row(ctx: &MenuCtx, row: Option<&gtk::Box>) {
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

pub(super) fn set_hovered_action_row(ctx: &MenuCtx, row: Option<&gtk::Box>) {
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

pub(super) fn update_hover(main_menu: &gtk::Box, x: i32, y: i32, ctx: &MenuCtx) {
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

pub(super) fn widget_contains_pointer(
  widget: &gtk::Box,
  x: i32,
  y: i32,
  overlay: &gtk::Window,
) -> bool {
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

pub(super) fn rebuild_action_menu(device: &Device, ctx: &MenuCtx) {
  clear_menu(&ctx.submenu);
  for entry in device_entries(device) {
    let action = menu_row(action_label(entry.kind), false);
    action.set_tooltip_text(Some(action_tooltip(entry.kind)));
    connect_action(&action, entry, ctx);
    ctx.submenu.append(&action);
  }
}

/// The direct children of a container, in display order.
pub(super) fn children(container: &impl IsA<gtk::Widget>) -> Vec<gtk::Widget> {
  let mut children = Vec::new();
  let mut child = container.as_ref().first_child();
  while let Some(widget) = child {
    child = widget.next_sibling();
    children.push(widget);
  }
  children
}

pub(super) fn clear_menu(container: &gtk::Box) {
  for child in children(container) {
    container.remove(&child);
  }
}

pub(super) fn connect_refresh(item: &gtk::Box, ctx: &MenuCtx) {
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

pub(super) fn connect_action(item: &gtk::Box, entry: DeviceEntry, ctx: &MenuCtx) {
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
