#!/usr/bin/env python3
"""Smoke-test the native Sifr LSP stdio protocol."""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from typing import Any

from lsp_protocol import LspClient, LspProtocolError, assert_has_keys, file_uri


SAMPLE = """\
def helper(value: int) -> int:
    return value + 1

def main() -> int:
    result: int = helper(41)
    return result
"""

UNFORMATTED_SAMPLE = """\
def main()->int:
    value: int=1
    return value
"""

UNICODE_POSITION_SAMPLE = """\
def helper(value: int) -> int:
    return value + 1

def main() -> int:
    result: int = 0 if "🦀" else helper(41)
    return result
"""

UNICODE_DIAGNOSTIC_SAMPLE = """\
def main() -> int:
    value: int = "🦀" @
    return 0
"""

def initialize(
    client: LspClient,
    root: Path,
    initialization_options: dict[str, Any] | None = None,
    *,
    work_done_progress: bool = False,
) -> dict[str, Any]:
    capabilities: dict[str, Any] = {
        "textDocument": {
            "publishDiagnostics": {"relatedInformation": True},
            "semanticTokens": {"requests": {"full": True, "range": True}},
        },
        "general": {"positionEncodings": ["utf-16"]},
        "workspace": {"configuration": True, "workspaceFolders": True},
    }
    if work_done_progress:
        capabilities["window"] = {"workDoneProgress": True}
    result = client.request(
        "initialize",
        {
            "processId": None,
            "rootUri": file_uri(root),
            "capabilities": capabilities,
            "workspaceFolders": [{"uri": file_uri(root), "name": "sifr-lsp-smoke"}],
            "initializationOptions": initialization_options or {},
        },
    )
    if not isinstance(result, dict):
        raise LspProtocolError("initialize returned non-object result")
    capabilities = result.get("capabilities")
    if not isinstance(capabilities, dict):
        raise LspProtocolError("initialize response missing capabilities")
    if capabilities.get("positionEncoding") != "utf-16":
        raise LspProtocolError(
            f"initialize negotiated unexpected position encoding: {capabilities.get('positionEncoding')!r}"
        )
    required_capabilities = {
        "textDocumentSync",
        "diagnosticProvider",
        "completionProvider",
        "hoverProvider",
        "semanticTokensProvider",
        "executeCommandProvider",
    }
    if not initialization_options or initialization_options.get("formatEnable", True):
        required_capabilities.update({"documentFormattingProvider", "documentRangeFormattingProvider"})
    assert_has_keys(capabilities, required_capabilities, "initialize capabilities")
    client.notify("initialized", {})
    return capabilities


def open_document(client: LspClient, path: Path, text: str, version: int = 1) -> None:
    client.notify(
        "textDocument/didOpen",
        {
            "textDocument": {
                "uri": file_uri(path),
                "languageId": "sifr",
                "version": version,
                "text": text,
            }
        },
    )
    published = client.wait_for_notification("textDocument/publishDiagnostics")
    params = published.get("params", {})
    if params.get("uri") != file_uri(path):
        raise LspProtocolError("publishDiagnostics used the wrong document URI")
    if params.get("version") != version:
        raise LspProtocolError("publishDiagnostics did not preserve document version")
    if not isinstance(params.get("diagnostics"), list):
        raise LspProtocolError("publishDiagnostics missing diagnostics list")


