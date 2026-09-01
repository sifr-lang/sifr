#!/usr/bin/env python3
"""Run the provider adapter against each embedded libpg_query source."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]


def main() -> int:
    for major in range(13, 19):
        environment = os.environ.copy()
        environment["SIFR_POSTGRESQL_MAJOR"] = str(major)
        command = [
            "cargo",
            "test",
            "--locked",
            "-p",
            "sifr_sql_postgresql",
            "--tests",
            "--",
            "--skip",
            "every_checked_in_component_executes_in_the_capability_free_host",
        ]
        print(f"PostgreSQL {major}: {' '.join(command)}", flush=True)
        result = subprocess.run(command, cwd=REPO_ROOT, env=environment, check=False)
        if result.returncode != 0:
            return result.returncode
    component_command = [
        "cargo",
        "test",
        "--locked",
        "-p",
        "sifr_sql_postgresql",
        "--test",
        "postgresql_components",
        "every_checked_in_component_executes_in_the_capability_free_host",
        "--",
        "--exact",
    ]
    print(f"PostgreSQL components: {' '.join(component_command)}", flush=True)
    result = subprocess.run(component_command, cwd=REPO_ROOT, check=False)
    if result.returncode != 0:
        return result.returncode
    print("PostgreSQL parser matrix ok: majors=6", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
