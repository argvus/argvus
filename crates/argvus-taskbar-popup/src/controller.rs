//! The layer-shell surface of a popup and its dismissal.
//!
//! Lifecycle of one presentation: the keyboard is taken exclusively so the
//! popup gets the focus, and after [`ARM_DELAY`] it is released to on-demand;
//! from then on, losing the focus while the pointer is outside the popup
//! dismisses it. A click on the taskbar icon that closes the popup this way
//! would otherwise reopen it right away, so a toggle request within
//! [`REOPEN_GUARD`] of such a dismissal is reported by
//! [`PopupController::reopen_suppressed`].

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk::glib;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::geometry::{PopupAnchor, PopupBounds, PopupSize, pointer_is_outside, popup_position};

/// Default time before focus-loss dismissal is armed.
pub const ARM_DELAY: Duration = Duration::from_millis(800);
/// A toggle this soon after a click-outside dismissal is the same click.
pub const REOPEN_GUARD: Duration = Duration::from_millis(400);

#[derive(Default)]
pub struct PopupController {
  bounds: Cell<Option<PopupBounds>>,
  dismiss_armed: Cell<bool>,
  presentation_id: Cell<u64>,
  pointer_dismiss_delay: Cell<Option<Duration>>,
  pointer_dismiss_generation: Cell<u64>,
  pointer_dismiss_pending: Cell<bool>,
  dismiss_handler: RefCell<Option<Rc<dyn Fn()>>>,
  /// Whether the window gained the keyboard focus since the current
  /// presentation. A spurious focus-out before the compositor ever granted
  /// focus (for example when the keyboard mode changes) must not be taken
  /// for a click outside.
  focused_once: Cell<bool>,
  dismissed_at: Cell<Option<Instant>>,
}

impl PopupController {
  pub fn new() -> Rc<Self> {
    Rc::default()
  }

  /// Makes `window` a layer-shell overlay with the given namespace. Margins
  /// are relative to the full monitor, so the exclusive zone is -1: with zone
  /// 0 the compositor would add the panels' reserved area again.
  pub fn apply_layer_shell(window: &gtk::Window, namespace: &str) {
    window.init_layer_shell();
    window.set_namespace(Some(namespace));
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_exclusive_zone(-1);
    for edge in [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left] {
      window.set_anchor(edge, false);
      window.set_margin(edge, 0);
    }
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
  }

  /// Shows the popup next to the taskbar. `content` is the widget whose size
  /// the window follows.
  pub fn present(
    self: &Rc<Self>,
    window: &gtk::Window,
    content: &impl IsA<gtk::Widget>,
    anchor: Option<PopupAnchor>,
    size: PopupSize,
  ) {
    self.present_with_dismiss_delay(window, content, anchor, size, ARM_DELAY);
  }

  /// Shows the popup and lets its caller choose how long focus loss remains
  /// unarmed before an outside click can dismiss it.
  pub fn present_with_dismiss_delay(
    self: &Rc<Self>,
    window: &gtk::Window,
    content: &impl IsA<gtk::Widget>,
    anchor: Option<PopupAnchor>,
    size: PopupSize,
    dismiss_delay: Duration,
  ) {
    let presentation_id = self.presentation_id.get().wrapping_add(1);
    self.presentation_id.set(presentation_id);
    self.cancel_pointer_dismiss();
    self.dismiss_armed.set(false);
    self.focused_once.set(false);
    let anchor = anchor.or_else(PopupAnchor::at_pointer);
    set_monitor_at(window, anchor);
    self.bounds.set(Some(position_window(window, anchor, size)));
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    content.queue_resize();
    window.queue_resize();
    window.present();

    self.schedule_pointer_dismiss_if_outside(window);

    // The real size is known after the first frame: keep the bounds exact,
    // so the click-outside test uses the popup's true rectangle.
    let state = Rc::clone(self);
    window.add_tick_callback(move |window, _| {
      if !window.is_visible() || state.presentation_id.get() != presentation_id {
        return glib::ControlFlow::Break;
      }
      let (width, height) = (window.width(), window.height());
      if width > 0
        && height > 0
        && let Some((x, y, old_width, old_height)) = state.bounds.get()
      {
        let y = if window.is_anchor(Edge::Bottom) {
          y + old_height - height
        } else {
          y
        };
        state
          .bounds
          .set(Some((x + old_width - width, y, width, height)));
      }
      glib::ControlFlow::Continue
    });
    window.grab_focus();
    window.queue_draw();

    let click_outside = window.clone();
    let state = Rc::clone(self);
    glib::timeout_add_local_once(dismiss_delay, move || {
      if click_outside.is_visible() && state.presentation_id.get() == presentation_id {
        click_outside.set_keyboard_mode(KeyboardMode::OnDemand);
        state.dismiss_armed.set(true);
      }
    });
  }

  /// Hides the popup and invalidates the running presentation.
  pub fn hide(&self, window: &gtk::Window) {
    self
      .presentation_id
      .set(self.presentation_id.get().wrapping_add(1));
    self.dismiss_armed.set(false);
    self.cancel_pointer_dismiss();
    self.bounds.set(None);
    window.set_visible(false);
  }

