#!/usr/bin/env python3
"""Run the MySQL differential and live provider suites on every supported series."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = ROOT / "verification/areas/sql_platform/data/mysql_server_matrix.json"
PASSWORD = "sifr-mysql-qualification"
SERVERS = [("8.4", "mysql:8.4"), ("9.7", "mysql:9.7"), ("26.7", "mysql:26.7")]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--surface",
        choices=["differential", "runtime", "schema-tools", "migrations", "all"],
        default="all",
    )
    parser.add_argument("--write", action="store_true", help="Replace checked-in evidence.")
    args = parser.parse_args()
    require_docker()
    rows = [run_server(series, image, args.surface) for series, image in SERVERS]
    payload = {
        "schema_version": 1,
        "authority": "live-mysql-provider-matrix",
        "surface": args.surface,
        "servers": rows,
    }
    rendered = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if args.write:
        EVIDENCE.write_text(rendered, encoding="utf-8")
        print(f"wrote {EVIDENCE.relative_to(ROOT)}")
    else:
        print(rendered, end="")
    return 0


def run_server(series: str, image: str, surface: str) -> dict[str, Any]:
    ensure_image(image)
    name = f"sifr-mysql-{series.replace('.', '-')}-{subprocess.check_output(['id', '-u'], text=True).strip()}"
    subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)
    try:
        run([
            "docker", "run", "--detach", "--name", name,
            "--publish", "127.0.0.1::3306",
            "--env", f"MYSQL_ROOT_PASSWORD={PASSWORD}",
            "--env", "MYSQL_DATABASE=app", image,
        ])
        wait_ready(name)
        host_port = run(["docker", "port", name, "3306/tcp"]).stdout.strip().rsplit(":", 1)[-1]
        mysql(name, "CREATE TABLE users(id BIGINT UNSIGNED PRIMARY KEY, email VARCHAR(255) NOT NULL UNIQUE); INSERT INTO users VALUES (1, 'first@example.test');")
        server_version = mysql(name, "SELECT VERSION();")
        env = {
            "SIFR_MYSQL_TEST_URL": f"mysql://root:{PASSWORD}@127.0.0.1:{host_port}/app",
            "SIFR_MYSQL_TEST_SERIES": series,
            "SIFR_MYSQL_TEST_CONTAINER": name,
        }
        if surface in {"differential", "all"}:
            run_cargo(["-p", "sifr_sql_mysql"], env)
        if surface in {"runtime", "all"}:
            run_cargo(["-p", "sifr_sql_mysql_runtime"], env)
        if surface in {"schema-tools", "migrations", "all"}:
            run_cargo(["-p", "sifr_sql_mysql_tools"], env)
        print(f"MySQL {series} {surface} matrix passed", flush=True)
        return {
            "series": series,
            "image": image,
            "image_digest": image_digest(image),
            "server_version": server_version,
            "surface": surface,
            "status": "passed",
        }
    finally:
        subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)


def mysql(container: str, sql: str) -> str:
    return run([
        "docker", "exec", "--env", f"MYSQL_PWD={PASSWORD}", container,
        "mysql", "--user=root", "--database=app", "--batch", "--skip-column-names",
        "--execute", sql,
    ]).stdout.strip()


def wait_ready(container: str) -> None:
    for _ in range(120):
        logs = subprocess.run(
            ["docker", "logs", container],
            check=False,
            capture_output=True,
            text=True,
        )
        initialized = "MySQL init process done. Ready for start up." in (
            logs.stdout + logs.stderr
        )
        probe = subprocess.run(
            [
                "docker", "exec", "--env", f"MYSQL_PWD={PASSWORD}", container,
                "mysql", "--user=root", "--database=app", "--batch",
                "--skip-column-names", "--execute", "SELECT 1",
            ],
            check=False,
            capture_output=True,
        )
        if initialized and probe.returncode == 0:
            return
        time.sleep(0.5)
    raise RuntimeError(f"MySQL container did not become ready: {container}")


def run_cargo(packages: list[str], env: dict[str, str]) -> None:
    command = [
        "cargo", "test", "--locked", *packages, "--",
        "--include-ignored", "--test-threads=1",
    ]
    complete_env = dict(os.environ)
    complete_env.update(env)
    result = subprocess.run(command, cwd=ROOT, env=complete_env, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}")


def require_docker() -> None:
    if subprocess.run(["docker", "info"], check=False, capture_output=True).returncode != 0:
        raise RuntimeError("Docker daemon is required for the live MySQL matrix")


def ensure_image(image: str) -> None:
    if subprocess.run(
        ["docker", "image", "inspect", image], check=False, capture_output=True
    ).returncode != 0:
        run(["docker", "pull", image])


def image_digest(image: str) -> str:
    result = run(["docker", "image", "inspect", image, "--format", "{{index .RepoDigests 0}}"])
    value = result.stdout.strip()
    if "@" not in value:
        raise RuntimeError(f"image has no repository digest: {image}")
    return value.split("@", 1)[1]


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}\n{result.stderr}")
    return result


if __name__ == "__main__":
    raise SystemExit(main())
