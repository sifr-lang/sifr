"""Qualification artifact-index validation."""

from __future__ import annotations

from datetime import datetime, timezone
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

ARTIFACT_KINDS = {
    "binary-archive",
    "checksum",
    "checksums",
    "installer",
    "report",
    "sysroot",
    "vsix",
}
TARGET_KINDS = {"binary-archive", "checksum", "sysroot"}
SINGLETON_KINDS = {"checksums", "installer", "vsix"}
EXPECTED_ARTIFACT_IDS = {
    "installer",
    "checksums",
    "vsix",
    "editor-qualification-report",
    *{
        f"{kind}-{target}"
        for target in TARGETS
        for kind in (
            "binary-archive",
            "checksum",
            "sysroot",
            "qualification-report",
        )
    },
}


def validate_qualification_artifact_index(
    payload: Any,
    *,
    require_unexpired: bool = False,
    now: datetime | None = None,
) -> dict[str, Any]:
    index = require_object(payload, "$")
    require_exact_keys(
        index,
        required={
            "schema_version",
            "candidate_version",
            "source_commit",
            "submodules",
            "workflow",
            "artifacts",
        },
        location="$",
    )
    require_schema_v2(index)
    if version_channel(index["candidate_version"], "$.candidate_version") != "stable":
        fail("$.candidate_version", "must be a stable version")
    require_commit(index["source_commit"], "$.source_commit")
    submodules = require_object(index["submodules"], "$.submodules")
    if not submodules:
        fail("$.submodules", "must contain recursive submodule identities")
    for path, commit in submodules.items():
        require_nonempty_string(path, "$.submodules key")
        require_commit(commit, f"$.submodules.{path}")
    workflow = require_object(index["workflow"], "$.workflow")
    require_exact_keys(
        workflow,
        required={
            "run_id",
            "run_attempt",
            "repository",
            "retention_days",
            "overwrite",
            "expires_at",
        },
        location="$.workflow",
    )
    require_positive_int(workflow["run_id"], "$.workflow.run_id")
    require_positive_int(workflow["run_attempt"], "$.workflow.run_attempt")
    if workflow["repository"] != "sifr-lang/sifr":
        fail("$.workflow.repository", "must be sifr-lang/sifr")
    if workflow["retention_days"] != 30:
        fail("$.workflow.retention_days", "must be exactly 30")
    if workflow["overwrite"] is not False:
        fail("$.workflow.overwrite", "must be false")
    expires_at = require_nonempty_string(
        workflow["expires_at"], "$.workflow.expires_at"
    )
    try:
        parsed_expiry = datetime.fromisoformat(expires_at.replace("Z", "+00:00"))
    except ValueError:
        fail("$.workflow.expires_at", "must be an ISO-8601 timestamp")
    if parsed_expiry.tzinfo is None:
        fail("$.workflow.expires_at", "must include a timezone")
    if require_unexpired:
        current = now or datetime.now(timezone.utc)
        if current.tzinfo is None:
            fail("$.workflow.expires_at", "comparison time must include a timezone")
        if parsed_expiry <= current:
            fail("$.workflow.expires_at", "qualification artifacts have expired")

    artifacts = require_array(index["artifacts"], "$.artifacts")
    if not artifacts:
        fail("$.artifacts", "must contain transported artifact evidence")
    ids: set[str] = set()
    target_coverage = {kind: set() for kind in TARGET_KINDS}
    singleton_counts = {kind: 0 for kind in SINGLETON_KINDS}
    report_ids: set[str] = set()
    for position, value in enumerate(artifacts):
        location = f"$.artifacts[{position}]"
        artifact = require_object(value, location)
        require_exact_keys(
            artifact,
            required={
                "id",
                "kind",
                "name",
                "sha256",
                "size_bytes",
                "workflow_artifact_id",
                "workflow_artifact_name",
                "expires_at",
            },
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
        name = require_nonempty_string(artifact["name"], f"{location}.name")
        require_sha256(artifact["sha256"], f"{location}.sha256")
        require_positive_int(artifact["size_bytes"], f"{location}.size_bytes")
        require_positive_int(
            artifact["workflow_artifact_id"],
            f"{location}.workflow_artifact_id",
        )
        workflow_name = require_nonempty_string(
            artifact["workflow_artifact_name"],
            f"{location}.workflow_artifact_name",
        )
        if "/" in workflow_name or workflow_name in {".", ".."}:
            fail(
                f"{location}.workflow_artifact_name",
                "must be a single governed upload name",
            )
        artifact_expiry = require_nonempty_string(
            artifact["expires_at"],
            f"{location}.expires_at",
        )
        try:
            parsed_artifact_expiry = datetime.fromisoformat(
                artifact_expiry.replace("Z", "+00:00")
            )
        except ValueError:
            fail(f"{location}.expires_at", "must be an ISO-8601 timestamp")
        if parsed_artifact_expiry.tzinfo is None:
            fail(f"{location}.expires_at", "must include a timezone")
        if parsed_artifact_expiry < parsed_expiry:
            fail(
                f"{location}.expires_at",
                "must not expire before the workflow qualification boundary",
            )
        expected_prefix = (
            f"sifr-stable-candidate-{index['candidate_version']}-"
            f"{index['source_commit']}-"
        )
        if not workflow_name.startswith(expected_prefix):
            fail(
                f"{location}.workflow_artifact_name",
                "must bind the candidate version and exact source commit",
            )
        (
            expected_kind,
            expected_target,
            expected_suffix,
            expected_name,
        ) = artifact_contract(
            artifact_id,
            version=index["candidate_version"],
        )
        if artifact_kind != expected_kind:
            fail(f"{location}.kind", f"must be {expected_kind} for {artifact_id}")
        if workflow_name != f"{expected_prefix}{expected_suffix}":
            fail(
                f"{location}.workflow_artifact_name",
                f"must use the governed {expected_suffix} upload",
            )
        if expected_name is not None and name != expected_name:
            fail(f"{location}.name", f"must be {expected_name}")
        if artifact_id == "vsix" and not name.endswith(".vsix"):
            fail(f"{location}.name", "must be the transported VSIX")
        target = artifact.get("target")
        if target is not None and target not in TARGETS:
            fail(f"{location}.target", "is not a supported target")
        if artifact_kind in TARGET_KINDS and target is None:
            fail(location, f"{artifact_kind} must name its target")
        if artifact_kind not in TARGET_KINDS and target is not None:
            fail(location, f"{artifact_kind} must not name a target")
        if target != expected_target:
            fail(f"{location}.target", f"does not match artifact id {artifact_id}")
        if target is not None:
            if not workflow_name.endswith(f"-{target}"):
                fail(
                    f"{location}.workflow_artifact_name",
                    "target artifact must use its target-qualified upload name",
                )
            target_coverage[artifact_kind].add(target)
        elif artifact_kind in SINGLETON_KINDS:
            singleton_counts[artifact_kind] += 1
        elif artifact_kind == "report":
            report_ids.add(artifact_id)
        if "/" in name or name in {".", ".."}:
            fail(f"{location}.name", "must be a single transported file name")
    if ids != EXPECTED_ARTIFACT_IDS:
        fail(
            "$.artifacts",
            "must contain the exact governed qualification artifact identifiers",
        )
    for kind, observed in target_coverage.items():
        if observed != set(TARGETS):
            missing = sorted(set(TARGETS).difference(observed))
            fail("$.artifacts", f"{kind} is missing target(s): {', '.join(missing)}")
    for kind, count in singleton_counts.items():
        if count != 1:
            fail("$.artifacts", f"must contain exactly one {kind} artifact")
    expected_report_ids = {
        "editor-qualification-report",
        *{f"qualification-report-{target}" for target in TARGETS},
    }
    if report_ids != expected_report_ids:
        fail(
            "$.artifacts",
            "must contain exactly one report for every target plus editor qualification evidence",
        )
    return index


def artifact_contract(
    artifact_id: str,
    *,
    version: str,
) -> tuple[str, str | None, str, str | None]:
    for target in TARGETS:
        archive = f"sifr-{version}-{target}.tar.gz"
        contracts = {
            f"binary-archive-{target}": (
                "binary-archive",
                target,
                target,
                archive,
            ),
            f"checksum-{target}": (
                "checksum",
                target,
                target,
                f"{archive}.sha256",
            ),
            f"sysroot-{target}": (
                "sysroot",
                target,
                target,
                f"sifr-{version}-{target}-sysroot.tar.gz",
            ),
            f"qualification-report-{target}": (
                "report",
                None,
                target,
                f"qualification-{target}.json",
            ),
        }
        if artifact_id in contracts:
            return contracts[artifact_id]
    singleton_contracts = {
        "installer": ("installer", None, "assemble", f"sifr-installer-{version}"),
        "checksums": ("checksums", None, "assemble", "checksums.txt"),
        "vsix": ("vsix", None, "editor", None),
        "editor-qualification-report": (
            "report",
            None,
            "editor",
            "qualification-editor.json",
        ),
    }
    if artifact_id in singleton_contracts:
        return singleton_contracts[artifact_id]
    fail("$.artifacts", f"unsupported qualification artifact id: {artifact_id}")
