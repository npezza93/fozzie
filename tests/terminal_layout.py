"""Black-box narrow-terminal regressions (stdlib only, Linux and macOS).

The application reads keys and renders on a controlling PTY while stdout is a
separate file, as it is in shell command substitution. Screen assertions model
VT autowrap, so overwriting the rightmost cell and wrapping the prompt both fail.
Run: python3 tests/terminal_layout.py path/to/fozzie
"""

import codecs
import errno
import fcntl
import os
import pty
import re
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time
import unicodedata


def cells(char):
    if unicodedata.combining(char) or unicodedata.category(char) in ("Mn", "Me", "Cf"):
        return 0
    return 2 if unicodedata.east_asian_width(char) in ("W", "F") else 1


class Screen:
    """Small VT model for the cursor/erase/SGR sequences emitted by fozzie."""

    def __init__(self, columns):
        self.columns = columns
        self.rows = [[" "] * columns for _ in range(24)]
        self.row = self.col = 0
        self.saved = (0, 0)
        self.wrap = True
        self.pending_wrap = False
        self.pending = ""
        self.decoder = codecs.getincrementaldecoder("utf-8")()

    def resize(self, columns):
        self.rows = [(row + [" "] * columns)[:columns] for row in self.rows]
        self.columns = columns
        self.col = min(self.col, columns - 1)
        self.pending_wrap = False

    def feed(self, data):
        self.pending += self.decoder.decode(data)
        while self.pending:
            char = self.pending[0]
            if char == "\x1b":
                if len(self.pending) < 2:
                    break
                if self.pending[1] == "[":
                    match = re.match(r"\x1b\[([0-?]*)([ -/]*)([@-~])", self.pending)
                    if not match:
                        break
                    args, _, op = match.groups()
                    self.pending = self.pending[match.end():]
                    self.csi(args, op)
                else:
                    op = self.pending[1]
                    self.pending = self.pending[2:]
                    if op == "7":
                        self.saved = (self.row, self.col)
                    elif op == "8":
                        self.row, self.col = self.saved
                        self.pending_wrap = False
                    else:
                        raise AssertionError(f"unhandled escape: {op!r}")
                continue
            self.pending = self.pending[1:]
            if char == "\r":
                self.col = 0
                self.pending_wrap = False
            elif char == "\n":
                self.row += 1
                self.pending_wrap = False
            elif char >= " ":
                width = cells(char)
                if not width:
                    previous = self.col if self.pending_wrap else max(0, self.col - 1)
                    self.rows[self.row][previous] += char
                    continue
                if self.pending_wrap and self.wrap:
                    self.row += 1
                    self.col = 0
                self.pending_wrap = False
                if width == 2 and self.col + width > self.columns:
                    if self.wrap:
                        self.row += 1
                        self.col = 0
                    else:
                        continue
                assert self.row < len(self.rows), "unexpected screen scroll"
                self.rows[self.row][self.col] = char
                if width == 2:
                    self.rows[self.row][self.col + 1] = ""
                new_col = self.col + width
                self.col = min(new_col, self.columns - 1)
                self.pending_wrap = new_col >= self.columns
            else:
                raise AssertionError(f"unhandled control: {char!r}")

    def csi(self, args, op):
        n = int(args or "1") if not args.startswith("?") and ";" not in args else 1
        if op in ("m",):
            return
        if op in ("h", "l") and args == "?7":
            self.wrap = op == "h"
            return
        self.pending_wrap = False
        if op == "G":
            self.col = min(self.columns - 1, max(0, n - 1))
        elif op == "A":
            self.row = max(0, self.row - n)
        elif op == "C":
            self.col = min(self.columns - 1, self.col + n)
        elif op == "D":
            self.col = max(0, self.col - n)
        elif op == "K" and args == "2":
            self.rows[self.row] = [" "] * self.columns
        elif op == "J" and args in ("", "0"):
            self.rows[self.row][self.col:] = [" "] * (self.columns - self.col)
            for row in range(self.row + 1, len(self.rows)):
                self.rows[row] = [" "] * self.columns
        else:
            raise AssertionError(f"unhandled CSI: {args!r} {op!r}")

    def line(self, row):
        return "".join(self.rows[row]).rstrip()


