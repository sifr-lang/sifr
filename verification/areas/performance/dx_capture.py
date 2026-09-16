#!/usr/bin/env python3
"""Capture DX.1 observations using existing process metrics and raw sample records."""
import argparse
import hashlib
import json
import os
import shutil
import statistics
import sys
import time
from pathlib import Path
from tempfile import TemporaryDirectory

import compiler_lanes
import run_benchmarks as runner
from sample_evidence import record_command_sample

ROOT = Path(__file__).resolve().parents[3]


def observed(command, output, case, number, timeout=300000):
    started = time.monotonic()
    result = runner.run_subprocess(command, timeout)
    record_command_sample(output, case, number, False, command, result)
    return {
        "id": case, "sample": number, "command": command,
        "elapsed_ms": (time.monotonic() - started) * 1000,
        "functional_status": "pass" if result["exit_code"] == 0 and not result["timed_out"] else "fail",
        "process": {k: v for k, v in result.items() if k not in ("stdout", "stderr")},
    }


def lsp_observation(output):
    sys.path.insert(0, str(ROOT / "verification/areas/developer_tooling"))
    from lsp_protocol import LspClient, file_uri
    from lsp_protocol_smoke import initialize, open_document
    from lsp_large_session import RssSampler, rss_bytes_for_pid
    source = ROOT / "demos/stdlib_intrinsics/main.sifr"
    records = []
    for sample in range(21):
        client = LspClient()
        try:
            with RssSampler(client.process.pid, interval_seconds=0.01) as sampler:
                initialize(client, source.parent)
                empty = rss_bytes_for_pid(client.process.pid)
                started = time.monotonic()
                diagnostics = open_document(client, source, source.read_text())
                document = {"uri": file_uri(source)}
                client.request("textDocument/diagnostic", {"textDocument": document})
                client.request("workspace/diagnostic", {})
                loaded = rss_bytes_for_pid(client.process.pid)
                client.notify("textDocument/didClose", {"textDocument": document})
                client.request("workspace/diagnostic", {})
                retained = rss_bytes_for_pid(client.process.pid)
                elapsed = (time.monotonic() - started) * 1000
                client.request("shutdown")
            records.append({
                "sample": sample, "warmup": sample == 0, "elapsed_ms": elapsed,
                "empty_rss_bytes": empty, "steady_rss_bytes": loaded,
                "retained_rss_bytes": retained,
                "peak_rss_bytes": max([r["rss_bytes"] for r in sampler.samples] + [loaded or 0]),
                "rss_samples": sampler.samples,
                "allocation_delta_bytes": None, "decoded_counts": None,
                "unavailable_reason": "pre-migration compiler has no metadata decoder/allocation telemetry",
                "diagnostics": diagnostics,
            })
        finally:
            client.close()
    samples = [r["elapsed_ms"] for r in records[1:]]
    noise = statistics.pstdev(samples) / statistics.mean(samples)
    return {"id": "dx-q09-demanded-stdlib", "source": str(source.relative_to(ROOT)),
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "samples": records, "coefficient_variation": noise,
            "performance_status": "inconclusive" if noise > 0.1 else "observed",
            "comparison_status": "baseline-only", "functional_status": "pass"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lane", choices=compiler_lanes.LANES, required=True)
    parser.add_argument("--receipt", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    compiler_lanes.configure(ROOT, args.lane, args.receipt)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler = str(compiler_lanes.selected_binary(Path("")))
    rows = []
    report = {
        "schema_version": 1, "runner_version": runner.RUNNER_VERSION,
        "metadata": runner.host_metadata(),
        "cache_conditions": {"filesystem_pages": "warm after explicit warmup",
            "project_state": "not implemented at baseline", "stdlib": "source-built before metadata migration",
            "native_cache": "fresh output root then repeated output root", "cargo_registry": "prepared"},
        "performance_status": "baseline observation; no optimized target claimed",
        "results": rows,
    }
    def save():
        (output / "baseline.json").write_text(json.dumps(report, indent=2) + "\n")
    try:
        source = ROOT / "verification/areas/performance/query_projects/edit_loop/main.sifr"
        for number in range(21):
            row = observed([compiler, "check", str(source)], output, "dx-fresh-unchanged-check", number)
            row["warmup"] = number == 0
            rows.append(row)
            save()
            if row["functional_status"] != "pass":
                raise RuntimeError("fixed check baseline failed; see raw evidence")
        with TemporaryDirectory(prefix="sifr-dx-native-") as directory:
            project = Path(directory) / "source"
            shutil.copytree(ROOT / "demos/additional_modules", project)
            destination = Path(directory) / "output"
            for number, label in enumerate(("first", "noop", "edited")):
                if label == "edited":
                    path = project / "main.sifr"
                    path.write_text(path.read_text().replace("Sifr stdlib gzip compression!", "DX.1 edited native rebuild!"))
                rows.append(observed([compiler, "build", str(project / "main.sifr"), "--output", str(destination)],
                                     output, f"dx-native-{label}", number))
                save()
                if rows[-1]["functional_status"] != "pass":
                    raise RuntimeError("native baseline failed; see raw evidence")
        if args.lane == "product-installed-optimized":
            rows.append(lsp_observation(output))
        else:
            rows.append(observed([
                "python3", str(ROOT / "verification/areas/performance/run_benchmarks.py"),
                "--compiler-lane", args.lane, "--compiler-receipt", args.receipt,
                "--case", "incremental-local-loop-001-unchanged-file-update",
                "--case", "interactive-tooling-foundation-004-changed-file-invalidation",
                "--output-root", str(output / "sessions"),
                "--json-out", str(output / "sessions.json"),
            ], output, "dx-unchanged-edited-sessions", 0, 3600000))
            save()
            rows.append(observed(["cargo", "test", "--locked", "-p", "sifr_driver", "--lib",
                                  "test_get_or_init_stdlib_cache"], output, "dx-bare-cargo-tests", 0, 3600000))
        save()
    except BaseException as error:
        report["functional_status"] = "failed"
        report["failure"] = str(error)
        save()
        raise
    report["functional_status"] = "pass" if all(r["functional_status"] == "pass" for r in rows) else "failed"
    save()
    validate_report(report)
    print(f"{output / 'baseline.json'} sha256={compiler_lanes.digest(output / 'baseline.json')}")
    if report["functional_status"] != "pass":
        raise SystemExit(1)


def validate_report(report):
    measurement = report["metadata"]["compiler_measurement"]
    lane = measurement["lane"]
    if measurement["compiler_build_profile"] != compiler_lanes.LANES[lane]:
        raise ValueError("baseline compiler lane/profile mismatch")
    if not measurement.get("artifact", {}).get("sha256"):
        raise ValueError("baseline lacks actual artifact identity")
    rows = report["results"]
    checks = [row for row in rows if row["id"] == "dx-fresh-unchanged-check"]
    if len(checks) != 21 or sum(row["warmup"] for row in checks) != 1:
        raise ValueError("baseline check samples/warmups incomplete")
    required = {"dx-native-first", "dx-native-noop", "dx-native-edited"}
    required |= ({"dx-q09-demanded-stdlib"} if lane == "product-installed-optimized"
                 else {"dx-bare-cargo-tests", "dx-unchanged-edited-sessions"})
    if not required <= {row["id"] for row in rows}:
        raise ValueError("baseline workload inventory incomplete")
    if report["functional_status"] != "pass" or any(row["functional_status"] != "pass" for row in rows):
        raise ValueError("baseline contains a functional failure")
    return report


if __name__ == "__main__":
    main()
