"""Opt-in recorder stream integration test with a private systemd impostor."""

import json
import os
from pathlib import Path
import selectors
import shutil
import subprocess
import tempfile
import time
import unittest

import gi

from gi.repository import Gio, GLib

NAME = "org.freedesktop.systemd1"
MANAGER = "/org/freedesktop/systemd1"
UNIT = "argvus-screen-recorder.service"
UNIT_PATH = MANAGER + "/unit/argvus_2dscreen_2drecorder_2eservice"
BINARY = os.environ.get("ARGVUS_TEST_RECORDING_BINARY")

XML = """<node>
  <interface name="org.freedesktop.systemd1.Manager">
    <method name="Subscribe"/>
    <method name="GetUnit"><arg type="s" direction="in"/><arg type="o" direction="out"/></method>
  </interface>
  <interface name="org.freedesktop.systemd1.Unit">
    <property name="ActiveState" type="s" access="read"/>
  </interface>
</node>"""


@unittest.skipUnless(BINARY and shutil.which("dbus-daemon"), "requires a built argvus and dbus-daemon")
class RecordingStreamTest(unittest.TestCase):
    def test_transitions_and_owner_recovery(self):
        Path("/tmp/argvus").mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir="/tmp/argvus") as directory:
            self.exercise(Path(directory))

    def exercise(self, root):
        self.state = None
        self.signal_on_snapshot = False
        address = f"unix:path={root}/bus"
        bus_config = root / "bus.conf"
        bus_config.write_text(f"""<busconfig><type>session</type><listen>{address}</listen>
<policy context="default"><allow send_destination="*"/><allow receive_sender="*"/><allow own="*"/></policy>
</busconfig>""")
        def start_bus():
            daemon = subprocess.Popen(
                ["dbus-daemon", f"--config-file={bus_config}", "--nofork", "--print-address=1"],
                stdout=subprocess.PIPE, text=True,
            )
            self.addCleanup(daemon.wait, 4)
            self.addCleanup(daemon.terminate)
            self.assertTrue(daemon.stdout.readline())
            self.addCleanup(daemon.stdout.close)
            connection = Gio.DBusConnection.new_for_address_sync(
                address,
                Gio.DBusConnectionFlags.AUTHENTICATION_CLIENT | Gio.DBusConnectionFlags.MESSAGE_BUS_CONNECTION,
                None, None,
            )
            self.addCleanup(lambda: not connection.is_closed() and connection.close_sync(None))
            return daemon, connection

        daemon, connection = start_bus()
        node = Gio.DBusNodeInfo.new_for_xml(XML)
        marker = root / "cache/hypr/gpu-screen-recorder.state"
        snapshots = 0
        props = "org.freedesktop.DBus.Properties"
        dbus = "org.freedesktop.DBus"

        def method(_connection, _sender, _path, _interface, method_name, _args, invocation):
            nonlocal snapshots
            if method_name == "Subscribe":
                invocation.return_value(None)
                return
            snapshots += 1
            if self.state is None:
                invocation.return_dbus_error(f"{NAME}.NoSuchUnit", "unit absent")
            else:
                invocation.return_value(GLib.Variant("(o)", (UNIT_PATH,)))

        def active(*_args):
            if self.signal_on_snapshot:
                self.signal_on_snapshot = False
                old_state = self.state
                self.state = "failed"
                properties_changed()
                return GLib.Variant("s", old_state)
            return GLib.Variant("s", self.state or "inactive")

        def properties_changed():
            connection.emit_signal(None, UNIT_PATH, props, "PropertiesChanged", GLib.Variant(
                "(sa{sv}as)", (f"{NAME}.Unit", {}, ["ActiveState"]),
            ))

        def unit_signal(signal):
            connection.emit_signal(None, MANAGER, f"{NAME}.Manager", signal,
                                   GLib.Variant("(so)", (UNIT, UNIT_PATH)))

        def own(method_name, name=NAME):
            args = GLib.Variant("(su)", (name, 0)) if method_name == "RequestName" else GLib.Variant("(s)", (name,))
            connection.call_sync(dbus, "/org/freedesktop/DBus", dbus, method_name, args,
                                 None, Gio.DBusCallFlags.NONE, 2000, None)

        def register_manager():
            connection.register_object(MANAGER, node.interfaces[0], method, None, None)
            connection.register_object(UNIT_PATH, node.interfaces[1], None, active, None)
            own("RequestName")
            own("RequestName", "org.freedesktop.login1")

        register_manager()
        env = dict(os.environ, XDG_RUNTIME_DIR=str(root), HOME=str(root),
                   ARGVUS_CACHE_HOME=str(root / "cache"), XDG_CONFIG_HOME=str(root / "config"),
                   DBUS_SESSION_BUS_ADDRESS=address, DBUS_SYSTEM_BUS_ADDRESS=address,
                   ARGVUS_SYSTEM_CONFIG="", ARGVUS_BOOTSTRAP="")
        catalog_root = root / "catalogs"
        env["ARGVUS_I18N_DIR"] = str(catalog_root)
        locale = root / "config/locale.conf"
        locale.parent.mkdir()
        locale.write_text("ARGVUS_LANGUAGE=en-US\n")
        def catalog(language, text):
            target = catalog_root / language / "hyprland.json"
            target.parent.mkdir(parents=True, exist_ok=True)
            replacement = target.with_suffix(".next")
            replacement.write_text(json.dumps({
                "screenshot.recording.none": f"{text} stopped",
                "screenshot.recording.started": f"{text} recording",
                "screenshot.recording.paused": f"{text} paused",
            }))
            replacement.replace(target)
            return target
        catalog("en-US", "English")
        secondary = catalog("pt-BR", "Portuguese")
        executable = [BINARY, "hypr-screenshot", "status", "--taskbar", "--follow"]
        child = subprocess.Popen(executable, env=env, stdout=subprocess.PIPE)
        self.addCleanup(child.wait, 4)
        self.addCleanup(child.terminate)
        self.addCleanup(child.stdout.close)
        fd = child.stdout.fileno()
        buffer = b""

        def line(expected_key, expected_value, timeout=5):
            nonlocal buffer
            context = GLib.MainContext.default()
            with selectors.DefaultSelector() as reader:
                reader.register(fd, selectors.EVENT_READ)
                deadline = time.monotonic() + timeout
                while time.monotonic() < deadline:
                    while b"\n" in buffer:
                        value, buffer = buffer.split(b"\n", 1)
                        status = json.loads(value)
                        self.assertEqual(status.get(expected_key), expected_value, status)
                        return status
                    while context.pending():
                        context.iteration(False)
                    if reader.select(0.02):
                        data = os.read(fd, 4096)
                        self.assertTrue(data, "provider closed stdout")
                        buffer += data
            self.fail(f"missing {expected_key}={expected_value}")

        line("class", "stopped")
        self.state = "active"
        unit_signal("UnitNew")
        line("class", "recording")
        marker.parent.mkdir(parents=True)
        replacement = marker.with_suffix(".next")
        replacement.write_text("paused\n")
        replacement.replace(marker)
        line("class", "paused")
        marker.unlink()
        line("class", "recording")
        self.state = "failed"
        properties_changed()
        line("class", "stopped")
        marker.write_text("paused\n")
        self.state = "active"
        self.signal_on_snapshot = True
        properties_changed()
        # A stale 'active' reply must never publish 'paused'. The accepted
        # failed snapshot duplicates the last stopped state.
        deadline = time.monotonic() + 3
        context = GLib.MainContext.default()
        while self.signal_on_snapshot and time.monotonic() < deadline:
            while context.pending():
                context.iteration(False)
            time.sleep(0.01)
        self.assertFalse(self.signal_on_snapshot, "the snapshot race was not exercised")
        own("ReleaseName")
        line("fallback", True)
        self.state = "active"
        own("RequestName")
        line("class", "paused")
        locale.write_text("ARGVUS_LANGUAGE=pt-BR\n")
        line("tooltip", "Portuguese paused")
        secondary.unlink()
        line("tooltip", "English paused")
        catalog("pt-BR", "Updated")
        line("tooltip", "Updated paused")
        self.state = None
        unit_signal("UnitRemoved")
        line("class", "stopped")
        daemon.terminate()
        daemon.wait(timeout=3)
        line("fallback", True)
        daemon, connection = start_bus()
        self.state = "active"
        register_manager()
        line("class", "paused")
        calls_before_idle = snapshots
        deadline = time.monotonic() + 2.2
        while time.monotonic() < deadline:
            while context.pending():
                context.iteration(False)
            time.sleep(0.01)
        self.assertEqual(snapshots, calls_before_idle, "healthy stream polled the unit")
        self.state = "inactive"
        connection.emit_signal(
            None, "/org/freedesktop/login1", "org.freedesktop.login1.Manager", "PrepareForSleep",
            GLib.Variant("(b)", (False,)),
        )
        line("class", "stopped")
        self.state = "active"
        properties_changed()
        line("class", "paused")
        child.stdout.close()
        self.assertEqual(child.wait(timeout=3), 0)

        # A new consumer must obtain the current state without waiting for a
        # transition, including startup while the existing unit is paused.
        with subprocess.Popen(executable, env=env, stdout=subprocess.PIPE) as restarted:
            try:
                fd = restarted.stdout.fileno()
                buffer = b""
                line("class", "paused")
            finally:
                restarted.stdout.close()
                if restarted.poll() is None:
                    restarted.wait(timeout=3)

        with subprocess.Popen(
            executable, env=dict(env, ARGVUS_BOOTSTRAP=str(root / "custom-bootstrap")),
            stdout=subprocess.PIPE,
        ) as unsupported:
            self.assertEqual(json.loads(unsupported.stdout.readline()), {"fallback": True})
            unsupported.stdout.close()
            self.assertEqual(unsupported.wait(timeout=3), 0)


if __name__ == "__main__":
    unittest.main()
