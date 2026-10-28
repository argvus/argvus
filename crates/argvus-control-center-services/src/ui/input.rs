//! Keys and text input on the Services page.

use super::jobs::run_action;
use super::*;

impl ServicesApp {
  /// Whether typed characters currently go to the unit or log search, so
  /// `q`/`?` must not act as the global quit/help keys. Mirrors the
  /// precedence of [`Self::handle`]: running jobs and the confirmation sit
  /// above the search and do not take text.
  pub fn captures_text(&self) -> bool {
    self.searching && self.job.is_none() && self.action.is_none() && self.pending.is_none()
  }

  pub fn handle(&mut self, key: KeyCode) -> bool {
    if matches!(key, KeyCode::Esc | KeyCode::Left) && self.pending.is_none() && !self.searching {
      if self.page == ServicePage::Home {
        return true;
      }
      if self.is_list_page() {
        // The filter is per visit: leaving a list clears it.
        self.filter = UnitFilter::All;
      }
      self.page = if self.page == ServicePage::Detail {
        self.detail_parent
      } else if matches!(self.page, ServicePage::LogDetail(_)) {
        ServicePage::Logs
      } else {
        ServicePage::Home
      };
      self.selected = 0;
      self.menu = MenuState::default();
      if self.page == ServicePage::Home {
        self.refresh();
      }
      return false;
    }
    if self.job.is_some() || self.action.is_some() {
      return false;
    }
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          self.confirmation = ConfirmationState::default();
          let Pending::Action(a, u) = self.pending.take().unwrap();
          let user = self.page == ServicePage::User
            || self.detail_parent == ServicePage::User
            || self
              .units
              .iter()
              .any(|unit| unit.name == u && unit.scope == "user");
          self.status = Some(StatusMessage {
            kind: StatusKind::Info,
            text: tr(self.lang, "control_center.applying_action").into(),
          });
          self.action = Some(self.manager.spawn(move |_| run_action(user, &a, &u)));
        }
        ConfirmationOutcome::Cancelled => {
          self.pending = None;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    if self.searching && self.is_list_page() && matches!(key, KeyCode::Up | KeyCode::Down) {
      // Arrows leave the search field and move the cursor of the list.
      self.searching = false;
    }
    if self.searching {
      let logs = self.page == ServicePage::Logs;
      match key {
        KeyCode::Esc => self.searching = false,
        KeyCode::Backspace => {
          if logs {
            self.log_search.pop();
          } else {
            self.search.pop();
          }
          self.selected = 0
        }
        KeyCode::Enter => self.searching = false,
        KeyCode::Char(c) => {
          if logs {
            self.log_search.push(c);
          } else {
            self.search.push(c);
          }
        }
        KeyCode::Down => self.selected += 1,
        KeyCode::Up => self.selected = self.selected.saturating_sub(1),
        _ => {}
      }
      let count = if logs {
        self.filtered_logs().len()
      } else {
        self.filtered().len()
      };
      self.selected = self.selected.min(count.saturating_sub(1));
      return false;
    }
    if self.is_list_page() || self.page == ServicePage::Detail {
      match key {
        KeyCode::Char('r') => self.refresh(),
        KeyCode::Char('/') if self.is_list_page() => self.searching = true,
        KeyCode::Char('f') if self.is_list_page() => self.cycle_filter(),
        _ => {
          let rows = self.menu_rows();
          match self.menu.handle(key, &rows, 8) {
            MenuEvent::Activate(Item::Unit(index)) => {
              self.selected = index;
              self.open();
            }
            MenuEvent::Activate(Item::DetailAction(action)) => self.run_detail(action),
            _ => {}
          }
        }
      }
      return false;
    }
    match key {
      KeyCode::Char('r') => self.refresh(),
      KeyCode::Char('/')
        if matches!(
          self.page,
          ServicePage::System | ServicePage::User | ServicePage::Failed | ServicePage::Logs
        ) =>
      {
        self.searching = true
      }
      // The Logs filters: boot, unit and priority, as on the filter line.
      KeyCode::Char('b') if self.page == ServicePage::Logs => self.toggle_previous_boot(),
      KeyCode::Char('u') if self.page == ServicePage::Logs => self.cycle_log_unit(),
      KeyCode::Char('p') if self.page == ServicePage::Logs => self.cycle_priority(),
      KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
      KeyCode::Down | KeyCode::Char('j') => self.selected = self.selected.saturating_add(1),
      KeyCode::Home => self.selected = 0,
      KeyCode::End => self.selected = self.selection_len().saturating_sub(1),
      KeyCode::PageUp => self.selected = self.selected.saturating_sub(8),
      KeyCode::PageDown => {
        self.selected = self
          .selected
          .saturating_add(8)
          .min(self.selection_len().saturating_sub(1))
      }
      KeyCode::Enter | KeyCode::Right => self.open(),
      _ => {}
    }
    let count = match self.page {
      ServicePage::Home => 4,
      ServicePage::System | ServicePage::User | ServicePage::Failed => self.filtered().len(),
      ServicePage::Logs => self.filtered_logs().len(),
      ServicePage::Detail => self.detail_actions().len(),
      _ => 1,
    };
    self.selected = self.selected.min(count.saturating_sub(1));
    false
  }

  pub(super) fn selection_len(&self) -> usize {
    match self.page {
      ServicePage::Home => 4,
      ServicePage::System | ServicePage::User | ServicePage::Failed => self.filtered().len(),
      ServicePage::Logs => self.logs.len(),
      ServicePage::Detail => self.detail_actions().len(),
      ServicePage::LogDetail(_) => 1,
    }
  }
}