def run_queries(client: LspClient, uri: str) -> None:
    document = {"uri": uri}
    helper_position = {"line": 4, "character": 19}
    result_position = {"line": 5, "character": 11}
    full_range = {"start": {"line": 0, "character": 0}, "end": {"line": 6, "character": 0}}

    checks = {
        "textDocument/documentSymbol": {"textDocument": document},
        "workspace/symbol": {"query": "helper"},
        "textDocument/hover": {"textDocument": document, "position": helper_position},
        "textDocument/signatureHelp": {"textDocument": document, "position": {"line": 4, "character": 24}},
        "textDocument/definition": {"textDocument": document, "position": helper_position},
        "textDocument/declaration": {"textDocument": document, "position": helper_position},
        "textDocument/typeDefinition": {"textDocument": document, "position": helper_position},
        "textDocument/references": {"textDocument": document, "position": result_position},
        "textDocument/prepareRename": {"textDocument": document, "position": result_position},
        "textDocument/documentHighlight": {"textDocument": document, "position": result_position},
        "textDocument/foldingRange": {"textDocument": document},
        "textDocument/selectionRange": {"textDocument": document, "positions": [result_position]},
        "textDocument/prepareTypeHierarchy": {"textDocument": document, "position": result_position},
        "textDocument/semanticTokens/full": {"textDocument": document},
        "textDocument/semanticTokens/range": {"textDocument": document, "range": full_range},
        "textDocument/inlayHint": {"textDocument": document, "range": full_range},
        "textDocument/codeAction": {
            "textDocument": document,
            "range": full_range,
            "context": {"diagnostics": [{"code": "SIFR-LINT-0001"}]},
        },
        "textDocument/formatting": {"textDocument": document, "options": {"tabSize": 4}},
        "textDocument/rangeFormatting": {"textDocument": document, "range": full_range, "options": {"tabSize": 4}},
        "textDocument/diagnostic": {"textDocument": document},
        "workspace/diagnostic": {},
    }
    for method, params in checks.items():
        client.request(method, params)

    rename = client.request(
        "textDocument/rename",
        {"textDocument": document, "position": result_position, "newName": "renamed_result"},
    )
    if not isinstance(rename, dict) or "changes" not in rename:
        raise LspProtocolError("rename did not return a workspace edit")

    preview = client.request(
        "workspace/executeCommand",
        {"command": "sifr.showGeneratedRust", "arguments": [uri]},
    )
    if not isinstance(preview, dict) or "rust" not in preview:
        raise LspProtocolError("generated Rust command did not return preview payload")

    client.request("workspace/executeCommand", {"command": "sifr.runTests", "arguments": []})


def run_formatting_checks(client: LspClient, path: Path) -> None:
    uri = file_uri(path)
    open_document(client, path, UNFORMATTED_SAMPLE)
    document = {"uri": uri}
    edits = client.request(
        "textDocument/formatting",
        {
            "textDocument": document,
            "options": {"tabSize": 4, "insertFinalNewline": True},
        },
    )
    if not isinstance(edits, list) or not edits:
        raise LspProtocolError("document formatting did not return formatter edits")
    replacement = edits[0].get("newText", "")
    if "def main() -> int:" not in replacement or "value: int = 1" not in replacement:
        raise LspProtocolError(f"document formatting did not match sifr fmt output: {edits}")

    range_edits = client.request(
        "textDocument/rangeFormatting",
        {
            "textDocument": document,
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 3, "character": 0}},
            "options": {"tabSize": 4, "lineLength": 88},
        },
    )
    if not isinstance(range_edits, list) or not range_edits:
        raise LspProtocolError("range formatting did not return formatter edits")


def run_disabled_formatting_check(root: Path) -> None:
    source = root / "disabled.sifr"
    source.write_text(UNFORMATTED_SAMPLE, encoding="utf-8")
    client = LspClient()
    try:
        capabilities = initialize(client, root, {"formatEnable": False})
        if "documentFormattingProvider" in capabilities or "documentRangeFormattingProvider" in capabilities:
            raise LspProtocolError("disabled formatting was still advertised")
        open_document(client, source, UNFORMATTED_SAMPLE)
        error = client.request_error(
            "textDocument/formatting",
            {"textDocument": {"uri": file_uri(source)}, "options": {"tabSize": 4}},
        )
        if error.get("code") != -32601:
            raise LspProtocolError(f"disabled formatting returned unexpected error: {error}")
        client.request("shutdown", {})
        client.notify("exit", {})
    finally:
        client.close()


