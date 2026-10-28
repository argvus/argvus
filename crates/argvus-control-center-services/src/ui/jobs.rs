//! Background work of the Services page: loading data, running actions and polling their results.

use super::*;

impl ServicesApp {
  pub fn reload(&mut self) {
    self.refresh();
  }

  pub(super) fn refresh(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let page = self.page;
    let previous = self.previous_boot;
    let priority = self.priority.clone();
    let logs_unit = self.logs_unit.clone();
    let detail_user = self.detail_parent == ServicePage::User;
    let (sender, receiver) = std::sync::mpsc::channel();
    self.updates = Some(receiver);
    self.requested_page = page;
    self.job = Some(self.manager.spawn(move |_| {
      match page {
        ServicePage::Home => {
          let system_sender = sender.clone();
          let user_sender = sender.clone();
          let (system, user) = rayon::join(
            || {
              backend::list_progressive(false, |units| {
                let _ = system_sender.send(units);
              })
            },
            || {
              backend::list_progressive(true, |units| {
                let _ = user_sender.send(units);
              })
            },
          );
          let mut system = system?;
          system.append(&mut user?);
          Ok(JobData::Units(system))
        }
        ServicePage::User => backend::list_progressive(true, |units| {
          let _ = sender.send(units);
        })
        .map(JobData::Units),
        ServicePage::Failed => {
          let system_sender = sender.clone();
          let user_sender = sender;
          let (system, user) = rayon::join(
            || {
              backend::list_progressive(false, |units| {
                let _ = system_sender.send(units);
              })
            },
            || {
              backend::list_progressive(true, |units| {
                let _ = user_sender.send(units);
              })
            },
          );
          let mut system = system?;
          let mut user = user?;
          system.append(&mut user);
          Ok(JobData::Units(system))
        }
        ServicePage::Logs | ServicePage::LogDetail(_) => {
          backend::logs(logs_unit.as_deref(), previous, priority.as_deref(), 200).map(JobData::Logs)
        }
        ServicePage::Detail => backend::list(detail_user).map(JobData::Units),
        _ => backend::list_progressive(false, |_| {}).map(JobData::Units),
      }
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_services").into(),
    });
  }

  pub fn poll(&mut self) -> bool {
    if self.job.is_some() && self.updates.is_some() && self.requested_page != self.page {
      if let Some(job) = self.job.take() {
        job.cancel();
      }
      self.updates = None;
      self.refresh();
    }
    let mut changed = false;
    let updates = self
      .updates
      .as_ref()
      .map(|receiver| receiver.try_iter().collect::<Vec<_>>())
      .unwrap_or_default();
    for units in updates {
      self.accept_units(units);
      changed = true;
    }
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      self.updates = None;
      match result {
        Ok(JobData::Units(v)) => {
          self.units = v;
          self.selected = self.selected.min(self.filtered().len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Logs(v)) => {
          self.logs = v;
          self.selected = self.selected.min(self.logs.len().saturating_sub(1));
          self.status = None;
        }
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          })
        }
      };
      changed = true
    }
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      match result {
        Ok(v) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: v,
          })
        }
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          })
        }
      };
      self.refresh();
      changed = true
    }
    changed
  }

  /// Runs one row of the service details: `logs` opens the journal, any
  /// other action asks for confirmation first.
  pub(super) fn run_detail(&mut self, action: &'static str) {
    if action == "logs" {
      self.open_logs();
    } else {
      self.request(action);
    }
  }
}

pub(super) fn run_action(user: bool, action: &str, unit: &str) -> Result<String, String> {
  let args = backend::action_args(action, unit)?;
  if !user {
    let executable = std::env::current_exe()
      .map_err(|error| error.to_string())?
      .to_string_lossy()
      .into_owned();
    let operation = SystemSettingsOperation::new(SystemProcessRunner, executable);
    let request = PrivilegedRequest::new("service", action, vec![unit.into()])?;
    let output = operation.execute(&request)?;
    return if output.status == Some(0) {
      Ok(format!("{} {}", action, unit))
    } else {
      Err(privileged_failure(&output))
    };
  }
  let mut request = ProcessRequest::new("systemctl");
  if user {
    request = request.arg("--user");
  }
  for a in &args {
    request = request.arg(a)
  }
  let out = SystemProcessRunner
    .run(&request)
    .map_err(|e| e.to_string())?;
  if out.status.is_some_and(|s| s == 0) {
    Ok(format!("{} {}", action, unit))
  } else {
    Err(terminal_text(String::from_utf8_lossy(&out.stderr).trim()))
  }
}
