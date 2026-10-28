//! Background work of the Hardware Overview page: loading data, running actions and polling their results.

use super::*;

impl HardwareApp {
  pub fn refresh(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let cap = self.cap.clone();
    self.job = Some(self.manager.spawn(move |_| Ok(backend::collect(&cap))));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_hardware").into(),
    });
  }

  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      match result {
        Ok(msg) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: msg,
          });
        }
        Err(err) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: err,
          });
        }
      }
      self.refresh();
      changed = true;
    }
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      match result {
        Ok(snapshot) => {
          self.snapshot = snapshot;
          self.normalize();
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: tr(self.lang, "control_center.hardware_refreshed").into(),
          });
        }
        Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: error,
          });
        }
      }
      changed = true;
    }
    changed
  }

  pub fn busy(&self) -> bool {
    self.job.is_some() || self.action.is_some()
  }
}

pub(super) fn run_governor(value: &str) -> Result<String, String> {
  let request = PrivilegedRequest::new("governor", "set", vec![value.into()])?;
  let executable = std::env::current_exe()
    .map_err(|error| error.to_string())?
    .to_string_lossy()
    .into_owned();
  let output = SystemSettingsOperation::new(SystemProcessRunner, executable).execute(&request)?;
  output
    .status
    .is_some_and(|v| v == 0)
    .then(|| format!("Governor: {value}"))
    .ok_or_else(|| String::from_utf8_lossy(&output.stderr).trim().into())
}

pub(super) fn run_profile(value: &str) -> Result<String, String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("powerprofilesctl")
        .arg("set")
        .arg(value),
    )
    .map_err(|e| e.to_string())?;
  output
    .status
    .is_some_and(|v| v == 0)
    .then(|| format!("Profile: {value}"))
    .ok_or_else(|| String::from_utf8_lossy(&output.stderr).trim().into())
}
