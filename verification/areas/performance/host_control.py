"""Controlled-host admission and telemetry for performance measurements."""

from __future__ import annotations

import os
import platform
import statistics
import subprocess
import tempfile
import threading
import time
from pathlib import Path
from typing import Any, Callable


MAX_NORMALIZED_LOAD = 0.85
MAX_CALIBRATION_CV = 0.12
DEFAULT_QUIET_SNAPSHOTS = 3
DEFAULT_QUIET_INTERVAL_SECONDS = 1.0
MONITOR_INTERVAL_SECONDS = 5.0


class HostControlError(Exception):
    """Raised when the host cannot provide a controlled measurement window."""


def capture_host_snapshot(*, include_calibration: bool = True) -> dict[str, Any]:
    logical_cpus = os.cpu_count() or 1
    try:
        load_1m, load_5m, load_15m = os.getloadavg()
    except OSError:
        load_1m = load_5m = load_15m = 0.0
    thermal = thermal_state()
    power = power_state()
    frequency = cpu_frequency_state(include_calibration=include_calibration)
    return {
        "captured_at_unix": round(time.time(), 3),
        "logical_cpus": logical_cpus,
        "load_average": {
            "one_minute": round(load_1m, 3),
            "five_minutes": round(load_5m, 3),
            "fifteen_minutes": round(load_15m, 3),
            "one_minute_per_logical_cpu": round(load_1m / logical_cpus, 6),
        },
        "thermal": thermal,
        "power": power,
        "cpu_frequency_behavior": frequency,
        "competing_processes": competing_processes(),
        "memory_pressure": memory_pressure_state(),
    }


def evaluate_snapshot(snapshot: dict[str, Any], *, enforce_load: bool) -> list[str]:
    reasons: list[str] = []
    competitors = snapshot.get("competing_processes", [])
    if competitors:
        reasons.append("competing-build-process")
    thermal = snapshot.get("thermal", {})
    if thermal.get("status") not in {"nominal", "unavailable"}:
        reasons.append("thermal-pressure")
    power = snapshot.get("power", {})
    if power.get("required") and power.get("source") != "ac":
        reasons.append("not-on-ac-power")
    frequency = snapshot.get("cpu_frequency_behavior", {})
    calibration_cv = frequency.get("calibration_cv")
    if isinstance(calibration_cv, int | float) and calibration_cv > MAX_CALIBRATION_CV:
        reasons.append("unstable-frequency-proxy")
    if enforce_load:
        normalized_load = snapshot.get("load_average", {}).get(
            "one_minute_per_logical_cpu"
        )
        if (
            isinstance(normalized_load, int | float)
            and normalized_load > MAX_NORMALIZED_LOAD
        ):
            reasons.append("host-load")
    return sorted(set(reasons))


def wait_for_controlled_host(
    timeout_seconds: float,
    *,
    snapshot_fn: Callable[..., dict[str, Any]] = capture_host_snapshot,
    sleep_fn: Callable[[float], None] = time.sleep,
    quiet_snapshots: int = DEFAULT_QUIET_SNAPSHOTS,
    interval_seconds: float = DEFAULT_QUIET_INTERVAL_SECONDS,
) -> dict[str, Any]:
    deadline = time.monotonic() + timeout_seconds
    consecutive: list[dict[str, Any]] = []
    observations: list[dict[str, Any]] = []
    while True:
        snapshot = snapshot_fn(include_calibration=True)
        reasons = evaluate_snapshot(snapshot, enforce_load=True)
        observation = {"snapshot": snapshot, "rejection_reasons": reasons}
        observations.append(observation)
        if reasons:
            consecutive.clear()
        else:
            consecutive.append(snapshot)
            if len(consecutive) >= quiet_snapshots:
                return {
                    "status": "controlled",
                    "policy": controlled_policy(),
                    "accepted_snapshots": consecutive,
                    "observation_count": len(observations),
                    "rejected_observation_count": sum(
                        1 for item in observations if item["rejection_reasons"]
                    ),
                    "recent_rejected_observations": [
                        item for item in observations if item["rejection_reasons"]
                    ][-10:],
                }
        if time.monotonic() >= deadline:
            last_reasons = reasons or ["quiet-window-not-established"]
            raise HostControlError(
                "controlled host admission timed out after "
                f"{timeout_seconds:.0f}s: {', '.join(last_reasons)}"
            )
        sleep_fn(interval_seconds)


