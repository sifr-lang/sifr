"""Resolve protected compiler paths from the existing producer authorities."""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

from .paths import REPO_ROOT


def producer(name: str, root: Path = REPO_ROOT):
    area = root / "verification/areas/sysroot_release"
    spec = importlib.util.spec_from_file_location("_validation_" + name, area / (name + ".py"))
    if spec is None or spec.loader is None:
        raise ValueError(f"cannot load sysroot producer {name}")
    module = importlib.util.module_from_spec(spec)
    previous = sys.path.copy()
    try:
        sys.path.insert(0, str(area))
        spec.loader.exec_module(module)
    finally:
        sys.path[:] = previous
    return module


def protected_compilers(root: Path, env: dict[str, str]) -> dict[str, Path]:
    source, package = producer("source_build"), producer("package_build")
    return {"source-cargo-target": source.source_build_configuration(root, env)[2],
            "cargo-target": package.package_compiler_path(root, env, package.host_target(env))}
