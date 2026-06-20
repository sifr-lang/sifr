"""Bridge projection check skeletons for Rust interop fixtures."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class BridgeProjectionCheck:
    fixture_id: str
    managed_files: tuple[Path, ...]
    user_owned_files: tuple[Path, ...]
    bridge_version: int


def load_projection_check(fixture_root: Path) -> BridgeProjectionCheck:
    """Return the future projection ownership check for a fixture root."""
    return BridgeProjectionCheck(
        fixture_id=fixture_root.name,
        managed_files=(),
        user_owned_files=(),
        bridge_version=1,
    )
