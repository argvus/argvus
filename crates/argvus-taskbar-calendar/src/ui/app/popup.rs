//! Popup window placement. The geometry, the layer-shell setup and the
//! click-outside dismissal live in `argvus-taskbar-popup`, shared with ARGVUS
//! Focus and ARGVUS Clock.

use argvus_taskbar_popup::PopupController;

use super::drafts::cancel_editor;
use super::*;

pub(super) fn hide_popup(view: &mut ViewState, window: &gtk::Window, controller: &PopupController) {
  controller.hide(window);
  cancel_editor(view);
  window.set_visible(false);
  relm4::main_application().quit();
}

pub(super) fn present_popup(
  window: &gtk::Window,
  popup: &gtk::Box,
  controller: &Rc<PopupController>,
  fixed_position: Option<PopupAnchor>,
) {
  let window = window.clone();
  let popup = popup.clone();
  let controller = Rc::clone(controller);
  let hold = relm4::main_application().hold();
  argvus_taskbar_popup::close_competing_surfaces(
    argvus_taskbar_popup::PopupApp::Calendar,
    move || {
      let _hold = hold;
      controller.present(&window, &popup, fixed_position, POPUP_SIZE);
    },
  );
}

pub(super) fn apply_layer_shell(window: &gtk::Window) {
  PopupController::apply_layer_shell(window, "argvus-taskbar-calendar");
}
