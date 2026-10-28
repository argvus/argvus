//! Device menu and action submenu widgets: building the rows, placing both
//! menus, opening and closing the submenu and painting the highlight.

use super::entries::{action_label, action_tooltip, device_entries, device_title};
use super::placement::{menu_top, submenu_top};
use super::*;

impl Menu {
  /// Replaces the rows of both menus with `devices` and resets the selection.
  pub(super) fn rebuild(self: &Rc<Self>, devices: &[Device]) {
    self.cancel_pending_hover();
    clear_children(&self.main_menu);
    self.close_submenu();
    *self.devices.borrow_mut() = devices.to_vec();
    self.main_selection.set(None);

    let mut rows = Vec::new();
    if devices.is_empty() {
      let empty = menu_row(
        i18n::tr("No removable devices", "Nenhum dispositivo removível"),
        false,
      );
      empty.set_sensitive(false);
      self.main_menu.append(&empty);
    }
    for (index, device) in devices.iter().enumerate() {
      let row = menu_row(&device_title(device), true);
      self.main_menu.append(&row);
      rows.push(self.main_row(row, MainRowKind::Device(index)));
    }

    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
    separator.set_margin_top(2);
    separator.set_margin_bottom(2);
    self.main_menu.append(&separator);

    for (kind, label) in [
      (MainRowKind::Refresh, i18n::tr("Refresh", "Atualizar")),
      (MainRowKind::Quit, i18n::tr("Quit", "Sair")),
    ] {
      let row = menu_row(label, false);
      self.main_menu.append(&row);
      rows.push(self.main_row(row, kind));
    }
    *self.main_rows.borrow_mut() = rows;

    self.place_main_menu();
    self.refresh_highlight();
  }

  /// Wires a main-menu row to its activation and returns its record.
  fn main_row(self: &Rc<Self>, widget: gtk::Box, kind: MainRowKind) -> MainRow {
    let menu = Rc::downgrade(self);
    on_release(&widget, move || {
      if let Some(menu) = menu.upgrade() {
        menu.activate_main_row(kind, false);
      }
    });
    MainRow { widget, kind }
  }

  /// Runs what a main-menu row does. `from_keyboard` also moves the keyboard
  /// selection into the submenu it opens.
  pub(super) fn activate_main_row(self: &Rc<Self>, kind: MainRowKind, from_keyboard: bool) {
    match kind {
      MainRowKind::Device(index) => {
        self.cancel_pending_hover();
        self.open_submenu(index);
        if from_keyboard && !self.action_rows.borrow().is_empty() {
          self.action_selection.set(Some(0));
          self.refresh_highlight();
        }
      }
      MainRowKind::Refresh => self.refresh(),
      MainRowKind::Quit => self.quit(),
    }
  }

  /// Moves the main menu to its anchor using its current natural height, so a
  /// menu above a bottom taskbar keeps its bottom edge when rows change.
  pub(super) fn place_main_menu(&self) {
    let (_, natural_height, _, _) = self
      .main_menu
      .measure(gtk::Orientation::Vertical, MENU_WIDTH);
    let menu_y = menu_top(self.anchor, natural_height, self.area.height);
    self
      .fixed
      .move_(&self.main_menu, f64::from(self.menu_x), f64::from(menu_y));
  }

  /// Shows the actions of `devices[index]` next to its row.
  pub(super) fn open_submenu(self: &Rc<Self>, index: usize) {
    if self.open_device.get() == Some(index) && self.submenu.is_visible() {
      return;
    }
    let Some(device) = self.devices.borrow().get(index).cloned() else {
      return;
    };
    let row_y = self
      .main_row_widget(MainRowKind::Device(index))
      .and_then(|row| row.compute_bounds(&self.fixed))
      .map_or(0, |bounds| bounds.y() as i32);

    self.fill_submenu(&device);
    self.open_device.set(Some(index));
    self.action_selection.set(None);

    let (_, submenu_height, _, _) = self
      .submenu
      .measure(gtk::Orientation::Vertical, ACTION_MENU_WIDTH);
    let submenu_y = submenu_top(row_y, submenu_height, self.area.height);
    self.fixed.move_(
      &self.submenu,
      f64::from(self.submenu_x),
      f64::from(submenu_y),
    );
    self.submenu.set_visible(true);
    self.refresh_highlight();
  }

