#!/usr/bin/env python3
"""Verify checked-in diagnostic-code docs match the Rust registry."""

from __future__ import annotations

import pathlib
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[4]


def main() -> int:
    checked = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "sifr_diagnostics",
            "--bin",
            "gen-error-docs",
            "--",
            "--check",
        ],
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if checked.returncode != 0:
        sys.stderr.write(
            "diagnostic docs are out of sync. Run "
            "`cargo run -q -p sifr_diagnostics --bin gen-error-docs`.\n"
        )
        sys.stderr.write(checked.stderr)
        return checked.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
