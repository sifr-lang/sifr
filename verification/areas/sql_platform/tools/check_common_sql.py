#!/usr/bin/env python3
"""Validate common SQL contract and runtime qualification."""

from __future__ import annotations

import argparse
import copy
import json
import sys
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
DATA_PATH = REPO_ROOT / "verification/areas/sql_platform/data/common_sql_qualification.json"
ANALYSIS_ROOT = REPO_ROOT / "crates/sifr_sql_contract"
RUNTIME_ROOT = REPO_ROOT / "crates/sifr_sql_runtime"

TYPE_FAMILIES = {
    "array", "binary", "boolean", "calendar-interval", "composite", "custom", "date",
    "decimal", "domain", "enum", "float32", "float64", "integer", "ip-address",
    "ip-network", "json", "local-date-time", "local-time", "mac-address", "offset-time",
    "range-and-multirange", "sqlite-dynamic", "text", "timestamp-instant", "uuid",
}
BIND_RESULTS = {
    "exact", "fallible-array-shape", "fallible-binary-length",
    "fallible-decimal-precision-and-scale",
    "fallible-exact-integer-range", "fallible-float32-range-and-precision",
    "fallible-text-length", "reject-array-element", "reject-integer-sign",
    "reject-integer-width", "reject-missing-codec", "reject-nominal-identity",
    "reject-nullability", "reject-unsupported-pair",
}
RUNTIME_ERRORS = {
    "authentication", "cancelled", "cardinality", "configuration", "connection",
    "constraint", "deadlock", "decode", "encode", "migration", "provider",
    "resource-limit", "schema-contract", "serialization", "timeout",
}
EFFECTS = {
    "read", "write", "read-write", "schema-change", "session-change",
    "transaction-control",
}
DIAGNOSTICS = {f"SIFR-SQL-{index:04d}" for index in range(1, 9)}
FORBIDDEN_DRIVERS = {
    "libsqlite3-sys", "mysql_async", "mysql_common", "postgres-types", "rusqlite",
    "sqlx", "tokio-postgres",
}