  pub(super) fn close_submenu(&self) {
    self.submenu.set_visible(false);
    clear_children(&self.submenu);
    self.action_rows.borrow_mut().clear();
    self.open_device.set(None);
    self.action_selection.set(None);
    self.refresh_highlight();
  }

  fn fill_submenu(self: &Rc<Self>, device: &Device) {
    clear_children(&self.submenu);
    let mut rows = Vec::new();
    for (index, entry) in device_entries(device).into_iter().enumerate() {
      let widget = menu_row(action_label(entry.kind), false);
      widget.set_tooltip_text(Some(action_tooltip(entry.kind)));
      let menu = Rc::downgrade(self);
      on_release(&widget, move || {
        if let Some(menu) = menu.upgrade() {
          menu.run_action(index);
        }
      });
      self.submenu.append(&widget);
      rows.push(ActionRow { widget, entry });
    }
    *self.action_rows.borrow_mut() = rows;
  }

  fn main_row_widget(&self, kind: MainRowKind) -> Option<gtk::Box> {
    self
      .main_rows
      .borrow()
      .iter()
      .find(|row| row.kind == kind)
      .map(|row| row.widget.clone())
  }

  /// Paints the selection state on the rows. The classes are the ones the
  /// theme stylesheets already style, so themes need no change.
  pub(super) fn refresh_highlight(&self) {
    let open_device = self.open_device.get();
    let main_selection = self.main_selection.get();
    for (index, row) in self.main_rows.borrow().iter().enumerate() {
      let owns_submenu = open_device.is_some_and(|open| row.kind == MainRowKind::Device(open));
      set_class(&row.widget, "submenu-open", owns_submenu);
      set_class(
        &row.widget,
        "pointer-over",
        !owns_submenu && main_selection == Some(index),
      );
    }
    let action_selection = self.action_selection.get();
    for (index, row) in self.action_rows.borrow().iter().enumerate() {
      set_class(&row.widget, "pointer-over", action_selection == Some(index));
    }
  }

  /// Whether the point (surface coordinates) is inside `widget`'s box.
  pub(super) fn contains(&self, widget: &gtk::Box, x: f64, y: f64) -> bool {
    widget.is_visible()
      && widget.compute_bounds(&self.fixed).is_some_and(|bounds| {
        let (x, y) = (x as f32, y as f32);
        x >= bounds.x()
          && x < bounds.x() + bounds.width()
          && y >= bounds.y()
          && y < bounds.y() + bounds.height()
      })
  }

  /// The main-menu row whose band contains `y`. Only the vertical band is
  /// tested: the caller already knows the point is inside the menu, and the
  /// row margins must not count as leaving the row.
  pub(super) fn main_row_at(&self, y: f64) -> Option<usize> {
    self
      .main_rows
      .borrow()
      .iter()
      .position(|row| in_vertical_band(&row.widget, &self.fixed, y))
  }

  /// The submenu action whose band contains `y`.
  pub(super) fn action_row_at(&self, y: f64) -> Option<usize> {
    self
      .action_rows
      .borrow()
      .iter()
      .position(|row| in_vertical_band(&row.widget, &self.fixed, y))
  }
}

fn in_vertical_band(widget: &gtk::Box, fixed: &gtk::Fixed, y: f64) -> bool {
  widget.compute_bounds(fixed).is_some_and(|bounds| {
    let y = y as f32;
    y >= bounds.y() && y < bounds.y() + bounds.height()
  })
}

fn set_class(widget: &gtk::Box, class: &str, enabled: bool) {
  if enabled {
    widget.add_css_class(class);
  } else {
    widget.remove_css_class(class);
  }
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

fn clear_children(container: &gtk::Box) {
  while let Some(child) = container.first_child() {
    container.remove(&child);
  }
}
