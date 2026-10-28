//! Background work of the Network & Wi-Fi page: loading data, running actions and polling their results.

use super::*;

impl NetworkApp {
  pub fn reload(&mut self) {
    // Opened from a route (search, CLI), the Add page still lists the
    // installed VPN plugins.
    if self.page == NetworkPage::AddConnection {
      self.refresh_vpn_plugins();
    }
    if self.requested_page != self.page
      || self.refreshed.is_none_or(|at| at.elapsed().as_secs() >= 5)
    {
      self.start_refresh(false);
    }
  }

  pub(super) fn start_refresh(&mut self, scan: bool) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let cap = self.capabilities.clone();
    self.scan = scan;
    let page = self.page;
    self.requested_page = page;
    let previous = self.snapshot.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    self.updates = Some(receiver);
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(
        NetworkBackend::new(SystemProcessRunner, cap)
          .load_page(page, scan, previous, |state| {
            let _ = sender.send(state);
          })
          .map_err(|e| e.to_string()),
      )
    }));
  }

  pub fn poll(&mut self) -> bool {
    if self.job.is_some() && self.requested_page != self.page {
      if let Some(job) = self.job.take() {
        job.cancel();
      }
      self.updates = None;
      self.scan = false;
      self.start_refresh(false);
    }
    let mut changed = false;
    let updates = self
      .updates
      .as_ref()
      .map(|receiver| receiver.try_iter().collect::<Vec<_>>())
      .unwrap_or_default();
    for snapshot in updates {
      self.accept_snapshot(snapshot);
      changed = true;
    }
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      self.updates = None;
      match result {
        Ok(Ok(snapshot)) => {
          self.accept_snapshot(snapshot);
          self.refreshed = Some(std::time::Instant::now());
          if self.status.is_none() {
            self.status = Some(StatusMessage {
              kind: StatusKind::Success,
              text: if self.scan {
                tr(self.lang, "control_center.networks_updated")
              } else {
                tr(self.lang, "control_center.network_refreshed")
              }
              .into(),
            });
          }
        }
        Ok(Err(error)) | Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: error,
          })
        }
      }
      self.scan = false;
      if self.requested_page != self.page {
        self.start_refresh(false);
      }
      changed = true;
    }
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      let (kind, text) = match result {
        Ok(Ok(text)) => (StatusKind::Success, text),
        Ok(Err(error)) | Err(error) => (StatusKind::Error, error),
      };
      self.status = Some(StatusMessage { kind, text });
      self.start_refresh(false);
      changed = true;
    }
    changed |= self.poll_editor();
    changed
  }

  pub(super) fn run_wifi(&mut self, index: usize, password: Option<String>) {
    let Some(network) = self.snapshot.wifi.get(index).cloned() else {
      return;
    };
    let cap = self.capabilities.clone();
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.connecting").into(),
    });
    self.action = Some(self.jobs.spawn(move |_| {
      Ok(
        NetworkBackend::new(SystemProcessRunner, cap)
          .connect_wifi(&network.ssid, password.as_deref())
          .map(|_| "Conectado".into())
          .map_err(|e| e.to_string()),
      )
    }));
  }

  /// The trailing Actions group of the list pages: `Refresh` (`r`).
  pub(super) fn refresh_section(&self) -> Vec<Row<Item>> {
    vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
    ]
  }
}
