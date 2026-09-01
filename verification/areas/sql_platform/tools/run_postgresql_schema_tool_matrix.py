#!/usr/bin/env python3
"""Qualify schema catalog pull against PostgreSQL 13 through 18."""

from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = REPO_ROOT / "verification/areas/sql_platform/data/postgresql_schema_tool_matrix.json"
PASSWORD = "sifr-schema-tool-qualification"

SETUP_SQL = r"""
CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');
CREATE DOMAIN email_address AS text CHECK (VALUE LIKE '%@%');
CREATE DOMAIN positive_id AS integer CHECK (VALUE > 0);
CREATE TYPE postal_address AS (street text, city text, unit_count integer, latitude numeric);
CREATE TYPE price_range AS RANGE (subtype = numeric);
CREATE DOMAIN price_window AS price_range;
CREATE COLLATION sifr_c (provider = libc, locale = 'C');
CREATE SEQUENCE audit_sequence;
CREATE TABLE accounts (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  email email_address UNIQUE NOT NULL,
  state mood NOT NULL,
  balance numeric CHECK (balance >= 0)
);
CREATE TABLE orders (
  id bigint PRIMARY KEY,
  account_id bigint REFERENCES accounts(id)
);
CREATE TABLE parity_users (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  name text NOT NULL,
  score integer CHECK (score >= 0)
);
CREATE TABLE type_samples (
  id bigint PRIMARY KEY,
  domain_values positive_id[],
  composite_values postal_address[],
  window price_window
);
CREATE INDEX orders_account_idx ON orders(account_id);
CREATE VIEW account_view AS SELECT id, email FROM accounts;
CREATE VIEW parity_user_view AS SELECT id, name, score FROM parity_users;
CREATE MATERIALIZED VIEW account_count AS SELECT count(*) AS count FROM accounts;
CREATE FUNCTION add_one(integer) RETURNS integer LANGUAGE SQL IMMUTABLE STRICT AS 'SELECT $1 + 1';
CREATE FUNCTION add_one(bigint) RETURNS bigint LANGUAGE SQL IMMUTABLE STRICT AS 'SELECT $1 + 1';
CREATE FUNCTION integer_same(integer, integer) RETURNS boolean LANGUAGE SQL IMMUTABLE STRICT AS 'SELECT $1 = $2';
CREATE OPERATOR === (LEFTARG = integer, RIGHTARG = integer, FUNCTION = integer_same);
CREATE FUNCTION postal_address_text(postal_address) RETURNS text LANGUAGE SQL IMMUTABLE STRICT AS 'SELECT ($1).street || '', '' || ($1).city';
CREATE CAST (postal_address AS text) WITH FUNCTION postal_address_text(postal_address) AS ASSIGNMENT;
CREATE FUNCTION audit_accounts() RETURNS trigger LANGUAGE plpgsql AS 'BEGIN RETURN NEW; END';
CREATE TRIGGER accounts_audit BEFORE INSERT ON accounts FOR EACH ROW EXECUTE FUNCTION audit_accounts();
"""


def main() -> int:
    require_docker()
    rows = [run_server(major) for major in range(13, 19)]
    payload = {
        "schema_version": 1,
        "authority": "postgresql-schema-tool-live-catalog",
        "servers": rows,
    }
    rendered = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if "--write" in __import__("sys").argv:
        EVIDENCE.write_text(rendered, encoding="utf-8")
        print(f"wrote {EVIDENCE.relative_to(REPO_ROOT)}")
    else:
        print(rendered, end="")
    return 0


def run_server(major: int) -> dict[str, Any]:
    print(f"qualifying PostgreSQL {major}", flush=True)
    image = f"postgres:{major}"
    ensure_image(image)
    uid = subprocess.check_output(["id", "-u"], text=True).strip()
    name = f"sifr-schema-tool-{major}-{uid}"
    subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)
    try:
        run([
            "docker", "run", "--detach", "--name", name,
            "--publish", "127.0.0.1::5432",
            "--env", f"POSTGRES_PASSWORD={PASSWORD}", image,
        ])
        wait_ready(name)
        psql(name, SETUP_SQL)
        port = run(["docker", "port", name, "5432/tcp"]).stdout.strip().rsplit(":", 1)[-1]
        url = f"postgresql://postgres:{PASSWORD}@127.0.0.1:{port}/postgres?sslmode=disable"
        environment = {
            **os.environ,
            "SIFR_POSTGRESQL_SCHEMA_TOOL_TEST_URL": url,
            "SIFR_POSTGRESQL_SCHEMA_TOOL_TEST_MAJOR": str(major),
        }
        run([
            "cargo", "test", "--locked", "-p", "sifr_sql_postgresql_tools",
            "--test", "live_catalog", "--", "--ignored", "--exact",
            "live_catalog_preserves_postgresql_semantic_objects",
        ], env=environment, cwd=REPO_ROOT)
        return {
            "major": major,
            "image": image,
            "image_digest": image_digest(image),
            "status": "passed",
        }
    finally:
        subprocess.run(["docker", "rm", "-f", name], check=False, capture_output=True)


def psql(container: str, sql: str) -> None:
    run([
        "docker", "exec", "--env", f"PGPASSWORD={PASSWORD}", container,
        "psql", "--username", "postgres", "--dbname", "postgres",
        "--no-psqlrc", "--quiet", "--set", "ON_ERROR_STOP=1", "--command", sql,
    ])


def wait_ready(container: str) -> None:
    for _ in range(120):
        logs = subprocess.run(
            ["docker", "logs", container], check=False, capture_output=True, text=True,
        )
        init_complete = "PostgreSQL init process complete; ready for start up." in (
            logs.stdout + logs.stderr
        )
        result = subprocess.run(
            [
                "docker", "exec", "--env", f"PGPASSWORD={PASSWORD}", container,
                "psql", "--username", "postgres", "--dbname", "postgres",
                "--no-psqlrc", "--quiet", "--command", "SELECT 1;",
            ],
            check=False, capture_output=True,
        )
        if init_complete and result.returncode == 0:
            return
        time.sleep(0.5)
    raise RuntimeError(f"PostgreSQL container {container} did not become ready")


def ensure_image(image: str) -> None:
    if subprocess.run(["docker", "image", "inspect", image], check=False, capture_output=True).returncode != 0:
        run(["docker", "pull", image])


def image_digest(image: str) -> str:
    output = run(["docker", "image", "inspect", image, "--format", "{{json .RepoDigests}}"])
    digests = json.loads(output.stdout)
    return sorted(digests)[0] if digests else image


def require_docker() -> None:
    result = subprocess.run(["docker", "info"], check=False, capture_output=True)
    if result.returncode != 0:
        raise RuntimeError("Docker is required for PostgreSQL schema tool qualification")


def run(command: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, check=False, capture_output=True, text=True, **kwargs)
    if result.returncode != 0:
        rendered = " ".join(command)
        raise RuntimeError(
            f"command failed ({result.returncode}): {rendered}\n{result.stdout}\n{result.stderr}"
        )
    return result


if __name__ == "__main__":
    raise SystemExit(main())
