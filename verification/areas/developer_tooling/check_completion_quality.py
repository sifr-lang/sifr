#!/usr/bin/env python3
"""Validate editor tooling completion-quality fixtures and regression thresholds."""

from __future__ import annotations

import argparse
import copy
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
MANIFEST = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "parity_manifest.json"


def load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise QualityError(f"expected JSON object at {path.relative_to(REPO_ROOT)}")
    return payload


class QualityError(Exception):
    pass


def completion_score(query: str, label: str) -> int:
    if not query:
        return 1
    if label == query:
        return 4
    if label.startswith(query):
        return 3
    if query in label:
        return 2
    return 0


def ranked_top_label(query: str, labels: list[str]) -> str | None:
    if not labels:
        return None
    return sorted(labels, key=lambda label: (-completion_score(query, label), label, "function"))[0]


def validate_quality_fixture(data: dict[str, Any], *, path: Path, minimum_pass_rate: float) -> None:
    cases = data.get("cases")
    if not isinstance(cases, list) or not cases:
        raise QualityError(f"{path.relative_to(REPO_ROOT)} must include non-empty cases")

    fixture_minimum = data.get("minimum_pass_rate")
    if not isinstance(fixture_minimum, int | float):
        raise QualityError(f"{path.relative_to(REPO_ROOT)} must include numeric minimum_pass_rate")
    if float(fixture_minimum) < minimum_pass_rate:
        raise QualityError(
            f"{path.relative_to(REPO_ROOT)} minimum_pass_rate {fixture_minimum} is below manifest threshold "
            f"{minimum_pass_rate}"
        )

    passed = 0
    failures: list[str] = []
    for index, case in enumerate(cases, start=1):
        if not isinstance(case, dict):
            raise QualityError(f"{path.relative_to(REPO_ROOT)} case {index} must be an object")
        query = case.get("query")
        expected = case.get("expected_top_label")
        labels = case.get("candidate_labels")
        if not isinstance(query, str) or not isinstance(expected, str):
            raise QualityError(f"{path.relative_to(REPO_ROOT)} case {index} must include query and expected_top_label")
        if not isinstance(labels, list) or not all(isinstance(label, str) for label in labels):
            raise QualityError(f"{path.relative_to(REPO_ROOT)} case {index} must include string candidate_labels")
        actual = ranked_top_label(query, labels)
        if actual == expected:
            passed += 1
        else:
            failures.append(f"case {index}: query {query!r} expected {expected!r}, got {actual!r}")

    pass_rate = passed / len(cases)
    if pass_rate < minimum_pass_rate:
        joined = "\n".join(f"  - {failure}" for failure in failures)
        raise QualityError(
            f"{path.relative_to(REPO_ROOT)} pass rate {pass_rate:.3f} is below {minimum_pass_rate:.3f}\n{joined}"
        )


def validate_manifest(manifest: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    quality_entries = manifest.get("completion_quality")
    if not isinstance(quality_entries, list) or not quality_entries:
        return ["parity manifest must include completion_quality entries"]

    for entry in quality_entries:
        if not isinstance(entry, dict):
            failures.append("completion_quality entries must be objects")
            continue
        raw_path = entry.get("path")
        test_name = entry.get("cargo_test")
        minimum = entry.get("minimum_pass_rate")
        if not isinstance(raw_path, str) or not raw_path:
            failures.append("completion_quality entry must include path")
            continue
        if not isinstance(test_name, str) or not test_name:
            failures.append(f"{raw_path}: completion_quality entry must include cargo_test")
        if not isinstance(minimum, int | float):
            failures.append(f"{raw_path}: completion_quality entry must include numeric minimum_pass_rate")
            continue
        path = REPO_ROOT / raw_path
        if not path.is_file():
            failures.append(f"completion quality fixture missing: {path.relative_to(REPO_ROOT)}")
            continue
        try:
            validate_quality_fixture(load_json(path), path=path, minimum_pass_rate=float(minimum))
        except QualityError as error:
            failures.append(str(error))
    return failures


def run_cargo_tests(manifest: dict[str, Any]) -> int:
    tests = {
        entry["cargo_test"]
        for entry in manifest.get("completion_quality", [])
        if isinstance(entry, dict) and isinstance(entry.get("cargo_test"), str)
    }
    for test in sorted(tests):
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
    manifest = load_json(MANIFEST)
    bad_manifest = copy.deepcopy(manifest)
    first = bad_manifest["completion_quality"][0]
    fixture = load_json(REPO_ROOT / first["path"])
    fixture["cases"][0]["expected_top_label"] = "__seeded_bad_completion__"
    with (REPO_ROOT / "target").joinpath("completion_quality_bad_seed.json").open("w", encoding="utf-8") as handle:
        json.dump(fixture, handle)
    first["path"] = "target/completion_quality_bad_seed.json"
    failures = validate_manifest(bad_manifest)
    if not failures:
        raise SystemExit("completion quality self-test failed: seeded top-label regression passed")
    print("completion quality self-test: PASS")


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
        print("completion quality: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    return run_cargo_tests(manifest)


if __name__ == "__main__":
    sys.exit(main())