def run_utf8_negotiation_check(root: Path) -> None:
    client = LspClient()
    try:
        capabilities = initialize(
            client,
            root,
            work_done_progress=False,
        )
        if capabilities.get("positionEncoding") != "utf-16":
            raise LspProtocolError("default smoke initialization should negotiate utf-16")
        client.request("shutdown", {})
        client.notify("exit", {})
    finally:
        client.close()

    client = LspClient()
    try:
        result = client.request(
            "initialize",
            {
                "processId": None,
                "rootUri": file_uri(root),
                "capabilities": {
                    "general": {"positionEncodings": ["utf-16", "utf-8"]},
                    "workspace": {"configuration": True, "workspaceFolders": True},
                },
                "workspaceFolders": [{"uri": file_uri(root), "name": "sifr-lsp-smoke"}],
                "initializationOptions": {},
            },
        )
        capabilities = result.get("capabilities") if isinstance(result, dict) else None
        if not isinstance(capabilities, dict):
            raise LspProtocolError("utf-8 negotiation initialize missing capabilities")
        if capabilities.get("positionEncoding") != "utf-8":
            raise LspProtocolError(
                f"utf-8 capable client negotiated {capabilities.get('positionEncoding')!r}"
            )
        client.notify("initialized", {})
        client.request("shutdown", {})
        client.notify("exit", {})
    finally:
        client.close()


def run_utf16_position_behavior_check(root: Path) -> None:
    source = root / "unicode-position.sifr"
    source.write_text(UNICODE_POSITION_SAMPLE, encoding="utf-8")
    client = LspClient()
    try:
        initialize(client, root)
        open_document(client, source, UNICODE_POSITION_SAMPLE)
        hover = client.request(
            "textDocument/hover",
            {
                "textDocument": {"uri": file_uri(source)},
                "position": {"line": 4, "character": 33},
            },
        )
        contents = hover.get("contents", {}) if isinstance(hover, dict) else {}
        if "helper" not in contents.get("value", ""):
            raise LspProtocolError(f"utf-16 hover position did not resolve helper: {hover}")
        client.request("shutdown", {})
        client.notify("exit", {})
    finally:
        client.close()


def run_utf16_diagnostic_range_check(root: Path) -> None:
    source = root / "unicode-diagnostic.sifr"
    source.write_text(UNICODE_DIAGNOSTIC_SAMPLE, encoding="utf-8")
    client = LspClient()
    try:
        initialize(client, root)
        open_document(client, source, UNICODE_DIAGNOSTIC_SAMPLE)
        report = client.request("textDocument/diagnostic", {"textDocument": {"uri": file_uri(source)}})
        items = report.get("items", []) if isinstance(report, dict) else []
        starts = [
            item.get("range", {}).get("start")
            for item in items
            if isinstance(item, dict) and item.get("range", {}).get("start", {}).get("line") == 1
        ]
        if {"line": 1, "character": 23} not in starts:
            raise LspProtocolError(f"utf-16 diagnostic range did not land on invalid token: {items}")
        client.request("shutdown", {})
        client.notify("exit", {})
    finally:
        client.close()


def run_smoke() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-smoke-") as raw:
        root = Path(raw)
        source = root / "main.sifr"
        formatting_source = root / "formatting.sifr"
        source.write_text(SAMPLE, encoding="utf-8")
        formatting_source.write_text(UNFORMATTED_SAMPLE, encoding="utf-8")
        client = LspClient()
        try:
            initialize(client, root)
            open_document(client, source, SAMPLE)
            run_queries(client, file_uri(source))
            run_formatting_checks(client, formatting_source)
            client.request("shutdown", {})
            client.notify("exit", {})
        finally:
            client.close()
        run_disabled_formatting_check(root)
        run_utf8_negotiation_check(root)
        run_utf16_position_behavior_check(root)
        run_utf16_diagnostic_range_check(root)


def run_self_test() -> None:
    try:
        assert_has_keys({"capabilities": {}}, {"missing"}, "negative smoke")
    except LspProtocolError:
        print("LSP protocol smoke self-test: PASS")
        return
    raise SystemExit("LSP protocol smoke self-test failed: malformed capabilities passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0
    try:
        run_smoke()
    except (LspProtocolError, OSError, json.JSONDecodeError) as error:
        print(f"LSP protocol smoke: FAIL: {error}", file=sys.stderr)
        return 1
    print("LSP protocol smoke: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
