#!/usr/bin/env python3
"""Validate schema polymorphism and portable SQL qualification."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
RECORD = REPO_ROOT / "verification/areas/sql_platform/data/schema_polymorphism_qualification.json"
CONTRACT = REPO_ROOT / "crates/sifr_sql_contract/src/requirement.rs"
PROVIDER = REPO_ROOT / "crates/sifr_sql_contract/src/provider.rs"
FRONTEND = REPO_ROOT / "crates/sifr_frontend/src/sql_schema_polymorphism.rs"
QUERY = REPO_ROOT / "crates/sifr_frontend/src/sql_queries.rs"
MANIFEST = REPO_ROOT / "crates/sifr_package/src/manifest/sql_profiles.rs"
RESOLVER = REPO_ROOT / "crates/sifr_package/src/sql_requirements.rs"
DRIVER = REPO_ROOT / "crates/sifr_driver/src/build/sql_profiles.rs"
POSTGRESQL = REPO_ROOT / "crates/sifr_sql_postgresql/src/component.rs"
DOC = REPO_ROOT / "internal_docs/sql_schema_polymorphism.md"
FIXTURE = REPO_ROOT / "verification/areas/sql_platform/fixtures/schema_polymorphism"

MANIFEST_CONTRACT = {"capabilities", "provider", "providers", "server-version", "source"}
PROOF_CONTRACT = {
    "capability-subset", "minimum-server-version", "provider-identity", "schema-object-subset",
}
ALLOWED = {"constrained-generic-parameter", "direct-profile-namespace-export"}
PROHIBITED = {
    "capture", "return", "runtime-storage", "selection", "unconstrained-generic-parameter",
}
RESOURCES = {"verified-connection", "verified-pool", "verified-transaction"}
RULES = {
    "concrete-profile-analysis", "no-runtime-provider-dispatch",
    "no-silent-capability-rewrite", "undeclared-behavior-rejection",
    "provider-owned-capability-account", "provider-owned-object-account",
    "undeclared-object-rejection", "witness-erasure",
}
POSTGRESQL_CAPABILITIES = {
    "sql.bind.parameters", "sql.expression.case", "sql.expression.equality",
    "sql.query.aggregate", "sql.query.common-table-expression", "sql.query.delete",
    "sql.query.insert", "sql.query.join", "sql.query.row-locking", "sql.query.select",
    "sql.query.set-operation", "sql.query.subquery", "sql.query.update", "sql.query.window",
    "sql.write.conflict", "sql.write.returning",
}
EVIDENCE = {
    "capability-negative", "execution-profile-negative", "fingerprint-property",
    "manifest-resolution", "postgresql-ddl-normalization", "provider-mismatch-negative",
    "schema-subset-property", "undeclared-behavior-negative", "undeclared-object-negative",
    "witness-use-negative",
}


class QualificationError(ValueError):
    """The schema polymorphism qualification is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise QualificationError(message)


def exact(payload: dict[str, Any], field: str, expected: set[str]) -> None:
    values = payload.get(field)
    require(isinstance(values, list), f"{field} must be a list")
    require(len(values) == len(set(values)), f"{field} contains duplicates")
    require(set(values) == expected, f"{field} is incomplete")


def validate_payload(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "schema_version must be 1")
    require(
        payload.get("requirement_language") == "provider-normalized-ddl-to-schema-ir-subset",
        "requirements must use provider-normalized DDL and SchemaIR",
    )
    exact(payload, "manifest_contract", MANIFEST_CONTRACT)
    exact(payload, "proof_contract", PROOF_CONTRACT)
    exact(payload, "witness_allowed_uses", ALLOWED)
    exact(payload, "witness_prohibited_uses", PROHIBITED)
    exact(payload, "execution_resources", RESOURCES)
    exact(payload, "specialization_rules", RULES)
    exact(payload, "postgresql_capabilities", POSTGRESQL_CAPABILITIES)
    exact(payload, "provider_evidence", {"postgresql", "provider-neutral-harness"})
    exact(payload, "evidence", EVIDENCE)


