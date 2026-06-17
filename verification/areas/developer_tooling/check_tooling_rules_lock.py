#!/usr/bin/env python3
"""Validate the editor tooling rules lock."""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
MATRIX_PATH = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "lsp_protocol_matrix.json"
VSCODE_RULES_PATH = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "vscode_extension_rules.json"

REQUIRED_DOCS = [
    REPO_ROOT / "internal_docs" / "tooling_analysis.md",
    REPO_ROOT / "internal_docs" / "lsp_server.md",
    REPO_ROOT / "internal_docs" / "vscode_extension.md",
    REPO_ROOT / "internal_docs" / "editor_integrations.md",
    REPO_ROOT / "internal_docs" / "tooling_verification.md",
]

REQUIRED_CRATES = {
    "analysis": "sifr_analysis",
    "formatter": "sifr_format",
    "lint": "sifr_lint",
    "lsp": "sifr_lsp",
}

REQUIRED_METHODS = {
    "initialize",
    "initialized",
    "shutdown",
    "exit",
    "workspace/didChangeConfiguration",
    "workspace/didChangeWatchedFiles",
    "workspace/symbol",
    "workspace/executeCommand",
    "textDocument/didOpen",
    "textDocument/didChange",
    "textDocument/didSave",
    "textDocument/didClose",
    "textDocument/publishDiagnostics",
    "textDocument/diagnostic",
    "workspace/diagnostic",
    "textDocument/completion",
    "completionItem/resolve",
    "textDocument/hover",
    "textDocument/signatureHelp",
    "textDocument/definition",
    "textDocument/declaration",
    "textDocument/typeDefinition",
    "textDocument/references",
    "textDocument/prepareRename",
    "textDocument/rename",
    "textDocument/documentSymbol",
    "textDocument/semanticTokens/full",
    "textDocument/semanticTokens/range",
    "textDocument/inlayHint",
    "textDocument/documentHighlight",
    "textDocument/foldingRange",
    "textDocument/selectionRange",
    "textDocument/prepareTypeHierarchy",
    "typeHierarchy/supertypes",
    "typeHierarchy/subtypes",
    "textDocument/codeAction",
    "codeAction/resolve",
    "textDocument/formatting",
    "textDocument/rangeFormatting",
}

REQUIRED_COMMANDS = {
    "sifr.restartServer",
    "sifr.showServerLogs",
    "sifr.explainDiagnostic",
    "sifr.showGeneratedRust",
    "sifr.checkWorkspace",
    "sifr.runTests",
}

REQUIRED_SETTINGS = {
    "sifr.diagnostics.mode",
    "sifr.lsp.trace.server",
    "sifr.format.enable",
    "sifr.lint.enable",
}

