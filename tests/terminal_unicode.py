"""Exercise the real terminal-size branch, including when stdout is redirected.

Run directly with `python3 tests/terminal_unicode.py path/to/fozzie`.
Uses only Python's standard library; supported on Linux and macOS.
"""

import errno
import fcntl
import os
import pty
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time


def check(binary, content, columns, stdout_tty, query=""):
    master, slave = pty.openpty()
    process = None
    status_read, status_write = os.pipe()
    release_read, release_write = os.pipe()
    transcript = bytearray()
    try:
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, columns, 0, 0))
        original = termios.tcgetattr(slave)

        def setup():
            os.setsid()
            fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

        def read_terminal():
            try:
                data = os.read(master, 65536)
            except OSError as error:
                if error.errno != errno.EIO:
                    raise
                return False
            transcript.extend(data)
            # Some PTYs signal EOF with an empty read, others with EIO.
            return bool(data)

        with tempfile.TemporaryFile() as source, tempfile.TemporaryFile() as result:
            source.write((content + "\n").encode())
            source.seek(0)
            # macOS revokes a controlling PTY when its session leader exits.
            # Keep a supervisor alive until output and restored modes are checked.
            supervisor = """
import os, subprocess, sys
result = subprocess.run(sys.argv[3:])
os.write(int(sys.argv[1]), str(result.returncode).encode())
os.read(int(sys.argv[2]), 1)
sys.exit(result.returncode)
"""
            process = subprocess.Popen(
                [sys.executable, "-c", supervisor, str(status_write), str(release_read),
                 binary, "--lines", "2"],
                stdin=source,
                stdout=slave if stdout_tty else result,
                stderr=result,
                preexec_fn=setup,
                pass_fds=(status_write, release_read),
            )
            deadline = time.monotonic() + 10
            sent = False
            exit_code = None
            while time.monotonic() < deadline:
                readable, _, _ = select.select([master], [], [], 0.02)
                if readable:
                    read_terminal()
                if not sent and not termios.tcgetattr(slave)[3] & termios.ICANON:
                    os.write(master, query.encode() + b"\r")
                    sent = True
                if select.select([status_read], [], [], 0)[0]:
                    exit_code = int(os.read(status_read, 32))
                    while select.select([master], [], [], 0)[0]:
                        if not read_terminal():
                            break
                    break
                if process.poll() is not None:
                    break
            assert exit_code is not None, "terminal session timed out or supervisor exited"
            result.seek(0)
            output = result.read().decode()
            terminal = transcript.decode()
            assert exit_code == 0, (exit_code, output, terminal)
            assert sent, "application never entered raw mode"
            restored = termios.tcgetattr(slave)
            if sys.platform == "darwin":
                # Darwin sets this transient input-reprocessing bit when
                # restoring ICANON; it is not an unrestored terminal mode.
                # https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/tty.c
                original[3] &= ~termios.PENDIN
                restored[3] &= ~termios.PENDIN
            assert restored == original, ("terminal mode was not restored", original, restored)
            # Keep the existing character limit and no-wrap behavior. This is
            # deliberately not a change to grapheme or display-cell clipping.
            visible = content[:columns] if stdout_tty else content
            assert "\x1b[?7l\x1b[7m" + visible + "\x1b[27m\x1b[?7h" in terminal, terminal
            if query:
                for char in query:
                    if char in visible:
                        assert "\x1b[33m" + char + "\x1b[39m" in terminal, terminal
            if stdout_tty:
                assert terminal.endswith(content + "\n"), terminal
                assert output == "", output
            else:
                assert output == content + "\n", output
    finally:
        if process is not None and process.poll() is None:
            os.write(release_write, b"x")
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
        for descriptor in [master, slave, status_read, status_write, release_read, release_write]:
            os.close(descriptor)


if __name__ == "__main__":
    for text, width, query in [
        ("café", 80, "é"),
        ("東京", 80, "東"),
        ("a💣b", 2, "💣"),
        ("e\u0301lan", 2, "\u0301"),
        ("src/東京/" + "long/" * 30 + "café.rs", 12, "東"),
        ("some/long/ascii/path.rs", 8, "s"),
    ]:
        for tty in [True, False]:
            check(sys.argv[1], text, width, tty, query)
            print(f"PASS columns={width} stdout_tty={tty} {text!r}")
