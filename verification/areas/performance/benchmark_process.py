"""Own a timed command's POSIX process group through output drain and cleanup."""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any, Callable
from unittest.mock import Mock, patch

from benchmark_manifest import BenchmarkError

TERMINATE_GRACE_SECONDS = 1.0
REAP_TIMEOUT_SECONDS = 5.0


def group_exists(pgid: int) -> bool:
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return False
    return True


def signal_group(pgid: int, signum: int) -> None:
    try:
        os.killpg(pgid, signum)
    except ProcessLookupError:
        pass


def finish_group(process: subprocess.Popen[str]) -> tuple[str, str]:
    """Drain/reap the leader and require the entire owned group to disappear.

    Descendants inherit the session leader's group, including /usr/bin/time's
    children. Give parents a chance to reap their children, then kill resistant
    members. Orphans are reaped by the OS's adopter; never start another sample
    while even a zombie remains in this group. A broken adopter fails closed.
    """
    signal_group(process.pid, signal.SIGTERM)
    try:
        process.communicate(timeout=TERMINATE_GRACE_SECONDS)
    except subprocess.TimeoutExpired:
        pass
    deadline = time.monotonic() + TERMINATE_GRACE_SECONDS
    while group_exists(process.pid) and time.monotonic() < deadline:
        time.sleep(0.01)
    signal_group(process.pid, signal.SIGKILL)
    try:
        output = process.communicate(timeout=REAP_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired as error:
        raise BenchmarkError(
            f"benchmark process group {process.pid} did not close its output pipes"
        ) from error
    deadline = time.monotonic() + REAP_TIMEOUT_SECONDS
    while group_exists(process.pid):
        if time.monotonic() >= deadline:
            raise BenchmarkError(
                f"benchmark process group {process.pid} was not fully reaped; "
                "refusing to start another sample"
            )
        time.sleep(0.01)
    return output


def run_owned_process(
    command: list[str], cwd: Path, timeout_seconds: float
) -> tuple[subprocess.CompletedProcess[str], bool]:
    # The timing wrapper itself is the session/group leader. Never signal the
    # caller's group, and do not rely on killing just the wrapper process.
    process = subprocess.Popen(
        command,
        cwd=cwd,
        text=True,
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    timed_out = False
    try:
        stdout, stderr = process.communicate(timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        timed_out = True
        stdout, stderr = finish_group(process)
    except BaseException:
        finish_group(process)
        raise
    else:
        # A leader can exit while a descendant with redirected output survives.
        # Preserve the leader's status/output, but close that ownership too.
        if group_exists(process.pid):
            finish_group(process)
    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr), timed_out


_TREE_PROGRAM = r'''
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

root, depth, mode = Path(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
child = None
def terminate(signum, frame):
    if child is not None:
        child.wait()
    sys.exit(0)
signal.signal(signal.SIGTERM, terminate)
if mode == "resistant":
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
if depth:
    child = subprocess.Popen(
        [sys.executable, __file__, str(root), str(depth - 1), mode],
        stdout=subprocess.DEVNULL if mode == "closed-pipes" else None,
        stderr=subprocess.DEVNULL if mode == "closed-pipes" else None,
    )
    while not (root / f"ready-{depth - 1}").exists():
        time.sleep(0.01)
(root / f"pid-{depth}").write_text(str(os.getpid()))
if depth == 2:
    print("timeout stdout: café", flush=True)
    print("timeout stderr: café", file=sys.stderr, flush=True)
(root / f"ready-{depth}").touch()
if depth == 2 and mode in ("leader-exit", "closed-pipes"):
    sys.exit(0)
while True:
    signal.pause()
'''


def run_self_test(run_sample: Callable[[list[str], int], dict[str, Any]]) -> None:
    """Exercise the public result shape with real child/grandchild groups.

    The test-only launch handshake ensures the whole tree is ready before
    communicate starts its short timeout; correctness never depends on a
    guessed Python startup delay. No compiler or benchmark input is executed.
    """
    for exit_code in (0, 7):
        result = run_sample(
            [sys.executable, "-c", "import sys; print('ordinary stdout'); "
             "print('ordinary stderr', file=sys.stderr); "
             f"sys.exit({exit_code})"],
            10000,
        )
        assert result["exit_code"] == exit_code and not result["timed_out"], result
        assert result["stdout"] == "ordinary stdout\n", result
        assert "ordinary stderr\n" in result["stderr_tail"], result
        assert result["duration_ms"] > 0 and result["peak_rss_bytes"] is not None
        print(f"benchmark process: ordinary exit {exit_code} passed")

    # Even an unreaped orphan must block the caller rather than yield a normal
    # timed-out sample. Simulate only this OS failure, without leaking a process.
    fake_process = Mock(pid=123, communicate=Mock(return_value=("", "")))
    with (
        patch("benchmark_process.signal_group"),
        patch("benchmark_process.group_exists", return_value=True),
        patch("benchmark_process.time.monotonic", side_effect=[0, 2, 3, 9]),
    ):
        try:
            finish_group(fake_process)
        except BenchmarkError as error:
            assert "refusing to start another sample" in str(error), error
        else:
            raise AssertionError("unreaped process group did not block sampling")
    print("benchmark process: unreaped group fails closed passed")

    real_popen = subprocess.Popen
    for mode in ("cooperative", "resistant", "leader-exit", "closed-pipes"):
        with TemporaryDirectory(prefix="sifr-benchmark-process-") as raw:
            root = Path(raw)
            script = root / "tree.py"
            script.write_text(_TREE_PROGRAM, encoding="utf-8")
            owned: list[subprocess.Popen[str]] = []

            def launch_ready(*args: Any, **kwargs: Any) -> subprocess.Popen[str]:
                process = real_popen(*args, **kwargs)
                owned.append(process)
                deadline = time.monotonic() + 10.0
                while not (root / "ready-2").exists():
                    if time.monotonic() >= deadline:
                        finish_group(process)
                        raise AssertionError("process tree did not become ready")
                    time.sleep(0.01)
                return process

            try:
                with patch("benchmark_process.subprocess.Popen", launch_ready):
                    result = run_sample(
                        [sys.executable, str(script), str(root), "2", mode], 100
                    )
                assert len(owned) == 1
                assert owned[0].pid != os.getpgrp()
                assert owned[0].returncode is not None
                assert not group_exists(owned[0].pid), "owned process group survived"
                pids = [int((root / f"pid-{depth}").read_text()) for depth in range(3)]
                # Verify PID disappearance inside the NEXT sample as well as
                # here, so a return-before-cleanup regression cannot overlap it.
                next_sample = run_sample(
                    [sys.executable, "-c",
                     "import os, sys\n"
                     "for pid in map(int, sys.argv[1:]):\n"
                     "    try: os.kill(pid, 0)\n"
                     "    except ProcessLookupError: continue\n"
                     "    raise SystemExit('previous sample still exists')\n"
                     "print('no overlap')\n", *map(str, pids)],
                    10000,
                )
                assert next_sample["exit_code"] == 0, next_sample
                assert next_sample["stdout"] == "no overlap\n", next_sample
                assert result["stdout"] == "timeout stdout: café\n", result
                assert "timeout stderr: café\n" in result["stderr_tail"], result
                assert isinstance(result["stdout"], str)
                assert isinstance(result["stderr_tail"], str)
                if mode == "closed-pipes":
                    assert result["exit_code"] == 0 and not result["timed_out"], result
                else:
                    assert result["timed_out"] and result["exit_code"] is None, result
                    for key in ("peak_rss_bytes", "retired_instructions",
                                "cycles_elapsed", "cpu_time_ms"):
                        assert result[key] is None, result
                    assert result["work_counter_source"] == "unavailable", result
                print(f"benchmark process: {mode} tree cleanup and no-overlap passed")
            finally:
                for process in owned:
                    if process.poll() is None or group_exists(process.pid):
                        finish_group(process)