REQUIRED_TOKEN_TYPES = {
    "keyword",
    "type",
    "function",
    "method",
    "variable",
    "parameter",
    "property",
    "module",
    "comment",
    "string",
    "number",
    "operator",
    "decorator",
    "mutableBinding",
    "ownershipSensitive",
}


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_matrix(matrix: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if matrix.get("protocol", {}).get("target_version") != "3.17":
        failures.append("LSP protocol target must be 3.17")
    if matrix.get("protocol", {}).get("launch") != ["sifr", "lsp", "--stdio"]:
        failures.append("LSP launch command must be sifr lsp --stdio")
    if matrix.get("crate_names") != REQUIRED_CRATES:
        failures.append("final tooling crate names are not locked to the editor tooling rules")

    modes = set(matrix.get("diagnostics_modes", []))
    if modes != {"off", "open-files", "workspace"}:
        failures.append("diagnostics modes must be exactly off/open-files/workspace")

    missing_tokens = REQUIRED_TOKEN_TYPES - set(matrix.get("semantic_token_legend", []))
    if missing_tokens:
        failures.append(f"semantic token legend missing: {sorted(missing_tokens)}")

    settings = {item.get("key") for item in matrix.get("settings", [])}
    missing_settings = REQUIRED_SETTINGS - settings
    if missing_settings:
        failures.append(f"LSP settings missing: {sorted(missing_settings)}")

    methods = {item.get("method") for item in matrix.get("required_methods", [])}
    missing_methods = REQUIRED_METHODS - methods
    if missing_methods:
        failures.append(f"LSP methods missing: {sorted(missing_methods)}")

    commands = {item.get("command") for item in matrix.get("required_commands", [])}
    missing_commands = REQUIRED_COMMANDS - commands
    if missing_commands:
        failures.append(f"LSP commands missing: {sorted(missing_commands)}")

    for item in matrix.get("required_methods", []):
        method = item.get("method", "<unknown>")
        if not item.get("owner"):
            failures.append(f"{method} has no LSP owner")
        if "positive" not in item or "negative" not in item:
            failures.append(f"{method} must record positive and negative coverage")

    unsupported = {item.get("surface") for item in matrix.get("unsupported_surfaces", [])}
    for surface in ["notebookDocumentSync", "python.importResolution", "python.environmentDiscovery"]:
        if surface not in unsupported:
            failures.append(f"unsupported surface missing from matrix: {surface}")

    return failures


def validate_vscode_rules(rules: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    repo = rules.get("repository", {})
    if repo.get("boundary") != "separate-repository":
        failures.append("VS Code extension boundary must be separate-repository")
    if repo.get("full_name") != "sifr-lang/sifr-vscode":
        failures.append("VS Code extension repository must be sifr-lang/sifr-vscode")
    if rules.get("launch", {}).get("default_command") != "sifr":
        failures.append("VS Code default launch command must be sifr")
    if rules.get("launch", {}).get("default_args") != ["lsp", "--stdio"]:
        failures.append("VS Code launch args must be lsp --stdio")
    if repo.get("submodule_checkout") != "editor_integrations/vscode":
        failures.append("VS Code extension submodule checkout must be editor_integrations/vscode")
    extension = rules.get("extension", {})
    if extension.get("language_id") != "sifr":
        failures.append("VS Code language id must be sifr")
    if ".sifr" not in extension.get("extensions", []):
        failures.append("VS Code extension must register .sifr")
    forbidden = set(rules.get("forbidden_extension_behavior", []))
    for required in ["parser", "typeChecker", "formatter", "linter", "codegen", "pyrightFallback"]:
        if required not in forbidden:
            failures.append(f"VS Code forbidden behavior missing: {required}")
    return failures


def validate_paths() -> list[str]:
    failures = [f"required doc missing: {path.relative_to(REPO_ROOT)}" for path in REQUIRED_DOCS if not path.exists()]
    if not MATRIX_PATH.exists():
        failures.append(f"required matrix missing: {MATRIX_PATH.relative_to(REPO_ROOT)}")
    if not VSCODE_RULES_PATH.exists():
        failures.append(f"required VS Code rules missing: {VSCODE_RULES_PATH.relative_to(REPO_ROOT)}")
    return failures


def validate() -> list[str]:
    failures = validate_paths()
    if not failures:
        failures.extend(validate_matrix(read_json(MATRIX_PATH)))
        failures.extend(validate_vscode_rules(read_json(VSCODE_RULES_PATH)))
    return failures


def run_self_test() -> None:
    matrix = read_json(MATRIX_PATH)
    matrix["required_methods"] = [
        item for item in matrix["required_methods"] if item.get("method") != "textDocument/completion"
    ]
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "bad_matrix.json"
        path.write_text(json.dumps(matrix), encoding="utf-8")
        failures = validate_matrix(read_json(path))
    if not any("textDocument/completion" in failure for failure in failures):
        raise SystemExit("rules lock self-test failed: missing completion method passed")
    print("tooling rules lock self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = validate()
    if failures:
        print("tooling rules lock: FAIL")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("tooling rules lock: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
