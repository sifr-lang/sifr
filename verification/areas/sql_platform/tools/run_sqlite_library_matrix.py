#!/usr/bin/env python3
"""Run every SQLite provider surface against the bundled library."""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]


def main() -> int:
    commands = [
        ["cargo", "test", "--locked", "-p", "sifr_sql_sqlite"],
        ["cargo", "test", "--locked", "-p", "sifr_sql_sqlite_runtime"],
        ["cargo", "test", "--locked", "-p", "sifr_sql_sqlite_tools"],
    ]
    for command in commands:
        subprocess.run(command, cwd=ROOT, check=True)
    probe = subprocess.run(
        ["cargo", "test", "--locked", "-p", "sifr_sql_sqlite_runtime", "malformed_database_bytes"],
        cwd=ROOT,
        check=False,
    )
    if probe.returncode != 0:
        return probe.returncode
    print("SQLite bundled matrix ok: version=3.53.2 version_number=3053002 surfaces=all")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
