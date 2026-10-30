//! Keys on the displays pages: moving, activating rows, confirmations and going back.

use super::*;

impl DisplaysApp {
  /// Opens `page` with the cursor on its first row.
  pub(super) fn go(&mut self, page: DisplayPage) {
    self.page = page;
    self.menu = MenuState::default();
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
          DisplayPage::Home | DisplayPage::NightLight | DisplayPage::Detail(_) => self.refresh(),
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
  pub(super) fn back(&mut self) -> bool {
    match self.page {
      DisplayPage::Home => return true,
      DisplayPage::Profiles | DisplayPage::NightLight | DisplayPage::Detail(_) => {
        self.go(DisplayPage::Home)
      }
      DisplayPage::Profile(_) => self.go(DisplayPage::Profiles),
      DisplayPage::Picker { monitor, .. } => self.go(DisplayPage::Detail(monitor)),
      DisplayPage::Prompt { goal } => self.restore_prompt(goal),
    }
    false
  }

  /// Runs the row `item`.
  pub(super) fn activate(&mut self, item: Item) {
    match item {
      Item::Monitor(index) => self.go(DisplayPage::Detail(index)),
      Item::Stale(offset) => self.go(DisplayPage::Detail(self.stale_start() + offset)),
      Item::NightLight => {
        self.go(DisplayPage::NightLight);
        self.refresh();
      }
      Item::NightToggle => self.night_action(vec!["toggle".into()]),
      Item::NightTemperature => self.open_prompt(PromptGoal::NightTemperature),
      Item::NightStart => self.open_prompt(PromptGoal::NightStart),
      Item::NightEnd => self.open_prompt(PromptGoal::NightEnd),
      Item::NightSchedule => self.night_schedule(
        !self.night_light.settings.schedule_enabled,
        self.night_light.settings.start.clone(),
        self.night_light.settings.end.clone(),
      ),
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
  pub(super) fn run_confirmation(&mut self, confirmation: Confirmation) {
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
}
