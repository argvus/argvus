//! Pointer and keyboard handling of the popup: hover intent for the submenu,
//! click-outside dismissal and keyboard navigation.

use super::*;

/// Attaches the motion, click and key controllers to the popup surface.
pub(super) fn connect(menu: &Rc<Menu>) {
  let motion = gtk::EventControllerMotion::new();
  let weak = Rc::downgrade(menu);
  motion.connect_motion(move |_, x, y| {
    if let Some(menu) = weak.upgrade() {
      menu.pointer_moved(x, y);
    }
  });
  menu.fixed.add_controller(motion);

  // A press outside both menus dismisses the popup.
  let outside_click = gtk::GestureClick::new();
  outside_click.set_button(0);
  let weak = Rc::downgrade(menu);
  outside_click.connect_pressed(move |_, _, x, y| {
    if let Some(menu) = weak.upgrade()
      && !menu.contains(&menu.main_menu, x, y)
      && !menu.contains(&menu.submenu, x, y)
    {
      menu.quit();
    }
  });
  menu.fixed.add_controller(outside_click);

  let keys = gtk::EventControllerKey::new();
  let weak = Rc::downgrade(menu);
  keys.connect_key_pressed(move |_, keyval, _, _| match weak.upgrade() {
    Some(menu) => menu.key_pressed(keyval),
    None => glib::Propagation::Proceed,
  });
  menu.overlay.add_controller(keys);
}

impl Menu {
  /// Hover rules, as in desktop menus: the submenu stays open while the
  /// pointer is on it or anywhere outside the menus, and follows another row
  /// only after `SUBMENU_SWITCH_DELAY`, so a diagonal path towards the
  /// submenu does not switch or close it.
  fn pointer_moved(self: &Rc<Self>, x: f64, y: f64) {
    if self.contains(&self.submenu, x, y) {
      self.cancel_pending_hover();
      // The row that owns the submenu stays the selected one while the
      // pointer is in its submenu.
      if let Some(open) = self.open_device.get() {
        self
          .main_selection
          .set(self.main_row_index(MainRowKind::Device(open)));
      }
      self.action_selection.set(self.action_row_at(y));
      self.refresh_highlight();
      return;
    }
    if !self.contains(&self.main_menu, x, y) {
      self.cancel_pending_hover();
      return;
    }
    // Padding and the separator keep the current state.
    let Some(row_index) = self.main_row_at(y) else {
      return;
    };
    let Some(kind) = self.main_rows.borrow().get(row_index).map(|row| row.kind) else {
      return;
    };

    self.main_selection.set(Some(row_index));
    self.action_selection.set(None);
    self.refresh_highlight();

    match (kind, self.open_device.get()) {
      (MainRowKind::Device(index), Some(open)) if index == open => self.cancel_pending_hover(),
      (MainRowKind::Device(index), None) => self.open_submenu(index),
      (MainRowKind::Device(index), Some(_)) => self.schedule_hover(PendingHover::Open(index)),
      (MainRowKind::Refresh | MainRowKind::Quit, Some(_)) => {
        self.schedule_hover(PendingHover::Close)
      }
      (MainRowKind::Refresh | MainRowKind::Quit, None) => self.cancel_pending_hover(),
    }
  }

  /// Applies `change` after the hover delay unless the pointer moves on. A
  /// change that is already waiting keeps its original deadline.
  fn schedule_hover(self: &Rc<Self>, change: PendingHover) {
    if self
      .pending_hover
      .borrow()
      .as_ref()
      .is_some_and(|(pending, _)| *pending == change)
    {
      return;
    }
    self.cancel_pending_hover();
    let weak = Rc::downgrade(self);
    let source = glib::timeout_add_local_once(SUBMENU_SWITCH_DELAY, move || {
      let Some(menu) = weak.upgrade() else {
        return;
      };
      // The source has fired; forget it without removing it.
      menu.pending_hover.borrow_mut().take();
      match change {
        PendingHover::Open(index) => menu.open_submenu(index),
        PendingHover::Close => menu.close_submenu(),
      }
    });
    *self.pending_hover.borrow_mut() = Some((change, source));
  }

  pub(super) fn cancel_pending_hover(&self) {
    if let Some((_, source)) = self.pending_hover.borrow_mut().take() {
      source.remove();
    }
  }

