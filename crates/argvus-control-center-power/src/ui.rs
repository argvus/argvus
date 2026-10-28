//! Terminal UI rendering and interaction of the Power page.
use crate::{
  backend,
  model::{LidContext, PowerBehavior, PowerButtonBehavior, PowerPage, PowerState},
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::action_buttons::{ActionButton, ActionFocus, draw_aligned_transparent};
use argvus_tui::chrome::centered;
use argvus_tui::components::{
  ConfirmationDialog, ConfirmationOutcome, ConfirmationState, StatusKind, StatusMessage,
  draw_confirmation,
};
use argvus_tui::icons;
use argvus_tui::page::{list, shell, status};
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

mod input;
mod jobs;
mod render;
mod rows;

use rows::{behavior_label, idle_label};

const LID_BEHAVIORS: [PowerBehavior; 5] = [
  PowerBehavior::Suspend,
  PowerBehavior::Hibernate,
  PowerBehavior::Lock,
  PowerBehavior::Ignore,
  PowerBehavior::Poweroff,
];

const BUTTON_BEHAVIORS: [PowerButtonBehavior; 4] = [
  PowerButtonBehavior::Poweroff,
  PowerButtonBehavior::Suspend,
  PowerButtonBehavior::Lock,
  PowerButtonBehavior::Ignore,
];

const IDLE_OPTIONS: [u32; 7] = [5, 10, 15, 30, 60, 120, 0];

const LOCK_OPTIONS: [u32; 6] = [1, 5, 10, 15, 30, 0];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickerTarget {
  LidBattery,
  LidAc,
  PowerButton,
  ScreenOff,
  Lock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Picker {
  target: PickerTarget,
  selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PowerButton {
  Suspend,
  Hibernate,
  Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Pending {
  Suspend,
  Hibernate,
}

#[derive(Debug, Clone)]
enum JobData {
  State(PowerState),
  Action(String),
}

pub struct PowerApp {
  pub page: PowerPage,
  selected: usize,
  actions: ActionFocus,
  state: Option<PowerState>,
  job: Option<JobHandle<JobData>>,
  action: Option<JobHandle<JobData>>,
  pending: Option<Pending>,
  confirmation: ConfirmationState,
  picker: Option<Picker>,
  manager: JobManager,
  pub lang: Lang,
  pub theme: Theme,
  pub status: Option<StatusMessage>,
}

impl PowerApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme) -> Self {
    let mut app = Self {
      page: PowerPage::Home,
      selected: 0,
      actions: ActionFocus::default(),
      state: None,
      job: None,
      action: None,
      pending: None,
      confirmation: ConfirmationState::default(),
      picker: None,
      manager: JobManager::default(),
      lang,
      theme,
      status: None,
    };
    app.refresh();
    app
  }

  fn open_row_picker(&mut self) {
    let Some(state) = &self.state else {
      return;
    };
    let is_laptop = state.is_laptop;
    let keep_awake = state.keep_awake;

    let lid_battery_idx = 0;
    let lid_ac_idx = 1;
    let power_btn_idx = 2;
    let screen_off_idx = if is_laptop { 3 } else { 0 };
    let lock_idx = if is_laptop { 4 } else { 1 };
    let keep_awake_idx = if is_laptop { 5 } else { 2 };

    let target = match self.selected {
      i if i == lid_battery_idx && is_laptop => PickerTarget::LidBattery,
      i if i == lid_ac_idx && is_laptop => PickerTarget::LidAc,
      i if i == power_btn_idx && is_laptop => PickerTarget::PowerButton,
      i if i == screen_off_idx => PickerTarget::ScreenOff,
      i if i == lock_idx => PickerTarget::Lock,
      i if i == keep_awake_idx => {
        self.apply_keep_awake(!keep_awake);
        return;
      }
      _ => return,
    };
    self.picker = Some(Picker {
      target,
      selected: picker_default(target, state),
    });
  }

  fn picker_apply(&mut self, target: PickerTarget, selected: usize) {
    match target {
      PickerTarget::LidBattery => {
        let Some(&behavior) = LID_BEHAVIORS.get(selected) else {
          return;
        };
        if let Some(state) = &mut self.state {
          state.lid[0] = behavior;
        }
        self.apply_lid(LidContext::Battery, behavior);
      }
      PickerTarget::LidAc => {
        let Some(&behavior) = LID_BEHAVIORS.get(selected) else {
          return;
        };
        if let Some(state) = &mut self.state {
          state.lid[1] = behavior;
        }
        self.apply_lid(LidContext::Ac, behavior);
      }
      PickerTarget::PowerButton => {
        let Some(&behavior) = BUTTON_BEHAVIORS.get(selected) else {
          return;
        };
        if let Some(state) = &mut self.state {
          state.power_button = behavior;
        }
        self.apply_button(behavior);
      }
      PickerTarget::ScreenOff => {
        let Some(&minutes) = IDLE_OPTIONS.get(selected) else {
          return;
        };
        if let Some(state) = &mut self.state {
          state.screen_off_minutes = (minutes > 0).then_some(minutes);
        }
        self.apply_idle(minutes);
      }
      PickerTarget::Lock => {
        let Some(&minutes) = LOCK_OPTIONS.get(selected) else {
          return;
        };
        if let Some(state) = &mut self.state {
          state.lock_minutes = (minutes > 0).then_some(minutes);
        }
        self.apply_lock(minutes);
      }
    }
  }

  fn apply_lid(&mut self, context: LidContext, behavior: PowerBehavior) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_6c556f").into(),
    });
    let description = format!(
      "{} · {}",
      tr(self.lang, "control_center.lid_close"),
      behavior_label(self.lang, behavior.value()),
    );
    self.action = Some(self.manager.spawn(move |_| {
      backend::set_lid(context, behavior)?;
      Ok(JobData::Action(description))
    }));
  }

  fn apply_button(&mut self, behavior: PowerButtonBehavior) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_6c556f").into(),
    });
    let description = format!(
      "{} · {}",
      tr(self.lang, "control_center.power_button"),
      behavior_label(self.lang, behavior.value()),
    );
    self.action = Some(self.manager.spawn(move |_| {
      backend::set_power_button(behavior)?;
      Ok(JobData::Action(description))
    }));
  }

  fn apply_idle(&mut self, minutes: u32) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_6c556f").into(),
    });
    let description = format!(
      "{} · {}",
      tr(self.lang, "control_center.screen_off_after"),
      idle_label(minutes),
    );
    self.action = Some(self.manager.spawn(move |_| {
      backend::apply_idle(minutes)?;
      Ok(JobData::Action(description))
    }));
  }

  fn apply_lock(&mut self, minutes: u32) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_6c556f").into(),
    });
    let description = format!(
      "{} · {}",
      tr(self.lang, "control_center.lock_screen_after"),
      idle_label(minutes),
    );
    self.action = Some(self.manager.spawn(move |_| {
      backend::apply_lock(minutes)?;
      Ok(JobData::Action(description))
    }));
  }

  fn apply_keep_awake(&mut self, enabled: bool) {
    if let Some(state) = &mut self.state {
      state.keep_awake = enabled;
    }
    let description = tr(
      self.lang,
      if enabled {
        "control_center.keep_awake_enabled"
      } else {
        "control_center.keep_awake_disabled"
      },
    );
    self.action = Some(self.manager.spawn(move |_| {
      backend::set_keep_awake(enabled)?;
      Ok(JobData::Action(description.into()))
    }));
  }

  fn capabilities_line(&self) -> String {
    let Some(state) = &self.state else {
      return String::new();
    };
    if !state.is_laptop {
      return String::new();
    }
    let suspend_icon = if state.can_suspend { "✓" } else { "✕" };
    let hibernate_icon = if state.can_hibernate { "✓" } else { "✕" };
    format!(
      " {}: {}   {}: {}",
      tr(self.lang, "control_center.suspend"),
      suspend_icon,
      tr(self.lang, "control_center.hibernate"),
      hibernate_icon,
    )
  }

  fn buttons(&self) -> Vec<(PowerButton, ActionButton)> {
    let state = self.state.as_ref();
    let mut buttons = Vec::new();
    if state.is_none_or(|s| s.can_suspend) {
      buttons.push((
        PowerButton::Suspend,
        ActionButton::primary(
          action_icon(icons::SLEEP),
          tr(self.lang, "control_center.suspend_now"),
          "",
        ),
      ));
    }
    if state.is_none_or(|s| s.can_hibernate) {
      buttons.push((
        PowerButton::Hibernate,
        ActionButton::danger(
          action_icon(icons::HIBERNATE),
          tr(self.lang, "control_center.hibernate_now"),
          "",
        ),
      ));
    }
    buttons.push((
      PowerButton::Refresh,
      ActionButton::secondary(
        action_icon(icons::REFRESH),
        tr(self.lang, "control_center.refresh"),
        "",
      ),
    ));
    buttons
  }

  /// Indexes of the action buttons; every one of them is always enabled.
  fn enabled_actions(&self) -> Vec<usize> {
    (0..self.buttons().len()).collect()
  }

  /// `Tab`: moves the focus between the list and the action bar.
  fn toggle_actions(&mut self) {
    let enabled = self.enabled_actions();
    self.actions.toggle(&enabled);
  }

  /// Enter/Space on the focused button: suspend and hibernate ask first,
  /// refresh runs at once.
  fn activate_button(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let Some(index) = self.actions.cursor() else {
      return;
    };
    let actions = self.buttons();
    let Some((power_button, _)) = actions.get(index) else {
      return;
    };
    match power_button {
      PowerButton::Suspend => self.pending = Some(Pending::Suspend),
      PowerButton::Hibernate => self.pending = Some(Pending::Hibernate),
      PowerButton::Refresh => self.refresh(),
    }
  }
}

