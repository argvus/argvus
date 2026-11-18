//! Wayland input protection scoped to the mapped Lifecycle menu.
use gtk::{gdk, gio, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
struct Protection {
  shields: Vec<(gdk::Monitor, gtk::Window)>,
  monitor_handler: Option<gtk::glib::SignalHandlerId>,
}

pub fn connect(window: &gtk::ApplicationWindow, monitors: &gio::ListModel, primary: &gdk::Monitor) {
  let protection = Rc::new(RefCell::new(Protection::default()));
  let mapped = protection.clone();
  let outputs = monitors.clone();
  let primary = primary.clone();
  window.connect_map(move |window| {
    if let Some(surface) = window.surface().and_downcast::<gdk::Toplevel>() {
      surface.inhibit_system_shortcuts(None::<gdk::Event>);
    }
    reconcile(&mut mapped.borrow_mut(), &outputs, &primary);
    let state = Rc::downgrade(&mapped);
    let primary = primary.clone();
    mapped.borrow_mut().monitor_handler =
      Some(outputs.connect_items_changed(move |outputs, _, _, _| {
        if let Some(state) = state.upgrade() {
          reconcile(&mut state.borrow_mut(), outputs, &primary);
        }
      }));
  });
  let outputs = monitors.clone();
  window.connect_unmap(move |window| {
    let mut protection = protection.borrow_mut();
    if let Some(handler) = protection.monitor_handler.take() {
      outputs.disconnect(handler);
    }
    for (_, shield) in protection.shields.drain(..) {
      shield.destroy();
    }
    if let Some(surface) = window.surface().and_downcast::<gdk::Toplevel>()
      && surface.is_shortcuts_inhibited()
    {
      surface.restore_system_shortcuts();
    }
  });
}

fn reconcile(protection: &mut Protection, outputs: &gio::ListModel, primary: &gdk::Monitor) {
  let monitors: Vec<_> = (0..outputs.n_items())
    .filter_map(|index| outputs.item(index).and_downcast::<gdk::Monitor>())
    .collect();
  protection.shields.retain(|(monitor, shield)| {
    if monitors.contains(monitor) {
      true
    } else {
      shield.destroy();
      false
    }
  });
  if !monitors.contains(primary) {
    return;
  }
  for monitor in monitors.into_iter().filter(|monitor| monitor != primary) {
    if protection
      .shields
      .iter()
      .any(|(existing, _)| existing == &monitor)
    {
      continue;
    }
    let shield = gtk::Window::new();
    shield.init_layer_shell();
    shield.set_namespace(Some("argvus-lifecycle-shield"));
    shield.set_layer(Layer::Overlay);
    shield.set_keyboard_mode(KeyboardMode::None);
    shield.set_exclusive_zone(-1);
    shield.set_monitor(Some(&monitor));
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
      shield.set_anchor(edge, true);
    }
    // A transparent buffer still receives pointer input over the entire surface.
    shield.add_css_class("lifecycle-shield");
    shield.set_focusable(false);
    shield.set_visible(true);
    protection.shields.push((monitor, shield));
  }
}
