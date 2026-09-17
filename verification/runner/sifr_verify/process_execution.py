"""Owned, bounded process execution for verification commands.

Child bytes are never runner events. Deadlines are safety outcomes, not timing
budgets. Session groups are torn down even when the direct child exits first.
"""
from __future__ import annotations

import dataclasses
import os
import selectors
import signal
import subprocess
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


def execute(
    command: list[str], *, cwd: Path, env: dict[str, str] | None = None,
    deadline_seconds: float = 2400, limit_bytes: int = 1048576,
    emit: Callable[[str, bytes], None] | None = None,
) -> Outcome:
    start = time.monotonic()
    proc = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, start_new_session=True)
    selector = selectors.DefaultSelector()
    assert proc.stdout is not None and proc.stderr is not None
    selector.register(proc.stdout, selectors.EVENT_READ, "stdout")
    selector.register(proc.stderr, selectors.EVENT_READ, "stderr")
    streams = {"stdout": bytearray(), "stderr": bytearray()}
    truncated = False
    cause = "exit"
    cancelled = False
    previous = {}
    def cancel(signum, frame):
        nonlocal cancelled
        cancelled = True
    for sig in (signal.SIGINT, signal.SIGTERM):
        previous[sig] = signal.signal(sig, cancel)
    def kill_group():
        try:
            os.killpg(proc.pid, signal.SIGTERM)
            time.sleep(0.25)
        except ProcessLookupError:
            pass
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    try:
        while selector.get_map() or proc.poll() is None:
            if cancelled or time.monotonic() - start >= deadline_seconds:
                cause = "cancelled" if cancelled else "safety_deadline"
                kill_group()
            # A direct child may abandon grandchildren that inherited its pipes.
            if proc.poll() is not None:
                kill_group()
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
        kill_group()
        proc.wait()
        selector.close()
        for sig, handler in previous.items():
            signal.signal(sig, handler)
    if cause != "exit":
        code = 130 if cause == "cancelled" else 124
    return Outcome(code, cause, bytes(streams["stdout"]), bytes(streams["stderr"]),
                   truncated, time.monotonic() - start)
