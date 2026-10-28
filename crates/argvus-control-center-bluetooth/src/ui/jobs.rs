//! Background work of the Bluetooth page: loading data, running actions and polling their results.

use super::*;

impl BluetoothApp {
  pub fn reload(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let caps = self.capabilities.clone();
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(
        BluetoothBackend::new(SystemProcessRunner, caps)
          .snapshot()
          .map_err(|e| e.to_string()),
      )
    }));
  }

  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    let mut events = Vec::new();
    if let Some(agent) = &self.agent {
      self.agent_ready = agent.is_ready();
      while let Some(event) = agent.try_event() {
        events.push(event);
      }
    }
    for event in events {
      match event {
        AgentEvent::Cancelled | AgentEvent::Released => {
          if self.prompt.is_some() {
            if let Some(agent) = &self.agent {
              agent.reply(AgentReply::Reject);
            }
            self.prompt = None;
            self.prompt_confirm = false;
            changed = true;
          }
        }
        start => self.open_prompt(start),
      }
    }
    if let Some(j) = self.job.take() {
      match j.try_state() {
        JobState::Running => self.job = Some(j),
        JobState::Finished(r) => {
          match r {
            Ok(Ok(s)) => {
              self.snapshot = s;
              self.selected.normalize(self.row_count());
            }
            Ok(Err(e)) | Err(e) => {
              self.status = Some(StatusMessage {
                kind: StatusKind::Error,
                text: e,
              })
            }
          };
          self.scanning = false;
          changed = true;
        }
      }
    }
    if let Some(j) = self.action.take() {
      match j.try_state() {
        JobState::Running => self.action = Some(j),
        JobState::Finished(r) => {
          match r {
            Ok(Ok(msg)) => {
              self.status = Some(StatusMessage {
                kind: StatusKind::Success,
                text: msg,
              });
              self.reload();
            }
            Ok(Err(e)) | Err(e) => {
              self.status = Some(StatusMessage {
                kind: StatusKind::Error,
                text: e,
              })
            }
          };
          if self.prompt.is_some() {
            if let Some(agent) = &self.agent {
              agent.reply(AgentReply::Reject);
            }
            self.prompt = None;
            self.prompt_confirm = false;
          }
          changed = true;
        }
      }
    }
    changed
  }

  pub(super) fn start_action(
    &mut self,
    operation: impl FnOnce(
      BluetoothBackend<SystemProcessRunner>,
    ) -> Result<(), crate::backend::BluetoothError>
    + Send
    + 'static,
    message: String,
  ) {
    if self.action.is_some() {
      return;
    }
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_bluetooth_change").into(),
    });
    let caps = self.capabilities.clone();
    self.action = Some(self.jobs.spawn(move |_| {
      Ok(
        operation(BluetoothBackend::new(SystemProcessRunner, caps))
          .map(|_| message)
          .map_err(|e| e.to_string()),
      )
    }));
  }

  pub(super) fn start_scan(&mut self) {
    if self.action.is_some() || self.scanning {
      return;
    }
    self.scanning = true;
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.scanning_for_devices").into(),
    });
    let caps = self.capabilities.clone();
    self.action = Some(self.jobs.spawn(move |_| {
      let b = BluetoothBackend::new(SystemProcessRunner, caps);
      b.scan(true).map_err(|e| e.to_string())?;
      std::thread::sleep(std::time::Duration::from_secs(2));
      b.scan(false).map_err(|e| e.to_string())?;
      b.snapshot().map_err(|e| e.to_string())?;
      Ok(Ok("Scan concluído.".into()))
    }));
  }
}
