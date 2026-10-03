"""Own benchmark samples through the canonical bounded process supervisor."""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any, Callable
from unittest.mock import patch

from benchmark_manifest import BenchmarkError

# Standalone area entrypoints use the same reviewed process owner as the runner.
RUNNER_ROOT = Path(__file__).resolve().parents[2] / "runner"
if str(RUNNER_ROOT) not in sys.path:
    sys.path.insert(0, str(RUNNER_ROOT))
from sifr_verify import process_execution

OUTPUT_LIMIT_BYTES = 16 * 1024 * 1024


def group_exists(pgid: int) -> bool:
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return False
    return True


def run_owned_process(
    command: list[str], cwd: Path, timeout_seconds: float
) -> tuple[subprocess.CompletedProcess[str], bool]:
    try:
        outcome = process_execution.execute(
            command, cwd=cwd, deadline_seconds=timeout_seconds,
            limit_bytes=OUTPUT_LIMIT_BYTES,
        )
    except (OSError, ValueError, RuntimeError) as error:
        raise BenchmarkError(f"benchmark process custody failed: {error}") from error
    if outcome.truncated:
        raise BenchmarkError("benchmark output exceeded the declared capture limit")
    if outcome.cause not in {"exit", "safety_deadline"}:
        raise BenchmarkError(f"benchmark process did not complete: {outcome.cause}")
    completed = subprocess.CompletedProcess(
        command, outcome.returncode, outcome.stdout.decode("utf-8", "replace"),
        outcome.stderr.decode("utf-8", "replace"),
    )
    return completed, outcome.cause == "safety_deadline"


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
if mode in ("resistant", "detached"):
    if mode == "detached" and depth < 2:
        os.setsid()
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
    the test inspects the complete tree; correctness never depends on a
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

    # Missing cleanup confirmation, cancellation and output loss cannot become
    # timed-out observations or permit the sampling loop to continue.
    for failure in (OSError("missing cleanup confirmation"), RuntimeError("cleanup failed")):
        with patch.object(process_execution, "execute", side_effect=failure):
            try:
                run_owned_process(["unused"], Path.cwd(), 1)
            except BenchmarkError:
                pass
            else:
                raise AssertionError("failed custody authorized another sample")
    for cause, truncated in (("cancelled", False), ("exit", True)):
        outcome = process_execution.Outcome(0, cause, b"", b"", truncated, .01)
        with patch.object(process_execution, "execute", return_value=outcome):
            try:
                run_owned_process(["unused"], Path.cwd(), 1)
            except BenchmarkError:
                pass
            else:
                raise AssertionError("incomplete capture authorized another sample")
    if sys.platform.startswith("linux"):
        completed, timed_out = run_owned_process(
            [sys.executable, "-c", "import os, signal; os.kill(os.getpid(), signal.SIGKILL)"],
            Path.cwd(), 10,
        )
        assert completed.returncode == -signal.SIGKILL and not timed_out
    print("benchmark process: failed custody and incomplete capture fail closed passed")

    real_popen = subprocess.Popen
    for mode in ("cooperative", "resistant", "leader-exit", "closed-pipes", "detached"):
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
                        process.terminate()
                        process.wait(timeout=5)
                        raise AssertionError("process tree did not become ready")
                    time.sleep(0.01)
                return process

            try:
                with patch.object(process_execution.subprocess, "Popen", launch_ready):
                    result = run_sample(
                        [sys.executable, str(script), str(root), "2", mode], 12000
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
                if mode in ("leader-exit", "closed-pipes"):
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
                        process.terminate()
                        process.wait(timeout=5)
