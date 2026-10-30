//! Terminal UI rendering and interaction of the Displays page.
use crate::{
  backend,
  model::{
    DisplayPage, Monitor, MonitorProfile, MonitorSetting, PersistedConfig, PersistedMonitor,
    PromptGoal,
  },
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::components::{StatusKind, StatusMessage};
use argvus_tui::confirm::{ConfirmDialog, ConfirmOutcome, ConfirmState, draw_confirm};
use argvus_tui::hints::{HintContext, confirm_hints, hints};
use argvus_tui::icons;
use argvus_tui::menu::{MenuEvent, MenuState, MenuStyle, Row, draw_menu};
use argvus_tui::page::{shell, status};
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use std::time::{Duration, Instant};

mod input;
mod jobs;
mod night_light;
mod picker;
mod profiles;
mod prompts;
mod render;
mod rows;

const REVERT_SECONDS: u64 = 15;
const HOTPLUG_INTERVAL: Duration = Duration::from_millis(1500);
/// Space between footer segments, as in `argvus_tui::hints`.
const FOOTER_GAP: &str = "   ";

#[derive(Debug, Clone, PartialEq)]
enum PersistChange {
  Mode(String),
  Scale(f64),
  Position(i32, i32),
  Vrr(i32),
  Hdr(i32),
  Mirror(String),
  BitDepth(i32),
  Disabled(bool),
  Workspace(u32, Option<String>),
  Dpms(bool),
  Primary,
  None,
}

#[derive(Debug, Clone, PartialEq)]
struct PickerOption {
  label: String,
  args: String,
  persist: PersistChange,
  /// When set, selecting this row opens the free-form editor goal instead.
  prompt: Option<PromptGoal>,
}

#[derive(Debug, Clone)]
enum JobData {
  Snapshot {
    monitors: Vec<Monitor>,
    config: PersistedConfig,
    state: crate::model::DisplayState,
    version: (u32, u32),
    night_light: crate::model::NightLightStatus,
  },
  Hotplug(Vec<Monitor>),
  Action(String),
  NightLight(crate::model::NightLightStatus, Option<String>),
}

struct RevertState {
  name: String,
  previous: Monitor,
  previous_config: PersistedConfig,
  deadline: Instant,
}

/// A destructive action waiting for the single confirmation component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confirmation {
  DeleteProfile(usize),
  RemoveConfig,
}

/// Answer to the revert countdown.
enum RevertAnswer {
  /// Still open (focus moved, or a key without meaning such as Space).
  Pending,
  /// `y`, or Enter on Keep: the new configuration stays.
  Keep(String),
  /// `n`, Esc, or Enter on Revert: the previous configuration comes back.
  Revert(Box<RevertState>),
}

/// Stable identity of a Displays menu row. Monitors, saved configurations
/// of disconnected monitors, picker options and profiles keep their index
/// in the source list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
  /// Index into the connected monitors.
  Monitor(usize),
  /// Index into the saved configurations of monitors that are not present.
  Stale(usize),
  Profiles,
  NightLight,
  NightToggle,
  NightTemperature,
  NightSchedule,
  NightStart,
  NightEnd,
  Refresh,
  Setting(MonitorSetting),
  /// Re-applies the saved configuration with the revert countdown.
  Apply,
  /// Drops the monitor's saved overrides and applies (immediate).
  Reset,
  RemoveConfig,
  /// Index into the picker options of the open setting.
  Option(usize),
  /// The free-form value of the open setting.
  Custom,
  /// Index into the saved profiles.
  Profile(usize),
  NewProfile,
  ApplyProfile,
  RenameProfile,
  DeleteProfile,
}

pub struct DisplaysApp {
  pub page: DisplayPage,
  pub lang: Lang,
  pub theme: Theme,
  pub status: Option<StatusMessage>,
  monitors: Vec<Monitor>,
  config: PersistedConfig,
  state: crate::model::DisplayState,
  version: (u32, u32),
  menu: MenuState,
  list_height: u16,
  job: Option<JobHandle<JobData>>,
  action: Option<JobHandle<JobData>>,
  hotplug: Option<JobHandle<JobData>>,
  last_hotplug: Instant,
  revert: Option<RevertState>,
  /// Focus of the revert confirmation; starts on Revert (Cancel).
  revert_focus: ConfirmState,
  confirm: Option<(Confirmation, ConfirmState)>,
  prompt_buffer: String,
  prompt_error: Option<String>,
  prompt_back: Option<DisplayPage>,
  night_light: crate::model::NightLightStatus,
  night_error: Option<String>,
  manager: JobManager,
}

impl DisplaysApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme) -> Self {
    let mut app = Self {
      page: DisplayPage::Home,
      lang,
      theme,
      status: None,
      monitors: vec![],
      config: PersistedConfig::default(),
      state: crate::model::DisplayState::default(),
      version: (0, 0),
      menu: MenuState::default(),
      list_height: 1,
      job: None,
      action: None,
      hotplug: None,
      last_hotplug: Instant::now()
        .checked_sub(HOTPLUG_INTERVAL * 6)
        .unwrap_or(Instant::now()),
      revert: None,
      revert_focus: ConfirmState::new(),
      confirm: None,
      prompt_buffer: String::new(),
      prompt_error: None,
      prompt_back: None,
      night_light: Default::default(),
      night_error: None,
      manager: JobManager::default(),
    };
    app.refresh();
    app
  }
}

#[cfg(test)]
mod tests;
