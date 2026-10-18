//! Implements terminal UI rendering and interaction in crate `argvus control center displays`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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

/// Defines the constant `REVERT_SECONDS`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const REVERT_SECONDS: u64 = 15;
/// Defines the constant `HOTPLUG_INTERVAL`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const HOTPLUG_INTERVAL: Duration = Duration::from_millis(1500);
/// Space between footer segments, as in `argvus_tui::hints`.
const FOOTER_GAP: &str = "   ";

#[derive(Debug, Clone, PartialEq)]
/// Defines `PersistChange`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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
/// Represents `PickerOption`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
struct PickerOption {
  label: String,
  args: String,
  persist: PersistChange,
  /// When set, selecting this row opens the free-form editor goal instead.
  prompt: Option<PromptGoal>,
}

#[derive(Debug, Clone)]
/// Defines `JobData`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum JobData {
  Snapshot {
    monitors: Vec<Monitor>,
    config: PersistedConfig,
    state: crate::model::DisplayState,
    version: (u32, u32),
  },
  Hotplug(Vec<Monitor>),
  Action(String),
}

/// Represents `RevertState`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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

/// Represents `DisplaysApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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
  manager: JobManager,
}

impl DisplaysApp {
  /// Reloads the snapshot after a route opened a page; the cursor starts
  /// on the page's first row.
  pub fn reload(&mut self) {
    self.menu = MenuState::default();
    self.refresh();
  }

  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
      manager: JobManager::default(),
    };
    app.refresh();
    app
  }

  /// Opens `page` with the cursor on its first row.
  fn go(&mut self, page: DisplayPage) {
    self.page = page;
    self.menu = MenuState::default();
  }

  /// Executes the `refresh` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn refresh(&mut self) {
    if self.job.is_some() {
      return;
    }
    let lang = self.lang;
    self.job = Some(self.manager.spawn(move |_| {
      let version = backend::hyprctl_version();
      let monitors = backend::list_monitors().unwrap_or_default();
      let config = backend::load_config();
      let state = backend::load_state();
      Ok(JobData::Snapshot {
        monitors,
        config,
        state,
        version,
      })
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(lang, "control_center.loading_monitors").into(),
    });
  }

  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      match result {
        Ok(JobData::Snapshot {
          monitors,
          config,
          state,
          version,
        }) => {
          self.monitors = monitors;
          self.config = config;
          self.state = state;
          self.version = version;
          self.status = None;
        }
        Ok(JobData::Hotplug(_)) | Ok(JobData::Action(_)) => {}
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          })
        }
      };
      self.last_hotplug = Instant::now();
      changed = true;
    }
    if let Some(action_job) = &self.action
      && let JobState::Finished(result) = action_job.try_state()
    {
      self.action = None;
      match result {
        Ok(JobData::Action(text)) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text,
          });
          self.refresh();
        }
        Ok(_) => {}
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          });
          self.refresh();
        }
      };
      changed = true;
    }
    if let Some(revert) = self.take_expired_revert(Instant::now()) {
      self.revert_to(revert);
      changed = true;
    }
    if let Some(hotplug) = &self.hotplug
      && let JobState::Finished(result) = hotplug.try_state()
    {
      self.hotplug = None;
      if let Ok(JobData::Hotplug(monitors)) = result {
        let changed_monitors = fingerprint(&self.monitors) != fingerprint(&monitors);
        if changed_monitors {
          self.monitors = monitors;
          if let DisplayPage::Detail(index) = self.page
            && index >= self.monitors.len()
          {
            self.go(DisplayPage::Home);
          }
          self.status = Some(StatusMessage {
            kind: StatusKind::Info,
            text: tr(self.lang, "control_center.monitors_changed_hotplug").into(),
          });
          changed = true;
        }
      }
      self.last_hotplug = Instant::now();
    }
    if self.job.is_none()
      && self.action.is_none()
      && self.hotplug.is_none()
      && self.revert.is_none()
      && !matches!(self.page, DisplayPage::Prompt { .. })
      && self.last_hotplug.elapsed() >= HOTPLUG_INTERVAL
    {
      self.hotplug = Some(self.manager.spawn(|_| {
        let monitors = backend::list_monitors().unwrap_or_default();
        Ok(JobData::Hotplug(monitors))
      }));
      self.last_hotplug = Instant::now();
    }
    changed
  }

  /// Arms the revert countdown: without an answer before
  /// `now + REVERT_SECONDS`, `previous_config` is applied again.
  fn arm_revert(
    &mut self,
    name: String,
    previous: Monitor,
    previous_config: PersistedConfig,
    now: Instant,
  ) {
    self.revert = Some(RevertState {
      name,
      previous,
      previous_config,
      deadline: now + Duration::from_secs(REVERT_SECONDS),
    });
    self.revert_focus = ConfirmState::new();
  }

  /// A key while the revert countdown is open. The focus starts on Revert,
  /// so Enter pressed by reflex on a broken screen never keeps the change.
  fn answer_revert(&mut self, key: KeyCode) -> RevertAnswer {
    if self.revert.is_none() {
      return RevertAnswer::Pending;
    }
    match self.revert_focus.handle(key) {
      ConfirmOutcome::Pending => RevertAnswer::Pending,
      ConfirmOutcome::Confirmed => match self.revert.take() {
        Some(revert) => RevertAnswer::Keep(revert.name),
        None => RevertAnswer::Pending,
      },
      ConfirmOutcome::Cancelled => match self.revert.take() {
        Some(revert) => RevertAnswer::Revert(Box::new(revert)),
        None => RevertAnswer::Pending,
      },
    }
  }

  /// The armed revert whose deadline has passed at `now`, if any.
  fn take_expired_revert(&mut self, now: Instant) -> Option<RevertState> {
    self.revert.take_if(|revert| now >= revert.deadline)
  }

  /// Applies the configuration saved before the change ("Reverted <name>").
  fn revert_to(&mut self, revert: RevertState) {
    let message = revert_message(self.lang, &revert);
    self.spawn_revert(revert.previous_config, message);
  }

  /// Whether typed characters currently go to a prompt field (profile name,
  /// custom values), so `q`/`?` must not act as the global quit/help keys.
  /// Mirrors the precedence of [`Self::handle`]: the profile deletion
  /// confirmation and the revert countdown are not typing.
  pub fn captures_text(&self) -> bool {
    self.confirm.is_none()
      && self.revert.is_none()
      && self.prompt_back.is_some()
      && matches!(self.page, DisplayPage::Prompt { .. })
  }

  /// Rows of the current page.
  fn rows(&self) -> Vec<Row<Item>> {
    match self.page {
      DisplayPage::Home => self.home_rows(),
      DisplayPage::Detail(index) => self.detail_rows(index),
      DisplayPage::Picker { monitor, setting } => self.picker_rows(monitor, setting),
      DisplayPage::Profiles => self.profile_rows(),
      DisplayPage::Profile(index) => self.profile_page_rows(index),
      DisplayPage::Prompt { .. } => Vec::new(),
    }
  }

  /// Processes `handle` in this module's event flow. Returns `true` when Esc
  /// leaves the Displays home.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if let Some((confirmation, mut focus)) = self.confirm.take() {
      match focus.handle(key) {
        ConfirmOutcome::Confirmed => self.run_confirmation(confirmation),
        ConfirmOutcome::Cancelled => {
          if let Confirmation::DeleteProfile(_) = confirmation {
            self.status = Some(StatusMessage {
              kind: StatusKind::Info,
              text: tr(self.lang, "control_center.deletion_cancelled").into(),
            });
          }
        }
        ConfirmOutcome::Pending => self.confirm = Some((confirmation, focus)),
      }
      return false;
    }
    if self.revert.is_some() {
      match self.answer_revert(key) {
        RevertAnswer::Keep(name) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: format!(
              "{} · {name}",
              tr(self.lang, "control_center.configuration_kept")
            ),
          });
        }
        RevertAnswer::Revert(revert) => self.revert_to(*revert),
        RevertAnswer::Pending => {}
      }
      return false;
    }
    if self.prompt_back.is_some() {
      return self.prompt_key(key);
    }
    let rows = self.rows();
    let busy = self.job.is_some() || self.action.is_some();
    match key {
      // Tab only switches tabs or panes; Displays has none.
      KeyCode::Tab | KeyCode::BackTab => return false,
      KeyCode::Char('r') if !busy => {
        match self.page {
          DisplayPage::Home | DisplayPage::Detail(_) => self.refresh(),
          DisplayPage::Profiles | DisplayPage::Profile(_) => self.reload_state(),
          // `r` (and `q`, when it reaches the page) cancel the picker.
          DisplayPage::Picker { .. } => self.exit_picker(),
          DisplayPage::Prompt { .. } => {}
        }
        return false;
      }
      KeyCode::Char('q') if matches!(self.page, DisplayPage::Picker { .. }) => {
        self.exit_picker();
        return false;
      }
      _ => {}
    }
    let key = match key {
      KeyCode::Char('h') => KeyCode::Left,
      KeyCode::Char('l') => KeyCode::Right,
      key => key,
    };
    let page_size = usize::from(self.list_height.max(1));
    let event = self.menu.handle(key, &rows, page_size);
    match event {
      MenuEvent::Back => return self.back(),
      // While a snapshot or an action runs, only navigation and Esc work.
      _ if busy => {}
      MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item) => {
        self.activate(item)
      }
      MenuEvent::Adjust(..) | MenuEvent::Moved | MenuEvent::None => {}
    }
    false
  }

  /// Esc/`←`: one level up; `true` leaves the Displays home.
  fn back(&mut self) -> bool {
    match self.page {
      DisplayPage::Home => return true,
      DisplayPage::Profiles | DisplayPage::Detail(_) => self.go(DisplayPage::Home),
      DisplayPage::Profile(_) => self.go(DisplayPage::Profiles),
      DisplayPage::Picker { monitor, .. } => self.go(DisplayPage::Detail(monitor)),
      DisplayPage::Prompt { goal } => self.restore_prompt(goal),
    }
    false
  }

  /// Runs the row `item`.
  fn activate(&mut self, item: Item) {
    match item {
      Item::Monitor(index) => self.go(DisplayPage::Detail(index)),
      Item::Stale(offset) => self.go(DisplayPage::Detail(self.stale_start() + offset)),
      Item::Profiles => self.go(DisplayPage::Profiles),
      Item::Refresh => self.refresh(),
      Item::Setting(setting) => {
        if let DisplayPage::Detail(monitor) = self.page {
          self.go(DisplayPage::Picker { monitor, setting });
        }
      }
      Item::Apply => self.apply_button(),
      Item::Reset => self.reset_button(),
      Item::RemoveConfig => self.confirm = Some((Confirmation::RemoveConfig, ConfirmState::new())),
      Item::Option(index) => self.apply_picker_selection(index),
      Item::Custom => {
        if let Some(prompt) = self.custom_prompt() {
          self.open_prompt(prompt);
        }
      }
      Item::Profile(index) => self.go(DisplayPage::Profile(index)),
      Item::NewProfile => self.open_prompt(PromptGoal::ProfileCreate),
      Item::ApplyProfile => {
        if let DisplayPage::Profile(index) = self.page {
          self.apply_profile(index);
        }
      }
      Item::RenameProfile => {
        if let DisplayPage::Profile(index) = self.page {
          self.open_prompt(PromptGoal::ProfileRename(index));
        }
      }
      Item::DeleteProfile => {
        if let DisplayPage::Profile(index) = self.page {
          self.confirm = Some((Confirmation::DeleteProfile(index), ConfirmState::new()));
        }
      }
    }
  }

  /// Runs a confirmed destructive action.
  fn run_confirmation(&mut self, confirmation: Confirmation) {
    match confirmation {
      Confirmation::DeleteProfile(index) => {
        self.delete_profile(index);
        self.status = Some(StatusMessage {
          kind: StatusKind::Success,
          text: tr(self.lang, "control_center.profile_deleted").into(),
        });
        self.go(DisplayPage::Profiles);
      }
      Confirmation::RemoveConfig => self.remove_button(),
    }
  }

  /// Executes the `prompt_key` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn prompt_key(&mut self, key: KeyCode) -> bool {
    let DisplayPage::Prompt { goal } = self.page else {
      return false;
    };
    match key {
      KeyCode::Esc => {
        self.restore_prompt(goal);
        false
      }
      KeyCode::Backspace => {
        self.prompt_buffer.pop();
        false
      }
      KeyCode::Char(c) if prompt_allowed(goal, c) => {
        self.prompt_buffer.push(c);
        false
      }
      KeyCode::Enter => {
        self.commit_prompt(goal);
        false
      }
      _ => false,
    }
  }

  /// Executes the `restore_prompt` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn restore_prompt(&mut self, goal: PromptGoal) {
    self.prompt_buffer.clear();
    self.prompt_error = None;
    let page = self.prompt_back.take().unwrap_or(match goal {
      PromptGoal::Position(i)
      | PromptGoal::Scale(i)
      | PromptGoal::SdrBrightness(i)
      | PromptGoal::SdrSaturation(i) => DisplayPage::Detail(i),
      PromptGoal::ProfileCreate => DisplayPage::Profiles,
      PromptGoal::ProfileRename(i) => DisplayPage::Profile(i),
    });
    self.go(page);
  }

  /// Executes the `commit_prompt` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn commit_prompt(&mut self, goal: PromptGoal) {
    let buffer = self.prompt_buffer.clone();
    match goal {
      PromptGoal::Position(index) => {
        let Some((x, y)) = parse_position(&buffer) else {
          self.prompt_error = Some(
            tr(
              self.lang,
              "control_center.invalid_position_use_x_y_e_g_1920_0",
            )
            .into(),
          );
          return;
        };
        let Some(monitor) = self.monitors.get(index).cloned() else {
          return;
        };
        let name = monitor.name.clone();
        let mut config = self.config.clone();
        let mut persisted = config.persisted(&name);
        persisted.position = Some(format!("{x}x{y}"));
        config.set_monitor(&name, persisted);
        let prev = self.config.clone();
        self.config = config.clone();
        self.arm_revert(name.clone(), monitor.clone(), prev, Instant::now());
        let message = format!(
          "{} · {name} ({x}, {y})",
          tr(self.lang, "control_center.position")
        );
        self.leave_prompt(DisplayPage::Detail(index));
        self.spawn_apply_with_config(config, message);
      }
      PromptGoal::Scale(index) => {
        let Some(value) = parse_number(&buffer) else {
          self.prompt_error = Some(
            tr(
              self.lang,
              "control_center.invalid_value_use_a_number_e_g_1_25",
            )
            .into(),
          );
          return;
        };
        let Some(monitor) = self.monitors.get(index).cloned() else {
          return;
        };
        let name = monitor.name.clone();
        let mut config = self.config.clone();
        let mut persisted = config.persisted(&name);
        persisted.scale = Some(value);
        config.set_monitor(&name, persisted);
        self.config = config.clone();
        let message = format!(
          "{} · {name}: {value}",
          tr(self.lang, "control_center.scale")
        );
        self.leave_prompt(DisplayPage::Detail(index));
        self.spawn_apply_with_config(config, message);
      }
      PromptGoal::SdrBrightness(index) => {
        let Some(value) = parse_number(&buffer) else {
          self.prompt_error =
            Some(tr(self.lang, "control_center.invalid_value_use_0_0_to_1_0").into());
          return;
        };
        self.apply_sdr(index, |persisted| {
          persisted.sdr_brightness = Some(value.clamp(0.0, 1.0))
        });
      }
      PromptGoal::SdrSaturation(index) => {
        let Some(value) = parse_number(&buffer) else {
          self.prompt_error =
            Some(tr(self.lang, "control_center.invalid_value_use_0_0_to_2_0").into());
          return;
        };
        self.apply_sdr(index, |saturation| {
          saturation.sdr_saturation = Some(value.clamp(0.0, 2.0))
        });
      }
      PromptGoal::ProfileCreate => {
        if buffer.trim().is_empty() {
          self.prompt_error =
            Some(tr(self.lang, "control_center.profile_name_cannot_be_empty").into());
          return;
        }
        if self.state.profile_index(buffer.trim()).is_some() {
          self.prompt_error = Some(
            tr(
              self.lang,
              "control_center.a_profile_with_this_name_already_exists",
            )
            .into(),
          );
          return;
        }
        let profile = MonitorProfile {
          name: buffer.trim().to_string(),
          apply_wallpapers: false,
          config: self.config.clone(),
        };
        self.state.profiles.push(profile);
        self.state.active_profile = None;
        let state = self.state.clone();
        self.spawn_save_state(
          state,
          format!(
            "{} · {}",
            tr(self.lang, "control_center.profile_created"),
            buffer.trim()
          ),
        );
        self.leave_prompt(DisplayPage::Profiles);
        let created = Item::Profile(self.state.profiles.len().saturating_sub(1));
        let rows = self.rows();
        self.menu.select(&rows, &created);
      }
      PromptGoal::ProfileRename(index) => {
        if buffer.trim().is_empty() {
          self.prompt_error =
            Some(tr(self.lang, "control_center.profile_name_cannot_be_empty").into());
          return;
        }
        if let Some(profile) = self.state.profiles.get_mut(index) {
          profile.name = buffer.trim().to_string();
        }
        let state = self.state.clone();
        self.spawn_save_state(
          state,
          format!(
            "{} · {}",
            tr(self.lang, "control_center.profile_renamed"),
            buffer.trim()
          ),
        );
        self.leave_prompt(DisplayPage::Profile(index));
      }
    }
  }

  /// Closes the prompt after a committed value and opens `page`.
  fn leave_prompt(&mut self, page: DisplayPage) {
    self.prompt_buffer.clear();
    self.prompt_error = None;
    self.prompt_back = None;
    self.go(page);
  }

  /// Applies the `apply_sdr` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_sdr<F: FnOnce(&mut PersistedMonitor)>(&mut self, index: usize, mutate: F) {
    let Some(monitor) = self.monitors.get(index).cloned() else {
      return;
    };
    let name = monitor.name.clone();
    let mut config = self.config.clone();
    let mut persisted = config.persisted(&name);
    mutate(&mut persisted);
    config.set_monitor(&name, persisted.clone());
    self.config = config.clone();
    let message = format!("{} · {name}", tr(self.lang, "control_center.sdr"));
    self.leave_prompt(DisplayPage::Detail(index));
    self.spawn_apply_with_config(config, message);
  }

  /// Executes the `spawn_save_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn spawn_save_state(&mut self, state: crate::model::DisplayState, message: String) {
    self.action = Some(self.manager.spawn(move |_| {
      backend::save_state(&state)?;
      Ok(JobData::Action(message))
    }));
  }

  /// Executes the `exit_picker` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn exit_picker(&mut self) {
    if let DisplayPage::Picker { monitor, .. } = self.page {
      self.go(DisplayPage::Detail(monitor));
    }
  }

  /// Executes the `stale_start` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn stale_start(&self) -> usize {
    self.monitors.len()
  }

  /// Names of saved configurations whose monitor is not present.
  fn stale_names(&self) -> Vec<String> {
    self
      .config
      .monitors
      .iter()
      .filter(|(name, _)| !self.monitors.iter().any(|monitor| &monitor.name == name))
      .map(|(name, _)| name.clone())
      .collect()
  }

  /// Executes the `stale_name` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn stale_name(&self, index: usize) -> Option<String> {
    let offset = index.checked_sub(self.stale_start())?;
    self.stale_names().get(offset).cloned()
  }

  /// Executes the `detail_settings` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn detail_settings(&self) -> Vec<MonitorSetting> {
    let index = match self.monitor_index() {
      Some(index) => index,
      None => return vec![],
    };
    let Some(monitor) = self.monitors.get(index) else {
      return vec![];
    };
    if !monitor.connected {
      return vec![];
    }
    let mut settings = vec![
      MonitorSetting::Resolution,
      MonitorSetting::RefreshRate,
      MonitorSetting::Scale,
      MonitorSetting::Position,
      MonitorSetting::Orientation,
      MonitorSetting::Enabled,
      MonitorSetting::Mirror,
      MonitorSetting::BitDepth,
    ];
    if self.vrr_supported() {
      settings.push(MonitorSetting::Vrr);
    }
    if self.hdr_supported() && monitor.has_10bit() {
      settings.push(MonitorSetting::Hdr);
    }
    settings.push(MonitorSetting::Dpms);
    if self.hdr_supported() && monitor.has_10bit() {
      settings.push(MonitorSetting::SdrBrightness);
      settings.push(MonitorSetting::SdrSaturation);
    }
    settings.push(MonitorSetting::Workspaces);
    settings.push(MonitorSetting::Primary);
    settings
  }

  /// Executes the `monitor_index` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn monitor_index(&self) -> Option<usize> {
    match self.page {
      DisplayPage::Detail(index) | DisplayPage::Picker { monitor: index, .. } => Some(index),
      DisplayPage::Home
      | DisplayPage::Profiles
      | DisplayPage::Profile(_)
      | DisplayPage::Prompt { .. } => None,
    }
  }

  /// Home: connected monitors, saved configurations of absent monitors,
  /// Profiles and Refresh.
  fn home_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let mut rows: Vec<Row<Item>> = Vec::new();
    for (index, monitor) in self.monitors.iter().enumerate() {
      let role = if self.config.primary_monitor.as_deref() == Some(monitor.name.as_str()) {
        tr(lang, "control_center.primary_ead365")
      } else if monitor.focused {
        tr(lang, "control_center.focused")
      } else {
        ""
      };
      let power = if monitor.disabled || monitor.dpms_status == "off" {
        tr(lang, "control_center.off")
      } else {
        tr(lang, "control_center.on")
      };
      let mut detail = format!(
        "{}x{} @ {:.3} Hz  ·  {}x{}  ·  {} {}  ·  {power}",
        monitor.width,
        monitor.height,
        monitor.refresh_rate,
        monitor.x,
        monitor.y,
        tr(lang, "control_center.scale"),
        monitor.scale,
      );
      if !role.is_empty() {
        detail.push_str(&format!("  ·  {role}"));
      }
      rows.push(
        Row::submenu(Item::Monitor(index), monitor.name.clone())
          .icon(icons::MONITOR)
          .detail(detail),
      );
    }
    for (offset, name) in self.stale_names().into_iter().enumerate() {
      let persisted = self.config.persisted(&name);
      rows.push(
        Row::submenu(Item::Stale(offset), name)
          .icon(icons::LINK_OFF)
          .detail(format!(
            "{}  ·  {}",
            tr(lang, "control_center.disconnected"),
            persisted
              .mode
              .as_deref()
              .unwrap_or(tr(lang, "control_center.configured")),
          )),
      );
    }
    if rows.is_empty() {
      rows.push(Row::info(
        tr(lang, "control_center.no_monitors_found_is_hyprland_running"),
        "",
      ));
    }
    rows.push(
      Row::submenu(Item::Profiles, tr(lang, "control_center.profiles"))
        .icon(icons::PROFILE)
        .detail(self.state.profiles.len().to_string()),
    );
    rows.push(Row::separator());
    rows.push(Row::action(Item::Refresh, tr(lang, "control_center.refresh")).icon(icons::REFRESH));
    rows
  }

  /// Detail of a monitor: its information, one row per setting and the
  /// actions; a disconnected monitor only offers to remove its saved
  /// configuration.
  fn detail_rows(&self, index: usize) -> Vec<Row<Item>> {
    let lang = self.lang;
    let connected = self.monitors.get(index).filter(|monitor| monitor.connected);
    let Some(monitor) = connected else {
      return vec![
        Row::info(tr(lang, "control_center.monitor_disconnected"), ""),
        Row::section(tr(lang, "control_center.danger_zone")),
        Row::destructive(Item::RemoveConfig, tr(lang, "control_center.remove_config"))
          .icon(icons::DELETE),
      ];
    };
    let mut rows = Vec::new();
    let info = monitor.info_rows();
    if !info.is_empty() {
      rows.push(Row::section(tr(lang, "control_center.monitor")));
      rows.extend(
        info
          .into_iter()
          .map(|(label, value)| Row::info(label, value)),
      );
    }
    rows.push(Row::section(tr(lang, "control_center.configuration")));
    rows.extend(self.detail_settings().into_iter().map(|setting| {
      Row::submenu(Item::Setting(setting), setting_label(lang, setting))
        .icon(setting_icon(setting))
        .detail(self.setting_value(monitor, setting))
    }));
    rows.push(Row::section(tr(lang, "control_center.section_actions")));
    rows.push(Row::action(Item::Apply, tr(lang, "control_center.apply")).icon(icons::APPLY));
    rows.push(
      Row::action(Item::Reset, tr(lang, "control_center.restore_default")).icon(icons::RESTORE),
    );
    rows
  }

  /// Current value of `setting`, preferring the saved override.
  fn setting_value(&self, monitor: &Monitor, setting: MonitorSetting) -> String {
    let lang = self.lang;
    let persisted = self.config.persisted(&monitor.name);
    match setting {
      MonitorSetting::Resolution => persisted
        .mode
        .clone()
        .and_then(|mode| mode.split('@').next().map(str::to_string))
        .unwrap_or_else(|| format!("{}x{}", monitor.width, monitor.height)),
      MonitorSetting::RefreshRate => format!(
        "{:.3} Hz",
        persisted
          .mode
          .as_deref()
          .and_then(rate_of)
          .unwrap_or(monitor.refresh_rate)
      ),
      MonitorSetting::Scale => persisted.scale.unwrap_or(monitor.scale).to_string(),
      MonitorSetting::Position => persisted
        .position
        .clone()
        .unwrap_or(format!("{}x{}", monitor.x, monitor.y)),
      MonitorSetting::Orientation => {
        transform_label(lang, persisted.transform.unwrap_or(monitor.transform))
      }
      MonitorSetting::Enabled => on_off(lang, persisted.disabled != Some(true)),
      MonitorSetting::Mirror => self
        .current_mirror(monitor)
        .filter(|mirror| !mirror.is_empty())
        .unwrap_or_else(|| tr(lang, "control_center.none").into()),
      MonitorSetting::BitDepth => self.bitdepth_label(monitor, &persisted),
      MonitorSetting::Vrr => vrr_label(lang, persisted.vrr.unwrap_or(monitor.vrr)),
      MonitorSetting::Hdr => hdr_label(lang, persisted.hdr.unwrap_or(0)),
      MonitorSetting::Dpms => if monitor.dpms_status == "off" {
        tr(lang, "control_center.off")
      } else {
        tr(lang, "control_center.on")
      }
      .into(),
      MonitorSetting::SdrBrightness => persisted
        .sdr_brightness
        .or(monitor.info.sdr_brightness)
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| tr(lang, "control_center.auto").into()),
      MonitorSetting::SdrSaturation => persisted
        .sdr_saturation
        .or(monitor.info.sdr_saturation)
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| tr(lang, "control_center.auto").into()),
      MonitorSetting::Workspaces => {
        let workspaces = self.config.workspaces_of(&monitor.name);
        if workspaces.is_empty() {
          tr(lang, "control_center.unbound").into()
        } else {
          workspaces
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ")
        }
      }
      MonitorSetting::Primary => on_off(
        lang,
        self.config.primary_monitor.as_deref() == Some(monitor.name.as_str()),
      ),
    }
  }

  fn current_mirror(&self, monitor: &Monitor) -> Option<String> {
    self
      .config
      .persisted(&monitor.name)
      .mirror
      .or_else(|| monitor.info.mirror_of.clone())
  }

  /// Executes the `bitdepth_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn bitdepth_label(&self, monitor: &Monitor, persisted: &PersistedMonitor) -> String {
    if !monitor.has_10bit() {
      return tr(self.lang, "control_center.unsupported").into();
    }
    match persisted.bitdepth {
      Some(10) => "10".into(),
      Some(8) => "8".into(),
      _ if monitor.info.current_format.contains("10") => "10".into(),
      _ => "8".into(),
    }
  }

  /// Options of the open setting: Choice rows with `●` on the current
  /// value, Toggle rows for workspaces, and a Value row for a custom value.
  fn picker_rows(&self, monitor_index: usize, setting: MonitorSetting) -> Vec<Row<Item>> {
    let lang = self.lang;
    let Some(monitor) = self.monitors.get(monitor_index) else {
      return vec![Row::info(
        tr(lang, "control_center.no_options_available"),
        "",
      )];
    };
    let mut rows = Vec::new();
    let mut custom = None;
    for (index, option) in self
      .options_for(monitor_index, setting)
      .into_iter()
      .enumerate()
    {
      if option.prompt.is_some() {
        custom = Some(option);
        continue;
      }
      let item = Item::Option(index);
      rows.push(match (&option.persist, setting) {
        (PersistChange::Workspace(workspace, target), MonitorSetting::Workspaces) => {
          let (label, detail) = option
            .label
            .split_once("  ·  ")
            .map(|(label, detail)| (label.to_string(), detail.to_string()))
            .unwrap_or((workspace.to_string(), String::new()));
          // Bound here means selecting it unbinds (target `None`).
          Row::toggle(item, label, target.is_none()).detail(detail)
        }
        (PersistChange::None, _)
          if option.label == tr(lang, "control_center.unsupported_7c5d5f") =>
        {
          // "Unsupported" placeholder of a gated setting.
          Row::info(option.label, "")
        }
        _ => {
          let current = self.option_is_current(monitor, setting, index, &option);
          Row::choice(item, option.label, current)
        }
      });
    }
    if let Some(option) = custom {
      rows.push(Row::separator());
      rows.push(
        Row::value(
          Item::Custom,
          option.label.trim_start_matches("✎ ").to_string(),
          self.setting_value(monitor, setting),
          None,
        )
        .icon(icons::EDIT),
      );
    }
    if rows.is_empty() {
      rows.push(Row::info(
        tr(lang, "control_center.no_options_available"),
        "",
      ));
    }
    rows
  }

  /// Whether `option` is the value currently in effect for `setting`.
  fn option_is_current(
    &self,
    monitor: &Monitor,
    setting: MonitorSetting,
    index: usize,
    option: &PickerOption,
  ) -> bool {
    let persisted = self.config.persisted(&monitor.name);
    let current_mode = persisted.mode.clone().unwrap_or_else(|| {
      format!(
        "{}x{}@{}",
        monitor.width,
        monitor.height,
        trimmed_rate(monitor.refresh_rate)
      )
    });
    let size_of = |mode: &str| mode.split('@').next().unwrap_or_default().to_string();
    match (&option.persist, setting) {
      (PersistChange::Mode(mode), MonitorSetting::Resolution) => {
        size_of(mode) == size_of(&current_mode)
      }
      (PersistChange::Mode(mode), _) => {
        size_of(mode) == size_of(&current_mode)
          && match (rate_of(mode), rate_of(&current_mode)) {
            (Some(left), Some(right)) => (left - right).abs() < 0.01,
            _ => false,
          }
      }
      (PersistChange::Scale(value), _) => {
        (value - persisted.scale.unwrap_or(monitor.scale)).abs() < 0.001
      }
      (PersistChange::Disabled(disabled), _) => *disabled == (persisted.disabled == Some(true)),
      (PersistChange::Mirror(mirror), _) => {
        *mirror == self.current_mirror(monitor).unwrap_or_default()
      }
      (PersistChange::BitDepth(depth), _) => {
        depth.to_string() == self.bitdepth_label(monitor, &persisted)
      }
      (PersistChange::Vrr(value), _) => *value == persisted.vrr.unwrap_or(monitor.vrr),
      (PersistChange::Hdr(value), _) => *value == persisted.hdr.unwrap_or(0),
      (PersistChange::Dpms(on), _) => *on == (monitor.dpms_status != "off"),
      (PersistChange::Primary, _) => {
        self.config.primary_monitor.as_deref() == Some(option.args.as_str())
      }
      (PersistChange::None, MonitorSetting::Orientation) => {
        index as i32 == persisted.transform.unwrap_or(monitor.transform)
      }
      _ => false,
    }
  }

  /// The free-form editor of the open setting.
  fn custom_prompt(&self) -> Option<PromptGoal> {
    self
      .picker_options()
      .into_iter()
      .find_map(|option| option.prompt)
  }

  /// Executes the `picker_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn picker_options(&self) -> Vec<PickerOption> {
    match self.monitor_ctx() {
      Some((monitor_index, setting)) => self.options_for(monitor_index, setting),
      None => vec![],
    }
  }

  /// Options offered for `setting` of the monitor `monitor_index`.
  fn options_for(&self, monitor_index: usize, setting: MonitorSetting) -> Vec<PickerOption> {
    let Some(monitor) = self.monitors.get(monitor_index) else {
      return vec![];
    };
    match setting {
      MonitorSetting::Resolution => resolution_options(monitor),
      MonitorSetting::RefreshRate => refresh_options(monitor),
      MonitorSetting::Scale => {
        let mut options = scale_options(monitor);
        options.push(PickerOption {
          label: tr(self.lang, "control_center.custom").into(),
          args: String::new(),
          persist: PersistChange::None,
          prompt: Some(PromptGoal::Scale(monitor_index)),
        });
        options
      }
      MonitorSetting::Position => {
        let mut options = self.position_options(monitor_index);
        options.push(PickerOption {
          label: tr(self.lang, "control_center.custom").into(),
          args: String::new(),
          persist: PersistChange::None,
          prompt: Some(PromptGoal::Position(monitor_index)),
        });
        options
      }
      MonitorSetting::Primary => self.primary_options(),
      MonitorSetting::Enabled => enabled_options(monitor, self.lang),
      MonitorSetting::Mirror => mirror_options(monitor, &self.monitors, self.lang),
      MonitorSetting::BitDepth if monitor.has_10bit() => bitdepth_options(monitor, self.lang),
      MonitorSetting::Vrr if self.vrr_supported() => vrr_options(monitor, self.lang),
      MonitorSetting::Hdr if self.hdr_supported() && monitor.has_10bit() => {
        hdr_options(monitor, self.lang)
      }
      MonitorSetting::Dpms => dpms_options(monitor, self.lang),
      MonitorSetting::Orientation => orientation_options(monitor, self.lang),
      MonitorSetting::SdrBrightness if self.hdr_supported() && monitor.has_10bit() => {
        let mut options = sdr_brightness_options(self.lang, monitor);
        options.push(PickerOption {
          label: tr(self.lang, "control_center.custom").into(),
          args: String::new(),
          persist: PersistChange::None,
          prompt: Some(PromptGoal::SdrBrightness(monitor_index)),
        });
        options
      }
      MonitorSetting::SdrSaturation if self.hdr_supported() && monitor.has_10bit() => {
        let mut options = sdr_saturation_options(self.lang, monitor);
        options.push(PickerOption {
          label: tr(self.lang, "control_center.custom").into(),
          args: String::new(),
          persist: PersistChange::None,
          prompt: Some(PromptGoal::SdrSaturation(monitor_index)),
        });
        options
      }
      MonitorSetting::Workspaces => self.workspace_options(monitor_index),
      MonitorSetting::BitDepth => vec![PickerOption {
        label: tr(self.lang, "control_center.unsupported_7c5d5f").into(),
        args: String::new(),
        persist: PersistChange::None,
        prompt: None,
      }],
      MonitorSetting::Vrr | MonitorSetting::Hdr => vec![PickerOption {
        label: tr(self.lang, "control_center.unsupported_7c5d5f").into(),
        args: String::new(),
        persist: PersistChange::None,
        prompt: None,
      }],
      MonitorSetting::SdrBrightness | MonitorSetting::SdrSaturation => vec![PickerOption {
        label: tr(self.lang, "control_center.unsupported_7c5d5f").into(),
        args: String::new(),
        persist: PersistChange::None,
        prompt: None,
      }],
    }
  }

  /// Executes the `primary_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn primary_options(&self) -> Vec<PickerOption> {
    self
      .monitors
      .iter()
      .filter(|monitor| monitor.connected)
      .map(|monitor| PickerOption {
        label: monitor.name.clone(),
        args: monitor.name.clone(),
        persist: PersistChange::Primary,
        prompt: None,
      })
      .collect()
  }

  /// Executes the `workspace_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn workspace_options(&self, index: usize) -> Vec<PickerOption> {
    let Some(monitor) = self.monitors.get(index) else {
      return vec![];
    };
    let name = monitor.name.clone();
    let owned = self.config.workspaces_of(&name);
    let mut options = Vec::new();
    for workspace in 1..=10 {
      let bound_here = owned.contains(&workspace);
      let bound_elsewhere = self
        .config
        .workspaces
        .iter()
        .filter(|(other, _)| *other != name)
        .any(|(_, ids)| ids.contains(&workspace));
      let label = if bound_here {
        format!(
          "{}  ·  {}",
          workspace,
          tr(self.lang, "control_center.this_monitor")
        )
      } else if bound_elsewhere {
        format!(
          "{}  ·  {}",
          workspace,
          tr(self.lang, "control_center.another_monitor")
        )
      } else {
        format!(
          "{}  ·  {}",
          workspace,
          tr(self.lang, "control_center.unbound")
        )
      };
      let persist = if bound_here {
        PersistChange::Workspace(workspace, None)
      } else {
        PersistChange::Workspace(workspace, Some(name.clone()))
      };
      options.push(PickerOption {
        label,
        args: String::new(),
        persist,
        prompt: None,
      });
    }
    options
  }

  /// Executes the `position_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn position_options(&self, index: usize) -> Vec<PickerOption> {
    let Some(monitor) = self.monitors.get(index) else {
      return vec![];
    };
    let mut options = vec![PickerOption {
      label: tr(self.lang, "control_center.top_left_corner_0_0").into(),
      args: monitor.position_args(0, 0),
      persist: PersistChange::Position(0, 0),
      prompt: None,
    }];
    for (other_index, other) in self.monitors.iter().enumerate() {
      if other_index == index || !other.connected {
        continue;
      }
      let placements = [
        (
          tr(self.lang, "control_center.to_the_right_of"),
          other.x + other.width as i32,
          other.y,
        ),
        (
          tr(self.lang, "control_center.to_the_left_of"),
          other.x - monitor.width as i32,
          other.y,
        ),
        (
          tr(self.lang, "control_center.below"),
          other.x,
          other.y + other.height as i32,
        ),
        (
          tr(self.lang, "control_center.above"),
          other.x,
          other.y - monitor.height as i32,
        ),
      ];
      for (label, x, y) in placements {
        options.push(PickerOption {
          label: format!("{label} {}", other.name),
          args: monitor.position_args(x, y),
          persist: PersistChange::Position(x, y),
          prompt: None,
        });
      }
    }
    options.sort_by(|left, right| left.label.cmp(&right.label));
    options
  }

  /// Applies the `apply_persist` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_persist(config: &mut PersistedConfig, name: &str, persist: &PersistChange) {
    match persist {
      PersistChange::Workspace(workspace, target) => {
        for (_, ids) in config.workspaces.iter_mut() {
          ids.retain(|id| id != workspace);
        }
        if let Some(target) = target {
          let mut ids = config.workspaces_of(target);
          if !ids.contains(workspace) {
            ids.push(*workspace);
          }
          ids.sort_unstable();
          config.set_workspaces(target, ids);
        }
      }
      _ => {
        let mut monitored = config.persisted(name);
        match persist {
          PersistChange::Mode(value) => monitored.mode = Some(value.clone()),
          PersistChange::Scale(value) => monitored.scale = Some(*value),
          PersistChange::Position(x, y) => monitored.position = Some(format!("{x}x{y}")),
          PersistChange::Vrr(value) => monitored.vrr = Some(*value),
          PersistChange::Hdr(value) => monitored.hdr = Some(*value),
          PersistChange::Mirror(value) => monitored.mirror = Some(value.clone()),
          PersistChange::BitDepth(value) => monitored.bitdepth = Some(*value),
          PersistChange::Disabled(value) => monitored.disabled = Some(*value),
          _ => return,
        }
        config.set_monitor(name, monitored);
      }
    }
  }

  /// Executes the `monitor_ctx` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn monitor_ctx(&self) -> Option<(usize, MonitorSetting)> {
    match self.page {
      DisplayPage::Picker { monitor, setting } => Some((monitor, setting)),
      DisplayPage::Detail(_)
      | DisplayPage::Home
      | DisplayPage::Profiles
      | DisplayPage::Profile(_)
      | DisplayPage::Prompt { .. } => None,
    }
  }

  /// Executes the `spawn_revert` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn spawn_revert(&mut self, config: PersistedConfig, message: String) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.reverting").into(),
    });
    self.action = Some(self.manager.spawn(move |_| {
      backend::apply_all(&config)?;
      Ok(JobData::Action(message))
    }));
  }

  /// Executes the `spawn_apply_with_config` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn spawn_apply_with_config(&mut self, config: PersistedConfig, message: String) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_to_monitor").into(),
    });
    self.action = Some(self.manager.spawn(move |_| {
      backend::apply_all(&config)?;
      Ok(JobData::Action(message))
    }));
  }

  /// Executes the `vrr_supported` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn vrr_supported(&self) -> bool {
    self.version >= (0, 31)
  }

  /// Executes the `hdr_supported` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn hdr_supported(&self) -> bool {
    self.version >= (0, 42)
  }

  /// Executes the `delete_profile` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn delete_profile(&mut self, index: usize) {
    if index < self.state.profiles.len() {
      self.state.profiles.remove(index);
    }
    let state = self.state.clone();
    self.action = Some(self.manager.spawn(move |_| {
      backend::save_state(&state)?;
      Ok(JobData::Action(String::new()))
    }));
  }

  /// Executes the `open_prompt` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_prompt(&mut self, goal: PromptGoal) {
    self.prompt_back = Some(self.page);
    self.prompt_buffer = self.prompt_prefill(goal);
    self.prompt_error = None;
    self.page = DisplayPage::Prompt { goal };
  }

  /// Executes the `prompt_prefill` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn prompt_prefill(&self, goal: PromptGoal) -> String {
    match goal {
      PromptGoal::Position(index) => self
        .monitors
        .get(index)
        .map(|monitor| {
          self
            .config
            .persisted(&monitor.name)
            .position
            .clone()
            .unwrap_or_else(|| format!("{},{}", monitor.x, monitor.y))
        })
        .unwrap_or_default(),
      PromptGoal::Scale(index) => self
        .monitors
        .get(index)
        .map(|monitor| {
          self
            .config
            .persisted(&monitor.name)
            .scale
            .map(|value| value.to_string())
            .unwrap_or_else(|| monitor.scale.to_string())
        })
        .unwrap_or_default(),
      PromptGoal::SdrBrightness(index) => self
        .monitors
        .get(index)
        .and_then(|monitor| {
          self
            .config
            .persisted(&monitor.name)
            .sdr_brightness
            .or(monitor.info.sdr_brightness)
            .map(|value| value.to_string())
        })
        .unwrap_or_else(|| "1.0".into()),
      PromptGoal::SdrSaturation(index) => self
        .monitors
        .get(index)
        .and_then(|monitor| {
          self
            .config
            .persisted(&monitor.name)
            .sdr_saturation
            .or(monitor.info.sdr_saturation)
            .map(|value| value.to_string())
        })
        .unwrap_or_else(|| "1.0".into()),
      PromptGoal::ProfileCreate => String::new(),
      PromptGoal::ProfileRename(index) => self
        .state
        .profiles
        .get(index)
        .map(|profile| profile.name.clone())
        .unwrap_or_default(),
    }
  }

  /// Executes the `reload_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn reload_state(&mut self) {
    self.state = backend::load_state();
  }

  /// Executes the `remove_button` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn remove_button(&mut self) {
    let index = self.monitor_index();
    let name = match index {
      Some(index) if index < self.monitors.len() => Some(self.monitors[index].name.clone()),
      Some(index) => self.stale_name(index),
      None => None,
    };
    let Some(name) = name else {
      return;
    };
    self
      .config
      .monitors
      .retain(|(existing, _)| existing != &name);
    self
      .config
      .workspaces
      .retain(|(monitor, _)| monitor != &name);
    if self.config.primary_monitor.as_deref() == Some(&name) {
      self.config.primary_monitor = None;
      self.state.primary_monitor = None;
    }
    let config = self.config.clone();
    let state = self.state.clone();
    let lang = self.lang;
    let removed = tr(lang, "control_center.config_removed").to_string();
    self.action = Some(self.manager.spawn(move |_| {
      backend::save_config(&config)?;
      backend::save_state(&state)?;
      Ok(JobData::Action(format!("{removed} · {name}")))
    }));
    self.go(DisplayPage::Home);
  }

  /// Executes the `reset_button` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn reset_button(&mut self) {
    let Some(index) = self.monitor_index() else {
      return;
    };
    let Some(monitor) = self.monitors.get(index) else {
      return;
    };
    let name = monitor.name.clone();
    self
      .config
      .monitors
      .retain(|(existing, _)| existing != &name);
    let config = self.config.clone();
    self.spawn_apply_with_config(
      config,
      format!(
        "{} · {name}",
        tr(self.lang, "control_center.default_applied")
      ),
    );
  }

  /// Applies the `apply_button` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_button(&mut self) {
    let Some(index) = self.monitor_index() else {
      return;
    };
    let Some(monitor) = self.monitors.get(index) else {
      return;
    };
    let name = monitor.name.clone();
    let _persisted = self.config.persisted(&name);
    let previous = monitor.clone();
    let previous_config = self.config.clone();
    self.arm_revert(name.clone(), previous, previous_config, Instant::now());
    self.spawn_apply_with_config(
      self.config.clone(),
      format!("{} · {name}", tr(self.lang, "control_center.applied")),
    );
  }

  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::DesktopInput.label(self.lang),
      tr(self.lang, "control_center.displays")
    );
    match self.page {
      DisplayPage::Home => root,
      DisplayPage::Detail(index) => {
        let name = self
          .monitors
          .get(index)
          .map(|monitor| monitor.name.clone())
          .or_else(|| self.stale_name(index))
          .unwrap_or_else(|| tr(self.lang, "control_center.monitor").into());
        format!("{root} > {name}")
      }
      DisplayPage::Picker { setting, .. } => {
        format!("{root} > {}", setting_label(self.lang, setting))
      }
      DisplayPage::Profiles => format!("{root} > {}", tr(self.lang, "control_center.profiles")),
      DisplayPage::Profile(index) => format!(
        "{root} > {} > {}",
        tr(self.lang, "control_center.profiles"),
        self
          .state
          .profiles
          .get(index)
          .map(|profile| profile.name.as_str())
          .unwrap_or_default()
      ),
      DisplayPage::Prompt { goal } => format!(
        "{root} > {}",
        match goal {
          PromptGoal::Position(_) => tr(self.lang, "control_center.position"),
          PromptGoal::Scale(_) => tr(self.lang, "control_center.scale"),
          PromptGoal::SdrBrightness(_) => tr(self.lang, "control_center.sdr_brightness"),
          PromptGoal::SdrSaturation(_) => tr(self.lang, "control_center.sdr_saturation"),
          PromptGoal::ProfileCreate => tr(self.lang, "control_center.new_profile"),
          PromptGoal::ProfileRename(_) => tr(self.lang, "control_center.rename_profile"),
        }
      ),
    }
  }

  /// Renders `draw_prompt` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn draw_prompt(&mut self, frame: &mut Frame, area: Rect, goal: PromptGoal) {
    let width = area.width.saturating_sub(6).clamp(34, 76);
    let height = 9u16.min(area.height);
    let x = area.x + (area.width - width).saturating_div(2);
    let y = area.y + (area.height.saturating_sub(height)).saturating_div(2);
    let box_area = Rect::new(x, y, width, height);
    frame.render_widget(Clear, box_area);
    let title = match goal {
      PromptGoal::Position(_) => tr(self.lang, "control_center.position_x_y"),
      PromptGoal::Scale(_) => tr(self.lang, "control_center.scale"),
      PromptGoal::SdrBrightness(_) => tr(self.lang, "control_center.sdr_brightness"),
      PromptGoal::SdrSaturation(_) => tr(self.lang, "control_center.sdr_saturation"),
      PromptGoal::ProfileCreate => tr(self.lang, "control_center.new_profile"),
      PromptGoal::ProfileRename(_) => tr(self.lang, "control_center.rename_profile"),
    };
    let display = if self.prompt_buffer.is_empty() {
      " ".into()
    } else {
      self.prompt_buffer.clone()
    };
    let mut lines = vec![Line::from("")];
    lines.push(Line::from(vec![
      Span::styled("> ", Style::new().fg(self.theme.accent)),
      Span::styled(
        display,
        Style::new()
          .fg(self.theme.foreground)
          .add_modifier(Modifier::BOLD),
      ),
    ]));
    lines.push(Line::from(""));
    if let Some(error) = &self.prompt_error {
      lines.push(Line::from(vec![Span::styled(
        argvus_tui::icons::icon_label(argvus_tui::icons::WARNING, error),
        Style::new().fg(self.theme.error),
      )]));
    }
    frame.render_widget(
      Paragraph::new(lines)
        .block(
          Block::bordered()
            .title(format!(" {title} "))
            .border_style(Style::new().fg(self.theme.border_active)),
        )
        .style(Style::new().bg(self.theme.background)),
      box_area,
    );
    self.overlays(frame, area);
  }

  /// Applies the picker option `index` (immediate, as before; risky options
  /// arm the revert countdown) and goes back to the monitor.
  fn apply_picker_selection(&mut self, index: usize) {
    let Some((monitor_index, setting)) = self.monitor_ctx() else {
      return;
    };
    let options = self.picker_options();
    let Some(option) = options.get(index).cloned() else {
      return;
    };
    if let Some(prompt) = option.prompt {
      self.open_prompt(prompt);
      return;
    }
    if setting == MonitorSetting::Primary {
      if let Some(monitor) = self.monitors.get(monitor_index) {
        let name = monitor.name.clone();
        self.config.primary_monitor = Some(name.clone());
        self.state.primary_monitor = Some(name.clone());
        let config = self.config.clone();
        let state = self.state.clone();
        let message = format!(
          "{}: {name}",
          tr(self.lang, "control_center.primary_monitor")
        );
        self.action = Some(self.manager.spawn(move |_| {
          backend::save_config(&config)?;
          backend::save_state(&state)?;
          Ok(JobData::Action(message))
        }));
      }
      self.go(DisplayPage::Detail(monitor_index));
      return;
    }
    let Some(monitor) = self.monitors.get(monitor_index).cloned() else {
      return;
    };
    let name = monitor.name.clone();
    let previous = monitor.clone();
    let prev_config = self.config.clone();
    let mut config = self.config.clone();
    Self::apply_persist(&mut config, &name, &option.persist);
    if option_is_risky(&option.persist) {
      self.arm_revert(name.clone(), previous, prev_config, Instant::now());
    }
    let persist = option.persist.clone();
    let message = format!(
      "{} · {}",
      monitor.name,
      option.label.split("  ·  ").next().unwrap_or("")
    );
    self.config = config.clone();
    self.go(DisplayPage::Detail(monitor_index));
    self.action = Some(self.manager.spawn(move |_| {
      match persist {
        PersistChange::Dpms(true) => backend::set_dpms(&name, true)?,
        PersistChange::Dpms(_) => backend::set_dpms(&name, false)?,
        PersistChange::Workspace(workspace, target) => {
          backend::apply_workspace(workspace, target.as_deref())?;
          backend::save_config(&config)?;
        }
        _ => {
          backend::apply_all(&config)?;
        }
      }
      Ok(JobData::Action(message))
    }));
  }

  /// Saved profiles (Enter opens one) and New profile.
  fn profile_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let mut rows: Vec<Row<Item>> = self
      .state
      .profiles
      .iter()
      .enumerate()
      .map(|(index, profile)| {
        let row = Row::submenu(Item::Profile(index), profile.name.clone());
        if self.state.active_profile.as_deref() == Some(profile.name.as_str()) {
          row.detail(tr(lang, "control_center.active"))
        } else {
          row
        }
      })
      .collect();
    if !rows.is_empty() {
      rows.push(Row::separator());
    }
    rows
      .push(Row::action(Item::NewProfile, tr(lang, "control_center.new_profile")).icon(icons::ADD));
    rows
  }

  /// One profile: Apply and Rename, with Delete in the Danger zone (D4).
  fn profile_page_rows(&self, index: usize) -> Vec<Row<Item>> {
    let lang = self.lang;
    let Some(profile) = self.state.profiles.get(index) else {
      return vec![Row::info(
        tr(lang, "control_center.no_options_available"),
        "",
      )];
    };
    let mut rows = vec![
      Row::info(tr(lang, "control_center.profiles"), profile.name.clone()),
      Row::info(
        tr(lang, "control_center.profile_monitors"),
        profile.config.monitors.len().to_string(),
      ),
    ];
    if self.state.active_profile.as_deref() == Some(profile.name.as_str()) {
      rows.push(Row::info(tr(lang, "control_center.active"), ""));
    }
    rows.extend([
      Row::section(tr(lang, "control_center.section_actions")),
      Row::action(Item::ApplyProfile, tr(lang, "control_center.apply_profile")).icon(icons::APPLY),
      Row::action(Item::RenameProfile, tr(lang, "control_center.rename")).icon(icons::EDIT),
      Row::section(tr(lang, "control_center.danger_zone")),
      Row::destructive(
        Item::DeleteProfile,
        tr(lang, "control_center.delete_profile"),
      )
      .icon(icons::DELETE),
    ]);
    rows
  }

  /// Applies the `apply_profile` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_profile(&mut self, index: usize) {
    let Some(profile) = self.state.profiles.get(index).cloned() else {
      self.status = Some(StatusMessage {
        kind: StatusKind::Warning,
        text: tr(self.lang, "control_center.select_a_profile_to_apply").into(),
      });
      return;
    };
    self.config = profile.config.clone();
    self.state.primary_monitor = profile.config.primary_monitor.clone();
    self.state.active_profile = Some(profile.name.clone());
    let config = self.config.clone();
    let state = self.state.clone();
    let rules: Vec<String> = profile
      .config
      .monitors
      .iter()
      .map(|(name, persisted)| rule_from_persisted(name, persisted))
      .collect();
    let workspaces: Vec<(u32, String)> = profile
      .config
      .workspaces
      .iter()
      .flat_map(|(monitor, ids)| ids.iter().map(|id| (*id, monitor.clone())))
      .collect();
    let lang = self.lang;
    let profile_applied = tr(lang, "control_center.profile_applied").to_string();
    self.action = Some(self.manager.spawn(move |_| {
      for rule in &rules {
        backend::apply_keyword(rule)?;
      }
      for (workspace, monitor) in &workspaces {
        backend::apply_workspace(*workspace, Some(monitor))?;
      }
      backend::save_config(&config)?;
      backend::save_state(&state)?;
      Ok(JobData::Action(format!(
        "{profile_applied} · {}",
        state.active_profile.as_deref().unwrap_or("")
      )))
    }));
    self.refresh();
  }

  /// Footer: the revert keys while the countdown runs, the editor keys in a
  /// prompt, otherwise derived from the selected row.
  fn footer_hints(&self, rows: &[Row<Item>]) -> String {
    let label = |key: &str| tr(self.lang, key);
    if self.revert.is_some() {
      // Keeping is the explicit choice, so `y` comes first.
      return [
        ("y", label("control_center.keep")),
        ("n/Esc", label("control_center.revert")),
        ("↑/↓", label("control_center.hint.select")),
      ]
      .map(|(keys, action)| format!("{keys} {action}"))
      .join(FOOTER_GAP);
    }
    if self.confirm.is_some() {
      return confirm_hints(self.lang);
    }
    if let DisplayPage::Prompt { .. } = self.page {
      return [
        ("Enter", label("control_center.hint.confirm")),
        ("Esc", label("control_center.hint.back")),
      ]
      .map(|(keys, action)| format!("{keys} {action}"))
      .join(FOOTER_GAP);
    }
    let mut extra = Vec::new();
    if matches!(self.page, DisplayPage::Picker { .. }) {
      extra.push(("r", label("control_center.cancel")));
    }
    let mut menu = self.menu;
    menu.normalize(rows);
    hints(
      self.lang,
      &HintContext {
        row: menu.selected_kind(rows),
        can_go_back: true,
        refresh: matches!(
          self.page,
          DisplayPage::Home
            | DisplayPage::Detail(_)
            | DisplayPage::Profiles
            | DisplayPage::Profile(_)
        ),
        extra: &extra,
        ..HintContext::default()
      },
    )
  }

  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let rows = self.rows();
    self.menu.normalize(&rows);
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(&rows),
    );
    if let DisplayPage::Prompt { goal } = self.page {
      self.draw_prompt(frame, body, goal);
      return;
    }
    self.list_height = body.height;
    draw_menu(
      frame,
      body,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    self.overlays(frame, area);
  }

  /// Executes the `overlays` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn overlays(&self, frame: &mut Frame, area: Rect) {
    if let Some(revert) = &self.revert {
      let message = format!(
        "{}\n{}",
        revert.name,
        tr(self.lang, "control_center.keep_configuration_description")
      );
      let dialog = ConfirmDialog {
        title: tr(self.lang, "control_center.keep_configuration_title"),
        message: &message,
        confirm: tr(self.lang, "control_center.keep"),
        cancel: tr(self.lang, "control_center.revert"),
        danger: false,
        deadline: Some(revert.deadline.saturating_duration_since(Instant::now())),
      };
      draw_confirm(frame, area, &self.theme, dialog, &self.revert_focus);
      return;
    }
    if let Some((confirmation, focus)) = &self.confirm {
      let (title, message, confirm) = match confirmation {
        Confirmation::DeleteProfile(index) => (
          "control_center.delete_profile",
          format!(
            "{}\n{}",
            tr(self.lang, "control_center.delete_this_profile"),
            self
              .state
              .profiles
              .get(*index)
              .map(|profile| profile.name.as_str())
              .unwrap_or("")
          ),
          "control_center.delete",
        ),
        Confirmation::RemoveConfig => (
          "control_center.remove_config",
          format!(
            "{}\n{}",
            self
              .monitor_index()
              .and_then(|index| {
                self
                  .monitors
                  .get(index)
                  .map(|monitor| monitor.name.clone())
                  .or_else(|| self.stale_name(index))
              })
              .unwrap_or_default(),
            tr(self.lang, "control_center.remove_config_description")
          ),
          "control_center.remove_config",
        ),
      };
      let dialog = ConfirmDialog {
        title: tr(self.lang, title),
        message: &message,
        confirm: tr(self.lang, confirm),
        cancel: tr(self.lang, "control_center.cancel"),
        danger: true,
        deadline: None,
      };
      draw_confirm(frame, area, &self.theme, dialog, focus);
      return;
    }
    if let Some(s) = &self.status {
      status(frame, area, &self.theme, s);
    }
  }
}

