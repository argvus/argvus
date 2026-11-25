//! Run with `cargo test -p argvus-cli --test notifications_taskbar -- --ignored`
//! on an ARGVUS installation. The bus, fixture and catalogs are private.

use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib::{self, Variant};
use serde_json::{Value, json};

const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";
const DUNST: &str = "org.dunstproject.cmd0";
const PROPS: &str = "org.freedesktop.DBus.Properties";
const CONTROL: &str = "org.argvus.TestNotifications";
const XML: &str = r#"<node>
  <interface name="org.freedesktop.Notifications">
    <method name="GetServerInformation">
      <arg type="s" direction="out"/><arg type="s" direction="out"/>
      <arg type="s" direction="out"/><arg type="s" direction="out"/>
    </method>
  </interface>
  <interface name="org.dunstproject.cmd0">
    <property name="paused" type="b" access="read"/>
  </interface>
  <interface name="org.argvus.TestNotifications">
    <method name="Apply">
      <arg type="s" direction="in"/><arg type="b" direction="in"/>
      <arg type="u" direction="out"/>
    </method>
  </interface>
</node>"#;

struct Process(Child);

impl Drop for Process {
  fn drop(&mut self) {
    let _ = self.0.kill();
    let _ = self.0.wait();
  }
}

fn bus(address: &str) -> Process {
  let path = Path::new(address.strip_prefix("unix:path=").unwrap());
  let config = path.with_extension("conf");
  fs::write(&config, format!(r#"<busconfig>
  <type>session</type><listen>{address}</listen>
  <policy context="default"><allow send_destination="*"/><allow receive_sender="*"/><allow own="*"/></policy>
</busconfig>"#)).unwrap();
  let mut child = Command::new("dbus-daemon")
    .args(["--nofork", "--print-address", "--config-file"])
    .arg(config)
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
  let mut ready = String::new();
  BufReader::new(child.stdout.take().unwrap())
    .read_line(&mut ready)
    .unwrap();
  assert!(ready.starts_with(address));
  Process(child)
}

fn connection(address: &str) -> gio::DBusConnection {
  gio::DBusConnection::for_address_sync(
    address,
    gio::DBusConnectionFlags::AUTHENTICATION_CLIENT
      | gio::DBusConnectionFlags::MESSAGE_BUS_CONNECTION,
    None,
    gio::Cancellable::NONE,
  )
  .unwrap()
}

fn signal(
  connection: &gio::DBusConnection,
  path: &str,
  interface: &str,
  change: &str,
  paused: bool,
) {
  let (changed, invalidated): (HashMap<&str, Variant>, Vec<&str>) = match change {
    "invalidated" => (HashMap::new(), vec!["paused"]),
    "count" => (
      HashMap::from([("historyLength", 10u32.to_variant())]),
      vec![],
    ),
    _ => (HashMap::from([("paused", paused.to_variant())]), vec![]),
  };
  connection
    .emit_signal(
      None,
      path,
      PROPS,
      "PropertiesChanged",
      Some(&(interface, changed, invalidated).to_variant()),
    )
    .unwrap();
}

fn fixture(address: &str, paused: bool, race: bool, emits_change: bool) -> String {
  let (ready, result) = mpsc::sync_channel(1);
  let address = address.to_owned();
  thread::spawn(move || {
    let context = glib::MainContext::new();
    context.with_thread_default(|| {
      let loop_ = glib::MainLoop::new(Some(&context), false);
      let connection = connection(&address);
      connection.connect_closed({
        let loop_ = loop_.clone();
        move |_, _, _| loop_.quit()
      });
      let xml = XML.replace("access=\"read\"/>", &format!(
        "access=\"read\"><annotation name=\"org.freedesktop.DBus.Property.EmitsChangedSignal\" value=\"{emits_change}\"/></property>"
      ));
      let info = gio::DBusNodeInfo::for_xml(&xml).unwrap();
      let paused = Rc::new(Cell::new(paused));
      let reads = Rc::new(Cell::new(0u32));
      let _server = connection.register_object(PATH, &info.lookup_interface(NAME).unwrap())
        .method_call(|_, _, _, _, _, _, invocation| {
          invocation.return_value(Some(&("dunst", "test", "1.13.2", "1.2").to_variant()));
        }).build().unwrap();
      let _property = connection.register_object(PATH, &info.lookup_interface(DUNST).unwrap())
        .property({
          let paused = paused.clone();
          let reads = reads.clone();
          let race = Cell::new(race);
          move |connection, _, _, _, _| {
            reads.set(reads.get() + 1);
            let initial = paused.get();
            if race.replace(false) {
              paused.set(!initial);
              signal(&connection, PATH, DUNST, "changed", !initial);
            }
            initial.to_variant()
          }
        }).build().unwrap();
      let _control = connection.register_object(PATH, &info.lookup_interface(CONTROL).unwrap())
        .method_call(move |connection, _, _, _, _, args, invocation| {
          let (action, next) = args.get::<(String, bool)>().unwrap();
          match action.as_str() {
            "own" | "own-logind" | "release" => {
              let (method, args) = if action != "release" {
                let name = if action == "own-logind" { "org.freedesktop.login1" } else { NAME };
                ("RequestName", (name, 0u32).to_variant())
              } else {
                ("ReleaseName", (NAME,).to_variant())
              };
              connection.call_sync(
                Some("org.freedesktop.DBus"), "/org/freedesktop/DBus", "org.freedesktop.DBus",
                method, Some(&args), None, gio::DBusCallFlags::NONE, 2_000, gio::Cancellable::NONE,
              ).unwrap();
            }
            "reads" => {}
            "silent" => paused.set(next),
            "resume" => connection.emit_signal(
              None, "/org/freedesktop/login1", "org.freedesktop.login1.Manager",
              "PrepareForSleep", Some(&(false,).to_variant()),
            ).unwrap(),
            "wrong-path" => signal(&connection, "/org/argvus/Elsewhere", DUNST, "changed", next),
            "wrong-interface" => signal(&connection, PATH, CONTROL, "changed", next),
            "foreign" | "count" => signal(&connection, PATH, DUNST, &action, next),
            _ => {
              paused.set(next);
              signal(&connection, PATH, DUNST, &action, next);
            }
          }
          invocation.return_value(Some(&(reads.get(),).to_variant()));
        }).build().unwrap();
      ready.send(connection.unique_name().unwrap().to_string()).unwrap();
      loop_.run();
    }).unwrap();
  });
  result.recv_timeout(Duration::from_secs(3)).unwrap()
}

fn control(connection: &gio::DBusConnection, destination: &str, action: &str, paused: bool) -> u32 {
  connection
    .call_sync(
      Some(destination),
      PATH,
      CONTROL,
      "Apply",
      Some(&(action, paused).to_variant()),
      None,
      gio::DBusCallFlags::NONE,
      2_000,
      gio::Cancellable::NONE,
    )
    .unwrap()
    .get::<(u32,)>()
    .unwrap()
    .0
}

fn data_ready(reader: &BufReader<ChildStdout>, timeout: i32) -> bool {
  if !reader.buffer().is_empty() {
    return true;
  }
  let mut fd = libc::pollfd {
    fd: reader.get_ref().as_raw_fd(),
    events: libc::POLLIN,
    revents: 0,
  };
  // SAFETY: poll has exclusive access to one initialized descriptor.
  unsafe { libc::poll(&mut fd, 1, timeout) == 1 }
}

fn read(reader: &mut BufReader<ChildStdout>) -> Value {
  assert!(
    data_ready(reader, 6_000),
    "provider did not publish a status"
  );
  let mut line = String::new();
  assert!(reader.read_line(&mut line).unwrap() != 0, "provider exited");
  serde_json::from_str(&line).unwrap()
}

fn replace(path: &Path, text: &str) {
  let pending = path.with_extension("pending");
  fs::write(&pending, text).unwrap();
  fs::rename(pending, path).unwrap();
}

fn catalog(root: &Path, locale: &str) -> PathBuf {
  root.join("catalogs").join(locale).join("taskbar.json")
}

fn put_catalog(root: &Path, locale: &str, paused: &str) {
  let path = catalog(root, locale);
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  replace(
    &path,
    &json!({
      "notifications.enabled": "enabled",
      "notifications.dnd_enabled": paused,
    })
    .to_string(),
  );
}

fn provider(root: &Path, address: &str) -> (Process, BufReader<ChildStdout>) {
  let mut child = Command::new(env!("CARGO_BIN_EXE_argvus"))
    .args(["notifications", "taskbar", "--follow"])
    .env_remove("ARGVUS_COMMANDS_DIR")
    .env("DBUS_SESSION_BUS_ADDRESS", address)
    // Simulate logind on the same private bus.
    .env("DBUS_SYSTEM_BUS_ADDRESS", address)
    .env("ARGVUS_I18N_DIR", root.join("catalogs"))
    .env("XDG_CONFIG_HOME", root.join("config"))
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
  let stdout = BufReader::new(child.stdout.take().unwrap());
  (Process(child), stdout)
}

#[test]
#[ignore = "requires dbus-daemon and the packaged notifications helper and dunstctl"]
fn notification_stream_recovers_without_periodic_reads() {
  fs::create_dir_all("/tmp/argvus").unwrap();
  let temp = tempfile::tempdir_in("/tmp/argvus").unwrap();
  let root = temp.path();
  let address = format!("unix:path={}/bus", root.display());
  put_catalog(root, "en-US", "paused");
  let initial_bus = bus(&address);
  let client = connection(&address);
  let first = fixture(&address, false, true, true);
  let (mut stream, mut output) = provider(root, &address);
  assert_eq!(read(&mut output), json!({"fallback": true}));
  control(&client, &first, "own", false);
  assert_eq!(
    read(&mut output),
    json!({"text": "󰂛", "class": "dnd", "tooltip": "paused"})
  );
  let reads = control(&client, &first, "reads", false);
  assert!(reads >= 2, "snapshot must retry when changed in flight");
  let other = fixture(&address, false, false, true);
  control(&client, &other, "foreign", false);
  for action in ["wrong-path", "wrong-interface", "count", "changed"] {
    control(&client, &first, action, action == "changed");
  }
  assert!(
    !data_ready(&output, 2_150),
    "irrelevant and duplicate signals must be ignored"
  );
  assert_eq!(
    control(&client, &first, "reads", false),
    reads,
    "healthy state must not poll"
  );
  control(&client, &first, "invalidated", false);
  assert_eq!(read(&mut output)["class"], "enabled");
  assert_eq!(control(&client, &first, "reads", false), reads + 1);
  control(&client, &first, "own-logind", false);
  control(&client, &first, "silent", true);
  control(&client, &first, "resume", false);
  assert_eq!(read(&mut output)["class"], "dnd");

  control(&client, &first, "release", false);
  assert_eq!(read(&mut output), json!({"fallback": true}));
  let incompatible = fixture(&address, false, false, false);
  control(&client, &incompatible, "own", false);
  assert!(
    !data_ready(&output, 300),
    "a server with silent properties must use fallback"
  );
  control(&client, &incompatible, "release", false);
  control(&client, &other, "own", false);
  assert_eq!(read(&mut output)["class"], "enabled");

  control(&client, &other, "changed", true);
  assert_eq!(read(&mut output)["tooltip"], "paused");
  put_catalog(root, "pt-BR", "pausado");
  let locale = root.join("config/locale.conf");
  fs::create_dir_all(locale.parent().unwrap()).unwrap();
  replace(&locale, "ARGVUS_LANGUAGE=pt-BR\n");
  assert_eq!(read(&mut output)["tooltip"], "pausado");
  put_catalog(root, "pt-BR", "em pausa");
  assert_eq!(read(&mut output)["tooltip"], "em pausa");
  fs::remove_file(catalog(root, "pt-BR")).unwrap();
  assert_eq!(read(&mut output)["tooltip"], "paused");
  fs::remove_file(catalog(root, "en-US")).unwrap();
  assert_eq!(read(&mut output), json!({"fallback": true}));
  put_catalog(root, "en-US", "fallback restored");
  assert_eq!(read(&mut output)["tooltip"], "fallback restored");
  put_catalog(root, "pt-BR", "pt-BR restored");
  assert_eq!(read(&mut output)["tooltip"], "pt-BR restored");
  drop(initial_bus);
  assert_eq!(read(&mut output), json!({"fallback": true}));
  let _second_bus = bus(&address);
  let client = connection(&address);
  let new_owner = fixture(&address, false, false, true);
  control(&client, &new_owner, "own", false);
  assert_eq!(read(&mut output)["class"], "enabled");
  drop(output);
  let deadline = Instant::now() + Duration::from_secs(2);
  loop {
    if let Some(exit) = stream.0.try_wait().unwrap() {
      assert!(exit.success());
      break;
    }
    assert!(
      Instant::now() < deadline,
      "idle provider outlived its consumer"
    );
    thread::sleep(Duration::from_millis(10));
  }
}

#[test]
fn provider_handles_stdout_write_failure_without_panicking() {
  let exit = Command::new(env!("CARGO_BIN_EXE_argvus"))
    .args(["notifications", "taskbar", "--follow"])
    .env("ARGVUS_COMMANDS_DIR", "/nonexistent/argvus-commands")
    .stdout(fs::File::options().write(true).open("/dev/full").unwrap())
    .status()
    .unwrap();
  assert!(exit.success());
}
