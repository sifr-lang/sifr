#!/usr/bin/env python3
"""Validate the Sifr formatter rules."""

from __future__ import annotations

import argparse
import contextlib
import io
import subprocess
import sys
import tempfile
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[3]


def run(command: list[str], *, expect: int = 0) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(command, cwd=REPO_ROOT, text=True, capture_output=True)
    if completed.returncode != expect:
        print(f"formatter rules command failed (expected exit {expect}, observed {completed.returncode}): {' '.join(command)}", file=sys.stderr)
        print(completed.stdout, file=sys.stderr)
        print(completed.stderr, file=sys.stderr)
        raise SystemExit(1)
    return completed


def run_positive() -> None:
    run(["cargo", "test", "-p", "sifr_format"])
    with tempfile.TemporaryDirectory() as tmp:
        source = Path(tmp) / "main.sifr"
        source.write_text('def main():  \n    value: str = "kept "  \n    print(value)', encoding="utf-8")
        first = run(["cargo", "run", "-q", "-p", "sifr", "--", "fmt", "--check", str(source)], expect=1)
        if "SIFR-FMT-0001" not in first.stderr and "source is not formatted" not in first.stderr:
            raise SystemExit("formatter rules failed: unformatted source did not report formatting drift")
        run(["cargo", "run", "-q", "-p", "sifr", "--", "fmt", str(source)])
        formatted = source.read_text(encoding="utf-8")
        if '"kept "' not in formatted:
            raise SystemExit("formatter rules failed: string contents were changed")
        run(["cargo", "run", "-q", "-p", "sifr", "--", "fmt", "--check", str(source)])
        second = source.read_text(encoding="utf-8")
        if formatted != second:
            raise SystemExit("formatter rules failed: formatter is not idempotent")
    print("formatter rules: PASS")


def run_exit_status_self_test() -> None:
    for actual, expected in ((0, 0), (1, 1), (0, 1), (1, 0), (2, 1), (-9, 0)):
        completed = subprocess.CompletedProcess(["exit-status-probe"], actual, "", "")
        diagnostic = io.StringIO()
        with patch.object(subprocess, "run", return_value=completed), contextlib.redirect_stderr(diagnostic):
            if actual == expected:
                assert run(["exit-status-probe"], expect=expected) is completed
            else:
                try:
                    run(["exit-status-probe"], expect=expected)
                except SystemExit as error:
                    assert error.code == 1, (actual, expected, error.code)
                else:
                    raise AssertionError(("unexpected child status accepted", actual, expected))
                assert f"expected exit {expected}, observed {actual}" in diagnostic.getvalue()
    print("formatter exit-status self-test: PASS")


def run_self_test() -> None:
    run_exit_status_self_test()
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
    print("formatter rules self-test: PASS")


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
