//! Background work of the Diagnostics page: loading data, running actions and polling their results.

use super::*;

impl DiagnosticsApp {
  pub fn reload(&mut self) {
    self.refresh()
  }

  pub fn refresh(&mut self) {
    self.start_refresh(false);
  }

  pub fn refresh_forced(&mut self) {
    self.start_refresh(true);
  }

  fn start_refresh(&mut self, force: bool) {
    if self.job.is_some() {
      return;
    }
    let cap = self.cap.clone();
    let cache = self.cache.clone();
    let lang = self.lang;
    self.job = Some(self.jobs.spawn(move |_| {
      Ok::<_, String>(backend::evaluate(
        lang,
        &backend::collect(&cap, &cache, force),
        &cap,
      ))
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.running_checks").into(),
    });
  }

  pub fn poll(&mut self) -> bool {
    let Some(job) = &self.job else {
      return false;
    };
    if let JobState::Finished(result) = job.try_state() {
      self.job = None;
      match result {
        Ok(checks) => {
          let selected_id = match self.page {
            DiagnosticPage::Detail(index) => self.checks.get(index).map(|check| check.id.clone()),
            DiagnosticPage::Home | DiagnosticPage::Summary => None,
            _ => self
              .category_checks()
              .nth(self.selected.index)
              .map(|(_, check)| check.id.clone()),
          };
          self.checks = checks;
          if let Some(id) = selected_id {
            if matches!(self.page, DiagnosticPage::Detail(_)) {
              self.page = self
                .checks
                .iter()
                .position(|check| check.id == id)
                .map(DiagnosticPage::Detail)
                .unwrap_or(DiagnosticPage::Home);
            } else {
              let position = self.category_checks().position(|(_, check)| check.id == id);
              if let Some(index) = position {
                self.selected.index = index;
              }
            }
          }
          self.last_run = Some(std::time::SystemTime::now());
          self.selected.normalize(self.len());
          self.status = Some(completion_status(self.lang, &self.checks));
        }
        Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: error,
          });
        }
      }
      true
    } else {
      false
    }
  }

  pub fn busy(&self) -> bool {
    self.job.is_some()
  }
}
