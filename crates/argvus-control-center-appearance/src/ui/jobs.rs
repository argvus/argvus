//! Loading the appearance data and applying changes in the background, and handling their results.

use super::*;

impl AppearanceApp {
  pub fn reload(&mut self) {
    self.ensure_page();
  }

  pub(super) fn refresh(&mut self) {
    if Self::shows_control_panel_cards(self.page) && self.availability.is_none() {
      self.availability = Some(
        self
          .manager
          .spawn(|_| Ok(JobData::Availability(backend::control_panel_availability()))),
      );
    }
    if self.job.is_none() && self.action.is_none() {
      self.status_loading = true;
      let page = self.page;
      self.job_page = Some(page);
      let state = self.state.clone();
      self.job = Some(self.manager.spawn(move |_| {
        if page == AppearancePage::ThemeImport {
          Ok(JobData::Archives(backend::list_import_archives()))
        } else {
          Ok(JobData::Loaded(
            page,
            Box::new(backend::load_page(page, state)),
          ))
        }
      }));
    }
  }

  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(availability) = &self.availability
      && let JobState::Finished(result) = availability.try_state()
    {
      self.availability = None;
      // A failed probe keeps the previous availability.
      if let Ok(JobData::Availability(cards)) = result {
        self.apply_card_availability(&cards);
        self.known_availability = Some(cards);
      }
      changed = true;
    }
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      self.job_page = None;
      match result {
        Ok(JobData::Loaded(page, state)) => self.on_loaded(page, *state),
        Ok(JobData::Archives(paths)) => {
          self.import_archives = paths;
          self.status_loading = false;
        }
        Err(error) => {
          self.status_loading = false;
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: format!("{} {error}", tr(self.lang, "control_center.error")),
          });
        }
        Ok(JobData::Action(_)) => {}
        Ok(JobData::ImportReady { .. }) => {}
        Ok(JobData::Availability(_)) => {}
      }
      changed = true;
    }
    if let Some(action) = &self.action
      && let JobState::Finished(result) = action.try_state()
    {
      self.action = None;
      // The action already ran the external scripts that regenerate the theme
      // files, so this is the first safe point to signal a palette reload.
      self.theme_dirty = true;
      if self.reload_requested {
        self.reload_requested = false;
        self.refreshed.clear();
        self.refresh();
      }
      match result {
        Ok(JobData::Action(text)) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: text.clone(),
          })
        }
        Ok(JobData::ImportReady {
          path,
          name,
          duplicate,
        }) => {
          self.pending_import = Some(path);
          self.pending_import_name = Some(name);
          if duplicate {
            self.status = None;
            self.confirm = Some((Confirmation::ReplaceImport, ConfirmState::new()));
          } else {
            self.start_pending_import();
            self.go(AppearancePage::Themes);
          }
        }
        Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: format!("{} {error}", tr(self.lang, "control_center.error")),
          })
        }
        Ok(_) => {}
      }
      changed = true;
    }
    changed
  }

  /// Stores a freshly loaded state. The page's draft is rebuilt from it only
  /// while the draft has no unapplied changes, so a reload (`r`, or the
  /// automatic one when a page is entered) never drops the user's edits;
  /// after `Apply` the draft matches the applied values and is rebuilt.
  pub(super) fn on_loaded(&mut self, page: AppearancePage, state: AppearanceState) {
    self.refreshed.retain(|(old, _)| *old != page);
    self.refreshed.push((page, std::time::Instant::now()));
    let keep_draft = self.surface_draft_has_changes();
    self.state = state;
    if let Some(cards) = self.known_availability.clone() {
      self.apply_card_availability(&cards);
    }
    self.loaded = true;
    self.status_loading = false;
    if !self.state.theme_notices.is_empty() {
      self.status = Some(StatusMessage {
        kind: StatusKind::Info,
        text: format!(
          "{}: {}",
          tr(self.lang, "control_center.theme_profiles_converted"),
          self.state.theme_notices.join(", ")
        ),
      });
    }
    if page == AppearancePage::ControlPanel && self.page == page {
      self.control_panel_draft = Some(self.state.control_panel_cards.clone());
    }
    if let Some(surface) = Self::surface_for_page(page)
      && self.page == page
      && !keep_draft
    {
      self.surface_draft = Some(self.make_surface_draft(surface));
    }
    if self.page != page {
      self.refresh();
    }
  }

  /// Whether `page` lists the Control Panel cards, whose availability
  /// depends on a hardware probe.
  pub(super) fn shows_control_panel_cards(page: AppearancePage) -> bool {
    matches!(
      page,
      AppearancePage::ControlPanel
        | AppearancePage::SurfaceSection {
          surface: EffectSurface::ControlPanel,
          ..
        }
    )
  }

  /// Stores the probed card availability in the state and in both Control
  /// Panel drafts. Only availability changes, never the enabled flags, so a
  /// draft without edits still matches the state afterwards.
  pub(super) fn apply_card_availability(&mut self, cards: &[(ControlPanelCard, bool)]) {
    for &(card, available) in cards {
      self
        .state
        .control_panel_cards
        .set_available(card, available);
      if let Some(draft) = self.surface_draft.as_mut() {
        draft.control_panel_cards.set_available(card, available);
      }
      if let Some(draft) = self.control_panel_draft.as_mut() {
        draft.set_available(card, available);
      }
    }
  }

  pub(super) fn start_pending_import(&mut self) {
    let Some(path) = self.pending_import.take() else {
      return;
    };
    self.apply(
      tr(self.lang, "control_center.theme_profile_imported").into(),
      move || backend::import_theme(&path).map(|_| ()),
    );
  }

  pub(super) fn apply(
    &mut self,
    message: String,
    task: impl FnOnce() -> Result<(), String> + Send + 'static,
  ) {
    if self.action.is_some() {
      return;
    }
    self.reload_requested = true;
    self.action = Some(self.manager.spawn(move |_| {
      task()?;
      Ok(JobData::Action(message))
    }));
  }

  pub(super) fn apply_result(
    &mut self,
    task: impl FnOnce() -> Result<String, String> + Send + 'static,
  ) {
    if self.action.is_some() {
      return;
    }
    self.reload_requested = true;
    self.action = Some(self.manager.spawn(move |_| {
      let text = task()?;
      Ok(JobData::Action(text))
    }));
  }
}
