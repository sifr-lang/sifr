"""Shared-cloud correctness and performance have independent outcomes."""

from __future__ import annotations

import os
import sys

from .paths import REPO_ROOT
from .profile_commands import CommandFailed, run_command
from .errors import VerificationError


def run_cloud_profile(runner) -> int:
    try:
        functional = runner.run()
    except (VerificationError, OSError, ValueError) as error:
        print(f"Cloud execution blocked: {error}", file=sys.stderr)
        functional = 2
    runner.functional_exit_status = functional
    path = os.environ.get("SIFR_CLOUD_PERFORMANCE_RECEIPT")
    verdict = 3
    if path:
        script = REPO_ROOT / "verification/areas/performance/cloud_benchmarks.py"
        try:
            source=os.environ.get('SIFR_CLOUD_PERFORMANCE_SOURCE_WORKTREE')
            if source:
                from .cloud_receipt_reuse import consume
                consume(path,source,REPO_ROOT,runner.env['SIFR_GCQ_BIN'],runner.env,run_command)
            else:
                run_command([sys.executable, str(script), "check", "--receipt", path], env=runner.env)
            verdict = 0
        except CommandFailed as error:
            verdict = 1 if error.returncode == 1 else 3
            if error.returncode not in {1, 3}:
                print("Cloud performance: inconclusive (invalid or unavailable evidence)")
        except (VerificationError,OSError,ValueError,KeyError,TypeError) as error:
            print('Cloud performance: inconclusive (reuse identity rejected: '+str(error)+')')
    else:
        print("Cloud performance: inconclusive (no candidate-bound paired receipt)")
    runner.performance_exit_status = verdict or runner.performance_exit_status
    print(f"[sifr-cloud-verdict] functional={'fail' if functional else 'pass'} performance={runner.performance_exit_status}")
    if functional:
        return functional
    if runner.performance_exit_status == 1 or runner.require_performance:
        return runner.performance_exit_status
    return 0
