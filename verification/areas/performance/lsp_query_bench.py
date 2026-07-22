#!/usr/bin/env python3
"""Benchmark protocol-level Sifr LSP request families."""

from __future__ import annotations

import json
import sys
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path
from typing import Callable, Iterator

REPO_ROOT = Path(__file__).resolve().parents[3]
TOOLING_ROOT = REPO_ROOT / "verification" / "areas" / "developer_tooling"
sys.path.insert(0, str(TOOLING_ROOT))

from lsp_protocol import LspClient, file_uri  # noqa: E402
from lsp_protocol_smoke import initialize, open_document  # noqa: E402


def request_context(
    uri: str,
) -> tuple[dict[str, str], dict[str, int], dict[str, int], dict[str, dict[str, int]]]:
    document = {"uri": uri}
    helper_position = {"line": 4, "character": 19}
    result_position = {"line": 5, "character": 11}
    range_value = {
        "start": {"line": 0, "character": 0},
        "end": {"line": 6, "character": 0},
    }
    return document, helper_position, result_position, range_value


def run_family(client: LspClient, uri: str) -> None:
    document, helper_position, result_position, range_value = request_context(uri)
    client.request("textDocument/documentSymbol", {"textDocument": document})
    client.request("workspace/symbol", {"query": "helper"})
    client.request("textDocument/completion", {"textDocument": document, "position": helper_position})
    client.request("textDocument/hover", {"textDocument": document, "position": helper_position})
    client.request(
        "textDocument/signatureHelp",
        {"textDocument": document, "position": {"line": 4, "character": 24}},
    )
    client.request("textDocument/definition", {"textDocument": document, "position": helper_position})
    client.request("textDocument/declaration", {"textDocument": document, "position": helper_position})
    client.request("textDocument/typeDefinition", {"textDocument": document, "position": helper_position})
    client.request("textDocument/references", {"textDocument": document, "position": result_position})
    client.request("textDocument/prepareRename", {"textDocument": document, "position": result_position})
    client.request(
        "textDocument/rename",
        {"textDocument": document, "position": result_position, "newName": "renamed_result"},
    )
    client.request("textDocument/semanticTokens/full", {"textDocument": document})
    client.request("textDocument/semanticTokens/range", {"textDocument": document, "range": range_value})
    client.request("textDocument/inlayHint", {"textDocument": document, "range": range_value})
    client.request("textDocument/documentHighlight", {"textDocument": document, "position": result_position})
    client.request("textDocument/foldingRange", {"textDocument": document})
    client.request("textDocument/selectionRange", {"textDocument": document, "positions": [result_position]})
    client.request("textDocument/prepareTypeHierarchy", {"textDocument": document, "position": result_position})
    client.request(
        "textDocument/codeAction",
        {"textDocument": document, "range": range_value, "context": {"diagnostics": []}},
    )
    client.request("textDocument/formatting", {"textDocument": document, "options": {"tabSize": 4}})
    client.request(
        "textDocument/rangeFormatting",
        {"textDocument": document, "range": range_value, "options": {"tabSize": 4}},
    )
    client.request("textDocument/diagnostic", {"textDocument": document})
    client.request("workspace/diagnostic", {})
    client.request("workspace/executeCommand", {"command": "sifr.server.showGeneratedRust", "arguments": [uri]})


def run_completion(client: LspClient, uri: str) -> None:
    document, helper_position, _, _ = request_context(uri)
    client.request("textDocument/completion", {"textDocument": document, "position": helper_position})


def run_hover(client: LspClient, uri: str) -> None:
    document, helper_position, _, _ = request_context(uri)
    client.request("textDocument/hover", {"textDocument": document, "position": helper_position})


def run_signature_help(client: LspClient, uri: str) -> None:
    document, _, _, _ = request_context(uri)
    client.request(
        "textDocument/signatureHelp",
        {"textDocument": document, "position": {"line": 4, "character": 24}},
    )


def run_navigation(client: LspClient, uri: str) -> None:
    document, helper_position, result_position, _ = request_context(uri)
    client.request("textDocument/documentSymbol", {"textDocument": document})
    client.request("workspace/symbol", {"query": "helper"})
    client.request("textDocument/definition", {"textDocument": document, "position": helper_position})
    client.request("textDocument/declaration", {"textDocument": document, "position": helper_position})
    client.request("textDocument/typeDefinition", {"textDocument": document, "position": helper_position})
    client.request("textDocument/documentHighlight", {"textDocument": document, "position": result_position})
    client.request("textDocument/foldingRange", {"textDocument": document})


def run_references(client: LspClient, uri: str) -> None:
    document, _, result_position, _ = request_context(uri)
    client.request("textDocument/references", {"textDocument": document, "position": result_position})


