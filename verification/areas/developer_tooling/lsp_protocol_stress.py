#!/usr/bin/env python3
"""Stress-test deterministic Sifr LSP protocol error and sync behavior."""

from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path

from lsp_protocol import LspClient, LspProtocolError, file_uri
from lsp_protocol_smoke import SAMPLE, initialize, open_document


UPDATED = SAMPLE.replace("return result\n", "return result + 1\n")


def run_stress() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-stress-") as raw:
        root = Path(raw)
        source = root / "main.sifr"
        source.write_text(SAMPLE, encoding="utf-8")
        uri = file_uri(source)
        client = LspClient(extra_args=["--parent-pid", str(os.getpid())])
        try:
            initialize(client, root, work_done_progress=True)
            open_document(client, source, SAMPLE)
            secondary = root / "secondary.sifr"
            secondary.write_text(SAMPLE, encoding="utf-8")
            open_document(client, secondary, SAMPLE)

            client.notify("workspace/didChangeWatchedFiles", {"changes": [{"uri": uri, "type": 2}]})
            progress = client.wait_for_notification("$/progress")
            if progress.get("params", {}).get("value", {}).get("kind") != "begin":
                raise LspProtocolError("workspace diagnostics progress did not begin")
            for _ in range(2):
                client.wait_for_notification("textDocument/publishDiagnostics")
            trace = client.request("sifr/debugTrace", {})
            if not isinstance(trace, str) or "progress_start" not in trace:
                raise LspProtocolError("debug trace did not record workspace diagnostics progress")
            progress = client.wait_for_notification("$/progress")
            if progress.get("params", {}).get("value", {}).get("kind") != "end":
                raise LspProtocolError("workspace diagnostics progress did not end")

            client.notify("$/cancelRequest", {"id": 99999})
            client.notify(
                "textDocument/didChange",
                {
                    "textDocument": {"uri": uri, "version": 2},
                    "contentChanges": [{"text": UPDATED}],
                },
            )
            diagnostics = client.wait_for_notification("textDocument/publishDiagnostics")
            if diagnostics.get("params", {}).get("version") != 2:
                raise LspProtocolError("didChange diagnostics did not carry latest version")

            client.notify(
                "textDocument/didChange",
                {
                    "textDocument": {"uri": uri, "version": 1},
                    "contentChanges": [{"text": SAMPLE}],
                },
            )
            hover = client.request(
                "textDocument/hover",
                {"textDocument": {"uri": uri}, "position": {"line": 4, "character": 19}},
            )
            if hover is None:
                raise LspProtocolError("hover failed after stale notification rejection")

            error = client.request_error("workspace/executeCommand", {"command": "sifr.unknown", "arguments": []})
            if error.get("code") not in {-32601, -32602, -32001}:
                raise LspProtocolError(f"unknown command returned unexpected error code: {error}")

            error = client.request_error(
                "textDocument/semanticTokens/range",
                {
                    "textDocument": {"uri": uri},
                    "range": {
                        "start": {"line": 999, "character": 0},
                        "end": {"line": 999, "character": 1},
                    },
                },
            )
            if error.get("code") != -32602:
                raise LspProtocolError(f"invalid range returned unexpected error: {error}")

            error = client.request_error(
                "textDocument/rangeFormatting",
                {
                    "textDocument": {"uri": uri},
                    "range": {
                        "start": {"line": 999, "character": 0},
                        "end": {"line": 999, "character": 1},
                    },
                    "options": {"tabSize": 4},
                },
            )
            if error.get("code") != -32602:
                raise LspProtocolError(f"invalid formatter range returned unexpected error: {error}")

            client.notify("workspace/didChangeConfiguration", {"settings": {"sifr.format.enable": False}})
            error = client.request_error(
                "textDocument/formatting",
                {"textDocument": {"uri": uri}, "options": {"tabSize": 4}},
            )
            if error.get("code") != -32601:
                raise LspProtocolError(f"disabled formatter returned unexpected error: {error}")
            client.notify("workspace/didChangeConfiguration", {"settings": {"sifr.format.enable": True}})
            edits = client.request(
                "textDocument/formatting",
                {"textDocument": {"uri": uri}, "options": {"tabSize": 4}},
            )
            if not isinstance(edits, list):
                raise LspProtocolError("re-enabled formatter did not return an edit list")

            client.notify("workspace/didChangeConfiguration", {"settings": {"sifr.diagnostics.mode": "off"}})
            client.notify("workspace/didChangeWatchedFiles", {"changes": [{"uri": uri, "type": 2}]})
            client.notify("textDocument/didSave", {"textDocument": {"uri": uri}, "text": UPDATED})
            client.notify("textDocument/didClose", {"textDocument": {"uri": uri}})

            error = client.request_error("textDocument/documentSymbol", {"textDocument": {"uri": uri}})
            if error.get("code") != -32602:
                raise LspProtocolError(f"closed document query returned unexpected error: {error}")

            client.request("shutdown", {})
            error = client.request_error(
                "textDocument/hover",
                {"textDocument": {"uri": uri}, "position": {"line": 4, "character": 19}},
            )
            if error.get("code") != -32800:
                raise LspProtocolError(f"post-shutdown request returned unexpected error: {error}")
            client.notify("exit", {})
        finally:
            client.close()
    run_project_cross_file_queries()
    run_project_save_reuses_project_owner()
    run_multi_project_workspace_symbols()


