"""Validation for the canonical governed release index."""

from __future__ import annotations

from copy import deepcopy
from typing import Any

from .common import (
    CHANNELS,
    TARGETS,
    fail,
    require_commit,
    require_exact_keys,
    require_enum,
    require_incident_id,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    preview_version_order,
    version_channel,
)

INDEX_REQUIRED = {"schema_version", "generation", "ga_status", "channels", "releases"}
RELEASE_REQUIRED = {"channel", "status", "source_commit", "installer_sha256", "targets"}


def validate_release_index(payload: Any) -> dict[str, Any]:
    index = require_object(payload, "$")
    require_exact_keys(index, required=INDEX_REQUIRED, location="$")
    require_schema_v2(index)
    require_positive_int(index["generation"], "$.generation")
    ga_status = require_enum(index["ga_status"], {"preview", "active"}, "$.ga_status")

    channels = require_object(index["channels"], "$.channels")
    expected_channel_keys = {"alpha", "beta"}
    if ga_status == "active":
        expected_channel_keys.add("stable")
    require_exact_keys(channels, required=expected_channel_keys, location="$.channels")

    releases = require_object(index["releases"], "$.releases")
    if len(releases) < 2:
        fail("$.releases", "must contain at least the alpha and beta releases")
    for version, release in releases.items():
        channel = version_channel(version, f"$.releases[{version!r}]")
        validate_release_record(release, version=version, expected_channel=channel)

    for channel, version in channels.items():
        if channel not in CHANNELS:
            fail("$.channels", f"unsupported channel: {channel}")
        actual_class = version_channel(version, f"$.channels.{channel}")
        if actual_class != channel:
            fail(f"$.channels.{channel}", f"points at {actual_class} version {version}")
        release = releases.get(version)
        if not isinstance(release, dict):
            fail(f"$.channels.{channel}", f"points at missing release {version}")
        if release.get("channel") != channel or release.get("status") != "active":
            fail(f"$.channels.{channel}", "must point at an active matching release")
    return index


def validate_release_record(
    payload: Any,
    *,
    version: str,
    expected_channel: str | None = None,
) -> dict[str, Any]:
    release = require_object(payload, f"release[{version}]")
    status = release.get("status")
    optional = {"incident_id"} if status == "withdrawn" else set()
    require_exact_keys(
        release,
        required=RELEASE_REQUIRED,
        optional=optional,
        location=f"release[{version}]",
    )
    channel = require_enum(
        release["channel"],
        set(CHANNELS),
        f"release[{version}].channel",
    )
    version_class = version_channel(version, f"release[{version}]")
    if channel != version_class or (expected_channel is not None and channel != expected_channel):
        fail(f"release[{version}].channel", "does not match the version class")
    status = require_enum(
        status,
        {"active", "withdrawn"},
        f"release[{version}].status",
    )
    if status == "withdrawn":
        require_incident_id(release.get("incident_id"), f"release[{version}].incident_id")
    elif "incident_id" in release:
        fail(f"release[{version}].incident_id", "is allowed only on withdrawn releases")
    require_commit(release["source_commit"], f"release[{version}].source_commit")
    require_sha256(release["installer_sha256"], f"release[{version}].installer_sha256")

    targets = require_object(release["targets"], f"release[{version}].targets")
    require_exact_keys(targets, required=set(TARGETS), location=f"release[{version}].targets")
    for target, evidence_value in targets.items():
        evidence = require_object(evidence_value, f"release[{version}].targets.{target}")
        require_exact_keys(
            evidence,
            required={"artifact_sha256", "sysroot_content_sha256"},
            location=f"release[{version}].targets.{target}",
        )
        require_sha256(
            evidence["artifact_sha256"],
            f"release[{version}].targets.{target}.artifact_sha256",
        )
        require_sha256(
            evidence["sysroot_content_sha256"],
            f"release[{version}].targets.{target}.sysroot_content_sha256",
        )
    return release


def validate_release_index_transition(previous_value: Any, proposed_value: Any) -> None:
    previous = validate_release_index(previous_value)
    proposed = validate_release_index(proposed_value)
    if proposed["generation"] <= previous["generation"]:
        fail("$.generation", "must increase monotonically")
    if previous["ga_status"] == "active" and proposed["ga_status"] != "active":
        fail("$.ga_status", "active cannot transition back to preview")