def run_rename(client: LspClient, uri: str) -> None:
    document, _, result_position, _ = request_context(uri)
    client.request("textDocument/prepareRename", {"textDocument": document, "position": result_position})
    client.request(
        "textDocument/rename",
        {"textDocument": document, "position": result_position, "newName": "renamed_result"},
    )


def run_semantic_tokens(client: LspClient, uri: str) -> None:
    document, _, _, range_value = request_context(uri)
    client.request("textDocument/semanticTokens/full", {"textDocument": document})
    client.request("textDocument/semanticTokens/range", {"textDocument": document, "range": range_value})


def run_inlay_hints(client: LspClient, uri: str) -> None:
    document, _, _, range_value = request_context(uri)
    client.request("textDocument/inlayHint", {"textDocument": document, "range": range_value})


def run_selection_range(client: LspClient, uri: str) -> None:
    document, _, result_position, _ = request_context(uri)
    client.request("textDocument/selectionRange", {"textDocument": document, "positions": [result_position]})


def run_type_hierarchy(client: LspClient, uri: str) -> None:
    document, _, result_position, _ = request_context(uri)
    client.request("textDocument/prepareTypeHierarchy", {"textDocument": document, "position": result_position})


def run_code_actions(client: LspClient, uri: str) -> None:
    document, _, _, range_value = request_context(uri)
    client.request(
        "textDocument/codeAction",
        {"textDocument": document, "range": range_value, "context": {"diagnostics": []}},
    )


def run_formatting(client: LspClient, uri: str) -> None:
    document, _, _, range_value = request_context(uri)
    client.request("textDocument/formatting", {"textDocument": document, "options": {"tabSize": 4}})
    client.request(
        "textDocument/rangeFormatting",
        {"textDocument": document, "range": range_value, "options": {"tabSize": 4}},
    )


def run_workspace_diagnostics(client: LspClient, _uri: str) -> None:
    client.request("workspace/diagnostic", {})


def run_generated_rust_preview(client: LspClient, uri: str) -> None:
    client.request("workspace/executeCommand", {"command": "sifr.server.showGeneratedRust", "arguments": [uri]})


WARM_SCENARIOS: dict[str, Callable[[LspClient, str], None]] = {
    "lsp.request_families": run_family,
    "lsp.code_actions": run_code_actions,
    "lsp.completion": run_completion,
    "lsp.formatting": run_formatting,
    "lsp.generated_rust_preview": run_generated_rust_preview,
    "lsp.hover": run_hover,
    "lsp.inlay_hints": run_inlay_hints,
    "lsp.navigation": run_navigation,
    "lsp.references": run_references,
    "lsp.rename": run_rename,
    "lsp.selection_range": run_selection_range,
    "lsp.semantic_tokens": run_semantic_tokens,
    "lsp.signature_help": run_signature_help,
    "lsp.type_hierarchy": run_type_hierarchy,
    "lsp.workspace_diagnostics": run_workspace_diagnostics,
}


def run_document_diagnostics(client: LspClient, uri: str, source: str, state: dict[str, int]) -> None:
    document, _, _, _ = request_context(uri)
    state["version"] += 1
    replacement = source.replace("return result\n", f"return result + {state['version'] % 2}\n")
    client.notify(
        "textDocument/didChange",
        {
            "textDocument": {"uri": uri, "version": state["version"]},
            "contentChanges": [{"text": replacement}],
        },
    )
    client.wait_for_notification("textDocument/publishDiagnostics")
    client.request("textDocument/diagnostic", {"textDocument": document})


def run_cold_start(source_path: Path, iterations: int) -> list[float]:
    samples: list[float] = []
    for _ in range(iterations):
        started = time.perf_counter()
        client = LspClient(timeout=90.0)
        try:
            initialize(client, source_path.parent)
            samples.append((time.perf_counter() - started) * 1000.0)
            client.request("shutdown", {})
        finally:
            client.close()
    return samples


def run_warm_scenario(
    scenario: str,
    source_path: Path,
    source: str,
    iterations: int,
    inner_repetitions: int,
) -> list[float]:
    client = LspClient(timeout=90.0)
    samples: list[float] = []
    try:
        initialize(client, source_path.parent)
        open_document(client, source_path, source)
        uri = file_uri(source_path)
        diagnostic_state = {"version": 1}
        for _ in range(iterations):
            started = time.perf_counter()
            for _ in range(inner_repetitions):
                if scenario == "lsp.diagnostics":
                    run_document_diagnostics(client, uri, source, diagnostic_state)
                else:
                    WARM_SCENARIOS[scenario](client, uri)
            samples.append((time.perf_counter() - started) * 1000.0 / inner_repetitions)
        client.request("shutdown", {})
    finally:
        client.close()
    return samples


