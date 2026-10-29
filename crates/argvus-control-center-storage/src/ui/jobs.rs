//! Background work of the Storage page: loading data, running actions and polling their results.

use super::*;

impl StorageApp {
  pub fn reload(&mut self) {
    self.refresh();
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
    self.job = Some(
      self
        .jobs
        .spawn(move |_| cache.collect(&cap, force).map_err(|e| e.to_string())),
    );
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_storage").into(),
    });
  }

  pub fn poll(&mut self) -> bool {
    let Some(job) = &self.job else {
      return false;
    };
    if let JobState::Finished(result) = job.try_state() {
      self.job = None;
      match result {
        Ok(snapshot) => {
          self.snapshot = snapshot;
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: tr(self.lang, "control_center.storage_refreshed").into(),
          });
        }
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          })
        }
      };
      true
    } else {
      false
    }
  }

  pub fn busy(&self) -> bool {
    self.job.is_some()
  }
}
