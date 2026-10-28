//! Background work of the Boot page: loading data, running actions and polling their results.

use super::actions::boot_request;
use super::render::transaction_bottom_offset;
use super::*;

impl BootApp {
  pub fn reload(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let capabilities = self.capabilities.clone();
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(Ok(
        BootBackend::new(SystemProcessRunner, capabilities).collect(),
      ))
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_boot").into(),
    });
  }

  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      match result {
        Ok(Ok(snapshot)) => {
          self.snapshot = snapshot;
          self.normalize();
          self.success(tr(self.lang, "control_center.boot_refreshed"));
        }
        Ok(Err(error)) | Err(error) => self.error(error),
      }
      changed = true;
    }
    if let Some(live) = &self.transaction_live
      && self.transaction_open
      && self.transaction_follow
      && self
        .action
        .as_ref()
        .is_some_and(|job| matches!(job.try_state(), JobState::Running))
    {
      self.transaction_scroll = transaction_bottom_offset(&live.output());
      changed = true;
    }
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      match result {
        Ok((outcome, _)) => match outcome {
          ActionResult::Success => self.success(tr(
            self.lang,
            "control_center.operation_completed_the_change_will_be_used_on_the_next_boot",
          )),
        },
        Err(error) => self.error(error),
      }
      self.reload();
      changed = true;
    }
    changed
  }

  pub(super) fn busy(&self) -> bool {
    self.job.is_some() || self.action.is_some()
  }

  pub(super) fn start_pending(&mut self) {
    let Some(Pending::Action(action)) = self.pending.take() else {
      return;
    };
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_boot_change").into(),
    });
    let live = LiveProcess::new();
    self.transaction_live = Some(live.clone());
    self.transaction_open = true;
    self.transaction_scroll = 0;
    self.transaction_follow = true;
    self.action = Some(self.jobs.spawn(move |_| run_action(action, live)));
  }
}

/// Runs `action` through `<executable> system-settings boot ...`, which
/// elevates itself with `pkexec`, streaming its output into `live`.
pub(super) fn run_action(
  action: BootAction,
  live: LiveProcess,
) -> Result<(ActionResult, String), String> {
  let (echo, request) = boot_request(action)?;
  live.push_line(&echo);
  let executable = std::env::current_exe()
    .map_err(|error| error.to_string())?
    .to_string_lossy()
    .into_owned();
  let output =
    SystemSettingsOperation::new(SystemProcessRunner, executable).execute_live(&request, &live)?;
  if output.status == Some(0) {
    let stdout = terminal_text(&String::from_utf8_lossy(&output.stdout))
      .trim()
      .to_owned();
    Ok((
      ActionResult::Success,
      if stdout.is_empty() {
        "ok".to_owned()
      } else {
        stdout
      },
    ))
  } else {
    Err(
      terminal_text(&String::from_utf8_lossy(&output.stderr))
        .trim()
        .into(),
    )
  }
}