  fn main_row_index(&self, kind: MainRowKind) -> Option<usize> {
    self
      .main_rows
      .borrow()
      .iter()
      .position(|row| row.kind == kind)
  }

  /// Keyboard navigation. In the main menu: Up/Down select, Right/Enter/Space
  /// open the submenu or run the row, Escape closes. In the submenu: Up/Down
  /// select, Enter/Space run the action, Left/Escape return to the main menu.
  fn key_pressed(self: &Rc<Self>, key: gdk::Key) -> glib::Propagation {
    self.cancel_pending_hover();
    let in_submenu = self.submenu.is_visible() && self.action_selection.get().is_some();
    let handled = if in_submenu {
      self.submenu_key(key)
    } else {
      self.main_menu_key(key)
    };
    if handled {
      glib::Propagation::Stop
    } else {
      glib::Propagation::Proceed
    }
  }

  fn main_menu_key(self: &Rc<Self>, key: gdk::Key) -> bool {
    let row_count = self.main_rows.borrow().len();
    match key {
      gdk::Key::Escape => self.quit(),
      gdk::Key::Down | gdk::Key::Up => {
        let step = if key == gdk::Key::Down { 1 } else { -1 };
        let next = cycle(self.main_selection.get(), row_count, step);
        self.main_selection.set(next);
        // A submenu that belongs to another row closes, as in desktop menus.
        let selected_kind =
          next.and_then(|index| self.main_rows.borrow().get(index).map(|row| row.kind));
        let keeps_submenu = self
          .open_device
          .get()
          .is_some_and(|open| selected_kind == Some(MainRowKind::Device(open)));
        if !keeps_submenu {
          self.close_submenu();
        }
        self.refresh_highlight();
      }
      gdk::Key::Right | gdk::Key::Return | gdk::Key::KP_Enter | gdk::Key::space => {
        let selected = self.main_selection.get().or(
          self
            .open_device
            .get()
            .and_then(|open| self.main_row_index(MainRowKind::Device(open))),
        );
        let Some(kind) =
          selected.and_then(|index| self.main_rows.borrow().get(index).map(|row| row.kind))
        else {
          // Nothing selected yet: the first key selects the first row.
          self.main_selection.set(cycle(None, row_count, 1));
          self.refresh_highlight();
          return true;
        };
        // Right only enters submenus; it never runs Refresh or Quit.
        if key == gdk::Key::Right && !matches!(kind, MainRowKind::Device(_)) {
          return true;
        }
        self.main_selection.set(selected);
        self.activate_main_row(kind, true);
      }
      _ => return false,
    }
    true
  }

  fn submenu_key(self: &Rc<Self>, key: gdk::Key) -> bool {
    let action_count = self.action_rows.borrow().len();
    match key {
      gdk::Key::Left | gdk::Key::Escape => self.close_submenu(),
      gdk::Key::Down | gdk::Key::Up => {
        let step = if key == gdk::Key::Down { 1 } else { -1 };
        self
          .action_selection
          .set(cycle(self.action_selection.get(), action_count, step));
        self.refresh_highlight();
      }
      gdk::Key::Return | gdk::Key::KP_Enter | gdk::Key::space => {
        if let Some(index) = self.action_selection.get() {
          self.run_action(index);
        }
      }
      _ => return false,
    }
    true
  }
}

/// Moves `current` by `step` through `count` rows, wrapping at both ends.
/// Without a current row, Down starts at the first row and Up at the last.
fn cycle(current: Option<usize>, count: usize, step: isize) -> Option<usize> {
  if count == 0 {
    return None;
  }
  let next = match current {
    None if step > 0 => 0,
    None => count - 1,
    Some(index) => (index as isize + step).rem_euclid(count as isize) as usize,
  };
  Some(next)
}

#[cfg(test)]
mod tests {
  use super::cycle;

  #[test]
  fn selection_wraps_in_both_directions() {
    assert_eq!(cycle(None, 3, 1), Some(0));
    assert_eq!(cycle(None, 3, -1), Some(2));
    assert_eq!(cycle(Some(2), 3, 1), Some(0));
    assert_eq!(cycle(Some(0), 3, -1), Some(2));
    assert_eq!(cycle(Some(0), 0, 1), None);
  }
}
