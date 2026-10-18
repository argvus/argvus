//! Implements terminal UI rendering and interaction in crate `argvus control center hardware`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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

#[derive(Debug, Clone)]
/// Defines `Pending`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum Pending {
  Action(HardwareAction),
}

#[derive(Debug, Clone)]
/// Defines `HardwareAction`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum HardwareAction {
  SetGovernor(String),
  SetProfile(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `ActionButton`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum ActionButton {
  ApplyGovernor,
  ApplyProfile,
}

/// Represents `HardwareApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `refresh` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn refresh(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let cap = self.cap.clone();
    self.job = Some(self.manager.spawn(move |_| Ok(backend::collect(&cap))));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_hardware").into(),
    });
  }

  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      match result {
        Ok(msg) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: msg,
          });
        }
        Err(err) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: err,
          });
        }
      }
      self.refresh();
      changed = true;
    }
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      match result {
        Ok(snapshot) => {
          self.snapshot = snapshot;
          self.normalize();
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: tr(self.lang, "control_center.hardware_refreshed").into(),
          });
        }
        Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: error,
          });
        }
      }
      changed = true;
    }
    changed
  }

  /// Executes the `busy` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn busy(&self) -> bool {
    self.job.is_some() || self.action.is_some()
  }

  /// Executes the `normalize` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn normalize(&mut self) {
    self.selected.normalize(self.selection_len());
  }

  /// Executes the `selection_len` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn selection_len(&self) -> usize {
    match self.page {
      HardwarePage::Home => 6,
      HardwarePage::Cpu => self.snapshot.cpu.governors.len(),
      HardwarePage::Gpu => self.snapshot.gpus.len(),
      HardwarePage::Power => self.snapshot.energy.profiles.len(),
      HardwarePage::Devices => self.snapshot.devices.len(),
      _ => 1,
    }
  }

  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          if let Some(Pending::Action(act)) = self.pending.take() {
            self.execute_action(act);
          }
        }
        ConfirmationOutcome::Cancelled => {
          self.pending = None;
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }

    if matches!(key, KeyCode::Esc | KeyCode::Left) && !self.on_buttons {
      if self.page == HardwarePage::Home {
        return true;
      }
      self.navigate_back();
      return false;
    }

    if self.busy() {
      return false;
    }

    let buttons = self.buttons();

    if key == KeyCode::Tab && !buttons.is_empty() {
      if self.on_buttons {
        self.on_buttons = false;
        if let Some(old) = self.button_from {
          self.selected.index = old;
        }
      } else {
        self.on_buttons = true;
        self.button_from = Some(self.selected.index);
        self.button_selected = 0;
      }
      return false;
    }

    if self.on_buttons {
      match key {
        KeyCode::Left | KeyCode::Char('h') => {
          self.button_selected = self.button_selected.saturating_sub(1);
        }
        KeyCode::Right | KeyCode::Char('l') => {
          self.button_selected += 1;
          if !buttons.is_empty() {
            self.button_selected = self.button_selected.min(buttons.len() - 1);
          }
        }
        KeyCode::Enter => {
          if let Some((action, _)) = buttons.get(self.button_selected) {
            self.trigger_button(*action);
          }
        }
        KeyCode::Esc => {
          self.on_buttons = false;
          if let Some(old) = self.button_from {
            self.selected.index = old;
          }
        }
        _ => {}
      }
      return false;
    }

    if self.selected.handle(key, self.selection_len(), 8) {
      return false;
    }

    match key {
      KeyCode::Char('r') => self.refresh(),

      KeyCode::Char('g') if self.page == HardwarePage::Cpu => {
        if let Some(gov) = self.snapshot.cpu.governors.get(self.selected.index) {
          self.pending = Some(Pending::Action(HardwareAction::SetGovernor(gov.clone())));
          self.confirmation = ConfirmationState::default();
        }
      }
      KeyCode::Char('e') if self.page == HardwarePage::Power => {
        if let Some(prof) = self.snapshot.energy.profiles.get(self.selected.index) {
          self.pending = Some(Pending::Action(HardwareAction::SetProfile(prof.clone())));
          self.confirmation = ConfirmationState::default();
        }
      }

      KeyCode::Enter | KeyCode::Right => match self.page {
        HardwarePage::Home => {
          self.page = [
            HardwarePage::Summary,
            HardwarePage::Cpu,
            HardwarePage::Gpu,
            HardwarePage::Memory,
            HardwarePage::Power,
            HardwarePage::Devices,
          ][self.selected.index];
          self.selected.index = 0;
        }
        HardwarePage::Devices if self.snapshot.devices.get(self.selected.index).is_some() => {
          self.page = HardwarePage::DeviceDetail(self.selected.index);
        }
        HardwarePage::Gpu if self.snapshot.gpus.get(self.selected.index).is_some() => {
          self.page = HardwarePage::GpuDetail(self.selected.index);
          self.selected.index = 0;
        }
        HardwarePage::Cpu => {
          if let Some(gov) = self.snapshot.cpu.governors.get(self.selected.index) {
            self.pending = Some(Pending::Action(HardwareAction::SetGovernor(gov.clone())));
            self.confirmation = ConfirmationState::default();
          }
        }
        HardwarePage::Power => {
          if let Some(prof) = self.snapshot.energy.profiles.get(self.selected.index) {
            self.pending = Some(Pending::Action(HardwareAction::SetProfile(prof.clone())));
            self.confirmation = ConfirmationState::default();
          }
        }
        _ => {}
      },
      _ => {}
    }
    false
  }

  /// Executes the `navigate_back` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn navigate_back(&mut self) {
    self.page = match self.page {
      HardwarePage::DeviceDetail(_) => HardwarePage::Devices,
      HardwarePage::GpuDetail(_) => HardwarePage::Gpu,
      _ => HardwarePage::Home,
    };
    self.selected.index = 0;
    self.on_buttons = false;
  }

  /// Executes the `execute_action` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `trigger_button` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `buttons` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn rows(&self) -> Vec<String> {
    match self.page {
      HardwarePage::Home => self.home_rows(),
      HardwarePage::Cpu => self
        .snapshot
        .cpu
        .governors
        .iter()
        .map(|gov| {
          let badge = if self.snapshot.cpu.governor.as_deref() == Some(gov) {
            format!("   ★ {}", tr(self.lang, "control_center.active"))
          } else {
            String::new()
          };
          format!("{gov}{badge}")
        })
        .collect(),
      HardwarePage::Gpu => self
        .snapshot
        .gpus
        .iter()
        .map(|gpu| {
          format!(
            "GPU {}  {}  {}",
            gpu.index,
            gpu
              .model
              .as_deref()
              .unwrap_or(tr(self.lang, "control_center.unavailable_6a8fc3")),
            gpu
              .driver
              .as_deref()
              .unwrap_or(tr(self.lang, "control_center.no_driver")),
          )
        })
        .collect(),
      HardwarePage::Power => self
        .snapshot
        .energy
        .profiles
        .iter()
        .map(|prof| {
          let badge = if self.snapshot.energy.profile.as_deref() == Some(prof) {
            format!("   ★ {}", tr(self.lang, "control_center.active"))
          } else {
            String::new()
          };
          format!("{prof}{badge}")
        })
        .collect(),
      HardwarePage::Devices => self
        .snapshot
        .devices
        .iter()
        .map(|dev| format!("[{}] {}", dev.kind, dev.name))
        .collect(),
      _ => vec![],
    }
  }

  /// Executes the `home_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_rows(&self) -> Vec<String> {
    let summary_label = tr(self.lang, "control_center.summary");
    let cpu_label = "CPU";
    let gpu_label = "GPU";
    let memory_label = tr(self.lang, "control_center.memory");
    let power_label = tr(self.lang, "control_center.power");
    let devices_label = tr(self.lang, "control_center.devices");

    let cpu_str = self
      .snapshot
      .cpu
      .model
      .clone()
      .unwrap_or_else(|| na(self.lang));

    let gpu_str = if self.snapshot.gpus.is_empty() {
      tr(self.lang, "control_center.no_gpu").into()
    } else {
      format!("{} GPU(s)", self.snapshot.gpus.len())
    };

    let mem_str = format_kib(self.snapshot.memory.total_kib);

    let power_str = self
      .snapshot
      .energy
      .profile
      .clone()
      .unwrap_or_else(|| tr(self.lang, "control_center.default").into());

    let dev_str = format!(
      "{} {}",
      self.snapshot.devices.len(),
      tr(self.lang, "control_center.devices_a41e65")
    );

    vec![
      format!(
        "{} {}  ·  {} · {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        summary_label,
        self.snapshot.architecture,
        self.snapshot.kernel
      ),
      format!(
        "{} {}  ·  {} · {}",
        AppConfig::icon(argvus_tui::icons::MEMORY),
        cpu_label,
        cpu_str,
        self.snapshot.cpu.governor.as_deref().unwrap_or("—")
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::GPU),
        gpu_label,
        gpu_str
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::POWER),
        memory_label,
        mem_str
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::BATTERY),
        power_label,
        power_str
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::ETHERNET),
        devices_label,
        dev_str
      ),
    ]
  }
}

