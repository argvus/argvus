//! Best-effort diagnostics through the local syslog socket, never through the TUI terminal.
#[cfg(any(test, not(feature = "testing")))]
use std::{io, os::unix::net::UnixDatagram, path::Path};

/// Writes a diagnostic to the journal without blocking or altering the terminal.
/// Missing or busy logging infrastructure must not interfere with a user operation.
pub fn error(message: &str) {
  #[cfg(not(feature = "testing"))]
  let _ = send_error_to(Path::new("/dev/log"), message);
  #[cfg(feature = "testing")]
  let _ = message;
}

#[cfg(any(test, not(feature = "testing")))]
fn send_error_to(path: &Path, message: &str) -> io::Result<()> {
  let socket = UnixDatagram::unbound()?;
  socket.set_nonblocking(true)?;
  let text = crate::sanitize::terminal_text(message);
  let bounded: String = text.chars().take(4096).collect();
  // syslog user facility (1 << 3) and error severity (3).
  let record = format!(
    "<11>argvus-control-center[{}]: {bounded}",
    std::process::id()
  );
  socket.send_to(record.as_bytes(), path)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn diagnostics_go_to_a_socket_without_terminal_controls() {
    let root = std::env::temp_dir()
      .join("argvus")
      .join(format!("cc-log-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("socket");
    let receiver = UnixDatagram::bind(&path).unwrap();
    receiver
      .set_read_timeout(Some(Duration::from_secs(1)))
      .unwrap();
    send_error_to(&path, "shortcut failed: \x1b[31minvalid\x1b[0m").unwrap();
    let mut buffer = [0; 8192];
    let count = receiver.recv(&mut buffer).unwrap();
    let record = std::str::from_utf8(&buffer[..count]).unwrap();
    assert!(record.starts_with("<11>argvus-control-center["));
    assert!(record.ends_with("shortcut failed: invalid"));
    assert!(!record.contains('\x1b'));
    assert!(send_error_to(&root.join("missing"), "unavailable logger").is_err());
    std::fs::remove_dir_all(root).unwrap();
  }
}
