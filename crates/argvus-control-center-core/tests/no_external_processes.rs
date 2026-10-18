//! With the `testing` feature (enabled by every crate's dev-dependencies),
//! no external program starts: tests can never reach `pkexec`, `pacman`,
//! `systemctl`, `hyprctl` or `argvus-config` on the developer's machine.
use argvus_control_center_core::process::{
  LiveProcess, ProcessRequest, ProcessRunner, SystemProcessRunner, command,
};
use std::io::ErrorKind;

#[test]
fn commands_fail_to_start_as_if_the_tool_were_missing() {
  let error = command("true").status().unwrap_err();
  assert_eq!(error.kind(), ErrorKind::NotFound);
  let error = command("/usr/bin/pkexec").arg("true").output().unwrap_err();
  assert_eq!(error.kind(), ErrorKind::NotFound);
}

#[test]
fn the_system_runner_starts_nothing() {
  let request = ProcessRequest::new("sh").arg("-c").arg("exit 0");
  assert!(SystemProcessRunner.run(&request).is_err());
  assert!(
    SystemProcessRunner
      .run_live(&request, &LiveProcess::new())
      .is_err()
  );
}
