#!/usr/bin/env python3
"""Benchmark protocol-level Sifr LSP request families."""

from __future__ import annotations

import json
import sys
import time
from pathlib import Path

TOOLING_ROOT = Path(__file__).resolve().parents[1] / "tooling"
sys.path.insert(0, str(TOOLING_ROOT))

from lsp_protocol import LspClient, file_uri  # noqa: E402
from lsp_protocol_smoke import initialize, open_document  # noqa: E402


def run_family(client: LspClient, uri: str) -> None:
    document = {"uri": uri}
    position = {"line": 4, "character": 19}
    range_value = {"start": {"line": 0, "character": 0}, "end": {"line": 6, "character": 0}}
    client.request("textDocument/documentSymbol", {"textDocument": document})
    client.request("workspace/symbol", {"query": "helper"})
    client.request("textDocument/completion", {"textDocument": document, "position": position})
    client.request("textDocument/hover", {"textDocument": document, "position": position})
    client.request("textDocument/definition", {"textDocument": document, "position": position})
    client.request("textDocument/references", {"textDocument": document, "position": position})
    client.request("textDocument/semanticTokens/full", {"textDocument": document})
    client.request("textDocument/inlayHint", {"textDocument": document, "range": range_value})
    client.request("textDocument/foldingRange", {"textDocument": document})
    client.request("textDocument/codeAction", {"textDocument": document, "range": range_value, "context": {"diagnostics": []}})
    client.request("textDocument/formatting", {"textDocument": document, "options": {"tabSize": 4}})
    client.request("textDocument/diagnostic", {"textDocument": document})


def main() -> int:
    if len(sys.argv) != 4:
        print("usage: lsp_query_bench.py <scenario> <source-path> <iterations>", file=sys.stderr)
        return 2
    scenario = sys.argv[1]
    source_path = Path(sys.argv[2]).resolve()
    iterations = int(sys.argv[3])
    source = source_path.read_text(encoding="utf-8")
    client = LspClient(timeout=90.0)
    samples: list[float] = []
    try:
        initialize(client, source_path.parent)
        open_document(client, source_path, source)
        uri = file_uri(source_path)
        for _ in range(iterations):
            started = time.perf_counter()
            if scenario == "lsp.request_families":
                run_family(client, uri)
            else:
                raise ValueError(f"unknown LSP benchmark scenario {scenario!r}")
            samples.append((time.perf_counter() - started) * 1000.0)
        client.request("shutdown", {})
    finally:
        client.close()
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
