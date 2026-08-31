#!/usr/bin/env python3
"""Run the PostgreSQL migration contract on every supported server major."""

from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
SERVER_MATRIX = REPO_ROOT / "verification/areas/sql_platform/data/postgresql_server_matrix.json"
MIGRATION_MATRIX = REPO_ROOT / "verification/areas/sql_platform/data/postgresql_migration_matrix.json"
PASSWORD = "sifr-postgresql-migrations"


def main() -> int:
    require_docker()
    servers = json.loads(SERVER_MATRIX.read_text(encoding="utf-8"))["servers"]
    expected = json.loads(MIGRATION_MATRIX.read_text(encoding="utf-8"))["servers"]
    require_matrix_match(servers, expected)
    for server in servers:
        run_server(server)
    print(f"PostgreSQL migration matrix passed: majors={len(servers)}")
    return 0


def require_matrix_match(servers: list[dict[str, Any]], expected: list[dict[str, Any]]) -> None:
    actual_rows = [
        (row["major"], row["image"], row["image_digest"], "passed") for row in servers
    ]
    expected_rows = [
        (row["major"], row["image"], row["image_digest"], row["status"]) for row in expected
    ]
    if actual_rows != expected_rows:
        raise RuntimeError("PostgreSQL migration matrix differs from the pinned server authority")


def run_server(server: dict[str, Any]) -> None:
    major = int(server["major"])
    image = f"{server['image']}@{server['image_digest']}"
    user = subprocess.check_output(["id", "-u"], text=True).strip()
    name = f"sifr-postgresql-migrations-{major}-{user}"
    subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)
    try:
        run(["docker", "pull", image])
        run([
            "docker", "run", "--detach", "--name", name,
            "--publish", "127.0.0.1::5432",
            "--env", f"POSTGRES_PASSWORD={PASSWORD}", image,
        ])
        wait_ready(name)
        port = run(["docker", "port", name, "5432/tcp"]).stdout.strip().rsplit(":", 1)[-1]
        url = f"postgresql://postgres:{PASSWORD}@127.0.0.1:{port}/postgres?sslmode=disable"
        command = [
            "cargo", "test", "--locked", "-p", "sifr_sql_postgresql_tools",
            "--test", "live_migrations", "--", "--ignored", "--exact",
            "live_postgresql_migration_contract", "--nocapture",
        ]
        result = subprocess.run(
            command,
            cwd=REPO_ROOT,
            check=False,
            text=True,
            env={
                **os.environ,
                "SIFR_POSTGRESQL_MAJOR": str(major),
                "SIFR_POSTGRESQL_MIGRATION_TEST_MAJOR": str(major),
                "SIFR_POSTGRESQL_MIGRATION_TEST_URL": url,
            },
        )
        if result.returncode != 0:
            raise RuntimeError(f"PostgreSQL {major} migration test failed")
        print(f"PostgreSQL {major} migrations passed", flush=True)
    finally:
        subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)


def wait_ready(container: str) -> None:
    for _ in range(120):
        result = subprocess.run(
            ["docker", "exec", container, "pg_isready", "--username", "postgres"],
            check=False,
            capture_output=True,
        )
        if result.returncode == 0:
            return
        time.sleep(0.5)
    raise RuntimeError(f"PostgreSQL container did not become ready: {container}")


def require_docker() -> None:
    if subprocess.run(["docker", "info"], check=False, capture_output=True).returncode != 0:
        raise RuntimeError("Docker daemon is required for the PostgreSQL migration matrix")


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}\n{result.stderr}")
    return result


if __name__ == "__main__":
    raise SystemExit(main())
