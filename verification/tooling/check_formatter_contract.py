#!/usr/bin/env python3
"""Validate the Sifr formatter contract."""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


def run(command: list[str], *, expect: int = 0) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(command, cwd=REPO_ROOT, text=True, capture_output=True)
    if completed.returncode != expect:
        print(f"formatter contract command failed: {' '.join(command)}", file=sys.stderr)
        print(completed.stdout, file=sys.stderr)
        print(completed.stderr, file=sys.stderr)
        raise SystemExit(completed.returncode)
    return completed


def run_positive() -> None:
    run(["cargo", "test", "-p", "sifr_format"])
    with tempfile.TemporaryDirectory() as tmp:
        source = Path(tmp) / "main.sifr"
        source.write_text('def main():  \n    value: str = "kept "  \n    print(value)', encoding="utf-8")
        first = run(["cargo", "run", "-q", "-p", "sifr", "--", "fmt", "--check", str(source)], expect=1)
        if "SIFR-FMT-0001" not in first.stderr and "source is not formatted" not in first.stderr:
            raise SystemExit("formatter contract failed: unformatted source did not report formatting drift")
        run(["cargo", "run", "-q", "-p", "sifr", "--", "fmt", str(source)])
        formatted = source.read_text(encoding="utf-8")
        if '"kept "' not in formatted:
            raise SystemExit("formatter contract failed: string contents were changed")
        run(["cargo", "run", "-q", "-p", "sifr", "--", "fmt", "--check", str(source)])
        second = source.read_text(encoding="utf-8")
        if formatted != second:
            raise SystemExit("formatter contract failed: formatter is not idempotent")
    print("formatter contract: PASS")


def run_self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        source = Path(tmp) / "bad.sifr"
        source.write_text("def main(:\n", encoding="utf-8")
        completed = run(
            ["cargo", "run", "-q", "-p", "sifr", "--", "fmt", "--check", str(source)],
            expect=1,
        )
        if (
            "SIFR-FMT-0001" not in completed.stderr
            and "formatter could not parse Sifr source" not in completed.stderr
        ):
            raise SystemExit("formatter self-test failed: invalid syntax did not fail closed")
    print("formatter contract self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
    else:
        run_positive()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
