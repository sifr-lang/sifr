#!/usr/bin/env python3
"""Prepare native executables separately from the compiler, then test their real sandbox."""
from __future__ import annotations

import argparse
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[4]
NATIVE_BUILD = [
    "cargo", "build", "--locked", "--offline",
    "-p", "sifr_sql_sqlite_tools", "--bin", "sifr-sql-sqlite",
    "-p", "sifr_sql_mysql_tools", "--bin", "sifr-sql-mysql",
    "-p", "sifr_sql_postgresql_tools", "--bin", "sifr-sql-postgresql",
]
TESTS = [
    "cargo", "test", "--locked", "--offline", "-p", "sifr",
    "--test", "sql_native_application", "compiler_sandbox",
]

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare-only", action="store_true")
    args = parser.parse_args()
    subprocess.run(NATIVE_BUILD, cwd=ROOT, check=True)
    # These are distinct Cargo graphs. Combining the compiler with the three
    # native tools creates additional dependency variants and unnecessary rebuilds.
    subprocess.run([*TESTS[:2], "--no-run", *TESTS[2:]] if args.prepare_only else TESTS,
                   cwd=ROOT, check=True)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
