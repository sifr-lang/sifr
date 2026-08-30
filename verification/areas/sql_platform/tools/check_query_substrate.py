#!/usr/bin/env python3
"""Validate the typed SQL query and fragment substrate qualification."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
RECORD = REPO_ROOT / "verification/areas/sql_platform/data/query_substrate_qualification.json"
CONTRACT = REPO_ROOT / "crates/sifr_sql_contract/src"
RUNTIME = REPO_ROOT / "crates/sifr_sql_runtime/src"
FRONTEND = REPO_ROOT / "crates/sifr_frontend/src/sql_queries.rs"
DRIVER = REPO_ROOT / "crates/sifr_driver/src/build/sql_profiles.rs"

STATES = {"QueryTemplate", "BoundQuery"}
CATEGORIES = {
    "assignment-list", "command", "expression", "identifier", "join", "order-by",
    "predicate", "query", "relation", "returning-list", "select-list", "values",
}
FRAGMENT_FIELDS = {
    "category", "dialect", "effect-transformation", "free-identifiers",
    "hygienic-aliases", "input-output-scope", "parameter-slots", "precedence",
    "profile", "query-identity", "result-transformation",
}
STATIC_RULES = {
    "no-runtime-branches", "no-runtime-containers", "no-runtime-loops",
    "no-returned-aliases", "source-location-alias-identity", "top-level-row-of-only",
}
LOWERING = {
    "bound-query-consumption", "capture-once-left-to-right", "conditional-clone",
    "normal-sifr-query-types", "runtime-cardinality-round-trip", "runtime-effect-round-trip",
}
EVIDENCE = {
    "alias-escape-negatives", "binding-order-and-failure-timing",
    "cardinality-method-negatives", "fragment-composition-properties",
    "generated-name-collision-properties", "profile-registry-production-consumer",
    "row-of-symbol-negatives", "unsafe-capability-negatives",
}


class QualificationError(ValueError):
    """The query substrate qualification is invalid."""


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
    require(payload.get("compiler_contract") == "sifr_sql_contract", "wrong compiler contract")
    require(payload.get("frontend_consumer") == "sifr_frontend::SqlQueryCompiler", "wrong frontend consumer")
    require(payload.get("runtime_contract") == "sifr_sql_runtime", "wrong runtime contract")
    exact(payload, "public_query_states", STATES)
    exact(payload, "fragment_categories", CATEGORIES)
    exact(payload, "fragment_contract", FRAGMENT_FIELDS)
    exact(payload, "static_identity_rules", STATIC_RULES)
    exact(payload, "predicate_combinators", {"all", "any", "not"})
    exact(payload, "cardinality_adapters", {"expect-at-most-one", "first-with-determinism-lint"})
    exact(payload, "profile_registry_indexes", {"module-path", "nominal-identity", "profile-name"})
    exact(payload, "lowering_contract", LOWERING)
    exact(payload, "evidence", EVIDENCE)
    require(
        payload.get("generated_identifier_codec") == "injective-reversible-reserved-prefix-utf8",
        "generated identifier codec is not injective and reversible",
    )
    unsafe = payload.get("unsafe_syntax")
    require(isinstance(unsafe, dict), "unsafe syntax contract is missing")
    require(unsafe.get("capability") == "sql.unsafe-syntax", "unsafe capability is wrong")
    require(unsafe.get("requires_reason") is True, "unsafe syntax needs an audit reason")
    require(set(unsafe.get("lint_policies", [])) == {"deny", "warn"}, "unsafe lint policies are incomplete")
    require(unsafe.get("runtime_text_is_syntax") is False, "runtime text cannot become SQL syntax")


def validate_sources() -> None:
    contract = "\n".join(path.read_text(encoding="utf-8") for path in sorted(CONTRACT.glob("*.rs")))
    runtime = "\n".join(path.read_text(encoding="utf-8") for path in sorted(RUNTIME.glob("*.rs")))
    frontend = FRONTEND.read_text(encoding="utf-8")
    driver = DRIVER.read_text(encoding="utf-8")
    for token in (
        "pub struct QueryTemplateContract", "pub struct QuerySignatureRegistry",
        "pub enum FragmentCategory", "pub struct SqlFragment",
        "pub struct ProfileModuleRegistry", "pub fn encode_generated_identifier",
        "pub fn decode_generated_identifier",
    ):
        require(token in contract, f"compiler contract is missing {token}")
    for token in (
        "pub struct QueryTemplate", "pub struct BoundQuery", "pub trait EncodeParameters",
        "pub struct OrderedParameterEncoder", "into_execution_request",
    ):
        require(token in runtime, f"runtime contract is missing {token}")
    for token in (
        "pub struct SqlQueryCompiler", "pub fn compile", "pub fn bind", "pub fn execution",
        "runtime_cardinality: query.cardinality.clone()", "runtime_effects: query.effects.clone()",
    ):
        require(token in frontend, f"frontend query consumer is missing {token}")
    require("SqlQueryCompiler::new(&registry)" in driver, "driver does not invoke the production registry consumer")
    require("ProfileModuleRegistry" in driver, "driver still lacks a queryable profile registry")


def self_test(payload: dict[str, Any]) -> None:
    mutations: dict[str, dict[str, Any]] = {}
    for field in (
        "public_query_states", "fragment_categories", "fragment_contract",
        "static_identity_rules", "predicate_combinators", "cardinality_adapters",
        "profile_registry_indexes", "lowering_contract", "evidence",
    ):
        candidate = copy.deepcopy(payload)
        candidate[field].pop()
        mutations[field] = candidate
    candidate = copy.deepcopy(payload)
    candidate["unsafe_syntax"]["runtime_text_is_syntax"] = True
    mutations["runtime-text"] = candidate
    candidate = copy.deepcopy(payload)
    candidate["generated_identifier_codec"] = "suffix-on-collision"
    mutations["identifier-codec"] = candidate

    accepted: list[str] = []
    for label, candidate in mutations.items():
        try:
            validate_payload(candidate)
        except QualificationError:
            continue
        accepted.append(label)
    require(not accepted, f"query substrate mutations were accepted: {', '.join(accepted)}")
    print(f"query substrate qualification self-test ok: mutations={len(mutations)}")


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
            "query substrate qualification ok: "
            f"states={len(STATES)} categories={len(CATEGORIES)} rules={len(STATIC_RULES)}"
        )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, QualificationError, json.JSONDecodeError) as error:
        print(f"query substrate qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
