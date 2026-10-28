//! BlueZ pairing agent that forwards PIN, passkey and confirmation requests to the Bluetooth page.
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

pub const AGENT_PATH: &str = "/org/argvus/agent";
const BLUEZ: &str = "org.bluez";
const AGENT_MANAGER_PATH: &str = "/org/bluez";
const AGENT_MANAGER_INTERFACE: &str = "org.bluez.AgentManager1";
const AGENT_CAPABILITY: &str = "KeyboardDisplay";
const ASK_POLL_INTERVAL: Duration = Duration::from_millis(120);
const REGISTER_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.bluez.Error")]
pub enum AgentError {
  Canceled(String),
  Rejected(String),
}

struct BlueZAgent {
  events: SyncSender<AgentEvent>,
  replies: Mutex<Receiver<AgentReply>>,
  stopped: Arc<AtomicBool>,
}

impl BlueZAgent {
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
  fn request_pin_code(&self, device: ObjectPath<'_>) -> Result<String, AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::RequestPinCode(mac))? {
      AgentReply::Pin(pin) if valid_pin(&pin) => Ok(pin),
      _ => Err(AgentError::Rejected("no valid PIN provided".into())),
    }
  }
  fn request_passkey(&self, device: ObjectPath<'_>) -> Result<u32, AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::RequestPasskey(mac))? {
      AgentReply::Passkey(passkey) => Ok(passkey),
      _ => Err(AgentError::Rejected("no valid passkey provided".into())),
    }
  }
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
  fn request_confirmation(&self, device: ObjectPath<'_>, passkey: u32) -> Result<(), AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::RequestConfirmation(mac, passkey))? {
      AgentReply::Accept => Ok(()),
      _ => Err(AgentError::Rejected("confirmation denied".into())),
    }
  }
  fn request_authorization(&self, device: ObjectPath<'_>) -> Result<(), AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::RequestAuthorization(mac))? {
      AgentReply::Accept => Ok(()),
      _ => Err(AgentError::Rejected("authorization denied".into())),
    }
  }
  fn authorize_service(&self, device: ObjectPath<'_>, uuid: &str) -> Result<(), AgentError> {
    let mac = device_mac(&device)?;
    match self.ask(AgentEvent::AuthorizeService(mac, terminal_text(uuid)))? {
      AgentReply::Accept => Ok(()),
      _ => Err(AgentError::Rejected("service not authorized".into())),
    }
  }
  fn cancel(&self) {
    let _ = self.events.try_send(AgentEvent::Cancelled);
  }
  fn release(&self) {
    let _ = self.events.try_send(AgentEvent::Released);
  }
}

pub struct AgentHandle {
  stop: Sender<()>,
  events: Receiver<AgentEvent>,
  replies: Sender<AgentReply>,
  ready: Arc<AtomicBool>,
  stopped: Arc<AtomicBool>,
  thread: Option<JoinHandle<()>>,
}

impl AgentHandle {
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
  pub fn is_ready(&self) -> bool {
    self.ready.load(Ordering::Acquire)
  }
  pub fn try_event(&self) -> Option<AgentEvent> {
    self.events.try_recv().ok()
  }
  pub fn reply(&self, reply: AgentReply) {
    let _ = self.replies.send(reply);
  }
  fn stop_signal(&self) {
    self.stopped.store(true, Ordering::Release);
    let _ = self.stop.send(());
  }
  pub fn unregister(&mut self) {
    if let Some(thread) = self.thread.take() {
      let _ = self.replies.send(AgentReply::Reject);
      self.stop_signal();
      let _ = thread.join();
    }
  }
}

impl Drop for AgentHandle {
  fn drop(&mut self) {
    if self.thread.is_some() {
      self.unregister();
    }
  }
}

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
  fn extracts_mac_from_bluez_device_path() {
    assert_eq!(
      device_mac(&ObjectPath::try_from("/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF").unwrap()).unwrap(),
      "AA:BB:CC:DD:EE:FF"
    );
    assert!(device_mac(&ObjectPath::try_from("/org/bluez/hci0").unwrap()).is_err());
  }
  #[test]
  fn passkey_and_pin_replies_are_validated() {
    assert!(valid_pin("1234"));
    assert!(valid_passkey("123456"));
    assert!(!valid_passkey("1234567"));
  }
}
