#!/usr/bin/env python3
"""Exercise selection and cancellation with the real ARGVUS Superfile."""
import fcntl
import os
import pathlib
import pty
import select
import shutil
import struct
import subprocess
import tempfile
import termios
import time


def drain(master, duration):
    end = time.monotonic() + duration
    while time.monotonic() < end:
        if select.select([master], [], [], min(0.1, max(0, end - time.monotonic())))[0]:
            try:
                os.read(master, 65536)
            except OSError:
                break


def main():
    workspace = pathlib.Path(__file__).resolve().parents[3]
    os.makedirs('/tmp/argvus', exist_ok=True)
    with tempfile.TemporaryDirectory(dir='/tmp/argvus', prefix='du-superfile-') as tmp:
        root = pathlib.Path(tmp)
        home = root / 'home'
        home.mkdir()
        chosen = root / 'folder with spaces 雪'
        chosen.mkdir()
        config = root / 'config'
        config.mkdir()
        shutil.copytree('/usr/share/argvus/app-profiles/config/superfile', config / 'superfile')
        config_file = config / 'superfile/config.toml'
        config_file.write_text(config_file.read_text(encoding='utf-8').replace('auto_check_update = true', 'auto_check_update = false'), encoding='utf-8')
        original_hotkeys = (config / 'superfile/hotkeys.toml').read_bytes()
        env = {**os.environ, 'HOME': str(home), 'XDG_CONFIG_HOME': str(config),
               'ARGVUS_CONFIG_HOME': str(config), 'XDG_CACHE_HOME': str(root / 'cache'),
               'TERM': 'xterm-256color'}
        binary = workspace / 'target/debug/argvus-du'
        for key, expected in [(b' ', str(chosen)), (b'q', ''), (b'\x1b', '')]:
            selection = root / 'selection'
            selection.write_text('', encoding='utf-8')
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 40, 120, 0, 0))
            process = subprocess.Popen([str(binary), '--directory-picker', str(selection), str(chosen)],
                                       stdin=slave, stdout=slave, stderr=slave, env=env)
            try:
                drain(master, 3)
                assert process.poll() is None, 'Superfile exited before interaction'
                os.write(master, b'\r')  # dismiss first-run Superfile welcome
                drain(master, 1)
                if key == b'q':
                    os.write(master, b'/hello world')
                    drain(master, 1)
                    assert process.poll() is None, 'space interrupted text input'
                    os.write(master, b'\r')
                    drain(master, 0.5)
                os.write(master, key)
                drain(master, 2)
                assert process.wait(timeout=5) == 0
                assert selection.read_text(encoding='utf-8') == expected
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=5)
                os.close(master)
                os.close(slave)
        assert (config / 'superfile/hotkeys.toml').read_bytes() == original_hotkeys, 'user hotkeys changed'
    print('Superfile: space selection, text input and q/Esc cancellation passed')


if __name__ == '__main__':
    main()
