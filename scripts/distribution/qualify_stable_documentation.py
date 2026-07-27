#!/usr/bin/env python3
"""Produce canonical stable-documentation qualification evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
EXPECTED_SUITES = ("structure", "ga-release")


class DocumentationQualificationError(ValueError):
    """Stable documentation qualification failed."""


def canonical_bytes(payload: Any) -> bytes:
    return (
        json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
        + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def git_output(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise DocumentationQualificationError(
            f"git {' '.join(args)} failed: {(result.stderr or result.stdout).strip()}"
        )
    return result.stdout.strip()


def validate_result(payload: Any) -> list[dict[str, Any]]:
    if not isinstance(payload, dict) or payload.get("area") != "documentation":
        raise DocumentationQualificationError(
            "documentation area result has the wrong identity"
        )
    raw_suites = payload.get("suites")
    if not isinstance(raw_suites, list):
        raise DocumentationQualificationError("documentation suites must be an array")
    observed: dict[str, dict[str, Any]] = {}
    for suite in raw_suites:
        if not isinstance(suite, dict):
            raise DocumentationQualificationError(
                "documentation suite result must be an object"
            )
        name = suite.get("name")
        if not isinstance(name, str) or name in observed:
            raise DocumentationQualificationError(
                "documentation suite names must be unique strings"
            )
        if suite.get("blocking") is not True:
            raise DocumentationQualificationError(
                f"documentation suite {name} must be blocking"
            )
        variants = suite.get("total_variants")
        failures = suite.get("total_failures")
        if type(variants) is not int or variants < 1:
            raise DocumentationQualificationError(
                f"documentation suite {name} emitted no evidence"
            )
        if type(failures) is not int or failures != 0:
            raise DocumentationQualificationError(
                f"documentation suite {name} did not pass"
            )
        observed[name] = suite
    if tuple(observed) != EXPECTED_SUITES:
        raise DocumentationQualificationError(
            "documentation suites must be structure then ga-release"
        )
    summary = payload.get("summary")
    if (
        not isinstance(summary, dict)
        or summary.get("blocking_failures") != 0
        or type(summary.get("total_variants")) is not int
        or summary["total_variants"] < len(EXPECTED_SUITES)
    ):
        raise DocumentationQualificationError(
            "documentation result summary is not passing"
        )
    return [
        {
            "name": name,
            "status": "pass",
            "total_variants": observed[name]["total_variants"],
        }
        for name in EXPECTED_SUITES
    ]


def qualify(args: argparse.Namespace) -> dict[str, Any]:
    source_root = Path(args.source_root).resolve()
    output = Path(args.out).resolve()
    result_path = source_root / "target" / "verification" / "areas" / (
        "documentation-stable-qualification-results.json"
    )
    if not COMMIT_RE.fullmatch(args.source_commit):
        raise DocumentationQualificationError(
            "source commit must be exact lowercase 40-hex"
        )
    if output.exists():
        raise DocumentationQualificationError("output path already exists")
    if output.is_relative_to(source_root):
        raise DocumentationQualificationError(
            "documentation evidence must be written outside the source checkout"
        )
    if git_output(source_root, "rev-parse", "HEAD") != args.source_commit:
        raise DocumentationQualificationError(
            "source checkout does not match source commit"
        )
    if git_output(source_root, "status", "--porcelain", "--untracked-files=all"):
        raise DocumentationQualificationError(
            "stable documentation qualification requires a clean source checkout"
        )

    command = [
        "uv",
        "run",
        "--project",
        "verification",
        "--locked",
        "python",
        "-m",
        "sifr_verify",
        "areas",
        "run",
        "--area",
        "documentation",
        "--suite",
        "structure",
        "--suite",
        "ga-release",
        "--result-json",
        str(result_path.relative_to(source_root)),
    ]
    result = subprocess.run(
        command,
        cwd=source_root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise DocumentationQualificationError(
            "documentation gate failed: "
            + (result.stderr or result.stdout).strip()
        )
    try:
        result_payload = json.loads(result_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise DocumentationQualificationError(
            f"documentation result cannot be read: {exc}"
        ) from exc
    suite_results = validate_result(result_payload)
    result_digest = sha256_bytes(result_path.read_bytes())
    payload = {
        "schema_version": 2,
        "kind": "stable-documentation-qualification",
        "report_id": f"docs-{args.source_commit[:12]}-{result_digest[:12]}",
        "source_commit": args.source_commit,
        "suites": suite_results,
        "result_sha256": result_digest,
        "status": "pass",
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(canonical_bytes(payload))
    return payload


def run_self_test() -> None:
    valid = {
        "area": "documentation",
        "suites": [
            {
                "name": name,
                "blocking": True,
                "total_variants": 1,
                "total_failures": 0,
            }
            for name in EXPECTED_SUITES
        ],
        "summary": {"blocking_failures": 0, "total_variants": 2},
    }
    validate_result(valid)
    mutations = (
        {**valid, "area": "other"},
        {**valid, "suites": valid["suites"][:1]},
        {
            **valid,
            "suites": [
                valid["suites"][0],
                {**valid["suites"][1], "total_failures": 1},
            ],
        },
        {**valid, "summary": {"blocking_failures": 0, "total_variants": True}},
    )
    for mutation in mutations:
        try:
            validate_result(mutation)
        except DocumentationQualificationError:
            continue
        raise DocumentationQualificationError(
            "invalid documentation result mutation passed"
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--source-root", default=str(REPO_ROOT))
    parser.add_argument("--source-commit")
    parser.add_argument("--out")
    args = parser.parse_args()
    if not args.self_test and (args.source_commit is None or args.out is None):
        parser.error("--source-commit and --out are required")
    return args


def main() -> int:
    args = parse_args()
    try:
        if args.self_test:
            run_self_test()
            print("stable documentation qualification self-test: PASS")
        else:
            payload = qualify(args)
            print(
                "stable documentation qualification: PASS "
                f"report_id={payload['report_id']}"
            )
    except (DocumentationQualificationError, OSError, UnicodeError) as exc:
        print(f"stable documentation qualification: FAIL: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