/// The item's own icon for each monitor setting.
fn setting_icon(setting: MonitorSetting) -> &'static str {
  match setting {
    MonitorSetting::Resolution => icons::ASPECT_RATIO,
    MonitorSetting::RefreshRate => icons::SINE_WAVE,
    MonitorSetting::Scale => icons::ZOOM,
    MonitorSetting::Position => icons::ARROW_ALL,
    MonitorSetting::Orientation => icons::ROTATE,
    MonitorSetting::Enabled => icons::POWER,
    MonitorSetting::Mirror => icons::MONITOR_MULTIPLE,
    MonitorSetting::BitDepth => icons::PALETTE,
    MonitorSetting::Vrr => icons::SYNC,
    MonitorSetting::Hdr => icons::HDR,
    MonitorSetting::Dpms => icons::SLEEP,
    MonitorSetting::SdrBrightness => icons::BRIGHTNESS,
    MonitorSetting::SdrSaturation => icons::CONTRAST,
    MonitorSetting::Workspaces => icons::GRID,
    MonitorSetting::Primary => icons::STAR,
  }
}

/// Status shown after an automatic or chosen revert ("Reverted <name>").
fn revert_message(lang: Lang, revert: &RevertState) -> String {
  format!(
    "{} {}",
    tr(lang, "control_center.reverted"),
    revert.previous.name
  )
}

