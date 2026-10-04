"""Owned, bounded process execution for verification commands.

Child bytes are never runner events. Deadlines are safety outcomes, not timing
budgets. Session groups are torn down even when the direct child exits first.
"""
from __future__ import annotations

import contextlib
import dataclasses
import errno
import json
import math
import os
import selectors
import signal
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path
from typing import Callable

from .process_disk_budget import DiskBudget


@dataclasses.dataclass(frozen=True)
class Outcome:
    returncode: int
    cause: str
    stdout: bytes
    stderr: bytes
    truncated: bool
    elapsed_seconds: float


SAFETY_DEADLINE_ENV = "SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC"


def deadline_environment(env: dict[str, str] | None, seconds: float | str = 2400) -> tuple[dict[str, str], float]:
    """Keep one process safety deadline across nested commands and lock waits."""
    child_env = dict(os.environ if env is None else env)
    if isinstance(seconds, bool):
        raise ValueError("safety deadline duration must be finite and positive")
    try:
        duration = float(seconds)
    except (TypeError, ValueError) as error:
        raise ValueError("safety deadline duration must be finite and positive") from error
    if not math.isfinite(duration) or duration <= 0:
        raise ValueError("safety deadline duration must be finite and positive")
    deadline = time.monotonic() + duration
    if not math.isfinite(deadline):
        raise ValueError("safety deadline must be finite")
    inherited = child_env.get(SAFETY_DEADLINE_ENV)
    if inherited is not None:
        if isinstance(inherited, bool):
            raise ValueError("absolute safety deadline must be finite and positive")
        try:
            inherited_deadline = float(inherited)
        except (TypeError, ValueError) as error:
            raise ValueError("absolute safety deadline must be finite and positive") from error
        if not math.isfinite(inherited_deadline) or inherited_deadline <= 0:
            raise ValueError("absolute safety deadline must be finite and positive")
        deadline = min(deadline, inherited_deadline)
    child_env[SAFETY_DEADLINE_ENV] = repr(deadline)
    return child_env, deadline


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
    deadline_seconds: float | str = 2400, limit_bytes: int = 1048576,
    emit: Callable[[str, bytes], None] | None = None,
    input_bytes: bytes | None = None,
    observer: Callable[[int, float], str | None] | None = None,
) -> Outcome:
    # signal.signal is main-thread-only. Reject before creating a process that
    # this thread could not own through cancellation and cleanup.
    if threading.current_thread() is not threading.main_thread():
        raise RuntimeError("verification subprocesses require the main thread")

    start = time.monotonic()
    child_env, deadline = deadline_environment(env, deadline_seconds)
    disk_budget = DiskBudget.from_environment(child_env)
    if disk_budget is not None:
        disk_budget.check()
    streams = {"stdout": bytearray(), "stderr": bytearray()}
    truncated = False
    cause = "exit"
    cancelled = False
    previous = {}
    proc: subprocess.Popen[bytes] | None = None
    group_killed = False
    selector: selectors.BaseSelector | None = None
    status_read: int | None = None
    status_write: int | None = None

    def cancel(signum, frame):
        nonlocal cancelled
        cancelled = True

    def child_exited() -> bool:
        assert proc is not None
        if proc.returncode is not None:
            return True
        if sys.platform.startswith("linux"):
            # Keep the leader's PID reserved until session teardown completes.
            # Popen.poll would reap it and permit a different group to reuse it.
            return os.waitid(os.P_PID, proc.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT) is not None
        return proc.poll() is not None

    def kill_group(pid: int):
        nonlocal group_killed
        if group_killed:
            return
        group_killed = True
        if sys.platform.startswith("linux"):
            # The dedicated subreaper owns all descendants, including those
            # that call setsid. Let it terminate and reap before killing it.
            assert proc is not None
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            stop = time.monotonic() + 5
            while not child_exited() and time.monotonic() < stop:
                time.sleep(.01)
            if not child_exited():
                os.killpg(pid, signal.SIGKILL)
                proc.wait()
                raise RuntimeError("verification process supervisor did not complete cleanup")
            # If the supervisor itself was killed, retain the previous session
            # teardown as a last containment action. Missing completion metadata
            # below still rejects the run: group teardown cannot prove reaping.
            try:
                os.killpg(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            proc.wait()
            return
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
        if cancelled or time.monotonic() >= deadline:
            cause = "cancelled" if cancelled else "safety_deadline"
            code = 130 if cancelled else 124
        else:
            with _input_stream(input_bytes) as stdin:
                spawned_command = command
                spawn_options = {}
                if sys.platform.startswith("linux"):
                    status_read, status_write = os.pipe()
                    # Control-plane imports cannot depend on the command's site
                    # hooks, PYTHONPATH, or mutable bytecode caches.
                    spawned_command = [sys.executable, "-I", "-S", "-B", "-X", "pycache_prefix=/dev/null",
                                       str(Path(__file__).with_name("process_supervisor.py")),
                                       "--status-fd", str(status_write), "--", *command]
                    spawn_options["pass_fds"] = (status_write,)
                previous_mask = None
                if sys.platform.startswith("linux"):
                    previous_mask = signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT, signal.SIGTERM})
                try:
                    proc = subprocess.Popen(spawned_command, cwd=cwd, env=child_env, stdin=stdin,
                                            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                            start_new_session=True, **spawn_options)
                finally:
                    if previous_mask is not None:
                        signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)
                if status_write is not None:
                    os.close(status_write)
                    status_write = None
        # Ownership begins at spawn. Selector, registration and user callback
        # failures all pass through the same group teardown and child wait.
        if proc is None:
            return Outcome(code, cause, b"", b"", False, time.monotonic() - start)
        selector = selectors.DefaultSelector()
        assert proc.stdout is not None and proc.stderr is not None
        selector.register(proc.stdout, selectors.EVENT_READ, "stdout")
        selector.register(proc.stderr, selectors.EVENT_READ, "stderr")
        while selector.get_map() or not child_exited():
            if disk_budget is not None:
                disk_budget.check()
            if cancelled or time.monotonic() >= deadline:
                cause = "cancelled" if cancelled else "safety_deadline"
                kill_group(proc.pid)
                # An escaped descendant can retain a pipe indefinitely. On a
                # safety outcome, keep the bytes already read and close our ends.
                break
            if observer is not None:
                observed_stop = observer(proc.pid, time.monotonic())
                if observed_stop is not None:
                    if observed_stop not in {'observer_memory', 'observer_reserve', 'observer_unavailable',
                                             'observer_evidence_limit'}:
                        raise ValueError('invalid process observer stop cause')
                    cause = observed_stop
                    kill_group(proc.pid)
                    break
            # A direct child may abandon grandchildren that inherited its pipes.
            exited = child_exited()
            if exited:
                kill_group(proc.pid)
            events = selector.select(0 if exited else min(0.05, max(0, deadline - time.monotonic())))
            if exited and not events and sys.platform.startswith("linux"):
                # A dead supervisor cannot authorize waiting on abandoned pipes.
                # Drain available bytes, then require its completion metadata.
                break
            for key, _ in events:
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
        kill_group(proc.pid)
        code = proc.wait()
        if status_read is not None:
            try:
                status = json.loads(os.read(status_read, 4096))
            except (ValueError, OSError) as error:
                raise OSError(errno.EIO, "supervisor did not confirm owned-tree cleanup") from error
            if not isinstance(status, dict) or status.get("kind") != "completed":
                detail = status.get("detail", "invalid supervisor status") if isinstance(status, dict) else "invalid supervisor status"
                number = status.get("errno", errno.EIO) if isinstance(status, dict) else errno.EIO
                raise OSError(number if type(number) is int and number > 0 else errno.EIO, detail)
            if type(status.get("returncode")) is not int or status["returncode"] != code:
                raise OSError(errno.EIO, "supervisor completion status differs from process exit")
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
                for descriptor in (status_read, status_write):
                    if descriptor is not None:
                        os.close(descriptor)
            finally:
                for sig, handler in previous.items():
                    signal.signal(sig, handler)
    if cause != "exit":
        code = 130 if cause == "cancelled" else 124
    return Outcome(code, cause, bytes(streams["stdout"]), bytes(streams["stderr"]),
                   truncated, time.monotonic() - start)
