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
import re
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


_DARWIN_PS = ['/bin/ps', '-axo', 'pid=,ppid=,pgid=,uid=,ruid=,stat=,lstart=']
_DARWIN_CLEANUP_SECONDS = 5.0
_DARWIN_PROBE_SECONDS = .5
_DARWIN_PROBE_BYTES = 1024**2


def _darwin_require_waitid():
    if (not callable(getattr(os, 'waitid', None)) or any(not hasattr(os, name) for name in
            ('P_PID', 'WEXITED', 'WNOHANG', 'WNOWAIT'))):
        raise RuntimeError('Darwin process ownership requires waitid/WNOWAIT')


def _darwin_child_exited(proc):
    if proc.returncode is not None:
        return True
    value = os.waitid(os.P_PID, proc.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
    if value is not None and value.si_pid != proc.pid:
        raise OSError(errno.EIO, 'Darwin non-reaping child identity differs')
    return value is not None


def _darwin_process_rows(raw):
    rows = {}
    pattern = (r'\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\S+)\s+'
               r'([A-Za-z]{3} [A-Za-z]{3}\s+\d{1,2} \d\d:\d\d:\d\d \d{4})\s*')
    for line in raw.splitlines():
        match = re.fullmatch(pattern, line)
        if not match:
            raise ValueError('invalid Darwin teardown process inventory')
        pid, parent, group, uid, ruid = map(int, match.groups()[:5])
        if pid in rows:
            raise ValueError('duplicate Darwin teardown PID')
        rows[pid] = {'pid': pid, 'parent': parent, 'group': group, 'uid': uid, 'ruid': ruid,
                     'state': match[6], 'start': ' '.join(match[7].split())}
    if not rows:
        raise ValueError('empty Darwin teardown process inventory')
    return rows


def _darwin_snapshot(deadline):
    """One fixed, bounded OS query, used only during Darwin teardown."""
    end = min(deadline, time.monotonic()+_DARWIN_PROBE_SECONDS)
    if time.monotonic() >= end:
        raise TimeoutError('Darwin teardown probe deadline')
    probe = subprocess.Popen(_DARWIN_PS, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             stdin=subprocess.DEVNULL, env=os.environ | {'LC_ALL': 'C'})
    streams = {'stdout': bytearray(), 'stderr': bytearray()}
    original_error = None
    try:
        # Keep control-plane probing separate from the command's selector and
        # callback failure paths. Never recursively enter execute here.
        with selectors.SelectSelector() as selector:
            for name in streams:
                selector.register(getattr(probe, name), selectors.EVENT_READ, name)
            while selector.get_map():
                remaining = end-time.monotonic()
                if remaining <= 0:
                    raise TimeoutError('Darwin teardown probe deadline')
                for key, _ in selector.select(min(.01, remaining)):
                    capacity = _DARWIN_PROBE_BYTES-sum(map(len, streams.values()))
                    data = os.read(key.fd, min(65536, capacity+1))
                    if not data:
                        selector.unregister(key.fileobj)
                    elif len(data) > capacity:
                        raise ValueError('Darwin teardown probe output limit')
                    else:
                        streams[key.data].extend(data)
        code = probe.wait(timeout=max(.001, end-time.monotonic()))
        if code or streams['stderr']:
            raise ValueError('Darwin teardown process probe failed')
        return _darwin_process_rows(streams['stdout'].decode('utf-8', errors='strict'))
    except BaseException as error:
        original_error = error
        raise
    finally:
        try:
            try:
                if probe.poll() is None:
                    probe.kill()
                probe.wait(timeout=max(.001, min(_DARWIN_PROBE_SECONDS, deadline-time.monotonic())))
            finally:
                probe.stdout.close()
                probe.stderr.close()
        except BaseException as cleanup_error:
            if original_error is None:
                raise
            original_error.add_note('Darwin ps cleanup also failed: '+type(cleanup_error).__name__+': '+str(cleanup_error))


class _DarwinGroup:
    """Hold the leader PID through signalling; reap before final absence proof."""
    def __init__(self, proc):
        self.proc = proc
        self.deadline = None
        self.identities = {}
        self.reaped = False
        self.complete = False
        self.signal_errors = []

    def remaining(self):
        remaining = self.deadline-time.monotonic()
        if remaining <= 0:
            raise TimeoutError('Darwin owned-group cleanup deadline')
        return remaining

    def members(self):
        self.remaining()
        if not self.reaped:
            if self.proc.returncode is not None:
                raise RuntimeError('Darwin owned leader was reaped before teardown')
            _darwin_child_exited(self.proc)  # ECHILD cannot authorize a reused PID.
        rows = _darwin_snapshot(self.deadline)
        members = [row for pid, row in rows.items() if pid > 0 and row['group'] == self.proc.pid]
        anchor = rows.get(self.proc.pid)
        if not self.reaped and (anchor is None or anchor['group'] != self.proc.pid
                or anchor['parent'] != os.getpid()):
            raise RuntimeError('Darwin owned group anchor unavailable')
        for row in members:
            identity = (row['start'], row['group'], row['uid'], row['ruid'])
            prior = self.identities.get(row['pid'])
            if (row['uid'] != os.geteuid() or row['ruid'] != os.getuid()
                    or prior is not None and prior != identity or self.reaped and prior is None):
                raise RuntimeError('Darwin process group identity changed or unknown')
            # A held session-leader PID reserves the original group. Its newly
            # observed same-UID members are within that group scope; setsid
            # escapes remain outside the existing Darwin custody guarantee.
            self.identities[row['pid']] = identity
        if not self.reaped and anchor is not None and anchor['group'] != self.proc.pid:
            raise RuntimeError('Darwin owned leader group changed')
        return members

    @staticmethod
    def live(members):
        return any(not row['state'].startswith('Z') for row in members)

    def send(self, number, members):
        self.remaining()
        if self.reaped or self.proc.returncode is not None:
            raise RuntimeError('Darwin group signal after leader reaping refused')
        if not self.live(members):
            return members
        try:
            os.killpg(self.proc.pid, number)
        except OSError as error:
            if error.errno not in (errno.ESRCH, errno.EPERM):
                raise
            self.signal_errors.append(error)
            # The last live member can exit between ps and killpg. Neither
            # errno proves cleanup; only stable dead-only evidence may proceed
            # to direct-child reaping and the mandatory final absence check.
            after = self.members()
            if self.live(after):
                raise
            return after
        return self.members()

    def cleanup(self):
        if self.complete:
            return
        if self.deadline is None:
            self.deadline = time.monotonic()+_DARWIN_CLEANUP_SECONDS
        try:
            if not self.reaped:
                members = self.members()
                if self.live(members):
                    members = self.send(signal.SIGTERM, members)
                    if self.remaining() < .25:
                        raise TimeoutError('Darwin TERM grace exceeds cleanup deadline')
                    time.sleep(.25)
                    members = self.members()
                if self.live(members):
                    members = self.send(signal.SIGKILL, members)
                while self.live(members) or not _darwin_child_exited(self.proc):
                    time.sleep(min(.01, self.remaining()))
                    members = self.members()
                self.proc.wait(timeout=self.remaining())
                self.reaped = True
            # No signals occur beyond this point, even if the PGID is reused.
            while self.members():
                time.sleep(min(.05, self.remaining()))
            self.complete = True
        except BaseException as error:
            if self.signal_errors and error is not self.signal_errors[0]:
                original = self.signal_errors[0]
                original.add_note('Darwin cleanup proof failed: '+type(error).__name__+': '+str(error))
                raise original from error
            raise


def execute(
    command: list[str], *, cwd: Path, env: dict[str, str] | None = None,
    deadline_seconds: float | str = 2400, limit_bytes: int = 1048576,
    emit: Callable[[str, bytes], None] | None = None,
    input_bytes: bytes | None = None,
) -> Outcome:
    # signal.signal is main-thread-only. Reject before creating a process that
    # this thread could not own through cancellation and cleanup.
    if threading.current_thread() is not threading.main_thread():
        raise RuntimeError("verification subprocesses require the main thread")

    if sys.platform == 'darwin':
        _darwin_require_waitid()
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
    darwin_group = None
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
        if sys.platform == 'darwin':
            return _darwin_child_exited(proc)
        return proc.poll() is not None

    def kill_group(pid: int):
        nonlocal group_killed
        if group_killed:
            return
        if sys.platform == 'darwin':
            assert darwin_group is not None
            darwin_group.cleanup()
            group_killed = True
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

    original_error = None
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
                    if sys.platform == 'darwin':
                        darwin_group = _DarwinGroup(proc)
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
    except BaseException as error:
        original_error = error
        raise
    finally:
        try:
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
        except BaseException as cleanup_error:
            if sys.platform != 'darwin' or original_error is None:
                raise
            original_error.add_note('Darwin cleanup also failed: '+type(cleanup_error).__name__+': '+str(cleanup_error))
    if cause != "exit":
        code = 130 if cause == "cancelled" else 124
    return Outcome(code, cause, bytes(streams["stdout"]), bytes(streams["stderr"]),
                   truncated, time.monotonic() - start)