/// Icon of an action button, left out when icons are disabled.
fn action_icon(glyph: &'static str) -> &'static str {
  if AppConfig::icons_enabled() {
    glyph
  } else {
    ""
  }
}

fn picker_target_len(target: PickerTarget) -> usize {
  match target {
    PickerTarget::LidBattery | PickerTarget::LidAc => LID_BEHAVIORS.len(),
    PickerTarget::PowerButton => BUTTON_BEHAVIORS.len(),
    PickerTarget::ScreenOff => IDLE_OPTIONS.len(),
    PickerTarget::Lock => LOCK_OPTIONS.len(),
  }
}

/// The current value's position within the picker option list.
fn picker_default(target: PickerTarget, state: &PowerState) -> usize {
  match target {
    PickerTarget::LidBattery => LID_BEHAVIORS
      .iter()
      .position(|&value| value == state.lid[0])
      .unwrap_or(0),
    PickerTarget::LidAc => LID_BEHAVIORS
      .iter()
      .position(|&value| value == state.lid[1])
      .unwrap_or(0),
    PickerTarget::PowerButton => BUTTON_BEHAVIORS
      .iter()
      .position(|&value| value == state.power_button)
      .unwrap_or(0),
    PickerTarget::ScreenOff => {
      let current = state.screen_off_minutes.unwrap_or(0);
      IDLE_OPTIONS
        .iter()
        .position(|&value| value == current)
        .unwrap_or_else(|| IDLE_OPTIONS.len().saturating_sub(1))
    }
    PickerTarget::Lock => {
      let current = state.lock_minutes.unwrap_or(0);
      LOCK_OPTIONS
        .iter()
        .position(|&value| value == current)
        .unwrap_or_else(|| LOCK_OPTIONS.len().saturating_sub(1))
    }
  }
}

fn picker_options(target: PickerTarget, lang: Lang) -> Vec<String> {
  match target {
    PickerTarget::LidBattery | PickerTarget::LidAc => LID_BEHAVIORS
      .iter()
      .map(|&behavior| behavior_label(lang, behavior.value()))
      .collect(),
    PickerTarget::PowerButton => BUTTON_BEHAVIORS
      .iter()
      .map(|&behavior| behavior_label(lang, behavior.value()))
      .collect(),
    PickerTarget::ScreenOff => IDLE_OPTIONS
      .iter()
      .map(|&minutes| idle_label(minutes))
      .collect(),
    PickerTarget::Lock => LOCK_OPTIONS
      .iter()
      .map(|&minutes| idle_label(minutes))
      .collect(),
  }
}

fn picker_title(lang: Lang, target: PickerTarget) -> String {
  match target {
    PickerTarget::LidBattery => tr(lang, "control_center.lid_close_battery").into(),
    PickerTarget::LidAc => tr(lang, "control_center.lid_close_ac").into(),
    PickerTarget::PowerButton => tr(lang, "control_center.power_button").into(),
    PickerTarget::ScreenOff => tr(lang, "control_center.screen_off_after").into(),
    PickerTarget::Lock => tr(lang, "control_center.lock_screen_after").into(),
  }
}

#[cfg(test)]
mod tests;
