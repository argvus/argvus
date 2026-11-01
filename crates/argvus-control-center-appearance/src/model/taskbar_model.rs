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

/// Identifies an optional taskbar module with a persisted visibility setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarModule {
  AudioPlayer,
  Focus,
  Launcher,
  Network,
  PowerProfile,
  KeyboardLayout,
  Memory,
  Cpu,
  CpuTemperature,
  GpuTemperature,
  Date,
  Clock,
  Battery,
  Tray,
  Notifications,
  RemovableDevices,
  Recording,
  Bluetooth,
  AudioInput,
  AudioOutput,
  KeepAwake,
}

impl TaskbarModule {
  pub const ALL: [Self; 21] = [
    Self::AudioPlayer,
    Self::Focus,
    Self::Launcher,
    Self::Network,
    Self::PowerProfile,
    Self::KeyboardLayout,
    Self::Memory,
    Self::Cpu,
    Self::CpuTemperature,
    Self::GpuTemperature,
    Self::Date,
    Self::Clock,
    Self::Battery,
    Self::Tray,
    Self::Notifications,
    Self::RemovableDevices,
    Self::Recording,
    Self::Bluetooth,
    Self::AudioInput,
    Self::AudioOutput,
    Self::KeepAwake,
  ];

  /// Returns the stable config-pointer identifier under `taskbar.icons`.
  pub fn key(self) -> &'static str {
    match self {
      Self::AudioPlayer => "audio_player",
      Self::Focus => "focus",
      Self::Launcher => "launcher",
      Self::Network => "network",
      Self::PowerProfile => "power_profile",
      Self::KeyboardLayout => "keyboard_layout",
      Self::Memory => "memory",
      Self::Cpu => "cpu",
      Self::CpuTemperature => "cpu_temperature",
      Self::GpuTemperature => "gpu_temperature",
      Self::Date => "date",
      Self::Clock => "clock",
      Self::Battery => "battery",
      Self::Tray => "tray",
      Self::Notifications => "notifications",
      Self::RemovableDevices => "removable_devices",
      Self::Recording => "recording",
      Self::Bluetooth => "bluetooth",
      Self::AudioInput => "audio_input",
      Self::AudioOutput => "audio_output",
      Self::KeepAwake => "keep_awake",
    }
  }

  /// Returns the localization key for the Control Center row.
  pub fn label_key(self) -> &'static str {
    match self {
      Self::AudioPlayer => "control_center.taskbar_audio_player_view",
      Self::Focus => "control_center.taskbar_focus_icon",
      Self::Launcher => "control_center.launcher",
      Self::Network => "control_center.network",
      Self::PowerProfile => "control_center.taskbar_power_profile",
      Self::KeyboardLayout => "control_center.keyboard_layout",
      Self::Memory => "control_center.taskbar_memory_usage",
      Self::Cpu => "control_center.taskbar_cpu_usage",
      Self::CpuTemperature => "control_center.taskbar_cpu_temperature",
      Self::GpuTemperature => "control_center.taskbar_gpu_temperature",
      Self::Date => "control_center.date",
      Self::Clock => "control_center.time",
      Self::Battery => "control_center.taskbar_battery",
      Self::Tray => "control_center.taskbar_tray",
      Self::Notifications => "control_center.notifications",
      Self::RemovableDevices => "control_center.taskbar_removable_devices",
      Self::Recording => "control_center.taskbar_recording",
      Self::Bluetooth => "control_center.bluetooth",
      Self::AudioInput => "control_center.taskbar_audio_input",
      Self::AudioOutput => "control_center.taskbar_audio_output",
      Self::KeepAwake => "control_center.keep_awake",
    }
  }
}

/// Stores visibility preferences for every optional taskbar module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskbarModules {
  pub battery_available: bool,
  pub bluetooth_available: bool,
  pub audio_player: bool,
  pub focus: bool,
  pub launcher: bool,
  pub network: bool,
  pub power_profile: bool,
  pub keyboard_layout: bool,
  pub memory: bool,
  pub cpu: bool,
  pub cpu_temperature: bool,
  pub gpu_temperature: bool,
  pub date: bool,
  pub clock: bool,
  pub battery: bool,
  pub tray: bool,
  pub notifications: bool,
  pub removable_devices: bool,
  pub recording: bool,
  pub bluetooth: bool,
  pub audio_input: bool,
  pub audio_output: bool,
  pub keep_awake: bool,
}

