#!/usr/bin/env python3
"""local benchmark runner."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import resource
import subprocess
import sys
import time
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

from benchmark_baseline import (
    baseline_from_run,
    validate_baseline_capture,
    work_budgets_from_run,
)
from benchmark_baseline import run_self_test as run_benchmark_baseline_self_test
from benchmark_manifest import (
    RUNNER_VERSION,
    BenchmarkCase,
    BenchmarkError,
    load_manifest,
    select_cases,
    validate_manifest,
)
from controlled_sampling import (
    run_controlled_case,
)
from controlled_sampling import (
    run_self_test as run_controlled_sampling_self_test,
)
from host_control import (
    HostControlError,
    cache_state,
    capture_host_snapshot,
    controlled_policy,
    evaluate_snapshot,
    wait_for_controlled_host,
)
from host_control import (
    run_self_test as run_host_control_self_test,
)
from process_metrics import (
    latency_metrics,
    parse_process_metrics,
    timed_command,
    work_metrics,
    work_sample_evidence,
)
from process_metrics import (
    run_self_test as run_process_metrics_self_test,
)
from query_processes import run_query_processes
from query_processes import run_self_test as run_query_processes_self_test
from trend_baseline import (
    TrendBaselineError,
    baseline_from_reference_run,
    validate_capture_request,
)
from trend_baseline import (
    run_self_test as run_trend_baseline_self_test,
)
from trend_reports import (
    TrendReportError,
    build_trend_report,
)
from trend_reports import (
    run_self_test as run_trend_report_self_test,
)
from work_baseline import (
    WorkBaselineError,
)
from work_baseline import (
    run_self_test as run_work_baseline_self_test,
)
from work_baseline import (
    validate_capture_request as validate_work_capture_request,
)
from work_baseline import (
    validate_source_unchanged as validate_work_source_unchanged,
)

REPO_ROOT = Path(__file__).resolve().parents[3]
PERF_ROOT = REPO_ROOT / "verification" / "areas" / "performance"
PERF_DATA = PERF_ROOT / "data"
DEFAULT_MANIFEST = PERF_DATA / "benchmark_manifest.json"
DEFAULT_TREND_BASELINES = PERF_DATA / "trend" / "current.json"
DEFAULT_BUDGETS = PERF_DATA / "budgets.json"
DEFAULT_OUTPUT_ROOT = REPO_ROOT / "target" / "performance"
NEGATIVE_ROOT = PERF_ROOT / "negative_seeds"
SIZE_METRIC_DEFAULTS = {
    "emitted_rust_lines": None,
    "emitted_rust_bytes": None,
    "generated_binary_bytes": None,
}
_FRONTEND_BENCH_READY = False
_SIFR_BINARY_READY = False


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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    parser.add_argument("--output-root", default=str(DEFAULT_OUTPUT_ROOT))
    parser.add_argument("--groups", default="")
    parser.add_argument("--case", action="append", default=[])
    parser.add_argument("--case-limit", type=int, default=0)
    parser.add_argument(
        "--sample-scale", choices=["manifest", "smoke"], default="manifest"
    )
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--capture-baseline", action="store_true")
    parser.add_argument("--capture-work-baseline", action="store_true")
    parser.add_argument("--baseline-output", default="")
    parser.add_argument("--work-budget-output", default="")
    parser.add_argument("--capture-trend-baseline", action="store_true")
    parser.add_argument("--trend-baseline-output", default="")
    parser.add_argument("--reference-approval", default="")
    parser.add_argument("--trend-baselines", default=str(DEFAULT_TREND_BASELINES))
    parser.add_argument("--trend-json-out", default="")
    parser.add_argument("--json-out", default="")
    parser.add_argument("--invocation-id", default="")
    parser.add_argument("--require-controlled-host", action="store_true")
    parser.add_argument(
        "--controlled-host-mode",
        choices=["latency", "work"],
        default="latency",
    )
    parser.add_argument("--controlled-host-timeout-seconds", type=float, default=180.0)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    try:
        if args.self_test:
            run_self_test()
            print("performance benchmark runner self-test passed")
            return 0

        json_out = (REPO_ROOT / args.json_out).resolve() if args.json_out else None
        if json_out is not None:
            invalidate_output(json_out)

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
        reference_source_commit = validate_capture_request(
            capture_requested=args.capture_trend_baseline,
            capture_budget_baseline=args.capture_baseline,
            require_controlled_host=args.require_controlled_host,
            sample_scale=args.sample_scale,
            selected_count=len(selected),
            manifest_count=len(cases),
            groups=parse_csv(args.groups),
            case_ids=set(args.case),
            case_limit=args.case_limit,
            approval_owner=args.reference_approval,
            profile=os.environ.get("SIFR_VALIDATION_PROFILE", "standalone"),
            thermal_policy=os.environ.get("SIFR_THERMAL_POLICY", "unspecified"),
            control_mode=args.controlled_host_mode,
            repo_root=REPO_ROOT,
        )
        work_source_commit = validate_work_capture_request(
            capture_requested=args.capture_work_baseline,
            capture_budget_baseline=args.capture_baseline,
            require_controlled_host=args.require_controlled_host,
            control_mode=args.controlled_host_mode,
            sample_scale=args.sample_scale,
            selected_count=len(selected),
            manifest_count=len(cases),
            groups=parse_csv(args.groups),
            case_ids=set(args.case),
            case_limit=args.case_limit,
            approval_owner=args.reference_approval,
            repo_root=REPO_ROOT,
        )

        invocation_id = (
            args.invocation_id or f"standalone-{int(time.time())}-{os.getpid()}"
        )
        run_report = run_cases(
            selected,
            Path(args.output_root),
            args.sample_scale,
            invocation_id=invocation_id,
            require_controlled_host=args.require_controlled_host,
            control_mode=args.controlled_host_mode,
            controlled_host_timeout_seconds=args.controlled_host_timeout_seconds,
        )
        if work_source_commit is not None:
            validate_work_source_unchanged(work_source_commit, REPO_ROOT)
            run_report["metadata"]["work_baseline_source_commit"] = (
                work_source_commit
            )
        evidence_path = write_run_report(run_report, Path(args.output_root))
        trend_report = build_trend_report(
            run_report, load_json(Path(args.trend_baselines)), RUNNER_VERSION
        )
        if args.trend_json_out:
            trend_path = (REPO_ROOT / args.trend_json_out).resolve()
            write_json(trend_path, trend_report)
        else:
            trend_path = write_trend_report(trend_report, Path(args.output_root))
        if json_out is not None:
            write_json(json_out, run_report)
        if args.capture_work_baseline:
            validate_baseline_capture(run_report, {case.id: case for case in cases})
            work_budgets = work_budgets_from_run(
                load_json(DEFAULT_BUDGETS), run_report
            )
            work_budget_output = (
                (REPO_ROOT / args.work_budget_output).resolve()
                if args.work_budget_output
                else PERF_DATA / "work_budgets.json"
            )
            write_json(work_budget_output, work_budgets)
            print(
                "performance work budgets captured: "
                f"{work_budget_output.relative_to(REPO_ROOT)}"
            )
        if args.capture_baseline:
            validate_baseline_capture(run_report, {case.id: case for case in cases})
            baseline = baseline_from_run(run_report, manifest, Path(args.manifest))
            baseline_output = (
                (REPO_ROOT / args.baseline_output).resolve()
                if args.baseline_output
                else PERF_DATA / "baselines.json"
            )
            write_json(baseline_output, baseline)
            print(
                f"performance baseline captured: {baseline_output.relative_to(REPO_ROOT)}"
            )
        if args.capture_trend_baseline:
            if reference_source_commit is None:
                raise TrendBaselineError(
                    "approved trend baseline capture did not bind a source commit"
                )
            validate_baseline_capture(run_report, {case.id: case for case in cases})
            trend_baseline = baseline_from_reference_run(
                run_report,
                Path(args.manifest),
                evidence_path,
                repo_root=REPO_ROOT,
                approval_owner=args.reference_approval,
                expected_source_commit=reference_source_commit,
            )
            trend_baseline_output = (
                (REPO_ROOT / args.trend_baseline_output).resolve()
                if args.trend_baseline_output
                else DEFAULT_TREND_BASELINES
            )
            write_json(trend_baseline_output, trend_baseline)
            print(
                f"performance trend baseline captured: {trend_baseline_output.relative_to(REPO_ROOT)}"
            )
        print(f"performance benchmarks passed: {evidence_path.relative_to(REPO_ROOT)}")
        print(f"performance trend report: {trend_path.relative_to(REPO_ROOT)}")
        return 0
    except (
        BenchmarkError,
        HostControlError,
        TrendBaselineError,
        TrendReportError,
        WorkBaselineError,
    ) as error:
        print(f"performance benchmark error: {error}", file=sys.stderr)
        return 1


def load_json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise BenchmarkError(f"failed to read JSON {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise BenchmarkError(f"malformed JSON {path}: {error}") from error
    if not isinstance(data, dict):
        raise BenchmarkError(f"{path} root must be an object")
    return data


def run_cases(
    cases: list[BenchmarkCase],
    output_root: Path,
    sample_scale: str,
    *,
    invocation_id: str,
    require_controlled_host: bool,
    control_mode: str,
    controlled_host_timeout_seconds: float,
) -> dict[str, Any]:
    run_id = f"bench-{int(time.time())}-{os.getpid()}"
    run_root = output_root / run_id
    run_root.mkdir(parents=True, exist_ok=True)
    cache_before = current_cache_state()
    if require_controlled_host:
        admission = wait_for_controlled_host(
            controlled_host_timeout_seconds, control_mode=control_mode
        )
    else:
        snapshot = capture_host_snapshot(
            include_calibration=True,
            control_mode=control_mode,
        )
        admission = {
            "status": "record-only",
            "mode": control_mode,
            "policy": controlled_policy(control_mode),
            "accepted_snapshots": [snapshot],
            "observation_count": 1,
            "rejected_observation_count": 0,
            "recent_rejected_observations": [],
            "advisory_reasons": evaluate_snapshot(
                snapshot,
                enforce_load=True,
                control_mode=control_mode,
                require_work_counter=control_mode == "work",
            ),
        }
    results = []
    for case in cases:
        started = time.perf_counter()
        status = "pass"
        try:
            result = run_controlled_case(
                case,
                run_root,
                sample_scale,
                require_controlled_host=require_controlled_host,
                control_mode=control_mode,
                run_case_fn=run_case,
                retry_admission_fn=(
                    (
                        lambda: wait_for_controlled_host(
                            controlled_host_timeout_seconds,
                            control_mode=control_mode,
                        )
                    )
                    if require_controlled_host
                    else None
                ),
                repo_root=REPO_ROOT,
            )
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
        "invocation_id": invocation_id,
        "metadata": host_metadata()
        | {
            "host_control": admission,
            "cache_state_before": cache_before,
            "cache_state_after": current_cache_state(),
        },
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
    samples: list[float] = []
    peak_rss_values: list[int] = []
    instruction_samples: list[int] = []
    cycle_samples: list[int] = []
    shared_build_dir = run_root / "artifacts" / case.id / "shared-build"
    for sample_index in range(warmups + measured):
        output_dir = (
            shared_build_dir
            if case.raw.get("mode") == "build"
            else run_root / "artifacts" / case.id / str(sample_index)
        )
        command = command_for_case(case, output_dir)
        result = run_subprocess(command, case.timeout_ms)
        if sample_index < warmups:
            continue
        samples.append(result["duration_ms"])
        if result["peak_rss_bytes"] is not None:
            peak_rss_values.append(result["peak_rss_bytes"])
        if result["retired_instructions"] is not None:
            instruction_samples.append(result["retired_instructions"])
        if result["cycles_elapsed"] is not None:
            cycle_samples.append(result["cycles_elapsed"])
        if result["timed_out"]:
            raise BenchmarkError(
                f"benchmark {case.id} timed out after {case.timeout_ms}ms"
            )
        expected_exit_codes = set(case.raw["expected_exit_codes"])
        if result["exit_code"] not in expected_exit_codes:
            raise BenchmarkError(
                f"benchmark {case.id} exited {result['exit_code']}, expected {sorted(expected_exit_codes)}"
            )

    stats = latency_metrics(samples)
    size_metrics = (
        collect_build_size_metrics(shared_build_dir)
        if case.raw.get("mode") == "build"
        else SIZE_METRIC_DEFAULTS
    )
    return {
        "id": case.id,
        "group": case.group,
        "kind": case.kind,
        "budget_id": case.raw["budget_id"],
        "evidence_category": case.raw["evidence_category"],
        "sample_count": len(samples),
        "samples_ms": samples,
        **work_sample_evidence(instruction_samples),
        "metrics": stats
        | work_metrics(instruction_samples, cycle_samples)
        | {"peak_rss_bytes": max(peak_rss_values) if peak_rss_values else None}
        | size_metrics,
        "cache": {"hits": 0, "misses": 0},
        "timed_out": False,
    }


def run_frontend_query_case(case: BenchmarkCase, measured: int) -> dict[str, Any]:
    ensure_frontend_query_bench()
    return run_query_processes(
        case,
        measured,
        "frontend query",
        lambda iterations: [
            str(frontend_bench_binary()),
            str(case.raw["scenario"]),
            str(REPO_ROOT / case.raw["source_path"]),
            str(iterations),
            str(case.raw.get("inner_repetitions", 100)),
        ],
        run_subprocess,
    )


def run_lsp_query_case(case: BenchmarkCase, measured: int) -> dict[str, Any]:
    return run_query_processes(
        case,
        measured,
        "LSP query",
        lambda iterations: [
            "python3",
            str(PERF_ROOT / "lsp_query_bench.py"),
            str(case.raw["scenario"]),
            str(REPO_ROOT / case.raw["project_root"]),
            str(REPO_ROOT / case.raw["source_path"]),
            str(iterations),
            str(case.raw.get("inner_repetitions", 1)),
            str(case.raw.get("minimum_project_modules", 1)),
        ],
        run_subprocess,
    )


def ensure_frontend_query_bench() -> None:
    global _FRONTEND_BENCH_READY
    binary = frontend_bench_binary()
    if _FRONTEND_BENCH_READY and binary.exists():
        return
    result = run_subprocess(
        [
            "cargo",
            "build",
            "-q",
            "-p",
            "sifr_frontend",
            "--bin",
            "frontend_query_bench",
        ],
        180000,
    )
    if result["timed_out"]:
        raise BenchmarkError("building frontend query benchmark helper timed out")
    if result["exit_code"] != 0:
        raise BenchmarkError(
            f"failed to build frontend query benchmark helper: {result['stderr_tail']}"
        )
    if not binary.exists():
        raise BenchmarkError(
            f"frontend query benchmark helper was not built at {binary}"
        )
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
        raise BenchmarkError(
            f"failed to build sifr benchmark binary: {result['stderr_tail']}"
        )
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


def collect_build_size_metrics(output_dir: Path) -> dict[str, int | None]:
    project_dir = output_dir / "sifr_output"
    source_dir = project_dir / "src"
    binary_path = project_dir / "target" / "release" / executable_name("sifr_output")
    metrics = dict(SIZE_METRIC_DEFAULTS)
    rust_files = sorted(source_dir.rglob("*.rs"))
    if not rust_files:
        raise BenchmarkError(
            f"build benchmark did not produce emitted Rust files under {source_dir}"
        )
    emitted_lines = 0
    emitted_bytes = 0
    for rust_path in rust_files:
        try:
            rust_source = rust_path.read_text(encoding="utf-8")
        except OSError as error:
            raise BenchmarkError(
                f"failed to read emitted Rust file {rust_path}: {error}"
            ) from error
        emitted_lines += len(rust_source.splitlines())
        emitted_bytes += len(rust_source.encode("utf-8"))
    metrics["emitted_rust_lines"] = emitted_lines
    metrics["emitted_rust_bytes"] = emitted_bytes
    try:
        metrics["generated_binary_bytes"] = binary_path.stat().st_size
    except OSError as error:
        raise BenchmarkError(
            f"build benchmark did not produce release binary {binary_path}: {error}"
        ) from error
    return metrics


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
            "retired_instructions": None,
            "cycles_elapsed": None,
            "cpu_time_ms": None,
            "work_counter_source": "unavailable",
            "exit_code": None,
            "timed_out": True,
            "stdout": error.stdout or "",
            "stderr_tail": tail(error.stderr or ""),
        }
    rss_after = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    process_data = parse_process_metrics(completed.stderr)
    peak_rss_bytes = process_data["peak_rss_bytes"]
    if peak_rss_bytes is None:
        peak_rss_bytes = normalize_rss(max(rss_after, rss_before))
    return {
        "duration_ms": (time.perf_counter() - started) * 1000.0,
        "peak_rss_bytes": peak_rss_bytes,
        "retired_instructions": process_data["retired_instructions"],
        "cycles_elapsed": process_data["cycles_elapsed"],
        "cpu_time_ms": process_data["cpu_time_ms"],
        "work_counter_source": process_data["work_counter_source"],
        "exit_code": completed.returncode,
        "timed_out": False,
        "stdout": completed.stdout,
        "stderr_tail": tail(completed.stderr),
    }


def host_metadata() -> dict[str, Any]:
    return {
        "captured_at_unix": int(time.time()),
        "host_cpu": platform.processor() or platform.machine(),
        "host_os": platform.platform(),
        "architecture": platform.machine(),
        "python": platform.python_version(),
        "uv": command_output(["uv", "--version"]),
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "profile": os.environ.get("SIFR_VALIDATION_PROFILE", "standalone"),
        "thermal_policy": os.environ.get("SIFR_THERMAL_POLICY", "unspecified"),
        "cargo_lock_sha256": sha256(REPO_ROOT / "Cargo.lock"),
        "compiler_fingerprint": command_output(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"]
        )[:64],
    }


def current_cache_state() -> dict[str, Any]:
    return cache_state(
        REPO_ROOT,
        cargo_debug_dir(),
        ["sifr", "frontend_query_bench"],
    )


def write_run_report(report: dict[str, Any], output_root: Path) -> Path:
    evidence_dir = output_root / "evidence"
    evidence_dir.mkdir(parents=True, exist_ok=True)
    path = evidence_dir / f"{report['run_id']}.json"
    write_json(path, report)
    return path


def write_trend_report(report: dict[str, Any], output_root: Path) -> Path:
    trend_dir = output_root / "trend"
    trend_dir.mkdir(parents=True, exist_ok=True)
    path = trend_dir / f"{report['run_id']}.trend.json"
    write_json(path, report)
    return path


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temporary.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    temporary.replace(path)


def invalidate_output(path: Path) -> None:
    try:
        path.unlink(missing_ok=True)
    except OSError as error:
        raise BenchmarkError(
            f"failed to invalidate prior benchmark output {path}: {error}"
        ) from error


def run_self_test() -> None:
    run_benchmark_baseline_self_test()
    run_process_metrics_self_test()
    run_query_processes_self_test()
    run_work_baseline_self_test()
    run_trend_report_self_test()
    run_host_control_self_test()
    run_trend_baseline_self_test()
    with TemporaryDirectory(prefix="sifr-performance-output-self-test-") as raw:
        stale_output = Path(raw) / "latest.json"
        stale_output.write_text("stale\n", encoding="utf-8")
        invalidate_output(stale_output)
        if stale_output.exists():
            raise BenchmarkError(
                "benchmark output invalidation self-test retained stale evidence"
            )
        run_controlled_sampling_self_test(Path(raw))
    assert_fails(
        lambda: validate_manifest(
            load_manifest(NEGATIVE_ROOT / "malformed_manifest.json")
        ),
        "missing required fields",
    )
    assert_fails(
        lambda: validate_manifest(
            load_manifest(NEGATIVE_ROOT / "missing_input_manifest.json")
        ),
        "input path does not exist",
    )
    manifest = load_manifest(DEFAULT_MANIFEST)
    cases = {case.id: case for case in validate_manifest(manifest)}
    old_flat_manifest = json.loads(json.dumps(manifest))
    old_flat_case = next(
        case for case in old_flat_manifest["cases"] if case["kind"] == "lsp-query"
    )
    old_flat_case["project_root"] = str(Path(old_flat_case["source_path"]).parent)
    assert_fails(
        lambda: validate_manifest(old_flat_manifest),
        "project root requires sifr.toml",
    )
    assert_fails(
        lambda: validate_baseline_capture(
            load_manifest(NEGATIVE_ROOT / "timeout_result.json"), cases
        ),
        "timed out benchmark",
    )
    assert_fails(
        lambda: validate_baseline_capture(
            load_manifest(NEGATIVE_ROOT / "missing_metric_result.json"), cases
        ),
        "missing metric",
    )
    assert_fails(
        lambda: validate_baseline_capture(
            load_manifest(NEGATIVE_ROOT / "unstable_result.json"), cases
        ),
        "unstable",
    )


def assert_fails(action: Any, expected: str) -> None:
    try:
        action()
    except BenchmarkError as error:
        if expected not in str(error):
            raise BenchmarkError(
                f"negative self-test failed with wrong diagnostic: {error}"
            ) from error
        return
    raise BenchmarkError(
        f"negative self-test did not fail; expected diagnostic containing {expected!r}"
    )


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
