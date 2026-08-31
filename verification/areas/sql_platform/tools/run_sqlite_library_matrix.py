#!/usr/bin/env python3
"""Run every SQLite provider surface against the bundled library."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = ROOT / "verification/areas/sql_platform/data/sqlite_library_matrix.json"


def main() -> int:
    commands = [
        ["cargo", "test", "--locked", "-p", "sifr_sql_sqlite"],
        ["cargo", "test", "--locked", "-p", "sifr_sql_sqlite_runtime"],
        ["cargo", "test", "--locked", "-p", "sifr_sql_sqlite_tools"],
    ]
    for command in commands:
        subprocess.run(command, cwd=ROOT, check=True)
    probe = subprocess.run(
        ["cargo", "run", "--locked", "-q", "-p", "sifr_sql_sqlite_tools", "--bin", "sqlite-runtime-probe"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    observed = json.loads(probe.stdout)
    observed["compile_options"] = [
        option for option in observed["compile_options"] if not option.startswith("COMPILER=")
    ]
    evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))["libraries"][0]
    expected = {
        "version": evidence["version"],
        "version_number": evidence["version_number"],
        "compile_options": evidence["runtime_compile_options"],
    }
    if observed != expected:
        raise RuntimeError("bundled SQLite runtime identity differs from checked-in evidence")
    print(
        "SQLite bundled matrix ok: "
        f"version={observed['version']} version_number={observed['version_number']} "
        f"compile_options={len(observed['compile_options'])} surfaces=all"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
