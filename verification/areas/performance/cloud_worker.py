"""Prepared, source-bound endpoint execution for shared-cloud comparisons."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
from pathlib import Path

import compiler_lanes
import run_benchmarks as bench
from benchmark_manifest import BenchmarkError, load_manifest, validate_manifest
from query_processes import run_query_invocation
from reference_host import reference_identity
from sample_evidence import record_command_sample

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = Path(__file__).parent / "data/benchmark_manifest.json"


def measured_identity() -> dict:
    identity = reference_identity(ROOT, MANIFEST, "latency")
    identity["execution"]["cloud_runtime_environment"] = {
        key: os.environ.get(key) for key in ("RAYON_NUM_THREADS", "OMP_NUM_THREADS")
    }
    return identity


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def clean_source() -> str:
    if git("status", "--porcelain"):
        raise BenchmarkError("cloud endpoint source must be committed and clean")
    return git("rev-parse", "HEAD")


def prepare(output: Path) -> None:
    clean_source()
    subprocess.run(["python3", str(Path(__file__).parent / "prepare_compiler_lane.py"),
                    "--lane", "contributor-dev", "--output", str(output)], cwd=ROOT, check=True)
    receipt = json.loads((output / "receipt.json").read_text())
    executable = output / "sifr"
    shutil.copy2(receipt["artifact"]["path"], executable)
    receipt["artifact"]["path"] = str(executable)
    command = ["cargo", "build", "--locked", "--offline", "-p", "sifr_frontend",
               "--bin", "frontend_query_bench", "--message-format=json-render-diagnostics"]
    with (output / "frontend-cargo.jsonl").open("w") as log:
        subprocess.run(command, cwd=ROOT, stdout=log, check=True)
    messages = [json.loads(line) for line in (output / "frontend-cargo.jsonl").read_text().splitlines()]
    artifacts = [row for row in messages if row.get("reason") == "compiler-artifact"
                 and row.get("target", {}).get("name") == "frontend_query_bench" and row.get("executable")]
    if len(artifacts) != 1 or artifacts[0]["profile"]["opt_level"] != "1":
        raise BenchmarkError("frontend helper must have a measured dev artifact")
    helper = output / "frontend_query_bench"
    shutil.copy2(artifacts[0]["executable"], helper)
    receipt["cloud_frontend_helper"] = {"path": str(helper), "sha256": compiler_lanes.digest(helper),
        "profile": artifacts[0]["profile"], "cargo_messages_sha256": compiler_lanes.digest(output / "frontend-cargo.jsonl")}
    receipt["cloud_source"] = clean_source()
    receipt["cloud_repo"] = str(ROOT)
    receipt["cloud_identity"] = measured_identity()
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")


def endpoint(receipt_path: Path) -> dict:
    receipt = json.loads(receipt_path.read_text())
    if receipt["cloud_source"] != clean_source() or receipt["cloud_repo"] != str(ROOT):
        raise BenchmarkError("cloud endpoint source changed")
    compiler_lanes.configure(ROOT, "contributor-dev", str(receipt_path))
    helper = receipt["cloud_frontend_helper"]
    path = Path(helper["path"])
    if not path.is_absolute() or compiler_lanes.digest(path) != helper["sha256"]:
        raise BenchmarkError("cloud frontend helper changed")
    bench.frontend_bench_binary = lambda: path
    bench._FRONTEND_BENCH_READY = True
    return receipt


def measure(receipt: Path, case_id: str, output: Path, artifacts: Path, warmup: bool = False) -> None:
    endpoint(receipt)
    case = next(case for case in validate_manifest(load_manifest(MANIFEST)) if case.id == case_id)
    output.mkdir(parents=True, exist_ok=False)
    if case.kind == "command":
        command = bench.command_for_case(case, artifacts / case.id / "shared-build")
        result = bench.run_subprocess(command, case.timeout_ms)
        record_command_sample(output, case.id, 0, warmup, command, result)
        if result["timed_out"] or result["exit_code"] not in case.raw["expected_exit_codes"]:
            raise BenchmarkError(f"cloud command correctness failed: {case.id}")
        values = [result["duration_ms"]]
        cache = {"hits": 0, "misses": 0}
    else:
        # Preserve the manifest's internal query counts and cache contract.
        # The whole fresh process supplies one paired inference summary.
        iterations = case.warmups + case.measured
        if case.kind == "frontend-query":
            command = [str(bench.frontend_bench_binary()), case.raw["scenario"],
                       str(ROOT / case.raw["source_path"]), str(iterations),
                       str(case.raw.get("inner_repetitions", 100))]
        else:
            command = ["python3", str(Path(__file__).parent / "lsp_query_bench.py"),
                       case.raw["scenario"], str(ROOT / case.raw["project_root"]),
                       str(ROOT / case.raw["source_path"]), str(iterations),
                       str(case.raw.get("inner_repetitions", 1))]
        def recorded(command, timeout):
            result = bench.run_subprocess(command, timeout)
            record_command_sample(output, case.id, 0, warmup, command, result,
                                  query_role="one-endpoint-with-internal-warmups")
            return result
        result, payload, values = run_query_invocation(case, case.kind, command, iterations, recorded)
        values = values[case.warmups:]
        cache = {"hits": int(payload.get("cache_hits", 0)), "misses": int(payload.get("cache_misses", 0))}
    from measurement_timer import require_managed_counters
    require_managed_counters(result)
    summary = {"case_id": case.id, "latencies_ms": values, "peak_rss_bytes": result["peak_rss_bytes"],
               "cpu_time_ms": result["cpu_time_ms"], "cache": cache,
               "source": clean_source(), "endpoint_receipt_sha256": compiler_lanes.digest(receipt)}
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["prepare", "measure", "identity"])
    parser.add_argument("--output", type=Path)
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--case")
    parser.add_argument("--artifacts", type=Path)
    parser.add_argument("--warmup", action="store_true")
    args = parser.parse_args()
    if args.mode == "prepare":
        prepare(args.output.resolve())
    elif args.mode == "identity":
        endpoint(args.receipt)
        print(json.dumps(measured_identity()))
    else:
        measure(args.receipt, args.case, args.output, args.artifacts, args.warmup)


if __name__ == "__main__":
    main()