/// Executes the `resolution_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn resolution_options(monitor: &Monitor) -> Vec<PickerOption> {
  let mut unique: Vec<(u32, u32, f64)> = Vec::new();
  for mode in &monitor.modes {
    if !unique
      .iter()
      .any(|(w, h, _)| *w == mode.width && *h == mode.height)
    {
      unique.push((mode.width, mode.height, mode.refresh_rate));
    }
  }
  unique.sort_by(|left, right| {
    (right.0 * right.1).cmp(&(left.0 * left.1)).then(
      right
        .2
        .partial_cmp(&left.2)
        .unwrap_or(std::cmp::Ordering::Equal),
    )
  });
  unique
    .into_iter()
    .map(|(width, height, rate)| PickerOption {
      label: format!("{width}x{height}  ·  {:.3} Hz", rate),
      args: monitor.resolution_args(width, height, rate),
      persist: PersistChange::Mode(format!("{}x{}@{}", width, height, trimmed_rate(rate))),
      prompt: None,
    })
    .collect()
}

/// Executes the `refresh_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn refresh_options(monitor: &Monitor) -> Vec<PickerOption> {
  let mut modes = monitor.modes.clone();
  modes.sort_by(|left, right| {
    right
      .refresh_rate
      .partial_cmp(&left.refresh_rate)
      .unwrap_or(std::cmp::Ordering::Equal)
  });
  modes
    .into_iter()
    .map(|mode: crate::model::Mode| PickerOption {
      label: mode.label(),
      args: monitor.mode_id_args(&mode),
      persist: PersistChange::Mode(format!(
        "{}x{}@{}",
        mode.width,
        mode.height,
        trimmed_rate(mode.refresh_rate)
      )),
      prompt: None,
    })
    .collect()
}

