#!/usr/bin/env python3
"""Exercise Sifr LSP over the large synthetic verification submodule."""

from __future__ import annotations

import argparse
import json
import math
import os
import statistics
import subprocess
import sys
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

from lsp_protocol import LspClient, LspProtocolError, file_uri
from lsp_protocol_smoke import initialize


REPO_ROOT = Path(__file__).resolve().parents[2]
SUBMODULE_ROOT = REPO_ROOT / "verification" / "sifr-large-lsp-verification"
MANIFEST_PATH = SUBMODULE_ROOT / "manifest.json"
DEFAULT_OUTPUT_ROOT = REPO_ROOT / "target" / "lsp_large_session"
RSS_POLL_SECONDS = 0.25
BYTES_PER_KIB = 1024
BYTES_PER_MIB = 1024 * 1024


@dataclass(frozen=True)
class ModeConfig:
    opened_modules_per_package: int
    edit_rounds: int
    package_count: int
    diagnostics_mode: str
    diagnostic_requests: bool
    storm_burst_size: int
    max_peak_rss_mib: int
    max_p95_ms: float
    max_rss_slope_mib_per_min: float


MODES = {
    "smoke": ModeConfig(
        opened_modules_per_package=0,
        edit_rounds=8,
        package_count=1,
        diagnostics_mode="off",
        diagnostic_requests=False,
        storm_burst_size=1,
        max_peak_rss_mib=128,
        max_p95_ms=1_000.0,
        max_rss_slope_mib_per_min=32.0,
    ),
    "full": ModeConfig(
        opened_modules_per_package=10,
        edit_rounds=300,
        package_count=3,
        diagnostics_mode="off",
        diagnostic_requests=True,
        storm_burst_size=10,
        max_peak_rss_mib=256,
        max_p95_ms=1_000.0,
        max_rss_slope_mib_per_min=64.0,
    ),
}


class RssSampler:
    def __init__(self, pid: int, interval_seconds: float = RSS_POLL_SECONDS) -> None:
        self.pid = pid
        self.interval_seconds = interval_seconds
        self.samples: list[dict[str, float | int]] = []
        self._started = time.monotonic()
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, name="lsp-rss-sampler", daemon=True)

    def __enter__(self) -> RssSampler:
        self._thread.start()
        return self

    def __exit__(self, _exc_type: object, _exc: object, _tb: object) -> None:
        self._stop.set()
        self._thread.join(timeout=2)

    def _run(self) -> None:
        while not self._stop.is_set():
            rss = rss_bytes_for_pid(self.pid)
            if rss is not None:
                self.samples.append(
                    {
                        "elapsed_ms": round((time.monotonic() - self._started) * 1000.0, 3),
                        "rss_bytes": rss,
                    }
                )
            self._stop.wait(self.interval_seconds)


@dataclass
class DocumentState:
    path: Path
    text: str
    version: int = 1

    @property
    def uri(self) -> str:
        return file_uri(self.path)

    def replace_text(self, text: str) -> dict[str, Any]:
        self.version += 1
        self.text = text
        return {
            "textDocument": {"uri": self.uri, "version": self.version},
            "contentChanges": [{"text": text}],
        }


