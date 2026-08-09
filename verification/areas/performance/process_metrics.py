"""Portable subprocess timing and low-variance work-counter parsing."""

from __future__ import annotations

import math
import platform
import re
import statistics
from pathlib import Path
from typing import Any

DARWIN_TIME_RE = re.compile(
    r"^\s*(?P<real>[0-9.]+) real\s+(?P<user>[0-9.]+) user\s+(?P<sys>[0-9.]+) sys\s*$"
)


def timed_command(command: list[str]) -> list[str]:
    if platform.system() == "Darwin" and Path("/usr/bin/time").exists():
        return ["/usr/bin/time", "-l", *command]
    if platform.system() == "Linux" and Path("/usr/bin/time").exists():
        return ["/usr/bin/time", "-v", *command]
    return command


def parse_process_metrics(stderr: str) -> dict[str, int | float | str | None]:
    peak_rss_bytes: int | None = None
    retired_instructions: int | None = None
    cycles_elapsed: int | None = None
    cpu_time_ms: float | None = None
    for line in stderr.splitlines():
        stripped = line.strip()
        match = DARWIN_TIME_RE.match(line)
        if match is not None:
            cpu_time_ms = (
                float(match.group("user")) + float(match.group("sys"))
            ) * 1000.0
        if stripped.endswith("maximum resident set size"):
            peak_rss_bytes = leading_integer(stripped)
        elif "Maximum resident set size (kbytes):" in stripped:
            value = trailing_integer(stripped)
            peak_rss_bytes = value * 1024 if value is not None else None
        elif stripped.endswith("instructions retired"):
            retired_instructions = leading_integer(stripped)
        elif stripped.endswith("cycles elapsed"):
            cycles_elapsed = leading_integer(stripped)
        elif stripped.startswith("User time (seconds):"):
            user_seconds = trailing_float(stripped)
            if user_seconds is not None:
                cpu_time_ms = user_seconds * 1000.0
        elif stripped.startswith("System time (seconds):"):
            system_seconds = trailing_float(stripped)
            if system_seconds is not None:
                cpu_time_ms = (cpu_time_ms or 0.0) + system_seconds * 1000.0
    source = (
        "darwin-rusage-instructions"
        if retired_instructions is not None
        else "unavailable"
    )
    return {
        "peak_rss_bytes": peak_rss_bytes,
        "retired_instructions": retired_instructions,
        "cycles_elapsed": cycles_elapsed,
        "cpu_time_ms": round(cpu_time_ms, 3) if cpu_time_ms is not None else None,
        "work_counter_source": source,
    }


def sample_stats(samples: list[float]) -> dict[str, float]:
    if not samples:
        raise ValueError("cannot compute metrics for an empty sample list")
    median = statistics.median(samples)
    p95 = percentile(samples, 95)
    mad = statistics.median([abs(sample - median) for sample in samples])
    mean = statistics.mean(samples)
    stdev = statistics.pstdev(samples) if len(samples) > 1 else 0.0
    cv = 0.0 if mean == 0 else stdev / mean
    return {
        "median": round(median, 3),
        "p95": round(p95, 3),
        "mad": round(mad, 3),
        "coefficient_variation": round(cv, 6),
    }


def latency_metrics(samples: list[float]) -> dict[str, float]:
    stats = sample_stats(samples)
    return {
        "median_ms": stats["median"],
        "p95_ms": stats["p95"],
        "mad_ms": stats["mad"],
        "coefficient_variation": stats["coefficient_variation"],
    }


def percentile(samples: list[float], percentile_value: int) -> float:
    ordered = sorted(samples)
    rank = math.ceil((percentile_value / 100.0) * len(ordered))
    return ordered[max(0, rank - 1)]


def work_metrics(
    instruction_samples: list[int], cycle_samples: list[int]
) -> dict[str, Any]:
    if not instruction_samples:
        return {
            "median_instructions": None,
            "p95_instructions": None,
            "instructions_mad": None,
            "instructions_coefficient_variation": None,
            "median_cycles_per_instruction": None,
        }
    stats = sample_stats([float(value) for value in instruction_samples])
    ratios = [
        cycles / instructions
        for cycles, instructions in zip(cycle_samples, instruction_samples, strict=False)
        if instructions > 0
    ]
    return {
        "median_instructions": round(stats["median"]),
        "p95_instructions": round(stats["p95"]),
        "instructions_mad": round(stats["mad"]),
        "instructions_coefficient_variation": stats["coefficient_variation"],
        "median_cycles_per_instruction": (
            round(statistics.median(ratios), 6) if ratios else None
        ),
    }


def work_sample_evidence(instruction_samples: list[int]) -> dict[str, list[int]]:
    if not instruction_samples:
        return {}
    return {"samples_instructions": instruction_samples}


def run_self_test() -> None:
    darwin = parse_process_metrics(
        "        1.25 real         0.80 user         0.20 sys\n"
        "            123456  maximum resident set size\n"
        "          98765432  instructions retired\n"
        "          45678901  cycles elapsed\n"
    )
    expected = {
        "peak_rss_bytes": 123456,
        "retired_instructions": 98765432,
        "cycles_elapsed": 45678901,
        "cpu_time_ms": 1000.0,
        "work_counter_source": "darwin-rusage-instructions",
    }
    if darwin != expected:
        raise ValueError(f"Darwin process metric parsing mismatch: {darwin!r}")
    linux = parse_process_metrics(
        "User time (seconds): 1.50\n"
        "System time (seconds): 0.25\n"
        "Maximum resident set size (kbytes): 1024\n"
    )
    if linux["peak_rss_bytes"] != 1024 * 1024 or linux["cpu_time_ms"] != 1750.0:
        raise ValueError(f"Linux process metric parsing mismatch: {linux!r}")
    metrics = work_metrics([100, 101, 99], [50, 51, 49])
    if metrics["median_instructions"] != 100:
        raise ValueError(f"work metric median mismatch: {metrics!r}")
    if work_sample_evidence([]) != {}:
        raise ValueError("empty work samples must not emit evidence")
    if work_sample_evidence([100]) != {"samples_instructions": [100]}:
        raise ValueError("available work samples must emit evidence")


def leading_integer(value: str) -> int | None:
    token = value.split(maxsplit=1)[0]
    return int(token) if token.isdigit() else None


def trailing_integer(value: str) -> int | None:
    token = value.rsplit(":", maxsplit=1)[-1].strip()
    return int(token) if token.isdigit() else None


def trailing_float(value: str) -> float | None:
    token = value.rsplit(":", maxsplit=1)[-1].strip()
    try:
        return float(token)
    except ValueError:
        return None