/// Executes the `scale_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn scale_options(monitor: &Monitor) -> Vec<PickerOption> {
  [1.0_f64, 0.75, 1.1, 1.25, 1.5, 2.0]
    .into_iter()
    .map(|scale| PickerOption {
      label: format!("{scale}"),
      args: monitor.scale_args(scale),
      persist: PersistChange::Scale(scale),
      prompt: None,
    })
    .collect()
}

/// Executes the `enabled_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn enabled_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  vec![
    PickerOption {
      label: tr(lang, "control_center.enable").into(),
      args: monitor.enabled_with_current_args(),
      persist: PersistChange::Disabled(false),
      prompt: None,
    },
    PickerOption {
      label: tr(lang, "control_center.disable").into(),
      args: monitor.disabled_args(),
      persist: PersistChange::Disabled(true),
      prompt: None,
    },
  ]
}

/// Executes the `orientation_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn orientation_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  (0..8)
    .map(|transform| PickerOption {
      label: transform_label(lang, transform),
      args: monitor.transform_args(transform),
      persist: PersistChange::None,
      prompt: None,
    })
    .collect()
}

/// Executes the `mirror_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn mirror_options(monitor: &Monitor, monitors: &[Monitor], lang: Lang) -> Vec<PickerOption> {
  let mut options = vec![PickerOption {
    label: tr(lang, "control_center.no_mirror").into(),
    args: monitor.mirror_args(""),
    persist: PersistChange::Mirror(String::new()),
    prompt: None,
  }];
  for other in monitors {
    if other.name != monitor.name && other.connected {
      options.push(PickerOption {
        label: other.name.clone(),
        args: monitor.mirror_args(&other.name),
        persist: PersistChange::Mirror(other.name.clone()),
        prompt: None,
      });
    }
  }
  options
}

