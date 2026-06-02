#!/usr/bin/env python3
"""M1 pre-flight guardrails for the TypeScript-Go architecture transfer."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DOC = REPO_ROOT / "internal_docs" / "typescript_go_architecture_transfer_m1_guardrails.md"
SOURCE_MAPS = REPO_ROOT / "crates" / "sifr_frontend" / "src" / "source_maps.rs"
DOCUMENT_STORE = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "document_store.rs"
LSP_ANALYSIS_WORKSPACE = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "analysis_workspace.rs"
LSP_SESSION = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "session.rs"
LSP_REQUESTS = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "requests" / "mod.rs"
SCHEDULER = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "scheduler.rs"
REQUEST_QUEUE = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "request_queue.rs"
CANCELLATION = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "cancellation.rs"
PROGRESS = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "progress.rs"
WATCHDOG = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "watchdog.rs"
ANALYSIS_SYMBOLS = REPO_ROOT / "crates" / "sifr_analysis" / "src" / "symbols.rs"
ANALYSIS_WORKER_LANES = REPO_ROOT / "crates" / "sifr_analysis" / "src" / "worker_lanes.rs"
WORKSPACE_SESSION = REPO_ROOT / "crates" / "sifr_frontend" / "src" / "workspace_session.rs"
WORKSPACE_RESIDENCY = REPO_ROOT / "crates" / "sifr_frontend" / "src" / "workspace_residency.rs"
WORKSPACE_TRACE = REPO_ROOT / "crates" / "sifr_frontend" / "src" / "workspace_trace.rs"
ANALYSIS_DEBUG_STATUS = REPO_ROOT / "crates" / "sifr_analysis" / "src" / "host" / "debug_status.rs"
TRACE_CLI = REPO_ROOT / "crates" / "sifr" / "src" / "trace_cli.rs"
PERF_MANIFEST = REPO_ROOT / "verification" / "performance" / "manifest.json"
SOURCE_DEP_GUARD = REPO_ROOT / "scripts" / "check_source_crate_dependency_direction.py"
DIRECT_FS_PATTERN = re.compile(
    r"(?:std::fs::|fs::)(?:read_to_string|read_dir)|\.is_file\(\)|\.is_dir\(\)"
)
DIRECT_FS_SCAN_ROOTS = [
    REPO_ROOT / "crates" / "sifr" / "src",
    REPO_ROOT / "crates" / "sifr_driver" / "src",
    REPO_ROOT / "crates" / "sifr_frontend" / "src",
    REPO_ROOT / "crates" / "sifr_format" / "src",
    REPO_ROOT / "crates" / "sifr_lint" / "src",
    REPO_ROOT / "crates" / "sifr_package" / "src",
]
SOURCE_PROVIDER_BOUNDARY = REPO_ROOT / "crates" / "sifr_frontend" / "src" / "source_provider.rs"

REQUIRED_DOC_SNIPPETS = [
    "SourceProvider",
    "WorkspaceSession",
    "WorkspaceSnapshot",
    "DirtyScope",
    "DirtyReason",
    "ModuleSignature",
    "CompilerFingerprint",
    "CacheKeyFingerprint",
    "FlowGraph",
    ".sifrbuildinfo",
    "crates/sifr_frontend/src/graph_cache_and_queries.rs:312",
    "crates/sifr_frontend/src/graph_cache_and_queries.rs:322",
    "crates/sifr_frontend/src/graph_cache_and_queries.rs:359",
    "crates/sifr_driver/src/project/discovery.rs:90",
    "crates/sifr_driver/src/project/discovery.rs:118",
    "crates/sifr_driver/src/project/discovery.rs:180",
    "crates/sifr_driver/src/project/discovery.rs:332",
    "crates/sifr_driver/src/project/discovery.rs:574",
    "crates/sifr_driver/src/workspace/mod.rs:32",
    "crates/sifr_driver/src/workspace/mod.rs:49",
    "crates/sifr_driver/src/workspace/mod.rs:156",
    "crates/sifr_driver/src/project/package_discovery.rs:53",
    "crates/sifr_driver/src/build/workspace.rs:219",
    "crates/sifr_driver/src/build/workspace.rs:282",
    "crates/sifr_driver/src/build/workspace.rs:296",
    "crates/sifr_lint/src/engine.rs:134",
    "crates/sifr_lint/src/config.rs:48",
    "crates/sifr_lint/src/config.rs:72",
    "crates/sifr_lint/src/discovery.rs:29",
    "crates/sifr_lint/src/discovery.rs:33",
    "crates/sifr_lint/src/discovery.rs:79",
    "crates/sifr_format/src/lib.rs:177",
    "crates/sifr_format/src/lib.rs:180",
    "crates/sifr_format/src/lib.rs:197",
    "crates/sifr_format/src/lib.rs:215",
    "crates/sifr_format/src/lib.rs:446",
    "crates/sifr_format/src/lib.rs:456",
    "crates/sifr_format/src/config.rs:85",
    "crates/sifr_format/src/config.rs:109",
    "crates/sifr_package/src/manifest/sifr.rs:55",
    "crates/sifr_package/src/manifest/validate.rs:14",
    "crates/sifr_package/src/manifest/validate.rs:43",
    "crates/sifr_package/src/manifest/validate.rs:44",
    "crates/sifr_package/src/imports/source_map.rs:240",
    "crates/sifr_package/src/imports/source_map.rs:254",
    "crates/sifr_package/src/imports/namespace_api.rs:32",
    "crates/sifr_package/src/imports/namespace_api.rs:264",
    "crates/sifr_package/src/source/layout.rs:30",
    "crates/sifr_package/src/ops/session_discovery.rs:6",
    "crates/sifr_package/src/ops/session_discovery.rs:13",
    "crates/sifr_package/src/ops/session_discovery.rs:25",
    "crates/sifr_package/src/ops/session_targets.rs:17",
    "crates/sifr_package/src/ops/session_targets.rs:34",
    "crates/sifr_package/src/ops/session_targets.rs:42",
    "crates/sifr_package/src/projection.rs:100",
    "crates/sifr_package/src/projection.rs:109",
    "crates/sifr_package/src/projection.rs:127",
    "crates/sifr_package/src/projection.rs:129",
    "crates/sifr_package/src/projection.rs:169",
    "crates/sifr_package/src/projection.rs:187",
    "crates/sifr/src/lint_cli.rs:308",
    "crates/sifr/src/lint_cli.rs:496",
    "crates/sifr/src/lint_cli.rs:499",
    "crates/sifr/src/check_and_package_commands.rs:409",
    "crates/sifr/src/check_and_package_commands.rs:415",
    "crates/sifr/src/check_and_package_commands.rs:427",
    "crates/sifr/src/check_and_package_commands.rs:551",
    "crates/sifr/src/check_and_package_commands.rs:554",
    "crates/sifr/src/check_and_package_commands.rs:579",
    "crates/sifr/src/check_and_package_commands.rs:583",
    "crates/sifr/src/check_and_package_commands.rs:590",
    "crates/sifr/src/check_and_package_commands.rs:601",
    "crates/sifr/src/cli_model_and_entrypoint.rs:634",
    "crates/sifr/src/cli_model_and_entrypoint.rs:690",
    "crates/sifr/src/cli_model_and_entrypoint.rs:716",
    "crates/sifr/src/cli_model_and_entrypoint.rs:721",
    "path probes",
    "DocumentState::rebuild",
    "AnalysisHost::open_single_file",
    "M1-M4 remain serialized",
    "M5 updated",
    "M12 updated",
    "M13 updated",
    "M14 updated",
    "M15 updated",
    "M16 updated",
    "CancellationToken",
    "ParentWatchdog",
    "ProgressState",
    "SymbolBucketReadiness",
    "SymbolBucketReadinessState",
    "ApprovedWorkerLane",
    "SingleOwnerCompilerPhase",
    "ProjectResidencyKind",
    "WatchRegistrationReason",
    "SifrBuildInfoCandidate",
    "WorkspaceTracePhase",
    "WorkspaceStatusSnapshot",
    "WorkspaceDebugSnapshot",
    "sifr trace",
    "perf.lsp.request_families",
    "perf.lsp.generated_rust_preview.document",
]

EXPECTED_LSP_SCENARIOS = [
    "lsp.code_actions",
    "lsp.cold_start",
    "lsp.completion",
    "lsp.diagnostics",
    "lsp.did_open_diagnostics",
    "lsp.formatting",
    "lsp.generated_rust_preview",
    "lsp.hover",
    "lsp.inlay_hints",
    "lsp.navigation",
    "lsp.references",
    "lsp.rename",
    "lsp.request_families",
    "lsp.selection_range",
    "lsp.semantic_tokens",
    "lsp.signature_help",
    "lsp.type_hierarchy",
    "lsp.workspace_diagnostics",
]

EXPECTED_LSP_BUDGET_IDS = [
    "perf.lsp.code_action.diagnostic",
    "perf.lsp.cold_start.workspace",
    "perf.lsp.completion.local_scope",
    "perf.lsp.diagnostics.document",
    "perf.lsp.diagnostics.workspace",
    "perf.lsp.document_sync.did_open",
    "perf.lsp.formatting.document",
    "perf.lsp.generated_rust_preview.document",
    "perf.lsp.hover.symbol",
    "perf.lsp.inlay_hints.module",
    "perf.lsp.navigation.symbol",
    "perf.lsp.references.workspace_symbol",
    "perf.lsp.rename.workspace_edit",
    "perf.lsp.request_families",
    "perf.lsp.selection_ranges.nested",
    "perf.lsp.semantic_tokens.full",
    "perf.lsp.signature_help.call",
    "perf.lsp.type_hierarchy.symbol",
]


def require(condition: bool, message: str, failures: list[str]) -> None:
    if not condition:
        failures.append(message)


def validate_doc(text: str, failures: list[str]) -> None:
    for snippet in REQUIRED_DOC_SNIPPETS:
        require(snippet in text, f"M1 guardrail doc missing snippet: {snippet}", failures)


def is_production_source(path: Path) -> bool:
    if path == SOURCE_PROVIDER_BOUNDARY:
        return False
    relative_parts = path.relative_to(REPO_ROOT).parts
    if "tests" in relative_parts or "bin" in relative_parts:
        return False
    name = path.name
    return not (name.endswith("_tests.rs") or name == "tests.rs")


def direct_fs_sites() -> list[tuple[str, int, str]]:
    sites: list[tuple[str, int, str]] = []
    for root in DIRECT_FS_SCAN_ROOTS:
        for path in sorted(root.rglob("*.rs")):
            if not is_production_source(path):
                continue
            for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
                if DIRECT_FS_PATTERN.search(line):
                    sites.append((path.relative_to(REPO_ROOT).as_posix(), line_number, line.strip()))
    return sites


def validate_direct_fs_inventory(text: str, failures: list[str]) -> None:
    for path, line_number, source_line in direct_fs_sites():
        reference = f"{path}:{line_number}"
        require(
            reference in text,
            f"M1 direct-read/probe inventory missing {reference}: {source_line}",
            failures,
        )


def validate_source_maps(text: str, failures: list[str]) -> None:
    # These string checks intentionally pin the current `sifr_source` parameter
    # shape. If that API is renamed, update this guardrail with the signature
    # change so source-map conversions cannot silently become stubs again.
    require(
        "byte_offset_with_encoding(position, encoding)" in text,
        "SourceMapView::text_position_to_span must delegate to sifr_source conversion",
        failures,
    )
    require(
        "range_at(span, encoding)" in text,
        "SourceMapView::span_to_text_range must delegate to sifr_source conversion",
        failures,
    )
    require(
        "pub fn text_position_to_span" in text and "pub fn span_to_text_range" in text,
        "SourceMapView conversion APIs are missing",
        failures,
    )
    require(
        "-> Option<TextRange> {\n        None" not in text and "-> Option<TextRangeUtf> {\n        None" not in text,
        "SourceMapView conversion APIs must not be no-op stubs",
        failures,
    )


def validate_lsp_current_state(failures: list[str]) -> None:
    document_store = DOCUMENT_STORE.read_text(encoding="utf-8")
    analysis_workspace = LSP_ANALYSIS_WORKSPACE.read_text(encoding="utf-8")
    session = LSP_SESSION.read_text(encoding="utf-8")
    scheduler = SCHEDULER.read_text(encoding="utf-8")
    request_queue = REQUEST_QUEUE.read_text(encoding="utf-8")
    cancellation = CANCELLATION.read_text(encoding="utf-8")
    progress = PROGRESS.read_text(encoding="utf-8")
    watchdog = WATCHDOG.read_text(encoding="utf-8")
    require(
        "AnalysisHost::open_single_file" not in document_store
        and "FrontendMode::SingleFile" not in document_store
        and "with_host" not in document_store,
        "M5 requires DocumentStore to keep protocol text/version state without per-document hosts",
        failures,
    )
    require(
        "analysis: LspAnalysisWorkspace" in session
        and "with_document_analysis" in session
        and "LspAnalysisWorkspace::default()" in session,
        "M5 requires Session to own the persistent LSP analysis workspace",
        failures,
    )
    require(
        "open_single_file_overlay" in analysis_workspace
        and "upsert_overlay_document" in analysis_workspace
        and "load_diagnostics" in analysis_workspace,
        "M5 requires LSP analysis to load and update documents through workspace overlays",
        failures,
    )
    require(
        "pub(crate) fn lane_for_method" in scheduler
        and "Background" in scheduler
        and "CancellationToken" not in scheduler,
        "scheduler guardrail expects lane classification to remain separate from cancellation state",
        failures,
    )
    require(
        "VecDeque<QueuedRequest>" in request_queue
        and "start_next" in request_queue
        and "FAIRNESS_INTERVAL" in request_queue
        and "CancellationTarget" in request_queue
        and "is_cancelled" in request_queue,
        "request queue guardrail expects M11 priority queues plus M13 cancellation state",
        failures,
    )
    require(
        "ProgressState" in progress
        and "ProgressKind" in progress
        and "FullDiagnostics" in progress,
        "M13 requires delayed LSP progress state for long-running work",
        failures,
    )
    require(
        "CancellationToken" in cancellation
        and "request_id" in cancellation
        and "RequestId" in cancellation,
        "M13 requires explicit LSP request cancellation tokens",
        failures,
    )
    require(
        "ParentWatchdog" in watchdog
        and "LspServerOptions" in watchdog
        and "parent_pid" in watchdog,
        "M13 requires parent-pid watchdog options for LSP",
        failures,
    )


def validate_lsp_budget_reality(failures: list[str]) -> None:
    manifest = json.loads(PERF_MANIFEST.read_text(encoding="utf-8"))
    cases = manifest.get("cases", []) if isinstance(manifest, dict) else manifest
    lsp_cases = [
        case
        for case in cases
        if case.get("kind") == "lsp-query" or str(case.get("scenario", "")).startswith("lsp.")
    ]
    scenarios = sorted(case.get("scenario") for case in lsp_cases)
    require(
        scenarios == EXPECTED_LSP_SCENARIOS,
        f"M12 expects split per-request LSP budget scenarios, found: {scenarios}",
        failures,
    )
    budget_ids = sorted(case.get("budget_id") for case in lsp_cases)
    require(
        budget_ids == EXPECTED_LSP_BUDGET_IDS,
        f"M12 expects split per-request LSP budget ids, found: {budget_ids}",
        failures,
    )
    aggregate = [case for case in lsp_cases if case.get("scenario") == "lsp.request_families"]
    require(
        len(aggregate) == 1 and aggregate[0].get("evidence_category") == "lsp-query-smoke",
        "M12 requires perf.lsp.request_families to remain as aggregate smoke coverage only",
        failures,
    )


def validate_m14_bucket_and_lane_state(failures: list[str]) -> None:
    symbols = ANALYSIS_SYMBOLS.read_text(encoding="utf-8")
    worker_lanes = ANALYSIS_WORKER_LANES.read_text(encoding="utf-8")
    require(
        "SymbolBucketReadiness" in symbols
        and "SymbolBucketKind::Workspace" in symbols
        and "SymbolBucketKind::Package" in symbols
        and "SymbolBucketKind::Stdlib" in symbols
        and "refresh_modules" in symbols
        and "completion_symbols" in symbols
        and "workspace_import_symbols" in symbols
        and "import_entry_count" in symbols,
        "M14 requires bucketed workspace/package/stdlib symbol and import readiness state",
        failures,
    )
    require(
        "SymbolBucketReadinessState::Exact" in symbols
        and "SymbolBucketReadinessState::StaleButUsable" in symbols
        and "SymbolBucketReadinessState::NeedsBackgroundRefresh" in symbols
        and "SymbolBucketReadinessState::Unavailable" in symbols,
        "M14 requires all symbol bucket readiness states to remain representable",
        failures,
    )
    require(
        "ApprovedWorkerLane" in worker_lanes
        and "SourceMapCreation" in worker_lanes
        and "IndependentHirLower" in worker_lanes
        and "SingleOwnerCompilerPhase" in worker_lanes
        and "TypeIdentityCreation" in worker_lanes
        and "OwnershipMutation" in worker_lanes
        and "PackageGraphMutation" in worker_lanes
        and "CodegenState" in worker_lanes,
        "M14 requires approved worker lanes plus explicit single-owner compiler phases",
        failures,
    )
    host = (REPO_ROOT / "crates" / "sifr_analysis" / "src" / "host" / "implementation.rs").read_text(
        encoding="utf-8"
    )
    tests = (REPO_ROOT / "crates" / "sifr_analysis" / "src" / "host" / "tests.rs").read_text(
        encoding="utf-8"
    )
    require(
        "completion_symbols" in host
        and "workspace_import_symbols" in host
        and "project_symbol_index_refreshes_dirty_module_buckets_only" in tests
        and "workspace_diagnostic_order_is_stable_across_repeated_queries" in tests,
        "M14 requires host queries and regression tests to exercise bucketed symbols/imports",
        failures,
    )


def validate_m15_residency_state(failures: list[str]) -> None:
    session = WORKSPACE_SESSION.read_text(encoding="utf-8")
    residency = WORKSPACE_RESIDENCY.read_text(encoding="utf-8")
    require(
        "WorkspaceResidencySnapshot" in session
        and "refresh_residency" in session
        and "verify_build_info" in session
        and "mark_config_pending_reload" in session,
        "M15 requires WorkspaceSession to expose residency snapshots and build-info verification",
        failures,
    )
    require(
        "ProjectResidencyKind" in residency
        and "OpenFileOwner" in residency
        and "ExplicitApiOpen" in residency
        and "Evictable" in residency
        and "ConfigRegistryEntry" in residency
        and "pending_reload" in residency
        and "WatchRegistrationReason" in residency
        and "FailedLookup" in residency
        and "GeneratedArtifact" in residency
        and "StdlibRoot" in residency
        and "SifrBuildInfoCandidate" in residency
        and "SifrBuildInfoVerification" in residency
        and "SourceHashMismatch" in residency,
        "M15 requires residency/config/watch/build-info state vocabulary",
        failures,
    )


def validate_m16_trace_status_state(failures: list[str]) -> None:
    session = WORKSPACE_SESSION.read_text(encoding="utf-8")
    trace = WORKSPACE_TRACE.read_text(encoding="utf-8")
    lsp_session = LSP_SESSION.read_text(encoding="utf-8")
    lsp_analysis = LSP_ANALYSIS_WORKSPACE.read_text(encoding="utf-8")
    lsp_requests = LSP_REQUESTS.read_text(encoding="utf-8")
    analysis_debug = ANALYSIS_DEBUG_STATUS.read_text(encoding="utf-8")
    trace_cli = TRACE_CLI.read_text(encoding="utf-8")
    require(
        "WorkspaceDebugSnapshot" in session
        and "record_compiler_phase_trace" in session
        and "record_stale_rejection" in session
        and "record_update_latency_ms" in session,
        "M16 requires WorkspaceSession snapshots to expose deterministic debug trace/status state",
        failures,
    )
    require(
        "WorkspaceTracePhase" in trace
        and "MAX_TRACE_EVENTS" in trace
        and "SourceUpdate" in trace
        and "Parse" in trace
        and "Lower" in trace
        and "TypeCheck" in trace
        and "Ownership" in trace
        and "Flow" in trace
        and "Cache" in trace
        and "Invalidation" in trace
        and "Scheduler" in trace
        and "Cancellation" in trace
        and "StaleRejection" in trace
        and "LspTiming" in trace
        and "WorkspaceStatusSnapshot" in trace
        and "WorkspaceMemoryCounters" in trace,
        "M16 requires normalized trace phases plus status and memory-counter vocabulary",
        failures,
    )
    require(
        "WorkspaceTraceEvent" in lsp_session
        and "MAX_LSP_TRACE_EVENTS" in lsp_session
        and "WorkspaceTracePhase::Scheduler" in lsp_session
        and "WorkspaceTracePhase::Cancellation" in lsp_session
        and "WorkspaceTracePhase::StaleRejection" in lsp_session
        and "WorkspaceTracePhase::LspTiming" in lsp_session,
        "M16 requires LSP scheduler/cancellation/stale/timing trace events",
        failures,
    )
    require(
        "record_update_latency_ms" in lsp_analysis and "elapsed_ms" in lsp_analysis,
        "M16 requires LSP analysis open/update paths to feed status latency counters",
        failures,
    )
    require(
        '"sifr/debugTrace"' in lsp_requests and "trace_snapshot().render_text()" in lsp_requests,
        "M16 requires an editor-reachable LSP debug trace request",
        failures,
    )
    require(
        "WorkspaceIndexReadinessStatus" in analysis_debug
        and "bucket_readiness" in analysis_debug
        and "symbol_index.as_ref()" in analysis_debug
        and "symbol_index()?" not in analysis_debug,
        "M16 requires editor debug status to include readiness without building indexes on demand",
        failures,
    )
    require(
        "cmd_trace" in trace_cli and "trace_entrypoint" in trace_cli and "render_text" in trace_cli,
        "M16 requires a representative CLI trace snapshot command",
        failures,
    )


def validate_source_dep_guard(failures: list[str]) -> None:
    result = subprocess.run(
        [sys.executable, str(SOURCE_DEP_GUARD)],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    require(result.returncode == 0, f"sifr_source dependency guard failed:\n{result.stdout}", failures)


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        incomplete_doc = Path(tmp) / "m1.md"
        incomplete_doc.write_text("WorkspaceSession only\n", encoding="utf-8")
        failures: list[str] = []
        validate_doc(incomplete_doc.read_text(encoding="utf-8"), failures)
    if not failures:
        raise SystemExit("M1 guardrail self-test failed: incomplete doc passed")
    failures = []
    validate_direct_fs_inventory("WorkspaceSession only\n", failures)
    if not failures:
        raise SystemExit("M1 guardrail self-test failed: incomplete inventory passed")
    print("TypeScript-Go M1 guardrail self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures: list[str] = []
    doc_text = DOC.read_text(encoding="utf-8")
    validate_doc(doc_text, failures)
    validate_direct_fs_inventory(doc_text, failures)
    validate_source_maps(SOURCE_MAPS.read_text(encoding="utf-8"), failures)
    validate_lsp_current_state(failures)
    validate_lsp_budget_reality(failures)
    validate_m14_bucket_and_lane_state(failures)
    validate_m15_residency_state(failures)
    validate_m16_trace_status_state(failures)
    validate_source_dep_guard(failures)

    if failures:
        print("TypeScript-Go M1 guardrails: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("TypeScript-Go M1 guardrails: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
