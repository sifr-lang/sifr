#!/usr/bin/env python3
"""Run PostgreSQL compiler facts against live PostgreSQL 13 through 18 servers."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = REPO_ROOT / "verification/areas/sql_platform/data/postgresql_server_matrix.json"
PASSWORD = "sifr-postgresql-qualification"

SETUP_SQL = """
CREATE TABLE users (
  id bigint PRIMARY KEY,
  name text NOT NULL,
  nickname text
);
INSERT INTO users(id, name) VALUES (1, 'first');
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="Replace the checked-in evidence.")
    args = parser.parse_args()
    require_docker()
    rows = [run_server(major) for major in range(13, 19)]
    payload = {
        "schema_version": 1,
        "authority": "live-postgresql-server-differential",
        "servers": rows,
    }
    rendered = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if args.write:
        EVIDENCE.write_text(rendered, encoding="utf-8")
        print(f"wrote {EVIDENCE.relative_to(REPO_ROOT)}")
    else:
        print(rendered, end="")
    return 0


def run_server(major: int) -> dict[str, Any]:
    image = f"postgres:{major}"
    ensure_image(image)
    name = f"sifr-postgresql-{major}-{subprocess.check_output(['id', '-u'], text=True).strip()}"
    subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)
    try:
        run([
            "docker", "run", "--detach", "--name", name,
            "--env", f"POSTGRES_PASSWORD={PASSWORD}", image,
        ])
        wait_ready(name)
        psql(name, SETUP_SQL)
        server_version_num = psql(name, "SHOW server_version_num;")
        parameter_types = psql(
            name,
            "PREPARE sifr_query(bigint, text) AS SELECT id, name, nickname FROM users WHERE id = $1 AND name = $2; SELECT parameter_types::text FROM pg_prepared_statements WHERE name = 'sifr_query';",
        )
        result_types = psql(
            name,
            "SELECT pg_typeof(id)::text || '|' || pg_typeof(name)::text || '|' || pg_typeof(nickname)::text FROM users LIMIT 1;",
        )
        nullability = psql(
            name,
            "SELECT string_agg(attname || ':' || attnotnull::text, '|' ORDER BY attnum) FROM pg_attribute WHERE attrelid = 'users'::regclass AND attnum > 0 AND NOT attisdropped;",
        ).replace("true", "t").replace("false", "f")
        write_result = psql(
            name,
            "INSERT INTO users(id, name) VALUES (1, 'second') ON CONFLICT(id) DO UPDATE SET name = excluded.name RETURNING id::text || '|' || name;",
        )
        diagnostic_sqlstate = error_sqlstate(name, "SELECT missing_column FROM users;")
        digest = image_digest(image)
        row = {
            "major": major,
            "image": image,
            "image_digest": digest,
            "server_version_num": server_version_num,
            "parameter_types": parameter_types,
            "result_types": result_types,
            "nullability": nullability,
            "write_result": write_result,
            "diagnostic_sqlstate": diagnostic_sqlstate,
            "status": "passed",
        }
        print(f"PostgreSQL {major} live differential passed", flush=True)
        return row
    finally:
        subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)


def psql(container: str, sql: str) -> str:
    result = run([
        "docker", "exec", "--env", f"PGPASSWORD={PASSWORD}", container,
        "psql", "--username", "postgres", "--dbname", "postgres",
        "--no-psqlrc", "--quiet", "--tuples-only", "--no-align", "--set", "ON_ERROR_STOP=1",
        "--command", sql,
    ])
    lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    return lines[-1] if lines else ""


def error_sqlstate(container: str, sql: str) -> str:
    result = subprocess.run([
        "docker", "exec", "--env", f"PGPASSWORD={PASSWORD}", container,
        "psql", "--username", "postgres", "--dbname", "postgres", "--no-psqlrc",
        "--set", "VERBOSITY=verbose", "--command", sql,
    ], check=False, capture_output=True, text=True)
    if result.returncode == 0:
        raise RuntimeError("expected PostgreSQL diagnostic did not occur")
    match = re.search(r"ERROR:\s+([0-9A-Z]{5}):", result.stderr)
    if match is None:
        raise RuntimeError(f"PostgreSQL diagnostic has no SQLSTATE: {result.stderr}")
    return match.group(1)


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


def ensure_image(image: str) -> None:
    result = subprocess.run(["docker", "image", "inspect", image], check=False, capture_output=True)
    if result.returncode != 0:
        run(["docker", "pull", image])


def image_digest(image: str) -> str:
    result = run(["docker", "image", "inspect", image, "--format", "{{index .RepoDigests 0}}"])
    value = result.stdout.strip()
    if "@" not in value:
        raise RuntimeError(f"image has no repository digest: {image}")
    return value.split("@", 1)[1]


def require_docker() -> None:
    result = subprocess.run(["docker", "info"], check=False, capture_output=True)
    if result.returncode != 0:
        raise RuntimeError("Docker daemon is required for the live PostgreSQL matrix")


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}\n{result.stderr}")
    return result


if __name__ == "__main__":
    raise SystemExit(main())
