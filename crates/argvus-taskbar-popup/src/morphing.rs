//! Private transport for the Taskbar's embedded application frontends.

use std::cell::Cell;
use std::io;
use std::os::fd::OwnedFd;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::{gio, glib, prelude::*};
use serde::Deserialize;
use serde_json::{Value, json};

const VERSION: u32 = 1;
const MAX_REQUEST_BYTES: usize = 64 * 1024;
const SOCKET_TIMEOUT_SECONDS: u32 = 15;
const MAX_CONNECTIONS: usize = 8;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
  version: u32,
  id: u32,
  action: String,
  #[serde(default)]
  args: Value,
}

type Operation<'a> = dyn Fn(&str, Value) -> Result<Value, String> + 'a;

/// A listener owned by the application's main context. Reads and writes
/// use GIO async operations. The application handles requests serially on
/// the same context as its other UI and notification actions.
pub struct Server {
  service: gio::SocketService,
  path: PathBuf,
}

impl Server {
  pub fn start(
    name: &'static str,
    opened: impl Fn() -> Option<Box<dyn FnOnce()>> + 'static,
    operation: impl Fn(&str, Value) -> Result<Value, String> + 'static,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let path = socket_path(name)?;
    let listener = bind(&path)?;
    let service = gio::SocketService::new();
    let result = gio::Socket::from_fd(OwnedFd::from(listener))
      .and_then(|socket| service.add_socket(&socket, None::<&glib::Object>));
    if let Err(error) = result {
      let _ = std::fs::remove_file(&path);
      return Err(error.into());
    }

    let opened = Rc::new(opened);
    let operation = Rc::new(operation);
    let open_connections = Rc::new(Cell::new(0));
    let next_session = Cell::new(0_u64);
    service.connect_incoming(move |_, connection, _| {
      if open_connections.get() >= MAX_CONNECTIONS {
        return true;
      }
      open_connections.set(open_connections.get() + 1);
      next_session.set(next_session.get() + 1);
      let session_id = format!("{}-{}", std::process::id(), next_session.get());
      let open_connections = Rc::clone(&open_connections);
      let connection = connection.clone();
      let opened = Rc::clone(&opened);
      let operation = Rc::clone(&operation);
      connection.socket().set_timeout(SOCKET_TIMEOUT_SECONDS);
      glib::MainContext::ref_thread_default().spawn_local(async move {
        let _ = serve(
          &connection,
          &session_id,
          opened.as_ref(),
          operation.as_ref(),
        )
        .await;
        open_connections.set(open_connections.get() - 1);
      });
      true
    });
    service.start();
    Ok(Self { service, path })
  }
}

impl Drop for Server {
  fn drop(&mut self) {
    self.service.stop();
    self.service.close();
    let _ = std::fs::remove_file(&self.path);
  }
}

fn socket_path(name: &str) -> io::Result<PathBuf> {
  let runtime = std::env::var_os("XDG_RUNTIME_DIR")
    .filter(|path| Path::new(path).is_absolute())
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "XDG_RUNTIME_DIR is unavailable"))?;
  let runtime = PathBuf::from(runtime);
  let metadata = runtime.symlink_metadata()?;
  if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
    return Err(io::Error::new(
      io::ErrorKind::PermissionDenied,
      "XDG_RUNTIME_DIR must be private",
    ));
  }
  let directory = runtime.join("argvus");
  match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
    Ok(()) => {}
    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
    Err(error) => return Err(error),
  }
  let metadata = directory.symlink_metadata()?;
  if !metadata.is_dir() {
    return Err(io::Error::new(
      io::ErrorKind::PermissionDenied,
      "ARGVUS runtime path must be a directory",
    ));
  }
  Ok(directory.join(format!("{name}-morphing.sock")))
}

