#!/usr/bin/env python3
"""Refetch exact GitHub qualification uploads by immutable artifact ID."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from scripts.distribution.extract_github_artifact import (  # noqa: E402
    extract_artifact,
)
from verification.areas.distribution_release.governance.artifact_index import (  # noqa: E402
    validate_qualification_artifact_index,
)
from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError,
    load_json_strict,
)
from verification.areas.distribution_release.governance.planner import (  # noqa: E402
    verify_transported_artifacts,
)


def fetch_qualification_artifacts(
    *,
    qualification_index_path: Path,
    repository: str,
    expected_source_commit: str,
    output_root: Path,
) -> None:
    """Fetch, extract, and hash-verify the exact six governed uploads."""
    qualification = validate_qualification_artifact_index(
        load_json_strict(qualification_index_path, require_canonical=True),
        require_unexpired=True,
    )
    workflow = qualification["workflow"]
    if workflow["repository"] != repository:
        raise GovernanceError("qualification repository identity mismatch")
    if qualification["source_commit"] != expected_source_commit:
        raise GovernanceError("qualification source commit identity mismatch")
    if output_root.exists() or output_root.is_symlink():
        raise GovernanceError("artifact output must not already exist")
    if output_root.parent.is_symlink() or not output_root.parent.is_dir():
        raise GovernanceError("artifact output parent must be an existing directory")

    run_id = workflow["run_id"]
    run_attempt = workflow["run_attempt"]
    run = _gh_json(
        f"/repos/{repository}/actions/runs/{run_id}/attempts/{run_attempt}"
    )
    if (
        run.get("id") != run_id
        or run.get("run_attempt") != run_attempt
        or run.get("head_sha") != expected_source_commit
        or run.get("conclusion") != "success"
    ):
        raise GovernanceError("qualification workflow-run provenance mismatch")

    uploads: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for artifact in qualification["artifacts"]:
        uploads[artifact["workflow_artifact_id"]].append(artifact)
    if len(uploads) != 6:
        raise GovernanceError("qualification must bind exactly six uploads")

    with tempfile.TemporaryDirectory(
        prefix=".qualification-artifacts-",
        dir=output_root.parent,
    ) as temporary:
        staging = Path(temporary) / "artifacts"
        staging.mkdir()
        archives = Path(temporary) / "archives"
        archives.mkdir()
        for artifact_id, entries in sorted(uploads.items()):
            name = entries[0]["workflow_artifact_name"]
            expires_at = entries[0]["expires_at"]
            metadata = _gh_json(
                f"/repos/{repository}/actions/artifacts/{artifact_id}"
            )
            workflow_run = metadata.get("workflow_run")
            if (
                metadata.get("id") != artifact_id
                or metadata.get("name") != name
                or metadata.get("expired") is not False
                or metadata.get("expires_at") != expires_at
                or not isinstance(metadata.get("size_in_bytes"), int)
                or metadata["size_in_bytes"] < 1
                or not isinstance(workflow_run, dict)
                or workflow_run.get("id") != run_id
            ):
                raise GovernanceError(
                    f"qualification artifact provenance mismatch: {artifact_id}"
                )
            archive_path = archives / f"{artifact_id}.zip"
            _gh_to_file(
                f"/repos/{repository}/actions/artifacts/{artifact_id}/zip",
                archive_path,
                expected_bytes=metadata["size_in_bytes"],
            )
            destination = staging / name
            destination.mkdir()
            try:
                extract_artifact(
                    archive_path,
                    destination,
                    expected_uncompressed_bytes=sum(
                        artifact["size_bytes"] for artifact in entries
                    ),
                )
            except ValueError as exc:
                raise GovernanceError(
                    f"qualification artifact extraction failed: {artifact_id}: {exc}"
                ) from exc
        verify_transported_artifacts(qualification, staging)
        staging.rename(output_root)


def _gh_json(endpoint: str) -> dict[str, Any]:
    raw = _gh_bytes(endpoint)
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise GovernanceError(f"GitHub API returned invalid JSON: {endpoint}") from exc
    if not isinstance(value, dict):
        raise GovernanceError(f"GitHub API returned a non-object: {endpoint}")
    return value


def _gh_bytes(endpoint: str) -> bytes:
    result = subprocess.run(
        ["gh", "api", endpoint],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        detail = result.stderr.decode(errors="replace").strip()
        raise GovernanceError(f"GitHub API request failed: {endpoint}: {detail}")
    if not result.stdout:
        raise GovernanceError(f"GitHub API returned empty bytes: {endpoint}")
    return result.stdout


def _gh_to_file(endpoint: str, destination: Path, *, expected_bytes: int) -> None:
    with tempfile.TemporaryFile() as error_stream:
        process = subprocess.Popen(
            ["gh", "api", endpoint],
            stdout=subprocess.PIPE,
            stderr=error_stream,
        )
        assert process.stdout is not None
        written = 0
        try:
            with destination.open("xb") as output:
                while chunk := process.stdout.read(1024 * 1024):
                    written += len(chunk)
                    if written > expected_bytes:
                        process.kill()
                        raise GovernanceError(
                            f"GitHub API response exceeded declared size: {endpoint}"
                        )
                    output.write(chunk)
            returncode = process.wait()
            error_stream.seek(0)
            stderr = error_stream.read()
        except BaseException:
            if process.poll() is None:
                process.kill()
            process.wait()
            destination.unlink(missing_ok=True)
            raise
    if returncode != 0:
        destination.unlink(missing_ok=True)
        detail = stderr.decode(errors="replace").strip()
        raise GovernanceError(f"GitHub API request failed: {endpoint}: {detail}")
    if written != expected_bytes:
        destination.unlink(missing_ok=True)
        raise GovernanceError(
            f"GitHub API response size disagreed with metadata: {endpoint}"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--qualification-index", type=Path, required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--expected-source-commit", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    try:
        fetch_qualification_artifacts(
            qualification_index_path=args.qualification_index,
            repository=args.repository,
            expected_source_commit=args.expected_source_commit,
            output_root=args.out,
        )
    except GovernanceError as exc:
        parser.error(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
