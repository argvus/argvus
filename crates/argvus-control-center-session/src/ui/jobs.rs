//! Background work of the Session page: loading data, running actions and polling their results.

use super::*;

impl SessionApp {
  pub fn reload(&mut self) {
    self.refresh();
  }

  pub(super) fn refresh(&mut self) {
    if self.job.is_some() {
      return;
    }
    let page = self.page;
    let filter = self.log_filter.clone();
    let home = self.page == SessionPage::Home;
    self.job = Some(self.manager.spawn(move |_| match page {
      SessionPage::Components => Ok(JobData::Components(backend::list_components())),
      SessionPage::Autostart => Ok(JobData::Autostart(backend::autostart_entries())),
      SessionPage::Diagnostics => {
        let components = backend::list_components();
        Ok(JobData::Diagnostics(backend::diagnostics(&components)))
      }
      SessionPage::Logs | SessionPage::LogDetail(_) => {
        backend::logs(filter.as_deref(), 200).map(JobData::Logs)
      }
      _ if home => {
        let (components, autostart) =
          rayon::join(backend::list_components, backend::autostart_entries);
        Ok(JobData::HomeSummary(components, autostart))
      }
      _ => Ok(JobData::Components(backend::list_components())),
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_session").into(),
    });
  }

  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      match result {
        Ok(JobData::HomeSummary(components, autostart)) => {
          self.components = merge_component_order(components);
          self.autostart = autostart;
          self.selected = self.selected.min(self.selection_len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Components(components)) => {
          self.components = merge_component_order(components);
          self.selected = self.selected.min(self.selection_len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Autostart(entries)) => {
          self.autostart = entries;
          self.selected = self.selected.min(self.selection_len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Diagnostics(entries)) => {
          self.diagnostics = entries;
          self.selected = self.selected.min(self.selection_len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Logs(entries)) => {
          self.logs = entries;
          self.selected = self.selected.min(self.logs.len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Action(text)) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text,
          });
          self.refresh();
        }
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          })
        }
      };
      changed = true;
    }
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
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
    changed
  }
}
