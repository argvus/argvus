//! Minimal greetd IPC client used by the login application.
//!
//! Authentication is deliberately isolated on a worker thread. The Ratatui
//! event loop must remain responsive while greetd waits for PAM responses, and
//! the UI should communicate through typed commands/events rather than owning
//! a Unix socket directly.

use crate::{handoff::WaylandSplash, monotonic_ns, session::Session};
use greetd_ipc::{
  AuthMessageType, ErrorType, Request, Response,
  codec::{Error as CodecError, SyncCodec},
};
use std::{
  env,
  os::unix::net::UnixStream,
  sync::mpsc::{Receiver, Sender},
  thread,
};
use thiserror::Error;
use tracing::{error, info, warn};

/// Commands sent from the UI to the greetd worker.
#[derive(Debug)]
pub enum Command {
  Begin {
    username: String,
    session: Session,
    theme: String,
    accent: Option<String>,
  },
  AuthResponse(Option<String>),
}

/// Notifications sent by the worker back to the UI.
#[derive(Debug, Clone)]
pub enum Event {
  Ready,
  AuthMessage(AuthPrompt),
  Info(String),
  Error(String),
  AuthFailed(String),
  AuthPromptUnavailable,
  GreeterUnavailable,
  SessionStarting,
  SessionStarted,
}

/// A prompt received from greetd, including whether its response is secret.
#[derive(Debug, Clone)]
pub struct AuthPrompt {
  pub message: String,
  pub secret: bool,
}

