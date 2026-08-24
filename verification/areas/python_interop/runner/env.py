from __future__ import annotations

import os
import platform
import re
import sysconfig
from dataclasses import dataclass
from pathlib import Path

import tomllib


@dataclass(frozen=True)
class RunnerPaths:
    repo_root: Path
    area_root: Path
    packages_root: Path
    fixtures_root: Path
    reports_root: Path


def discover_paths() -> RunnerPaths:
    area_root = Path(__file__).resolve().parents[1]
    return RunnerPaths(
        repo_root=area_root.parents[2],
        area_root=area_root,
        packages_root=area_root / "packages",
        fixtures_root=area_root / "fixtures",
        reports_root=area_root / "reports",
    )


def cargo_env_for_repo_manifest(repo_root: Path) -> dict[str, str]:
    env = os.environ.copy()
    target_dir = env.get("CARGO_TARGET_DIR")
    if target_dir is not None:
        target_path = Path(target_dir)
        if not target_path.is_absolute():
            env["CARGO_TARGET_DIR"] = str((repo_root / target_path).resolve())
    return env


def require_canonical_python(area_root: Path) -> str:
    with (area_root / "pyproject.toml").open("rb") as project_file:
        requirement = tomllib.load(project_file)["project"]["requires-python"]
    if re.fullmatch(r"==\d+\.\d+\.\d+", requirement) is None:
        raise SystemExit("python interop requires-python must be an exact == pin")
    expected = requirement.removeprefix("==")
    implementation = platform.python_implementation()
    version = platform.python_version()
    if implementation != "CPython" or version != expected:
        raise SystemExit(
            f"python interop requires CPython {expected}, found {implementation} {version}"
        )
    if sysconfig_free_threaded():
        raise SystemExit(
            "python interop requires the canonical GIL-enabled CPython build"
        )
    return version


def sysconfig_free_threaded() -> bool:
    return bool(sysconfig.get_config_var("Py_GIL_DISABLED"))
