"""Effective shared-VM resources and prospective per-stage admission.

Tmpfs is charged to the same cgroup memory as resident processes. Advertised
/tmp and /dev/shm sizes never become two additional memory allowances.
"""
from __future__ import annotations

import math
import os
import shutil
from dataclasses import asdict, dataclass
from pathlib import Path

from .errors import VerificationError


class ResourceError(VerificationError):
    """Required work cannot be admitted under observed capacity."""

    def __init__(self, message: str, classification: str = "admission"):
        super().__init__(message)
        self.classification = classification


@dataclass(frozen=True)
class Resources:
    affinity_cpus: int
    effective_cpus: float
    memory_limit_bytes: int
    memory_available_bytes: int
    disk_available_bytes: int
    tmpfs_available_bytes: dict[str, int]
    cgroup_limits: list[dict]
    diagnostics: dict[str, str]

    def identity(self) -> dict:
        return {"affinity_cpus": self.affinity_cpus, "effective_cpus": self.effective_cpus,
                "memory_limit_bytes": self.memory_limit_bytes, "cgroup_limits": [
                    {key: row[key] for key in ("path", "cpu_max", "memory_max")} for row in self.cgroup_limits]}


def memory_information(path: Path) -> dict[str, int]:
    values = {}
    for line in path.read_text().splitlines():
        key, raw = line.split(":", 1)
        parts = raw.split()
        value = int(parts[0])
        values[key] = value * (1024 if len(parts) > 1 and parts[1] == "kB" else 1)
    return values


def own_cgroup(root: Path, membership: Path = Path("/proc/self/cgroup")) -> Path:
    """Resolve the namespace-visible unified hierarchy, rejecting ambiguous input."""
    entries = [line.split(":", 2) for line in membership.read_text().splitlines()]
    unified = [parts[2] for parts in entries if len(parts) == 3 and parts[:2] == ["0", ""]]
    if len(unified) != 1 or not unified[0].startswith("/"):
        raise ValueError("a single cgroup v2 membership is required")
    relative = Path(unified[0].lstrip("/"))
    if ".." in relative.parts:
        raise ValueError("cgroup membership escapes its mount")
    return root / relative


def discover(*, disk_path: Path, cgroup_root: Path = Path("/sys/fs/cgroup"),
             cgroup_path: Path | None = None, meminfo: Path = Path("/proc/meminfo"),
             affinity: int | None = None, tmpfs_paths: tuple[Path, ...] = (Path("/tmp"), Path("/dev/shm"))) -> Resources:
    try:
        cpus = affinity if affinity is not None else len(os.sched_getaffinity(0))
        if isinstance(cpus, bool) or not isinstance(cpus, int) or cpus <= 0:
            raise ValueError("CPU affinity must expose positive capacity")
        memory = memory_information(meminfo)
        limit, available = memory["MemTotal"], memory["MemAvailable"]
        effective = float(cpus)
        root = cgroup_root.resolve()
        current = (cgroup_path or own_cgroup(cgroup_root)).resolve()
        if not current.is_relative_to(root):
            raise ValueError("cgroup path escapes its mount")
        rows, diagnostics = [], {}
        while True:
            cpu_path, memory_path = current / "cpu.max", current / "memory.max"
            cpu_raw = cpu_path.read_text().strip() if cpu_path.is_file() else "unavailable"
            memory_raw = memory_path.read_text().strip() if memory_path.is_file() else "unavailable"
            if cpu_raw != "unavailable":
                quota, period = cpu_raw.split()
                if int(period) <= 0:
                    raise ValueError("invalid cgroup CPU period")
                if quota != "max":
                    if int(quota) <= 0:
                        raise ValueError("invalid cgroup CPU quota")
                    effective = min(effective, int(quota) / int(period))
            if memory_raw not in {"unavailable", "max"}:
                capacity = int(memory_raw)
                usage = int((current / "memory.current").read_text().strip())
                if capacity <= 0 or usage < 0:
                    raise ValueError("invalid cgroup memory accounting")
                limit = min(limit, capacity)
                available = min(available, max(0, capacity - usage))
            rows.append({"path": str(current), "cpu_max": cpu_raw, "memory_max": memory_raw})
            for name in ("cpu.stat", "memory.events", "cpu.pressure", "memory.pressure", "io.pressure"):
                path = current / name
                if path.is_file():
                    diagnostics[str(path)] = path.read_text().strip()
            if current == root:
                break
            current = current.parent
        if effective <= 0 or limit <= 0 or available < 0:
            raise ValueError("invalid effective resource capacity")
        if not any(row["cpu_max"] != "unavailable" for row in rows) or not any(
                row["memory_max"] != "unavailable" for row in rows):
            raise ValueError("cgroup CPU and memory accounting are required on the cloud route")
        tmpfs = {str(path): shutil.disk_usage(path).free for path in tmpfs_paths if path.exists()}
        return Resources(cpus, effective, limit, min(limit, available),
                         shutil.disk_usage(disk_path).free, tmpfs, rows, diagnostics)
    except (OSError, ValueError, KeyError, AttributeError) as error:
        raise ResourceError(f"effective resource discovery unavailable: {error}", "unavailable") from error


def admit(resources: Resources, requirements: dict) -> dict:
    fields = {"disk_growth_bytes", "retained_copy_bytes", "disk_reserve_bytes",
              "memory_peak_bytes", "tmpfs_growth_bytes", "memory_reserve_bytes"}
    if set(requirements) != fields or any(isinstance(value, bool) or not isinstance(value, int) or value < 0
                                          for value in requirements.values()):
        raise ResourceError("stage resource requirements must be explicit nonnegative integer bytes", "unavailable")
    disk = sum(requirements[key] for key in ("disk_growth_bytes", "retained_copy_bytes", "disk_reserve_bytes"))
    memory = sum(requirements[key] for key in ("memory_peak_bytes", "tmpfs_growth_bytes", "memory_reserve_bytes"))
    if disk > resources.disk_available_bytes:
        raise ResourceError(f"disk admission: required={disk} available={resources.disk_available_bytes}", "enospc")
    if memory > resources.memory_available_bytes:
        raise ResourceError(f"shared resident/tmpfs memory admission: required={memory} available={resources.memory_available_bytes}", "admission")
    return {"resources": asdict(resources), "requirements": requirements,
            "disk_admitted_bytes": disk, "memory_admitted_bytes": memory}


def worker_limit(resources: Resources, requested: int) -> int:
    if isinstance(requested, bool) or not isinstance(requested, int) or requested <= 0:
        raise ResourceError("requested workers must be positive integers", "unavailable")
    return min(requested, max(1, math.floor(resources.effective_cpus)))
