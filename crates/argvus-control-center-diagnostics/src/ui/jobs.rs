//! Background work of the Diagnostics page: loading data, running actions and polling their results.

use super::*;

impl DiagnosticsApp {
  pub fn reload(&mut self) {
    self.refresh()
  }

  pub fn refresh(&mut self) {
    if self.job.is_some() {
      return;
    }
    let cap = self.cap.clone();
    let lang = self.lang;
    self.job = Some(
      self
        .jobs
        .spawn(move |_| Ok::<_, String>(backend::evaluate(lang, &backend::collect(&cap), &cap))),
    );
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
          self.checks = checks;
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
