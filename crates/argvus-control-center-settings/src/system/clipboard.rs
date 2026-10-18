//! Copies text to the Wayland clipboard with `wl-copy`. The SSH and GPG key
//! pages both copy this way, so the spawn and the pipe handling live here.
use std::io::Write;
use std::process::Stdio;

use argvus_control_center_core::process::command;

/// `wl-copy` is not installed, or it did not take the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardError;

/// Copies `text` exactly as given; callers trim it when they need to.
pub fn copy(text: &str) -> Result<(), ClipboardError> {
  let mut child = command("wl-copy")
    .stdin(Stdio::piped())
    .spawn()
    .map_err(|_| ClipboardError)?;
  // The pipe is taken and dropped at the end of the closure, so wl-copy sees
  // end of input before it is awaited.
  let written = child
    .stdin
    .take()
    .is_some_and(|mut stdin| stdin.write_all(text.as_bytes()).is_ok());
  let status = child.wait().map_err(|_| ClipboardError)?;
  if written && status.success() {
    Ok(())
  } else {
    Err(ClipboardError)
  }
}
