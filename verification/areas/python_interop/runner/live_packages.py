from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from env import RunnerPaths, cargo_env_for_repo_manifest
from example_packages import ordinary_python_api_policy_violations
from live_case_config import LIVE_CASES, LiveCase

BUILD_TIMEOUT_SECONDS = 900
EXECUTION_TIMEOUT_SECONDS = 120


@dataclass(frozen=True)
class BuiltLiveBinary:
    case: LiveCase
    package_root: Path
    binary_path: Path
    binary_sha256: str
    build_command: tuple[str, ...]


def validate_live_source_presence(paths: RunnerPaths) -> list[dict[str, Any]]:
    return [_validate_case_sources(paths, case) for case in LIVE_CASES.values()]


def build_live_binaries(
    paths: RunnerPaths,
) -> tuple[list[dict[str, Any]], dict[str, BuiltLiveBinary]]:
    checks: list[dict[str, Any]] = []
    binaries: dict[str, BuiltLiveBinary] = {}
    for case in LIVE_CASES.values():
        validation = _validate_case_sources(paths, case)
        if validation["status"] != "pass":
            checks.append(validation)
            continue
        check, binary = _build_case(paths, case)
        checks.append(check)
        if binary is not None:
            binaries[case.case_id] = binary
    return checks, binaries


def execute_live_binary(
    binary: BuiltLiveBinary,
    environment: dict[str, str],
) -> dict[str, Any]:
    runtime_environment = os.environ.copy()
    runtime_environment.update(environment)
    command = [str(binary.binary_path)]
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            command,
            cwd=binary.package_root,
            env=runtime_environment,
            text=True,
            capture_output=True,
            check=False,
            timeout=EXECUTION_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        raise RuntimeError(
            f"compiled Sifr binary timed out after {EXECUTION_TIMEOUT_SECONDS}s: "
            f"stdout={(error.stdout or '')[-2000:]!r} stderr={(error.stderr or '')[-2000:]!r}"
        ) from error
    elapsed_ms = round((time.perf_counter() - started) * 1000.0)
    marker_observed = binary.case.stdout_marker in proc.stdout
    if proc.returncode != 0 or not marker_observed:
        raise RuntimeError(
            f"compiled Sifr binary failed: exit={proc.returncode} marker={marker_observed} "
            f"stdout={proc.stdout[-2000:]!r} stderr={proc.stderr[-2000:]!r}"
        )
    return {
        "execution_model": "compiled-sifr-binary",
        "binary_built": True,
        "binary_executed": True,
        "binary_path": str(binary.binary_path),
        "binary_sha256": binary.binary_sha256,
        "execution_command": command,
        "exit_code": proc.returncode,
        "stdout_marker": binary.case.stdout_marker,
        "stdout_marker_observed": marker_observed,
        "stdout": proc.stdout[-4000:],
        "stderr": proc.stderr[-4000:],
        "binary_elapsed_ms": elapsed_ms,
    }


def _validate_case_sources(paths: RunnerPaths, case: LiveCase) -> dict[str, Any]:
    source_path = paths.fixtures_root / case.relative_source
    bridge_path = paths.fixtures_root / case.bridge_file
    reason = None
    if not source_path.is_file():
        reason = "missing source fixture"
    elif not bridge_path.is_file():
        reason = "missing hermetic bridge"
    else:
        violations = ordinary_python_api_policy_violations(source_path.read_text(encoding="utf-8"))
        if violations:
            reason = f"live source violates declaration-first policy: {sorted(violations)}"
    if reason is not None:
        return {
            "id": case.case_id,
            "status": "fail",
            "sifr_source": case.relative_source,
            "bridge_file": case.bridge_file,
            "reason": reason,
        }
    return {
        "id": case.case_id,
        "status": "pass",
        "sifr_source": case.relative_source,
        "bridge_file": case.bridge_file,
        "check": "declaration-first-source-and-bridge",
    }