class Session:
    def __init__(self, binary, choice, columns=48, extra_args=()):
        self.master, self.slave = pty.openpty()
        self.source = tempfile.TemporaryFile()
        self.result = tempfile.TemporaryFile()
        self.errors = tempfile.TemporaryFile()
        self.process = None
        self.status_read, self.status_write = os.pipe()
        self.release_read, self.release_write = os.pipe()
        self.screen = Screen(columns)
        self.transcript = bytearray()
        self.resize(columns)
        self.original_mode = termios.tcgetattr(self.slave)
        self.source.write((choice + "\n").encode())
        self.source.seek(0)

        def setup():
            os.setsid()
            fcntl.ioctl(self.slave, termios.TIOCSCTTY, 0)

        # Keep the session leader alive while inspecting restored terminal modes:
        # macOS revokes its controlling PTY when the session leader exits.
        supervisor = """
import os, subprocess, sys
result = subprocess.run(sys.argv[3:])
os.write(int(sys.argv[1]), str(result.returncode).encode())
os.read(int(sys.argv[2]), 1)
sys.exit(result.returncode)
"""
        self.process = subprocess.Popen(
            [sys.executable, "-c", supervisor, str(self.status_write), str(self.release_read),
             binary, "--lines", "2", "--prompt", "> ", *extra_args],
            stdin=self.source, stdout=self.result, stderr=self.errors, preexec_fn=setup,
            pass_fds=(self.status_write, self.release_read),
        )
        deadline = time.monotonic() + 10
        while termios.tcgetattr(self.slave)[3] & termios.ICANON:
            assert time.monotonic() < deadline, "application never entered raw mode"
            assert not select.select([self.status_read], [], [], 0)[0], "application exited before raw mode"
            assert self.process.poll() is None, "supervisor exited before raw mode"
            self.drain(0.02)
        self.drain()

    def resize(self, columns):
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, columns, 0, 0))
        self.screen.resize(columns)

    def drain(self, quiet=0.08):
        deadline = time.monotonic() + 10
        while select.select([self.master], [], [], quiet)[0]:
            try:
                data = os.read(self.master, 65536)
            except OSError as error:
                if error.errno != errno.EIO:
                    raise
                break
            if not data:
                break
            self.transcript.extend(data)
            self.screen.feed(data)
            assert time.monotonic() < deadline, "terminal did not settle"

    def key(self, keys):
        os.write(self.master, keys.encode() if isinstance(keys, str) else keys)
        self.drain()
        assert not select.select([self.status_read], [], [], 0)[0], self.transcript.decode()
        assert self.process.poll() is None, self.transcript.decode()

    def stable(self, expected_result):
        screen = self.screen
        assert screen.row == 0, (screen.row, screen.col, self.transcript.decode())
        assert "".join(screen.rows[0]).startswith("> "), screen.line(0)
        assert screen.line(1) == expected_result, (screen.line(1), expected_result)
        assert all(not screen.line(row) for row in range(2, 24)), [screen.line(i) for i in range(24)]
        assert 0 <= screen.col < screen.columns

    def finish(self, expected):
        os.write(self.master, b"\r")
        assert select.select([self.status_read], [], [], 10)[0], "application did not exit"
        exit_code = int(os.read(self.status_read, 32))
        self.drain()
        self.result.seek(0)
        self.errors.seek(0)
        assert exit_code == 0, self.errors.read().decode()
        assert self.result.read().decode() == expected + "\n"
        assert self.errors.read() == b""
        restored = termios.tcgetattr(self.slave)
        if sys.platform == "darwin":
            # Darwin sets transient PENDIN when ICANON is restored, to retype
            # pending input. Every persistent flag, speed and control char must match.
            restored[3] &= ~termios.PENDIN
            self.original_mode[3] &= ~termios.PENDIN
        assert restored == self.original_mode, ("terminal mode not restored", self.original_mode, restored)

    def close(self):
        if self.process is not None and self.process.poll() is None:
            os.write(self.release_write, b"x")
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait()
        for descriptor in (self.master, self.slave, self.status_read, self.status_write,
                           self.release_read, self.release_write):
            os.close(descriptor)
        self.source.close()
        self.result.close()
        self.errors.close()


def ascii_layout(binary):
    prefix = "projects/" + "a" * 32 + "/icons/"
    assert len(prefix) == 48
    choice = prefix + "arrow.svg"
    session = Session(binary, choice)
    try:
        session.stable(prefix)
        assert session.screen.rows[1][-1] == "/"  # Must not be overwritten by final 'g'.
        session.key(b"\t")
        session.stable(prefix)
        assert session.screen.line(0).endswith("arrow.svg")
        session.key(b"\x7f")
        session.stable(prefix)
        assert session.screen.line(0).endswith("arrow.sv")
        session.key(b"\x1b[D")
        session.key(b"\x04")  # Ctrl-D is fozzie's forward-delete binding.
        session.stable(prefix)
        assert session.screen.line(0).endswith("arrow.s")
        session.key("vg")
        session.stable(prefix)
        assert session.screen.line(0).endswith("arrow.svg")
        session.key(b"\x1b[D" * len(choice))
        session.stable(prefix)
        assert session.screen.col == 2
        assert session.screen.line(0).startswith("> projects/")
        session.key(b"\x1b[C" * len(choice))
        session.stable(prefix)
        assert session.screen.line(0).endswith("arrow.svg")
        # Matching must use all of the query, including the hidden prefix.
        session.key("X")
        session.stable("")
        session.key(b"\x7f")
        session.stable(prefix)
        session.resize(24)
        session.key(b"\x7f")
        session.stable(choice[:24])
        session.key("g")
        session.stable(choice[:24])
        assert session.screen.line(0).endswith("arrow.svg")
        session.resize(64)
        session.key(b"\x7f")
        session.key("g")
        session.stable(choice)
        assert session.screen.line(0) == "> " + choice
        session.finish(choice)
    finally:
        session.close()


