#!/usr/bin/env python3
"""Validate the SQL migration compiler and engine qualification record."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
RECORD = REPO_ROOT / "verification/areas/sql_platform/data/migration_engine_qualification.json"

GRAPH = {
    "canonical-target", "checked-dag", "explicit-head", "parent-fingerprints",
    "provider-constraints", "sequencable-branches", "stable-checksums",
}
STEPS = {
    "affine-plan", "bounded-backfill", "checked-assertion",
    "explicit-transaction-boundary", "intermediate-schema",
    "nonescaping-data-callback", "reflected-or-declared-ddl", "state-scoped-data",
}
RUNTIME = {
    "advisory-lock-record", "ambiguous-recovery-rejected", "checksum-drift-rejected",
    "explicit-rollback-only", "head-record", "incomplete-merge-rejected", "panic-contained",
    "provider-identity-rejected", "recovery-point-record", "rollback-prefix-checked",
    "schema-drift-rejected", "step-duration-record",
}


class ContractError(ValueError):
    """The migration qualification record is invalid."""


def validate(payload: Any) -> None:
    if not isinstance(payload, dict) or payload.get("schema_version") != 1:
        raise ContractError("schema_version must be 1")
    if payload.get("format_version") != 2:
        raise ContractError("migration format_version must be 2")
    owners = {
        "compiler_owner": "sifr_sql_contract",
        "hir_owner": "sifr_frontend",
        "runtime_owner": "sifr_sql_runtime",
    }
    for field, expected in owners.items():
        if payload.get(field) != expected:
            raise ContractError(f"{field} must be {expected}")
    for field, expected in (
        ("graph_invariants", GRAPH),
        ("step_invariants", STEPS),
        ("runtime_invariants", RUNTIME),
    ):
        if set(payload.get(field, [])) != expected:
            raise ContractError(f"{field} is incomplete")
    evidence = payload.get("evidence")
    if not isinstance(evidence, dict) or set(evidence) != {
        "compiler", "contract-tests", "documentation", "frontend", "frontend-tests",
        "runtime", "runtime-manifest", "runtime-plan", "runtime-tests",
        "runtime-rollback", "tool-artifacts", "tool-tests",
    }:
        raise ContractError("migration evidence map is incomplete")
    if any(not (REPO_ROOT / str(path)).is_file() for path in evidence.values()):
        raise ContractError("migration evidence path is missing")
    sources = {
        name: (REPO_ROOT / str(path)).read_text(encoding="utf-8")
        for name, path in evidence.items()
    }
    required = {
        "compiler": [
            "pub trait MigrationDialect", "fn compile_path", "semantic_diff",
            "validate_sequencable_branches",
        ],
        "contract-tests": ["opaque DDL", "nullable assertions", "affine_plan"],
        "frontend": ["MigrationPlan", "MigrationDb", "input_state_identity"],
        "runtime": ["acquire_lock", "validate_progress", "AssertionZeroRows"],
        "runtime-manifest": ["sifr_runtime", "tokio"],
        "runtime-plan": ["MigrationExecutionPlan", "MigrationExecutionStepKind"],
        "runtime-rollback": ["rollback_last", "validate_rollback_progress", "prior_heads"],
        "runtime-tests": [
            "pauses_and_resumes", "provider_panics_fail_closed", "branching_plan",
            "schema_changing_merge_requires", "explicit_reverse_plan_rolls_back",
        ],
        "tool-artifacts": [
            "MIGRATION_SCHEMA_PATH", "build_migration_artifacts",
            "lower_migration_execution_plan",
        ],
        "tool-tests": ["deterministic_complete_and_atomic", "target_authority_mismatch"],
        "documentation": ["explicit reverse", "private callback lifetime", "closed execution plan"],
    }
    for owner, tokens in required.items():
        if any(token not in sources[owner] for token in tokens):
            raise ContractError(f"{owner} is missing a required migration mechanism")
    if "sifr_sql_contract" in sources["runtime-manifest"]:
        raise ContractError("application runtime links the compiler-only SQL contract crate")


def self_test(payload: dict[str, Any]) -> None:
    mutations: list[tuple[str, dict[str, Any]]] = []
    for field in ("graph_invariants", "step_invariants", "runtime_invariants"):
        mutated = copy.deepcopy(payload)
        mutated[field].pop()
        mutations.append((field, mutated))
    owner = copy.deepcopy(payload)
    owner["runtime_owner"] = "provider-tool"
    mutations.append(("runtime-owner", owner))
    evidence = copy.deepcopy(payload)
    del evidence["evidence"]["runtime-tests"]
    mutations.append(("runtime-evidence", evidence))
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
    print("migration-engine qualification ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
