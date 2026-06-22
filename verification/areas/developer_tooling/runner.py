"""Developer tooling verification area adapter."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
MANIFEST_PATH = AREA_ROOT / "manifest.json"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "developer-tooling-results.json"
LSP_CORPUS = AREA_ROOT / "corpora" / "sifr-large-lsp-verification"

SUITE_COMMANDS: dict[str, list[tuple[str, list[str]]]] = {
    "typescript-go-transfer": [
        ("typescript-go-transfer", [sys.executable, str(AREA_ROOT / "check_typescript_go_transfer_guardrails.py")]),
        ("typescript-go-transfer-self-test", [sys.executable, str(AREA_ROOT / "check_typescript_go_transfer_guardrails.py"), "--self-test"]),
    ],
    "diagnostic-rules": [
        ("diagnostic-presentation", [sys.executable, str(AREA_ROOT / "check_diagnostic_presentation_rules.py")]),
        (
            "diagnostic-presentation-self-test",
            [sys.executable, str(AREA_ROOT / "check_diagnostic_presentation_rules.py"), "--self-test"],
        ),
        (
            "diagnostic-source-canonicalization",
            [sys.executable, str(AREA_ROOT / "check_diagnostic_source_canonicalization_rules.py")],
        ),
        (
            "diagnostic-source-canonicalization-self-test",
            [
                sys.executable,
                str(AREA_ROOT / "check_diagnostic_source_canonicalization_rules.py"),
                "--self-test",
            ],
        ),
    ],
    "static": [
        ("tooling-rules-lock", [sys.executable, str(AREA_ROOT / "check_tooling_rules_lock.py")]),
        ("tooling-rules-lock-self-test", [sys.executable, str(AREA_ROOT / "check_tooling_rules_lock.py"), "--self-test"]),
        ("tooling-dependency-boundaries", [sys.executable, str(AREA_ROOT / "check_tooling_dependency_boundaries.py")]),
        (
            "tooling-dependency-boundaries-self-test",
            [sys.executable, str(AREA_ROOT / "check_tooling_dependency_boundaries.py"), "--self-test"],
        ),
        ("lsp-split-brain", [sys.executable, str(AREA_ROOT / "check_lsp_split_brain.py")]),
        ("lsp-split-brain-self-test", [sys.executable, str(AREA_ROOT / "check_lsp_split_brain.py"), "--self-test"]),
        ("linter-diagnostic-class", [sys.executable, str(AREA_ROOT / "check_linter_diagnostic_class.py")]),
        (
            "linter-diagnostic-class-self-test",
            [sys.executable, str(AREA_ROOT / "check_linter_diagnostic_class.py"), "--self-test"],
        ),
        ("rule-suppression-rules", [sys.executable, str(AREA_ROOT / "check_rule_suppression_rules.py")]),
        (
            "rule-suppression-rules-self-test",
            [sys.executable, str(AREA_ROOT / "check_rule_suppression_rules.py"), "--self-test"],
        ),
        ("completion-quality", [sys.executable, str(AREA_ROOT / "check_completion_quality.py")]),
        ("completion-quality-self-test", [sys.executable, str(AREA_ROOT / "check_completion_quality.py"), "--self-test"]),
    ],
    "formatter": [
        ("formatter-rules", [sys.executable, str(AREA_ROOT / "check_formatter_rules.py")]),
        ("formatter-rules-self-test", [sys.executable, str(AREA_ROOT / "check_formatter_rules.py"), "--self-test"]),
        ("formatter-rules-manifests", [sys.executable, str(AREA_ROOT / "check_formatter_rules_manifests.py")]),
        (
            "formatter-rules-manifests-self-test",
            [sys.executable, str(AREA_ROOT / "check_formatter_rules_manifests.py"), "--self-test"],
        ),
        ("formatter-ast-coverage", [sys.executable, str(AREA_ROOT / "check_formatter_ast_coverage.py")]),
        (
            "formatter-ast-coverage-self-test",
            [sys.executable, str(AREA_ROOT / "check_formatter_ast_coverage.py"), "--self-test"],
        ),
    ],
    "analysis": [
        ("analysis-snapshot-rules", [sys.executable, str(AREA_ROOT / "check_analysis_snapshot_rules.py")]),
        (
            "analysis-snapshot-rules-self-test",
            [sys.executable, str(AREA_ROOT / "check_analysis_snapshot_rules.py"), "--self-test"],
        ),
        ("analysis-snapshot-coherence", [sys.executable, str(AREA_ROOT / "check_analysis_snapshot_coherence.py")]),
        (
            "analysis-snapshot-coherence-self-test",
            [sys.executable, str(AREA_ROOT / "check_analysis_snapshot_coherence.py"), "--self-test"],
        ),
        ("analysis-split-brain", [sys.executable, str(AREA_ROOT / "check_analysis_split_brain.py")]),
        ("analysis-split-brain-self-test", [sys.executable, str(AREA_ROOT / "check_analysis_split_brain.py"), "--self-test"]),
        ("tooling-parity", [sys.executable, str(AREA_ROOT / "run_tooling_parity.py")]),
        ("tooling-parity-self-test", [sys.executable, str(AREA_ROOT / "run_tooling_parity.py"), "--self-test"]),
    ],
    "lsp-smoke": [
        ("lsp-protocol-smoke", [sys.executable, str(AREA_ROOT / "lsp_protocol_smoke.py")]),
        ("lsp-protocol-smoke-self-test", [sys.executable, str(AREA_ROOT / "lsp_protocol_smoke.py"), "--self-test"]),
        ("lsp-marker-corpus", [sys.executable, str(AREA_ROOT / "check_lsp_marker_corpus.py")]),
        ("lsp-marker-corpus-self-test", [sys.executable, str(AREA_ROOT / "check_lsp_marker_corpus.py"), "--self-test"]),
        ("lsp-transcript-replay", [sys.executable, str(AREA_ROOT / "check_lsp_transcript_replay.py")]),
        ("lsp-transcript-replay-self-test", [sys.executable, str(AREA_ROOT / "check_lsp_transcript_replay.py"), "--self-test"]),
    ],
    "lsp-semantic-editor": [
        ("lsp-semantic-editor", [sys.executable, str(AREA_ROOT / "lsp_semantic_editor_corpus.py")]),
        (
            "lsp-semantic-editor-self-test",
            [sys.executable, str(AREA_ROOT / "lsp_semantic_editor_corpus.py"), "--self-test"],
        ),
    ],
    "editor-release": [
        ("vscode-extension-rules", [sys.executable, str(AREA_ROOT / "check_vscode_extension_rules.py")]),
        (
            "vscode-extension-rules-self-test",
            [sys.executable, str(AREA_ROOT / "check_vscode_extension_rules.py"), "--self-test"],
        ),
        ("vscode-extension", [sys.executable, str(AREA_ROOT / "check_vscode_extension.py")]),
        ("vscode-extension-self-test", [sys.executable, str(AREA_ROOT / "check_vscode_extension.py"), "--self-test"]),
        ("editor-assets", [sys.executable, str(AREA_ROOT / "check_editor_assets.py")]),
        ("editor-assets-self-test", [sys.executable, str(AREA_ROOT / "check_editor_assets.py"), "--self-test"]),
    ],
    "lsp-stress": [
        ("lsp-protocol-stress", [sys.executable, str(AREA_ROOT / "lsp_protocol_stress.py")]),
        ("lsp-protocol-stress-self-test", [sys.executable, str(AREA_ROOT / "lsp_protocol_stress.py"), "--self-test"]),
        ("large-lsp-submodule", ["git", "submodule", "update", "--init", str(LSP_CORPUS.relative_to(REPO_ROOT))]),
        (
            "large-lsp-corpus-check",
            [sys.executable, str(LSP_CORPUS / "tools" / "generate_corpus.py"), "check"],
        ),
        ("large-lsp-session-self-test", [sys.executable, str(AREA_ROOT / "lsp_large_session.py"), "--self-test"]),
        (
            "large-lsp-session-smoke",
            [sys.executable, str(AREA_ROOT / "lsp_large_session.py"), "--mode", "smoke", "--require-submodule"],
        ),
    ],
    "tooling-readiness": [
        ("tooling-readiness", [sys.executable, str(AREA_ROOT / "check_tooling_readiness.py")]),
        ("tooling-readiness-self-test", [sys.executable, str(AREA_ROOT / "check_tooling_readiness.py"), "--self-test"]),
    ],
}

FULL_SUITES = [
    "static",
    "formatter",
    "analysis",
    "lsp-smoke",
    "lsp-semantic-editor",
    "editor-release",
    "lsp-stress",
    "tooling-readiness",
]


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable developer tooling result summary.",
    )
    parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit a legacy verification summary line for direct area invocations.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("developer_tooling area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running developer tooling verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "developer_tooling",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": total_failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = REPO_ROOT / args.result_json
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}", flush=True)

    if total_failures:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={total_failures}, non_blocking_failures=0",
            file=sys.stderr,
            flush=True,
        )
        return 1
    prefix = "verification ok" if args.hardening_summary else "developer tooling verification ok"
    print(
        f"{prefix}: variants={total_variants}, failures={total_failures}, "
        "blocking_failures=0, non_blocking_failures=0",
        flush=True,
    )
    return 0


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [suite for suite in suites if not requested or str(suite.get("name")) in requested]
    if requested:
        present = {str(suite.get("name")) for suite in selected}
        missing = sorted(requested.difference(present))
        if missing:
            raise SystemExit(f"unknown developer_tooling suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no developer_tooling suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    if suite_name == "full":
        commands = [(f"{name}:{label}", argv) for name in FULL_SUITES for label, argv in SUITE_COMMANDS[name]]
    else:
        commands = SUITE_COMMANDS[suite_name]
    variants = [run_command_variant(suite_name, label, argv) for label, argv in commands]
    failures = sum(1 for variant in variants if variant["status"] != "pass")
    cases = suite.get("cases", [])
    case = cases[0] if cases else {"id": suite_name, "entry": str(MANIFEST_PATH.relative_to(REPO_ROOT))}
    return {
        "name": suite_name,
        "owner": "compiler/tooling",
        "blocking": True,
        "runner": "developer-tooling",
        "cases": [
            {
                "id": str(case["id"]),
                "entry": str(case["entry"]),
                "command": str(case.get("command", "developer-tooling-suite")),
                "variants": variants,
            }
        ],
        "failed_cases": 1 if failures else 0,
        "total_variants": len(variants),
        "total_failures": failures,
    }


def run_command_variant(suite_name: str, label: str, argv: list[str]) -> dict[str, Any]:
    started = time.perf_counter()
    proc = subprocess.run(argv, cwd=REPO_ROOT, text=True, check=False)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    status = "pass" if proc.returncode == 0 else "fail"
    print(
        f"[sifr-case-timing] bucket=developer_tooling case={timing_token(suite_name)}/{timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )
    return {
        "label": label,
        "argv": argv,
        "status": status,
        "mismatches": [] if status == "pass" else ["unexpected-exit"],
        "expected_exit_code": 0,
        "actual_exit_code": proc.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
