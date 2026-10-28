//! Widget telemetry blocks and which ones are enabled.

/// Identifies a separately configurable Widget Telemetry section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetTelemetryBlock {
  System,
  CpuGpu,
  Memory,
  Storage,
  Processes,
  Network,
  DevDashboard,
  Shortcuts,
}

impl WidgetTelemetryBlock {
  pub const ALL: [Self; 8] = [
    Self::System,
    Self::CpuGpu,
    Self::Memory,
    Self::Storage,
    Self::Processes,
    Self::Network,
    Self::DevDashboard,
    Self::Shortcuts,
  ];

  /// Returns the stable command-line identifier owned by the widget package.
  pub fn key(self) -> &'static str {
    match self {
      Self::System => "system",
      Self::CpuGpu => "cpu_gpu",
      Self::Memory => "memory",
      Self::Storage => "storage",
      Self::Processes => "processes",
      Self::Network => "network",
      Self::Shortcuts => "keys",
      Self::DevDashboard => "dev_dashboard",
    }
  }

  /// Returns the localization key for the Control Center row.
  pub fn label_key(self) -> &'static str {
    match self {
      Self::System => "control_center.widget_telemetry_system",
      Self::CpuGpu => "control_center.widget_telemetry_cpu_gpu",
      Self::Memory => "control_center.widget_telemetry_memory",
      Self::Storage => "control_center.widget_telemetry_storage",
      Self::Processes => "control_center.widget_telemetry_processes",
      Self::Network => "control_center.widget_telemetry_network",
      Self::Shortcuts => "control_center.widget_telemetry_shortcuts",
      Self::DevDashboard => "control_center.widget_telemetry_dev_dashboard",
    }
  }
}

/// Stores the sparse Widget Telemetry preferences represented by the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetTelemetryBlocks {
  pub system: bool,
  pub cpu_gpu: bool,
  pub memory: bool,
  pub storage: bool,
  pub processes: bool,
  pub network: bool,
  pub shortcuts: bool,
  pub dev_dashboard: bool,
}

impl Default for WidgetTelemetryBlocks {
  fn default() -> Self {
    Self {
      system: true,
      cpu_gpu: true,
      memory: true,
      storage: true,
      processes: true,
      network: true,
      shortcuts: true,
      dev_dashboard: true,
    }
  }
}

impl WidgetTelemetryBlocks {
  pub fn enabled(&self, block: WidgetTelemetryBlock) -> bool {
    match block {
      WidgetTelemetryBlock::System => self.system,
      WidgetTelemetryBlock::CpuGpu => self.cpu_gpu,
      WidgetTelemetryBlock::Memory => self.memory,
      WidgetTelemetryBlock::Storage => self.storage,
      WidgetTelemetryBlock::Processes => self.processes,
      WidgetTelemetryBlock::Network => self.network,
      WidgetTelemetryBlock::Shortcuts => self.shortcuts,
      WidgetTelemetryBlock::DevDashboard => self.dev_dashboard,
    }
  }

  pub fn set(&mut self, block: WidgetTelemetryBlock, enabled: bool) {
    match block {
      WidgetTelemetryBlock::System => self.system = enabled,
      WidgetTelemetryBlock::CpuGpu => self.cpu_gpu = enabled,
      WidgetTelemetryBlock::Memory => self.memory = enabled,
      WidgetTelemetryBlock::Storage => self.storage = enabled,
      WidgetTelemetryBlock::Processes => self.processes = enabled,
      WidgetTelemetryBlock::Network => self.network = enabled,
      WidgetTelemetryBlock::Shortcuts => self.shortcuts = enabled,
      WidgetTelemetryBlock::DevDashboard => self.dev_dashboard = enabled,
    }
  }
}
