"""Measured host and execution identity for named performance references."""

from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import subprocess
from pathlib import Path
from typing import Any

from compiler_lanes import selection


def output(argv: list[str]) -> str:
    result = subprocess.run(argv, capture_output=True, text=True, check=True, timeout=30)
    return result.stdout.strip()


def file_hash(path: Path) -> str | None:
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


def linux_memory() -> dict[str, int]:
    values = {}
    for line in Path("/proc/meminfo").read_text().splitlines():
        key, value = line.split(":", 1)
        values[key] = int(value.split()[0]) * 1024
    swaps = {}
    for line in Path("/proc/vmstat").read_text().splitlines():
        key, value = line.split()
        if key in {"pswpin", "pswpout"}:
            swaps[key] = int(value)
    return {
        "total_bytes": values["MemTotal"],
        "available_bytes": values["MemAvailable"],
        "swap_used_bytes": values["SwapTotal"] - values["SwapFree"],
        **swaps,
    }


def cpu_power_policy() -> dict[str, Any]:
    if platform.system() == "Darwin":
        return {"source": "pmset", "configuration": output(["pmset", "-g", "custom"])}
    root = Path("/sys/devices/system/cpu/cpufreq")
    policies = []
    for policy in sorted(root.glob("policy*")):
        policies.append({
            field: (policy / field).read_text().strip()
            for field in (
                "affected_cpus", "scaling_driver", "scaling_governor",
                "scaling_min_freq", "scaling_max_freq",
            )
        })
    if not policies:
        raise ValueError("named reference requires measurable CPU frequency policy")
    boost_paths = (
        root / "boost",
        Path("/sys/devices/system/cpu/intel_pstate/no_turbo"),
    )
    return {
        "source": "sysfs",
        "policies": policies,
        "boost_controls": {
            str(path): path.read_text().strip() for path in boost_paths if path.is_file()
        },
    }


def host_details() -> dict[str, Any]:
    system = platform.system()
    if system == "Linux":
        records = [
            dict(
                (key.strip(), value.strip())
                for line in block.splitlines() if ":" in line
                for key, value in [line.split(":", 1)]
            )
            for block in Path("/proc/cpuinfo").read_text().strip().split("\n\n")
        ]
        models = sorted({record["model name"] for record in records})
        cores = {(record.get("physical id"), record.get("core id")) for record in records}
        if any(None in core for core in cores):
            raise ValueError("reference host does not expose physical CPU topology")
        memory = linux_memory()
        physical_cores = len(cores)
        available_cpus = len(os.sched_getaffinity(0))
        os_release = platform.freedesktop_os_release()
        os_version = os_release.get("PRETTY_NAME", "")
    elif system == "Darwin":
        models = [output(["sysctl", "-n", "machdep.cpu.brand_string"])]
        physical_cores = int(output(["sysctl", "-n", "hw.physicalcpu"]))
        available_cpus = int(output(["sysctl", "-n", "hw.logicalcpu"]))
        memory = {"total_bytes": int(output(["sysctl", "-n", "hw.memsize"]))}
        os_version = output(["sw_vers", "-productVersion"])
    else:
        raise ValueError(f"named reference capture is unsupported on {system}")
    return {
        "system": system,
        "os_version": os_version,
        "kernel": platform.release(),
        "architecture": platform.machine(),
        "cpu_models": models,
        "physical_cores": physical_cores,
        "logical_cpus": os.cpu_count(),
        "available_cpus": available_cpus,
        "memory_capacity_gib": math.ceil(memory["total_bytes"] / (1024 ** 3)),
        "memory": memory,
        "cpu_power_policy": cpu_power_policy(),
    }


