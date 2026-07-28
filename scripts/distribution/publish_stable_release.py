#!/usr/bin/env python3
"""Publish or resume one exact write-once stable GitHub release."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError,
    fail,
    require_commit,
    require_enum,
    sha256_file,
    write_canonical_json,
)

STABLE_VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
REPOSITORY_RE = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")


def publish_stable_release(
    *,
    repository: str,
    version: str,
    source_commit: str,
    mode: str,
    assets_root: Path,
    notes_path: Path,
    output_path: Path,
) -> dict[str, str]:
    """Create/verify the release, upload only missing assets, then reverify."""
    if REPOSITORY_RE.fullmatch(repository) is None:
        fail("repository", "must be an owner/name repository identity")
    if STABLE_VERSION_RE.fullmatch(version) is None:
        fail("version", "must be exact stable SemVer")
    source_commit = require_commit(source_commit, "source_commit")
    mode = require_enum(mode, {"initial", "resume"}, "mode")
    if not notes_path.is_file() or notes_path.is_symlink():
        fail("notes_path", "must be a regular non-symlink file")
    if output_path.exists() or output_path.is_symlink():
        fail("output_path", "must not already exist")

    local = _local_assets(assets_root)
    release = _gh_json_allow_404(
        f"repos/{repository}/releases/tags/{version}"
    )
    tag = _gh_json_allow_404(f"repos/{repository}/git/ref/tags/{version}")
    if mode == "initial" and (release is not None or tag is not None):
        fail("mode", "initial publication requires an absent release and tag")
    if release is None and tag is not None:
        fail("remote release", "tag exists without an attributable release")
    if release is not None and tag is None:
        fail("remote release", "release exists without its exact tag")
    if release is None:
        _run_gh(
            "release",
            "create",
            version,
            "--repo",
            repository,
            "--target",
            source_commit,
            "--title",
            version,
            "--notes-file",
            str(notes_path),
        )
        release = _require_json(
            _gh_json_allow_404(
                f"repos/{repository}/releases/tags/{version}"
            ),
            "created release",
        )
        tag = _require_json(
            _gh_json_allow_404(
                f"repos/{repository}/git/ref/tags/{version}"
            ),
            "created tag",
        )

    release_id = _validate_release(
        release,
        tag=tag,
        version=version,
        source_commit=source_commit,
    )
    remote = _remote_assets(repository, release_id)
    _verify_remote_inventory(
        repository=repository,
        remote=remote,
        local=local,
        allow_missing=True,
    )
    missing = sorted(set(local).difference(remote))
    if missing:
        _run_gh(
            "release",
            "upload",
            version,
            *(str(assets_root / name) for name in missing),
            "--repo",
            repository,
        )

    final = _remote_assets(repository, release_id)
    _verify_remote_inventory(
        repository=repository,
        remote=final,
        local=local,
        allow_missing=False,
    )
    digests = {name: sha256_file(path) for name, path in sorted(local.items())}
    write_canonical_json(output_path, digests, refuse_existing=True)
    return digests


def _local_assets(root: Path) -> dict[str, Path]:
    entries = list(root.iterdir()) if root.is_dir() else []
    if not entries:
        fail("assets_root", "must contain at least one release asset")
    if any(path.is_symlink() or not path.is_file() for path in entries):
        fail("assets_root", "must contain only regular non-symlink files")
    return {path.name: path for path in entries}


def _validate_release(
    release: dict[str, Any],
    *,
    tag: dict[str, Any] | None,
    version: str,
    source_commit: str,
) -> int:
    if (
        release.get("tag_name") != version
        or release.get("target_commitish") != source_commit
        or release.get("draft") is not False
        or release.get("prerelease") is not False
    ):
        fail("remote release", "identity or immutable release policy drifted")
    release_id = release.get("id")
    if not isinstance(release_id, int) or isinstance(release_id, bool) or release_id <= 0:
        fail("remote release.id", "must be a positive integer")
    tag_value = _require_json(tag, "release tag")
    tag_object = tag_value.get("object")
    if not isinstance(tag_object, dict) or tag_object.get("sha") != source_commit:
        fail("remote release tag", "does not resolve to the exact source commit")
    return release_id


def _remote_assets(repository: str, release_id: int) -> dict[str, dict[str, Any]]:
    observed: dict[str, dict[str, Any]] = {}
    page = 1
    while True:
        value = _gh_json(
            f"repos/{repository}/releases/{release_id}/assets?per_page=100&page={page}"
        )
        if not isinstance(value, list):
            fail("remote assets", "GitHub returned a non-array asset inventory")
        if not value:
            return observed
        for asset in value:
            if not isinstance(asset, dict):
                fail("remote assets", "GitHub returned an invalid asset record")
            name = asset.get("name")
            asset_id = asset.get("id")
            if not isinstance(name, str) or not name:
                fail("remote assets", "asset name must be non-empty")
            if (
                not isinstance(asset_id, int)
                or isinstance(asset_id, bool)
                or asset_id <= 0
                or name in observed
            ):
                fail("remote assets", "asset ids and names must be unique")
            observed[name] = asset
        page += 1


def _verify_remote_inventory(
    *,
    repository: str,
    remote: dict[str, dict[str, Any]],
    local: dict[str, Path],
    allow_missing: bool,
) -> None:
    unknown = sorted(set(remote).difference(local))
    if unknown:
        fail("remote assets", f"contains unplanned asset(s): {', '.join(unknown)}")
    if not allow_missing:
        missing = sorted(set(local).difference(remote))
        if missing:
            fail("remote assets", f"is missing planned asset(s): {', '.join(missing)}")
    for name, asset in remote.items():
        asset_id = asset["id"]
        downloaded = _run_gh(
            "api",
            "-H",
            "Accept: application/octet-stream",
            f"repos/{repository}/releases/assets/{asset_id}",
            text=False,
        )
        assert isinstance(downloaded.stdout, bytes)
        if downloaded.stdout != local[name].read_bytes():
            fail("remote assets", f"published bytes drifted for {name}")


def _gh_json(endpoint: str) -> Any:
    completed = _run_gh("api", endpoint)
    assert isinstance(completed.stdout, str)
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError:
        fail("gh api", f"returned invalid JSON for {endpoint}")


def _gh_json_allow_404(endpoint: str) -> dict[str, Any] | None:
    completed = subprocess.run(
        ["gh", "api", endpoint],
        cwd=REPO_ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if completed.returncode != 0:
        if "404" in completed.stderr and "Not Found" in completed.stderr:
            return None
        fail("gh api", f"failed for {endpoint}: {completed.stderr.strip()}")
    try:
        value = json.loads(completed.stdout)
    except json.JSONDecodeError:
        fail("gh api", f"returned invalid JSON for {endpoint}")
    if not isinstance(value, dict):
        fail("gh api", f"returned a non-object for {endpoint}")
    return value


def _require_json(value: dict[str, Any] | None, location: str) -> dict[str, Any]:
    if value is None:
        fail(location, "was not found after publication")
    return value


def _run_gh(
    *args: str,
    text: bool = True,
) -> subprocess.CompletedProcess[str] | subprocess.CompletedProcess[bytes]:
    completed = subprocess.run(
        ["gh", *args],
        cwd=REPO_ROOT,
        check=False,
        text=text,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if completed.returncode != 0:
        stderr = (
            completed.stderr
            if isinstance(completed.stderr, str)
            else completed.stderr.decode(errors="replace")
        )
        fail("gh", f"command failed: {stderr.strip()}")
    return completed


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--mode", choices=("initial", "resume"), required=True)
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--notes", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        publish_stable_release(
            repository=args.repository,
            version=args.version,
            source_commit=args.source_commit,
            mode=args.mode,
            assets_root=args.assets,
            notes_path=args.notes,
            output_path=args.out,
        )
    except GovernanceError as exc:
        print(f"stable release publication failed: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
