use std::cell::{Cell, RefCell};
use std::future::{pending, poll_fn};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::task::Poll;
use std::time::Duration;

use gtk::gio::{self, prelude::*};
use gtk::glib::{
  self, Variant,
  variant::{FromVariant, ObjectPath},
};

use super::{RecordingState, marker};

const DBUS: &str = "org.freedesktop.DBus";
const DBUS_PATH: &str = "/org/freedesktop/DBus";
const NAME: &str = "org.freedesktop.systemd1";
const PATH: &str = "/org/freedesktop/systemd1";
const MANAGER: &str = "org.freedesktop.systemd1.Manager";
const UNIT: &str = "org.freedesktop.systemd1.Unit";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const UNIT_NAME: &str = "argvus-screen-recorder.service";
const UNIT_PATH: &str = "/org/freedesktop/systemd1/unit/argvus_2dscreen_2drecorder_2eservice";
const CALL_TIMEOUT_MS: i32 = 2_000;

async fn connect(bus: gio::BusType) -> Result<gio::DBusConnection, glib::Error> {
  let address = gio::dbus_address_get_for_bus_sync(bus, gio::Cancellable::NONE)?;
  let mut connection = gio::DBusConnection::for_address_future(
    &address,
    gio::DBusConnectionFlags::AUTHENTICATION_CLIENT
      | gio::DBusConnectionFlags::MESSAGE_BUS_CONNECTION,
    None,
  );
  let mut timeout = glib::timeout_future(Duration::from_millis(CALL_TIMEOUT_MS as u64));
  poll_fn(|context| {
    if let Poll::Ready(result) = connection.as_mut().poll(context) {
      return Poll::Ready(result);
    }
    timeout.as_mut().poll(context).map(|()| {
      Err(glib::Error::new(
        gio::IOErrorEnum::TimedOut,
        "D-Bus handshake timed out",
      ))
    })
  })
  .await
  .inspect(|connection| {
    connection.set_exit_on_close(false);
  })
}

fn decode<T: FromVariant>(value: &Variant) -> Result<T, glib::Error> {
  value
    .try_get::<T>()
    .map_err(|error| glib::Error::new(gio::IOErrorEnum::InvalidData, &error.to_string()))
}
async fn call(
  connection: &gio::DBusConnection,
  destination: &str,
  path: &str,
  interface: &str,
  method: &str,
  args: Option<&Variant>,
) -> Result<Variant, glib::Error> {
  connection
    .call_future(
      Some(destination),
      path,
      interface,
      method,
      args,
      None,
      gio::DBusCallFlags::NO_AUTO_START,
      CALL_TIMEOUT_MS,
    )
    .await
}

#[derive(Default)]
struct Events {
  generation: Cell<u64>,
  resume: Cell<bool>,
  waiter: RefCell<Option<gio::Cancellable>>,
}

impl Events {
  fn invalidate(&self) {
    self.generation.set(self.generation.get().wrapping_add(1));
    if let Some(waiter) = self.waiter.borrow().as_ref() {
      waiter.cancel();
    }
  }

  async fn wait(&self, generation: u64, closed: &gio::Cancellable) {
    let wake = gio::Cancellable::new();
    *self.waiter.borrow_mut() = Some(wake.clone());
    let close_handler = closed.connect_cancelled({
      let wake = wake.clone();
      move |_| wake.cancel()
    });
    if self.generation.get() == generation {
      let _ = gio::CancellableFuture::new(pending::<()>(), wake).await;
    }
    if let Some(handler) = close_handler {
      closed.disconnect_cancelled(handler);
    }
    self.waiter.borrow_mut().take();
  }
}

async fn current_owner(connection: &gio::DBusConnection) -> Result<String, glib::Error> {
  let reply = call(
    connection,
    DBUS,
    DBUS_PATH,
    DBUS,
    "GetNameOwner",
    Some(&(NAME,).to_variant()),
  )
  .await?;
  Ok(decode::<(String,)>(&reply)?.0)
}

fn unexpected(message: &str) -> glib::Error {
  glib::Error::new(gio::IOErrorEnum::InvalidData, message)
}

