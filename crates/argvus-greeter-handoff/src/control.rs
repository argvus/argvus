//! Commands of the private handoff socket (retire, release, hold) and the client that sends them.

use super::drawing::Palette;
use super::*;
use std::io::Read;

/// A request a client sends to the surface, one per connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Command<'a> {
  /// Changes the theme of the frame.
  Theme(&'a str),
  /// Answered once the surface draws, so a client can wait for its first frame.
  Ping,
  /// Drops DRM master for the user compositor.
  Release,
  /// The session spinner is on screen; the overlay can go.
  Retire,
  Unknown,
}

pub(super) fn parse_command(line: &str) -> Command<'_> {
  match line.trim() {
    "ping" => Command::Ping,
    "release" => Command::Release,
    "retire" => Command::Retire,
    other => other
      .strip_prefix("theme=")
      .map_or(Command::Unknown, Command::Theme),
  }
}

#[derive(Clone, Copy)]
pub(super) struct ClientPolicy {
  pub(super) owner_uid: u32,
  pub(super) session_uid: Option<u32>,
}

impl ClientPolicy {
  pub(super) fn allows(self, uid: u32) -> bool {
    uid == 0 || uid == self.owner_uid || self.session_uid == Some(uid)
  }

  fn authorize(self, stream: &UnixStream) -> io::Result<()> {
    let mut credentials = libc::ucred {
      pid: 0,
      uid: 0,
      gid: 0,
    };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: both output pointers reference writable values of the advertised
    // size, and the stream owns a valid Unix socket for the duration of the call.
    let result = unsafe {
      libc::getsockopt(
        stream.as_raw_fd(),
        libc::SOL_SOCKET,
        libc::SO_PEERCRED,
        (&mut credentials as *mut libc::ucred).cast(),
        &mut length,
      )
    };
    if result != 0 {
      return Err(io::Error::last_os_error());
    }
    if length as usize != std::mem::size_of::<libc::ucred>() || !self.allows(credentials.uid) {
      return Err(io::Error::from(io::ErrorKind::PermissionDenied));
    }
    Ok(())
  }
  pub(super) fn current(session_uid: Option<u32>) -> Self {
    // SAFETY: geteuid has no pointer arguments or preconditions.
    Self {
      owner_uid: unsafe { libc::geteuid() },
      session_uid,
    }
  }
}

/// Authenticate before reading any command, including in the overlay stage.
/// Filesystem access permits the greeter and a different session UID to connect;
/// SO_PEERCRED is the authorization boundary. Authorized reads are bounded.
pub(super) fn read_command(stream: &mut UnixStream, policy: ClientPolicy) -> io::Result<String> {
  policy.authorize(stream)?;
  stream.set_read_timeout(Some(CLIENT_TIMEOUT))?;
  stream.set_write_timeout(Some(CLIENT_TIMEOUT))?;
  let mut command = String::new();
  const MAX_COMMAND_BYTES: usize = 4096;
  BufReader::new((&mut *stream).take((MAX_COMMAND_BYTES + 1) as u64)).read_line(&mut command)?;
  if command.len() > MAX_COMMAND_BYTES || !command.ends_with('\n') {
    return Err(io::Error::from(io::ErrorKind::InvalidData));
  }
  Ok(command)
}

/// Commands remain available while DRM master belongs to the greeter. A ping
/// must not acknowledge a frame that has not been drawn yet.
#[derive(Default)]
pub(super) struct AcquisitionRequests {
  ping: Option<UnixStream>,
}

impl AcquisitionRequests {
  pub(super) fn poll(
    &mut self,
    listener: &UnixListener,
    palette: &mut Palette,
    policy: ClientPolicy,
  ) -> io::Result<Option<UnixStream>> {
    let (mut stream, _) = match listener.accept() {
      Ok(client) => client,
      Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(None),
      Err(error) => return Err(error),
    };
    let line = match read_command(&mut stream, policy) {
      Ok(line) => line,
      // A malformed, silent or unauthorized client must not end the helper.
      Err(_) => return Ok(None),
    };
    match parse_command(&line) {
      Command::Release => return Ok(Some(stream)),
      Command::Ping => {
        // Bound retained connections even when a client repeatedly pings.
        self.ping = Some(stream);
        return Ok(None);
      }
      Command::Theme(theme) => *palette = palette.with_theme(theme),
      Command::Retire => {}
      Command::Unknown => {
        let _ = stream.write_all(b"error\n");
        return Ok(None);
      }
    }
    let _ = stream.write_all(b"ok\n");
    Ok(None)
  }

  pub(super) fn first_frame(&mut self) {
    if let Some(mut ping) = self.ping.take() {
      let _ = ping.write_all(b"ok\n");
    }
  }
}

/// Handles one client request while the surface owns the output. A release
/// returns its stream still open, so the acknowledgement can be written after
/// DRM master has been dropped.
pub(super) fn handle_command(
  mut stream: UnixStream,
  palette: &mut Palette,
  policy: ClientPolicy,
) -> io::Result<Option<UnixStream>> {
  let line = read_command(&mut stream, policy)?;
  match parse_command(&line) {
    Command::Theme(theme) => *palette = palette.with_theme(theme),
    Command::Release => return Ok(Some(stream)),
    // Nothing is on an overlay before the release.
    Command::Ping | Command::Retire => {}
    Command::Unknown => {
      stream.write_all(b"error\n")?;
      return Ok(None);
    }
  }
  stream.write_all(b"ok\n")?;
  Ok(None)
}

/// Sends one command to the running surface and waits for its acknowledgement.
/// A missing socket means no surface is running, which is not an error.
pub(super) fn send_command(socket: &Path, command: &str) -> io::Result<()> {
  let mut stream = match UnixStream::connect(socket) {
    Ok(stream) => stream,
    Err(error)
      if matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
      ) =>
    {
      return Ok(());
    }
    Err(error) => return Err(error),
  };
  stream.set_read_timeout(Some(ACK_TIMEOUT))?;
  writeln!(stream, "{command}")?;
  let mut acknowledgement = String::new();
  BufReader::new(&mut stream).read_line(&mut acknowledgement)?;
  if acknowledgement.trim().is_empty() {
    return Err(io::Error::new(
      io::ErrorKind::UnexpectedEof,
      "handoff surface closed without acknowledging the command",
    ));
  }
  if !matches!(acknowledgement.trim(), "ok" | "released") {
    return Err(io::Error::from(io::ErrorKind::PermissionDenied));
  }
  Ok(())
}

pub(super) fn prepare_socket(path: &Path) -> io::Result<UnixListener> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let _ = fs::remove_file(path);
  let listener = UnixListener::bind(path)?;
  fs::set_permissions(path, fs::Permissions::from_mode(0o666))?;
  listener.set_nonblocking(true)?;
  Ok(listener)
}
