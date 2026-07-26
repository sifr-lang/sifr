"""Qualification artifact-index validation."""

from __future__ import annotations

from datetime import datetime
from typing import Any

from .common import (
    TARGETS,
    fail,
    require_array,
    require_artifact_id,
    require_commit,
    require_enum,
    require_exact_keys,
    require_nonempty_string,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    version_channel,
)

ARTIFACT_KINDS = {"binary-archive", "installer", "sysroot", "vsix", "report"}


def validate_qualification_artifact_index(payload: Any) -> dict[str, Any]:
    index = require_object(payload, "$")
    require_exact_keys(
        index,
        required={
            "schema_version",
            "candidate_version",
            "source_commit",
            "workflow",
            "artifacts",
        },
        location="$",
    )
    require_schema_v2(index)
    if version_channel(index["candidate_version"], "$.candidate_version") != "stable":
        fail("$.candidate_version", "must be a stable version")
    require_commit(index["source_commit"], "$.source_commit")
    workflow = require_object(index["workflow"], "$.workflow")
    require_exact_keys(
        workflow,
        required={"run_id", "run_attempt", "repository", "expires_at"},
        location="$.workflow",
    )
    require_positive_int(workflow["run_id"], "$.workflow.run_id")
    require_positive_int(workflow["run_attempt"], "$.workflow.run_attempt")
    if workflow["repository"] != "sifr-lang/sifr":
        fail("$.workflow.repository", "must be sifr-lang/sifr")
    expires_at = require_nonempty_string(workflow["expires_at"], "$.workflow.expires_at")
    try:
        parsed_expiry = datetime.fromisoformat(expires_at.replace("Z", "+00:00"))
    except ValueError:
        fail("$.workflow.expires_at", "must be an ISO-8601 timestamp")
    if parsed_expiry.tzinfo is None:
        fail("$.workflow.expires_at", "must include a timezone")

    artifacts = require_array(index["artifacts"], "$.artifacts")
    if not artifacts:
        fail("$.artifacts", "must contain transported artifact evidence")
    ids: set[str] = set()
    for position, value in enumerate(artifacts):
        location = f"$.artifacts[{position}]"
        artifact = require_object(value, location)
        require_exact_keys(
            artifact,
            required={"id", "kind", "name", "sha256", "size_bytes", "workflow_artifact_id"},
            optional={"target"},
            location=location,
        )
        artifact_id = require_artifact_id(artifact["id"], f"{location}.id")
        if artifact_id in ids:
            fail(f"{location}.id", "must be unique")
        ids.add(artifact_id)
        artifact_kind = require_enum(
            artifact["kind"],
            ARTIFACT_KINDS,
            f"{location}.kind",
        )
        require_nonempty_string(artifact["name"], f"{location}.name")
        require_sha256(artifact["sha256"], f"{location}.sha256")
        require_positive_int(artifact["size_bytes"], f"{location}.size_bytes")
        require_positive_int(
            artifact["workflow_artifact_id"],
            f"{location}.workflow_artifact_id",
        )
        if "target" in artifact and artifact["target"] not in TARGETS:
            fail(f"{location}.target", "is not a supported target")
        if artifact_kind in {"binary-archive", "sysroot"} and "target" not in artifact:
            fail(location, f"{artifact_kind} must name its target")
    return index