def controlled_policy() -> dict[str, Any]:
    return {
        "quiet_snapshots": DEFAULT_QUIET_SNAPSHOTS,
        "quiet_interval_seconds": DEFAULT_QUIET_INTERVAL_SECONDS,
        "max_one_minute_load_per_logical_cpu": MAX_NORMALIZED_LOAD,
        "max_frequency_proxy_cv": MAX_CALIBRATION_CV,
        "requires_ac_power_on_macos": True,
        "rejects_competing_build_processes": True,
        "rejects_thermal_pressure": True,
    }


class HostActivityMonitor:
    """Record host pressure that appears while one benchmark case is running."""

    def __init__(self, interval_seconds: float = MONITOR_INTERVAL_SECONDS) -> None:
        self._interval_seconds = interval_seconds
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None
        self.snapshots: list[dict[str, Any]] = []

    def __enter__(self) -> HostActivityMonitor:
        self.snapshots.append(capture_host_snapshot(include_calibration=False))
        self._thread = threading.Thread(
            target=self._run, name="sifr-performance-host-monitor", daemon=True
        )
        self._thread.start()
        return self

    def __exit__(self, _exc_type: object, _exc: object, _traceback: object) -> None:
        self._stop.set()
        if self._thread is not None:
            self._thread.join(timeout=max(1.0, self._interval_seconds * 2.0))
        self.snapshots.append(capture_host_snapshot(include_calibration=False))

    def _run(self) -> None:
        while not self._stop.wait(self._interval_seconds):
            self.snapshots.append(capture_host_snapshot(include_calibration=False))

    def rejection_reasons(self) -> list[str]:
        reasons: set[str] = set()
        for snapshot in self.snapshots:
            reasons.update(evaluate_snapshot(snapshot, enforce_load=False))
        return sorted(reasons)


def cache_state(
    repo_root: Path, cargo_debug_dir: Path, helper_names: list[str]
) -> dict[str, Any]:
    artifact_root = Path(tempfile.gettempdir()) / "sifr_generated_artifact_cache"
    artifact_entries = 0
    if artifact_root.is_dir():
        for namespace in artifact_root.iterdir():
            if namespace.is_dir():
                artifact_entries += sum(
                    1 for entry in namespace.iterdir() if entry.is_dir()
                )
    helpers = {
        name: cargo_debug_dir.joinpath(executable_name(name)).is_file()
        for name in helper_names
    }
    return {
        "cargo_debug_dir": str(cargo_debug_dir),
        "helper_binaries": helpers,
        "helper_cache": "warm" if helpers and all(helpers.values()) else "cold",
        "generated_artifact_cache_root": str(artifact_root),
        "generated_artifact_entries": artifact_entries,
        "cargo_lock_present": repo_root.joinpath("Cargo.lock").is_file(),
    }


def cpu_frequency_state(*, include_calibration: bool) -> dict[str, Any]:
    direct = direct_cpu_frequencies_khz()
    samples = calibration_samples_ms() if include_calibration else []
    calibration_cv = coefficient_variation(samples) if samples else None
    return {
        "source": "scaling_cur_freq" if direct else "calibration-throughput-proxy",
        "frequencies_khz": direct,
        "calibration_samples_ms": samples,
        "calibration_cv": calibration_cv,
        "direct_frequency_available": bool(direct),
    }


