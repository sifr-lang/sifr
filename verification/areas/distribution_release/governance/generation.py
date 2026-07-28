"""Allocate governed release-index generations from retained canonical state."""

from __future__ import annotations

import re
from pathlib import Path

from .common import (
    GovernanceError,
    load_json_strict,
    sha256_file,
)
from .release_index import validate_release_index

SNAPSHOT_RE = re.compile(r"^channels-generation-([1-9][0-9]*)\.json$")


def allocate_next_generation(
    *,
    live_index_path: Path,
    snapshot_root: Path,
) -> int:
    """Return one greater than every live or retained canonical generation."""
    live = validate_release_index(
        load_json_strict(live_index_path, require_canonical=True)
    )
    if snapshot_root.is_symlink() or not snapshot_root.is_dir():
        raise GovernanceError("snapshot root must be an existing non-symlink directory")

    generations = {live["generation"]}
    live_snapshot_matches = False
    live_sha256 = sha256_file(live_index_path)
    for path in sorted(snapshot_root.iterdir()):
        if path.is_symlink() or not path.is_file():
            raise GovernanceError(f"unsupported generation snapshot entry: {path.name}")
        match = SNAPSHOT_RE.fullmatch(path.name)
        if match is None:
            raise GovernanceError(f"invalid generation snapshot name: {path.name}")
        named_generation = int(match.group(1))
        snapshot = validate_release_index(
            load_json_strict(path, require_canonical=True)
        )
        if snapshot["generation"] != named_generation:
            raise GovernanceError(
                f"generation snapshot name and payload disagree: {path.name}"
            )
        generations.add(named_generation)
        if (
            named_generation == live["generation"]
            and sha256_file(path) == live_sha256
        ):
            live_snapshot_matches = True
    if not live_snapshot_matches:
        raise GovernanceError(
            "live release index must equal its retained generation snapshot"
        )
    return max(generations) + 1
