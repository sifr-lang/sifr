#!/usr/bin/env python3
"""Validate Sifr policy-rule and suppression behavior."""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
DEFAULT_SIFR_BIN = REPO_ROOT / "target" / "debug" / "sifr"
sys.path.insert(0, str(REPO_ROOT / "verification" / "areas" / "common"))

from sifr_binary import resolve_sifr_binary  # noqa: E402


def run(command: list[str], *, expect: int = 0) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(command, cwd=REPO_ROOT, text=True, capture_output=True)
    if completed.returncode != expect:
        print(f"rule/suppression rules command failed: {' '.join(command)}", file=sys.stderr)
        print(completed.stdout, file=sys.stderr)
        print(completed.stderr, file=sys.stderr)
        raise SystemExit(completed.returncode)
    return completed


def sifr_command(*args: str) -> list[str]:
    binary = resolve_sifr_binary(
        REPO_ROOT,
        explicit_env_var="SIFR_RULE_SUPPRESSION_BIN",
        default_binary=DEFAULT_SIFR_BIN,
    )
    return [str(binary), *args]


def run_positive() -> None:
    run(["cargo", "test", "-p", "sifr_lint"])
    with tempfile.TemporaryDirectory() as tmp:
        source = Path(tmp) / "main.sifr"
        source.write_text(
            "def main():  # sifr: ignore[not-a-rule]\n    pass  \n",
            encoding="utf-8",
        )
        completed = run(sifr_command("lint", str(source)), expect=1)
        stderr = completed.stderr
        for expected in [
            "unknown Sifr policy rule id",
            "line has trailing whitespace",
        ]:
            if expected not in stderr:
                raise SystemExit(f"rule/suppression rules failed: missing {expected!r}")

        suppressed = Path(tmp) / "suppressed.sifr"
        suppressed.write_text(
            "def main():  # sifr: ignore[trailing-whitespace]  \n    pass\n",
            encoding="utf-8",
        )
        completed = run(sifr_command("lint", str(suppressed)))
        if "line has trailing whitespace" in completed.stderr:
            raise SystemExit("rule/suppression rules failed: explicit policy suppression did not apply")
    print("rule/suppression rules: PASS")


def run_self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        source = Path(tmp) / "blanket.sifr"
        source.write_text("def main(): # sifr: ignore\n    pass\n", encoding="utf-8")
        completed = run(sifr_command("lint", str(source)), expect=1)
        if "sifr suppression must list explicit policy rule ids" not in completed.stderr:
            raise SystemExit("rule/suppression self-test failed: blanket suppression passed")
    print("rule/suppression rules self-test: PASS")


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