def propose_preview_release(
    current_value: Any,
    *,
    channel: str,
    version: str,
    release_value: Any,
    proposed_generation: int | None = None,
) -> dict[str, Any]:
    current = validate_release_index(current_value)
    channel = require_enum(channel, {"alpha", "beta"}, "channel")
    release = validate_release_record(
        release_value,
        version=version,
        expected_channel=channel,
    )
    current_version = current["channels"][channel]
    if preview_version_order(version, "new release version") <= preview_version_order(
        current_version,
        f"current {channel} version",
    ):
        fail(
            "new release version",
            f"refusing to move {channel} backward from {current_version} to {version}",
        )
    if version in current["releases"]:
        fail("$.releases", f"release record already exists: {version}")
    generation = current["generation"] + 1
    if proposed_generation is not None:
        if not isinstance(proposed_generation, int) or isinstance(proposed_generation, bool):
            fail("proposed_generation", "must be an integer")
        if proposed_generation <= current["generation"]:
            fail("proposed_generation", "must exceed the current generation")
        generation = proposed_generation
    proposed = {
        **current,
        "generation": generation,
        "channels": {**current["channels"], channel: version},
        "releases": {**current["releases"], version: release},
    }
    proposed["channels"] = dict(sorted(proposed["channels"].items()))
    proposed["releases"] = dict(sorted(proposed["releases"].items()))
    validate_release_index_transition(current, proposed)
    return proposed


def propose_stable_release(
    current_value: Any,
    *,
    transition: str,
    version: str,
    release_value: Any,
    expected_predecessor: str | None,
    proposed_generation: int,
) -> dict[str, Any]:
    """Return a GA activation or normal stable release-index mutation."""
    current = validate_release_index(current_value)
    transition = require_enum(
        transition,
        {"ga-activation", "normal"},
        "transition",
    )
    release = validate_release_record(
        release_value,
        version=version,
        expected_channel="stable",
    )
    if release["status"] != "active" or "incident_id" in release:
        fail("release", "stable publication requires an active qualified release")
    if version in current["releases"]:
        fail("$.releases", f"release record already exists: {version}")
    _require_incident_generation(current, proposed_generation)

    live_stable = current["channels"].get("stable")
    if transition == "ga-activation":
        if current["ga_status"] != "preview" or live_stable is not None:
            fail("transition", "ga-activation requires preview metadata without stable")
        if expected_predecessor is not None:
            fail("expected_predecessor", "ga-activation cannot name a predecessor")
    else:
        if current["ga_status"] != "active" or live_stable is None:
            fail("transition", "normal requires an active stable predecessor")
        if expected_predecessor != live_stable:
            fail("expected_predecessor", "does not equal the live stable version")
        if _stable_order(version, "version") <= _stable_order(live_stable, "live stable"):
            fail("version", "normal stable publication must move forward")

    proposed = deepcopy(current)
    proposed["generation"] = proposed_generation
    proposed["ga_status"] = "active"
    proposed["channels"]["stable"] = version
    proposed["channels"] = dict(sorted(proposed["channels"].items()))
    proposed["releases"][version] = deepcopy(release)
    proposed["releases"] = dict(sorted(proposed["releases"].items()))
    validate_release_index_transition(current, proposed)

    for channel in ("alpha", "beta"):
        if proposed["channels"][channel] != current["channels"][channel]:
            fail(f"$.channels.{channel}", "stable publication must preserve preview channels")
    for retained_version, retained_release in current["releases"].items():
        if proposed["releases"].get(retained_version) != retained_release:
            fail(
                f"$.releases.{retained_version}",
                "stable publication must preserve retained release bytes",
            )
    return proposed


def validate_incident_index_mutation(
    previous_value: Any,
    proposed_value: Any,
    *,
    operation: str,
    incident_id: str,
    affected_version: str,
    successor_version: str,
) -> None:
    previous = validate_release_index(previous_value)
    proposed = validate_release_index(proposed_value)
    validate_release_index_transition(previous, proposed)
    operation = require_enum(
        operation,
        {"rollback", "incident-roll-forward"},
        "operation",
    )
    if previous["ga_status"] != "active" or previous["channels"].get("stable") != affected_version:
        fail("$.channels.stable", "affected version is not the live stable predecessor")
    if proposed["ga_status"] != "active":
        fail("$.ga_status", "incident mutation must preserve active GA status")
    for channel, version in previous["channels"].items():
        if channel != "stable" and proposed["channels"].get(channel) != version:
            fail(f"$.channels.{channel}", "incident mutation may change only stable")
    affected = proposed["releases"].get(affected_version)
    if not isinstance(affected, dict):
        fail("$.releases", "incident mutation removed the affected release")
    if affected.get("status") != "withdrawn" or affected.get("incident_id") != incident_id:
        fail("$.releases", "incident mutation must withdraw the affected version atomically")
    expected_affected = {
        **previous["releases"][affected_version],
        "status": "withdrawn",
        "incident_id": incident_id,
    }
    if affected != expected_affected:
        fail(
            f"$.releases.{affected_version}",
            "incident mutation may only add withdrawal status and incident identity",
        )
    successor = proposed["releases"].get(successor_version)
    if not isinstance(successor, dict) or successor.get("status") != "active":
        fail("$.releases", "incident mutation must activate the successor atomically")
    if proposed["channels"].get("stable") != successor_version:
        fail("$.channels.stable", "incident mutation must point stable at the successor")
    prior_versions = set(previous["releases"])
    proposed_versions = set(proposed["releases"])
    if operation == "rollback":
        if successor_version not in prior_versions or proposed_versions != prior_versions:
            fail("$.releases", "rollback must reuse one retained active release")
    else:
        if successor_version in prior_versions or proposed_versions != prior_versions | {successor_version}:
            fail("$.releases", "incident roll-forward must add exactly the qualified successor")
    for version, release in previous["releases"].items():
        if version == affected_version:
            continue
        if proposed["releases"].get(version) != release:
            fail(f"$.releases.{version}", "incident mutation must preserve retained release bytes")