/// Executes the `bitdepth_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn bitdepth_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  let _ = lang;
  [8_i32, 10]
    .into_iter()
    .map(|bitdepth| PickerOption {
      label: format!("{bitdepth} bpp"),
      args: monitor.bitdepth_args(bitdepth),
      persist: PersistChange::BitDepth(bitdepth),
      prompt: None,
    })
    .collect()
}

/// Executes the `vrr_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn vrr_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  [0_i32, 1, 2]
    .into_iter()
    .map(|value| PickerOption {
      label: vrr_label(lang, value),
      args: monitor.vrr_args(value),
      persist: PersistChange::Vrr(value),
      prompt: None,
    })
    .collect()
}

/// Executes the `hdr_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn hdr_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  [0_i32, 1]
    .into_iter()
    .map(|value| PickerOption {
      label: hdr_label(lang, value),
      args: monitor.hdr_args(value),
      persist: PersistChange::Hdr(value),
      prompt: None,
    })
    .collect()
}

/// Executes the `dpms_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn dpms_options(_monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  vec![
    PickerOption {
      label: tr(lang, "control_center.on_5ddab3").into(),
      args: String::new(),
      persist: PersistChange::Dpms(true),
      prompt: None,
    },
    PickerOption {
      label: tr(lang, "control_center.off_688c4e").into(),
      args: String::new(),
      persist: PersistChange::Dpms(false),
      prompt: None,
    },
  ]
}

