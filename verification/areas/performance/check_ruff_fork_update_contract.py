#!/usr/bin/env python3
"""Validate that the checked-in Ruff fork pin has syntax fixture evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
PERF_ROOT = REPO_ROOT / "verification" / "areas" / "performance"
CONTRACT = PERF_ROOT / "ruff_fork_revalidation.json"
FIXTURE_DIR = PERF_ROOT / "sifr_syntax_token_fixtures"


def current_ruff_revision() -> str:
    completed = subprocess.run(
        ["git", "-C", "third_party/ruff", "rev-parse", "HEAD"],
        cwd=REPO_ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return completed.stdout.strip()


def main() -> int:
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    recorded_revision = contract.get("ruff_fork_revision")
    actual_revision = current_ruff_revision()
    failures: list[str] = []
    if recorded_revision != actual_revision:
        failures.append(
            f"Ruff fork revision changed from {recorded_revision} to {actual_revision} without fixture revalidation"
        )
    fixtures = sorted(FIXTURE_DIR.glob("*.json"))
    if not fixtures:
        failures.append("no sifr_syntax token fixtures are checked in")
    if len(fixtures) < 5:
        failures.append(f"expected at least 5 representative syntax token fixtures, found {len(fixtures)}")
    for fixture in fixtures:
        data = json.loads(fixture.read_text(encoding="utf-8"))
        if data.get("ruff_fork_revision") != actual_revision:
            failures.append(f"{fixture.relative_to(REPO_ROOT)} does not record current Ruff fork revision")
        kinds = data.get("expected_token_kinds")
        if not isinstance(kinds, list) or not kinds:
            failures.append(f"{fixture.relative_to(REPO_ROOT)} has no expected_token_kinds")
    if failures:
        print("ruff fork update contract: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("ruff fork update contract: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
