#!/usr/bin/env python3
"""Validate the editor tooling AnalysisHost snapshot/session contract."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]

REQUIRED_TESTS = {
    "stale_document_version_is_rejected",
    "stale_snapshot_is_rejected_after_update",
    "single_file_session_updates_versions_and_invalidates_symbols",
    "project_symbol_index_is_stable_for_workspace_queries",
    "all_editor_query_methods_expose_current_revision_metadata",
    "completion_ranking_prefers_exact_then_prefix_then_substring",
}


def run_cargo_test(extra_args: list[str] | None = None) -> subprocess.CompletedProcess[str]:
    cmd = ["cargo", "test", "-p", "sifr_analysis"]
    if extra_args:
        cmd.extend(extra_args)
    return subprocess.run(
        cmd,
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )


def run_self_test() -> None:
    result = run_cargo_test(["stale_snapshot_is_rejected_after_update"])
    if result.returncode != 0 or "stale_snapshot_is_rejected_after_update ... ok" not in result.stdout:
        print(result.stdout, file=sys.stderr)
        raise SystemExit("analysis snapshot contract self-test failed: stale snapshot test did not pass")
    print("analysis snapshot contract self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    result = run_cargo_test()
    if result.returncode != 0:
        print(result.stdout, file=sys.stderr)
        return result.returncode
    missing = sorted(test for test in REQUIRED_TESTS if f"{test} ... ok" not in result.stdout)
    if missing:
        print("analysis snapshot contract: FAIL", file=sys.stderr)
        for test in missing:
            print(f"  - missing required test evidence: {test}", file=sys.stderr)
        return 1
    print("analysis snapshot contract: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