/// Failures that prevent the authentication exchange from continuing.
#[derive(Debug, Error)]
enum GreeterError {
  #[error("GREETD_SOCK is not set")]
  MissingSocket,
  #[error("could not connect to greetd socket: {0}")]
  Connect(std::io::Error),
  #[error("greetd IPC failed: {0}")]
  Ipc(#[from] CodecError),
}

/// Starts the background worker and returns immediately.
///
/// The worker owns the mutable IPC state. If the command channel closes, the
/// thread exits naturally because the parent application is no longer able to
/// request authentication.
pub fn spawn_worker(commands: Receiver<Command>, events: Sender<Event>) {
  thread::spawn(move || {
    // Ready is an informational event, but sending it establishes that the
    // worker was created before the UI begins issuing commands.
    let _ = events.send(Event::Ready);
    // Disconnected is the safe state after every failed or completed exchange.
    let mut state = WorkerState::Disconnected;

    while let Ok(command) = commands.recv() {
      match command {
        Command::Begin {
          username,
          session,
          theme,
          accent,
        } => {
          // greetd allows only one CreateSession exchange per worker at a
          // time. Keep the existing socket when a repeated UI event arrives;
          // replacing it would abandon the in-flight authentication and make
          // every subsequent attempt appear to be concurrently configured.
          if matches!(&state, WorkerState::Authenticating { .. }) {
            warn!(%username, "ignoring duplicate greetd session request");
            continue;
          }
          // A failed begin operation must not leave a partially initialized
          // socket available for a later password submission.
          state = match begin_session(username, session, theme, accent, &events) {
            Ok(state) => state,
            Err(error) => {
              warn!(%error, "could not begin greetd session");
              let _ = events.send(Event::GreeterUnavailable);
              WorkerState::Disconnected
            }
          };
        }
        Command::AuthResponse(response) => {
          // Responses are valid only while greetd is waiting for an answer to
          // an authentication prompt.
          let WorkerState::Authenticating {
            mut stream,
            session,
            theme,
            accent,
          } = state
          else {
            let _ = events.send(Event::AuthPromptUnavailable);
            state = WorkerState::Disconnected;
            continue;
          };

          state = match send_auth_response(&mut stream, response, session, theme, accent, &events) {
            Ok(next) => next,
            Err(error) => {
              warn!(%error, "authentication exchange failed");
              let _ = events.send(Event::GreeterUnavailable);
              WorkerState::Disconnected
            }
          };
        }
      }
    }
  });
}

/// Internal state of the greetd protocol exchange.
enum WorkerState {
  Disconnected,
  Authenticating {
    stream: UnixStream,
    session: Session,
    theme: String,
    accent: Option<String>,
  },
}

/// Creates a greetd session for the selected user and session command.
fn begin_session(
  username: String,
  session: Session,
  theme: String,
  accent: Option<String>,
  events: &Sender<Event>,
) -> Result<WorkerState, GreeterError> {
  // greetd provides the socket path through the environment of the greeter
  // process; no socket path is guessed because that could target the wrong
  // session manager instance.
  let socket = env::var("GREETD_SOCK").map_err(|_| GreeterError::MissingSocket)?;
  let mut stream = UnixStream::connect(socket).map_err(GreeterError::Connect)?;
  info!(%username, "connected to greetd");

  Request::CreateSession { username }.write_to(&mut stream)?;
  handle_response(&mut stream, session, theme, accent, events)
}

/// Posts one authentication response and continues processing greetd output.
fn send_auth_response(
  stream: &mut UnixStream,
  response: Option<String>,
  session: Session,
  theme: String,
  accent: Option<String>,
  events: &Sender<Event>,
) -> Result<WorkerState, GreeterError> {
  Request::PostAuthMessageResponse { response }.write_to(stream)?;
  handle_response(stream, session, theme, accent, events)
}

/// Consumes greetd responses until the protocol needs UI input or terminates.
///
/// Informational and error messages are acknowledged immediately. Secret and
/// visible prompts return an `Authenticating` state with a cloned stream so
/// subsequent UI input can continue the same protocol exchange.
fn handle_response(
  stream: &mut UnixStream,
  session: Session,
  theme: String,
  accent: Option<String>,
  events: &Sender<Event>,
) -> Result<WorkerState, GreeterError> {
  loop {
    match Response::read_from(stream)? {
      Response::Success => {
        // Authentication succeeded, so the selected session is handed to
        // greetd rather than launched by the greeter process itself.
        info!(session = %session.id, monotonic_ns = monotonic_ns(), "authentication succeeded");
        // The overlay must be mapped while Kitty and the greeter compositor
        // are still alive. A Wayland client cannot survive their later DRM
        // handover, but it eliminates the pre-handover empty screen.
        let splash = match WaylandSplash::start(&theme, accent.as_deref()) {
          Ok(splash) => splash,
          Err(error) => {
            error!(%error, "could not start the greeter handoff splash");
            let _ = events.send(Event::Error(
              "Could not start the session transition.".into(),
            ));
            return Ok(WorkerState::Disconnected);
          }
        };
        let _ = events.send(Event::SessionStarting);
        Request::StartSession {
          cmd: session.command.clone(),
          env: session.launch_env(),
        }
        .write_to(stream)?;

        match Response::read_from(stream)? {
          Response::Success => {
            info!(session = %session.id, "session started");
            // greetd terminates the greeter session after accepting the user
            // command. Keep the splash parented here until that SIGTERM, so
            // terminal cleanup cannot uncover the compositor in between.
            splash.detach();
            let _ = events.send(Event::SessionStarted);
          }
          Response::Error {
            error_type: _,
            description,
          } => {
            error!(session = %session.id, "session start failed");
            // A failed StartSession still owns greetd's configuring slot on
            // some PAM/session-manager paths. Explicitly cancel it before
            // notifying the UI so the next login can create a fresh session.
            cancel_session(stream);
            let _ = events.send(Event::Error(description));
          }
          Response::AuthMessage { .. } => {
            // Authentication after StartSession is a protocol violation. The
            // best-effort cancellation keeps a malformed backend response
            // from poisoning the next login attempt.
            cancel_session(stream);
            let _ = events.send(Event::Error(
              "greetd requested authentication after StartSession.".into(),
            ));
          }
        }
        return Ok(WorkerState::Disconnected);
      }
      Response::Error {
        error_type,
        description,
      } => {
        // greetd documents automatic cleanup for errors, but an explicit
        // CancelSession is required for reliable recovery across PAM stacks
        // and daemon versions. Consume its response before exposing failure
        // to the UI, preventing a rapid retry from racing cleanup.
        cancel_session(stream);
        if matches!(error_type, ErrorType::AuthError) {
          info!("authentication failed");
          let _ = events.send(Event::AuthFailed(description));
        } else {
          warn!(%description, "greetd returned an error");
          let _ = events.send(Event::Error(description));
        }
        return Ok(WorkerState::Disconnected);
      }
      Response::AuthMessage {
        auth_message_type,
        auth_message,
      } => match auth_message_type {
        AuthMessageType::Secret => {
          // Returning after a prompt prevents reading past the point where
          // greetd expects user input.
          let _ = events.send(Event::AuthMessage(AuthPrompt {
            message: auth_message,
            secret: true,
          }));
          return Ok(WorkerState::Authenticating {
            stream: stream.try_clone().map_err(GreeterError::Connect)?,
            session,
            theme,
            accent,
          });
        }
        AuthMessageType::Visible => {
          let _ = events.send(Event::AuthMessage(AuthPrompt {
            message: auth_message,
            secret: false,
          }));
          return Ok(WorkerState::Authenticating {
            stream: stream.try_clone().map_err(GreeterError::Connect)?,
            session,
            theme,
            accent,
          });
        }
        AuthMessageType::Info => {
          // Non-interactive messages must be acknowledged to unblock greetd.
          let _ = events.send(Event::Info(auth_message));
          Request::PostAuthMessageResponse { response: None }.write_to(stream)?;
        }
        AuthMessageType::Error => {
          let _ = events.send(Event::Error(auth_message));
          Request::PostAuthMessageResponse { response: None }.write_to(stream)?;
        }
      },
    }
  }
}

/// Releases the session currently owned by the greetd IPC connection.
///
/// Cancellation is deliberately best-effort: the original response may have
/// already caused greetd to tear the session down, in which case the socket
/// can legitimately reject this cleanup request. Any cleanup failure is
/// logged without replacing the authentication error shown to the user.
fn cancel_session(stream: &mut UnixStream) {
  if let Err(error) = Request::CancelSession.write_to(stream) {
    warn!(%error, "could not request greetd session cancellation");
    return;
  }

  match Response::read_from(stream) {
    Ok(Response::Success) => info!("greetd session cancelled"),
    Ok(Response::Error { description, .. }) => {
      warn!(%description, "greetd rejected session cancellation");
    }
    Ok(Response::AuthMessage { .. }) => {
      warn!("greetd returned an authentication prompt while cancelling");
    }
    Err(error) => warn!(%error, "could not read greetd cancellation response"),
  }
}