def run_did_open_diagnostics(source: str, iterations: int, inner_repetitions: int) -> list[float]:
    samples: list[float] = []
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-did-open-bench-") as raw:
        root = Path(raw)
        client = LspClient(timeout=90.0)
        try:
            initialize(client, root)
            for index in range(iterations):
                started = time.perf_counter()
                for repetition in range(inner_repetitions):
                    path = root / f"main_{index}_{repetition}.sifr"
                    path.write_text(source, encoding="utf-8")
                    uri = file_uri(path)
                    client.notify(
                        "textDocument/didOpen",
                        {
                            "textDocument": {
                                "uri": uri,
                                "languageId": "sifr",
                                "version": 1,
                                "text": source,
                            }
                        },
                    )
                    client.wait_for_notification("textDocument/publishDiagnostics")
                samples.append((time.perf_counter() - started) * 1000.0 / inner_repetitions)
            client.request("shutdown", {})
        finally:
            client.close()
    return samples


@contextmanager
def benchmark_source(
    scenario: str, source_path: Path, source: str, workspace_mode: str
) -> Iterator[Path]:
    if workspace_mode == "package":
        if not source_path.parent.joinpath("sifr.toml").is_file():
            raise ValueError("package LSP benchmark source requires a sibling sifr.toml")
        yield source_path
        return
    if workspace_mode != "isolated":
        raise ValueError(f"unknown LSP benchmark workspace mode {workspace_mode!r}")
    if source_path.parent.joinpath("sifr.toml").is_file():
        raise ValueError("isolated LSP benchmark cannot use a package source")
    if scenario == "lsp.did_open_diagnostics":
        # This case deliberately measures document synchronization without a
        # package. run_did_open_diagnostics owns that temporary workspace, so
        # constructing a second locked package here would be dead setup.
        yield source_path
        return
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-isolated-bench-") as raw:
        root = Path(raw)
        source_root = root / "src"
        source_root.mkdir()
        isolated_path = source_root / source_path.name
        isolated_path.write_text(source, encoding="utf-8")
        source_root.joinpath("lib.rs").write_text(
            "// Pure Sifr package marker.\n", encoding="utf-8"
        )
        root.joinpath("Cargo.toml").write_text(
            "[package]\n"
            'name = "sifr-lsp-isolated-benchmark"\n'
            'version = "0.0.0"\n'
            'edition = "2024"\n\n'
            "[package.metadata.sifr]\n"
            'manifest = "sifr.toml"\n\n'
            "[workspace]\n",
            encoding="utf-8",
        )
        root.joinpath("Cargo.lock").write_text(
            "# This file is automatically @generated by Cargo.\n"
            "# It is not intended for manual editing.\n"
            "version = 4\n\n"
            "[[package]]\n"
            'name = "sifr-lsp-isolated-benchmark"\n'
            'version = "0.0.0"\n',
            encoding="utf-8",
        )
        root.joinpath("sifr.toml").write_text(
            "[package]\n"
            'name = "sifr-lsp-isolated-benchmark"\n'
            'edition = "2026"\n'
            'sifr-version = ">=0.3,<0.4"\n\n'
            "[source]\n"
            'root = "src"\n',
            encoding="utf-8",
        )
        yield isolated_path


def run_scenario(
    scenario: str,
    source_path: Path,
    source: str,
    iterations: int,
    inner_repetitions: int,
) -> list[float]:
    if scenario == "lsp.cold_start":
        return run_cold_start(source_path, iterations)
    if scenario == "lsp.did_open_diagnostics":
        return run_did_open_diagnostics(source, iterations, inner_repetitions)
    if scenario == "lsp.diagnostics" or scenario in WARM_SCENARIOS:
        return run_warm_scenario(
            scenario, source_path, source, iterations, inner_repetitions
        )
    raise ValueError(f"unknown LSP benchmark scenario {scenario!r}")


def main() -> int:
    if len(sys.argv) != 6:
        print(
            "usage: lsp_query_bench.py <scenario> <source-path> <iterations> "
            "<inner-repetitions> <workspace-mode>",
            file=sys.stderr,
        )
        return 2
    scenario = sys.argv[1]
    source_path = Path(sys.argv[2]).resolve()
    iterations = int(sys.argv[3])
    inner_repetitions = int(sys.argv[4])
    workspace_mode = sys.argv[5]
    if inner_repetitions <= 0:
        raise ValueError("inner-repetitions must be positive")
    source = source_path.read_text(encoding="utf-8")
    with benchmark_source(
        scenario, source_path, source, workspace_mode
    ) as benchmark_path:
        samples = run_scenario(
            scenario, benchmark_path, source, iterations, inner_repetitions
        )
    print(
        json.dumps(
            {
                "samples_ms": samples,
                "cache_hits": 0,
                "cache_misses": iterations,
                "diagnostics_count": 0,
                "timed_out": False,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
