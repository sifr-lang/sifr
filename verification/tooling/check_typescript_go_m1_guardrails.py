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
SCHEDULER = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "scheduler.rs"
REQUEST_QUEUE = REPO_ROOT / "crates" / "sifr_lsp" / "src" / "request_queue.rs"
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
    "M12 must update",
    "perf.lsp.request_families",
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
        "scheduler guardrail expects M11 lane classification, with cancellation deferred",
        failures,
    )
    require(
        "VecDeque<QueuedRequest>" in request_queue
        and "start_next" in request_queue
        and "FAIRNESS_INTERVAL" in request_queue
        and "remove_pending" in request_queue,
        "request queue guardrail expects M11 priority queues with bounded fairness",
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
        scenarios == ["lsp.request_families"],
        f"M1 expects aggregate-only LSP budget reality until M12, found: {scenarios}",
        failures,
    )
    require(
        lsp_cases[0].get("budget_id") == "perf.lsp.request_families" if lsp_cases else False,
        "M1 aggregate LSP budget id must be perf.lsp.request_families",
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