def run_project_cross_file_queries() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-project-") as raw:
        root = Path(raw)
        (root / "sifr.toml").write_text('[package]\nname = "lsp_project"\n', encoding="utf-8")
        main = root / "main.sifr"
        helper = root / "utils.sifr"
        main_text = "from utils import helper\n\ndef main() -> int:\n    return helper(41)\n"
        helper_text = "def helper(value: int) -> int:\n    return value + 1\n"
        main.write_text(main_text, encoding="utf-8")
        helper.write_text(helper_text, encoding="utf-8")
        client = LspClient()
        try:
            initialize(client, root)
            open_document(client, main, main_text)
            symbols = client.request("workspace/symbol", {"query": "helper"})
            helper_symbols = [
                item for item in symbols
                if item.get("name") == "helper"
                and item.get("location", {}).get("uri") == file_uri(helper)
            ]
            if len(helper_symbols) != 1:
                raise LspProtocolError(f"project workspace/symbol did not return one helper definition: {symbols}")
            references = client.request(
                "textDocument/references",
                {"textDocument": {"uri": file_uri(main)}, "position": {"line": 3, "character": 14}},
            )
            reference_uris = {item.get("uri") for item in references}
            if file_uri(main) not in reference_uris or file_uri(helper) not in reference_uris:
                raise LspProtocolError(f"project references did not cross files: {references}")
            rename = client.request(
                "textDocument/rename",
                {
                    "textDocument": {"uri": file_uri(main)},
                    "position": {"line": 3, "character": 14},
                    "newName": "renamed_helper",
                },
            )
            changes = rename.get("changes", {}) if isinstance(rename, dict) else {}
            if file_uri(main) not in changes or file_uri(helper) not in changes:
                raise LspProtocolError(f"project rename did not edit both files: {rename}")
            client.request("shutdown", {})
            client.notify("exit", {})
        finally:
            client.close()


