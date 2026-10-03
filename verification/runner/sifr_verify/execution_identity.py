"""Conservative identity for correctness evidence, including runtime bytes."""
from __future__ import annotations

import hashlib
import importlib.metadata
import json
import os
import platform
import re
import shutil
import subprocess
import sys
from pathlib import Path

from .errors import VerificationError
from .fixture_inventory import inventory
from .paths import REPO_ROOT

RUNTIME_ENVIRONMENT = (
    "CARGO_BUILD_JOBS", "CARGO_INCREMENTAL", "CARGO_PROFILE_DEV_DEBUG", "CARGO_TARGET_DIR", "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
    "RUSTUP_TOOLCHAIN", "RAYON_NUM_THREADS", "OMP_NUM_THREADS", "SIFR_SYSROOT",
    "SIFR_RELEASE_VERSION", "SYNTAQLITE_SQLITE_VERSION", "WASI_SDK_PATH",
    "PATH", "CC", "CXX", "LIBCLANG_PATH", "LD_LIBRARY_PATH",
)


class EvidenceError(VerificationError):
    """Evidence is incomplete, stale, or bound to different inputs."""


def digest(value: object) -> str:
    raw = json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()
    return hashlib.sha256(raw).hexdigest()


def artifact_identity(path: Path) -> dict:
    requested = path.absolute()
    actual = requested.resolve(strict=True)
    if not actual.is_file():
        raise EvidenceError(f"artifact is not a file: {requested}")
    hasher = hashlib.sha256()
    with actual.open("rb") as stream:
        before = os.fstat(stream.fileno())
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
        after = os.fstat(stream.fileno())
    stable = lambda info: (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)
    if (stable(before) != stable(after) or stable(after) != stable(actual.stat())
            or requested.resolve(strict=True) != actual):
        raise EvidenceError(f"artifact changed while establishing identity: {requested}")
    return {"requested_path": str(requested), "path": str(actual),
            "sha256": hasher.hexdigest(), "size_bytes": after.st_size}


def version(*command: str, env: dict[str, str] | None = None) -> str:
    try:
        result = subprocess.run(command, env=env, check=True, capture_output=True,
                                text=True, timeout=30)
    except (OSError, subprocess.SubprocessError) as error:
        raise EvidenceError(f"cannot establish tool identity: {command[0]}") from error
    value = result.stdout.strip()
    if not value:
        raise EvidenceError(f"tool emitted no identity: {command[0]}")
    return value


def dependency_identity() -> list[dict]:
    """Bind installed dependency versions and declared installed bytes."""
    result = []
    for distribution in importlib.metadata.distributions():
        files = []
        for declared in distribution.files or []:
            path = Path(distribution.locate_file(declared))
            if path.is_file():
                files.append(artifact_identity(path))
        result.append({"name": distribution.metadata["Name"], "version": distribution.version,
                       "files": sorted(files, key=lambda row: row["requested_path"])})
    return sorted(result, key=lambda row: (row["name"], row["version"]))


