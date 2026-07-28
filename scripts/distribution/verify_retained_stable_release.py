#!/usr/bin/env python3
"""Verify an immutable retained stable release against its approved evidence."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from verification.areas.distribution_release.governance.artifact_index import (  # noqa: E402
    validate_qualification_artifact_index,
)
from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError,
    fail,
    load_json_strict,
    require_commit,
    sha256_file,
    write_canonical_json,
)
from verification.areas.distribution_release.governance.release_plan import (  # noqa: E402
    validate_release_plan,
)


def verify_retained_release(
    *,
    plan_path: Path,
    qualification_path: Path,
    assets_root: Path,
    release_metadata_path: Path,
    tag_commit: str,
) -> dict[str, str]:
    """Validate exact release identity, inventory, and downloaded bytes."""
    plan = validate_release_plan(
        load_json_strict(plan_path, require_canonical=True)
    )
    qualification = validate_qualification_artifact_index(
        load_json_strict(qualification_path, require_canonical=True)
    )
    if (
        qualification["candidate_version"] != plan["version"]
        or qualification["source_commit"] != plan["source_commit"]
        or plan["qualification_artifact_index"]["sha256"]
        != sha256_file(qualification_path)
    ):
        fail("qualification_path", "does not bind the approved retained plan")
    metadata = _load_object(release_metadata_path)
    if (
        metadata.get("tagName") != plan["version"]
        or metadata.get("targetCommitish") != plan["source_commit"]
        or metadata.get("isDraft") is not False
        or metadata.get("isPrerelease") is not False
        or require_commit(tag_commit, "tag_commit") != plan["source_commit"]
    ):
        fail("release_metadata_path", "retained release identity drifted")
    expected = {
        artifact["name"]: artifact["sha256"]
        for artifact in qualification["artifacts"]
    }
    expected["stable-release-plan.json"] = sha256_file(plan_path)
    entries = list(assets_root.iterdir()) if assets_root.is_dir() else []
    if any(path.is_symlink() or not path.is_file() for path in entries):
        fail("assets_root", "must contain only regular non-symlink files")
    if {path.name for path in entries} != set(expected):
        fail("assets_root", "does not contain the exact retained asset inventory")
    actual = {path.name: sha256_file(path) for path in entries}
    if actual != expected:
        fail("assets_root", "retained release bytes drifted")
    return dict(sorted(actual.items()))


def _load_object(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise GovernanceError(f"{path}: invalid release metadata: {exc}") from exc
    if not isinstance(value, dict):
        fail(str(path), "must contain a JSON object")
    return value


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--qualification", type=Path, required=True)
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--release-metadata", type=Path, required=True)
    parser.add_argument("--tag-commit", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = verify_retained_release(
            plan_path=args.plan,
            qualification_path=args.qualification,
            assets_root=args.assets,
            release_metadata_path=args.release_metadata,
            tag_commit=args.tag_commit,
        )
        write_canonical_json(args.out, result, refuse_existing=True)
    except GovernanceError as exc:
        print(f"retained stable release verification failed: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
