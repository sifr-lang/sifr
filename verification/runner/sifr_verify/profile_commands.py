"""Subprocess command primitives for validation profile execution."""

from __future__ import annotations

import contextlib
import contextvars
import os
import signal
import subprocess
import sys
import tempfile
import threading
import time
from collections.abc import Iterator
from pathlib import Path

from .paths import REPO_ROOT


_COMMAND_DEADLINE_NS: contextvars.ContextVar[int | None] = contextvars.ContextVar("sifr_verify_command_deadline_ns", default=None)
_ACTIVE_PROCESS_GROUPS: dict[int, subprocess.Popen[str]] = {}
_ACTIVE_PROCESS_GROUPS_LOCK = threading.RLock()
_TERMINAL_HANDLERS_INSTALLED = False
_HANDLING_TERMINAL_SIGNAL = False


class CommandFailed(Exception):
    """A subprocess returned a non-zero exit code."""

    def __init__(self, returncode: int) -> None:
        super().__init__(f"command failed with exit code {returncode}")
        self.returncode = returncode


@contextlib.contextmanager
def command_deadline(budget_ms: int | None) -> Iterator[None]:
    """Apply one absolute deadline to every subprocess in a profile step."""
    deadline_ns = None if budget_ms is None else time.monotonic_ns() + budget_ms * 1_000_000
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


def register_process_group(proc: subprocess.Popen[str]) -> None:
    """Register a detached child so terminal signals can reach its process group."""
    with _ACTIVE_PROCESS_GROUPS_LOCK:
        _ACTIVE_PROCESS_GROUPS[proc.pid] = proc


def unregister_process_group(proc: subprocess.Popen[str]) -> None:
    with _ACTIVE_PROCESS_GROUPS_LOCK:
        _ACTIVE_PROCESS_GROUPS.pop(proc.pid, None)


def terminate_process_group(proc: subprocess.Popen[str], *, initial_signal: signal.Signals = signal.SIGTERM) -> None:
    """Terminate the command and descendants without leaving gate work running."""
    try:
        os.killpg(proc.pid, initial_signal)
    except ProcessLookupError:
        return
    try:
        proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(proc.pid, 0)
    except (ProcessLookupError, PermissionError):
        return
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        return
    proc.wait()


def install_terminal_signal_handlers() -> None:
    """Forward terminal interruption to every live detached child group."""
    global _TERMINAL_HANDLERS_INSTALLED
    if _TERMINAL_HANDLERS_INSTALLED:
        return
    if threading.current_thread() is not threading.main_thread():
        raise RuntimeError("terminal signal handlers must be installed on the main thread")
    signal.signal(signal.SIGINT, _forward_terminal_signal)
    signal.signal(signal.SIGTERM, _forward_terminal_signal)
    _TERMINAL_HANDLERS_INSTALLED = True


def _forward_terminal_signal(signum: int, _frame: object) -> None:
    global _HANDLING_TERMINAL_SIGNAL
    if _HANDLING_TERMINAL_SIGNAL:
        raise SystemExit(128 + signum)
    _HANDLING_TERMINAL_SIGNAL = True
    with _ACTIVE_PROCESS_GROUPS_LOCK:
        active = list(_ACTIVE_PROCESS_GROUPS.values())
    for proc in active:
        try:
            os.killpg(proc.pid, signal.Signals(signum))
        except (ProcessLookupError, PermissionError):
            pass
    raise SystemExit(128 + signum)


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
    register_process_group(proc)
    try:
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
    finally:
        unregister_process_group(proc)
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
        "import pathlib,signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); "
        "time.sleep(0.4); "
        f"pathlib.Path({str(marker)!r}).write_text('survived')"
    )
    parent = f"import subprocess,sys,time; subprocess.Popen([sys.executable, '-c', {child!r}]); time.sleep(5)"
    started = time.monotonic()
    try:
        with command_deadline(100):
            run_command([sys.executable, "-c", parent])
    except CommandFailed as error:
        if error.returncode != 124:
            raise AssertionError(f"deadline returned {error.returncode}, expected 124") from error
    else:
        raise AssertionError("deadline did not reject a hanging subprocess")
    if time.monotonic() - started > 3:
        raise AssertionError("deadline did not stop the subprocess promptly")
    time.sleep(0.5)
    if marker.exists():
        marker.unlink()
        raise AssertionError("deadline left a descendant process running")
    _terminal_signal_lock_self_test()
    _terminal_signal_self_test()


def _terminal_signal_lock_self_test() -> None:
    helper = "\n".join(
        [
            "import os, signal",
            "from sifr_verify.profile_commands import _ACTIVE_PROCESS_GROUPS_LOCK, install_terminal_signal_handlers",
            "install_terminal_signal_handlers()",
            "with _ACTIVE_PROCESS_GROUPS_LOCK:",
            "    os.kill(os.getpid(), signal.SIGTERM)",
        ]
    )
    proc = subprocess.Popen(
        [sys.executable, "-c", helper],
        cwd=REPO_ROOT,
        env=_self_test_environment(),
        text=True,
        start_new_session=True,
    )
    try:
        returncode = proc.wait(timeout=3)
    except subprocess.TimeoutExpired:
        terminate_process_group(proc)
        raise AssertionError("terminal signal deadlocked while the process-group registry was locked") from None
    if returncode != 128 + signal.SIGTERM:
        raise AssertionError(f"locked terminal signal returned {returncode}, expected {128 + signal.SIGTERM}")


def _terminal_signal_self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-terminal-signal-") as tmp:
        root = Path(tmp)
        ready = root / "ready"
        survived = root / "child-survived"
        child = f"import pathlib,time; time.sleep(0.6); pathlib.Path({str(survived)!r}).write_text('survived')"
        helper = (
            "import pathlib,sys; "
            "from sifr_verify.profile_commands import "
            "install_terminal_signal_handlers,run_command; "
            "install_terminal_signal_handlers(); "
            f"pathlib.Path({str(ready)!r}).write_text('ready'); "
            f"run_command([sys.executable, '-c', {child!r}])"
        )
        proc = subprocess.Popen(
            [sys.executable, "-c", helper],
            cwd=REPO_ROOT,
            env=_self_test_environment(),
            text=True,
            start_new_session=True,
        )
        deadline = time.monotonic() + 3
        while not ready.exists() and proc.poll() is None and time.monotonic() < deadline:
            time.sleep(0.01)
        if not ready.exists():
            terminate_process_group(proc)
            raise AssertionError("terminal-signal helper did not become ready")
        os.kill(proc.pid, signal.SIGTERM)
        try:
            returncode = proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            terminate_process_group(proc)
            raise AssertionError("terminal signal did not stop the runner") from None
        if returncode != 128 + signal.SIGTERM:
            raise AssertionError(f"terminal signal returned {returncode}, expected {128 + signal.SIGTERM}")
        time.sleep(0.8)
        if survived.exists():
            raise AssertionError("terminal signal left a descendant process running")


def _self_test_environment() -> dict[str, str]:
    helper_env = os.environ.copy()
    runner_root = str(REPO_ROOT / "verification" / "runner")
    existing_pythonpath = helper_env.get("PYTHONPATH")
    helper_env["PYTHONPATH"] = f"{runner_root}{os.pathsep}{existing_pythonpath}" if existing_pythonpath else runner_root
    return helper_env


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
