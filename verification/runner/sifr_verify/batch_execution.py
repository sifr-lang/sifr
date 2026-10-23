"""Bound independent verification commands under one main-thread process owner.

Only execution overlaps: each selected command produces its own fresh outcome.
Callbacks run serially, and a callback/setup failure tears down every owned group.
"""
from __future__ import annotations

from dataclasses import dataclass, field
import os
from pathlib import Path
import selectors
import signal
import subprocess
import threading
import time
from typing import Callable, Sequence

from .process_execution import Outcome, deadline_environment


@dataclass
class _Active:
    index: int
    process: subprocess.Popen
    started: float
    deadline: float
    streams: dict[str, bytearray] = field(
        default_factory=lambda: {"stdout": bytearray(), "stderr": bytearray()})
    open_streams: set[str] = field(default_factory=set)
    truncated: bool = False
    cause: str = "exit"
    stopped: bool = False

    def stop(self) -> None:
        if self.stopped:
            return
        self.stopped = True
        try:
            os.killpg(self.process.pid, signal.SIGTERM)
            time.sleep(0.25)
        except ProcessLookupError:
            pass
        try:
            os.killpg(self.process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass

    def close(self, selector) -> None:
        for pipe in (self.process.stdout, self.process.stderr):
            if pipe is not None and not pipe.closed:
                try:
                    selector.unregister(pipe)
                except KeyError:
                    pass
                pipe.close()
        self.open_streams.clear()

    def outcome(self) -> Outcome:
        code = self.process.wait()
        if self.cause != "exit":
            code = 130 if self.cause == "cancelled" else 124
        return Outcome(code, self.cause, bytes(self.streams["stdout"]),
                       bytes(self.streams["stderr"]), self.truncated,
                       time.monotonic() - self.started)


def execute_batch(
    commands: Sequence[list[str]], *, cwd: Path,
    env: dict[str, str] | None = None, max_parallel: int,
    completed: Callable[[int, Outcome], None],
    deadline_seconds: float | str = 2400, limit_bytes: int = 1048576,
) -> None:
    """Deliver each outcome once, in completion order, with its selection index.

    The caller retains canonical reporting order and validates every outcome.
    Nonzero exits do not suppress independent commands. Cancellation or an expired
    inherited deadline produces non-passing outcomes for commands not yet started.
    Per-command deadlines start at admission; an inherited invocation deadline also
    includes queue time. The enclosing invocation timer includes all scheduling.
    Bounded output is discarded after its callback, rather than accumulated for
    the entire corpus. This batch accepts noninteractive commands only.
    """
    if threading.current_thread() is not threading.main_thread():
        raise RuntimeError("verification subprocesses require the main thread")
    if type(max_parallel) is not int or max_parallel < 1:
        raise ValueError("batch parallelism must be a positive integer")
    if type(limit_bytes) is not int or limit_bytes < 1:
        raise ValueError("batch output limit must be a positive integer")
    if not commands or any(not command or any(not isinstance(arg, str) for arg in command)
                           for command in commands):
        raise ValueError("batch requires nonempty command argument lists")
    # Freeze selection/environment before any process is created.
    commands = [list(command) for command in commands]
    environment = dict(os.environ if env is None else env)
    deadline_environment(environment, deadline_seconds)  # validate before spawning
    previous = {}
    active: dict[int, _Active] = {}
    selector = None
    cancelled = False
    next_index = 0

    def cancel(_signum, _frame):
        nonlocal cancelled
        cancelled = True

    try:
        for sig in (signal.SIGINT, signal.SIGTERM):
            previous[sig] = signal.signal(sig, cancel)
        selector = selectors.DefaultSelector()
        while next_index < len(commands) or active:
            for index, state in list(active.items()):
                if cancelled or time.monotonic() >= state.deadline:
                    state.cause = "cancelled" if cancelled else "safety_deadline"
                    state.stop()
                    state.close(selector)
                elif state.process.poll() is not None:
                    # An exited direct child must not leave pipe-owning descendants.
                    state.stop()
                if state.process.poll() is not None and not state.open_streams:
                    state.close(selector)
                    outcome = state.outcome()
                    del active[index]
                    completed(index, outcome)
            while next_index < len(commands) and len(active) < max_parallel:
                index = next_index
                next_index += 1
                child_env, deadline = deadline_environment(environment, deadline_seconds)
                if cancelled or time.monotonic() >= deadline:
                    cause = "cancelled" if cancelled else "safety_deadline"
                    completed(index, Outcome(130 if cancelled else 124, cause,
                                             b"", b"", False, 0.0))
                    continue
                started = time.monotonic()
                process = subprocess.Popen(commands[index], cwd=cwd, env=child_env,
                                           stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                           stderr=subprocess.PIPE, start_new_session=True)
                state = _Active(index, process, started, deadline)
                active[index] = state  # ownership precedes fallible registration
                for stream, pipe in (("stdout", process.stdout), ("stderr", process.stderr)):
                    selector.register(pipe, selectors.EVENT_READ, (index, stream))
                    state.open_streams.add(stream)
            if not active:
                continue
            for key, _ in selector.select(0.05):
                index, stream = key.data
                state = active[index]
                data = os.read(key.fileobj.fileno(), 65536)
                if not data:
                    selector.unregister(key.fileobj)
                    key.fileobj.close()
                    state.open_streams.remove(stream)
                    continue
                target = state.streams[stream]
                remaining = max(0, limit_bytes - len(target))
                target.extend(data[:remaining])
                state.truncated |= len(data) > remaining
    finally:
        try:
            for state in active.values():
                try:
                    state.stop()
                    state.process.wait()
                finally:
                    if selector is not None:
                        state.close(selector)
        finally:
            try:
                if selector is not None:
                    selector.close()
            finally:
                for sig, handler in previous.items():
                    signal.signal(sig, handler)