/// Executes the `detail_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn detail_lines(app: &HardwareApp) -> Vec<Line<'static>> {
  let s = &app.snapshot;
  let mut lines = Vec::new();
  let mut add = |key: &str, value: String| {
    lines.push(Line::from(format!(
      "{:<20} {value}",
      field_label(app.lang, key)
    )))
  };
  match app.page {
    HardwarePage::Summary => {
      add(
        "Manufacturer",
        s.manufacturer.clone().unwrap_or_else(|| na(app.lang)),
      );
      add("Model", s.model.clone().unwrap_or_else(|| na(app.lang)));
      add("Chassis", s.chassis.clone().unwrap_or_else(|| na(app.lang)));
      add("Architecture", s.architecture.clone());
      add("Kernel", s.kernel.clone());
      add("CPU", s.cpu.model.clone().unwrap_or_else(|| na(app.lang)));
      add(
        "Firmware",
        s.firmware.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Threads",
        s.cpu
          .threads
          .map(|v| v.to_string())
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Physical cores",
        s.cpu
          .physical_cores
          .map(|v| v.to_string())
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Sockets",
        s.cpu
          .sockets
          .map(|v| v.to_string())
          .unwrap_or_else(|| na(app.lang)),
      );
      add("GPU(s)", s.gpus.len().to_string());
      add("RAM", format_kib(s.memory.total_kib));
      add("Boot", s.boot.clone());
      add(
        "Virtualization",
        s.virtualization
          .clone()
          .unwrap_or_else(|| tr(app.lang, "control_center.none_detected").into()),
      );
      add(
        "Battery",
        if s.battery.is_some() {
          tr(app.lang, "control_center.present").into()
        } else {
          tr(app.lang, "control_center.none_detected").into()
        },
      );
    }
    HardwarePage::Cpu => {
      add(
        "Vendor",
        s.cpu.vendor.clone().unwrap_or_else(|| na(app.lang)),
      );
      add("Model", s.cpu.model.clone().unwrap_or_else(|| na(app.lang)));
      add(
        "Threads",
        s.cpu
          .threads
          .map(|v| v.to_string())
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Current",
        s.cpu
          .current_mhz
          .map(|v| format!("{v} MHz"))
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Min/Max",
        format!(
          "{} / {} MHz",
          s.cpu
            .min_mhz
            .map(|v| v.to_string())
            .unwrap_or_else(|| na(app.lang)),
          s.cpu
            .max_mhz
            .map(|v| v.to_string())
            .unwrap_or_else(|| na(app.lang))
        ),
      );
      add(
        "Governor",
        s.cpu.governor.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Available",
        if s.cpu.governors.is_empty() {
          na(app.lang)
        } else {
          s.cpu.governors.join(", ")
        },
      );
      add(
        "Scaling driver",
        s.cpu.driver.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Management",
        s.cpu.management.clone().unwrap_or_else(|| na(app.lang)),
      );
    }
    HardwarePage::Gpu | HardwarePage::GpuDetail(_) => {
      let gpus: Vec<&crate::model::GpuInfo> = match app.page {
        HardwarePage::GpuDetail(index) => s.gpus.get(index).into_iter().collect(),
        _ => s.gpus.iter().collect(),
      };
      for gpu in gpus {
        add(
          &format!("GPU {}", gpu.index),
          gpu
            .model
            .clone()
            .or(gpu.vendor.clone())
            .unwrap_or_else(|| na(app.lang)),
        );
        add("PCI", gpu.pci.clone().unwrap_or_else(|| na(app.lang)));
        add(
          "Vendor ID",
          gpu.vendor_id.clone().unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Device ID",
          gpu.device_id.clone().unwrap_or_else(|| na(app.lang)),
        );
        add("Driver", gpu.driver.clone().unwrap_or_else(|| na(app.lang)));
        add("Module", gpu.module.clone().unwrap_or_else(|| na(app.lang)));
        add(
          "Driver status",
          gpu.driver_status.clone().unwrap_or_else(|| na(app.lang)),
        );
        add("DRM", gpu.drm.clone().unwrap_or_else(|| na(app.lang)));
        add("Render", gpu.render.clone().unwrap_or_else(|| na(app.lang)));
        add("OpenGL", gpu.opengl.clone().unwrap_or_else(|| na(app.lang)));
        add("Vulkan", gpu.vulkan.clone().unwrap_or_else(|| na(app.lang)));
      }
      if s.gpus.iter().any(|g| g.driver.as_deref() == Some("vmwgfx")) && s.virtualization.is_some()
      {
        add(
          "Note",
          tr(
            app.lang,
            "control_center.vmwgfx_detected_in_virtual_machine",
          )
          .into(),
        );
      }
      add(
        "OpenGL software",
        if s.software_rendering {
          tr(app.lang, "control_center.active_fb4e81").into()
        } else {
          tr(app.lang, "control_center.inactive").into()
        },
      );
    }
    HardwarePage::Memory => {
      add("Total", format_kib(s.memory.total_kib));
      add("Used", format_kib(s.memory.used_kib));
      add("Available", format_kib(s.memory.available_kib));
      add("Swap total", format_kib(s.memory.swap_total_kib));
      add("Swap free", format_kib(s.memory.swap_free_kib));
    }
    HardwarePage::Power => {
      add(
        "Backend",
        s.energy.backend.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Profile",
        s.energy.profile.clone().unwrap_or_else(|| na(app.lang)),
      );
      if s.energy.profiles.is_empty() {
        add(
          "Profiles",
          tr(app.lang, "control_center.no_profile_available").into(),
        );
      }
      if let Some(battery) = &s.battery {
        add("Battery", battery.device.clone());
        if let Some(manufacturer) = &battery.manufacturer {
          add("Manufacturer", manufacturer.clone());
        }
        if let Some(model) = &battery.model {
          add("Model", model.clone());
        }
        add(
          "Status",
          battery.status.clone().unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Capacity",
          battery
            .percent
            .map(|v| format!("{v}%"))
            .unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Health",
          battery
            .health
            .map(|v| format!("{v:.1}%"))
            .unwrap_or_else(|| na(app.lang)),
        );
        if let Some(ac) = battery.ac_online {
          add(
            "AC",
            if ac {
              tr(app.lang, "control_center.connected_cbc626").into()
            } else {
              tr(app.lang, "control_center.disconnected_53344a").into()
            },
          );
        }
      } else {
        add(
          "Battery",
          tr(app.lang, "control_center.no_battery_detected").into(),
        );
      }
    }
    HardwarePage::Devices => {
      for device in &s.devices {
        add(&device.kind, device.name.clone());
      }
      if s.devices.is_empty() {
        add(
          "Devices",
          tr(app.lang, "control_center.no_device_detected").into(),
        );
      }
    }
    HardwarePage::DeviceDetail(index) => {
      if let Some(device) = s.devices.get(index) {
        add("Name", device.name.clone());
        add("Type", device.kind.clone());
        add(
          "Vendor",
          device.vendor.clone().unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Driver",
          device.driver.clone().unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Product",
          device.product.clone().unwrap_or_else(|| na(app.lang)),
        );
        add("Bus", device.bus.clone().unwrap_or_else(|| na(app.lang)));
        add("Path", device.path.clone().unwrap_or_else(|| na(app.lang)));
      }
    }
    HardwarePage::Home => {}
  }
  lines
}

