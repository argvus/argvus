//! Drawing of the Hardware Overview page: lists, overlays, footer hints and the breadcrumb.

use super::rows::detail_lines;
use super::*;

impl HardwareApp {
  pub fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Hardware.label(self.lang),
      tr(self.lang, "control_center.hardware_overview")
    );
    if self.page == HardwarePage::Home {
      root
    } else {
      format!("{root} > {}", self.label(self.page))
    }
  }

  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let hints = if self.on_buttons {
      tr(
        self.lang,
        "control_center.select_button_enter_confirm_tab_switch_focus_esc_back",
      )
    } else if matches!(
      self.page,
      HardwarePage::Summary
        | HardwarePage::Memory
        | HardwarePage::GpuDetail(_)
        | HardwarePage::DeviceDetail(_)
    ) {
      tr(self.lang, "control_center.scroll_r_refresh_esc_back_help")
    } else if matches!(self.page, HardwarePage::Cpu) {
      tr(
        self.lang,
        "control_center.navigate_enter_g_apply_tab_focus_buttons_esc_back_r_refresh",
      )
    } else if matches!(self.page, HardwarePage::Power) {
      tr(
        self.lang,
        "control_center.navigate_enter_e_apply_tab_focus_buttons_esc_back_r_refresh",
      )
    } else {
      tr(
        self.lang,
        "control_center.navigate_enter_open_esc_back_r_refresh_help",
      )
    };

    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      hints,
    );
    let buttons = self.buttons();

    let (content_area, button_area) = if buttons.is_empty() {
      (body, None)
    } else {
      let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(body);
      (chunks[0], Some(chunks[1]))
    };

    if matches!(
      self.page,
      HardwarePage::Summary
        | HardwarePage::Memory
        | HardwarePage::GpuDetail(_)
        | HardwarePage::DeviceDetail(_)
    ) {
      readonly(frame, content_area, &self.theme, &detail_lines(self));
    } else {
      let rows = self.rows();
      list(
        frame,
        content_area,
        &self.theme,
        &rows,
        self.selected.index.min(rows.len().saturating_sub(1)),
      );
    }

    if let Some(btn_area) = button_area {
      let raw_buttons: Vec<Button> = buttons.iter().map(|(_, b)| b.clone()).collect();
      let active_idx = if self.on_buttons {
        self.button_selected
      } else {
        usize::MAX
      };
      argvus_tui::buttons::draw(frame, btn_area, &raw_buttons, active_idx, &self.theme);
    }

    if let Some(Pending::Action(action)) = &self.pending {
      let message = match action {
        HardwareAction::SetGovernor(gov) => format!(
          "{}: {}",
          tr(self.lang, "control_center.confirm_cpu_governor_change_to"),
          gov
        ),
        HardwareAction::SetProfile(prof) => format!(
          "{}: {}",
          tr(self.lang, "control_center.confirm_power_profile_change_to"),
          prof
        ),
      };
      draw_confirmation(
        frame,
        area,
        &self.theme,
        ConfirmationDialog {
          title: tr(self.lang, "control_center.confirm_hardware_operation"),
          message: &message,
          confirm_label: tr(self.lang, "control_center.continue"),
          cancel_label: tr(self.lang, "control_center.cancel"),
          confirm_selected: self.confirmation.confirm_selected,
        },
      );
    }

    if let Some(message) = &self.status {
      status(frame, body, &self.theme, message);
    }
  }
}
