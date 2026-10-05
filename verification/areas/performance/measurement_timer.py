"""Actual timer selection and identity for managed Linux measurements."""

import hashlib
import os
import subprocess
from pathlib import Path

from benchmark_manifest import BenchmarkError


def linux_timer() -> Path:
    path = Path(os.environ.get("SIFR_PERFORMANCE_TIME", "/usr/bin/time"))
    if not path.is_absolute() or not path.is_file() or not os.access(path, os.X_OK):
        raise BenchmarkError("managed Linux measurements require an absolute executable GNU Time")
    return path


def managed_timer_identity() -> dict[str, str]:
    path = linux_timer()
    try:
        result = subprocess.run([str(path), "--version"], capture_output=True,
                                text=True, check=True, timeout=30)
    except (OSError, subprocess.SubprocessError) as error:
        raise BenchmarkError(f"managed Linux measurement timer unavailable: {error}") from error
    if not result.stdout.startswith("time (GNU Time)"):
        raise BenchmarkError("managed Linux measurement timer must be GNU Time")
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "version": result.stdout.splitlines()[0]}


def require_managed_counters(metrics: dict) -> None:
    if os.environ.get("SIFR_PERFORMANCE_HOST_KIND") != "managed-linux":
        return
    if metrics.get("peak_rss_bytes") is None or metrics.get("cpu_time_ms") is None:
        raise BenchmarkError("managed Linux sample requires per-process RSS and CPU counters")
