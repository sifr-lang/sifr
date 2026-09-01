#!/usr/bin/env python3
"""Validate the PostgreSQL migration qualification contract."""

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
CONTRACT = AREA_ROOT / "data/postgresql_migration_qualification.json"
SERVER_MATRIX = AREA_ROOT / "data/postgresql_server_matrix.json"
CAPABILITIES = AREA_ROOT / "data/capability_matrix.json"
TOOLS_ROOT = REPO_ROOT / "crates/sifr_sql_postgresql_tools"
PROVIDER_ROOT = REPO_ROOT / "crates/sifr_sql_postgresql"
RUNTIME_ROOT = REPO_ROOT / "crates/sifr_sql_runtime"
REFLECTED_OBJECTS = {
    "check-constraint", "column", "composite", "domain", "enum", "foreign-key",
    "function", "index", "materialized-view", "namespace", "primary-key", "range",
    "sequence", "table", "unique-constraint", "view",
}
DECLARED_EFFECT_OBJECTS = {
    "array", "cast", "collation", "dialect-metadata", "extension", "generated-column",
    "identity-column", "multirange", "operator", "privilege", "server-capability", "trigger",
}
NONTRANSACTIONAL_OPERATIONS = {
    "alter-subscription", "alter-system", "alter-table-detach-partition-concurrently",
    "alter-type-add-value", "cluster",
    "create-database", "create-index-concurrently", "create-subscription", "create-tablespace",
    "drop-database", "drop-index-concurrently", "drop-subscription", "drop-tablespace",
    "refresh-materialized-view-concurrently", "reindex-concurrently", "vacuum",
}


class ContractError(ValueError):
    """The PostgreSQL migration qualification contract is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def validate(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "schema_version must be 1")
    require(payload.get("provider_family") == "postgresql", "provider family drift")
    require(payload.get("execution_plan_format") == 3, "execution plan format drift")
    require(payload.get("supported_server_majors") == list(range(13, 19)), "server matrix drift")
    reflected = set(payload.get("reflected_object_kinds", []))
    declared = set(payload.get("declared_effect_object_kinds", []))
    require(reflected == REFLECTED_OBJECTS, "reflected object inventory is incomplete")
    require(declared == DECLARED_EFFECT_OBJECTS, "declared-effect object inventory is incomplete")
    require(not reflected.intersection(declared), "object reflection inventories overlap")
    require(
        set(payload.get("fail_closed_mechanisms", []))
        == {
            "advisory-lock", "checksum-drift", "concurrent-start", "head-mismatch",
            "incomplete-merge", "provider-capability", "provider-version", "schema-drift",
        },
        "fail-closed mechanism inventory is incomplete",
    )
    require(
        set(payload.get("operator_actions", []))
        == {
            "assertion", "backfill", "ddl", "recovery-point", "sifr-data", "sql-data",
            "transaction-begin", "transaction-commit",
        },
        "operator action inventory is incomplete",
    )
    require(
        set(payload.get("nontransactional_operations", [])) == NONTRANSACTIONAL_OPERATIONS,
        "DDL boundary inventory drift",
    )
    require(len(payload.get("live_scenarios", [])) == 13, "live scenario inventory drift")
    validate_matrix(payload)
    validate_sources(payload)
    validate_capability()


def validate_matrix(payload: dict[str, Any]) -> None:
    server_rows = json.loads(SERVER_MATRIX.read_text(encoding="utf-8"))["servers"]
    evidence = json.loads((REPO_ROOT / payload["live_matrix"]).read_text(encoding="utf-8"))
    require(evidence.get("schema_version") == 1, "live matrix schema drift")
    evidence_rows = evidence.get("servers", [])
    require([row["major"] for row in evidence_rows] == list(range(13, 19)), "live evidence majors drift")
    require(all(row.get("status") == "passed" for row in evidence_rows), "live evidence is incomplete")
    for server, migration in zip(server_rows, evidence_rows, strict=True):
        require(
            (server["major"], server["image"], server["image_digest"])
            == (migration["major"], migration["image"], migration["image_digest"]),
            "migration evidence differs from the pinned server matrix",
        )


def validate_sources(payload: dict[str, Any]) -> None:
    for key in ("offline_test", "live_test", "live_matrix_runner", "operator_documentation"):
        require((REPO_ROOT / payload[key]).is_file(), f"missing {key}")
    root = load_toml(REPO_ROOT / "Cargo.toml")
    dependencies = root["workspace"]["dependencies"]
    exact_versions = {
        "rustls": "=0.23.43",
        "rustls-platform-verifier": "=0.7.0",
        "semver": "1.0.28",
        "sha2": "0.11.0",
        "tokio-postgres": "=0.7.18",
        "tokio-postgres-rustls": "=0.14.0",
    }
    for name, version in exact_versions.items():
        row = dependencies.get(name)
        require(isinstance(row, dict) and row.get("version") == version, f"{name} version drift")
    tools_manifest = load_toml(TOOLS_ROOT / "Cargo.toml")
    tool_dependencies = tools_manifest.get("dependencies", {})
    for name in (
        "rustls", "rustls-platform-verifier", "sha2", "sifr_sql_contract",
        "sifr_sql_postgresql", "sifr_sql_runtime", "tokio", "tokio-postgres",
        "tokio-postgres-rustls",
    ):
        row = tool_dependencies.get(name)
        require(isinstance(row, dict) and row.get("workspace") is True, f"tool omits {name}")
    source_files = [
        PROVIDER_ROOT / "src/migration.rs",
        TOOLS_ROOT / "src/migration_command.rs",
        TOOLS_ROOT / "src/migration_plan.rs",
        TOOLS_ROOT / "src/migration_runtime.rs",
        RUNTIME_ROOT / "src/migration/engine.rs",
        RUNTIME_ROOT / "src/migration/rollback.rs",
    ]
    source_texts = [path.read_text(encoding="utf-8") for path in source_files]
    text = "\n".join(source_texts)
    for token in (
        "pg_try_advisory_lock", "import_baseline", "rollback_last",
        "MigrationTransactionRequirement", "RecoveryPoint", "path_parent", "prior_heads",
    ):
        require(token in text, f"migration implementation is missing {token}")
    production = "\n".join(source.split("#[cfg(test)]", 1)[0] for source in source_texts)
    require(
        ".unwrap()" not in production and ".expect(" not in production,
        "migration source has panic-prone extraction",
    )


def validate_capability() -> None:
    capabilities = json.loads(CAPABILITIES.read_text(encoding="utf-8"))["capabilities"]
    row = next((item for item in capabilities if item.get("id") == "postgresql.migration"), None)
    require(row is not None, "PostgreSQL migration capability is absent")
    require(
        row.get("status") in {"active", "complete"},
        "PostgreSQL migration capability is neither active nor complete",
    )


def self_test(payload: dict[str, Any]) -> None:
    mutations: list[dict[str, Any]] = []
    candidate = copy.deepcopy(payload)
    candidate["supported_server_majors"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["fail_closed_mechanisms"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["operator_actions"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["execution_plan_format"] = 1
    mutations.append(candidate)
    accepted = 0
    for candidate in mutations:
        try:
            validate(candidate)
        except ContractError:
            continue
        accepted += 1
    require(accepted == 0, f"PostgreSQL migration mutations accepted: {accepted}")
    print(f"PostgreSQL migration self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(CONTRACT.read_text(encoding="utf-8"))
    validate(payload)
    if args.self_test:
        self_test(payload)
    else:
        print("PostgreSQL migration qualification ok: majors=6 scenarios=13")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, ContractError) as error:
        print(f"PostgreSQL migration qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