def unicode_layout(binary):
    for choice, expected in [("a" * 47 + "東京", "a" * 47), ("a" * 47 + "e\u0301x", "a" * 47 + "e\u0301")]:
        session = Session(binary, choice)
        try:
            session.stable(expected)
            session.key(b"\t")
            session.stable(expected)
            session.key(b"\x7f")
            session.stable(expected)
            session.finish(choice)
        finally:
            session.close()
    session = Session(binary, "東e\u0301")
    try:
        session.key(b"\t")
        session.stable("東e\u0301")
        assert session.screen.col == 5, session.screen.col
        session.key(b"\x1b[D")
        assert session.screen.col == 5  # Combining mark has zero display width.
        session.key(b"\x1b[D")
        assert session.screen.col == 4
        session.key(b"\x1b[D")
        assert session.screen.col == 2
        session.key(b"\x1b[C")
        assert session.screen.col == 4
        session.key(b"\x1b[C\x1b[C\x7f\x7f")
        session.stable("東e\u0301")
        assert session.screen.line(0) == "> 東"
        assert session.screen.col == 4
        session.finish("東e\u0301")
    finally:
        session.close()


def horizontal_result_scrolling(binary):
    choice = "projects/long/path/to/arrow.svg"
    session = Session(binary, choice, columns=12)
    try:
        session.stable(choice[:12])
        session.key(b"\x1b[C")
        session.stable(choice[1:13])
        assert session.screen.line(0) == ">"
        assert session.screen.col == 2
        session.key(b"\x1b[C" * 100)
        session.stable(choice[-12:])
        session.key(b"\x1b[D")
        session.stable(choice[-13:-1])
        session.key("p")
        session.stable(choice[:12])
        # Right within a query must move its cursor without moving results.
        session.key(b"\x1b[D")
        assert session.screen.col == 2
        session.key(b"\x1b[C")
        assert session.screen.col == 3
        session.stable(choice[:12])
        session.key(b"\x1b[C")
        session.stable(choice[1:13])
        session.key(b"\x7f")
        session.stable(choice[:12])
        session.key(b"\x1b[C" * 3)
        session.key(b"\t")
        session.stable(choice[:12])
        assert session.screen.line(0).endswith("arrow.svg")
        session.key(b"\x1b[C" * 100)
        session.stable(choice[-12:])
        session.resize(40)
        session.key(b"\x1b[C")
        session.stable(choice)
        session.resize(12)
        session.key(b"\x1b[C")
        session.stable(choice[1:13])
        session.key(b"\x15")  # Ctrl-U clears the query and resets the result view.
        session.stable(choice[:12])
        for edit in (b"\x7f", b"\x1b[D\x04", b"\x1b\x7f", b"\x1b[D\x1b[D\x1bd", b"\t", b"\x15"):
            session.key(b"\x15")
            session.key("pr")
            session.key(b"\x1b[C" * 3)
            session.stable(choice[3:15])
            session.key(edit)
            session.stable(choice[:12])
        session.key(b"\x15")
        session.key("ZZ")
        session.key(b"\x1b[C" * 20)
        session.stable("")
        session.key(b"\x15")
        session.finish(choice)
    finally:
        session.close()

    # A shared viewport pans every visible row; selecting still returns full text.
    rows = ["prefix/first/long/file.rs", "prefix/second/long/file.rs"]
    session = Session(binary, "\n".join(rows), columns=10)
    try:
        session.key(b"\x1b[C" * 3)
        assert session.screen.line(1) == rows[0][3:13]
        assert session.screen.line(2) == rows[1][3:13]
        session.key(b"\x1b[B")
        assert session.screen.line(1) == rows[0][3:13]
        assert session.screen.line(2) == rows[1][3:13]
        session.finish(rows[1])
    finally:
        session.close()

    choice = "ab東京e\u0301/filename.rs"
    session = Session(binary, choice, columns=8)
    try:
        session.key(b"\x1b[C")
        session.stable("b東京e\u0301/f")
        session.key(b"\x1b[C" * 2)
        session.stable(" 京e\u0301/fil")
        session.key("京")
        session.stable("ab東京e\u0301/")
        session.key(b"\x1b[C" * 7)
        session.stable("/filenam")
        session.finish(choice)
    finally:
        session.close()

    session = Session(binary, "abcdefghijklmnop", columns=12, extra_args=("--show-scores",))
    try:
        session.stable("(     ) abcd")
        session.key(b"\x1b[C" * 2)
        session.stable("(     ) cdef")
        session.key(b"\x1b[C" * 100)
        session.stable("(     ) mnop")
        session.finish("abcdefghijklmnop")
    finally:
        session.close()


if __name__ == "__main__":
    ascii_layout(sys.argv[1])
    print("PASS captured stdout: 48-column clipping, Tab/edit/navigation, resize, full selection")
    unicode_layout(sys.argv[1])
    print("PASS wide and combining characters: clipping, Tab, editing, cursor positions")

    horizontal_result_scrolling(sys.argv[1])
    print("PASS horizontal results: bounds, edit reset, cursor movement, navigation, resize, Unicode, scores")
