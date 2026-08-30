#!/usr/bin/env python3
"""Validate the incremental SQL analysis and embedded-editor contract."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification/areas/sql_platform"
CONTRACT = AREA_ROOT / "data/incremental_editor_qualification.json"


class ContractError(ValueError):
    """The incremental SQL editor qualification is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def validate(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "schema_version must be 1")
    require(payload.get("cache_owner") == "sifr_frontend::cache_keys", "cache owner drift")
    require(payload.get("invalidation_owner") == "sifr_analysis", "invalidation owner drift")
    require(payload.get("presentation_owner") == "sifr_lsp", "presentation owner drift")
    require(
        set(payload.get("production_wiring", []))
        == {
            "resolved-profile-registry", "provider-component-analysis",
            "dependency-scoped-cache", "live-lsp-cancellation", "static-sql-routing",
        },
        "production SQL editor wiring is incomplete",
    )
    require(
        set(payload.get("cache_identity", []))
        == {
            "template-segments", "hole-types", "fragment-identities", "schema-profile-and-slice",
            "provider-identity-and-version", "compatibility-settings", "component-protocol",
            "compiler-semantics",
        },
        "cache identity inventory is incomplete",
    )
    require(
        set(payload.get("editor_features", []))
        == {
            "highlighting", "completion", "hover", "definition", "references", "rename",
            "parameter-information", "result-information", "nullability", "cardinality",
            "formatting", "quick-fixes", "fragment-scope",
        },
        "editor feature inventory is incomplete",
    )
    require(
        set(payload.get("quick_fixes", []))
        == {"aliases", "casts", "missing-columns", "unsafe-collection", "migration-impact"},
        "quick-fix inventory is incomplete",
    )
    require(
        payload.get("performance_budgets_ms")
        == {
            "perf.lsp.sql.completion": 200,
            "perf.lsp.sql.hover": 100,
            "perf.lsp.sql.navigation": 500,
            "perf.lsp.sql.diagnostics": 250,
            "perf.lsp.sql.format": 500,
        },
        "SQL editor performance budget drift",
    )
    require(
        payload.get("cancellation_checkpoints")
        == ["before-component-entry", "between-provider-operations", "before-result-publication"],
        "SQL editor cancellation checkpoints drift",
    )
    require(payload.get("cache_capacity_entries") == 4096, "cache capacity drift")
    validate_sources()