/// Executes the `sdr_brightness_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn sdr_brightness_options(_lang: Lang, _monitor: &Monitor) -> Vec<PickerOption> {
  [0.25, 0.5, 0.75, 1.0]
    .into_iter()
    .map(|value| PickerOption {
      label: format!("{value:.2}"),
      args: String::new(),
      persist: PersistChange::None,
      prompt: None,
    })
    .collect()
}

/// Executes the `sdr_saturation_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn sdr_saturation_options(_lang: Lang, _monitor: &Monitor) -> Vec<PickerOption> {
  [0.5, 1.0, 1.5, 2.0]
    .into_iter()
    .map(|value| PickerOption {
      label: format!("{value:.2}"),
      args: String::new(),
      persist: PersistChange::None,
      prompt: None,
    })
    .collect()
}

/// Executes the `option_is_risky` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn option_is_risky(persist: &PersistChange) -> bool {
  matches!(
    persist,
    PersistChange::Mode(_)
      | PersistChange::Position(_, _)
      | PersistChange::Mirror(_)
      | PersistChange::Disabled(true)
      | PersistChange::BitDepth(_)
  )
}

/// Executes the `rule_from_persisted` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn rule_from_persisted(name: &str, persisted: &PersistedMonitor) -> String {
  let mode = persisted.mode.clone().unwrap_or_else(|| "auto".into());
  let position = persisted.position.clone().unwrap_or_else(|| "auto".into());
  let scale = persisted
    .scale
    .map(|value| value.to_string())
    .unwrap_or_else(|| "auto".into());
  let transform = persisted
    .transform
    .map(|value| value.to_string())
    .unwrap_or_else(|| "auto".into());
  let mut body = format!("{name}, {mode}, {position}, {scale}, transform, {transform}");
  if let Some(mirror) = &persisted.mirror {
    body.push_str(&format!(", mirror, {mirror}"));
  }
  if let Some(bitdepth) = persisted.bitdepth {
    body.push_str(&format!(", bitdepth, {bitdepth}"));
  }
  if let Some(vrr) = persisted.vrr {
    body.push_str(&format!(", vrr, {vrr}"));
  }
  if let Some(hdr) = persisted.hdr {
    body.push_str(&format!(", supports_hdr, {hdr}"));
  }
  if let Some(brightness) = persisted.sdr_brightness {
    body.push_str(&format!(", sdrbrightness, {brightness}"));
  }
  if let Some(saturation) = persisted.sdr_saturation {
    body.push_str(&format!(", sdrsaturation, {saturation}"));
  }
  if persisted.disabled == Some(true) {
    body = format!("{name}, disabled");
  }
  body
}

