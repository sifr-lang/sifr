#!/usr/bin/env python3
"""Assert measured LSP cache transitions in a real 25-module workspace."""

from __future__ import annotations

import json
import shutil
import tempfile
import time
from pathlib import Path

from lsp_query_bench import cache_delta, cache_stats, request_context, validate_benchmark_input

import sys

TOOLING_ROOT = Path(__file__).resolve().parents[1] / "developer_tooling"
sys.path.insert(0, str(TOOLING_ROOT))
from lsp_protocol import LspClient, file_uri  # noqa: E402
from lsp_protocol_smoke import initialize  # noqa: E402

FIXTURE_ROOT = Path(__file__).resolve().parent / "query_projects" / "lsp_workspace"


def measured(client: LspClient, action) -> dict[str, int | float]:
    before = cache_stats(client)
    started = time.perf_counter()
    action()
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    hits, misses = cache_delta(before, cache_stats(client))
    return {"hits": hits, "misses": misses, "elapsed_ms": round(elapsed_ms, 3)}


def expect(label: str, delta: dict[str, int | float], hits: int, misses: int) -> None:
    if delta["hits"] != hits or delta["misses"] != misses:
        raise AssertionError(f"{label}: expected hits={hits} misses={misses}, got {delta}")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-e03-") as raw:
        root = Path(raw) / "lsp_workspace"
        shutil.copytree(FIXTURE_ROOT, root)
        source_path = root / "src" / "main.sifr"
        validate_benchmark_input(root, source_path)
        module_count = len(list((root / "src").glob("*.sifr")))
        if module_count < 25:
            raise AssertionError(f"expected at least 25 source modules, found {module_count}")
        original = source_path.read_text(encoding="utf-8")
        api_path = root / "src" / "api.sifr"
        original_api = api_path.read_text(encoding="utf-8")
        api_uri = file_uri(api_path)
        uri = file_uri(source_path)
        document, position, _, _ = request_context(uri)
        params = {"textDocument": document, "position": position}
        client = LspClient(timeout=90.0)
        try:
            initialize(client, root, {"diagnosticsMode": "off"})
            client.notify(
                "textDocument/didOpen",
                {"textDocument": {"uri": uri, "languageId": "sifr", "version": 1, "text": original}},
            )
            client.notify(
                "textDocument/didOpen",
                {"textDocument": {"uri": api_uri, "languageId": "sifr", "version": 1, "text": original_api}},
            )
            cache_stats(client)
            results: dict[str, dict[str, int | float]] = {}
            results["cold"] = measured(
                client, lambda: client.request("textDocument/completion", params)
            )
            expect("cold", results["cold"], hits=0, misses=1)
            results["unchanged"] = measured(
                client,
                lambda: (
                    client.request("textDocument/completion", params),
                    client.request("textDocument/hover", params),
                ),
            )
            expect("unchanged", results["unchanged"], hits=2, misses=0)

            def edit(text: str, version: int) -> None:
                client.notify(
                    "textDocument/didChange",
                    {
                        "textDocument": {"uri": api_uri, "version": version},
                        "contentChanges": [{"text": text}],
                    },
                )
                # A response after didChange proves the server processed the edit.
                cache_stats(client)

            private_text = original_api.replace("return 10", "return 11")
            edit(private_text, 2)
            results["private_edit"] = measured(
                client, lambda: client.request("textDocument/completion", params)
            )
            expect("private_edit", results["private_edit"], hits=0, misses=1)
            api_text = private_text + "\ndef new_public_api() -> int:\n    return 7\n"
            edit(api_text, 3)
            results["api_edit"] = measured(
                client, lambda: client.request("textDocument/completion", params)
            )
            expect("api_edit", results["api_edit"], hits=0, misses=1)

            manifest = root / "sifr.toml"
            manifest.write_text(manifest.read_text(encoding="utf-8") + "\n# E03 external change\n", encoding="utf-8")
            results["external_change"] = measured(
                client, lambda: client.request("textDocument/completion", params)
            )
            expect("external_change", results["external_change"], hits=0, misses=1)

            edit(api_text.replace("return 11", "return 12"), 4)
            before = cache_stats(client)
            cancellation_id = 900_003
            cancellation_started = time.perf_counter()
            client.send_request(cancellation_id, "textDocument/completion", params)
            client.notify("$/cancelRequest", {"id": cancellation_id})
            response = client.wait_for_response(cancellation_id)
            if response.get("error", {}).get("code") != -32800:
                raise AssertionError(f"cancellation did not reject the request: {response}")
            after = cache_stats(client)
            results["cancellation"] = {
                "hits": after[0] - before[0],
                "misses": after[1] - before[1],
                "elapsed_ms": round((time.perf_counter() - cancellation_started) * 1000.0, 3),
            }
            results["after_cancellation"] = measured(
                client, lambda: client.request("textDocument/completion", params)
            )
            if results["cancellation"]["hits"] != 0 or results["cancellation"]["misses"] > 1:
                raise AssertionError("cancelled request reused or repeated a stale cache entry")
            if results["after_cancellation"]["hits"] + results["after_cancellation"]["misses"] != 1:
                raise AssertionError("post-cancellation request did not use the measured cache")
            if results["cancellation"]["misses"] + results["after_cancellation"]["misses"] != 1:
                raise AssertionError("cancellation and recovery did not make exactly one cold query")
            client.request("shutdown", {})
        finally:
            if client.process.poll() is None:
                try:
                    client.request("shutdown", {})
                except Exception:
                    pass
            client.close()
    print(json.dumps({"module_count": module_count, "cache_deltas": results}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