  /// Dismisses the popup after the pointer has stayed outside it for `delay`.
  /// Entering the popup cancels the countdown; leaving it starts a new one.
  /// This is used by Clock and Focus; other popups can keep focus dismissal.
  pub fn connect_pointer_dismiss(
    self: &Rc<Self>,
    window: &gtk::Window,
    delay: Duration,
    on_dismiss: impl Fn() + 'static,
  ) {
    self.pointer_dismiss_delay.set(Some(delay));
    *self.dismiss_handler.borrow_mut() = Some(Rc::new(on_dismiss));

    let state = Rc::clone(self);
    let motion = gtk::EventControllerMotion::new();
    motion.connect_enter(move |_, _, _| {
      state.cancel_pointer_dismiss();
    });

    let state = Rc::clone(self);
    let window_weak = window.downgrade();
    motion.connect_leave(move |_| {
      let Some(window) = window_weak.upgrade() else {
        return;
      };
      state.schedule_pointer_dismiss_if_outside(&window);
    });
    window.add_controller(motion);

    let state = Rc::clone(self);
    window.connect_has_focus_notify(move |window| {
      if window.has_focus() {
        state.focused_once.set(true);
      } else if window.is_visible() && state.dismiss_armed.get() && state.focused_once.get() {
        state.schedule_pointer_dismiss_if_outside(window);
      }
    });
  }

  /// Calls `on_dismiss` when the window loses the focus with the pointer
  /// outside it.
  pub fn connect_focus_dismiss(
    self: &Rc<Self>,
    window: &gtk::Window,
    on_dismiss: impl Fn() + 'static,
  ) {
    let state = Rc::clone(self);
    window.connect_has_focus_notify(move |window| {
      if window.has_focus() {
        state.focused_once.set(true);
      } else if window.is_visible()
        && state.dismiss_armed.get()
        && state.focused_once.get()
        && pointer_is_outside(state.bounds.get())
      {
        state.dismissed_at.set(Some(Instant::now()));
        on_dismiss();
      }
    });
  }

  /// Whether a toggle now is the click that just dismissed the popup.
  pub fn reopen_suppressed(&self) -> bool {
    self
      .dismissed_at
      .get()
      .is_some_and(|at| at.elapsed() < REOPEN_GUARD)
  }

  /// The popup's rectangle in desktop coordinates, while it is shown.
  pub fn bounds(&self) -> Option<PopupBounds> {
    self.bounds.get()
  }

  fn schedule_pointer_dismiss_if_outside(self: &Rc<Self>, window: &gtk::Window) {
    let Some(delay) = self.pointer_dismiss_delay.get() else {
      return;
    };
    if !window.is_visible() || !pointer_is_outside(self.bounds.get()) {
      return;
    }
    if self.pointer_dismiss_pending.replace(true) {
      return;
    }

    let generation = self.pointer_dismiss_generation.get().wrapping_add(1);
    self.pointer_dismiss_generation.set(generation);
    let presentation_id = self.presentation_id.get();
    let window_weak = window.downgrade();
    let state = Rc::clone(self);
    glib::timeout_add_local_once(delay, move || {
      if state.pointer_dismiss_generation.get() != generation
        || state.presentation_id.get() != presentation_id
      {
        return;
      }
      state.pointer_dismiss_pending.set(false);
      let Some(window) = window_weak.upgrade() else {
        return;
      };
      if window.is_visible() && pointer_is_outside(state.bounds.get()) {
        state.dismissed_at.set(Some(Instant::now()));
        let on_dismiss = state.dismiss_handler.borrow().clone();
        if let Some(on_dismiss) = on_dismiss {
          on_dismiss();
        }
      }
    });
  }

  fn cancel_pointer_dismiss(&self) {
    self.pointer_dismiss_pending.set(false);
    self
      .pointer_dismiss_generation
      .set(self.pointer_dismiss_generation.get().wrapping_add(1));
  }
}

/// Moves the popup to the monitor that holds the anchor.
pub fn set_monitor_at(window: &gtk::Window, anchor: Option<PopupAnchor>) {
  let Some(anchor) = anchor else {
    return;
  };
  let Some(display) = gtk::gdk::Display::default() else {
    return;
  };
  let monitors = display.monitors();
  for index in 0..monitors.n_items() {
    let Some(monitor) = monitors
      .item(index)
      .and_then(|item| item.downcast::<gtk::gdk::Monitor>().ok())
    else {
      continue;
    };
    let geometry = monitor.geometry();
    if anchor.x >= geometry.x()
      && anchor.x < geometry.x() + geometry.width()
      && anchor.y >= geometry.y()
      && anchor.y < geometry.y() + geometry.height()
    {
      window.set_monitor(Some(&monitor));
      return;
    }
  }
}

/// Sets anchors and margins for the popup position and returns its bounds.
pub fn position_window(
  window: &gtk::Window,
  anchor: Option<PopupAnchor>,
  size: PopupSize,
) -> PopupBounds {
  let position = popup_position(anchor, size);
  window.set_anchor(Edge::Top, !position.above);
  window.set_anchor(Edge::Bottom, position.above);
  window.set_margin(Edge::Bottom, position.margin_bottom);
  window.set_margin(Edge::Left, 0);
  window.set_margin(Edge::Top, position.margin_top);
  window.set_margin(Edge::Right, position.margin_right);
  position.bounds
}
