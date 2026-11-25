use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::{pending, poll_fn};
use std::rc::Rc;
use std::task::Poll;
use std::time::{Duration, Instant};

use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib::{self, Variant, translate::ToGlibPtr, variant::FromVariant};

const DBUS: &str = "org.freedesktop.DBus";
const DBUS_PATH: &str = "/org/freedesktop/DBus";
const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";
const DUNST: &str = "org.dunstproject.cmd0";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const CALL_TIMEOUT_MS: i32 = 2_000;
const MAX_BACKOFF: u64 = 30;

type Publish = Rc<dyn Fn(Option<bool>)>;

pub(super) async fn watch(publish: impl Fn(Option<bool>) + 'static) {
  let publish: Publish = Rc::new(publish);
  let mut backoff = 1;
  loop {
    if let Ok(connection) = connect(gio::BusType::Session).await {
      let connected_at = Instant::now();
      let recovered = Rc::new(Cell::new(false));
      watch_connection(connection, {
        let recovered = recovered.clone();
        let publish = publish.clone();
        Rc::new(move |paused| {
          if paused.is_some() {
            recovered.set(true);
          }
          publish(paused);
        })
      })
      .await;
      if recovered.get() || connected_at.elapsed() >= Duration::from_secs(MAX_BACKOFF) {
        backoff = 1;
      }
    }
    publish(None);
    wait_to_retry(&mut backoff).await;
  }
}

async fn wait_to_retry(backoff: &mut u64) {
  glib::timeout_future(Duration::from_secs(*backoff)).await;
  *backoff = (*backoff * 2).min(MAX_BACKOFF);
}

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

async fn name_owner(connection: &gio::DBusConnection) -> Result<String, glib::Error> {
  let response = call(
    connection,
    DBUS,
    DBUS_PATH,
    DBUS,
    "GetNameOwner",
    Some(&(NAME,).to_variant()),
  )
  .await?;
  Ok(decode::<(String,)>(&response)?.0)
}

async fn until_closed(connection: &gio::DBusConnection) {
  let closed = gio::Cancellable::new();
  let handler = connection.connect_closed({
    let closed = closed.clone();
    move |_, _, _| closed.cancel()
  });
  if !connection.is_closed() {
    let _ = gio::CancellableFuture::new(pending::<()>(), closed).await;
  }
  connection.disconnect(handler);
}

struct ConnectionWatch {
  connection: gio::DBusConnection,
  owner: RefCell<Option<Rc<Owner>>>,
  publish: Publish,
  owner_changed: Cell<bool>,
}

impl ConnectionWatch {
  fn set_owner(&self, name: &str) {
    *self.owner.borrow_mut() = None;
    if name.is_empty() {
      (self.publish)(None);
    } else {
      *self.owner.borrow_mut() = Some(Owner::new(&self.connection, name, self.publish.clone()));
    }
  }
}

async fn watch_connection(connection: gio::DBusConnection, publish: Publish) {
  let watch = Rc::new(ConnectionWatch {
    connection,
    owner: RefCell::new(None),
    publish,
    owner_changed: Cell::new(false),
  });
  let _names = watch.connection.subscribe_to_signal(
    Some(DBUS),
    Some(DBUS),
    Some("NameOwnerChanged"),
    Some(DBUS_PATH),
    Some(NAME),
    gio::DBusSignalFlags::NONE,
    {
      let watch = Rc::downgrade(&watch);
      move |signal| {
        if let (Some(watch), Some((_, _, new_name))) = (
          watch.upgrade(),
          signal.parameters.get::<(String, String, String)>(),
        ) {
          watch.owner_changed.set(true);
          watch.set_owner(&new_name);
        }
      }
    },
  );
  let result = name_owner(&watch.connection).await;
  if !watch.owner_changed.get() {
    match result {
      Ok(name) => watch.set_owner(&name),
      Err(error) if error.matches(gio::DBusError::NameHasNoOwner) => watch.set_owner(""),
      Err(_) => return,
    }
  }
  let resume =
    glib::MainContext::ref_thread_default().spawn_local(watch_resume(Rc::downgrade(&watch)));
  until_closed(&watch.connection).await;
  resume.abort();
}

async fn watch_resume(watch: std::rc::Weak<ConnectionWatch>) {
  // A session bus that survives sleep retains its subscriptions. If logind is
  // available, also reconcile on wake in case an event was lost during sleep.
  let Ok(connection) = connect(gio::BusType::System).await else {
    return;
  };
  let _resume = connection.subscribe_to_signal(
    Some("org.freedesktop.login1"),
    Some("org.freedesktop.login1.Manager"),
    Some("PrepareForSleep"),
    Some("/org/freedesktop/login1"),
    None,
    gio::DBusSignalFlags::NONE,
    move |signal| {
      if signal.parameters.get::<(bool,)>() == Some((false,))
        && let Some(watch) = watch.upgrade()
        && let Some(owner) = watch.owner.borrow().as_ref()
      {
        owner.refresh();
      }
    },
  );
  until_closed(&connection).await;
}

struct Owner {
  connection: gio::DBusConnection,
  name: String,
  publish: Publish,
  validated: Cell<bool>,
  refresh_needed: Cell<bool>,
  task: RefCell<Option<glib::JoinHandle<()>>>,
  subscription: RefCell<Option<gio::SignalSubscription>>,
}

