#!/usr/bin/env python3
"""Validate stdlib namespace cleanup across checked-in example corpora."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
LEETCODE_ROOT = REPO_ROOT / "audits" / "leetcode" / "src"
DEMOS_ROOT = REPO_ROOT / "demos"
DEFAULT_SIFR_BIN = REPO_ROOT / "target" / "debug" / "sifr"

BARE_STDLIB_ATTR_RE = re.compile(
    r"(?<!sifr\.)\b(?P<module>math|heapq|collections)\.[A-Za-z_]"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--scope",
        choices=("all", "leetcode", "demos"),
        default="all",
        help="Corpus scope to validate.",
    )
    parser.add_argument(
        "--command",
        choices=("run", "check"),
        default="run",
        help="Sifr command to execute for every discovered fixture.",
    )
    parser.add_argument(
        "--sifr-bin",
        default=str(DEFAULT_SIFR_BIN),
        help="Path to the Sifr CLI binary. Built with cargo if missing.",
    )
    parser.add_argument(
        "--fail-fast",
        action="store_true",
        help="Stop at the first failed fixture.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="Print discovered fixtures without running them.",
    )
    return parser.parse_args()


def rel(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def discover_leetcode() -> list[Path]:
    if not LEETCODE_ROOT.is_dir():
        raise SystemExit(f"missing LeetCode corpus root: {rel(LEETCODE_ROOT)}")
    return sorted(LEETCODE_ROOT.glob("*.sifr"))


def discover_demos() -> list[Path]:
    if not DEMOS_ROOT.is_dir():
        raise SystemExit(f"missing demo root: {rel(DEMOS_ROOT)}")
    return sorted(
        path
        for path in DEMOS_ROOT.rglob("main.sifr")
        if "negative_cases" not in path.relative_to(DEMOS_ROOT).parts
    )


def scan_bare_stdlib_attrs(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            stripped = line.strip()
            if stripped.startswith("#"):
                continue
            match = BARE_STDLIB_ATTR_RE.search(line)
            if match:
                failures.append(
                    f"{rel(path)}:{line_no}: bare stdlib module attribute `{match.group(0)}`"
                )
    return failures


def ensure_sifr_bin(sifr_bin: Path) -> None:
    if sifr_bin == DEFAULT_SIFR_BIN:
        subprocess.run(["cargo", "build", "-q", "-p", "sifr"], cwd=REPO_ROOT, check=True)
    elif not sifr_bin.exists():
        raise SystemExit(f"missing Sifr CLI binary: {sifr_bin}")


def run_fixture(sifr_bin: Path, command: str, path: Path) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("SIFR_ARTIFACT_CACHE", "1")
    return subprocess.run(
        [str(sifr_bin), command, rel(path)],
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def main() -> int:
    args = parse_args()
    paths: list[Path] = []
    if args.scope in ("all", "leetcode"):
        paths.extend(discover_leetcode())
    if args.scope in ("all", "demos"):
        paths.extend(discover_demos())

    if args.list:
        for path in paths:
            print(rel(path))
        return 0

    scan_failures = scan_bare_stdlib_attrs(paths)
    if scan_failures:
        print("bare stdlib namespace scan failed:", file=sys.stderr)
        for failure in scan_failures:
            print(f"  {failure}", file=sys.stderr)
        return 1

    sifr_bin = Path(args.sifr_bin)
    ensure_sifr_bin(sifr_bin)

    start = time.monotonic()
    failures: list[tuple[Path, subprocess.CompletedProcess[str]]] = []
    print(
        f"validating {len(paths)} fixture(s): scope={args.scope} command={args.command}",
        flush=True,
    )
    for index, path in enumerate(paths, start=1):
        result = run_fixture(sifr_bin, args.command, path)
        if result.returncode == 0:
            print(f"[{index}/{len(paths)}] PASS {rel(path)}", flush=True)
            continue
        print(f"[{index}/{len(paths)}] FAIL {rel(path)}", flush=True)
        failures.append((path, result))
        if args.fail_fast:
            break

    elapsed = time.monotonic() - start
    if failures:
        print(
            f"stdlib namespace corpus validation failed: "
            f"{len(paths) - len(failures)}/{len(paths)} passed in {elapsed:.1f}s",
            file=sys.stderr,
        )
        for path, result in failures:
            print(f"\n--- {rel(path)} exit={result.returncode} ---", file=sys.stderr)
            if result.stdout:
                print(result.stdout[-4000:], file=sys.stderr)
            if result.stderr:
                print(result.stderr[-4000:], file=sys.stderr)
        return 1

    print(
        f"stdlib namespace corpus validation passed: {len(paths)}/{len(paths)} "
        f"in {elapsed:.1f}s"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