async fn active_state(connection: &gio::DBusConnection, owner: &str) -> Result<bool, glib::Error> {
  for attempt in 0..2 {
    let unit = call(
      connection,
      owner,
      PATH,
      MANAGER,
      "GetUnit",
      Some(&(UNIT_NAME,).to_variant()),
    )
    .await;
    let reply = match unit {
      Ok(reply) => reply,
      Err(error)
        if gio::DBusError::remote_error(&error).as_deref()
          == Some("org.freedesktop.systemd1.NoSuchUnit") =>
      {
        return Ok(false);
      }
      Err(error) => return Err(error),
    };
    let path = decode::<(ObjectPath,)>(&reply)?.0;
    if path.as_str() != UNIT_PATH {
      return Err(unexpected("unexpected recorder object path"));
    }
    let property = call(
      connection,
      owner,
      &path,
      PROPERTIES,
      "Get",
      Some(&(UNIT, "ActiveState").to_variant()),
    )
    .await;
    let reply = match property {
      Ok(reply) => reply,
      Err(error)
        if attempt == 0
          && matches!(
            gio::DBusError::remote_error(&error).as_deref(),
            Some(
              "org.freedesktop.DBus.Error.UnknownObject" | "org.freedesktop.systemd1.NoSuchUnit"
            )
          ) =>
      {
        continue;
      }
      Err(error) => return Err(error),
    };
    let state = decode::<(Variant,)>(&reply)?.0;
    return match decode::<String>(&state)?.as_str() {
      "active" | "reloading" | "refreshing" => Ok(true),
      "inactive" | "failed" | "activating" | "deactivating" | "maintenance" => Ok(false),
      _ => Err(unexpected("unknown systemd ActiveState")),
    };
  }
  unreachable!("the second attempt returns")
}

async fn snapshot(
  connection: &gio::DBusConnection,
  owner: &str,
  path: &Path,
) -> Result<RecordingState, glib::Error> {
  let active = active_state(connection, owner).await?;
  let contents = if active {
    marker::read(path).map_err(|error| unexpected(&error.to_string()))?
  } else {
    Vec::new()
  };
  Ok(RecordingState::from_sources(active, &contents))
}

async fn watch_connection(
  connection: &gio::DBusConnection,
  events: &Rc<Events>,
  marker_path: &Path,
  publish: &impl Fn(Option<RecordingState>),
  recovered: &mut bool,
) {
  let closed = gio::Cancellable::new();
  let handler = connection.connect_closed({
    let closed = closed.clone();
    move |_, _, _| closed.cancel()
  });
  let close_guard = ConnectionHandler(connection, Some(handler));
  let replaced = Rc::new(Cell::new(false));
  let _names = connection.subscribe_to_signal(
    Some(DBUS),
    Some(DBUS),
    Some("NameOwnerChanged"),
    Some(DBUS_PATH),
    Some(NAME),
    gio::DBusSignalFlags::NONE,
    {
      let replaced = replaced.clone();
      let events = events.clone();
      move |signal| {
        if let Some((name, _, _)) = signal.parameters.get::<(String, String, String)>()
          && name == NAME
        {
          replaced.set(true);
          events.invalidate();
        }
      }
    },
  );
  let Ok(owner) = current_owner(connection).await else {
    return;
  };
  let _unit_signals = ["UnitNew", "UnitRemoved"].map(|signal_name| {
    let events = events.clone();
    connection.subscribe_to_signal(
      Some(&owner),
      Some(MANAGER),
      Some(signal_name),
      Some(PATH),
      Some(UNIT_NAME),
      gio::DBusSignalFlags::NONE,
      move |signal| {
        if let Some((name, path)) = signal.parameters.get::<(String, ObjectPath)>()
          && name == UNIT_NAME
          && path.as_str() == UNIT_PATH
        {
          events.invalidate();
        }
      },
    )
  });
  let _reload = connection.subscribe_to_signal(
    Some(&owner),
    Some(MANAGER),
    Some("Reloading"),
    Some(PATH),
    None,
    gio::DBusSignalFlags::NONE,
    {
      let events = events.clone();
      move |signal| {
        if signal.parameters.get::<(bool,)>() == Some((false,)) {
          events.invalidate();
        }
      }
    },
  );
  let _properties = connection.subscribe_to_signal(
    Some(&owner),
    Some(PROPERTIES),
    Some("PropertiesChanged"),
    Some(UNIT_PATH),
    Some(UNIT),
    gio::DBusSignalFlags::NONE,
    {
      let events = events.clone();
      move |signal| {
        type Properties = (
          String,
          std::collections::HashMap<String, Variant>,
          Vec<String>,
        );
        if let Some((interface, changed, invalidated)) = signal.parameters.get::<Properties>()
          && interface == UNIT
          && (changed.contains_key("ActiveState")
            || invalidated.iter().any(|name| name == "ActiveState"))
        {
          events.invalidate();
        }
      }
    },
  );
  if call(connection, &owner, PATH, MANAGER, "Subscribe", None)
    .await
    .is_err()
    || current_owner(connection).await.as_deref() != Ok(owner.as_str())
  {
    return;
  }
  let marker_event: Rc<dyn Fn()> = Rc::new({
    let events = events.clone();
    move || events.invalidate()
  });
  let mut watches = marker::Watches::default();
  let mut retry = 1;
  while !connection.is_closed() && !replaced.get() {
    let generation = events.generation.get();
    if events.resume.replace(false) {
      watches = marker::Watches::default();
    }
    let state = if watches.refresh(marker_path, &marker_event) {
      snapshot(connection, &owner, marker_path).await.ok()
    } else {
      None
    };
    if connection.is_closed() || replaced.get() {
      break;
    }
    if events.generation.get() != generation {
      continue;
    }
    publish(state);
    if state.is_some() {
      *recovered = true;
      retry = 1;
      events.wait(generation, &closed).await;
    } else {
      backoff(&mut retry).await;
    }
  }
  drop(close_guard);
}

