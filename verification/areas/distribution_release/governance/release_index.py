"""Validation for the canonical governed release index."""

from __future__ import annotations

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


def validate_incident_index_mutation(
    previous_value: Any,
    proposed_value: Any,
    *,
    incident_id: str,
    affected_version: str,
    successor_version: str,
) -> None:
    previous = validate_release_index(previous_value)
    proposed = validate_release_index(proposed_value)
    validate_release_index_transition(previous, proposed)
    if previous["ga_status"] != "active" or previous["channels"].get("stable") != affected_version:
        fail("$.channels.stable", "affected version is not the live stable predecessor")
    affected = proposed["releases"].get(affected_version)
    if not isinstance(affected, dict):
        fail("$.releases", "incident mutation removed the affected release")
    if affected.get("status") != "withdrawn" or affected.get("incident_id") != incident_id:
        fail("$.releases", "incident mutation must withdraw the affected version atomically")
    successor = proposed["releases"].get(successor_version)
    if not isinstance(successor, dict) or successor.get("status") != "active":
        fail("$.releases", "incident mutation must activate the successor atomically")
    if proposed["channels"].get("stable") != successor_version:
        fail("$.channels.stable", "incident mutation must point stable at the successor")
