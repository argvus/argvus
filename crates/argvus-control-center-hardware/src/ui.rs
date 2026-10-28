//! Terminal UI rendering and interaction of the Hardware page.
use crate::{
  backend,
  model::{HardwarePage, HardwareSnapshot, format_kib},
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  capabilities::Capabilities,
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
  privileged::{PrivilegedOperation, PrivilegedRequest, SystemSettingsOperation},
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  buttons::{Button, ButtonKind},
  components::{
    ConfirmationDialog, ConfirmationOutcome, ConfirmationState, StatusKind, StatusMessage,
    draw_confirmation,
  },
  page::{Selection, list, readonly, shell, status},
};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Constraint, Layout},
  text::Line,
};

mod input;
mod jobs;
mod render;
mod rows;

use jobs::{run_governor, run_profile};

#[derive(Debug, Clone)]
enum Pending {
  Action(HardwareAction),
}

#[derive(Debug, Clone)]
enum HardwareAction {
  SetGovernor(String),
  SetProfile(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActionButton {
  ApplyGovernor,
  ApplyProfile,
}

pub struct HardwareApp {
  pub page: HardwarePage,
  selected: Selection,
  on_buttons: bool,
  button_selected: usize,
  button_from: Option<usize>,
  pub snapshot: HardwareSnapshot,
  job: Option<JobHandle<HardwareSnapshot>>,
  action: Option<JobHandle<String>>,
  pending: Option<Pending>,
  confirmation: ConfirmationState,
  manager: JobManager,
  cap: Capabilities,
  pub lang: Lang,
  pub theme: Theme,
  pub status: Option<StatusMessage>,
}

impl HardwareApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme, cap: Capabilities) -> Self {
    let mut app = Self {
      page: HardwarePage::Home,
      selected: Selection::default(),
      on_buttons: false,
      button_selected: 0,
      button_from: None,
      snapshot: Default::default(),
      job: None,
      action: None,
      pending: None,
      confirmation: ConfirmationState::default(),
      manager: JobManager::default(),
      cap,
      lang,
      theme,
      status: None,
    };
    app.refresh();
    app
  }

  fn normalize(&mut self) {
    self.selected.normalize(self.selection_len());
  }

  fn navigate_back(&mut self) {
    self.page = match self.page {
      HardwarePage::DeviceDetail(_) => HardwarePage::Devices,
      HardwarePage::GpuDetail(_) => HardwarePage::Gpu,
      _ => HardwarePage::Home,
    };
    self.selected.index = 0;
    self.on_buttons = false;
  }

  fn execute_action(&mut self, action: HardwareAction) {
    let cap = self.cap.clone();
    match action {
      HardwareAction::SetGovernor(gov) => {
        self.status = Some(StatusMessage {
          kind: StatusKind::Info,
          text: tr(self.lang, "control_center.changing_governor").into(),
        });
        if cap.has_power_profiles_daemon {
          let profile = match gov.as_str() {
            "performance" => "performance".to_string(),
            "powersave" => "power-saver".to_string(),
            _ => gov.clone(),
          };
          self.action = Some(self.manager.spawn(move |_| run_profile(&profile)));
        } else {
          self.action = Some(self.manager.spawn(move |_| run_governor(&gov)));
        }
      }
      HardwareAction::SetProfile(prof) => {
        self.status = Some(StatusMessage {
          kind: StatusKind::Info,
          text: tr(self.lang, "control_center.changing_profile").into(),
        });
        self.action = Some(self.manager.spawn(move |_| run_profile(&prof)));
      }
    }
  }

  fn trigger_button(&mut self, btn: ActionButton) {
    match btn {
      ActionButton::ApplyGovernor => {
        if let Some(gov) = self.snapshot.cpu.governors.get(self.selected.index) {
          self.pending = Some(Pending::Action(HardwareAction::SetGovernor(gov.clone())));
          self.confirmation = ConfirmationState::default();
        }
      }
      ActionButton::ApplyProfile => {
        if let Some(prof) = self.snapshot.energy.profiles.get(self.selected.index) {
          self.pending = Some(Pending::Action(HardwareAction::SetProfile(prof.clone())));
          self.confirmation = ConfirmationState::default();
        }
      }
    }
  }

  fn buttons(&self) -> Vec<(ActionButton, Button)> {
    match self.page {
      HardwarePage::Cpu if !self.snapshot.cpu.governors.is_empty() => vec![(
        ActionButton::ApplyGovernor,
        Button::new(
          tr(self.lang, "control_center.apply_governor"),
          ButtonKind::Primary,
        ),
      )],
      HardwarePage::Power if !self.snapshot.energy.profiles.is_empty() => vec![(
        ActionButton::ApplyProfile,
        Button::new(
          tr(self.lang, "control_center.apply_profile"),
          ButtonKind::Primary,
        ),
      )],
      _ => vec![],
    }
  }

  fn label(&self, page: HardwarePage) -> &'static str {
    match page {
      HardwarePage::Summary => tr(self.lang, "control_center.summary"),
      HardwarePage::Cpu => tr(self.lang, "control_center.hardware_cpu"),
      HardwarePage::Gpu => tr(self.lang, "control_center.hardware_gpu"),
      HardwarePage::Memory => tr(self.lang, "control_center.memory"),
      HardwarePage::Power => tr(self.lang, "control_center.power"),
      HardwarePage::Devices => tr(self.lang, "control_center.devices"),
      HardwarePage::Home => tr(self.lang, "control_center.hardware"),
      HardwarePage::DeviceDetail(_) => tr(self.lang, "control_center.details"),
      HardwarePage::GpuDetail(_) => tr(self.lang, "control_center.gpu_details"),
    }
  }
}

fn na(lang: Lang) -> String {
  tr(lang, "control_center.unavailable_6a8fc3").into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn home_selection_opens_cpu_without_render_time_probe() {
    let mut app = HardwareApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.job = None;
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, HardwarePage::Cpu);
  }

  #[test]
  fn cpu_governor_selection_is_bounded() {
    let mut app = HardwareApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.job = None;
    app.snapshot.cpu.governors = vec!["schedutil".into(), "performance".into()];
    app.page = HardwarePage::Cpu;
    app.handle(KeyCode::End);
    assert_eq!(app.selected.index, 1);
    app.handle(KeyCode::Down);
    assert_eq!(app.selected.index, 1);
  }

  #[test]
  fn cpu_governor_enter_dispatches_even_with_power_profiles_daemon() {
    let cap = Capabilities {
      has_power_profiles_daemon: true,
      ..Capabilities::default()
    };
    let mut app = HardwareApp::new(Lang::for_locale("en-US"), Theme::load(), cap);
    app.job = None;
    app.action = None;
    app.snapshot.cpu.governors = vec!["performance".into(), "powersave".into()];
    app.page = HardwarePage::Cpu;
    app.handle(KeyCode::Enter);
    assert!(
      app.pending.is_some(),
      "Enter on a CPU governor must trigger confirmation dialog"
    );
  }

  #[test]
  fn cpu_governor_enter_does_not_dispatch_when_no_governors() {
    let mut app = HardwareApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.job = None;
    app.action = None;
    app.page = HardwarePage::Cpu;
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
  }
}