def direct_cpu_frequencies_khz() -> list[int]:
    values: list[int] = []
    for path in sorted(
        Path("/sys/devices/system/cpu").glob("cpu*/cpufreq/scaling_cur_freq")
    ):
        try:
            value = int(path.read_text(encoding="utf-8").strip())
        except (OSError, ValueError):
            continue
        if value > 0:
            values.append(value)
    return values


def calibration_samples_ms() -> list[float]:
    samples: list[float] = []
    for _ in range(3):
        value = 0x1234ABCD
        started = time.perf_counter()
        for index in range(250_000):
            value = ((value * 1_664_525) + index + 1_013_904_223) & 0xFFFFFFFF
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        if value < 0:
            raise AssertionError("unreachable calibration state")
        samples.append(round(elapsed_ms, 3))
    return samples


def coefficient_variation(samples: list[float]) -> float:
    if not samples:
        return 0.0
    mean = statistics.mean(samples)
    if mean == 0:
        return 0.0
    return round(statistics.pstdev(samples) / mean, 6)


def thermal_state() -> dict[str, Any]:
    if platform.system() == "Darwin":
        output = command_output(["pmset", "-g", "therm"])
        lowered = output.lower()
        status_lines = [line for line in lowered.splitlines() if line.strip()]
        pressured = any("no " not in line for line in status_lines)
        if "cpu_speed_limit" in lowered and "cpu_speed_limit\t100" not in lowered:
            pressured = True
        return {
            "status": "pressured" if pressured else "nominal",
            "source": "pmset-therm",
            "summary": "no-warning" if not pressured else "warning-recorded",
        }
    temperatures = []
    for path in sorted(Path("/sys/class/thermal").glob("thermal_zone*/temp")):
        try:
            temperatures.append(int(path.read_text(encoding="utf-8").strip()) / 1000.0)
        except (OSError, ValueError):
            continue
    return {
        "status": "pressured"
        if temperatures and max(temperatures) >= 90.0
        else "nominal",
        "source": "sysfs-thermal" if temperatures else "unavailable",
        "max_celsius": round(max(temperatures), 1) if temperatures else None,
    }


def power_state() -> dict[str, Any]:
    if platform.system() != "Darwin":
        return {"source": "not-applicable", "required": False}
    output = command_output(["pmset", "-g", "batt"])
    return {
        "source": "ac"
        if "AC Power" in output or "AC attached" in output
        else "battery",
        "required": True,
    }


def memory_pressure_state() -> dict[str, Any]:
    if platform.system() == "Darwin":
        output = command_output(["vm_stat"])
        values: dict[str, int] = {}
        for line in output.splitlines():
            if ":" not in line:
                continue
            key, raw_value = line.split(":", maxsplit=1)
            digits = raw_value.strip().rstrip(".")
            if digits.isdigit() and key in {
                "Pageins",
                "Pageouts",
                "Swapins",
                "Swapouts",
            }:
                values[key.lower()] = int(digits)
        return {"source": "vm_stat", **values}
    return {"source": "unavailable"}


def competing_processes() -> list[dict[str, Any]]:
    output = command_output(["ps", "-axo", "pid=,ppid=,comm=,args="])
    rows: list[tuple[int, int, str, str]] = []
    for line in output.splitlines():
        parts = line.strip().split(maxsplit=3)
        if len(parts) < 4 or not parts[0].isdigit() or not parts[1].isdigit():
            continue
        rows.append((int(parts[0]), int(parts[1]), parts[2], parts[3]))
    excluded = related_process_ids(rows, os.getpid())
    competitors = []
    for pid, _parent, command, args in rows:
        if pid in excluded:
            continue
        category = process_category(command, args)
        if category is not None:
            competitors.append({"pid": pid, "category": category})
    return competitors


