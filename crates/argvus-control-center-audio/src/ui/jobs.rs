//! Background work of the Audio page: loading data, running actions and polling their results.

use super::*;

impl AudioApp {
  pub fn reload(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let caps = self.capabilities.clone();
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(
        AudioBackend::new(SystemProcessRunner, caps)
          .snapshot()
          .map_err(|e| e.to_string()),
      )
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_audio").into(),
    });
  }

  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(j) = &self.job
      && let JobState::Finished(r) = j.try_state()
    {
      self.job = None;
      match r {
        Ok(Ok(s)) => {
          self.snapshot = s;
          self.normalize();
          self.success(tr(self.lang, "control_center.audio_refreshed"));
        }
        Ok(Err(e)) | Err(e) => self.error(e),
      };
      changed = true;
    }
    if let Some(j) = &self.action
      && let JobState::Finished(r) = j.try_state()
    {
      self.action = None;
      match r {
        Ok(Ok(msg)) => {
          if self.saving {
            self.draft = AudioDraft::default();
          }
          self.success(msg);
          self.reload();
        }
        Ok(Err(e)) | Err(e) => {
          self.error(e);
          self.reload();
        }
      };
      self.saving = false;
      changed = true;
    }
    changed
  }

  pub(super) fn start_action(
    &mut self,
    action: impl FnOnce(AudioBackend<SystemProcessRunner>) -> Result<(), AudioError> + Send + 'static,
    success: String,
  ) {
    if self.action.is_some() {
      return;
    }
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_audio_change").into(),
    });
    let caps = self.capabilities.clone();
    self.action = Some(self.jobs.spawn(move |_| {
      Ok(
        action(AudioBackend::new(SystemProcessRunner, caps))
          .map(|_| success)
          .map_err(|e| e.to_string()),
      )
    }));
  }
}
