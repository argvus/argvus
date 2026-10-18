//! Implements `agent` responsibilities in crate `argvus control center bluetooth`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::{AgentEvent, AgentReply, valid_pin};
use argvus_control_center_core::sanitize::terminal_text;
use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
  mpsc::{self, Receiver, Sender, SyncSender},
};
use std::thread::JoinHandle;
use std::time::Duration;
use zbus::zvariant::ObjectPath;

/// Defines the constant `AGENT_PATH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const AGENT_PATH: &str = "/org/argvus/agent";
/// Defines the constant `BLUEZ`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const BLUEZ: &str = "org.bluez";
/// Defines the constant `AGENT_MANAGER_PATH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const AGENT_MANAGER_PATH: &str = "/org/bluez";
/// Defines the constant `AGENT_MANAGER_INTERFACE`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const AGENT_MANAGER_INTERFACE: &str = "org.bluez.AgentManager1";
/// Defines the constant `AGENT_CAPABILITY`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const AGENT_CAPABILITY: &str = "KeyboardDisplay";
/// Defines the constant `ASK_POLL_INTERVAL`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const ASK_POLL_INTERVAL: Duration = Duration::from_millis(120);
/// Defines the constant `REGISTER_TIMEOUT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const REGISTER_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.bluez.Error")]
/// Defines `AgentError`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum AgentError {
  Canceled(String),
  Rejected(String),
}

/// Represents `BlueZAgent`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
struct BlueZAgent {
  events: SyncSender<AgentEvent>,
  replies: Mutex<Receiver<AgentReply>>,
  stopped: Arc<AtomicBool>,
}

impl BlueZAgent {
  /// Executes the `ask` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn ask(&self, event: AgentEvent) -> Result<AgentReply, AgentError> {
    self
      .events
      .send(event)
      .map_err(|_| AgentError::Canceled("agent channel closed".into()))?;
    loop {
      if self.stopped.load(Ordering::Acquire) {
        return Err(AgentError::Canceled("pairing canceled".into()));
      }
      match self
        .replies
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .recv_timeout(ASK_POLL_INTERVAL)
      {
        Ok(reply) => return Ok(reply),
        Err(mpsc::RecvTimeoutError::Timeout) => {}
        Err(mpsc::RecvTimeoutError::Disconnected) => {
          return Err(AgentError::Canceled("pairing canceled".into()));
        }
      }
    }
  }
}

#[zbus::interface(name = "org.bluez.Agent1")]
impl BlueZAgent {
  /// Executes the `request_pin_code` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn request_pin_code(&self, device: ObjectPath<'_>) -> Result<String, AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::RequestPinCode(mac))? {
      AgentReply::Pin(pin) if valid_pin(&pin) => Ok(pin),
      _ => Err(AgentError::Rejected("no valid PIN provided".into())),
    }
  }
  /// Executes the `request_passkey` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn request_passkey(&self, device: ObjectPath<'_>) -> Result<u32, AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::RequestPasskey(mac))? {
      AgentReply::Passkey(passkey) => Ok(passkey),
      _ => Err(AgentError::Rejected("no valid passkey provided".into())),
    }
  }
  /// Executes the `display_passkey` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn display_passkey(
    &self,
    device: ObjectPath<'_>,
    passkey: u32,
    _entered: u16,
  ) -> Result<(), AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::DisplayPasskey(mac, passkey))? {
      AgentReply::Accept => Ok(()),
      _ => Err(AgentError::Rejected("passkey not confirmed".into())),
    }
  }
  /// Executes the `request_confirmation` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn request_confirmation(&self, device: ObjectPath<'_>, passkey: u32) -> Result<(), AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::RequestConfirmation(mac, passkey))? {
      AgentReply::Accept => Ok(()),
      _ => Err(AgentError::Rejected("confirmation denied".into())),
    }
  }
  /// Executes the `request_authorization` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn request_authorization(&self, device: ObjectPath<'_>) -> Result<(), AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::RequestAuthorization(mac))? {
      AgentReply::Accept => Ok(()),
      _ => Err(AgentError::Rejected("authorization denied".into())),
    }
  }
  /// Executes the `authorize_service` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn authorize_service(&self, device: ObjectPath<'_>, uuid: &str) -> Result<(), AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::AuthorizeService(mac, terminal_text(uuid)))? {
      AgentReply::Accept => Ok(()),
      _ => Err(AgentError::Rejected("service not authorized".into())),
    }
  }
  /// Executes the `cancel` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn cancel(&self) {
    let _ = self.events.try_send(AgentEvent::Cancelled);
  }
  /// Executes the `release` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn release(&self) {
    let _ = self.events.try_send(AgentEvent::Released);
  }
}