impl Default for TaskbarModules {
  fn default() -> Self {
    Self {
      battery_available: false,
      bluetooth_available: false,
      audio_player: true,
      focus: true,
      launcher: false,
      network: true,
      power_profile: true,
      keyboard_layout: true,
      memory: true,
      cpu: true,
      cpu_temperature: true,
      gpu_temperature: true,
      date: true,
      clock: true,
      battery: true,
      tray: true,
      notifications: true,
      removable_devices: true,
      recording: true,
      bluetooth: true,
      audio_input: true,
      audio_output: true,
      keep_awake: true,
    }
  }
}

impl TaskbarModules {
  pub fn enabled(&self, module: TaskbarModule) -> bool {
    match module {
      TaskbarModule::AudioPlayer => self.audio_player,
      TaskbarModule::Focus => self.focus,
      TaskbarModule::Launcher => self.launcher,
      TaskbarModule::Network => self.network,
      TaskbarModule::PowerProfile => self.power_profile,
      TaskbarModule::KeyboardLayout => self.keyboard_layout,
      TaskbarModule::Memory => self.memory,
      TaskbarModule::Cpu => self.cpu,
      TaskbarModule::CpuTemperature => self.cpu_temperature,
      TaskbarModule::GpuTemperature => self.gpu_temperature,
      TaskbarModule::Date => self.date,
      TaskbarModule::Clock => self.clock,
      TaskbarModule::Battery => self.battery,
      TaskbarModule::Tray => self.tray,
      TaskbarModule::Notifications => self.notifications,
      TaskbarModule::RemovableDevices => self.removable_devices,
      TaskbarModule::Recording => self.recording,
      TaskbarModule::Bluetooth => self.bluetooth,
      TaskbarModule::AudioInput => self.audio_input,
      TaskbarModule::AudioOutput => self.audio_output,
      TaskbarModule::KeepAwake => self.keep_awake,
    }
  }

  pub fn set(&mut self, module: TaskbarModule, enabled: bool) {
    match module {
      TaskbarModule::AudioPlayer => self.audio_player = enabled,
      TaskbarModule::Focus => self.focus = enabled,
      TaskbarModule::Launcher => self.launcher = enabled,
      TaskbarModule::Network => self.network = enabled,
      TaskbarModule::PowerProfile => self.power_profile = enabled,
      TaskbarModule::KeyboardLayout => self.keyboard_layout = enabled,
      TaskbarModule::Memory => self.memory = enabled,
      TaskbarModule::Cpu => self.cpu = enabled,
      TaskbarModule::CpuTemperature => self.cpu_temperature = enabled,
      TaskbarModule::GpuTemperature => self.gpu_temperature = enabled,
      TaskbarModule::Date => self.date = enabled,
      TaskbarModule::Clock => self.clock = enabled,
      TaskbarModule::Battery => self.battery = enabled,
      TaskbarModule::Tray => self.tray = enabled,
      TaskbarModule::Notifications => self.notifications = enabled,
      TaskbarModule::RemovableDevices => self.removable_devices = enabled,
      TaskbarModule::Recording => self.recording = enabled,
      TaskbarModule::Bluetooth => self.bluetooth = enabled,
      TaskbarModule::AudioInput => self.audio_input = enabled,
      TaskbarModule::AudioOutput => self.audio_output = enabled,
      TaskbarModule::KeepAwake => self.keep_awake = enabled,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarWorkspacePresentation {
  Labels,
  AppIcons,
}

impl TaskbarWorkspacePresentation {
  pub const ALL: [Self; 2] = [Self::Labels, Self::AppIcons];
  pub fn value(self) -> &'static str {
    match self {
      Self::Labels => "labels",
      Self::AppIcons => "app-icons",
    }
  }

  pub fn from_value(value: &str) -> Self {
    match value.trim() {
      "app-icons" => Self::AppIcons,
      _ => Self::Labels,
    }
  }

  pub fn label_key(self) -> &'static str {
    match self {
      Self::Labels => "control_center.taskbar_workspace_labels",
      Self::AppIcons => "control_center.taskbar_workspace_app_icons",
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
