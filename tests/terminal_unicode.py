"""Exercise the real terminal-size branch, including when stdout is redirected.

Run directly with `python3 tests/terminal_unicode.py path/to/fozzie`.
Uses only Python's standard library; supported on Linux and macOS.
"""

import fcntl
import os
import pty
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time


def check(binary, content, columns, stdout_tty, query=""):
    master, slave = pty.openpty()
    process = None
    transcript = bytearray()
    try:
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, columns, 0, 0))
        original = termios.tcgetattr(slave)

        def setup():
            os.setsid()
            fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

        with tempfile.TemporaryFile() as source, tempfile.TemporaryFile() as result:
            source.write((content + "\n").encode())
            source.seek(0)
            process = subprocess.Popen(
                [binary, "--lines", "2"],
                stdin=source,
                stdout=slave if stdout_tty else result,
                stderr=result,
                preexec_fn=setup,
            )
            deadline = time.monotonic() + 10
            sent = False
            while time.monotonic() < deadline:
                readable, _, _ = select.select([master], [], [], 0.02)
                if readable:
                    transcript.extend(os.read(master, 65536))
                if not sent and not termios.tcgetattr(slave)[3] & termios.ICANON:
                    os.write(master, query.encode() + b"\r")
                    sent = True
                if process.poll() is not None:
                    while select.select([master], [], [], 0)[0]:
                        transcript.extend(os.read(master, 65536))
                    break
            assert process.poll() is not None, "terminal session timed out"
            result.seek(0)
            output = result.read().decode()
            terminal = transcript.decode()
            assert process.returncode == 0, (process.returncode, output, terminal)
            assert sent, "application never entered raw mode"
            assert termios.tcgetattr(slave) == original, "terminal mode was not restored"
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
            process.kill()
            process.wait()
        os.close(master)
        os.close(slave)


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