fn bind(path: &Path) -> io::Result<UnixListener> {
  match UnixListener::bind(path) {
    Err(error)
      if error.kind() == io::ErrorKind::AddrInUse
        && path.symlink_metadata()?.file_type().is_socket()
        && UnixStream::connect(path)
          .is_err_and(|error| error.kind() == io::ErrorKind::ConnectionRefused) =>
    {
      std::fs::remove_file(path)?;
      UnixListener::bind(path)
    }
    result => result,
  }
}

struct Session(Option<Box<dyn FnOnce()>>);

impl Drop for Session {
  fn drop(&mut self) {
    if let Some(closed) = self.0.take() {
      closed();
    }
  }
}

async fn serve(
  connection: &gio::SocketConnection,
  session_id: &str,
  opened: &dyn Fn() -> Option<Box<dyn FnOnce()>>,
  operation: &Operation<'_>,
) -> Option<()> {
  let input = connection.input_stream();
  let output = connection.output_stream();
  let mut buffer = Vec::new();
  let mut previous_id = 0;
  let mut revision = 0_u64;
  let mut session = Session(None);
  while let Ok(chunk) = input.read_bytes_future(4096, glib::Priority::DEFAULT).await {
    if chunk.is_empty() {
      break;
    }
    buffer.extend_from_slice(chunk.as_ref());
    while let Some(end) = buffer.iter().position(|&byte| byte == b'\n') {
      if end > MAX_REQUEST_BYTES {
        return None;
      }
      let request: Request = serde_json::from_slice(&buffer[..end]).ok()?;
      buffer.drain(..=end);
      let mut finish = true;
      let result = if request.version != VERSION {
        Err("incompatible".into())
      } else if request.id <= previous_id {
        Err("invalid_id".into())
      } else if session.0.is_none() {
        if request.action != "hello" {
          Err("handshake_required".into())
        } else if let Some(closed) = opened() {
          session.0 = Some(closed);
          finish = false;
          operation("state", Value::Null)
        } else {
          Err("unavailable".into())
        }
      } else if request.action == "close" {
        Ok(Value::Null)
      } else {
        finish = false;
        operation(&request.action, request.args)
      };
      previous_id = request.id;
      revision += 1;
      let mut response = json!({
        "version": VERSION,
        "id": request.id,
        "session": session_id,
        "revision": revision.to_string(),
        "ok": result.is_ok(),
      });
      match result {
        Ok(state) => response["state"] = state,
        Err(error) => response["error"] = error.into(),
      }
      let mut bytes = serde_json::to_vec(&response).ok()?;
      bytes.push(b'\n');
      let (_, _, error) = output
        .write_all_future(bytes, glib::Priority::DEFAULT)
        .await
        .ok()?;
      if error.is_some() || finish {
        return None;
      }
    }
    if buffer.len() > MAX_REQUEST_BYTES {
      return None;
    }
  }
  Some(())
}

/// Requests a presentation from the running Taskbar. The Gtk entry point
/// handles any unavailable or older shell.
pub fn present(name: &str, toggle: bool, anchor: Option<crate::PopupAnchor>, page: &str) -> bool {
  let action = if toggle { "toggle" } else { "show" };
  present_action(name, action, anchor, page)
}

pub fn hide(name: &str) -> bool {
  present_action(name, "hide", None, "")
}

fn present_action(
  name: &str,
  action: &str,
  anchor: Option<crate::PopupAnchor>,
  page: &str,
) -> bool {
  if std::env::var_os("ARGVUS_TASKBAR_GTK").is_some() {
    return false;
  }
  let system_config =
    std::env::var("ARGVUS_SYSTEM_CONFIG").unwrap_or_else(|_| "/usr/share/argvus".into());
  let shell = Path::new(&system_config).join("panels/qml/shell.qml");
  if !shell.is_file() {
    return false;
  }
  let request = json!({
    "version": VERSION,
    "kind": name,
    "action": action,
    "page": page,
    "anchor": anchor.map(|position| json!({
      "x": position.x,
      "y": position.y,
      "bar_bottom": position.bar_bottom,
      "bar_right": position.bar_right,
    })),
  });
  std::process::Command::new("qs")
    .arg("-p")
    .arg(shell)
    .args(["ipc", "call", "morphing", "present"])
    .arg(request.to_string())
    .output()
    .is_ok_and(|result| result.status.success() && result.stdout.trim_ascii() == b"accepted")
}