/// Executes the `fingerprint` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn fingerprint(monitors: &[Monitor]) -> Vec<String> {
  monitors
    .iter()
    .map(|monitor| {
      format!(
        "{}:{}x{}:{}x{}:{}:{}",
        monitor.name,
        monitor.width,
        monitor.height,
        monitor.x,
        monitor.y,
        monitor.disabled,
        monitor.dpms_status
      )
    })
    .collect()
}

/// Executes the `trimmed_rate` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn trimmed_rate(rate: f64) -> String {
  let rounded = (rate * 1000.0).round() / 1000.0;
  if rounded.fract() < 0.0005 {
    format!("{}", rounded as u64)
  } else {
    format!("{rounded:.3}")
  }
}

/// Executes the `rate_of` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn rate_of(mode: &str) -> Option<f64> {
  mode.rsplit('@').next()?.parse::<f64>().ok()
}

/// Converts input data into `parse_position` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_position(buffer: &str) -> Option<(i32, i32)> {
  let (x, y) = buffer.split_once(',')?;
  Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// Converts input data into `parse_number` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_number(buffer: &str) -> Option<f64> {
  buffer
    .trim()
    .parse::<f64>()
    .ok()
    .filter(|value| value.is_finite())
}

/// Executes the `prompt_allowed` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn prompt_allowed(goal: PromptGoal, character: char) -> bool {
  match goal {
    PromptGoal::Position(_) => {
      character.is_ascii_digit() || character == 'x' || character == ',' || character == '-'
    }
    PromptGoal::Scale(_) | PromptGoal::SdrBrightness(_) | PromptGoal::SdrSaturation(_) => {
      character.is_ascii_digit() || character == '.'
    }
    PromptGoal::ProfileCreate | PromptGoal::ProfileRename(_) => !character.is_control(),
  }
}

