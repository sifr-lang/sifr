#!/usr/bin/env python3
"""Phase 36 snapshot-coherence gate.

This preserves the script name required by the Phase 36 exit contract while
delegating to the m36.3 snapshot contract checker that owns the concrete
AnalysisHost stale-version and stale-snapshot evidence.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
CONTRACT_CHECK = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "check_analysis_snapshot_contract.py"


def run_contract(*args: str) -> int:
    return subprocess.run(
        ["python3", str(CONTRACT_CHECK), *args],
        cwd=REPO_ROOT,
        check=False,
    ).returncode


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_contract("--self-test")
    return run_contract()


if __name__ == "__main__":
    sys.exit(main())
