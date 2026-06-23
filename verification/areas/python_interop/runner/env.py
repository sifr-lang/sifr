from __future__ import annotations

import os
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
