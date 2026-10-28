//! Commands of the private handoff socket (retire, release, hold) and the client that sends them.

use super::drawing::Palette;
use super::*;

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

/// Reads the request line of one client. The socket is open to every local
/// user, so the read gives up after `CLIENT_TIMEOUT`.
pub(super) fn read_command(stream: &mut UnixStream) -> io::Result<String> {
  stream.set_read_timeout(Some(CLIENT_TIMEOUT))?;
  stream.set_write_timeout(Some(CLIENT_TIMEOUT))?;
  let mut command = String::new();
  BufReader::new(&mut *stream).read_line(&mut command)?;
  Ok(command)
}

/// Handles one client request while the surface owns the output. A release
/// returns its stream still open, so the acknowledgement can be written after
/// DRM master has been dropped.
pub(super) fn handle_command(
  mut stream: UnixStream,
  palette: &mut Palette,
) -> io::Result<Option<UnixStream>> {
  let line = read_command(&mut stream)?;
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
