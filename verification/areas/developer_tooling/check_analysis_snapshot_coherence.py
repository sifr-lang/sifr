#!/usr/bin/env python3
"""editor tooling snapshot-coherence gate.

This preserves the script name required by the editor tooling exit rules while
delegating to the snapshot rules checker that owns the concrete
AnalysisHost stale-version and stale-snapshot evidence.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
RULES_CHECK = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "check_analysis_snapshot_rules.py"


def run_rules(*args: str) -> int:
    return subprocess.run(
        ["python3", str(RULES_CHECK), *args],
        cwd=REPO_ROOT,
        check=False,
    ).returncode


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_rules("--self-test")
    return run_rules()


if __name__ == "__main__":
    sys.exit(main())