def run_project_save_reuses_project_owner() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-project-save-") as raw:
        root = Path(raw)
        (root / "sifr.toml").write_text(
            '[package]\nname = "lsp_project_save"\n[source]\nroot = "."\n',
            encoding="utf-8",
        )
        main = root / "main.sifr"
        initial = (
            "from sifr.random import randint\n\n"
            "def main() -> int:\n"
            "    value: int = randint(0, 100)\n"
            "    if value > 50:\n"
            "        return value\n"
            "    return 0\n"
        )
        saved = (
            "def helper(value: int) -> int:\n"
            "    return value + 1\n\n"
            "def main() -> int:\n"
            "    result: int = helper(41)\n"
            "    return result\n"
        )
        shortened = (
            "from sifr.random import randint\n\n"
            "def main():\n"
            "    value = randint(0, 100)\n"
            "    print(value)\n"
        )
        main.write_text(initial, encoding="utf-8")
        uri = file_uri(main)
        client = LspClient()
        try:
            initialize(client, root)
            open_document(client, main, initial)
            client.notify(
                "textDocument/didChange",
                {
                    "textDocument": {"uri": uri, "version": 2},
                    "contentChanges": [{"text": saved}],
                },
            )
            client.wait_for_notification("textDocument/publishDiagnostics")
            client.notify("textDocument/didSave", {"textDocument": {"uri": uri}, "text": saved})
            client.wait_for_notification("textDocument/publishDiagnostics")
            hover = client.request(
                "textDocument/hover",
                {"textDocument": {"uri": uri}, "position": {"line": 4, "character": 18}},
            )
            hover_value = hover.get("contents", {}).get("value", "") if isinstance(hover, dict) else ""
            if "helper" not in hover_value or "-> int" not in hover_value or "(Name)" in hover_value:
                raise LspProtocolError(f"project hover after save was not semantic: {hover}")
            signature = client.request(
                "textDocument/signatureHelp",
                {"textDocument": {"uri": uri}, "position": {"line": 4, "character": 25}},
            )
            signatures = signature.get("signatures", []) if isinstance(signature, dict) else []
            parameters = signatures[0].get("parameters", []) if signatures else []
            if (
                not signatures
                or "helper" not in signatures[0].get("label", "")
                or [item.get("label") for item in parameters] != ["value: int"]
            ):
                raise LspProtocolError(f"project signatureHelp after save was not semantic: {signature}")
            client.notify(
                "textDocument/didChange",
                {
                    "textDocument": {"uri": uri, "version": 3},
                    "contentChanges": [{"text": shortened}],
                },
            )
            client.wait_for_notification("textDocument/publishDiagnostics")
            folding = client.request("textDocument/foldingRange", {"textDocument": {"uri": uri}})
            if not isinstance(folding, list):
                raise LspProtocolError(f"project foldingRange after save returned non-list: {folding}")
            assert_ranges_within_source(folding, shortened, "project foldingRange after save")
            semantic = client.request("textDocument/semanticTokens/full", {"textDocument": {"uri": uri}})
            if not isinstance(semantic, dict) or "data" not in semantic:
                raise LspProtocolError(f"project semantic tokens after save returned invalid payload: {semantic}")
            assert_semantic_tokens_within_source(semantic["data"], shortened, "project semantic tokens after save")
            symbols = client.request("textDocument/documentSymbol", {"textDocument": {"uri": uri}})
            if not isinstance(symbols, list):
                raise LspProtocolError(f"project documentSymbol after save returned non-list: {symbols}")
            assert_document_symbols_within_source(symbols, shortened, "project documentSymbol after save")
            client.request("shutdown", {})
            client.notify("exit", {})
        finally:
            client.close()


def line_lengths(source: str) -> list[int]:
    lines = source.splitlines()
    if source.endswith("\n"):
        return [len(line) for line in lines] + [0]
    return [len(line) for line in lines] or [0]


def assert_ranges_within_source(ranges: list[dict], source: str, label: str) -> None:
    lengths = line_lengths(source)
    for item in ranges:
        start_line = item.get("startLine")
        end_line = item.get("endLine")
        start_character = item.get("startCharacter", 0)
        end_character = item.get("endCharacter", lengths[end_line] if isinstance(end_line, int) and end_line < len(lengths) else 0)
        if not isinstance(start_line, int) or not isinstance(end_line, int):
            raise LspProtocolError(f"{label} returned non-integer range lines: {item}")
        if start_line < 0 or end_line < start_line or end_line >= len(lengths):
            raise LspProtocolError(f"{label} returned range outside source lines: {item}")
        if not isinstance(start_character, int) or not isinstance(end_character, int):
            raise LspProtocolError(f"{label} returned non-integer range characters: {item}")
        if start_character < 0 or start_character > lengths[start_line]:
            raise LspProtocolError(f"{label} returned start outside source line: {item}")
        if end_character < 0 or end_character > lengths[end_line]:
            raise LspProtocolError(f"{label} returned end outside source line: {item}")