def validate_sources() -> None:
    frontend = (REPO_ROOT / "crates/sifr_frontend/src/sql_editor.rs").read_text(encoding="utf-8")
    cache = (REPO_ROOT / "crates/sifr_frontend/src/embedded_query_cache.rs").read_text(encoding="utf-8")
    control = (REPO_ROOT / "crates/sifr_frontend/src/embedded_analysis_control.rs").read_text(encoding="utf-8")
    key = (REPO_ROOT / "crates/sifr_frontend/src/embedded_analysis_cache_key.rs").read_text(encoding="utf-8")
    analysis = (REPO_ROOT / "crates/sifr_analysis/src/host/sql_editor.rs").read_text(encoding="utf-8")
    runtime = (REPO_ROOT / "crates/sifr_analysis/src/sql_editor_runtime.rs").read_text(encoding="utf-8")
    host = (REPO_ROOT / "crates/sifr_analysis/src/host/construction.rs").read_text(encoding="utf-8")
    invalidation = (REPO_ROOT / "crates/sifr_analysis/src/sql_incremental_cache.rs").read_text(encoding="utf-8")
    profiles = (REPO_ROOT / "crates/sifr_driver/src/build/sql_profiles.rs").read_text(encoding="utf-8")
    component = (REPO_ROOT / "crates/sifr_sql_contract/src/component.rs").read_text(encoding="utf-8")
    budgets = (REPO_ROOT / "crates/sifr_lsp/src/sql_editor_contract.rs").read_text(encoding="utf-8")
    cancellation = (REPO_ROOT / "crates/sifr_lsp/src/cancellation.rs").read_text(encoding="utf-8")
    session = (REPO_ROOT / "crates/sifr_lsp/src/session.rs").read_text(encoding="utf-8")
    server = (REPO_ROOT / "crates/sifr_lsp/src/server.rs").read_text(encoding="utf-8")
    lsp = (REPO_ROOT / "crates/sifr_lsp/src/session/tests/sql_editor_tests.rs").read_text(encoding="utf-8")
    templates = (REPO_ROOT / "crates/sifr_frontend/src/template_documents.rs").read_text(encoding="utf-8")
    for token in (
        "SqlEditorDocumentView", "SqlEditorCatalog::from_schema", "fragment_relations",
        "database_name_for_generated", "semantic_source_tokens", "parameter_source_ranges",
        "SqlEditorFixKind", "fixes_for_diagnostic", "relation_aliases",
    ):
        require(token in frontend, f"SQL editor source is missing {token}")
    for token in ("EmbeddedQueryCache", "pin", "evict", "evicted"):
        require(token in cache, f"incremental cache source is missing {token}")
    for token in ("SqlDependencyIndex", "invalidate_dependencies", "affected_by_observed"):
        require(token in invalidation, f"analysis dependency invalidation is missing {token}")
    for token in (
        "BeforeComponentEntry", "BetweenProviderOperations", "BeforeResultPublication",
        "run_embedded_provider_operations",
    ):
        require(token in control, f"embedded cancellation control is missing {token}")
    require("complete component request" in key, "cache-key authority is not documented")
    for token in (
        "EmbeddedAnalysisKey::new", "run_embedded_provider_items",
        "provider_analysis_from_response", "invalidate_dependencies",
        "schema_object_fingerprint", "with_provider_analysis",
        "schema-slice", "slice_keys",
    ):
        require(token in runtime, f"production SQL editor runtime is missing {token}")
    require("load_sql_editor_profiles" in host,
            "analysis host does not load resolved SQL profiles")
    require("processor_kind" in profiles and "Some(\"sql\")" in profiles
            and "query_component" in profiles,
            "driver does not resolve the SQL provider processor")
    require("PROVIDER_ANALYSIS_PAYLOAD_TAG" in component,
            "provider analysis has no common response envelope")
    for token in (
        "sql_completion", "sql_hover", "sql_locations", "sql_rename", "sql_semantic_tokens",
        "sql_inlay_hints", "sql_highlights", "sql_code_actions",
    ):
        require(token in analysis, f"analysis SQL editor routing is missing {token}")
    require("assert_snapshot" in lsp, "language-server SQL snapshot is missing")
    require("contains_static_source_offset" in analysis
            and "contains_interpolation_source_offset" in analysis,
            "SQL and Sifr cursor routing are not separated")
    require("set_sql_cancellation_flag" in session,
            "LSP requests do not thread cancellation into SQL analysis")
    require("CancellationRegistry" in cancellation and "cancellation.cancel" in server,
            "the live LSP message path cannot signal provider cancellation")
    for budget_id in (
        "perf.lsp.sql.completion", "perf.lsp.sql.hover", "perf.lsp.sql.navigation",
        "perf.lsp.sql.diagnostics", "perf.lsp.sql.format",
    ):
        require(budget_id in budgets, f"language-server budget is missing {budget_id}")
    require("virtual_offset_for_source" in templates and "source_offset_for_virtual" in templates,
            "bidirectional virtual offset mapping is incomplete")


def self_test(payload: dict[str, Any]) -> None:
    mutations: list[dict[str, Any]] = []
    candidate = copy.deepcopy(payload)
    candidate["cache_identity"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["editor_features"].pop()
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["performance_budgets_ms"]["perf.lsp.sql.hover"] = 101
    mutations.append(candidate)
    candidate = copy.deepcopy(payload)
    candidate["production_wiring"].pop()
    mutations.append(candidate)
    accepted = 0
    for candidate in mutations:
        try:
            validate(candidate)
        except ContractError:
            continue
        accepted += 1
    require(accepted == 0, f"incremental editor mutations accepted: {accepted}")
    print(f"SQL incremental editor self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(CONTRACT.read_text(encoding="utf-8"))
    validate(payload)
    if args.self_test:
        self_test(payload)
    else:
        print("SQL incremental editor qualification ok: features=13 budgets=5")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, ContractError) as error:
        print(f"SQL incremental editor qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