impl Owner {
  fn new(connection: &gio::DBusConnection, name: &str, publish: Publish) -> Rc<Self> {
    let owner = Rc::new(Self {
      connection: connection.clone(),
      name: name.to_owned(),
      publish,
      validated: Cell::new(false),
      refresh_needed: Cell::new(false),
      task: RefCell::new(None),
      subscription: RefCell::new(None),
    });
    *owner.subscription.borrow_mut() = Some(connection.subscribe_to_signal(
      Some(name),
      Some(PROPERTIES),
      Some("PropertiesChanged"),
      Some(PATH),
      Some(DUNST),
      gio::DBusSignalFlags::NONE,
      {
        let owner = Rc::downgrade(&owner);
        move |signal| {
          if let Some(owner) = owner.upgrade() {
            owner.property_changed(signal.parameters);
          }
        }
      },
    ));
    owner.refresh();
    owner
  }

  fn property_changed(self: &Rc<Self>, parameters: &Variant) {
    let Some((_, changed, invalidated)) =
      parameters.get::<(String, HashMap<String, Variant>, Vec<String>)>()
    else {
      self.refresh();
      return;
    };
    if let Some(value) = changed.get("paused") {
      if self.validated.get()
        && self.task.borrow().is_none()
        && let Some(paused) = value.get::<bool>()
      {
        (self.publish)(Some(paused));
      } else {
        self.refresh();
      }
    } else if invalidated.iter().any(|property| property == "paused") {
      self.refresh();
    }
  }

  fn refresh(self: &Rc<Self>) {
    self.refresh_needed.set(true);
    if self.task.borrow().is_some() {
      return;
    }
    let owner = Rc::downgrade(self);
    let connection = self.connection.clone();
    let name = self.name.clone();
    *self.task.borrow_mut() = Some(glib::MainContext::ref_thread_default().spawn_local(
      async move {
        let mut backoff = 1;
        while let Some(validated) = owner.upgrade().map(|owner| {
          owner.refresh_needed.set(false);
          owner.validated.get()
        }) {
          let result = read_snapshot(&connection, &name, validated).await;
          let Some(current) = owner.upgrade() else {
            return;
          };
          match result {
            Ok(Some(paused)) => {
              current.validated.set(true);
              if current.refresh_needed.get() {
                continue;
              }
              (current.publish)(Some(paused));
            }
            Ok(None) => (current.publish)(None),
            Err(_) => {
              (current.publish)(None);
              drop(current);
              wait_to_retry(&mut backoff).await;
              continue;
            }
          }
          current.task.take();
          break;
        }
      },
    ));
  }
}

impl Drop for Owner {
  fn drop(&mut self) {
    if let Some(task) = self.task.get_mut().take() {
      task.abort();
    }
  }
}

async fn read_snapshot(
  connection: &gio::DBusConnection,
  owner: &str,
  validated: bool,
) -> Result<Option<bool>, glib::Error> {
  if !validated && !supports_events(connection, owner).await? {
    return Ok(None);
  }
  let reply = call(
    connection,
    owner,
    PATH,
    PROPERTIES,
    "Get",
    Some(&(DUNST, "paused").to_variant()),
  )
  .await?;
  let (value,): (Variant,) = decode(&reply)?;
  let paused = decode::<bool>(&value)?;
  // Confirm the snapshot still belongs to the current server, even if GLib
  // has not dispatched the queued owner-change callback yet.
  Ok((name_owner(connection).await? == owner).then_some(paused))
}

async fn supports_events(
  connection: &gio::DBusConnection,
  owner: &str,
) -> Result<bool, glib::Error> {
  let info = call(connection, owner, PATH, NAME, "GetServerInformation", None).await?;
  let (name, _, version, _) = decode::<(String, String, String, String)>(&info)?;
  if name != "dunst" || !supported_version(&version) {
    return Ok(false);
  }
  let xml = call(
    connection,
    owner,
    PATH,
    "org.freedesktop.DBus.Introspectable",
    "Introspect",
    None,
  )
  .await?;
  let (xml,) = decode::<(String,)>(&xml)?;
  let info = gio::DBusNodeInfo::for_xml(&xml)?;
  let Some(interface) = info.lookup_interface(DUNST) else {
    return Ok(false);
  };
  let Some(property) = interface.lookup_property("paused") else {
    return Ok(false);
  };

  // SAFETY: the shared GIO objects retain GDBus introspection strings and
  // annotations for the lifetime of these borrows. Gio has no typed getters.
  unsafe {
    let definition: *mut gio::ffi::GDBusPropertyInfo = property.to_glib_none().0;
    let declared: *mut gio::ffi::GDBusInterfaceInfo = interface.to_glib_none().0;
    let key = c"org.freedesktop.DBus.Property.EmitsChangedSignal";
    let mut emits =
      gio::ffi::g_dbus_annotation_info_lookup((*definition).annotations, key.as_ptr());
    if emits.is_null() {
      emits = gio::ffi::g_dbus_annotation_info_lookup((*declared).annotations, key.as_ptr());
    }
    Ok(
      std::ffi::CStr::from_ptr((*definition).signature) == c"b"
        && (*definition).flags & gio::ffi::G_DBUS_PROPERTY_INFO_FLAGS_READABLE != 0
        && (emits.is_null()
          || matches!(
            std::ffi::CStr::from_ptr(emits).to_bytes(),
            b"true" | b"invalidates"
          )),
    )
  }
}

fn supported_version(version: &str) -> bool {
  let Some(version) = version.split_whitespace().next() else {
    return false;
  };
  let mut parts = version.split('.').map(str::parse::<u32>);
  match (parts.next(), parts.next(), parts.next(), parts.next()) {
    (Some(Ok(major)), Some(Ok(minor)), Some(Ok(patch)), None) => {
      (major, minor, patch) >= (1, 13, 2)
    }
    _ => false,
  }
}