#[cfg(test)]
mod tests {
  use std::io::{BufRead, BufReader, Write};
  use std::sync::Arc;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::time::Duration;

  use super::*;

  fn round_trip(frames: &[Value]) -> (Vec<Value>, usize, usize) {
    let (server, mut client) = UnixStream::pair().unwrap();
    client
      .set_read_timeout(Some(Duration::from_secs(2)))
      .unwrap();
    let mut input = BufReader::new(client.try_clone().unwrap());
    let operations = AtomicUsize::new(0);
    let closed = Arc::new(AtomicUsize::new(0));
    let frames = frames.to_vec();

    std::thread::scope(|scope| {
      let sender = scope.spawn(move || {
        frames
          .into_iter()
          .map(|frame| {
            writeln!(client, "{frame}").unwrap();
            let mut response = String::new();
            input.read_line(&mut response).unwrap();
            serde_json::from_str(&response).unwrap()
          })
          .collect()
      });
      let context = glib::MainContext::new();
      let socket = gio::Socket::from_fd(OwnedFd::from(server)).unwrap();
      let connection = socket.connection_factory_create_connection();
      context.block_on(serve(
        &connection,
        "test-session",
        &|| {
          let closed = Arc::clone(&closed);
          Some(Box::new(move || {
            closed.fetch_add(1, Ordering::Relaxed);
          }))
        },
        &|action, _| {
          Ok(json!({
            "action": action,
            "count": operations.fetch_add(1, Ordering::Relaxed) + 1,
          }))
        },
      ));
      (
        sender.join().unwrap(),
        operations.into_inner(),
        closed.load(Ordering::Relaxed),
      )
    })
  }

  fn request(version: u32, id: u32, action: &str) -> Value {
    json!({ "version": version, "id": id, "action": action })
  }

  #[test]
  fn handshake_and_mutations_use_ordered_session_responses() {
    let (replies, operations, attached) = round_trip(&[
      request(VERSION, 1, "hello"),
      request(VERSION, 2, "save"),
      request(VERSION, 3, "close"),
    ]);
    assert_eq!(operations, 2);
    assert_eq!(attached, 1);
    assert_eq!(replies[0]["state"]["action"], "state");
    assert_eq!(replies[1]["state"]["action"], "save");
    assert_eq!(replies[1]["session"], "test-session");
    assert_eq!(replies[1]["revision"], "2");
    assert!(replies[2]["state"].is_null());
  }

  #[test]
  fn incompatible_or_unattached_client_cannot_mutate() {
    for (request, error) in [
      (request(VERSION + 1, 1, "hello"), "incompatible"),
      (request(VERSION, 1, "save"), "handshake_required"),
    ] {
      let (replies, operations, attached) = round_trip(&[request]);
      assert_eq!(operations, 0);
      assert_eq!(attached, 0);
      assert_eq!(replies[0]["error"], error);
    }
  }

  #[test]
  fn duplicate_mutation_is_rejected_and_disconnect_releases_session() {
    let (replies, operations, attached) = round_trip(&[
      request(VERSION, 1, "hello"),
      request(VERSION, 2, "save"),
      request(VERSION, 2, "save"),
    ]);
    assert_eq!(operations, 2);
    assert_eq!(attached, 1);
    assert_eq!(replies[2]["error"], "invalid_id");

    let (_, operations, attached) = round_trip(&[request(VERSION, 1, "hello")]);
    assert_eq!(operations, 1);
    assert_eq!(attached, 1);
  }
}
