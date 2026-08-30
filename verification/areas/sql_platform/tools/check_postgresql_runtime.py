#!/usr/bin/env python3
"""Validate the PostgreSQL runtime bridge and its locked qualification contract."""

from __future__ import annotations

import argparse
import copy
import json
import sys
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification/areas/sql_platform"
CONTRACT = AREA_ROOT / "data/postgresql_runtime_qualification.json"
ROOT_MANIFEST = REPO_ROOT / "Cargo.toml"
RUNTIME_ROOT = REPO_ROOT / "crates/sifr_sql_postgresql_runtime"
COMMON_ROOT = REPO_ROOT / "crates/sifr_sql_runtime"


class ContractError(ValueError):
    """The PostgreSQL runtime qualification contract is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def validate(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "schema_version must be 1")
    require(payload.get("runtime_crate") == "sifr_sql_postgresql_runtime", "runtime crate drift")
    require(payload.get("common_crate") == "sifr_sql_runtime", "common crate drift")
    require(payload.get("supported_server_majors") == list(range(13, 19)), "server matrix drift")
    require(
        payload.get("driver_dependencies") == {
            "postgres-types": ["derive"],
            "tokio-postgres": ["runtime"],
            "tokio-postgres-rustls": ["aws-lc-rs"],
        },
        "raw driver feature allowlist drift",
    )
    require(
        set(payload.get("execution_methods", []))
        == {"execute", "fetch-one", "fetch-optional", "bounded-fetch-all", "stream", "scalar", "warm"},
        "execution method inventory is incomplete",
    )
    guarantees = set(payload.get("runtime_guarantees", []))
    required = {
        "bounded-cleanup", "bounded-connections", "bounded-decoded-rows", "bounded-parameters",
        "bounded-statement-cache", "compatible-absence-facts", "consuming-transaction-terminals",
        "fresh-transaction-retry", "panic-safe-codecs", "redacted-errors-and-debug",
        "session-reapply-and-reset", "task-local-connections-transactions-streams",
        "typed-schema-verification", "verified-execution-only",
    }
    require(guarantees == required, "runtime guarantee inventory is incomplete")
    validate_sources(payload)


def validate_sources(payload: dict[str, Any]) -> None:
    root = load_toml(ROOT_MANIFEST)
    members = set(root.get("workspace", {}).get("members", []))
    require("crates/sifr_sql_postgresql_runtime" in members, "workspace omits PostgreSQL runtime")
    dependencies = root.get("workspace", {}).get("dependencies", {})
    exact = {
        "tokio": "=1.53.1",
        "rustls": "=0.23.43",
        "tokio-rustls": "=0.26.4",
        "postgres-types": "=0.2.14",
        "tokio-postgres": "=0.7.18",
        "tokio-postgres-rustls": "=0.14.0",
    }
    for name, version in exact.items():
        row = dependencies.get(name)
        require(isinstance(row, dict) and row.get("version") == version, f"{name} version drift")
        require(row.get("default-features") is False, f"{name} default features must be disabled")

    manifest = load_toml(RUNTIME_ROOT / "Cargo.toml")
    runtime_dependencies = manifest.get("dependencies", {})
    for name, features in payload["driver_dependencies"].items():
        row = runtime_dependencies.get(name)
        require(isinstance(row, dict) and row.get("workspace") is True, f"runtime omits {name}")
        require(set(row.get("features", [])) == set(features), f"{name} feature allowlist drift")
    require("sqlx" not in runtime_dependencies, "runtime cannot use sqlx")

    required_sources = {
        "codec.rs", "config.rs", "connection.rs", "control.rs", "error.rs", "execute.rs",
        "lib.rs", "pool.rs", "stream.rs", "transaction.rs", "verification.rs",
    }
    sources = {path.name for path in (RUNTIME_ROOT / "src").glob("*.rs")}
    require(sources == required_sources, "runtime source ownership drift")
    source_texts = [
        path.read_text(encoding="utf-8") for path in sorted((RUNTIME_ROOT / "src").glob("*.rs"))
    ]
    text = "\n".join(source_texts)
    for token in (
        "pub struct PostgresPool", "pub struct PostgresConnection", "pub struct PostgresTransaction",
        "pub struct PostgresRowStream", "pub struct PostgresTransactionRowStream",
        "pub trait RetrySafeCallback", "verify_schema", "StatementCache",
    ):
        require(token in text, f"runtime implementation is missing {token}")
    production_text = "\n".join(source.split("#[cfg(test)]", 1)[0] for source in source_texts)
    require(
        ".unwrap()" not in production_text and ".expect(" not in production_text,
        "runtime source contains panic-prone extraction",
    )
    require((REPO_ROOT / payload["live_test"]).is_file(), "live runtime test is missing")
    require((AREA_ROOT / "tools/run_postgresql_runtime_matrix.py").is_file(), "live matrix runner is missing")

    common_manifest = load_toml(COMMON_ROOT / "Cargo.toml")
    common_dependencies = set(common_manifest.get("dependencies", {}))
    require(common_dependencies == {"sifr_runtime", "tokio"}, "common runtime dependency surface drift")


def self_test(payload: dict[str, Any]) -> None:
    mutations: list[dict[str, Any]] = []
    candidate = copy.deepcopy(payload)
    candidate["supported_server_majors"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["runtime_guarantees"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["driver_dependencies"]["tokio-postgres"] = []
    mutations.append(candidate)
    accepted = 0
    for candidate in mutations:
        try:
            validate(candidate)
        except ContractError:
            continue
        accepted += 1
    require(accepted == 0, f"runtime mutations accepted: {accepted}")
    print(f"PostgreSQL runtime self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(CONTRACT.read_text(encoding="utf-8"))
    validate(payload)
    if args.self_test:
        self_test(payload)
    else:
        print("PostgreSQL runtime qualification ok: majors=6 guarantees=14")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, ContractError) as error:
        print(f"PostgreSQL runtime qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