class QualificationError(ValueError):
    """The common SQL qualification is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise QualificationError(message)


def exact_set(payload: dict[str, Any], field: str, expected: set[str]) -> None:
    values = payload.get(field)
    require(isinstance(values, list), f"{field} must be a list")
    require(len(values) == len(set(values)), f"{field} contains duplicates")
    require(set(values) == expected, f"{field} is incomplete")


def validate_payload(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "schema_version must be 1")
    require(payload.get("analysis_artifact") == "sifr_sql_contract", "wrong analysis artifact")
    require(payload.get("runtime_artifact") == "sifr_sql_runtime", "wrong runtime artifact")
    exact_set(payload, "database_type_families", TYPE_FAMILIES)
    exact_set(payload, "bind_results", BIND_RESULTS)
    exact_set(payload, "compiler_diagnostics", DIAGNOSTICS)
    exact_set(payload, "runtime_errors", RUNTIME_ERRORS)
    exact_set(payload, "effects", EFFECTS)

    cardinality = payload.get("cardinality")
    require(isinstance(cardinality, dict), "cardinality contract is missing")
    require(cardinality.get("bottom") == "empty", "cardinality lattice has no bottom")
    require(
        set(cardinality.get("named_intervals", []))
        == {"zero", "at-most-one", "exactly-one", "one-or-more", "many"},
        "named cardinality intervals are incomplete",
    )
    require(set(cardinality.get("operations", [])) == {"join", "meet"}, "lattice operations are incomplete")
    require(
        cardinality.get("container_selection") == "explicit-fetch-method-only",
        "cardinality must not select containers",
    )

    ownership = payload.get("ownership")
    require(isinstance(ownership, dict), "ownership contract is missing")
    require(set(ownership) == {"pool", "connection", "transaction", "row-stream", "bound-parameters"}, "ownership handles are incomplete")
    require("share-safe" in ownership["pool"], "verified pool must be share-safe")
    for handle in ("connection", "transaction", "row-stream"):
        require("non-send" in ownership[handle], f"{handle} must be non-send")
    require("owned" in ownership["bound-parameters"], "bound parameters must own values")

    boundary = payload.get("provider_boundary")
    require(isinstance(boundary, dict), "provider boundary is missing")
    require(set(boundary.get("runtime_forbidden_dependencies", [])) == FORBIDDEN_DRIVERS, "forbidden driver list is incomplete")
    require("dialect-semantics" in boundary.get("provider_owns", []), "provider must own dialect semantics")
    require("panic-containment" in boundary.get("common_owns", []), "common runtime must own panic containment")

    evidence = set(payload.get("evidence", []))
    required_evidence = {
        "bind-matrix", "cardinality-lattice-properties", "codec-round-trip-and-malformed-values",
        "ownership-compile-fail-doctests", "panic-boundary-tests", "redaction-tests",
        "type-mapping-matrix",
    }
    require(evidence == required_evidence, "common SQL evidence list is incomplete")


def validate_sources() -> None:
    analysis_sources = "\n".join(
        path.read_text(encoding="utf-8") for path in sorted((ANALYSIS_ROOT / "src").glob("*.rs"))
    )
    runtime_sources = "\n".join(
        path.read_text(encoding="utf-8") for path in sorted((RUNTIME_ROOT / "src").glob("*.rs"))
    )
    for token in ("pub enum DatabaseType", "pub fn bind_compatibility", "pub enum Cardinality", "pub struct EffectContract", "pub trait DialectSemantics"):
        require(token in analysis_sources, f"analysis implementation is missing {token}")
    for token in ("pub struct Pool", "pub struct Connection", "pub struct Transaction", "pub struct RowStream", "pub trait ProviderRuntime", "catch_unwind"):
        require(token in runtime_sources, f"runtime implementation is missing {token}")

    with (RUNTIME_ROOT / "Cargo.toml").open("rb") as handle:
        manifest = tomllib.load(handle)
    dependencies = set(manifest.get("dependencies", {}))
    require(not dependencies.intersection(FORBIDDEN_DRIVERS), "common runtime links a database driver")
    require(
        dependencies == {"sifr_runtime", "tokio"},
        "common runtime dependency surface is not minimal",
    )


def self_test(payload: dict[str, Any]) -> None:
    mutations: dict[str, dict[str, Any]] = {}
    for label, field in (
        ("type-family", "database_type_families"),
        ("bind-result", "bind_results"),
        ("diagnostic", "compiler_diagnostics"),
        ("runtime-error", "runtime_errors"),
        ("effect", "effects"),
        ("evidence", "evidence"),
    ):
        candidate = copy.deepcopy(payload)
        candidate[field].pop()
        mutations[label] = candidate
    candidate = copy.deepcopy(payload)
    candidate["cardinality"]["container_selection"] = "inferred-container"
    mutations["cardinality-container"] = candidate
    candidate = copy.deepcopy(payload)
    candidate["ownership"]["connection"].remove("non-send")
    mutations["connection-send"] = candidate

    accepted: list[str] = []
    for label, candidate in mutations.items():
        try:
            validate_payload(candidate)
        except QualificationError:
            continue
        accepted.append(label)
    require(not accepted, f"qualification mutations were accepted: {', '.join(accepted)}")
    print(f"common SQL qualification self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(DATA_PATH.read_text(encoding="utf-8"))
    validate_payload(payload)
    validate_sources()
    if args.self_test:
        self_test(payload)
    else:
        print(
            "common SQL qualification ok: "
            f"types={len(TYPE_FAMILIES)} bind-results={len(BIND_RESULTS)} "
            f"diagnostics={len(DIAGNOSTICS)} runtime-errors={len(RUNTIME_ERRORS)}"
        )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, QualificationError, json.JSONDecodeError) as error:
        print(f"common SQL qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
