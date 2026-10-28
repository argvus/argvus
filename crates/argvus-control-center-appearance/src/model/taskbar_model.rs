//! Taskbar settings: position, utility widgets and their grouping, and the date and time formats.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarPosition {
  Top,
  Bottom,
}

impl TaskbarPosition {
  pub fn value(self) -> &'static str {
    match self {
      Self::Top => "top",
      Self::Bottom => "bottom",
    }
  }
  pub fn from_value(value: &str) -> Self {
    if value.trim() == "bottom" {
      Self::Bottom
    } else {
      Self::Top
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines the presentation mode for the taskbar utility group.
pub enum TaskbarUtilityGroupMode {
  Auto,
  AlwaysExpanded,
}

impl TaskbarUtilityGroupMode {
  pub fn value(self) -> &'static str {
    match self {
      Self::Auto => "auto",
      Self::AlwaysExpanded => "always-expanded",
    }
  }

  pub fn from_value(value: &str) -> Self {
    if value.trim() == "always-expanded" {
      Self::AlwaysExpanded
    } else {
      Self::Auto
    }
  }
}

/// Identifies a separately toggleable Taskbar utility widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarUtilityWidget {
  Network,
  PowerProfile,
  KeyboardLayout,
  Memory,
  Cpu,
  CpuTemperature,
  GpuTemperature,
}

impl TaskbarUtilityWidget {
  pub const ALL: [Self; 7] = [
    Self::Network,
    Self::PowerProfile,
    Self::KeyboardLayout,
    Self::Memory,
    Self::Cpu,
    Self::CpuTemperature,
    Self::GpuTemperature,
  ];

  /// Returns the stable config-pointer identifier (`taskbar.icons.<key>_enabled`).
  pub fn key(self) -> &'static str {
    match self {
      Self::Network => "network",
      Self::PowerProfile => "power_profile",
      Self::KeyboardLayout => "keyboard_layout",
      Self::Memory => "memory",
      Self::Cpu => "cpu",
      Self::CpuTemperature => "cpu_temperature",
      Self::GpuTemperature => "gpu_temperature",
    }
  }

  /// Returns the localization key for the Control Center row.
  pub fn label_key(self) -> &'static str {
    match self {
      Self::Network => "control_center.network",
      Self::PowerProfile => "control_center.taskbar_power_profile",
      Self::KeyboardLayout => "control_center.keyboard_layout",
      Self::Memory => "control_center.taskbar_memory_usage",
      Self::Cpu => "control_center.taskbar_cpu_usage",
      Self::CpuTemperature => "control_center.taskbar_cpu_temperature",
      Self::GpuTemperature => "control_center.taskbar_gpu_temperature",
    }
  }
}

/// Stores the sparse Taskbar Utilities preferences represented by the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskbarUtilityWidgets {
  pub network: bool,
  pub power_profile: bool,
  pub keyboard_layout: bool,
  pub memory: bool,
  pub cpu: bool,
  pub cpu_temperature: bool,
  pub gpu_temperature: bool,
}

impl Default for TaskbarUtilityWidgets {
  fn default() -> Self {
    Self {
      network: true,
      power_profile: true,
      keyboard_layout: true,
      memory: true,
      cpu: true,
      cpu_temperature: true,
      gpu_temperature: true,
    }
  }
}

impl TaskbarUtilityWidgets {
  pub fn enabled(&self, widget: TaskbarUtilityWidget) -> bool {
    match widget {
      TaskbarUtilityWidget::Network => self.network,
      TaskbarUtilityWidget::PowerProfile => self.power_profile,
      TaskbarUtilityWidget::KeyboardLayout => self.keyboard_layout,
      TaskbarUtilityWidget::Memory => self.memory,
      TaskbarUtilityWidget::Cpu => self.cpu,
      TaskbarUtilityWidget::CpuTemperature => self.cpu_temperature,
      TaskbarUtilityWidget::GpuTemperature => self.gpu_temperature,
    }
  }

  pub fn set(&mut self, widget: TaskbarUtilityWidget, enabled: bool) {
    match widget {
      TaskbarUtilityWidget::Network => self.network = enabled,
      TaskbarUtilityWidget::PowerProfile => self.power_profile = enabled,
      TaskbarUtilityWidget::KeyboardLayout => self.keyboard_layout = enabled,
      TaskbarUtilityWidget::Memory => self.memory = enabled,
      TaskbarUtilityWidget::Cpu => self.cpu = enabled,
      TaskbarUtilityWidget::CpuTemperature => self.cpu_temperature = enabled,
      TaskbarUtilityWidget::GpuTemperature => self.gpu_temperature = enabled,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines the taskbar clock's date format. Values mirror the
/// `taskbar.date.format` config pointer and the `argvus-i18n` taskbar domain
/// template keys (`date.format.<value>`).
pub enum TaskbarDateFormat {
  WeekdayDayMonth,
  WeekdayDayMonthYear,
  WeekdayDaySlashMonthYear,
  NumericShortLocaleAware,
}

impl TaskbarDateFormat {
  pub const ALL: [Self; 4] = [
    Self::WeekdayDayMonth,
    Self::WeekdayDayMonthYear,
    Self::WeekdayDaySlashMonthYear,
    Self::NumericShortLocaleAware,
  ];

  pub fn value(self) -> &'static str {
    match self {
      Self::WeekdayDayMonth => "weekday_day_month",
      Self::WeekdayDayMonthYear => "weekday_day_month_year",
      Self::WeekdayDaySlashMonthYear => "weekday_day_slash_month_year",
      Self::NumericShortLocaleAware => "numeric_short_locale_aware",
    }
  }

  pub fn from_value(value: &str) -> Self {
    match value.trim() {
      "weekday_day_month_year" => Self::WeekdayDayMonthYear,
      "weekday_day_slash_month_year" => Self::WeekdayDaySlashMonthYear,
      "numeric_short_locale_aware" => Self::NumericShortLocaleAware,
      _ => Self::WeekdayDayMonth,
    }
  }

  pub fn label_key(self) -> &'static str {
    match self {
      Self::WeekdayDayMonth => "control_center.taskbar_date_format_weekday_day_month",
      Self::WeekdayDayMonthYear => "control_center.taskbar_date_format_weekday_day_month_year",
      Self::WeekdayDaySlashMonthYear => {
        "control_center.taskbar_date_format_weekday_day_slash_month_year"
      }
      Self::NumericShortLocaleAware => "control_center.taskbar_date_format_numeric_short",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines the taskbar clock's time format (`taskbar.time.format`).
pub enum TaskbarTimeFormat {
  TwentyFourHour,
  TwelveHour,
}

impl TaskbarTimeFormat {
  pub const ALL: [Self; 2] = [Self::TwentyFourHour, Self::TwelveHour];

  pub fn value(self) -> &'static str {
    match self {
      Self::TwentyFourHour => "24h",
      Self::TwelveHour => "12h",
    }
  }

  pub fn from_value(value: &str) -> Self {
    if value.trim() == "12h" {
      Self::TwelveHour
    } else {
      Self::TwentyFourHour
    }
  }

  pub fn label_key(self) -> &'static str {
    match self {
      Self::TwentyFourHour => "control_center.taskbar_time_format_24h",
      Self::TwelveHour => "control_center.taskbar_time_format_12h",
    }
  }
}
