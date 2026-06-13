#!/usr/bin/env python3
"""Phase 35 local benchmark runner."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import resource
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
PERF_ROOT = REPO_ROOT / "verification" / "areas" / "performance"
PERF_DATA = PERF_ROOT / "data"
DEFAULT_MANIFEST = PERF_DATA / "benchmark_manifest.json"
DEFAULT_OUTPUT_ROOT = REPO_ROOT / "target" / "performance"
NEGATIVE_ROOT = PERF_ROOT / "negative_seeds"
RUNNER_VERSION = 1
COMMAND_KINDS = {"command", "frontend-query", "lsp-query"}
COMMAND_MODES = {"check", "build", "fmt-check"}
_FRONTEND_BENCH_READY = False
_SIFR_BINARY_READY = False


class BenchmarkError(Exception):
    pass


def cargo_debug_dir() -> Path:
    target_dir = os.environ.get("CARGO_TARGET_DIR")
    if target_dir:
        path = Path(target_dir)
        if not path.is_absolute():
            path = REPO_ROOT / path
    else:
        path = REPO_ROOT / "target"
    return path / "debug"


def executable_name(name: str) -> str:
    return f"{name}{'.exe' if platform.system() == 'Windows' else ''}"


def frontend_bench_binary() -> Path:
    return cargo_debug_dir() / executable_name("frontend_query_bench")


def sifr_binary() -> Path:
    return cargo_debug_dir() / executable_name("sifr")


@dataclass(frozen=True)
class BenchmarkCase:
    raw: dict[str, Any]

    @property
    def id(self) -> str:
        return str(self.raw["id"])

    @property
    def group(self) -> str:
        return str(self.raw["group"])

    @property
    def kind(self) -> str:
        return str(self.raw["kind"])

    @property
    def measured(self) -> int:
        return int(self.raw["measured"])

    @property
    def warmups(self) -> int:
        return int(self.raw["warmups"])

    @property
    def timeout_ms(self) -> int:
        return int(self.raw["timeout_ms"])

    @property
    def stability_limit(self) -> float:
        return float(self.raw.get("stability_limit", 0.10))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    parser.add_argument("--output-root", default=str(DEFAULT_OUTPUT_ROOT))
    parser.add_argument("--groups", default="")
    parser.add_argument("--case", action="append", default=[])
    parser.add_argument("--case-limit", type=int, default=0)
    parser.add_argument("--sample-scale", choices=["manifest", "smoke"], default="manifest")
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--capture-baseline", action="store_true")
    parser.add_argument("--baseline-output", default="")
    parser.add_argument("--json-out", default="")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    try:
        if args.self_test:
            run_self_test()
            print("performance benchmark runner self-test passed")
            return 0

        manifest = load_manifest(Path(args.manifest))
        cases = validate_manifest(manifest)
        if args.validate_only:
            print(f"performance manifest valid: {len(cases)} cases")
            return 0

        selected = select_cases(
            cases,
            groups=parse_csv(args.groups),
            case_ids=set(args.case),
            case_limit=args.case_limit,
        )
        if not selected:
            raise BenchmarkError("no benchmark cases selected")

        run_report = run_cases(selected, Path(args.output_root), args.sample_scale)
        evidence_path = write_run_report(run_report, Path(args.output_root))
        if args.json_out:
            write_json((REPO_ROOT / args.json_out).resolve(), run_report)
        if args.capture_baseline:
            validate_baseline_capture(run_report, {case.id: case for case in cases})
            baseline = baseline_from_run(run_report, manifest)
            baseline_output = (
                (REPO_ROOT / args.baseline_output).resolve()
                if args.baseline_output
                else PERF_DATA / "baselines.json"
            )
            write_json(baseline_output, baseline)
            print(f"performance baseline captured: {baseline_output.relative_to(REPO_ROOT)}")
        print(f"performance benchmarks passed: {evidence_path.relative_to(REPO_ROOT)}")
        return 0
    except BenchmarkError as error:
        print(f"performance benchmark error: {error}", file=sys.stderr)
        return 1


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise BenchmarkError(f"failed to read manifest {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise BenchmarkError(f"malformed benchmark manifest {path}: {error}") from error
    if not isinstance(data, dict):
        raise BenchmarkError("benchmark manifest root must be an object")
    return data


def validate_manifest(manifest: dict[str, Any]) -> list[BenchmarkCase]:
    if manifest.get("version") != 1:
        raise BenchmarkError("benchmark manifest version must be 1")
    if manifest.get("runner_version") != RUNNER_VERSION:
        raise BenchmarkError(f"benchmark manifest runner_version must be {RUNNER_VERSION}")
    cases_raw = manifest.get("cases")
    if not isinstance(cases_raw, list):
        raise BenchmarkError("benchmark manifest cases must be a list")

    cases: list[BenchmarkCase] = []
    ids: list[str] = []
    budget_ids: list[str] = []
    for raw in cases_raw:
        if not isinstance(raw, dict):
            raise BenchmarkError("benchmark manifest case entries must be objects")
        validate_case(raw)
        case = BenchmarkCase(raw)
        cases.append(case)
        ids.append(case.id)
        budget_ids.append(str(raw["budget_id"]))

    if ids != sorted(ids):
        raise BenchmarkError("benchmark cases must be sorted lexicographically by id")
    if len(ids) != len(set(ids)):
        raise BenchmarkError("benchmark case ids must be unique")
    if len(budget_ids) != len(set(budget_ids)):
        raise BenchmarkError("benchmark budget ids must be unique")

    required = manifest.get("required_groups", {})
    if not isinstance(required, dict):
        raise BenchmarkError("benchmark manifest required_groups must be an object")
    by_group: dict[str, int] = {}
    for case in cases:
        by_group[case.group] = by_group.get(case.group, 0) + 1
    for group, minimum in required.items():
        if not isinstance(minimum, int) or minimum < 0:
            raise BenchmarkError(f"required group {group!r} must have a non-negative integer threshold")
        actual = by_group.get(group, 0)
        if actual < minimum:
            raise BenchmarkError(f"manifest group {group!r} has {actual} cases, need >= {minimum}")

    return cases


def validate_case(raw: dict[str, Any]) -> None:
    required = {
        "id",
        "group",
        "kind",
        "source_path",
        "warmups",
        "measured",
        "timeout_ms",
        "budget_id",
        "evidence_category",
    }
    missing = sorted(required - raw.keys())
    if missing:
        raise BenchmarkError(f"benchmark case is missing required fields: {missing}")
    for field in ["id", "group", "kind", "source_path", "budget_id", "evidence_category"]:
        if not isinstance(raw[field], str) or not raw[field]:
            raise BenchmarkError(f"benchmark case field {field} must be a non-empty string")
    if raw["kind"] not in COMMAND_KINDS:
        raise BenchmarkError(f"benchmark case {raw['id']} has unsupported kind {raw['kind']!r}")
    for field in ["warmups", "measured", "timeout_ms"]:
        if not isinstance(raw[field], int) or raw[field] <= 0:
            raise BenchmarkError(f"benchmark case {raw['id']} field {field} must be a positive integer")
    if raw["kind"] == "command":
        if raw.get("mode") not in COMMAND_MODES:
            raise BenchmarkError(f"command benchmark {raw['id']} must use mode check, build, or fmt-check")
        exit_codes = raw.get("expected_exit_codes")
        if not isinstance(exit_codes, list) or not exit_codes or not all(isinstance(code, int) for code in exit_codes):
            raise BenchmarkError(f"command benchmark {raw['id']} must define integer expected_exit_codes")
        global_args = raw.get("global_args", [])
        if not isinstance(global_args, list) or not all(isinstance(value, str) for value in global_args):
            raise BenchmarkError(f"command benchmark {raw['id']} global_args must be a list of strings")
    if raw["kind"] == "frontend-query":
        if not isinstance(raw.get("scenario"), str) or not raw["scenario"]:
            raise BenchmarkError(f"frontend query benchmark {raw['id']} must define scenario")
    if raw["kind"] == "lsp-query":
        if not isinstance(raw.get("scenario"), str) or not raw["scenario"]:
            raise BenchmarkError(f"LSP query benchmark {raw['id']} must define scenario")
    path = REPO_ROOT / raw["source_path"]
    if not path.exists():
        raise BenchmarkError(f"benchmark case {raw['id']} input path does not exist: {raw['source_path']}")


def select_cases(
    cases: list[BenchmarkCase],
    groups: set[str],
    case_ids: set[str],
    case_limit: int,
) -> list[BenchmarkCase]:
    selected = [
        case
        for case in cases
        if (not groups or case.group in groups) and (not case_ids or case.id in case_ids)
    ]
    if case_ids:
        known = {case.id for case in cases}
        missing = sorted(case_ids - known)
        if missing:
            raise BenchmarkError(f"unknown benchmark case ids requested: {missing}")
    if case_limit:
        if case_limit < 0:
            raise BenchmarkError("--case-limit must be non-negative")
        selected = selected[:case_limit]
    return selected


def run_cases(cases: list[BenchmarkCase], output_root: Path, sample_scale: str) -> dict[str, Any]:
    run_id = f"bench-{int(time.time())}-{os.getpid()}"
    run_root = output_root / run_id
    run_root.mkdir(parents=True, exist_ok=True)
    results = []
    for case in cases:
        started = time.perf_counter()
        status = "pass"
        try:
            result = run_case(case, run_root, sample_scale)
        except Exception:
            status = "fail"
            raise
        finally:
            elapsed_ms = int((time.perf_counter() - started) * 1000.0)
            print(
                f"[sifr-case-timing] bucket=performance case={case.id} "
                f"elapsed_ms={elapsed_ms} status={status}"
            )
        results.append(result)
    return {
        "schema_version": 1,
        "runner_version": RUNNER_VERSION,
        "run_id": run_id,
        "metadata": host_metadata(),
        "results": results,
    }


def run_case(case: BenchmarkCase, run_root: Path, sample_scale: str) -> dict[str, Any]:
    warmups = 1 if sample_scale == "smoke" else case.warmups
    measured = 1 if sample_scale == "smoke" else case.measured
    if case.kind == "frontend-query":
        return run_frontend_query_case(case, measured)
    if case.kind == "lsp-query":
        return run_lsp_query_case(case, measured)

    ensure_sifr_binary()
    samples = []
    peak_rss_values = []
    for sample_index in range(warmups + measured):
        output_suffix = "shared-build" if case.raw.get("mode") == "build" else str(sample_index)
        command = command_for_case(case, run_root / "artifacts" / case.id / output_suffix)
        result = run_subprocess(command, case.timeout_ms)
        if sample_index < warmups:
            continue
        samples.append(result["duration_ms"])
        if result["peak_rss_bytes"] is not None:
            peak_rss_values.append(result["peak_rss_bytes"])
        if result["timed_out"]:
            raise BenchmarkError(f"benchmark {case.id} timed out after {case.timeout_ms}ms")
        expected_exit_codes = set(case.raw["expected_exit_codes"])
        if result["exit_code"] not in expected_exit_codes:
            raise BenchmarkError(
                f"benchmark {case.id} exited {result['exit_code']}, expected {sorted(expected_exit_codes)}"
            )

    stats = sample_stats(samples)
    return {
        "id": case.id,
        "group": case.group,
        "kind": case.kind,
        "budget_id": case.raw["budget_id"],
        "evidence_category": case.raw["evidence_category"],
        "sample_count": len(samples),
        "samples_ms": samples,
        "metrics": stats | {"peak_rss_bytes": max(peak_rss_values) if peak_rss_values else None},
        "cache": {"hits": 0, "misses": 0},
        "timed_out": False,
    }


def run_frontend_query_case(case: BenchmarkCase, measured: int) -> dict[str, Any]:
    ensure_frontend_query_bench()
    warmups = case.warmups if measured == case.measured else 1
    iterations = warmups + measured
    command = [
        str(frontend_bench_binary()),
        str(case.raw["scenario"]),
        str(REPO_ROOT / case.raw["source_path"]),
        str(iterations),
        str(case.raw.get("inner_repetitions", 100)),
    ]
    result = run_subprocess(command, case.timeout_ms)
    if result["timed_out"]:
        raise BenchmarkError(f"frontend query benchmark {case.id} timed out after {case.timeout_ms}ms")
    if result["exit_code"] != 0:
        raise BenchmarkError(f"frontend query benchmark {case.id} failed: {result['stderr_tail']}")
    try:
        payload = json.loads(result["stdout"])
    except json.JSONDecodeError as error:
        raise BenchmarkError(f"frontend query benchmark {case.id} emitted invalid JSON: {error}") from error
    samples = payload.get("samples_ms")
    if not isinstance(samples, list) or not all(isinstance(sample, int | float) for sample in samples):
        raise BenchmarkError(f"frontend query benchmark {case.id} did not emit numeric samples_ms")
    samples = samples[warmups:]
    stats = sample_stats([float(sample) for sample in samples])
    return {
        "id": case.id,
        "group": case.group,
        "kind": case.kind,
        "budget_id": case.raw["budget_id"],
        "evidence_category": case.raw["evidence_category"],
        "sample_count": len(samples),
        "samples_ms": [float(sample) for sample in samples],
        "metrics": stats | {"peak_rss_bytes": result["peak_rss_bytes"]},
        "cache": {
            "hits": int(payload.get("cache_hits", 0)),
            "misses": int(payload.get("cache_misses", 0)),
        },
        "diagnostics_count": int(payload.get("diagnostics_count", 0)),
        "timed_out": bool(payload.get("timed_out", False)),
    }


def run_lsp_query_case(case: BenchmarkCase, measured: int) -> dict[str, Any]:
    warmups = case.warmups if measured == case.measured else 1
    command = [
        "python3",
        str(PERF_ROOT / "lsp_query_bench.py"),
        str(case.raw["scenario"]),
        str(REPO_ROOT / case.raw["source_path"]),
        str(warmups + measured),
        str(case.raw.get("inner_repetitions", 1)),
    ]
    result = run_subprocess(command, case.timeout_ms)
    if result["timed_out"]:
        raise BenchmarkError(f"LSP query benchmark {case.id} timed out after {case.timeout_ms}ms")
    if result["exit_code"] != 0:
        raise BenchmarkError(f"LSP query benchmark {case.id} failed: {result['stderr_tail']}")
    try:
        payload = json.loads(result["stdout"])
    except json.JSONDecodeError as error:
        raise BenchmarkError(f"LSP query benchmark {case.id} emitted invalid JSON: {error}") from error
    samples = payload.get("samples_ms")
    if not isinstance(samples, list) or not all(isinstance(sample, int | float) for sample in samples):
        raise BenchmarkError(f"LSP query benchmark {case.id} did not emit numeric samples_ms")
    samples = [float(sample) for sample in samples[warmups:]]
    stats = sample_stats(samples)
    return {
        "id": case.id,
        "group": case.group,
        "kind": case.kind,
        "budget_id": case.raw["budget_id"],
        "evidence_category": case.raw["evidence_category"],
        "sample_count": len(samples),
        "samples_ms": samples,
        "metrics": stats | {"peak_rss_bytes": result["peak_rss_bytes"]},
        "cache": {
            "hits": int(payload.get("cache_hits", 0)),
            "misses": int(payload.get("cache_misses", 0)),
        },
        "diagnostics_count": int(payload.get("diagnostics_count", 0)),
        "timed_out": bool(payload.get("timed_out", False)),
    }


def ensure_frontend_query_bench() -> None:
    global _FRONTEND_BENCH_READY
    binary = frontend_bench_binary()
    if _FRONTEND_BENCH_READY and binary.exists():
        return
    result = run_subprocess(
        ["cargo", "build", "-q", "-p", "sifr_frontend", "--bin", "frontend_query_bench"],
        180000,
    )
    if result["timed_out"]:
        raise BenchmarkError("building frontend query benchmark helper timed out")
    if result["exit_code"] != 0:
        raise BenchmarkError(f"failed to build frontend query benchmark helper: {result['stderr_tail']}")
    if not binary.exists():
        raise BenchmarkError(f"frontend query benchmark helper was not built at {binary}")
    _FRONTEND_BENCH_READY = True


def ensure_sifr_binary() -> None:
    global _SIFR_BINARY_READY
    binary = sifr_binary()
    if _SIFR_BINARY_READY and binary.exists():
        return
    result = run_subprocess(["cargo", "build", "-q", "-p", "sifr"], 180000)
    if result["timed_out"]:
        raise BenchmarkError("building sifr benchmark binary timed out")
    if result["exit_code"] != 0:
        raise BenchmarkError(f"failed to build sifr benchmark binary: {result['stderr_tail']}")
    if not binary.exists():
        raise BenchmarkError(f"sifr benchmark binary was not built at {binary}")
    _SIFR_BINARY_READY = True


def command_for_case(case: BenchmarkCase, output_dir: Path) -> list[str]:
    source = str(REPO_ROOT / case.raw["source_path"])
    command = [str(sifr_binary())]
    command.extend(case.raw.get("global_args", []))
    if case.raw["mode"] == "fmt-check":
        command.extend(["fmt", "--check", "--no-cache", source])
        return command
    command.extend([str(case.raw["mode"]), source])
    if case.raw["mode"] == "build":
        command.extend(["--output", str(output_dir)])
    return command


def timed_command(command: list[str]) -> list[str]:
    if platform.system() == "Darwin" and Path("/usr/bin/time").exists():
        return ["/usr/bin/time", "-l", *command]
    if platform.system() == "Linux" and Path("/usr/bin/time").exists():
        return ["/usr/bin/time", "-v", *command]
    return command


def parse_peak_rss(stderr: str) -> int | None:
    for line in stderr.splitlines():
        stripped = line.strip()
        if stripped.endswith("maximum resident set size"):
            value = stripped.split(maxsplit=1)[0]
            if value.isdigit():
                return int(value)
        if "Maximum resident set size (kbytes):" in stripped:
            _, value = stripped.rsplit(":", maxsplit=1)
            value = value.strip()
            if value.isdigit():
                return int(value) * 1024
    return None


def run_subprocess(command: list[str], timeout_ms: int) -> dict[str, Any]:
    started = time.perf_counter()
    rss_before = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    try:
        completed = subprocess.run(
            timed_command(command),
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            timeout=timeout_ms / 1000.0,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        return {
            "duration_ms": (time.perf_counter() - started) * 1000.0,
            "peak_rss_bytes": None,
            "exit_code": None,
            "timed_out": True,
            "stdout": error.stdout or "",
            "stderr_tail": tail(error.stderr or ""),
        }
    rss_after = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    peak_rss_bytes = parse_peak_rss(completed.stderr)
    if peak_rss_bytes is None:
        peak_rss_bytes = normalize_rss(rss_after if rss_after >= rss_before else rss_before)
    return {
        "duration_ms": (time.perf_counter() - started) * 1000.0,
        "peak_rss_bytes": peak_rss_bytes,
        "exit_code": completed.returncode,
        "timed_out": False,
        "stdout": completed.stdout,
        "stderr_tail": tail(completed.stderr),
    }


def sample_stats(samples: list[float]) -> dict[str, float]:
    if not samples:
        raise BenchmarkError("cannot compute metrics for an empty sample list")
    median = statistics.median(samples)
    p95 = percentile(samples, 95)
    mad = statistics.median([abs(sample - median) for sample in samples])
    mean = statistics.mean(samples)
    stdev = statistics.pstdev(samples) if len(samples) > 1 else 0.0
    cv = 0.0 if mean == 0 else stdev / mean
    return {
        "median_ms": round(median, 3),
        "p95_ms": round(p95, 3),
        "mad_ms": round(mad, 3),
        "coefficient_variation": round(cv, 6),
    }


def percentile(samples: list[float], percentile_value: int) -> float:
    ordered = sorted(samples)
    if len(ordered) == 1:
        return ordered[0]
    rank = math.ceil((percentile_value / 100.0) * len(ordered)) - 1
    return ordered[max(0, min(rank, len(ordered) - 1))]


def validate_baseline_capture(run_report: dict[str, Any], cases_by_id: dict[str, BenchmarkCase]) -> None:
    for result in run_report.get("results", []):
        if not isinstance(result, dict):
            raise BenchmarkError("benchmark result entries must be objects")
        case_id = result.get("id")
        if case_id not in cases_by_id:
            raise BenchmarkError(f"baseline result references unknown benchmark id {case_id!r}")
        if result.get("timed_out"):
            raise BenchmarkError(f"baseline capture rejected timed out benchmark {case_id}")
        samples = result.get("samples_ms")
        metrics = result.get("metrics")
        cache = result.get("cache")
        if not isinstance(samples, list) or not all(isinstance(sample, int | float) for sample in samples):
            raise BenchmarkError(f"baseline result {case_id} is missing numeric samples_ms")
        if not isinstance(metrics, dict):
            raise BenchmarkError(f"baseline result {case_id} is missing metrics")
        for field in ["median_ms", "p95_ms", "mad_ms", "coefficient_variation", "peak_rss_bytes"]:
            if field not in metrics:
                raise BenchmarkError(f"baseline result {case_id} is missing metric {field}")
        if not isinstance(cache, dict) or "hits" not in cache or "misses" not in cache:
            raise BenchmarkError(f"baseline result {case_id} is missing cache hit/miss metrics")
        case = cases_by_id[str(case_id)]
        cv = float(metrics["coefficient_variation"])
        if cv > case.stability_limit:
            raise BenchmarkError(
                f"baseline result {case_id} is unstable: coefficient_variation={cv:.6f} "
                f"limit={case.stability_limit:.6f}"
            )


def baseline_from_run(run_report: dict[str, Any], manifest: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "runner_version": RUNNER_VERSION,
        "manifest_sha256": sha256(DEFAULT_MANIFEST),
        "default_stability_limit": manifest.get("default_stability_limit", 0.10),
        "metadata": run_report["metadata"],
        "results": run_report["results"],
    }


def host_metadata() -> dict[str, Any]:
    return {
        "captured_at_unix": int(time.time()),
        "host_os": platform.platform(),
        "architecture": platform.machine(),
        "python": platform.python_version(),
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "cargo_lock_sha256": sha256(REPO_ROOT / "Cargo.lock"),
        "compiler_fingerprint": command_output(["cargo", "metadata", "--no-deps", "--format-version", "1"])[:64],
    }


def write_run_report(report: dict[str, Any], output_root: Path) -> Path:
    evidence_dir = output_root / "evidence"
    evidence_dir.mkdir(parents=True, exist_ok=True)
    path = evidence_dir / f"{report['run_id']}.json"
    write_json(path, report)
    return path


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_self_test() -> None:
    assert_fails(lambda: validate_manifest(load_manifest(NEGATIVE_ROOT / "malformed_manifest.json")), "missing required fields")
    assert_fails(lambda: validate_manifest(load_manifest(NEGATIVE_ROOT / "missing_input_manifest.json")), "input path does not exist")
    manifest = load_manifest(DEFAULT_MANIFEST)
    cases = {case.id: case for case in validate_manifest(manifest)}
    assert_fails(
        lambda: validate_baseline_capture(load_manifest(NEGATIVE_ROOT / "timeout_result.json"), cases),
        "timed out benchmark",
    )
    assert_fails(
        lambda: validate_baseline_capture(load_manifest(NEGATIVE_ROOT / "missing_metric_result.json"), cases),
        "missing metric",
    )
    assert_fails(
        lambda: validate_baseline_capture(load_manifest(NEGATIVE_ROOT / "unstable_result.json"), cases),
        "unstable",
    )


def assert_fails(action: Any, expected: str) -> None:
    try:
        action()
    except BenchmarkError as error:
        if expected not in str(error):
            raise BenchmarkError(f"negative self-test failed with wrong diagnostic: {error}") from error
        return
    raise BenchmarkError(f"negative self-test did not fail; expected diagnostic containing {expected!r}")


def parse_csv(value: str) -> set[str]:
    return {part.strip() for part in value.split(",") if part.strip()}


def tail(value: str, limit: int = 2000) -> str:
    return value[-limit:]


def command_output(command: list[str]) -> str:
    try:
        completed = subprocess.run(
            command,
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            timeout=30,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return "unavailable"
    return (completed.stdout or completed.stderr).strip()


def sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def normalize_rss(value: int) -> int:
    if value <= 0:
        return 0
    if platform.system() == "Darwin":
        return int(value)
    return int(value) * 1024


if __name__ == "__main__":
    raise SystemExit(main())
