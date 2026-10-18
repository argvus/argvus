//! Implements typed privileged-operation integration in crate `argvus control center core`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! A request contains separate domain, action, and argument fields. The installed helper
//! is selected by the concrete operation, never by text sent from the TUI. This
//! combination limits the polkit/system-settings surface and makes it easy to test
//! argument construction without executing a privileged operation.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::process::{LiveProcess, ProcessOutput, ProcessRequest, ProcessRunner};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `PrivilegedRequest`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct PrivilegedRequest {
  /// Operation family validated by the dispatcher, such as `service` or `boot`.
  pub domain: String,
  /// Named action known by the helper responsible for the domain.
  pub action: String,
  /// Positional data kept separate to prevent shell interpretation.
  pub arguments: Vec<String>,
}

impl PrivilegedRequest {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(
    domain: impl Into<String>,
    action: impl Into<String>,
    arguments: Vec<String>,
  ) -> Result<Self, String> {
    let request = Self {
      domain: domain.into(),
      action: action.into(),
      arguments,
    };
    request.validate()?;
    Ok(request)
  }

  /// Executes the `validate` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn validate(&self) -> Result<(), String> {
    if self.domain.is_empty()
      || self.action.is_empty()
      || self.domain.chars().any(char::is_control)
      || self.action.chars().any(char::is_control)
    {
      return Err("privileged domain and action must be non-empty and printable".into());
    }
    let mirror_payload = self.domain == "package" && self.action == "mirror-apply";
    if self.arguments.iter().any(|value| {
      value.len() > 256 * 1024
        || value
          .chars()
          .any(|character| character.is_control() && !(mirror_payload && character == '\n'))
    }) {
      return Err("privileged arguments must not contain control characters".into());
    }
    Ok(())
  }
}

/// Defines the `PrivilegedOperation`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub trait PrivilegedOperation: Send + Sync {
  /// Executes the `execute` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn execute(&self, request: &PrivilegedRequest) -> Result<ProcessOutput, String>;
}

/// Represents `HelperOperation`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct HelperOperation<R> {
  runner: R,
  helper: String,
}

/// Represents `SystemSettingsOperation`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct SystemSettingsOperation<R> {
  runner: R,
  executable: String,
}

impl<R> SystemSettingsOperation<R> {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(runner: R, executable: impl Into<String>) -> Self {
    Self {
      runner,
      executable: executable.into(),
    }
  }

  /// The process that runs `request`: `<executable> system-settings
  /// <domain> <action> <arguments...>`, which elevates itself with `pkexec`.
  /// Built without running anything, so callers and tests can check the
  /// exact operation.
  pub fn process_for(&self, request: &PrivilegedRequest) -> Result<ProcessRequest, String> {
    request.validate()?;
    let mut process = ProcessRequest::new(&self.executable)
      .arg("system-settings")
      .arg(&request.domain)
      .arg(&request.action);
    for argument in &request.arguments {
      process = process.arg(argument);
    }
    Ok(process)
  }
}

impl<R: ProcessRunner> SystemSettingsOperation<R> {
  /// Executes the `execute_live` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn execute_live(
    &self,
    request: &PrivilegedRequest,
    live: &LiveProcess,
  ) -> Result<ProcessOutput, String> {
    let process = self.process_for(request)?;
    self
      .runner
      .run_live(&process, live)
      .map_err(|error| error.to_string())
  }
}

impl<R: ProcessRunner> PrivilegedOperation for SystemSettingsOperation<R> {
  /// Executes the `execute` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn execute(&self, request: &PrivilegedRequest) -> Result<ProcessOutput, String> {
    let process = self.process_for(request)?;
    self.runner.run(&process).map_err(|error| error.to_string())
  }
}

impl<R> HelperOperation<R> {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(runner: R, helper: impl Into<String>) -> Self {
    Self {
      runner,
      helper: helper.into(),
    }
  }
}

impl<R: ProcessRunner> PrivilegedOperation for HelperOperation<R> {
  /// Executes the `execute` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn execute(&self, request: &PrivilegedRequest) -> Result<ProcessOutput, String> {
    request.validate()?;
    let mut process = ProcessRequest::new(&self.helper)
      .arg(&request.domain)
      .arg(&request.action);
    for argument in &request.arguments {
      process = process.arg(argument);
    }
    self.runner.run(&process).map_err(|error| error.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::process::{ProcessError, SystemProcessRunner};
  use std::sync::Mutex;

  #[derive(Default)]
  /// Represents `RecordingRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  struct RecordingRunner(Mutex<Option<ProcessRequest>>);
  impl ProcessRunner for RecordingRunner {
    /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
      *self.0.lock().unwrap() = Some(request.clone());
      Ok(ProcessOutput {
        stdout: Vec::new(),
        stderr: Vec::new(),
        status: Some(0),
        timed_out: false,
      })
    }
  }

  #[test]
  /// Executes the `rejects_shell_like_control_input` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn rejects_shell_like_control_input() {
    assert!(PrivilegedRequest::new("service", "restart", vec!["foo\nbar".into()]).is_err());
    assert!(PrivilegedRequest::new("service", "restart", vec!["foo;bar".into()]).is_ok());
  }

  #[test]
  /// Executes the `only_the_typed_mirror_operation_accepts_a_multiline_payload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn only_the_typed_mirror_operation_accepts_a_multiline_payload() {
    assert!(
      PrivilegedRequest::new(
        "package",
        "mirror-apply",
        vec!["Server = https://mirror.example/$repo/os/$arch\n".into()],
      )
      .is_ok()
    );
    assert!(PrivilegedRequest::new("service", "restart", vec!["foo\nbar".into()]).is_err());
  }

  #[test]
  /// Executes the `helper_receives_structured_arguments` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn helper_receives_structured_arguments() {
    let operation = HelperOperation::new(SystemProcessRunner, "printf");
    let request =
      PrivilegedRequest::new("service", "restart", vec!["unit.service".into()]).unwrap();
    let output = operation.execute(&request).unwrap();
    assert_eq!(output.stdout, b"service");
  }

  #[test]
  /// Executes the `system_settings_operation_builds_the_installed_helper_chain` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn system_settings_operation_builds_the_installed_helper_chain() {
    let runner = RecordingRunner::default();
    let operation = SystemSettingsOperation::new(runner, "/usr/bin/argvus-control-center");
    let request =
      PrivilegedRequest::new("service", "restart", vec!["demo.service".into()]).unwrap();
    operation.execute(&request).unwrap();
    let command = operation.runner.0.lock().unwrap().clone().unwrap();
    assert_eq!(command.program, "/usr/bin/argvus-control-center");
    assert_eq!(
      command.args,
      ["system-settings", "service", "restart", "demo.service"]
    );
  }

  #[test]
  /// Executes the `system_settings_operation_live_builds_the_same_elevated_chain` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn system_settings_operation_live_builds_the_same_elevated_chain() {
    let runner = RecordingRunner::default();
    let operation = SystemSettingsOperation::new(runner, "/usr/bin/argvus-control-center");
    let request = PrivilegedRequest::new("boot", "initramfs-regenerate", vec![]).unwrap();
    let live = crate::process::LiveProcess::new();
    operation.execute_live(&request, &live).unwrap();
    let command = operation.runner.0.lock().unwrap().clone().unwrap();
    assert_eq!(command.program, "/usr/bin/argvus-control-center");
    assert_eq!(
      command.args,
      ["system-settings", "boot", "initramfs-regenerate"]
    );
  }
}
