from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


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
        repo_root=area_root.parents[1],
        area_root=area_root,
        packages_root=area_root / "packages",
        fixtures_root=area_root / "fixtures",
        reports_root=area_root / "reports",
    )