/// Represents `AgentHandle`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct AgentHandle {
  stop: Sender<()>,
  events: Receiver<AgentEvent>,
  replies: Sender<AgentReply>,
  ready: Arc<AtomicBool>,
  stopped: Arc<AtomicBool>,
  thread: Option<JoinHandle<()>>,
}

impl AgentHandle {
  /// Executes the `start` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn start() -> Self {
    let (events_tx, events_rx) = mpsc::sync_channel(32);
    let (replies_tx, replies_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();
    let ready = Arc::new(AtomicBool::new(false));
    let stopped = Arc::new(AtomicBool::new(false));
    let thread = std::thread::spawn({
      let ready = Arc::clone(&ready);
      let stopped = Arc::clone(&stopped);
      move || run(events_tx, replies_rx, stop_rx, ready, stopped)
    });
    Self {
      stop: stop_tx,
      events: events_rx,
      replies: replies_tx,
      ready,
      stopped,
      thread: Some(thread),
    }
  }
  /// Checks the condition represented by `is_ready` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn is_ready(&self) -> bool {
    self.ready.load(Ordering::Acquire)
  }
  /// Executes the `try_event` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn try_event(&self) -> Option<AgentEvent> {
    self.events.try_recv().ok()
  }
  /// Executes the `reply` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reply(&self, reply: AgentReply) {
    let _ = self.replies.send(reply);
  }
  /// Executes the `stop_signal` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn stop_signal(&self) {
    self.stopped.store(true, Ordering::Release);
    let _ = self.stop.send(());
  }
  /// Executes the `unregister` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn unregister(&mut self) {
    if let Some(thread) = self.thread.take() {
      let _ = self.replies.send(AgentReply::Reject);
      self.stop_signal();
      let _ = thread.join();
    }
  }
}

impl Drop for AgentHandle {
  /// Executes the `drop` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn drop(&mut self) {
    if self.thread.is_some() {
      self.unregister();
    }
  }
}

/// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run(
  events: SyncSender<AgentEvent>,
  replies: Receiver<AgentReply>,
  stop: Receiver<()>,
  ready: Arc<AtomicBool>,
  stopped: Arc<AtomicBool>,
) {
  let outcome: zbus::Result<()> = (|| {
    let connection = zbus::blocking::connection::Builder::system()?
      .method_timeout(REGISTER_TIMEOUT)
      .build()?;
    let agent = BlueZAgent {
      events,
      replies: Mutex::new(replies),
      stopped: Arc::clone(&stopped),
    };
    let path = ObjectPath::try_from(AGENT_PATH)?;
    connection.object_server().at(path.clone(), agent)?;
    connection.call_method(
      Some(BLUEZ),
      AGENT_MANAGER_PATH,
      Some(AGENT_MANAGER_INTERFACE),
      "RegisterAgent",
      &(path.clone(), AGENT_CAPABILITY),
    )?;
    connection.call_method(
      Some(BLUEZ),
      AGENT_MANAGER_PATH,
      Some(AGENT_MANAGER_INTERFACE),
      "RequestDefaultAgent",
      &(path.clone(),),
    )?;
    ready.store(true, Ordering::Release);
    loop {
      if stop.recv_timeout(Duration::from_millis(300)).is_ok() {
        break;
      }
      if connection.is_closed() {
        stopped.store(true, Ordering::Release);
        break;
      }
    }
    let _ = connection.call_method(
      Some(BLUEZ),
      AGENT_MANAGER_PATH,
      Some(AGENT_MANAGER_INTERFACE),
      "UnregisterAgent",
      &(path,),
    );
    let _ = connection.close();
    Ok(())
  })();
  let _ = outcome;
}

/// Executes the `device_mac` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn device_mac(path: &ObjectPath<'_>) -> Result<String, AgentError> {
  let mac = path
    .as_str()
    .rsplit('/')
    .next()
    .and_then(|segment| segment.strip_prefix("dev_"))
    .map(|mac| mac.replace('_', ":"))
    .unwrap_or_default();
  if crate::backend::validate_address(&mac).is_err() {
    return Err(AgentError::Rejected("unknown device".into()));
  }
  Ok(mac)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::valid_passkey;
  #[test]
  /// Executes the `extracts_mac_from_bluez_device_path` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn extracts_mac_from_bluez_device_path() {
    assert_eq!(
      device_mac(&ObjectPath::try_from("/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF").unwrap()).unwrap(),
      "AA:BB:CC:DD:EE:FF"
    );
    assert!(device_mac(&ObjectPath::try_from("/org/bluez/hci0").unwrap()).is_err());
  }
  #[test]
  /// Executes the `passkey_and_pin_replies_are_validated` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn passkey_and_pin_replies_are_validated() {
    assert!(valid_pin("1234"));
    assert!(valid_passkey("123456"));
    assert!(!valid_passkey("1234567"));
  }
}
