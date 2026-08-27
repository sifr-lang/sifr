"""Subprocess command primitives for validation profile execution."""

from __future__ import annotations

import contextlib
import contextvars
import os
import signal
import subprocess
import sys
import threading
import time
from collections.abc import Iterator

from .paths import REPO_ROOT


_COMMAND_DEADLINE_NS: contextvars.ContextVar[int | None] = contextvars.ContextVar(
    "sifr_verify_command_deadline_ns", default=None
)


class CommandFailed(Exception):
    """A subprocess returned a non-zero exit code."""

    def __init__(self, returncode: int) -> None:
        super().__init__(f"command failed with exit code {returncode}")
        self.returncode = returncode


@contextlib.contextmanager
def command_deadline(budget_ms: int | None) -> Iterator[None]:
    """Apply one absolute deadline to every subprocess in a profile step."""
    deadline_ns = (
        None if budget_ms is None else time.monotonic_ns() + budget_ms * 1_000_000
    )
    token = _COMMAND_DEADLINE_NS.set(deadline_ns)
    try:
        yield
    finally:
        _COMMAND_DEADLINE_NS.reset(token)


def remaining_deadline_seconds() -> float | None:
    deadline_ns = _COMMAND_DEADLINE_NS.get()
    if deadline_ns is None:
        return None
    return max(0.0, (deadline_ns - time.monotonic_ns()) / 1_000_000_000)


def terminate_process_group(proc: subprocess.Popen[str]) -> None:
    """Terminate the command and descendants without leaving gate work running."""
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    proc.wait()


def run_command(command: list[str], *, env: dict[str, str] | None = None) -> None:
    proc = subprocess.Popen(
        command,
        cwd=REPO_ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        start_new_session=True,
    )
    assert proc.stdout is not None
    output_thread = threading.Thread(
        target=_forward_output,
        args=(proc.stdout,),
        name="sifr-verify-output",
        daemon=True,
    )
    output_thread.start()
    timeout = remaining_deadline_seconds()
    try:
        returncode = proc.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        terminate_process_group(proc)
        output_thread.join(timeout=2)
        print(
            f"sifr_verify: subprocess deadline exceeded: {' '.join(command)}",
            file=sys.stderr,
        )
        raise CommandFailed(124) from None
    output_thread.join()
    if returncode != 0:
        raise CommandFailed(returncode)


def _forward_output(stream: Iterator[str]) -> None:
    for line in stream:
        sys.stdout.write(line)


def run_self_test() -> None:
    marker = REPO_ROOT / "target" / "verification" / "deadline-child-survived"
    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.unlink(missing_ok=True)
    child = (
        "import pathlib,time; time.sleep(0.4); "
        f"pathlib.Path({str(marker)!r}).write_text('survived')"
    )
    parent = (
        "import subprocess,sys,time; "
        f"subprocess.Popen([sys.executable, '-c', {child!r}]); time.sleep(5)"
    )
    started = time.monotonic()
    try:
        with command_deadline(100):
            run_command([sys.executable, "-c", parent])
    except CommandFailed as error:
        if error.returncode != 124:
            raise AssertionError(
                f"deadline returned {error.returncode}, expected 124"
            ) from error
    else:
        raise AssertionError("deadline did not reject a hanging subprocess")
    if time.monotonic() - started > 3:
        raise AssertionError("deadline did not stop the subprocess promptly")
    time.sleep(0.5)
    if marker.exists():
        marker.unlink()
        raise AssertionError("deadline left a descendant process running")


def uv_area_command(*args: str) -> list[str]:
    return [
        "uv",
        "run",
        "--project",
        "verification",
        "--locked",
        "python",
        "-m",
        "sifr_verify",
        "areas",
        "run",
        *args,
    ]


def cargo_command(*args: str) -> list[str]:
    command = ["cargo", *args]
    if "--" in command:
        separator = command.index("--")
        return [*command[:separator], "--locked", *command[separator:]]
    return [*command, "--locked"]


def run_python(script: str, *args: str) -> None:
    run_command(["python3", script, *args])
