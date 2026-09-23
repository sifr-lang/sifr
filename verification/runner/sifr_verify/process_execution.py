"""Owned, bounded process execution for verification commands.

Child bytes are never runner events. Deadlines are safety outcomes, not timing
budgets. Session groups are torn down even when the direct child exits first.
"""
from __future__ import annotations

import contextlib
import dataclasses
import os
import selectors
import signal
import subprocess
import tempfile
import threading
import time
from pathlib import Path
from typing import Callable


@dataclasses.dataclass(frozen=True)
class Outcome:
    returncode: int
    cause: str
    stdout: bytes
    stderr: bytes
    truncated: bool
    elapsed_seconds: float


@contextlib.contextmanager
def _input_stream(data: bytes | None):
    if data is None:
        yield None
        return
    # A regular temporary file avoids deadlocking on stdin pipe capacity while
    # the owned child concurrently fills its bounded output pipes.
    with tempfile.TemporaryFile() as stream:
        stream.write(data)
        stream.seek(0)
        yield stream


def execute(
    command: list[str], *, cwd: Path, env: dict[str, str] | None = None,
    deadline_seconds: float = 2400, limit_bytes: int = 1048576,
    emit: Callable[[str, bytes], None] | None = None,
    input_bytes: bytes | None = None,
) -> Outcome:
    # signal.signal is main-thread-only. Reject before creating a process that
    # this thread could not own through cancellation and cleanup.
    if threading.current_thread() is not threading.main_thread():
        raise RuntimeError("verification subprocesses require the main thread")

    start = time.monotonic()
    streams = {"stdout": bytearray(), "stderr": bytearray()}
    truncated = False
    cause = "exit"
    cancelled = False
    previous = {}
    proc: subprocess.Popen[bytes] | None = None
    selector: selectors.BaseSelector | None = None

    def cancel(signum, frame):
        nonlocal cancelled
        cancelled = True

    def kill_group(pid: int):
        try:
            os.killpg(pid, signal.SIGTERM)
            time.sleep(0.25)
        except ProcessLookupError:
            pass
        try:
            os.killpg(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass

    try:
        # A partial signal setup has no child to clean up; the finally block
        # still restores every handler that was installed successfully.
        for sig in (signal.SIGINT, signal.SIGTERM):
            previous[sig] = signal.signal(sig, cancel)
        with _input_stream(input_bytes) as stdin:
            proc = subprocess.Popen(command, cwd=cwd, env=env, stdin=stdin,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                    start_new_session=True)
        # Ownership begins at spawn. Selector, registration and user callback
        # failures all pass through the same group teardown and child wait.
        selector = selectors.DefaultSelector()
        assert proc.stdout is not None and proc.stderr is not None
        selector.register(proc.stdout, selectors.EVENT_READ, "stdout")
        selector.register(proc.stderr, selectors.EVENT_READ, "stderr")
        while selector.get_map() or proc.poll() is None:
            if cancelled or time.monotonic() - start >= deadline_seconds:
                cause = "cancelled" if cancelled else "safety_deadline"
                kill_group(proc.pid)
            # A direct child may abandon grandchildren that inherited its pipes.
            if proc.poll() is not None:
                kill_group(proc.pid)
            for key, _ in selector.select(0.05):
                data = os.read(key.fileobj.fileno(), 65536)
                if not data:
                    selector.unregister(key.fileobj)
                    key.fileobj.close()
                    continue
                stream = streams[key.data]
                remaining = max(0, limit_bytes - len(stream))
                stream.extend(data[:remaining])
                truncated |= len(data) > remaining
                if emit is not None:
                    emit(key.data, data[:remaining])
        code = proc.wait()
    finally:
        try:
            if proc is not None:
                kill_group(proc.pid)
                proc.wait()
        finally:
            try:
                if selector is not None:
                    selector.close()
                if proc is not None:
                    if proc.stdout is not None:
                        proc.stdout.close()
                    if proc.stderr is not None:
                        proc.stderr.close()
            finally:
                for sig, handler in previous.items():
                    signal.signal(sig, handler)
    if cause != "exit":
        code = 130 if cause == "cancelled" else 124
    return Outcome(code, cause, bytes(streams["stdout"]), bytes(streams["stderr"]),
                   truncated, time.monotonic() - start)