def _build_case(
    paths: RunnerPaths,
    case: LiveCase,
) -> tuple[dict[str, Any], BuiltLiveBinary | None]:
    package_root = _prepare_live_package(paths, case)
    output_root = package_root / "build"
    command = (
        "cargo",
        "run",
        "-q",
        "-p",
        "sifr",
        "--manifest-path",
        str(paths.repo_root / "Cargo.toml"),
        "--",
        "build",
        "src/main.sifr",
        "--output",
        str(output_root),
        "--quiet",
    )
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            command,
            cwd=package_root,
            env=cargo_env_for_repo_manifest(paths.repo_root),
            text=True,
            capture_output=True,
            check=False,
            timeout=BUILD_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        return (
            {
                "id": case.case_id,
                "status": "fail",
                "sifr_source": case.relative_source,
                "reason": f"compiled Sifr binary build timed out after {BUILD_TIMEOUT_SECONDS}s",
                "build_command": list(command),
                "stdout": (error.stdout or "")[-4000:],
                "stderr": (error.stderr or "")[-4000:],
            },
            None,
        )
    elapsed_ms = round((time.perf_counter() - started) * 1000.0)
    binary_name = "sifr_output.exe" if os.name == "nt" else "sifr_output"
    binary_path = output_root / "sifr_output" / "target" / "release" / binary_name
    if proc.returncode != 0 or not binary_path.is_file():
        return (
            {
                "id": case.case_id,
                "status": "fail",
                "sifr_source": case.relative_source,
                "reason": "failed to build compiled Sifr live binary",
                "build_command": list(command),
                "build_exit_code": proc.returncode,
                "elapsed_ms": elapsed_ms,
                "stdout": proc.stdout[-4000:],
                "stderr": proc.stderr[-4000:],
            },
            None,
        )
    with binary_path.open("rb") as binary_file:
        digest = hashlib.file_digest(binary_file, "sha256").hexdigest()
    binary = BuiltLiveBinary(
        case=case,
        package_root=package_root,
        binary_path=binary_path,
        binary_sha256=digest,
        build_command=command,
    )
    return (
        {
            "id": case.case_id,
            "status": "pass",
            "sifr_source": case.relative_source,
            "bridge_file": case.bridge_file,
            "check": "compiled-sifr-binary-built",
            "execution_model": "compiled-sifr-binary",
            "trusted_import_roots": list(case.import_roots),
            "trusted_native_roots": list(case.native_roots),
            "binary_built": True,
            "binary_path": str(binary_path),
            "binary_sha256": digest,
            "build_command": list(command),
            "build_exit_code": proc.returncode,
            "elapsed_ms": elapsed_ms,
        },
        binary,
    )


def _prepare_live_package(paths: RunnerPaths, case: LiveCase) -> Path:
    package_root = (
        paths.repo_root
        / "target"
        / "verification"
        / "areas"
        / "python_interop"
        / "live_binaries"
        / case.case_id
    )
    if package_root.exists():
        shutil.rmtree(package_root)
    source_root = package_root / "src"
    bridge_root = source_root / "python_bridges"
    bridge_root.mkdir(parents=True)
    shutil.copy2(paths.fixtures_root / case.relative_source, source_root / "main.sifr")
    shutil.copy2(paths.fixtures_root / case.bridge_file, bridge_root / Path(case.bridge_file).name)
    (source_root / "lib.rs").write_text(
        "// Cargo metadata marker; the executable source is src/main.sifr.\n",
        encoding="utf-8",
    )
    (package_root / "Cargo.toml").write_text(_cargo_manifest(case), encoding="utf-8")
    (package_root / "sifr.toml").write_text(_sifr_manifest(case), encoding="utf-8")
    area_venv = paths.area_root / ".venv"
    if not area_venv.exists():
        raise SystemExit(
            "python interop live binaries require the area uv environment; "
            "run through the locked python-interop project"
        )
    (package_root / ".venv").symlink_to(area_venv, target_is_directory=True)
    (package_root / "pyproject.toml").symlink_to(paths.area_root / "pyproject.toml")
    (package_root / "uv.lock").symlink_to(paths.area_root / "uv.lock")
    return package_root


def _cargo_manifest(case: LiveCase) -> str:
    return "\n".join(
        [
            "[package]",
            f'name = "sifr-python-interop-live-{case.case_id}"',
            'version = "0.1.0"',
            'edition = "2024"',
            "",
            "[package.metadata.sifr]",
            'manifest = "sifr.toml"',
            "",
            "[workspace]",
            "",
        ]
    )


def _sifr_manifest(case: LiveCase) -> str:
    roots = ", ".join(f'"{root}"' for root in case.import_roots)
    native_roots = ", ".join(f'"{root}"' for root in case.native_roots)
    return "\n".join(
        [
            "[package]",
            f'name = "python_interop_live_{case.case_id}"',
            'edition = "2026"',
            'sifr-version = ">=0.3,<0.4"',
            "",
            "[source]",
            'root = "src"',
            "",
            "[python]",
            'venv = ".venv"',
            'pyproject = "pyproject.toml"',
            'lock = "uv.lock"',
            f"requires-imports = [{roots}]",
            "",
            "[trust]",
            f"python = [{roots}]",
            f"python-native = [{native_roots}]",
            "",
        ]
    )
