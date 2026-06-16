#!/usr/bin/env python3
"""Replay manifest-owned Sifr LSP JSON-RPC transcript scenarios."""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import Any

from lsp_protocol import LspClient, LspProtocolError, file_uri
from lsp_protocol_smoke import SAMPLE, initialize, open_document


AREA_ROOT = Path(__file__).resolve().parent
MANIFEST_PATH = AREA_ROOT / "lsp_transcripts" / "manifest.json"

BROKEN_SAMPLE = """\
def helper(value: int) -> int:
    return value + 1

def main() -> int:
    result: int = missing_value
    return result
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            run_self_test()
        else:
            manifest = load_manifest(MANIFEST_PATH)
            validate_manifest(manifest, SCENARIO_RUNNERS)
            run_manifest(manifest)
    except (LspProtocolError, OSError, json.JSONDecodeError) as error:
        print(f"LSP transcript replay: FAIL: {error}", file=sys.stderr)
        return 1
    print("LSP transcript replay: PASS")
    return 0


def load_manifest(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise LspProtocolError("transcript manifest must be a JSON object")
    return payload


def validate_manifest(manifest: dict[str, Any], runners: dict[str, Callable[[dict[str, Any]], None]]) -> None:
    if manifest.get("schema_version") != 1:
        raise LspProtocolError("transcript manifest schema_version must be 1")
    scenarios = manifest.get("scenarios")
    if not isinstance(scenarios, list) or not scenarios:
        raise LspProtocolError("transcript manifest must declare at least one scenario")
    scenario_ids = [require_string(scenario, "id") for scenario in scenarios]
    if scenario_ids != sorted(scenario_ids):
        raise LspProtocolError("transcript scenarios must be sorted by id")
    duplicates = sorted({scenario_id for scenario_id in scenario_ids if scenario_ids.count(scenario_id) > 1})
    if duplicates:
        raise LspProtocolError(f"duplicate transcript scenario ids: {duplicates}")
    missing_runners = sorted(set(scenario_ids).difference(runners))
    if missing_runners:
        raise LspProtocolError(f"transcript scenarios without replay runners: {missing_runners}")

    required_categories = manifest.get("required_categories")
    if not isinstance(required_categories, list) or not required_categories or required_categories != sorted(required_categories):
        raise LspProtocolError("required_categories must be a sorted non-empty list")
    covered_categories: set[str] = set()
    for scenario in scenarios:
        category = require_string(scenario, "category")
        covered_categories.add(category)
        for extra in scenario.get("additional_categories", []):
            if not isinstance(extra, str) or not extra:
                raise LspProtocolError(f"{scenario['id']} has invalid additional category")
            covered_categories.add(extra)
        required_methods = scenario.get("required_methods")
        if not isinstance(required_methods, list) or not required_methods:
            raise LspProtocolError(f"{scenario['id']} must declare required_methods")
        if required_methods != sorted(required_methods):
            raise LspProtocolError(f"{scenario['id']} required_methods must be sorted")
        assertions = scenario.get("assertions")
        if not isinstance(assertions, list) or not assertions:
            raise LspProtocolError(f"{scenario['id']} must declare assertions")
        if scenario.get("profile") != "create-pr":
            raise LspProtocolError(f"{scenario['id']} must be owned by the create-pr profile")
    missing_categories = sorted(set(required_categories).difference(covered_categories))
    if missing_categories:
        raise LspProtocolError(f"transcript manifest does not cover categories: {missing_categories}")


def require_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise LspProtocolError(f"manifest entry missing string field {key}")
    return value


def run_manifest(manifest: dict[str, Any]) -> None:
    for scenario in manifest["scenarios"]:
        scenario_id = str(scenario["id"])
        SCENARIO_RUNNERS[scenario_id](scenario)


def replay_initialize_capability_snapshot(_: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-transcript-init-") as raw:
        root = Path(raw)
        client = LspClient()
        try:
            capabilities = initialize(client, root)
            workspace = require_dict(capabilities.get("workspace"), "initialize workspace capabilities")
            workspace_folders = require_dict(workspace.get("workspaceFolders"), "workspaceFolders capability")
            if workspace_folders.get("supported") is not True:
                raise LspProtocolError("workspace folders are not advertised as supported")
            commands = require_dict(capabilities.get("executeCommandProvider"), "executeCommandProvider").get("commands")
            expected_commands = {
                "sifr.restartServer",
                "sifr.showServerLogs",
                "sifr.explainDiagnostic",
                "sifr.showGeneratedRust",
                "sifr.checkWorkspace",
                "sifr.runTests",
            }
            if not isinstance(commands, list) or expected_commands.difference(commands):
                raise LspProtocolError(f"initialize command advertisement drifted: {commands}")
            legend = require_dict(
                require_dict(capabilities.get("semanticTokensProvider"), "semanticTokensProvider").get("legend"),
                "semantic token legend",
            )
            token_types = legend.get("tokenTypes")
            if not isinstance(token_types, list) or "ownershipSensitive" not in token_types:
                raise LspProtocolError(f"semantic token legend lost ownership-sensitive token: {legend}")
            shutdown(client)
        finally:
            close_client(client)


def replay_client_capability_combinations(_: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-transcript-client-caps-") as raw:
        root = Path(raw)
        source = root / "main.sifr"
        source.write_text(SAMPLE, encoding="utf-8")
        client = LspClient()
        try:
            capabilities = initialize(client, root, {"formatEnable": False}, work_done_progress=True)
            if "documentFormattingProvider" in capabilities or "documentRangeFormattingProvider" in capabilities:
                raise LspProtocolError("formatting capabilities were advertised while formatEnable=false")
            open_document(client, source, SAMPLE)
            error = client.request_error(
                "textDocument/formatting",
                {"textDocument": {"uri": file_uri(source)}, "options": {"tabSize": 4}},
            )
            if error.get("code") != -32601:
                raise LspProtocolError(f"disabled formatting returned unexpected error: {error}")
            shutdown(client)
        finally:
            close_client(client)


def replay_unsupported_capability_behavior(_: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-transcript-unsupported-") as raw:
        root = Path(raw)
        source = root / "main.sifr"
        source.write_text(SAMPLE, encoding="utf-8")
        client = LspClient()
        try:
            initialize(client, root)
            open_document(client, source, SAMPLE)
            error = client.request_error(
                "textDocument/implementation",
                {"textDocument": {"uri": file_uri(source)}, "position": {"line": 4, "character": 19}},
            )
            if error.get("code") != -32601:
                raise LspProtocolError(f"unsupported implementation request returned unexpected error: {error}")
            shutdown(client)
        finally:
            close_client(client)


def replay_cancellation_and_out_of_order_requests(_: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-transcript-order-") as raw:
        root = Path(raw)
        source = root / "main.sifr"
        source.write_text(SAMPLE, encoding="utf-8")
        client = LspClient()
        try:
            initialize(client, root)
            open_document(client, source, SAMPLE)
            client.send_request(100, "textDocument/documentSymbol", {"textDocument": {"uri": file_uri(source)}})
            client.send_request(101, "workspace/symbol", {"query": "helper"})
            client.notify("$/cancelRequest", {"id": 999_999})
            workspace_symbols = client.wait_for_response(101)
            if 100 not in client.responses:
                raise LspProtocolError("out-of-order replay did not exercise response buffering")
            document_symbols = client.wait_for_response(100)
            if "error" in document_symbols or "error" in workspace_symbols:
                raise LspProtocolError(
                    f"out-of-order transcript returned errors: {document_symbols}, {workspace_symbols}"
                )
            if not isinstance(document_symbols.get("result"), list):
                raise LspProtocolError(f"documentSymbol response was not retained correctly: {document_symbols}")
            if not isinstance(workspace_symbols.get("result"), list):
                raise LspProtocolError(f"workspace/symbol response was not retained correctly: {workspace_symbols}")
            hover = client.request(
                "textDocument/hover",
                {"textDocument": {"uri": file_uri(source)}, "position": {"line": 4, "character": 19}},
            )
            if hover is None:
                raise LspProtocolError("hover failed after cancellation notification")
            shutdown(client)
        finally:
            close_client(client)


def replay_stale_diagnostics_after_edit(_: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-transcript-stale-") as raw:
        root = Path(raw)
        source = root / "main.sifr"
        source.write_text(BROKEN_SAMPLE, encoding="utf-8")
        uri = file_uri(source)
        client = LspClient()
        try:
            initialize(client, root)
            diagnostics = open_and_wait_diagnostics(client, source, BROKEN_SAMPLE, 1)
            if not diagnostics:
                raise LspProtocolError("broken document did not publish diagnostics")
            client.notify(
                "textDocument/didChange",
                {"textDocument": {"uri": uri, "version": 2}, "contentChanges": [{"text": SAMPLE}]},
            )
            latest = wait_for_diagnostics(client, uri, 2)
            if latest:
                raise LspProtocolError(f"fixed document kept stale diagnostics: {latest}")
            client.notify(
                "textDocument/didChange",
                {"textDocument": {"uri": uri, "version": 1}, "contentChanges": [{"text": BROKEN_SAMPLE}]},
            )
            client.request(
                "textDocument/hover",
                {"textDocument": {"uri": uri}, "position": {"line": 4, "character": 19}},
            )
            shutdown(client)
        finally:
            close_client(client)


def replay_project_reload_watched_file(_: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-transcript-reload-") as raw:
        root = Path(raw)
        (root / "sifr.toml").write_text('[package]\nname = "lsp_reload"\n', encoding="utf-8")
        main = root / "main.sifr"
        helper = root / "helper.sifr"
        main.write_text("from helper import helper_value\n\ndef main() -> int:\n    return helper_value()\n", encoding="utf-8")
        helper.write_text("def helper_value() -> int:\n    return 1\n", encoding="utf-8")
        client = LspClient()
        try:
            initialize(client, root, work_done_progress=True)
            open_document(client, main, main.read_text(encoding="utf-8"))
            symbols = client.request("workspace/symbol", {"query": "helper_value"})
            if not symbol_locations_include(symbols, file_uri(helper)):
                raise LspProtocolError(f"workspace symbols missed helper before reload: {symbols}")
            helper.write_text("def helper_value() -> int:\n    return 2\n", encoding="utf-8")
            client.notify("workspace/didChangeWatchedFiles", {"changes": [{"uri": file_uri(helper), "type": 2}]})
            changed = client.wait_for_notification("textDocument/publishDiagnostics")
            changed_uri = changed.get("params", {}).get("uri")
            if changed_uri not in {file_uri(main), file_uri(helper)}:
                raise LspProtocolError(f"watched-file reload published diagnostics for wrong URI: {changed}")
            diagnostics = changed.get("params", {}).get("diagnostics")
            if not isinstance(diagnostics, list):
                raise LspProtocolError(f"watched-file reload did not publish diagnostics: {changed}")
            reloaded = client.request("workspace/symbol", {"query": "helper_value"})
            if not symbol_locations_include(reloaded, file_uri(helper)):
                raise LspProtocolError(f"watched-file reload lost workspace symbol identity: {reloaded}")
            shutdown(client)
        finally:
            close_client(client)


def require_dict(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise LspProtocolError(f"{label} must be an object")
    return value


def open_and_wait_diagnostics(client: LspClient, path: Path, text: str, version: int) -> list[dict[str, Any]]:
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
    return wait_for_diagnostics(client, file_uri(path), version)


def wait_for_diagnostics(client: LspClient, uri: str, version: int) -> list[dict[str, Any]]:
    for _ in range(8):
        published = client.wait_for_notification("textDocument/publishDiagnostics")
        params = published.get("params", {})
        if params.get("uri") == uri and params.get("version") == version:
            diagnostics = params.get("diagnostics")
            if not isinstance(diagnostics, list):
                raise LspProtocolError(f"diagnostics payload missing list: {published}")
            return diagnostics
    raise LspProtocolError(f"timed out waiting for diagnostics uri={uri} version={version}")


def symbol_locations_include(symbols: Any, uri: str) -> bool:
    if not isinstance(symbols, list):
        return False
    return any(isinstance(item, dict) and item.get("location", {}).get("uri") == uri for item in symbols)


def shutdown(client: LspClient) -> None:
    client.request("shutdown", {})
    client.notify("exit", {})


def close_client(client: LspClient) -> None:
    if sys.exc_info()[0] is None:
        client.close()
    elif client.process.poll() is None:
        client.process.kill()
        client.process.wait(timeout=10)


def run_self_test() -> None:
    try:
        validate_manifest(
            {
                "schema_version": 1,
                "required_categories": ["initialize"],
                "scenarios": [
                    {
                        "id": "missing-runner",
                        "category": "initialize",
                        "profile": "create-pr",
                        "required_methods": ["initialize"],
                        "assertions": ["must fail"],
                    }
                ],
            },
            {},
        )
    except LspProtocolError:
        print("LSP transcript replay self-test: PASS")
        return
    raise LspProtocolError("self-test failed: missing replay runner was accepted")


SCENARIO_RUNNERS: dict[str, Callable[[dict[str, Any]], None]] = {
    "cancellation-and-out-of-order-requests": replay_cancellation_and_out_of_order_requests,
    "client-capability-combinations": replay_client_capability_combinations,
    "initialize-capability-snapshot": replay_initialize_capability_snapshot,
    "project-reload-watched-file": replay_project_reload_watched_file,
    "stale-diagnostics-after-edit": replay_stale_diagnostics_after_edit,
    "unsupported-capability-behavior": replay_unsupported_capability_behavior,
}


if __name__ == "__main__":
    sys.exit(main())
