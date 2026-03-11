#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from phase31_leetcode_lib import (
    REPO_ROOT,
    build_runner_summary,
    load_seed_manifest,
    run_case,
    write_json,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the Phase 31 LeetCode corpus against the current sifr compiler."
    )
    parser.add_argument(
        "--manifest",
        default="verification/leetcode/phase31_seed_corpus.json",
        help="Path to the corpus manifest JSON, relative to the repo root.",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Where to write the structured JSON results, relative to the repo root.",
    )
    parser.add_argument(
        "--case",
        action="append",
        default=[],
        dest="case_ids",
        help="Optional case id filter. May be specified multiple times.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Optional case limit after deterministic sorting.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=20,
        help="Default timeout for each check/run stage.",
    )
    parser.add_argument(
        "--sifr-bin",
        default=None,
        help="Use an already-built sifr binary instead of resolving `target/release/sifr`.",
    )
    parser.add_argument(
        "--build-release-if-missing",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Build `target/release/sifr` once if it is missing.",
    )
    return parser.parse_args()


def resolve_command_prefix(
    sifr_bin: str | None, build_release_if_missing: bool
) -> list[str]:
    if sifr_bin:
        return [sifr_bin]

    release_bin = REPO_ROOT / "target" / "release" / "sifr"
    if release_bin.exists():
        return [str(release_bin)]

    if not build_release_if_missing:
        raise SystemExit(
            "missing target/release/sifr; rerun with --build-release-if-missing "
            "or provide --sifr-bin"
        )

    subprocess.run(
        ["cargo", "build", "--release", "-p", "sifr"],
        cwd=REPO_ROOT,
        check=True,
    )
    return [str(release_bin)]


def main() -> None:
    args = parse_args()
    manifest_path = REPO_ROOT / args.manifest
    output_path = REPO_ROOT / args.output
    cases = sorted(load_seed_manifest(manifest_path), key=lambda case: case["id"])
    if args.case_ids:
        requested = set(args.case_ids)
        cases = [case for case in cases if case["id"] in requested]
    if args.limit is not None:
        cases = cases[: args.limit]

    command_prefix = resolve_command_prefix(args.sifr_bin, args.build_release_if_missing)
    results = [run_case(case, command_prefix, args.timeout_seconds) for case in cases]
    payload = {
        "phase": 31,
        "manifest": args.manifest,
        "output": args.output,
        "command_prefix": command_prefix,
        "summary": build_runner_summary(results),
        "results": results,
    }
    write_json(output_path, payload)
    print(json.dumps(payload["summary"], sort_keys=True))


if __name__ == "__main__":
    main()
