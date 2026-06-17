#!/usr/bin/env python3
"""Run editor tooling editor-query parity and completion-quality checks."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
MANIFEST = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "parity_manifest.json"
REQUIRED_QUERIES = {
    "hover",
    "definition",
    "references",
    "rename",
    "document_symbols",
    "semantic_tokens",
    "inlay_hints",
    "document_highlights",
    "folding_ranges",
    "selection_ranges",
    "generated_rust_preview",
    "code_actions",
    "explain_diagnostic",
}


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_manifest(manifest: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if manifest.get("analysis_crate") != "sifr_analysis":
        failures.append("manifest must target sifr_analysis")
    snapshots = manifest.get("editor_query_snapshots")
    if not isinstance(snapshots, list) or not snapshots:
        failures.append("manifest must include editor_query_snapshots")
        snapshots = []
    covered: set[str] = set()
    tests: set[str] = set()
    for snapshot in snapshots:
        path = REPO_ROOT / str(snapshot.get("path", ""))
        if not path.is_file():
            failures.append(f"snapshot file missing: {path.relative_to(REPO_ROOT)}")
            continue
        load_json(path)
        queries = snapshot.get("queries", [])
        if not isinstance(queries, list) or not queries:
            failures.append(f"snapshot {snapshot.get('name')} must list queries")
        covered.update(str(query) for query in queries)
        test = snapshot.get("cargo_test")
        if not isinstance(test, str) or not test:
            failures.append(f"snapshot {snapshot.get('name')} must include cargo_test")
        else:
            tests.add(test)
    missing_queries = sorted(REQUIRED_QUERIES - covered)
    failures.extend(f"missing required query parity coverage: {query}" for query in missing_queries)

    completion_quality = manifest.get("completion_quality")
    if not isinstance(completion_quality, list) or not completion_quality:
        failures.append("manifest must include completion_quality")
        completion_quality = []
    for quality in completion_quality:
        path = REPO_ROOT / str(quality.get("path", ""))
        if not path.is_file():
            failures.append(f"completion quality file missing: {path.relative_to(REPO_ROOT)}")
            continue
        data = load_json(path)
        if float(data.get("minimum_pass_rate", -1.0)) < 1.0:
            failures.append(f"completion quality threshold must be 1.0: {path.relative_to(REPO_ROOT)}")
        test = quality.get("cargo_test")
        if not isinstance(test, str) or not test:
            failures.append(f"completion quality {quality.get('name')} must include cargo_test")
        else:
            tests.add(test)
    if not tests:
        failures.append("manifest did not register cargo tests")
    return failures


def run_tests(manifest: dict[str, Any]) -> int:
    tests: list[str] = []
    for snapshot in manifest["editor_query_snapshots"]:
        tests.append(snapshot["cargo_test"])
    for quality in manifest["completion_quality"]:
        tests.append(quality["cargo_test"])
    for test in sorted(set(tests)):
        result = subprocess.run(
            ["cargo", "test", "-p", "sifr_analysis", test],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        if result.returncode != 0 or f"{test} ... ok" not in result.stdout:
            print(result.stdout, file=sys.stderr)
            return 1
    return 0


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        bad = Path(tmp) / "bad_manifest.json"
        bad.write_text(json.dumps({"analysis_crate": "sifr_analysis"}), encoding="utf-8")
        failures = validate_manifest(load_json(bad))
    if not failures:
        raise SystemExit("tooling parity self-test failed: malformed manifest passed")
    print("tooling parity self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    manifest = load_json(MANIFEST)
    failures = validate_manifest(manifest)
    if failures:
        print("tooling parity: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    test_status = run_tests(manifest)
    if test_status != 0:
        return test_status
    print("tooling parity: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
