"""Linux-only dedicated subreaper for one verification process tree.

The supervisor never spawns unrelated work. Every child it adopts belongs to
its command, even when that child leaves the original process session.
"""
from __future__ import annotations

import ctypes
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import sys
import time


def identity(pid: int) -> tuple[int, int] | None:
    try:
        text = Path(f"/proc/{pid}/stat").read_text()
        # Names may contain spaces and closing parentheses.
        fields = text[text.rindex(")") + 2:].split()
        return int(fields[1]), int(fields[19])
    except (FileNotFoundError, ProcessLookupError):
        return None


def snapshot() -> dict[int, tuple[int, int]]:
    result = {}
    for entry in Path("/proc").iterdir():
        if not entry.name.isdecimal():
            continue
        pid = int(entry.name)
        observed = identity(pid)
        if observed is not None:
            result[pid] = observed
    return result


def descendants() -> dict[int, int]:
    table = snapshot()
    pending = [os.getpid()]
    result = {}
    while pending:
        parent = pending.pop()
        for pid, (ppid, started) in table.items():
            if ppid == parent and pid not in result:
                result[pid] = started
                pending.append(pid)
    return result


def deliver(sig: int) -> None:
    # All descendants are private to this supervisor. This also handles setsid
    # grandchildren, which group-only teardown misses.
    for pid, started in descendants().items():
        try:
            descriptor = os.pidfd_open(pid)
        except ProcessLookupError:
            continue
        try:
            observed = identity(pid)
            if observed is not None and observed[1] == started:
                try:
                    signal.pidfd_send_signal(descriptor, sig)
                except ProcessLookupError:
                    pass
        finally:
            os.close(descriptor)


def reap() -> None:
    while True:
        try:
            pid, _ = os.waitpid(-1, os.WNOHANG)
        except ChildProcessError:
            return
        if pid == 0:
            return


def supervise(command: list[str]) -> int:
    cancelled = False
    def cancel(_signum, _frame):
        nonlocal cancelled
        cancelled = True
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, cancel)
    signal.pthread_sigmask(signal.SIG_UNBLOCK, {signal.SIGINT, signal.SIGTERM})
    parent = os.getppid()
    libc = ctypes.CDLL(None, use_errno=True)
    # PR_SET_CHILD_SUBREAPER and PR_SET_PDEATHSIG. Fail before command spawn if
    # the host cannot provide the advertised ownership guarantee.
    for option, value in ((36, 1), (1, signal.SIGTERM)):
        if libc.prctl(option, value, 0, 0, 0) != 0:
            raise OSError(ctypes.get_errno(), "cannot establish process-tree custody")
    if os.getppid() != parent or cancelled:
        return 130
    # Establish pidfd support before spawning rather than discovering that
    # PID-reuse-safe teardown is unavailable after work has started.
    probe = os.pidfd_open(os.getpid())
    os.close(probe)
    proc = subprocess.Popen(command)
    code = None
    try:
        while not cancelled:
            code = proc.poll()
            if code is not None:
                break
            time.sleep(.02)
    finally:
        deliver(signal.SIGTERM)
        grace = time.monotonic() + .25
        while time.monotonic() < grace and descendants():
            # The direct child remains owned by Popen until its status is read.
            proc.poll()
            time.sleep(.01)
        deliver(signal.SIGKILL)
        actual = proc.wait(timeout=2)
        if code is None:
            code = actual
        end = time.monotonic() + 2
        while True:
            reap()
            if not descendants():
                break
            if time.monotonic() >= end:
                raise RuntimeError("owned descendant cleanup did not complete")
            deliver(signal.SIGKILL)
            time.sleep(.01)
    return 130 if cancelled else code


def main() -> None:
    if not sys.platform.startswith("linux") or len(sys.argv) < 5 or sys.argv[1] != "--status-fd" or sys.argv[3] != "--":
        raise RuntimeError("Linux process supervisor requires a command")
    status_fd = int(sys.argv[2])
    try:
        code = supervise(sys.argv[4:])
    except BaseException as error:
        # Non-BMP characters expand to twelve ASCII JSON bytes each. Keep this
        # complete atomic frame below the parent's 4096-byte protocol bound.
        status = {"kind": "infrastructure-failure", "detail": str(error)[:256],
                  "errno": error.errno if isinstance(error, OSError) else 5}
        code = 2
    else:
        status = {"kind": "completed", "returncode": code}
    os.write(status_fd, json.dumps(status).encode())
    os.close(status_fd)
    if code < 0:
        sig = -code
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        if sig not in {signal.SIGKILL, signal.SIGSTOP}:
            signal.signal(sig, signal.SIG_DFL)
        os.kill(os.getpid(), sig)
    raise SystemExit(code)


if __name__ == "__main__":
    main()