def assert_lsp_range_within_source(item: dict, source: str, label: str) -> None:
    lengths = line_lengths(source)
    start = item.get("start")
    end = item.get("end")
    if not isinstance(start, dict) or not isinstance(end, dict):
        raise LspProtocolError(f"{label} returned invalid LSP range: {item}")
    start_line = start.get("line")
    start_character = start.get("character")
    end_line = end.get("line")
    end_character = end.get("character")
    if not all(isinstance(value, int) for value in [start_line, start_character, end_line, end_character]):
        raise LspProtocolError(f"{label} returned non-integer LSP range: {item}")
    if start_line < 0 or end_line < start_line or end_line >= len(lengths):
        raise LspProtocolError(f"{label} returned range outside source lines: {item}")
    if start_character < 0 or start_character > lengths[start_line]:
        raise LspProtocolError(f"{label} returned start outside source line: {item}")
    if end_character < 0 or end_character > lengths[end_line]:
        raise LspProtocolError(f"{label} returned end outside source line: {item}")


def assert_document_symbols_within_source(symbols: list[dict], source: str, label: str) -> None:
    for symbol in symbols:
        symbol_range = symbol.get("range")
        if isinstance(symbol_range, dict):
            assert_lsp_range_within_source(symbol_range, source, label)
        selection_range = symbol.get("selectionRange")
        if isinstance(selection_range, dict):
            assert_lsp_range_within_source(selection_range, source, label)
        children = symbol.get("children")
        if isinstance(children, list):
            assert_document_symbols_within_source(children, source, label)


def assert_semantic_tokens_within_source(data: object, source: str, label: str) -> None:
    if not isinstance(data, list) or len(data) % 5 != 0:
        raise LspProtocolError(f"{label} returned invalid semantic token data: {data}")
    lengths = line_lengths(source)
    line = 0
    start = 0
    for index in range(0, len(data), 5):
        delta_line, delta_start, length, _token_type, _modifiers = data[index:index + 5]
        if not all(isinstance(value, int) for value in [delta_line, delta_start, length]):
            raise LspProtocolError(f"{label} returned non-integer semantic token tuple: {data[index:index + 5]}")
        if delta_line < 0 or delta_start < 0 or length < 0:
            raise LspProtocolError(f"{label} returned negative semantic token tuple: {data[index:index + 5]}")
        line += delta_line
        start = start + delta_start if delta_line == 0 else delta_start
        if line >= len(lengths):
            raise LspProtocolError(f"{label} returned token outside source lines: {data[index:index + 5]}")
        if start > lengths[line] or start + length > lengths[line]:
            raise LspProtocolError(f"{label} returned token outside source line: {data[index:index + 5]}")


def run_multi_project_workspace_symbols() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-multiproject-") as raw:
        root = Path(raw)
        alpha_root = root / "alpha"
        beta_root = root / "beta"
        alpha_root.mkdir()
        beta_root.mkdir()
        for project_root, package, function_name in [
            (alpha_root, "alpha", "alpha_entry"),
            (beta_root, "beta", "beta_entry"),
        ]:
            (project_root / "sifr.toml").write_text(
                f'[package]\nname = "{package}"\n',
                encoding="utf-8",
            )
            (project_root / "main.sifr").write_text(
                f"def {function_name}() -> int:\n    return 1\n",
                encoding="utf-8",
            )
        client = LspClient()
        try:
            initialize(client, root)
            open_document(client, alpha_root / "main.sifr", (alpha_root / "main.sifr").read_text(encoding="utf-8"))
            open_document(client, beta_root / "main.sifr", (beta_root / "main.sifr").read_text(encoding="utf-8"))
            symbols = client.request("workspace/symbol", {"query": "_entry"})
            actual = {
                (item.get("name"), item.get("location", {}).get("uri"))
                for item in symbols
            }
            expected = {
                ("alpha_entry", file_uri(alpha_root / "main.sifr")),
                ("beta_entry", file_uri(beta_root / "main.sifr")),
            }
            if expected - actual:
                raise LspProtocolError(f"multi-project symbols lost URI identity: {symbols}")
            client.request("shutdown", {})
            client.notify("exit", {})
        finally:
            client.close()


def run_self_test() -> None:
    try:
        raise LspProtocolError("seeded stress failure")
    except LspProtocolError:
        print("LSP protocol stress self-test: PASS")
        return
    raise SystemExit("LSP protocol stress self-test failed: seeded failure passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0
    try:
        run_stress()
    except (LspProtocolError, OSError, json.JSONDecodeError) as error:
        print(f"LSP protocol stress: FAIL: {error}", file=sys.stderr)
        return 1
    print("LSP protocol stress: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
