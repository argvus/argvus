//! Keys and text input on the Power page.

use super::*;

impl PowerApp {
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.picker.is_some() {
      self.handle_picker(key);
      return false;
    }
    // The action bar with the focus: `←/→` step through its buttons and Enter
    // runs the focused one. A list key gives the list the focus back and is
    // then handled as usual below.
    if self.pending.is_none() && self.action.is_none() && self.job.is_none() {
      match key {
        KeyCode::Tab | KeyCode::BackTab => {
          self.toggle_actions();
          return false;
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l')
          if self.actions.cursor().is_some() =>
        {
          self.move_action(matches!(key, KeyCode::Left | KeyCode::Char('h')));
          return false;
        }
        KeyCode::Enter | KeyCode::Char(' ') if self.actions.cursor().is_some() => {
          self.activate_button();
          return false;
        }
        KeyCode::Up
        | KeyCode::Char('k')
        | KeyCode::Down
        | KeyCode::Char('j')
        | KeyCode::Home
        | KeyCode::End
          if self.actions.cursor().is_some() =>
        {
          self.actions.focus_list();
        }
        _ => {}
      }
    }
    if key == KeyCode::Esc && self.pending.is_none() {
      if self.page == PowerPage::Home {
        return true;
      }
      self.page = PowerPage::Home;
      self.selected = 0;
      self.actions.focus_list();
      self.refresh();
      return false;
    }
    if self.job.is_some() || self.action.is_some() {
      return false;
    }
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          self.confirmation = ConfirmationState::default();
          let pending = self.pending.take().unwrap();
          self.status = Some(StatusMessage {
            kind: StatusKind::Info,
            text: tr(self.lang, "control_center.running_3f0393").into(),
          });
          self.action = Some(self.manager.spawn(move |_| match pending {
            Pending::Suspend => {
              backend::suspend_now()?;
              Ok(JobData::Action(
                tr(
                  Lang::for_locale("en-US"),
                  "control_center.suspension_started",
                )
                .into(),
              ))
            }
            Pending::Hibernate => {
              backend::hibernate_now()?;
              Ok(JobData::Action(
                tr(
                  Lang::for_locale("en-US"),
                  "control_center.hibernation_started",
                )
                .into(),
              ))
            }
          }));
        }
        ConfirmationOutcome::Cancelled => {
          self.pending = None;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    match key {
      KeyCode::Tab | KeyCode::BackTab => self.toggle_actions(),
      // On the list, `←/→` reach the action bar, as `Tab` does.
      KeyCode::Left | KeyCode::Char('h') => self.move_action(true),
      KeyCode::Right | KeyCode::Char('l') => self.move_action(false),
      KeyCode::Char('r') => self.refresh(),
      KeyCode::Up | KeyCode::Char('k') => {
        self.selected = self.selected.saturating_sub(1);
      }
      KeyCode::Down | KeyCode::Char('j') => {
        self.selected = self.selected.saturating_add(1);
      }
      KeyCode::Home => self.selected = 0,
      KeyCode::End => self.selected = self.row_count().saturating_sub(1),
      KeyCode::Enter => self.open_row_picker(),
      _ => {}
    }
    self.selected = self.selected.min(self.row_count().saturating_sub(1));
    false
  }

  pub(super) fn handle_picker(&mut self, key: KeyCode) {
    let mut apply: Option<(PickerTarget, usize)> = None;
    let mut close = false;
    {
      let Some(picker) = &mut self.picker else {
        return;
      };
      let count = picker_target_len(picker.target);
      match key {
        KeyCode::Up | KeyCode::Char('k') => {
          picker.selected = picker.selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
          picker.selected = (picker.selected + 1).min(count.saturating_sub(1));
        }
        KeyCode::Home => picker.selected = 0,
        KeyCode::End => picker.selected = count.saturating_sub(1),
        KeyCode::Enter => {
          apply = Some((picker.target, picker.selected));
          close = true;
        }
        KeyCode::Esc | KeyCode::Left => close = true,
        _ => {}
      }
    }
    if close {
      self.picker = None;
    }
    if let Some((target, selected)) = apply {
      self.picker_apply(target, selected);
    }
  }

  /// `←/→` on the action bar: steps through its buttons.
  pub(super) fn move_action(&mut self, backwards: bool) {
    let enabled = self.enabled_actions();
    self.actions.move_by(backwards, &enabled);
  }
}
