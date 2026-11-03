#!/usr/bin/env python3
"""Exercise the real application and ncdu in a terminal without a graphical session."""
import fcntl
import os
import pty
import select
import signal
import struct
import subprocess
import tempfile
import termios
import time


def collect(master, seconds):
    end = time.monotonic() + seconds
    output = bytearray()
    while time.monotonic() < end:
        ready, _, _ = select.select([master], [], [], min(0.1, max(0, end - time.monotonic())))
        if ready:
            try:
                output.extend(os.read(master, 65536))
            except OSError:
                break
    return bytes(output)


def children(pid):
    try:
        with open(f"/proc/{pid}/task/{pid}/children", encoding="ascii") as file:
            return [int(value) for value in file.read().split()]
    except FileNotFoundError:
        return []


def wait_child(pid):
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        found = children(pid)
        if found:
            return found[0]
        time.sleep(0.05)
    raise AssertionError("ncdu process did not start")


def run(binary, terminate=False):
    with tempfile.TemporaryDirectory(prefix="argvus du-雪-", dir="/tmp/argvus") as directory:
        os.mkdir(os.path.join(directory, "directory with spaces"))
        with open(os.path.join(directory, "directory with spaces", "test"), "w", encoding="utf-8") as file:
            file.write("sample")
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 35, 120, 0, 0))
        before = termios.tcgetattr(slave)
        env = {**os.environ, "TERM": "xterm-256color", "HOME": directory, "LC_ALL": "C"}
        process = subprocess.Popen([binary, "--ui"], stdin=slave, stdout=slave, stderr=slave, env=env)
        try:
            output = collect(master, 1)
            os.write(master, b"\t" * 8 + b"\r")
            child = wait_child(process.pid)
            output += collect(master, 2)
            assert b"ncdu" in output.lower(), "ncdu screen was not rendered"
            os.write(master, b"\x1bOB\r")
            output += collect(master, 0.5)
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 100, 0, 0))
            os.kill(process.pid, signal.SIGWINCH)
            output += collect(master, 0.5)
            if terminate:
                os.kill(process.pid, signal.SIGTERM)
            else:
                os.write(master, b"q")
                collect(master, 0.5)
                assert not children(process.pid), "ncdu did not exit after q"
                os.write(master, b"\x1b")
            assert process.wait(timeout=5) == 0
            assert termios.tcgetattr(slave) == before, "terminal settings were not restored"
            assert not os.path.exists(f"/proc/{child}"), "ncdu process was left behind"
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            os.close(master)
            os.close(slave)


def run_deletion(binary, directory_item=False):
    with tempfile.TemporaryDirectory(prefix="du-delete-", dir="/tmp/argvus") as tmp:
        root = os.path.join(tmp, "scan")
        os.mkdir(root)
        victim = os.path.join(root, "victim")
        if directory_item:
            os.mkdir(victim)
            with open(os.path.join(victim, "contents"), "w", encoding="utf-8") as file:
                file.write("fixture")
        else:
            with open(victim, "w", encoding="utf-8") as file:
                file.write("fixture")
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 35, 120, 0, 0))
        env = {**os.environ, "HOME": root, "XDG_CACHE_HOME": tmp + "/cache", "XDG_CONFIG_HOME": tmp + "/config", "TERM": "xterm-256color", "LC_ALL": "C"}
        process = subprocess.Popen([binary, "--ui"], stdin=slave, stdout=slave, stderr=slave, env=env)
        try:
            collect(master, 1)
            os.write(master, b"\t" * 8 + b"\r")
            wait_child(process.pid)
            collect(master, 2)
            os.write(master, b"d")
            prompt = collect(master, 0.8)
            assert b"Confirm delete" in prompt, "ncdu confirmation did not appear"
            assert os.path.exists(victim), "item deleted before confirmation"
            os.write(master, b"\r")  # ncdu defaults to No
            collect(master, 0.5)
            assert os.path.exists(victim), "cancelled deletion removed the item"
            os.write(master, b"d")
            collect(master, 0.5)
            os.write(master, b"\x1bOD\r")  # select Yes
            collect(master, 1)
            assert not os.path.exists(victim), "confirmed deletion failed"
            os.write(master, b"q")
            collect(master, 0.5)
            os.write(master, b"\x1b")
            assert process.wait(timeout=5) == 0
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            os.close(master)
            os.close(slave)


if __name__ == "__main__":
    os.makedirs("/tmp/argvus", exist_ok=True)
    workspace = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
    executable = os.path.join(workspace, "target/debug/argvus-du")
    run(executable)
    run(executable, terminate=True)
    run_deletion(executable)
    run_deletion(executable, directory_item=True)
    print("PTY integration: navigation, resize, cleanup and confirmed file/directory deletion passed")
