"""Separate cloud correctness from finite, independently referenced qualification."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

from benchmark_manifest import BenchmarkError, load_manifest, validate_manifest
from benchmark_process import run_owned_process
from cloud_contract import evaluate
from cloud_statistics import PAIRS, POLICY_VERSION, schedule
from reference_host import comparison_mismatches
from reference_profiles import validate_compiler_reference

ROOT = Path(__file__).resolve().parents[3]
AREA = Path(__file__).parent
MANIFEST = AREA / "data/benchmark_manifest.json"
BUDGETS = AREA / "data/budgets.json"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tooling_digest(repo: Path) -> str:
    result = hashlib.sha256()
    for path in sorted((repo / "verification/areas/performance").glob("*.py")):
        result.update(path.name.encode() + b"\0" + path.read_bytes())
    return result.hexdigest()


def read(path: Path) -> dict:
    return json.loads(path.read_text())


def atomic(path: Path, value: dict) -> None:
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")
    temporary.replace(path)


def worker(endpoint: dict, *args: str) -> str:
    repo = Path(endpoint["cloud_repo"])
    command = [sys.executable, str(repo / "verification/areas/performance/cloud_worker.py"), *args]
    completed, timed_out = run_owned_process(command, repo, 600)
    if timed_out or completed.returncode:
        raise BenchmarkError(f"cloud worker failed: {completed.stderr[-2000:]}")
    return completed.stdout


def current_identity(endpoint: dict, path: Path) -> dict:
    return json.loads(worker(endpoint, "identity", "--receipt", str(path)))


def configuration_mismatches(expected: dict, actual: dict) -> list[str]:
    mismatches = comparison_mismatches(expected, actual)
    if expected["execution"]["cloud_runtime_environment"] != actual["execution"]["cloud_runtime_environment"]:
        mismatches.append("execution.cloud_runtime_environment")
    return mismatches


def check_endpoints(paths: dict[str, Path], reference: str) -> dict:
    endpoints = {key: read(path) for key, path in paths.items()}
    if endpoints["baseline"]["cloud_source"] == endpoints["candidate"]["cloud_source"]:
        raise ValueError("candidate must not become its own compiler reference")
    validate_compiler_reference(Path(endpoints["baseline"]["cloud_repo"]), reference)
    for name, endpoint in endpoints.items():
        observed = current_identity(endpoint, paths[name])
        if configuration_mismatches(endpoint["cloud_identity"], observed):
            raise ValueError("prepared cloud configuration changed")
        repo = Path(endpoint["cloud_repo"])
        if tooling_digest(repo) != tooling_digest(ROOT):
            raise ValueError("endpoints require the same reviewed cloud tooling")
    if configuration_mismatches(endpoints["baseline"]["cloud_identity"], endpoints["candidate"]["cloud_identity"]):
        raise ValueError("cloud baseline/candidate host configurations differ")
    return endpoints


def capture(args) -> int:
    paths = {"baseline": args.baseline.resolve(), "candidate": args.candidate.resolve()}
    endpoints = check_endpoints(paths, args.reference_compiler_commit)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    invocation = f"cloud-{time.time_ns()}"
    manifest = load_manifest(MANIFEST)
    cases = validate_manifest(manifest)
    specification = {"policy": POLICY_VERSION, "invocation": invocation,
        "reference_compiler_commit": args.reference_compiler_commit,
        "manifest_sha256": digest(MANIFEST), "budgets_sha256": digest(BUDGETS),
        "started_unix": time.time(), "pairs_per_case": PAIRS, "tooling_sha256": tooling_digest(ROOT),
        "schedules": {case.id: schedule(case.id) for case in cases},
        "endpoints": endpoints, "endpoint_receipts": {key: str(path) for key, path in paths.items()},
        "endpoint_receipt_hashes": {key: digest(path) for key, path in paths.items()}}
    atomic(output / "specification.json", specification)
    pairs = {}
    for case in cases:
        started = time.monotonic()
        # Matched warmup count and persistent endpoint-specific build cache.
        for name in endpoints:
            for index in range(case.warmups):
                worker(endpoints[name], "measure", "--receipt", str(paths[name]), "--case", case.id,
                       "--output", str(output / "warmups" / case.id / name / str(index)),
                       "--artifacts", str(output / "artifacts" / name), "--warmup")
        rows = []
        for index, order in enumerate(schedule(case.id)):
            row = {"order": order, "pair_index": index, "invocation": invocation}
            for letter in order:
                name = "baseline" if letter == "A" else "candidate"
                sample = output / "pairs" / case.id / str(index) / name
                worker(endpoints[name], "measure", "--receipt", str(paths[name]), "--case", case.id,
                       "--output", str(sample), "--artifacts", str(output / "artifacts" / name))
                row[name] = read(sample / "summary.json")
            rows.append(row)
            atomic(output / "pairs" / case.id / str(index) / "pair.json", row)
        pairs[case.id] = rows
        print(f"cloud case={case.id} pairs={PAIRS} elapsed_seconds={time.monotonic() - started:.1f}", flush=True)
    check_endpoints(paths, args.reference_compiler_commit)
    result = evaluate(manifest, read(BUDGETS), pairs)
    evidence = {}
    for path in sorted(output.rglob('*')):
        if path.is_file() and path.parts[len(output.parts)] != "artifacts":
            evidence[str(path.relative_to(output))] = digest(path)
    receipt = {"schema_version": 1, "specification": specification, "completed_unix": time.time(),
               "pairs": pairs, "evaluation": result, "raw_evidence_sha256": evidence}
    atomic(output / "receipt.json", receipt)
    print(f"cloud performance={result['status']} receipt={output / 'receipt.json'}", flush=True)
    return 0 if result["status"] == "pass" else 1 if result["status"] == "regression" else 3


def check(args) -> int:
    path = args.receipt.resolve()
    receipt = read(path)
    specification = receipt["specification"]
    if receipt.get("schema_version") != 1 or specification.get("policy") != POLICY_VERSION:
        raise ValueError("unsupported cloud receipt")
    if (specification["manifest_sha256"] != digest(MANIFEST) or specification["budgets_sha256"] != digest(BUDGETS) or specification["tooling_sha256"] != tooling_digest(ROOT)):
        raise ValueError("cloud corpus or budgets changed")
    age = time.time() - receipt["completed_unix"]
    if age < 0 or age > 24 * 3600:
        raise ValueError("cloud receipt is stale")
    if specification["endpoints"]["candidate"]["cloud_repo"] != str(ROOT):
        raise ValueError("receipt candidate differs from the running worktree")
    paths = {key: Path(value) for key, value in specification["endpoint_receipts"].items()}
    # Measurement concurrency belongs to the immutable endpoint preparation,
    # independently of the enclosing correctness profile's worker count.
    os.environ["CARGO_BUILD_JOBS"] = specification["endpoints"]["candidate"]["cloud_identity"]["execution"]["cargo_jobs"]
    runtime = specification["endpoints"]["candidate"]["cloud_identity"]["execution"]["cloud_runtime_environment"]
    if set(runtime) != {"RAYON_NUM_THREADS", "OMP_NUM_THREADS"}:
        raise ValueError("incomplete cloud runtime configuration")
    for key, value in runtime.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
    for key, endpoint_path in paths.items():
        if digest(endpoint_path) != specification["endpoint_receipt_hashes"][key]:
            raise ValueError("cloud endpoint receipt changed")
    endpoints = check_endpoints(paths, specification["reference_compiler_commit"])
    if endpoints != specification["endpoints"]:
        raise ValueError("cloud source/artifact identities changed")
    for name, expected in receipt["raw_evidence_sha256"].items():
        raw = (path.parent / name).resolve()
        if not raw.is_relative_to(path.parent) or digest(raw) != expected:
            raise ValueError("raw cloud evidence absent or changed")
    pairs = receipt["pairs"]
    invocation = specification["invocation"]
    cases = {case["id"]: case for case in load_manifest(MANIFEST)["cases"]}
    for case_id, observations in pairs.items():
        for index, row in enumerate(observations):
            if row.get("invocation") != invocation or row.get("pair_index") != index:
                raise ValueError("selectively combined cloud invocations")
            raw_path = path.parent / "pairs" / case_id / str(index) / "pair.json"
            if str(raw_path.relative_to(path.parent)) not in receipt["raw_evidence_sha256"]:
                raise ValueError("unbound paired raw evidence")
            raw = read(raw_path)
            if raw != row:
                raise ValueError("cloud paired summary differs from raw evidence")
            for key in paths:
                summary_path = raw_path.parent / key / "summary.json"
                if str(summary_path.relative_to(path.parent)) not in receipt["raw_evidence_sha256"] or read(summary_path) != row[key]:
                    raise ValueError("sample summary differs from bound raw endpoint")
                sample_path = raw_path.parent / key / "samples" / case_id / "0.json"
                if str(sample_path.relative_to(path.parent)) not in receipt["raw_evidence_sha256"]:
                    raise ValueError("missing per-process sample provenance")
                sample = read(sample_path)
                result = sample["result"]
                case = cases[case_id]
                expected_codes = case["expected_exit_codes"] if case["kind"] == "command" else [0]
                if result["timed_out"] or result["exit_code"] not in expected_codes or sample["case_id"] != case_id or sample["warmup"]:
                    raise ValueError("failed or mismatched endpoint process sample")
                executable = endpoints[key]["artifact"]["path"] if case["kind"] == "command" else endpoints[key]["cloud_frontend_helper"]["path"] if case["kind"] == "frontend-query" else sys.executable
                # LSP query's Python command is deliberately recorded as python3.
                if case["kind"] != "lsp-query" and sample["command"][0] != executable:
                    raise ValueError("raw sample executable differs from prepared endpoint")
                if case["kind"] == "command":
                    latencies = [result["duration_ms"]]
                else:
                    stdout_path = Path(sample["stdout_path"]).resolve()
                    if not stdout_path.is_relative_to(path.parent) or str(stdout_path.relative_to(path.parent)) not in receipt["raw_evidence_sha256"]:
                        raise ValueError("query output provenance unavailable")
                    payload = read(stdout_path)
                    latencies = payload["samples_ms"][case["warmups"]:]
                    if {"hits": payload.get("cache_hits", 0), "misses": payload.get("cache_misses", 0)} != row[key]["cache"]:
                        raise ValueError("query cache differs from raw output")
                if latencies != row[key]["latencies_ms"]:
                    raise ValueError("latency summary differs from actual raw measurement")
                if sample["result"]["peak_rss_bytes"] != row[key]["peak_rss_bytes"] or sample["result"]["cpu_time_ms"] != row[key]["cpu_time_ms"]:
                    raise ValueError("cloud counters differ from actual process sample")
                if row[key]["source"] != endpoints[key]["cloud_source"] or row[key]["endpoint_receipt_sha256"] != digest(paths[key]):
                    raise ValueError("cloud sample source/artifact provenance differs")
    result = evaluate(load_manifest(MANIFEST), read(BUDGETS), pairs)
    if result != receipt["evaluation"]:
        raise ValueError("cloud verdict differs from recomputed evidence")
    print(f"Cloud performance: {result['status']} (population p95 unqualified)")
    return 0 if result["status"] == "pass" else 1 if result["status"] == "regression" else 3


def functional(args) -> int:
    import run_benchmarks as bench
    # This path verifies the corpus, not a benchmark or reference claim.
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    cases = validate_manifest(load_manifest(MANIFEST))
    for case in cases:
        bench.run_case(case, output, "smoke")
        print(f"cloud correctness case={case.id} pass", flush=True)
    print("Cloud corpus correctness passed; performance remains unqualified")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    command = commands.add_parser("capture")
    command.add_argument("--baseline", type=Path, required=True)
    command.add_argument("--candidate", type=Path, required=True)
    command.add_argument("--reference-compiler-commit", required=True)
    command.add_argument("--output", type=Path, required=True)
    command = commands.add_parser("check")
    command.add_argument("--receipt", type=Path, required=True)
    command = commands.add_parser("functional")
    command.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        return {"capture": capture, "check": check, "functional": functional}[args.command](args)
    except (BenchmarkError, ValueError, KeyError, TypeError, AttributeError,
            OSError, subprocess.SubprocessError) as error:
        print(f"cloud performance error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