def storage_details(path: Path) -> dict[str, str]:
    path = path.resolve()
    while not path.exists():
        path = path.parent
    if platform.system() == "Linux":
        mounts = json.loads(output(["findmnt", "-J", "-T", str(path), "-o", "FSTYPE,SOURCE"]))
        mount = mounts["filesystems"][0]
        return {"filesystem": mount["fstype"], "source": mount["source"]}
    return {
        "filesystem": output(["stat", "-f", "%T", str(path)]),
        "source": output(["df", "-P", str(path)]).splitlines()[-1].split()[0],
    }


def input_hash(repo_root: Path, manifest_path: Path) -> str:
    manifest = json.loads(manifest_path.read_text())
    paths = sorted({
        str(case[field])
        for case in manifest["cases"]
        for field in ("source_path", "project_root")
        if case.get(field)
    })
    tracked = output(["git", "-C", str(repo_root), "ls-files", "--", *paths]).splitlines()
    digest = hashlib.sha256(manifest_path.read_bytes())
    for name in sorted(tracked):
        digest.update(name.encode() + b"\0")
        digest.update((repo_root / name).read_bytes())
    return digest.hexdigest()


def execution_details(repo_root: Path, manifest_path: Path, mode: str) -> dict[str, Any]:
    target = Path(os.environ.get("CARGO_TARGET_DIR", str(repo_root / "target")))
    if not target.is_absolute():
        target = repo_root / target
    temporary = Path(os.environ.get("TMPDIR", "/tmp"))
    return {
        "rustc": output(["rustc", "--version"]),
        "cargo": output(["cargo", "--version"]),
        "python": platform.python_version(),
        "build_profile": selection()["compiler_build_profile"],
        "compiler_measurement_lane": selection()["lane"],
        "control_mode": mode,
        "cache_policy": "manifest per-case warmups",
        "cargo_jobs": os.environ.get("CARGO_BUILD_JOBS", "cargo-default"),
        "rust_test_threads": os.environ.get("RUST_TEST_THREADS", "default"),
        "build_environment": {
            key: value for key, value in sorted(os.environ.items())
            if key.startswith("CARGO_PROFILE_")
            or key in {"RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER"}
        },
        "benchmark_inputs_sha256": input_hash(repo_root, manifest_path),
        "target_storage": storage_details(target),
        "temporary_storage": storage_details(temporary),
        "cargo_manifest_sha256": file_hash(repo_root / "Cargo.toml"),
        "cargo_config_sha256": file_hash(repo_root / ".cargo/config.toml"),
        "user_cargo_config_sha256": file_hash(
            Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))) / "config.toml"
        ),
    }


def reference_identity(repo_root: Path, manifest_path: Path, mode: str) -> dict[str, Any]:
    return {
        "host": host_details(),
        "execution": execution_details(repo_root, manifest_path, mode),
    }


def comparison_mismatches(expected: dict[str, Any], actual: dict[str, Any]) -> list[str]:
    mismatches = []
    for key in (
        "system", "os_version", "kernel", "architecture", "cpu_models",
        "physical_cores", "logical_cpus", "available_cpus", "memory_capacity_gib",
        "cpu_power_policy",
    ):
        if expected["host"][key] != actual["host"][key]:
            mismatches.append(f"host.{key}")
    for key in (
        "rustc", "cargo", "python", "build_profile", "control_mode", "cargo_jobs",
        "rust_test_threads", "build_environment", "benchmark_inputs_sha256", "cache_policy",
        "target_storage", "temporary_storage",
        "user_cargo_config_sha256",
    ):
        if expected["execution"][key] != actual["execution"][key]:
            mismatches.append(f"execution.{key}")
    expected_lane = expected["execution"].get("compiler_measurement_lane", "contributor-dev")
    actual_lane = actual["execution"].get("compiler_measurement_lane", "contributor-dev")
    if expected_lane != actual_lane:
        mismatches.append("execution.compiler_measurement_lane")
    # Tracked compiler inputs, including project Cargo config (for example a
    # native grammar version), are candidate changes to measure. User Cargo
    # config and external build flags remain host configuration. Both project
    # hashes must still stay fixed within one producer invocation.
    return mismatches
