#!/usr/bin/env python3
"""Validate the canonical SQL schema-profile qualification record."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
RECORD = REPO_ROOT / "verification/areas/sql_platform/data/schema_profile_qualification.json"

SOURCE_KINDS = {"sql-ddl", "provider-metadata", "generated-definitions"}
GENERATED = {
    "component-schema-context", "profile-module", "profile-module-metadata",
    "runtime-schema-manifest",
}
EXPORTS = {
    "Schema", "schema", "sql", "query", "connect", "open_pool", "symbol",
    "all", "any", "not_", "enums", "domains", "composites",
}
OBJECT_KINDS = {
    "catalog", "namespace", "table", "column", "primary-key", "unique-constraint",
    "foreign-key", "check-constraint", "index", "sequence", "identity-column",
    "view", "materialized-view", "enum", "domain", "composite", "array", "range",
    "function", "operator", "cast", "collation", "character-set", "extension",
    "trigger", "server-capability", "dialect-metadata",
}


class ContractError(ValueError):
    """The schema-profile qualification record is invalid."""


def validate(payload: Any) -> None:
    if not isinstance(payload, dict) or payload.get("schema_version") != 1:
        raise ContractError("schema_version must be 1")
    if payload.get("schema_ir_owner") != "sifr_sql_contract":
        raise ContractError("SchemaIR owner must be sifr_sql_contract")
    if payload.get("configuration_owner") != "sifr_package":
        raise ContractError("profile configuration owner must be sifr_package")
    if set(payload.get("source_kinds", [])) != SOURCE_KINDS:
        raise ContractError("schema source-kind set is incomplete")
    if set(payload.get("object_kinds", [])) != OBJECT_KINDS:
        raise ContractError("SchemaIR object-kind set is incomplete")
    if set(payload.get("generated_artifacts", [])) != GENERATED:
        raise ContractError("generated schema artifact set is incomplete")
    if set(payload.get("compiler_known_exports", [])) != EXPORTS:
        raise ContractError("compiler-known profile export set is incomplete")
    if payload.get("normal_compilation_credentials") is not False:
        raise ContractError("normal compilation must not accept credentials")
    cargo = (REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8")
    if '"crates/sifr_sql_contract"' not in cargo or "sifr_sql_contract =" not in cargo:
        raise ContractError("workspace does not own sifr_sql_contract")
    sources = {
        "schema": (REPO_ROOT / "crates/sifr_sql_contract/src/schema.rs").read_text(encoding="utf-8"),
        "normalization": (REPO_ROOT / "crates/sifr_sql_contract/src/normalization.rs").read_text(encoding="utf-8"),
        "profile": (REPO_ROOT / "crates/sifr_sql_contract/src/profile.rs").read_text(encoding="utf-8"),
        "generated": (REPO_ROOT / "crates/sifr_sql_contract/src/generated.rs").read_text(encoding="utf-8"),
        "package": (REPO_ROOT / "crates/sifr_package/src/manifest/sql_profiles.rs").read_text(encoding="utf-8"),
        "component": (REPO_ROOT / "crates/sifr_compiler_component/src/protocol.rs").read_text(encoding="utf-8"),
    }
    required_tokens = {
        "schema": ["pub struct SchemaIr", "pub enum SchemaObjectKind", "pub enum SemanticValue"],
        "normalization": ["pub fn normalize_schema", "DuplicateObject", "MissingDependency"],
        "profile": ["pub struct RuntimeSchemaManifest", "profile_identity", "schema_fingerprint"],
        "generated": ["COMPILER_KNOWN_PROFILE_EXPORTS", "lookup_static_symbol", "GeneratedProfileModule"],
        "package": ["parse_sql_config", "connection URLs, credentials", "signed-manifest"],
        "component": ["pub struct ContextArtifact", "pub artifacts: Vec<ContextArtifact>"],
    }
    for owner, tokens in required_tokens.items():
        if any(token not in sources[owner] for token in tokens):
            raise ContractError(f"{owner} implementation is missing a required mechanism")
    evidence = payload.get("evidence")
    if not isinstance(evidence, dict) or any(
        not (REPO_ROOT / str(path)).is_file() for path in evidence.values()
    ):
        raise ContractError("schema-profile evidence path is missing")


def self_test(payload: dict[str, Any]) -> None:
    mutations: list[tuple[str, dict[str, Any]]] = []
    missing_kind = copy.deepcopy(payload)
    missing_kind["object_kinds"].pop()
    mutations.append(("object-kind", missing_kind))
    credentials = copy.deepcopy(payload)
    credentials["normal_compilation_credentials"] = True
    mutations.append(("credentials", credentials))
    missing_export = copy.deepcopy(payload)
    missing_export["compiler_known_exports"].remove("symbol")
    mutations.append(("static-symbol", missing_export))
    for name, mutated in mutations:
        try:
            validate(mutated)
        except ContractError:
            continue
        raise ContractError(f"self-test mutation '{name}' was not detected")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(RECORD.read_text(encoding="utf-8"))
    validate(payload)
    if args.self_test:
        self_test(payload)
    print("schema-profile qualification ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