def propose_rollback(
    current_value: Any,
    *,
    incident_id: str,
    affected_version: str,
    target_version: str,
    proposed_generation: int,
) -> dict[str, Any]:
    """Return the immutable-index rollback mutation for an approved request."""
    current = validate_release_index(current_value)
    require_incident_id(incident_id, "incident_id")
    _require_incident_generation(current, proposed_generation)
    if current["ga_status"] != "active" or current["channels"].get("stable") != affected_version:
        fail("affected_version", "must equal the live active stable version")
    target = current["releases"].get(target_version)
    if (
        target_version == affected_version
        or not isinstance(target, dict)
        or target.get("channel") != "stable"
        or target.get("status") != "active"
    ):
        fail("target_version", "must name a distinct retained active stable release")
    proposed = deepcopy(current)
    proposed["generation"] = proposed_generation
    proposed["channels"]["stable"] = target_version
    proposed["releases"][affected_version]["status"] = "withdrawn"
    proposed["releases"][affected_version]["incident_id"] = incident_id
    proposed["releases"] = dict(sorted(proposed["releases"].items()))
    validate_incident_index_mutation(
        current,
        proposed,
        operation="rollback",
        incident_id=incident_id,
        affected_version=affected_version,
        successor_version=target_version,
    )
    return proposed


def propose_incident_roll_forward(
    current_value: Any,
    *,
    incident_id: str,
    affected_version: str,
    successor_version: str,
    successor_release: Any,
    proposed_generation: int,
) -> dict[str, Any]:
    """Return the atomic withdrawal plus qualified-successor index mutation."""
    current = validate_release_index(current_value)
    require_incident_id(incident_id, "incident_id")
    _require_incident_generation(current, proposed_generation)
    if current["ga_status"] != "active" or current["channels"].get("stable") != affected_version:
        fail("affected_version", "must equal the live active stable version")
    if successor_version in current["releases"]:
        fail("successor_version", "must be a new immutable stable release")
    if _stable_order(successor_version, "successor_version") <= _stable_order(
        affected_version,
        "affected_version",
    ):
        fail("successor_version", "incident roll-forward must move to a newer stable version")
    successor = validate_release_record(
        successor_release,
        version=successor_version,
        expected_channel="stable",
    )
    if successor["status"] != "active" or "incident_id" in successor:
        fail("successor_release", "must be an active qualified release")
    proposed = deepcopy(current)
    proposed["generation"] = proposed_generation
    proposed["channels"]["stable"] = successor_version
    proposed["releases"][affected_version]["status"] = "withdrawn"
    proposed["releases"][affected_version]["incident_id"] = incident_id
    proposed["releases"][successor_version] = deepcopy(successor)
    proposed["releases"] = dict(sorted(proposed["releases"].items()))
    validate_incident_index_mutation(
        current,
        proposed,
        operation="incident-roll-forward",
        incident_id=incident_id,
        affected_version=affected_version,
        successor_version=successor_version,
    )
    return proposed


def _require_incident_generation(current: dict[str, Any], proposed_generation: int) -> None:
    if (
        not isinstance(proposed_generation, int)
        or isinstance(proposed_generation, bool)
        or proposed_generation <= current["generation"]
    ):
        fail("proposed_generation", "must be an integer greater than the live generation")


def _stable_order(version: Any, location: str) -> tuple[int, int, int]:
    if version_channel(version, location) != "stable":
        fail(location, "must be an exact stable version")
    assert isinstance(version, str)
    major, minor, patch = (int(part) for part in version.split("."))
    return major, minor, patch