def runtime_identity(env: dict[str, str], *, root: Path = REPO_ROOT) -> dict:
    cargo_home = Path(env.get("CARGO_HOME", str(Path.home() / ".cargo")))
    configurations = []
    # Hash configuration bytes, never publish their contents or read credentials.
    for directory in (root / ".cargo", cargo_home):
        for name in ("config", "config.toml"):
            path = directory / name
            if path.exists():
                configurations.append(artifact_identity(path))
    tool_bytes = {}
    for tool in ("cargo", "rustc", "uv"):
        located = shutil.which(tool, path=env.get("PATH"))
        if not located:
            raise EvidenceError(f"tool executable unavailable: {tool}")
        tool_bytes[tool] = artifact_identity(Path(located))
    # rustup shims do not identify the selected compiler toolchain bytes.
    if shutil.which("rustup", path=env.get("PATH")):
        for tool in ("cargo", "rustc"):
            tool_bytes["selected_" + tool] = artifact_identity(Path(version("rustup", "which", tool, env=env)))
    return {
        "interpreter": artifact_identity(Path(sys.executable)),
        "python": sys.version,
        "dependencies": dependency_identity(),
        "import_search_paths": list(sys.path),
        "tools": {"cargo": version("cargo", "--version", env=env),
                  "rustc": version("rustc", "-vV", env=env),
                  "uv": version("uv", "--version", env=env)},
        "tool_bytes": tool_bytes,
        "platform": {"system": platform.system(), "machine": platform.machine(),
                     "release": platform.release(), "libc": list(platform.libc_ver())},
        "cargo_configurations": configurations,
        "settings": {name: digest(env[name]) for name in
                     (*RUNTIME_ENVIRONMENT, "PYTHONPATH", "VIRTUAL_ENV", "UV_PROJECT_ENVIRONMENT") if name in env},
        "local_environment_presence": {name: bool(env.get(name)) for name in
                                       ("VIRTUAL_ENV", "UV_PROJECT_ENVIRONMENT", "PYTHONPATH")},
    }


def execution_key(*, selection: dict, commands: list[list[str]], artifacts: list[Path],
                  producer: dict, services: dict[str, str],
                  env: dict[str, str], root: Path = REPO_ROOT,
                  resource_identity: dict | None = None) -> dict:
    if not producer or not commands or any(not command for command in commands):
        raise EvidenceError("execution identity requires producer and actual commands")
    if any(not isinstance(key, str) or not key or not isinstance(value, str) or not value
           for key, value in services.items()):
        raise EvidenceError("service identities must be explicit nonempty versions")
    source = inventory(root)
    try:
        commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root,
                                         text=True, timeout=30).strip()
    except (OSError, subprocess.SubprocessError) as error:
        raise EvidenceError("cannot establish observed source commit") from error
    inputs = {"source": source, "selection": selection, "commands": commands,
              "runtime": runtime_identity(env, root=root),
              "artifacts": [artifact_identity(path) for path in artifacts],
              "services": services, "producer": producer,
              "resource_identity": resource_identity or {}}
    # Commit is deliberately conservative. A future cross-commit protocol must
    # prove every relevant dependency (including generated Git Cargo sources).
    inputs["observed_commit"] = commit
    return {"schema_version": 1, "observed_commit": commit,
            "input_digest": digest(inputs), "inputs": inputs}


def validate_key(key: dict) -> None:
    if (not isinstance(key, dict) or set(key) != {"schema_version", "observed_commit", "input_digest", "inputs"}
            or key.get("schema_version") != 1 or not isinstance(key.get("inputs"), dict)):
        raise EvidenceError("invalid execution identity document")
    required = {"source", "selection", "commands", "runtime", "artifacts", "services",
                "producer", "resource_identity", "observed_commit"}
    if set(key["inputs"]) != required or not re.fullmatch(r"[0-9a-f]{40}", key["observed_commit"]):
        raise EvidenceError("execution identity requires complete source/runtime/provenance bindings")
    source = key["inputs"]["source"]
    if (not isinstance(source, dict) or source.get("schema_version") != 1 or
            not isinstance(source.get("inputs"), list) or
            source.get("input_digest") != digest(source["inputs"])):
        raise EvidenceError("source input inventory is missing or has drifted")
    selection = key["inputs"].get("selection")
    if not isinstance(selection, dict) or not isinstance(selection.get("required_kinds"), dict):
        raise EvidenceError("selection must declare execution requirements")
    if not key["inputs"].get("producer") or not key["inputs"].get("runtime") or not key["inputs"].get("commands"):
        raise EvidenceError("empty runtime, producer or command binding")
    if key["observed_commit"] != key["inputs"].get("observed_commit"):
        raise EvidenceError("observed commit binding differs")
    if key["input_digest"] != digest(key["inputs"]):
        raise EvidenceError("execution identity digest differs")
