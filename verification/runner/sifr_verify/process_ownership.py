"""Keep nested verification process groups inside their area's owned session.

Only explicitly enrolled top-level area workers own sessions. Nested executors
retain separate groups for local cancellation, but cannot escape the outer owner.
This is process lifetime accounting, not a sandbox against deliberate setsid().
"""
from __future__ import annotations

import os
from pathlib import Path
import signal
import subprocess
import sys
import time

SESSION_ENV = "SIFR_VERIFY_OWNED_SESSION"


def session_environment(environment):
    """Resolve the root marker once; reject stale or foreign inherited owners."""
    result = dict(environment)
    marker = result.get(SESSION_ENV)
    if marker is None:
        return result
    session = os.getsid(0)
    if marker == "root":
        if session != os.getpid():
            raise ValueError("owned session root must be its session leader")
    elif not (marker.isascii() and marker.isdecimal() and int(marker) == session):
        raise ValueError("inherited verification session does not own this process")
    result[SESSION_ENV] = str(session)
    return result


def launch_options(environment):
    """Use a new group within an inherited session, or an ordinary new session."""
    resolved = session_environment(environment)
    environment.update(resolved)
    return ({"process_group": 0} if SESSION_ENV in resolved
            else {"start_new_session": True})


def live_groups(session):
    """Read only identities and liveness; never inspect process command lines."""
    if session <= 1 or session == os.getsid(0):
        raise ValueError("cannot supervise the caller's or system session")
    groups = set()
    if sys.platform.startswith("linux"):
        for path in Path("/proc").iterdir():
            if not path.name.isdecimal():
                continue
            try:
                fields = (path / "stat").read_text().rsplit(")", 1)[1].split()
                # state, ppid, pgrp, session follow the parenthesized command.
                if int(fields[3]) == session and fields[0] not in ("Z", "X"):
                    groups.add(int(fields[2]))
            except (FileNotFoundError, ProcessLookupError):
                continue
    else:
        rows = subprocess.check_output(["ps", "-axo", "pid=,stat="], text=True)
        for row in rows.splitlines():
            pid, state = row.split()
            if state.startswith(("Z", "X")):
                continue
            try:
                if os.getsid(int(pid)) == session:
                    groups.add(os.getpgid(int(pid)))
            except ProcessLookupError:
                continue
    return groups


def stop_session(session):
    """Stop every owned group, including nested owners whose cleanup was cut short."""
    groups = live_groups(session)
    for group in groups:
        try:
            os.killpg(group, signal.SIGTERM)
        except ProcessLookupError:
            pass
    if groups:
        time.sleep(0.25)
    deadline = time.monotonic() + 2
    while groups := live_groups(session):
        for group in groups:
            try:
                os.killpg(group, signal.SIGKILL)
            except ProcessLookupError:
                pass
        if time.monotonic() >= deadline:
            raise RuntimeError(f"owned session {session} still has live groups: {sorted(groups)}")
        time.sleep(0.01)
