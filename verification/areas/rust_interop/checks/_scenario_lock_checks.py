"""Lockfile validation helpers for Rust-interop scenario packages."""

from __future__ import annotations

import tomllib
from functools import lru_cache
from pathlib import Path
from typing import Any


@lru_cache(maxsize=1)
def _load_root_lock(path: Path) -> tuple[dict[str, Any] | None, str | None]:
    if not path.is_file():
        return None, None
    try:
        return tomllib.loads(path.read_text(encoding="utf-8")), None
    except tomllib.TOMLDecodeError as error:
        return None, str(error)


def read_root_lock(
    failures: list[str],
    fixture_id: str,
    path: Path,
) -> dict[str, Any] | None:
    """Read the immutable repository lock once per checker process."""
    lock, error = _load_root_lock(path)
    if error is not None:
        failures.append(
            f"{fixture_id}: repository root/{path.name} is not valid TOML: {error}"
        )
    return lock


def require_root_lock_subset(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    scenario_lock: dict[str, Any],
    root_lock: dict[str, Any],
) -> None:
    """Require every external scenario package identity in the root lock."""
    root_packages = {
        (str(package.get("name")), str(package.get("version")))
        for package in root_lock.get("package", [])
        if isinstance(package, dict) and package.get("source")
    }
    for package in scenario_lock.get("package", []):
        if not isinstance(package, dict) or not package.get("source"):
            continue
        identity = (str(package.get("name")), str(package.get("version")))
        if identity not in root_packages:
            failures.append(
                f"{fixture_id}: {raw_path}/Cargo.lock package "
                f"{identity[0]} {identity[1]} is not present in root Cargo.lock"
            )