def contains(path: Path, tokens: tuple[str, ...]) -> None:
    source = path.read_text(encoding="utf-8")
    for token in tokens:
        require(token in source, f"{path.relative_to(REPO_ROOT)} is missing {token}")


def validate_sources() -> None:
    contains(CONTRACT, (
        "pub struct ProviderSchemaRequirement", "pub struct SchemaRequirementProof",
        "pub struct SchemaRequirementRegistry", "verify_compatible_slice",
        "MissingCapability", "UndeclaredObject", "UndeclaredBehavior",
    ))
    contains(PROVIDER, (
        "pub accessed_objects: BTreeSet<ObjectId>",
        "pub required_capabilities: BTreeSet<String>",
        "callers cannot supply or narrow it",
    ))
    contains(FRONTEND, (
        "pub struct SqlSchemaWitness", "pub enum SqlSchemaWitnessUse",
        "pub struct SchemaPolymorphicQueryCompiler", "pub fn specialize",
        "validate_query_envelope", "SpecializedSqlQuery { query, proof }",
        "analysis.required_capabilities",
        "analysis.accessed_objects",
    ))
    contains(QUERY, (
        "pub struct VerifiedSqlExecutionResource", "pub enum SqlExecutionResourceKind",
        "query.profile_identity != resource.profile_identity",
    ))
    contains(MANIFEST, (
        "pub struct SqlRequirementConfig", "pub struct SqlRequirementProviderConfig",
        '"capabilities", "providers"',
    ))
    contains(RESOLVER, (
        "pub fn resolve_sql_requirements", "reachable_packages",
        "build_provider_schema_requirement",
    ))
    contains(DRIVER, (
        "SchemaRequirementRegistry", "resolve_sql_requirements",
        "schema_normalization_from_response", ".build_artifact(",
    ))
    contains(POSTGRESQL, (
        "pub fn postgresql_capabilities", '"sql.query.select"',
        "capabilities: postgresql_capabilities()",
    ))
    contains(DOC, (
        "No second schema language", "Witness erasure", "No runtime provider dispatch",
    ))
    contains(FIXTURE / "has_users.postgresql.sql", (
        "CREATE TABLE public.users", "PRIMARY KEY", "NOT NULL UNIQUE",
    ))
    contains(FIXTURE / "portable_by_email.sifr", (
        "has_users.Schema", "SqlSchema[S]", "by_email(app.schema",
    ))


def self_test(payload: dict[str, Any]) -> None:
    mutations: dict[str, dict[str, Any]] = {}
    for field in (
        "manifest_contract", "proof_contract", "witness_allowed_uses",
        "witness_prohibited_uses", "execution_resources", "specialization_rules",
        "postgresql_capabilities", "provider_evidence", "evidence",
    ):
        candidate = copy.deepcopy(payload)
        candidate[field].pop()
        mutations[field] = candidate
    candidate = copy.deepcopy(payload)
    candidate["requirement_language"] = "custom-schema-dsl"
    mutations["second-schema-language"] = candidate
    accepted: list[str] = []
    for label, candidate in mutations.items():
        try:
            validate_payload(candidate)
        except QualificationError:
            continue
        accepted.append(label)
    require(not accepted, f"schema polymorphism mutations were accepted: {', '.join(accepted)}")
    print(f"schema polymorphism qualification self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(RECORD.read_text(encoding="utf-8"))
    validate_payload(payload)
    validate_sources()
    if args.self_test:
        self_test(payload)
    else:
        print(
            "schema polymorphism qualification ok: "
            f"capabilities={len(POSTGRESQL_CAPABILITIES)} evidence={len(EVIDENCE)}"
        )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, QualificationError, json.JSONDecodeError) as error:
        print(f"schema polymorphism qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
