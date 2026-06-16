#!/usr/bin/env python3
"""Phase 35 frontend cache contract smoke gate."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]


def run(command: list[str]) -> None:
    completed = subprocess.run(command, cwd=REPO_ROOT, text=True)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def main() -> None:
    run(
        [
            "cargo",
            "test",
            "-p",
            "sifr_frontend",
            "--lib",
            "source_update_invalidates_cached_queries",
        ]
    )
    run(
        [
            "cargo",
            "test",
            "-p",
            "sifr_frontend",
            "--lib",
            "single_file_queries_are_cached_and_deterministic",
        ]
    )
    run(
        [
            "cargo",
            "test",
            "-p",
            "sifr_frontend",
            "--lib",
            "query_diagnostics_equivalence_tests",
        ]
    )
    print("frontend cache contract: PASS")


if __name__ == "__main__":
    sys.exit(main())
