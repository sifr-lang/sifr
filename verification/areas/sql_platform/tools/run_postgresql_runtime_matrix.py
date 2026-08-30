#!/usr/bin/env python3
"""Run the real PostgreSQL runtime bridge against PostgreSQL 13 through 18."""

from __future__ import annotations

import json
import subprocess
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
SERVER_MATRIX = REPO_ROOT / "verification/areas/sql_platform/data/postgresql_server_matrix.json"
PASSWORD = "sifr-postgresql-runtime"
SETUP_SQL = """
CREATE TABLE sifr_runtime_probe (id bigint PRIMARY KEY, value bigint NOT NULL);
INSERT INTO sifr_runtime_probe(id, value) VALUES (1, 0);
"""


def main() -> int:
    require_docker()
    servers = json.loads(SERVER_MATRIX.read_text(encoding="utf-8"))["servers"]
    for server in servers:
        run_server(server)
    print(f"PostgreSQL runtime matrix passed: majors={len(servers)}")
    return 0


def run_server(server: dict[str, object]) -> None:
    major = int(server["major"])
    image = f"{server['image']}@{server['image_digest']}"
    name = f"sifr-postgresql-runtime-{major}-{subprocess.check_output(['id', '-u'], text=True).strip()}"
    subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)
    try:
        run(["docker", "pull", image])
        run([
            "docker", "run", "--detach", "--name", name,
            "--publish", "127.0.0.1::5432",
            "--env", f"POSTGRES_PASSWORD={PASSWORD}", image,
        ])
        wait_ready(name)
        psql(name, SETUP_SQL)
        port = run(["docker", "port", name, "5432/tcp"]).stdout.strip().rsplit(":", 1)[-1]
        url = f"postgresql://postgres:{PASSWORD}@127.0.0.1:{port}/postgres"
        command = [
            "cargo", "test", "--locked", "-p", "sifr_sql_postgresql_runtime",
            "--test", "live_runtime", "--", "--ignored", "--exact",
            "live_postgresql_runtime_contract", "--nocapture",
        ]
        result = subprocess.run(
            command,
            cwd=REPO_ROOT,
            check=False,
            text=True,
            env={**__import__("os").environ, "SIFR_POSTGRESQL_TEST_URL": url},
        )
        if result.returncode != 0:
            raise RuntimeError(f"PostgreSQL {major} runtime test failed")
        print(f"PostgreSQL {major} runtime passed", flush=True)
    finally:
        subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)


def psql(container: str, sql: str) -> None:
    run([
        "docker", "exec", "--env", f"PGPASSWORD={PASSWORD}", container,
        "psql", "--username", "postgres", "--dbname", "postgres", "--no-psqlrc",
        "--quiet", "--set", "ON_ERROR_STOP=1", "--command", sql,
    ])


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
        raise RuntimeError("Docker daemon is required for the PostgreSQL runtime matrix")


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}\n{result.stderr}")
    return result


if __name__ == "__main__":
    raise SystemExit(main())