def rss_bytes_for_pid(pid: int) -> int | None:
    try:
        completed = subprocess.run(
            ["ps", "-o", "rss=", "-p", str(pid)],
            text=True,
            capture_output=True,
            timeout=2,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    value = completed.stdout.strip()
    if not value.isdigit():
        return None
    return int(value) * BYTES_PER_KIB


def load_manifest() -> dict[str, Any]:
    if not MANIFEST_PATH.exists():
        raise LspProtocolError(
            "large LSP verification submodule is not initialized; run "
            "`scripts/clone_subrepos.sh` or `git submodule update --init verification/sifr-large-lsp-verification`"
        )
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    if manifest.get("version") != 1:
        raise LspProtocolError("large LSP corpus manifest version must be 1")
    return manifest


def package_roots(manifest: dict[str, Any], config: ModeConfig) -> list[Path]:
    entrypoints = [SUBMODULE_ROOT / path for path in manifest["entrypoints"]]
    roots = [entrypoint.parents[1] for entrypoint in entrypoints]
    return roots[: config.package_count]


def run_large_session(mode: str, output_path: Path | None, require_submodule: bool) -> int:
    config = MODES[mode]
    try:
        manifest = load_manifest()
    except LspProtocolError as error:
        if require_submodule:
            print(f"LSP large session: FAIL: {error}", file=sys.stderr)
            return 1
        print(f"LSP large session: SKIP: {error}")
        return 0

    client = LspClient(timeout=180.0, extra_args=["--parent-pid", str(os.getpid())])
    latencies: list[dict[str, Any]] = []
    stale_or_cancelled = 0
    diagnostics_seen = 0
    opened: list[DocumentState] = []
    started = time.monotonic()
    operation_error: Exception | None = None
    report: dict[str, Any] | None = None
    try:
        with RssSampler(client.process.pid) as rss:
            initialize(
                client,
                SUBMODULE_ROOT,
                {"diagnosticsMode": config.diagnostics_mode},
                work_done_progress=True,
            )
            for root in package_roots(manifest, config):
                opened.extend(
                    open_project_documents(
                        client,
                        root,
                        config.opened_modules_per_package,
                        config.diagnostics_mode != "off",
                        latencies,
                    )
                )
            assert_entrypoint_hover(client, opened, latencies)
            for round_index in range(config.edit_rounds):
                category = edit_category(round_index)
                document = choose_document(opened, round_index, category)
                burst_size = config.storm_burst_size if category == "storm" else 1
                for burst_index in range(burst_size):
                    updated = edit_text(document, round_index, category, burst_index)
                    diagnostics_seen += timed_notify_change(
                        client,
                        document,
                        updated,
                        category,
                        config.diagnostics_mode != "off",
                        latencies,
                    )
                stale_or_cancelled += run_request_mix(
                    client,
                    document,
                    round_index,
                    config.diagnostic_requests,
                    latencies,
                )
            if config.diagnostics_mode != "off":
                timed_request(client, "workspace/diagnostic", {}, "workspace-diagnostic", latencies)
            client.request("shutdown", {})
            client.notify("exit", {})
            elapsed_ms = (time.monotonic() - started) * 1000.0
            report = build_report(
                mode,
                manifest,
                config,
                elapsed_ms,
                latencies,
                rss.samples,
                diagnostics_seen,
                stale_or_cancelled,
            )
    except Exception as error:
        operation_error = error
    finally:
        try:
            client.close()
        except LspProtocolError as error:
            if operation_error is None:
                operation_error = error

    if operation_error is not None:
        raise operation_error
    if report is None:
        raise LspProtocolError("large LSP session did not produce a report")

    write_report(report, output_path or DEFAULT_OUTPUT_ROOT / f"{mode}.latest.json")
    failures = threshold_failures(report)
    if failures:
        print("LSP large session: FAIL")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print(
        "LSP large session: PASS "
        f"mode={mode} ops={report['operation_count']} "
        f"p95_ms={report['metrics']['p95_ms']} "
        f"peak_rss_mib={report['metrics']['peak_rss_bytes'] / BYTES_PER_MIB:.1f}"
    )
    return 0


def open_project_documents(
    client: LspClient,
    root: Path,
    module_count: int,
    expect_diagnostics: bool,
    latencies: list[dict[str, Any]],
) -> list[DocumentState]:
    paths = [root / "src" / "main.sifr"]
    paths.extend(root / "src" / f"module_{index:04d}.sifr" for index in range(module_count))
    paths.extend(root / "src" / f"api_{index:02d}.sifr" for index in range(min(2, module_count // 4)))
    documents: list[DocumentState] = []
    for path in paths:
        state = DocumentState(path=path, text=path.read_text(encoding="utf-8"))
        timed_open_document(client, state, expect_diagnostics, latencies)
        documents.append(state)
    return documents


def timed_open_document(
    client: LspClient,
    state: DocumentState,
    expect_diagnostics: bool,
    latencies: list[dict[str, Any]],
) -> None:
    def action() -> None:
        client.notify(
            "textDocument/didOpen",
            {
                "textDocument": {
                    "uri": state.uri,
                    "languageId": "sifr",
                    "version": state.version,
                    "text": state.text,
                }
            },
        )
        if not expect_diagnostics:
            return
        published = client.wait_for_notification("textDocument/publishDiagnostics")
        params = published.get("params", {})
        if params.get("uri") != state.uri:
            raise LspProtocolError("publishDiagnostics used the wrong document URI")
        if params.get("version") != state.version:
            raise LspProtocolError("publishDiagnostics did not preserve document version")

    timed("didOpen", state.path.as_posix(), latencies, action)


def assert_entrypoint_hover(
    client: LspClient,
    documents: list[DocumentState],
    latencies: list[dict[str, Any]],
) -> None:
    entrypoints = [document for document in documents if document.path.name == "main.sifr"]
    for document in entrypoints:
        hover = timed_request(
            client,
            "textDocument/hover",
            {"textDocument": {"uri": document.uri}, "position": request_position(document)},
            "hover",
            latencies,
        )
        value = hover.get("contents", {}).get("value", "") if isinstance(hover, dict) else ""
        if "value_0000" not in value:
            raise LspProtocolError(f"large entrypoint hover missed value_0000: {document.uri}")


def edit_category(round_index: int) -> str:
    if round_index % 15 == 0:
        return "storm"
    if round_index % 5 == 0:
        return "shared-api"
    return "private-body"


def choose_document(
    documents: list[DocumentState],
    round_index: int,
    category: str,
) -> DocumentState:
    api_documents = [document for document in documents if document.path.name.startswith("api_")]
    module_documents = [document for document in documents if document.path.name.startswith("module_")]
    if category == "shared-api" and api_documents:
        return api_documents[(round_index // 5) % len(api_documents)]
    if category in {"private-body", "storm"} and module_documents:
        return module_documents[round_index % len(module_documents)]
    return documents[round_index % len(documents)]


def edit_text(
    document: DocumentState,
    round_index: int,
    category: str,
    burst_index: int,
) -> str:
    if category == "shared-api" and document.path.name.startswith("api_"):
        return edit_shared_api_text(document.text, round_index)
    if category == "private-body":
        return edit_private_body_text(document.text, round_index)
    marker = f"# lsp-large-session {category} {round_index}\n"
    if burst_index:
        marker = f"# lsp-large-session {category} {round_index}-{burst_index}\n"
    lines = [line for line in document.text.splitlines() if not line.startswith("# lsp-large-session")]
    lines.append(marker.rstrip())
    return "\n".join(lines) + "\n"


def edit_shared_api_text(source: str, round_index: int) -> str:
    parameter = "value" if round_index % 2 else "seed"
    lines = []
    for line in source.splitlines():
        if line.startswith("def api_"):
            lines.append(line.replace("(seed: int)", f"({parameter}: int)").replace("(value: int)", f"({parameter}: int)"))
        elif "return " in line:
            lines.append(line.replace("seed", parameter).replace("value", parameter))
        elif not line.startswith("# lsp-large-session"):
            lines.append(line)
    lines.append(f"# lsp-large-session shared-api {round_index}")
    return "\n".join(lines) + "\n"


def edit_private_body_text(source: str, round_index: int) -> str:
    adjustment = round_index % 3
    lines = []
    changed = False
    for line in source.splitlines():
        if line.startswith("# lsp-large-session"):
            continue
        if not changed and line.strip().startswith("return "):
            prefix, expression = line.split("return ", 1)
            lines.append(f"{prefix}return ({expression}) + {adjustment}")
            changed = True
        else:
            lines.append(line)
    lines.append(f"# lsp-large-session private-body {round_index}")
    return "\n".join(lines) + "\n"


def timed_notify_change(
    client: LspClient,
    document: DocumentState,
    updated_text: str,
    category: str,
    expect_diagnostics: bool,
    latencies: list[dict[str, Any]],
) -> int:
    def action() -> int:
        client.notify("textDocument/didChange", document.replace_text(updated_text))
        if not expect_diagnostics:
            return 0
        diagnostics = client.wait_for_notification("textDocument/publishDiagnostics")
        params = diagnostics.get("params", {})
        if params.get("uri") != document.uri:
            raise LspProtocolError("didChange diagnostics used wrong URI")
        if params.get("version") != document.version:
            raise LspProtocolError("didChange diagnostics used stale version")
        return 1

    return timed("didChange", category, latencies, action)


def run_request_mix(
    client: LspClient,
    document: DocumentState,
    round_index: int,
    include_diagnostics_requests: bool,
    latencies: list[dict[str, Any]],
) -> int:
    stale_or_cancelled = 0
    doc = {"uri": document.uri}
    symbol_position = request_position(document)
    full_range = {
        "start": {"line": 0, "character": 0},
        "end": {"line": max(0, len(document.text.splitlines()) - 1), "character": 0},
    }
    request_plan: list[tuple[str, dict[str, Any], str]] = [
        ("textDocument/documentSymbol", {"textDocument": doc}, "document-symbol"),
        ("textDocument/hover", {"textDocument": doc, "position": symbol_position}, "hover"),
        ("textDocument/completion", {"textDocument": doc, "position": symbol_position}, "completion"),
    ]
    if round_index % 3 == 0:
        request_plan.append(("textDocument/references", {"textDocument": doc, "position": symbol_position}, "references"))
    if round_index % 4 == 0:
        request_plan.append(("textDocument/semanticTokens/full", {"textDocument": doc}, "semantic-tokens"))
    if round_index % 6 == 0:
        request_plan.append(("workspace/symbol", {"query": "api_00_value"}, "workspace-symbol"))
    if include_diagnostics_requests and round_index % 10 == 0:
        request_plan.append(("textDocument/diagnostic", {"textDocument": doc}, "document-diagnostic"))
    if round_index % 12 == 0:
        request_plan.append(("textDocument/inlayHint", {"textDocument": doc, "range": full_range}, "inlay-hint"))

    for method, params, label in request_plan:
        try:
            timed_request(client, method, params, label, latencies)
        except LspProtocolError as error:
            if "Request cancelled" in str(error) or "superseded" in str(error):
                stale_or_cancelled += 1
            else:
                raise
    return stale_or_cancelled


def request_position(document: DocumentState) -> dict[str, int]:
    for line_index, line in enumerate(document.text.splitlines()):
        for token in ("value_", "api_"):
            offset = line.find(token)
            if offset >= 0:
                return {"line": line_index, "character": offset}
    return {"line": 0, "character": 0}


def timed_request(
    client: LspClient,
    method: str,
    params: dict[str, Any],
    label: str,
    latencies: list[dict[str, Any]],
) -> Any:
    return timed(label, method, latencies, lambda: client.request(method, params))


def timed(label: str, detail: str, latencies: list[dict[str, Any]], action: Callable[[], Any]) -> Any:
    started = time.perf_counter()
    try:
        result = action()
    except LspProtocolError as error:
        raise LspProtocolError(f"{label} {detail} failed: {error}") from error
    latencies.append(
        {
            "label": label,
            "detail": detail,
            "duration_ms": round((time.perf_counter() - started) * 1000.0, 3),
        }
    )
    return result


def build_report(
    mode: str,
    manifest: dict[str, Any],
    config: ModeConfig,
    elapsed_ms: float,
    latencies: list[dict[str, Any]],
    rss_samples: list[dict[str, float | int]],
    diagnostics_seen: int,
    stale_or_cancelled: int,
) -> dict[str, Any]:
    samples = [float(sample["duration_ms"]) for sample in latencies]
    metrics = sample_stats(samples)
    rss_values = [int(sample["rss_bytes"]) for sample in rss_samples]
    peak_rss = max(rss_values) if rss_values else 0
    metrics["peak_rss_bytes"] = peak_rss
    metrics["rss_slope_mib_per_min"] = round(rss_slope_mib_per_min(rss_samples), 6)
    return {
        "schema_version": 1,
        "mode": mode,
        "diagnostics_mode": config.diagnostics_mode,
        "corpus_sha256": manifest.get("corpus_sha256"),
        "shape": manifest.get("shape"),
        "operation_count": len(latencies),
        "elapsed_ms": round(elapsed_ms, 3),
        "samples_ms": samples,
        "metrics": metrics,
        "thresholds": {
            "max_peak_rss_bytes": config.max_peak_rss_mib * BYTES_PER_MIB,
            "max_p95_ms": config.max_p95_ms,
            "max_rss_slope_mib_per_min": config.max_rss_slope_mib_per_min,
            "max_stale_or_cancelled_rate": 0.05,
        },
        "diagnostics_seen": diagnostics_seen,
        "stale_or_cancelled": stale_or_cancelled,
        "rss_samples": rss_samples,
        "latencies": latencies,
        "timed_out": False,
    }


def sample_stats(samples: list[float]) -> dict[str, float]:
    if not samples:
        raise LspProtocolError("large LSP session did not record latency samples")
    median = statistics.median(samples)
    p95 = percentile(samples, 95)
    mad = statistics.median([abs(sample - median) for sample in samples])
    mean = statistics.mean(samples)
    stdev = statistics.pstdev(samples) if len(samples) > 1 else 0.0
    return {
        "median_ms": round(median, 3),
        "p95_ms": round(p95, 3),
        "mad_ms": round(mad, 3),
        "coefficient_variation": round(0.0 if mean == 0 else stdev / mean, 6),
    }


def percentile(samples: list[float], percentile_value: int) -> float:
    ordered = sorted(samples)
    if len(ordered) == 1:
        return ordered[0]
    rank = math.ceil((percentile_value / 100.0) * len(ordered)) - 1
    return ordered[max(0, min(rank, len(ordered) - 1))]


def rss_slope_mib_per_min(samples: list[dict[str, float | int]]) -> float:
    if len(samples) < 4:
        return 0.0
    second_half = samples[len(samples) // 2 :]
    xs = [float(sample["elapsed_ms"]) / 60_000.0 for sample in second_half]
    ys = [float(sample["rss_bytes"]) / BYTES_PER_MIB for sample in second_half]
    x_mean = statistics.mean(xs)
    y_mean = statistics.mean(ys)
    denominator = sum((x - x_mean) ** 2 for x in xs)
    if denominator == 0:
        return 0.0
    return sum((x - x_mean) * (y - y_mean) for x, y in zip(xs, ys, strict=True)) / denominator


def threshold_failures(report: dict[str, Any]) -> list[str]:
    metrics = report["metrics"]
    thresholds = report["thresholds"]
    failures = []
    if metrics["peak_rss_bytes"] > thresholds["max_peak_rss_bytes"]:
        failures.append(
            f"peak RSS {metrics['peak_rss_bytes']} > {thresholds['max_peak_rss_bytes']}"
        )
    if metrics["p95_ms"] > thresholds["max_p95_ms"]:
        failures.append(f"p95 latency {metrics['p95_ms']}ms > {thresholds['max_p95_ms']}ms")
    if metrics["rss_slope_mib_per_min"] > thresholds["max_rss_slope_mib_per_min"]:
        failures.append(
            "RSS growth slope "
            f"{metrics['rss_slope_mib_per_min']} MiB/min > "
            f"{thresholds['max_rss_slope_mib_per_min']} MiB/min"
        )
    stale_rate = report["stale_or_cancelled"] / max(report["operation_count"], 1)
    if stale_rate > thresholds["max_stale_or_cancelled_rate"]:
        failures.append(f"stale/cancelled rate {stale_rate:.3f} exceeds bound")
    return failures


def write_report(report: dict[str, Any], output_path: Path) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_self_test() -> None:
    stats = sample_stats([1.0, 2.0, 3.0, 4.0])
    if stats["median_ms"] != 2.5 or stats["p95_ms"] != 4.0:
        raise SystemExit("LSP large session self-test failed: bad stats")
    slope = rss_slope_mib_per_min(
        [
            {"elapsed_ms": 0.0, "rss_bytes": 10 * BYTES_PER_MIB},
            {"elapsed_ms": 60_000.0, "rss_bytes": 11 * BYTES_PER_MIB},
            {"elapsed_ms": 120_000.0, "rss_bytes": 12 * BYTES_PER_MIB},
            {"elapsed_ms": 180_000.0, "rss_bytes": 13 * BYTES_PER_MIB},
        ]
    )
    if slope <= 0:
        raise SystemExit("LSP large session self-test failed: bad RSS slope")
    print("LSP large session self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=sorted(MODES), default="smoke")
    parser.add_argument("--json-out", type=Path)
    parser.add_argument("--require-submodule", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0
    try:
        return run_large_session(args.mode, args.json_out, args.require_submodule)
    except (LspProtocolError, OSError, json.JSONDecodeError) as error:
        print(f"LSP large session: FAIL: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