struct ConnectionHandler<'a>(&'a gio::DBusConnection, Option<glib::SignalHandlerId>);

impl Drop for ConnectionHandler<'_> {
  fn drop(&mut self) {
    if let Some(handler) = self.1.take() {
      self.0.disconnect(handler);
    }
  }
}

async fn backoff(retry: &mut u64) {
  glib::timeout_future(Duration::from_secs(*retry)).await;
  *retry = (*retry * 2).min(30);
}

async fn watch_resume(events: Rc<Events>) {
  let mut retry = 1;
  loop {
    if let Ok(connection) = connect(gio::BusType::System).await {
      let _resume = connection.subscribe_to_signal(
        Some("org.freedesktop.login1"),
        Some("org.freedesktop.login1.Manager"),
        Some("PrepareForSleep"),
        Some("/org/freedesktop/login1"),
        None,
        gio::DBusSignalFlags::NONE,
        {
          let events = events.clone();
          move |signal| {
            if signal.parameters.get::<(bool,)>() == Some((false,)) {
              events.resume.set(true);
              events.invalidate();
            }
          }
        },
      );
      let _owner = connection.subscribe_to_signal(
        Some(DBUS),
        Some(DBUS),
        Some("NameOwnerChanged"),
        Some(DBUS_PATH),
        Some("org.freedesktop.login1"),
        gio::DBusSignalFlags::NONE,
        {
          let events = events.clone();
          move |_| {
            events.resume.set(true);
            events.invalidate();
          }
        },
      );
      events.resume.set(true);
      events.invalidate();
      let closed = gio::Cancellable::new();
      let handler = connection.connect_closed({
        let closed = closed.clone();
        move |_, _, _| closed.cancel()
      });
      let _guard = ConnectionHandler(&connection, Some(handler));
      if !connection.is_closed() {
        let _ = gio::CancellableFuture::new(pending::<()>(), closed).await;
      }
    }
    backoff(&mut retry).await;
  }
}

pub(super) async fn watch(path: PathBuf, publish: impl Fn(Option<RecordingState>)) {
  let events = Rc::new(Events::default());
  let resume = glib::MainContext::ref_thread_default().spawn_local(watch_resume(events.clone()));
  // Detach only while this future lives.
  struct Abort(glib::JoinHandle<()>);
  impl Drop for Abort {
    fn drop(&mut self) {
      self.0.abort();
    }
  }
  let _resume = Abort(resume);
  let mut retry = 1;
  loop {
    let mut recovered = false;
    if let Ok(connection) = connect(gio::BusType::Session).await {
      watch_connection(&connection, &events, &path, &publish, &mut recovered).await;
      let _ = connection.close_future().await;
    }
    publish(None);
    if recovered {
      retry = 1;
    }
    backoff(&mut retry).await;
  }
}
