"""Shared-cloud correctness and performance have independent outcomes."""

from __future__ import annotations

import os
import sys

from .paths import REPO_ROOT
from .profile_commands import CommandFailed, run_command


def run_cloud_profile(runner) -> int:
    functional = runner.run()
    if functional:
        runner.performance_exit_status = runner.performance_exit_status or 3
        return functional
    path = os.environ.get("SIFR_CLOUD_PERFORMANCE_RECEIPT")
    verdict = 3
    if path:
        script = REPO_ROOT / "verification/areas/performance/cloud_benchmarks.py"
        try:
            run_command([sys.executable, str(script), "check", "--receipt", path], env=runner.env)
            verdict = 0
        except CommandFailed as error:
            verdict = 1 if error.returncode == 1 else 3
            if error.returncode not in {1, 3}:
                print("Cloud performance: inconclusive (invalid or unavailable evidence)")
    else:
        print("Cloud performance: inconclusive (no candidate-bound paired receipt)")
    runner.performance_exit_status = verdict or runner.performance_exit_status
    print(f"[sifr-cloud-verdict] functional=pass performance={runner.performance_exit_status}")
    if runner.performance_exit_status == 1 or runner.require_performance:
        return runner.performance_exit_status
    return 0