/// Executes the `run_governor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_governor(value: &str) -> Result<String, String> {
  let request = PrivilegedRequest::new("governor", "set", vec![value.into()])?;
  let executable = std::env::current_exe()
    .map_err(|error| error.to_string())?
    .to_string_lossy()
    .into_owned();
  let output = SystemSettingsOperation::new(SystemProcessRunner, executable).execute(&request)?;
  output
    .status
    .is_some_and(|v| v == 0)
    .then(|| format!("Governor: {value}"))
    .ok_or_else(|| String::from_utf8_lossy(&output.stderr).trim().into())
}

/// Executes the `field_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn field_label(lang: Lang, key: &str) -> String {
  let label = match key {
    "Manufacturer" => "control_center.hardware_manufacturer",
    "Model" => "control_center.hardware_model",
    "Chassis" => "control_center.hardware_chassis",
    "Architecture" => "control_center.hardware_architecture",
    "Kernel" => "control_center.hardware_kernel",
    "CPU" => "control_center.hardware_cpu",
    "Threads" => "control_center.hardware_threads",
    "GPU(s)" => "control_center.hardware_gpu_s",
    "RAM" => "control_center.hardware_ram",
    "Boot" => "control_center.hardware_boot",
    "Virtualization" => "control_center.hardware_virtualization",
    "Battery" => "control_center.hardware_battery",
    "Firmware" => "control_center.hardware_firmware",
    "AC" => "control_center.hardware_ac_power",
    "Management" => "control_center.hardware_management",
    "Vendor" => "control_center.hardware_vendor",
    "Current" => "control_center.hardware_current",
    "Min/Max" => "control_center.hardware_min_max",
    "Governor" => "control_center.hardware_governor",
    "Available" => "control_center.hardware_available",
    "Scaling driver" => "control_center.hardware_scaling_driver",
    "Vendor ID" => "control_center.hardware_vendor_id",
    "Device ID" => "control_center.hardware_device_id",
    "Driver" => "control_center.hardware_driver",
    "DRM" => "control_center.hardware_drm",
    "Render" => "control_center.hardware_render",
    "OpenGL software" => "control_center.hardware_opengl_software",
    "Total" => "control_center.hardware_total",
    "Used" => "control_center.hardware_used",
    "Swap total" => "control_center.hardware_swap_total",
    "Swap free" => "control_center.hardware_swap_free",
    "Backend" => "control_center.hardware_backend",
    "Profile" => "control_center.hardware_profile",
    "Profiles" => "control_center.hardware_profiles",
    "Status" => "control_center.hardware_status",
    "Capacity" => "control_center.hardware_capacity",
    "Health" => "control_center.hardware_health",
    "Name" => "control_center.hardware_name",
    "Type" => "control_center.hardware_type",
    "Product" => "control_center.hardware_product",
    "Bus" => "control_center.hardware_bus",
    "Path" => "control_center.hardware_path",
    "Module" => "control_center.hardware_module",
    "Driver status" => "control_center.hardware_driver_status",
    "Devices" => "control_center.hardware_devices",
    _ => key,
  };
  tr(lang, label).into()
}

/// Executes the `na` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn na(lang: Lang) -> String {
  tr(lang, "control_center.unavailable_6a8fc3").into()
}

/// Executes the `run_profile` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_profile(value: &str) -> Result<String, String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("powerprofilesctl")
        .arg("set")
        .arg(value),
    )
    .map_err(|e| e.to_string())?;
  output
    .status
    .is_some_and(|v| v == 0)
    .then(|| format!("Profile: {value}"))
    .ok_or_else(|| String::from_utf8_lossy(&output.stderr).trim().into())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `home_selection_opens_cpu_without_render_time_probe` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `cpu_governor_selection_is_bounded` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `cpu_governor_enter_dispatches_even_with_power_profiles_daemon` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `cpu_governor_enter_does_not_dispatch_when_no_governors` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
