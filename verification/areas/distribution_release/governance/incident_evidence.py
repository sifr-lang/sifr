"""Evidence-only incident commit validation."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path, PurePosixPath
from typing import Any

from .common import (
    GovernanceError,
    canonical_json_bytes,
    require_incident_id,
    sha256_bytes,
)
from .incident import validate_incident_request


def validate_incident_evidence_commit(
    *,
    repository: Path,
    base: str,
    head: str,
    request_path: str,
    evidence_path: str,
) -> dict[str, Any]:
    """Require an additive commit range containing exactly request + evidence."""
    repository = repository.resolve()
    request_relative = _validate_relative_path(request_path)
    evidence_relative = _validate_relative_path(evidence_path)
    if request_relative.name != "stable-incident-request.json":
        raise GovernanceError("incident request path must end in stable-incident-request.json")
    if evidence_relative.name != "withdrawal-evidence.txt":
        raise GovernanceError("incident evidence path must end in withdrawal-evidence.txt")
    if request_relative.parent != evidence_relative.parent:
        raise GovernanceError("incident request and evidence must share one incident directory")
    parts = request_relative.parts
    if len(parts) != 5 or parts[:3] != ("plans", "releases", "incidents"):
        raise GovernanceError(
            "incident evidence must use plans/releases/incidents/<incident-id>/"
        )
    require_incident_id(parts[3], "incident evidence directory")

    _git(repository, "merge-base", "--is-ancestor", base, head)
    changed = _git(
        repository,
        "diff",
        "--name-status",
        "--no-renames",
        base,
        head,
    ).decode()
    expected = {request_relative.as_posix(), evidence_relative.as_posix()}
    observed: set[str] = set()
    for line in changed.splitlines():
        fields = line.split("\t")
        if len(fields) != 2 or fields[0] != "A":
            raise GovernanceError("incident evidence commit may contain only added files")
        observed.add(fields[1])
    if observed != expected:
        raise GovernanceError("incident evidence commit must add exactly request and withdrawal evidence")

    request_bytes = _git(repository, "show", f"{head}:{request_relative.as_posix()}")
    evidence_bytes = _git(repository, "show", f"{head}:{evidence_relative.as_posix()}")
    if not evidence_bytes:
        raise GovernanceError("withdrawal evidence must not be empty")
    request = _load_canonical_bytes(request_bytes)
    validate_incident_request(request)
    if request["incident_id"] != parts[3]:
        raise GovernanceError("incident directory does not match request incident_id")
    if request["withdrawal"]["evidence_sha256"] != sha256_bytes(evidence_bytes):
        raise GovernanceError("withdrawal evidence bytes do not match the request digest")
    return request


def _validate_relative_path(value: str) -> PurePosixPath:
    path = PurePosixPath(value)
    if (
        path.is_absolute()
        or ".." in path.parts
        or "." in path.parts
        or value != path.as_posix()
    ):
        raise GovernanceError("incident evidence paths must be normalized repository-relative paths")
    return path


def _git(repository: Path, *args: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(repository), *args],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        detail = result.stderr.decode(errors="replace").strip()
        raise GovernanceError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout


def _load_canonical_bytes(raw: bytes) -> dict[str, Any]:
    def pairs(items: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in items:
            if key in result:
                raise GovernanceError(f"incident request contains duplicate key: {key}")
            result[key] = value
        return result

    try:
        value = json.loads(raw, object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise GovernanceError(f"incident request is invalid JSON: {exc}") from exc
    if not isinstance(value, dict) or raw != canonical_json_bytes(value):
        raise GovernanceError("incident request must use canonical JSON object bytes")
    return value
