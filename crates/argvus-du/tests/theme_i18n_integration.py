#!/usr/bin/env python3
"""Check the real TUI's translations and live theme changes over a PTY."""
import fcntl
import os
import pathlib
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time


def collect(master, duration):
    end = time.monotonic() + duration
    output = bytearray()
    while time.monotonic() < end:
        if select.select([master], [], [], min(0.1, max(0, end - time.monotonic())))[0]:
            try:
                output.extend(os.read(master, 65536))
            except OSError:
                break
    return bytes(output)


def main():
    root = pathlib.Path(__file__).resolve().parents[3]
    os.makedirs('/tmp/argvus', exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='du-theme-i18n-', dir='/tmp/argvus') as tmp:
        base = pathlib.Path(tmp)
        data = base / 'config/argvus/data'
        current = data / 'generated/theme/current'
        (current / 'tui').mkdir(parents=True)
        (data / '.active-theme').write_text('test-dark\n', encoding='utf-8')
        (current / 'build.toml').write_text('theme = "test-dark"\nvariant = "sticky"\n', encoding='utf-8')
        css = current / 'tui/theme.css'
        css.write_text('@define-color argvus_bg #112233;\n@define-color argvus_accent #123456;\n', encoding='utf-8')
        xdg = base / 'xdg'
        xdg.mkdir()
        (xdg / 'locale.conf').write_text('LANG=pt_BR.UTF-8\n', encoding='utf-8')
        env = os.environ.copy()
        env.pop('NO_COLOR', None)
        env.update(HOME=tmp, XDG_CONFIG_HOME=str(xdg), ARGVUS_CONFIG_HOME=str(base / 'config'),
                   ARGVUS_I18N_DIR=str(root / 'resources/locales'),
                   TERM='xterm-256color', COLORTERM='truecolor')
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 35, 120, 0, 0))
        process = subprocess.Popen([str(root / 'target/debug/argvus-du'), '--ui'], stdin=slave, stdout=slave, stderr=slave, env=env)
        try:
            output = collect(master, 1.5)
            assert 'Diretório'.encode() in output, 'Portuguese catalog was not rendered'
            assert b'18;52;86' in output, 'initial accent was not rendered'
            css.write_text('@define-color argvus_bg #eeeeee;\n@define-color argvus_accent #ab1234;\n', encoding='utf-8')
            output = collect(master, 2)
            assert b'171;18;52' in output, 'theme change was not rendered'
            assert b'238;238;238' in output, 'light background was not rendered'
            os.write(master, b'\x1b')
            assert process.wait(timeout=5) == 0
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            os.close(master)
            os.close(slave)
    print('Theme reload, light background and pt-BR rendering passed')


if __name__ == '__main__':
    main()
