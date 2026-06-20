"""Cargo metadata and Rust signature probe skeletons for Rust interop fixtures."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class RustBridgeProbePlan:
    fixture_id: str
    package_root: Path
    target_paths: tuple[str, ...]
    locked: bool
    offline: bool
    frozen: bool


def load_probe_plan(fixture_root: Path) -> RustBridgeProbePlan:
    """Return the future probe plan hook for a fixture root."""
    return RustBridgeProbePlan(
        fixture_id=fixture_root.name,
        package_root=fixture_root,
        target_paths=(),
        locked=True,
        offline=True,
        frozen=False,
    )
