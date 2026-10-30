//! Background work: reading the monitors, applying a layout with its revert countdown and saving the state.

use super::profiles::fingerprint;
use super::*;

impl DisplaysApp {
  /// Reloads the snapshot after a route opened a page; the cursor starts
  /// on the page's first row.
  pub fn reload(&mut self) {
    self.menu = MenuState::default();
    self.refresh();
  }

  pub(super) fn refresh(&mut self) {
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
        night_light: backend::night_light_status(),
      })
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(
        lang,
        if self.page == DisplayPage::NightLight {
          "control_center.night_light.loading"
        } else {
          "control_center.loading_monitors"
        },
      )
      .into(),
    });
  }

  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    let mut night_action_finished = false;
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
          night_light,
        }) => {
          self.monitors = monitors;
          self.config = config;
          self.state = state;
          self.version = version;
          self.night_light = night_light;
          self.status = None;
        }
        Ok(JobData::Hotplug(_)) | Ok(JobData::Action(_)) | Ok(JobData::NightLight(..)) => {}
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
        Ok(JobData::NightLight(snapshot, error)) => {
          night_action_finished = true;
          self.night_light = snapshot;
          self.night_error = error;
          self.status = Some(StatusMessage {
            kind: if self.night_error.is_some() {
              StatusKind::Error
            } else {
              StatusKind::Success
            },
            text: tr(
              self.lang,
              if self.night_error.is_some() {
                "control_center.night_light.apply_failed"
              } else {
                "control_center.applied"
              },
            )
            .into(),
          });
        }
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
      if let Ok(JobData::NightLight(snapshot, _)) = &result
        && self.action.is_none()
        && !night_action_finished
      {
        self.night_light = snapshot.clone();
        changed = true;
      }
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
      self.hotplug = Some(if self.page == DisplayPage::NightLight {
        self
          .manager
          .spawn(|_| Ok(JobData::NightLight(backend::night_light_status(), None)))
      } else {
        self.manager.spawn(|_| {
          let monitors = backend::list_monitors().unwrap_or_default();
          Ok(JobData::Hotplug(monitors))
        })
      });
      self.last_hotplug = Instant::now();
    }
    changed
  }

  /// Arms the revert countdown: without an answer before
  /// `now + REVERT_SECONDS`, `previous_config` is applied again.
  pub(super) fn arm_revert(
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
  pub(super) fn answer_revert(&mut self, key: KeyCode) -> RevertAnswer {
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
  pub(super) fn take_expired_revert(&mut self, now: Instant) -> Option<RevertState> {
    self.revert.take_if(|revert| now >= revert.deadline)
  }

  /// Applies the configuration saved before the change ("Reverted <name>").
  pub(super) fn revert_to(&mut self, revert: RevertState) {
    let message = revert_message(self.lang, &revert);
    self.spawn_revert(revert.previous_config, message);
  }

  pub(super) fn apply_sdr<F: FnOnce(&mut PersistedMonitor)>(&mut self, index: usize, mutate: F) {
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

  pub(super) fn spawn_save_state(&mut self, state: crate::model::DisplayState, message: String) {
    self.action = Some(self.manager.spawn(move |_| {
      backend::save_state(&state)?;
      Ok(JobData::Action(message))
    }));
  }

  pub(super) fn apply_persist(config: &mut PersistedConfig, name: &str, persist: &PersistChange) {
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

  pub(super) fn monitor_ctx(&self) -> Option<(usize, MonitorSetting)> {
    match self.page {
      DisplayPage::Picker { monitor, setting } => Some((monitor, setting)),
      DisplayPage::Detail(_)
      | DisplayPage::Home
      | DisplayPage::NightLight
      | DisplayPage::Profiles
      | DisplayPage::Profile(_)
      | DisplayPage::Prompt { .. } => None,
    }
  }

  pub(super) fn spawn_revert(&mut self, config: PersistedConfig, message: String) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.reverting").into(),
    });
    self.action = Some(self.manager.spawn(move |_| {
      backend::apply_all(&config)?;
      Ok(JobData::Action(message))
    }));
  }

  pub(super) fn spawn_apply_with_config(&mut self, config: PersistedConfig, message: String) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_to_monitor").into(),
    });
    self.action = Some(self.manager.spawn(move |_| {
      backend::apply_all(&config)?;
      Ok(JobData::Action(message))
    }));
  }

  pub(super) fn reload_state(&mut self) {
    self.state = backend::load_state();
  }
}

/// Status shown after an automatic or chosen revert ("Reverted <name>").
pub(super) fn revert_message(lang: Lang, revert: &RevertState) -> String {
  format!(
    "{} {}",
    tr(lang, "control_center.reverted"),
    revert.previous.name
  )
}