def process_category(command: str, args: str) -> str | None:
    argument_tokens = args.lower().split()
    executables = {Path(command).name.lower()}
    if argument_tokens:
        # macOS truncates ps(1)'s comm column, while args still begins with the
        # exact executable path. Only inspect argv[0] so argument text cannot
        # turn an unrelated shell/editor process into a false competitor.
        executables.add(Path(argument_tokens[0]).name)
    build_executables = executables & {"cargo", "rustc"}
    if build_executables:
        return sorted(build_executables)[0]
    if executables & {"sifr", "frontend_query_bench"}:
        return "benchmark"
    if executables & {"git", "git-index-pack"}:
        if "git-index-pack" in executables or any(
            token in {"clone", "fetch", "index-pack", "submodule"}
            for token in argument_tokens[1:3]
        ):
            return "git"
    if any(executable.startswith("python") for executable in executables):
        script_names = {Path(token).name for token in argument_tokens}
        if "run_benchmarks.py" in script_names or "lsp_query_bench.py" in script_names:
            return "benchmark"
        if "-m" in argument_tokens and "sifr_verify" in argument_tokens:
            return "sifr_verify"
    return None


def related_process_ids(
    rows: list[tuple[int, int, str, str]], current_pid: int
) -> set[int]:
    parents = {pid: parent for pid, parent, _command, _args in rows}
    children: dict[int, list[int]] = {}
    for pid, parent, _command, _args in rows:
        children.setdefault(parent, []).append(pid)
    related = {current_pid}
    cursor = current_pid
    while cursor in parents and parents[cursor] > 0 and parents[cursor] not in related:
        cursor = parents[cursor]
        related.add(cursor)
    pending = [current_pid]
    while pending:
        parent = pending.pop()
        for child in children.get(parent, []):
            if child not in related:
                related.add(child)
                pending.append(child)
    return related


def command_output(argv: list[str]) -> str:
    try:
        completed = subprocess.run(
            argv, text=True, capture_output=True, timeout=5, check=False
        )
    except (OSError, subprocess.TimeoutExpired):
        return ""
    return (completed.stdout or completed.stderr).strip()


def executable_name(name: str) -> str:
    return f"{name}{'.exe' if platform.system() == 'Windows' else ''}"


def run_self_test() -> None:
    nominal = {
        "load_average": {"one_minute_per_logical_cpu": 0.1},
        "thermal": {"status": "nominal"},
        "power": {"source": "ac", "required": True},
        "cpu_frequency_behavior": {"calibration_cv": 0.01},
        "competing_processes": [],
    }
    if evaluate_snapshot(nominal, enforce_load=True):
        raise HostControlError("host control self-test rejected a nominal snapshot")
    pressured = dict(nominal)
    pressured["competing_processes"] = [{"pid": 42}]
    if evaluate_snapshot(pressured, enforce_load=True) != ["competing-build-process"]:
        raise HostControlError("host control self-test did not reject competing work")
    sequence = iter([pressured, nominal, nominal, nominal])
    admission = wait_for_controlled_host(
        1.0,
        snapshot_fn=lambda **_kwargs: next(sequence),
        sleep_fn=lambda _seconds: None,
    )
    if admission["status"] != "controlled" or admission["observation_count"] != 4:
        raise HostControlError(
            "host control self-test did not require a complete quiet window"
        )
    if coefficient_variation([10.0, 10.0, 10.0]) != 0.0:
        raise HostControlError("host control calibration CV self-test failed")
    if (
        process_category("/bin/zsh", "/bin/zsh -c rg cargo run_benchmarks.py")
        is not None
    ):
        raise HostControlError(
            "host control self-test classified argument text as an executable"
        )
    if process_category("/usr/bin/cargo", "cargo test") != "cargo":
        raise HostControlError("host control self-test missed a Cargo process")
    if (
        process_category(
            "/Users/example",
            "/Users/example/.rustup/toolchains/stable/bin/cargo check --quiet",
        )
        != "cargo"
    ):
        raise HostControlError(
            "host control self-test missed Cargo with a truncated comm column"
        )
    if (
        process_category(
            "/usr/bin/python3",
            "python3 verification/areas/performance/run_benchmarks.py",
        )
        != "benchmark"
    ):
        raise HostControlError("host control self-test missed the benchmark producer")
