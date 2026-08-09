"""Latency, cache, and process-work measurement for query benchmarks."""

from __future__ import annotations

import json
from collections.abc import Callable
from typing import Any

from benchmark_manifest import BenchmarkCase, BenchmarkError
from process_metrics import latency_metrics, work_metrics

SIZE_METRIC_DEFAULTS = {
    "generated_binary_bytes": None,
    "emitted_rust_lines": None,
    "emitted_rust_bytes": None,
}


def run_query_processes(
    case: BenchmarkCase,
    measured: int,
    label: str,
    command_for_iterations: Callable[[int], list[str]],
    process_runner: Callable[[list[str], int], dict[str, Any]],
) -> dict[str, Any]:
    warmups = case.warmups if measured == case.measured else 1
    aggregate_result, aggregate_payload, aggregate_samples = run_query_invocation(
        case,
        label,
        command_for_iterations(warmups + measured),
        warmups + measured,
        process_runner,
    )
    samples = aggregate_samples[warmups:]
    instruction_samples: list[int] = []
    cycle_samples: list[int] = []
    for _ in range(measured):
        result, _payload, _process_samples = run_query_invocation(
            case,
            label,
            command_for_iterations(warmups + 1),
            warmups + 1,
            process_runner,
        )
        if result["retired_instructions"] is not None:
            instruction_samples.append(result["retired_instructions"])
        if result["cycles_elapsed"] is not None:
            cycle_samples.append(result["cycles_elapsed"])
    return {
        "id": case.id,
        "group": case.group,
        "kind": case.kind,
        "budget_id": case.raw["budget_id"],
        "evidence_category": case.raw["evidence_category"],
        "sample_count": len(samples),
        "samples_ms": samples,
        "samples_instructions": instruction_samples,
        "metrics": latency_metrics(samples)
        | work_metrics(instruction_samples, cycle_samples)
        | {"peak_rss_bytes": aggregate_result["peak_rss_bytes"]}
        | SIZE_METRIC_DEFAULTS,
        "cache": {
            "hits": int(aggregate_payload.get("cache_hits", 0)),
            "misses": int(aggregate_payload.get("cache_misses", 0)),
        },
        "diagnostics_count": int(aggregate_payload.get("diagnostics_count", 0)),
        "timed_out": False,
    }


def run_query_invocation(
    case: BenchmarkCase,
    label: str,
    command: list[str],
    expected_samples: int,
    process_runner: Callable[[list[str], int], dict[str, Any]],
) -> tuple[dict[str, Any], dict[str, Any], list[float]]:
    result = process_runner(command, case.timeout_ms)
    if result["timed_out"]:
        raise BenchmarkError(
            f"{label} benchmark {case.id} timed out after {case.timeout_ms}ms"
        )
    if result["exit_code"] != 0:
        raise BenchmarkError(
            f"{label} benchmark {case.id} failed: {result['stderr_tail']}"
        )
    try:
        payload = json.loads(result["stdout"])
    except json.JSONDecodeError as error:
        raise BenchmarkError(
            f"{label} benchmark {case.id} emitted invalid JSON: {error}"
        ) from error
    if not isinstance(payload, dict):
        raise BenchmarkError(f"{label} benchmark {case.id} emitted non-object JSON")
    raw_samples = payload.get("samples_ms")
    if (
        not isinstance(raw_samples, list)
        or len(raw_samples) != expected_samples
        or not all(isinstance(sample, int | float) for sample in raw_samples)
    ):
        raise BenchmarkError(
            f"{label} benchmark {case.id} did not emit all requested numeric samples_ms"
        )
    if payload.get("timed_out"):
        raise BenchmarkError(f"{label} benchmark {case.id} reported a timeout")
    return result, payload, [float(sample) for sample in raw_samples]


def run_self_test() -> None:
    case = BenchmarkCase(
        {
            "id": "query-process-self-test",
            "group": "self-test",
            "kind": "frontend-query",
            "warmups": 1,
            "measured": 2,
            "timeout_ms": 1000,
            "budget_id": "perf.self-test.query-process",
            "evidence_category": "self-test",
        }
    )
    calls: list[int] = []

    def fake_process(command: list[str], _timeout_ms: int) -> dict[str, Any]:
        iterations = int(command[-1])
        calls.append(iterations)
        is_aggregate = len(calls) == 1
        payload = {
            "samples_ms": list(range(1, iterations + 1)),
            "cache_hits": 2300 if is_aggregate else 400,
            "cache_misses": 100 if is_aggregate else 400,
            "diagnostics_count": 7 if is_aggregate else 1,
            "timed_out": False,
        }
        return {
            "peak_rss_bytes": 1000 + len(calls),
            "retired_instructions": 10_000 + len(calls),
            "cycles_elapsed": 5_000 + len(calls),
            "exit_code": 0,
            "timed_out": False,
            "stdout": json.dumps(payload),
            "stderr_tail": "",
        }

    result = run_query_processes(
        case,
        2,
        "self-test query",
        lambda iterations: ["fake-query", str(iterations)],
        fake_process,
    )
    if calls != [3, 2, 2]:
        raise BenchmarkError(
            f"query process self-test used wrong iteration counts: {calls!r}"
        )
    if result["samples_ms"] != [2.0, 3.0]:
        raise BenchmarkError(
            "query process self-test did not preserve aggregate latency samples"
        )
    if result["samples_instructions"] != [10_002, 10_003]:
        raise BenchmarkError(
            "query process self-test did not collect independent work samples"
        )
    if result["cache"] != {"hits": 2300, "misses": 100}:
        raise BenchmarkError(
            "query process self-test did not preserve aggregate cache evidence"
        )
    if result["diagnostics_count"] != 7:
        raise BenchmarkError(
            "query process self-test did not preserve aggregate diagnostics"
        )
    if result["metrics"]["peak_rss_bytes"] != 1001:
        raise BenchmarkError(
            "query process self-test did not preserve aggregate peak RSS"
        )