/// Executes the `setting_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn setting_label(lang: Lang, setting: MonitorSetting) -> String {
  match setting {
    MonitorSetting::Resolution => tr(lang, "control_center.resolution").into(),
    MonitorSetting::RefreshRate => tr(lang, "control_center.refresh_rate").into(),
    MonitorSetting::Scale => tr(lang, "control_center.scale").into(),
    MonitorSetting::Position => tr(lang, "control_center.position").into(),
    MonitorSetting::Primary => tr(lang, "control_center.primary_monitor").into(),
    MonitorSetting::Enabled => tr(lang, "control_center.monitor_enabled").into(),
    MonitorSetting::Mirror => tr(lang, "control_center.mirror").into(),
    MonitorSetting::BitDepth => tr(lang, "control_center.color_depth").into(),
    MonitorSetting::Vrr => tr(lang, "control_center.vrr").into(),
    MonitorSetting::Hdr => tr(lang, "control_center.hdr").into(),
    MonitorSetting::Dpms => "DPMS".into(),
    MonitorSetting::SdrBrightness => tr(lang, "control_center.sdr_brightness").into(),
    MonitorSetting::SdrSaturation => tr(lang, "control_center.sdr_saturation").into(),
    MonitorSetting::Workspaces => tr(lang, "control_center.workspaces").into(),
    MonitorSetting::Orientation => tr(lang, "control_center.orientation").into(),
  }
}

/// Executes the `transform_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn transform_label(lang: Lang, transform: i32) -> String {
  match transform {
    0 => tr(lang, "control_center.normal").into(),
    1 => tr(lang, "control_center.90").into(),
    2 => tr(lang, "control_center.180").into(),
    3 => tr(lang, "control_center.270").into(),
    4 => tr(lang, "control_center.flipped").into(),
    5 => tr(lang, "control_center.flipped_90").into(),
    6 => tr(lang, "control_center.flipped_180").into(),
    7 => tr(lang, "control_center.flipped_270").into(),
    _ => format!("{transform}"),
  }
}

/// Executes the `vrr_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn vrr_label(lang: Lang, vrr: i32) -> String {
  match vrr {
    0 => tr(lang, "control_center.disabled").into(),
    1 => tr(lang, "control_center.enabled").into(),
    2 => tr(lang, "control_center.fullscreen_only").into(),
    _ => format!("{vrr}"),
  }
}

/// Executes the `hdr_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn hdr_label(lang: Lang, hdr: i32) -> String {
  if hdr != 0 {
    tr(lang, "control_center.forced").into()
  } else {
    tr(lang, "control_center.auto_d2c2e5").into()
  }
}

/// Processes `on_off` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn on_off(lang: Lang, on: bool) -> String {
  if on {
    tr(lang, "control_center.yes").into()
  } else {
    tr(lang, "control_center.no").into()
  }
}

#[cfg(test)]
mod tests;
