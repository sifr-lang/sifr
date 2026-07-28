"""One-time, fail-closed schema-v2 preview epoch bootstrap."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .common import (
    TARGETS,
    fail,
    load_json_strict,
    require_array,
    require_commit,
    require_enum,
    require_exact_keys,
    require_nonempty_string,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    sha256_bytes,
    sha256_file,
    version_channel,
    write_canonical_json,
)
from .release_index import validate_release_index, validate_release_record

LEGACY_INDEX_SHA256 = "71b3243925670f56dc510b8f45b6614a622f58097a0fea9492f61d20dc4bf9ef"
LEGACY_INDEX_SIZE_BYTES = 105
BOOTSTRAP_GENERATION = 1
PROTECTED_ENVIRONMENT = "stable-release"
SMOKE_IDS = {
    "dispatcher-default",
    "dispatcher-stable-rejection",
    "governance-index",
    "installed-self-update",
}


def require_opaque_legacy_identity(*, sha256: str, size_bytes: int) -> None:
    """Accept only the observed pre-epoch asset identity, never its data model."""
    if size_bytes != LEGACY_INDEX_SIZE_BYTES or sha256 != LEGACY_INDEX_SHA256:
        fail(
            "legacy channels.json",
            "does not match the one-time opaque bootstrap identity",
        )


def build_preview_epoch(
    *,
    legacy_index_sha256: str,
    legacy_index_size_bytes: int,
    alpha_wrapper: Any,
    beta_wrapper: Any,
) -> dict[str, Any]:
    require_opaque_legacy_identity(
        sha256=legacy_index_sha256,
        size_bytes=legacy_index_size_bytes,
    )
    alpha_version, alpha_release = _validate_release_wrapper(
        alpha_wrapper,
        expected_channel="alpha",
        location="alpha release",
    )
    beta_version, beta_release = _validate_release_wrapper(
        beta_wrapper,
        expected_channel="beta",
        location="beta release",
    )
    payload = {
        "schema_version": 2,
        "generation": BOOTSTRAP_GENERATION,
        "ga_status": "preview",
        "channels": {
            "alpha": alpha_version,
            "beta": beta_version,
        },
        "releases": {
            alpha_version: alpha_release,
            beta_version: beta_release,
        },
    }
    return validate_release_index(payload)


def resolve_distinct_approvers(
    approvals: Any,
    *,
    initiator: str,
    environment: str = PROTECTED_ENVIRONMENT,
) -> list[str]:
    require_nonempty_string(initiator, "initiator")
    require_nonempty_string(environment, "environment")
    normalized_initiator = initiator.casefold()
    values = require_array(approvals, "approval history")
    approved: dict[str, str] = {}
    for index, raw in enumerate(values):
        location = f"approval history[{index}]"
        review = require_object(raw, location)
        if review.get("state") != "approved":
            continue
        environments = require_array(
            review.get("environments"), f"{location}.environments"
        )
        if not any(
            isinstance(item, dict) and item.get("name") == environment
            for item in environments
        ):
            continue
        user = require_object(review.get("user"), f"{location}.user")
        login = require_nonempty_string(user.get("login"), f"{location}.user.login")
        normalized_login = login.casefold()
        if normalized_login != normalized_initiator:
            approved.setdefault(normalized_login, login)
    if not approved:
        fail(
            "approval history",
            f"requires a {environment} approval by someone other than {initiator}",
        )
    return [approved[key] for key in sorted(approved)]


def validate_bootstrap_evidence(payload: Any) -> dict[str, Any]:
    evidence = require_object(payload, "$")
    stage = evidence.get("stage")
    required = {
        "schema_version",
        "operation",
        "stage",
        "run_id",
        "run_attempt",
        "initiator",
        "approvers",
        "prepare_summary_sha256",
        "legacy_index",
        "alpha",
    }
    if stage == "preview-index":
        required.update({"alpha_evidence", "beta", "index", "public_smoke"})
    require_exact_keys(evidence, required=required, location="$")
    require_schema_v2(evidence)
    if evidence["operation"] != "schema-epoch-bootstrap":
        fail("$.operation", "must be schema-epoch-bootstrap")
    stage = require_enum(stage, {"alpha-assets", "preview-index"}, "$.stage")
    require_positive_int(evidence["run_id"], "$.run_id")
    require_positive_int(evidence["run_attempt"], "$.run_attempt")
    initiator = require_nonempty_string(evidence["initiator"], "$.initiator")
    _validate_approvers(evidence["approvers"], initiator, "$.approvers")
    require_sha256(
        evidence["prepare_summary_sha256"],
        "$.prepare_summary_sha256",
    )
    legacy = require_object(evidence["legacy_index"], "$.legacy_index")
    require_exact_keys(
        legacy,
        required={"sha256", "size_bytes"},
        location="$.legacy_index",
    )
    if legacy["sha256"] != LEGACY_INDEX_SHA256:
        fail("$.legacy_index.sha256", "does not match the opaque bootstrap asset")
    if legacy["size_bytes"] != LEGACY_INDEX_SIZE_BYTES:
        fail("$.legacy_index.size_bytes", "does not match the opaque bootstrap asset")
    _validate_release_evidence(evidence["alpha"], "alpha", "$.alpha")
    if stage == "alpha-assets":
        return evidence
    alpha_evidence = require_object(evidence["alpha_evidence"], "$.alpha_evidence")
    require_exact_keys(
        alpha_evidence,
        required={
            "sha256",
            "run_id",
            "run_attempt",
            "initiator",
            "approvers",
            "prepare_summary_sha256",
        },
        location="$.alpha_evidence",
    )
    require_sha256(alpha_evidence["sha256"], "$.alpha_evidence.sha256")
    require_positive_int(alpha_evidence["run_id"], "$.alpha_evidence.run_id")
    require_positive_int(
        alpha_evidence["run_attempt"],
        "$.alpha_evidence.run_attempt",
    )
    alpha_initiator = require_nonempty_string(
        alpha_evidence["initiator"],
        "$.alpha_evidence.initiator",
    )
    _validate_approvers(
        alpha_evidence["approvers"],
        alpha_initiator,
        "$.alpha_evidence.approvers",
    )
    require_sha256(
        alpha_evidence["prepare_summary_sha256"],
        "$.alpha_evidence.prepare_summary_sha256",
    )
    _validate_release_evidence(evidence["beta"], "beta", "$.beta")
    index = require_object(evidence["index"], "$.index")
    require_exact_keys(
        index,
        required={"generation", "sha256"},
        location="$.index",
    )
    if index["generation"] != BOOTSTRAP_GENERATION:
        fail("$.index.generation", "must start the canonical epoch at generation 1")
    require_sha256(index["sha256"], "$.index.sha256")
    smoke = require_array(evidence["public_smoke"], "$.public_smoke")
    if len(smoke) != len(SMOKE_IDS):
        fail("$.public_smoke", "must contain exactly four bootstrap smoke records")
    seen: set[str] = set()
    for index_value, raw in enumerate(smoke):
        location = f"$.public_smoke[{index_value}]"
        item = require_object(raw, location)
        require_exact_keys(item, required={"id", "status", "sha256"}, location=location)
        smoke_id = require_nonempty_string(item["id"], f"{location}.id")
        if smoke_id not in SMOKE_IDS or smoke_id in seen:
            fail(f"{location}.id", "must be a unique governed bootstrap smoke id")
        seen.add(smoke_id)
        if item["status"] != "pass":
            fail(f"{location}.status", "must be pass")
        require_sha256(item["sha256"], f"{location}.sha256")
    return evidence


def _validate_approvers(payload: Any, initiator: str, location: str) -> list[str]:
    values = require_array(payload, location)
    if not values:
        fail(location, "must contain at least one protected-environment approver")
    normalized_initiator = initiator.casefold()
    seen: set[str] = set()
    approvers: list[str] = []
    for index, raw in enumerate(values):
        approver = require_nonempty_string(raw, f"{location}[{index}]")
        normalized = approver.casefold()
        if normalized == normalized_initiator:
            fail(f"{location}[{index}]", "must differ from the workflow initiator")
        if normalized in seen:
            fail(f"{location}[{index}]", "must be a unique GitHub login")
        seen.add(normalized)
        approvers.append(approver)
    return approvers


def materialize_bootstrap_evidence(
    *,
    stage: str,
    run_id: int,
    run_attempt: int,
    initiator: str,
    approvers: list[str],
    prepare_summary_path: Path,
    legacy_index_path: Path,
    alpha_version: str,
    alpha_source_commit: str,
    alpha_record_path: Path,
    alpha_assets_dir: Path,
    out: Path,
    beta_version: str | None = None,
    beta_source_commit: str | None = None,
    beta_record_path: Path | None = None,
    beta_assets_dir: Path | None = None,
    index_path: Path | None = None,
    smoke_dir: Path | None = None,
    alpha_evidence_path: Path | None = None,
) -> dict[str, Any]:
    legacy_bytes = legacy_index_path.read_bytes()
    require_opaque_legacy_identity(
        sha256=sha256_bytes(legacy_bytes),
        size_bytes=len(legacy_bytes),
    )
    payload: dict[str, Any] = {
        "schema_version": 2,
        "operation": "schema-epoch-bootstrap",
        "stage": stage,
        "run_id": run_id,
        "run_attempt": run_attempt,
        "initiator": initiator,
        "approvers": approvers,
        "prepare_summary_sha256": sha256_file(prepare_summary_path),
        "legacy_index": {
            "sha256": sha256_bytes(legacy_bytes),
            "size_bytes": len(legacy_bytes),
        },
        "alpha": _materialize_release_evidence(
            version=alpha_version,
            channel="alpha",
            source_commit=alpha_source_commit,
            record_path=alpha_record_path,
            assets_dir=alpha_assets_dir,
        ),
    }
    if stage == "preview-index":
        if (
            beta_version is None
            or beta_source_commit is None
            or beta_record_path is None
            or beta_assets_dir is None
            or index_path is None
            or smoke_dir is None
            or alpha_evidence_path is None
        ):
            fail(
                "bootstrap evidence",
                "preview-index stage requires beta, index, and smoke inputs",
            )
        index = validate_release_index(
            load_json_strict(index_path, require_canonical=True)
        )
        if index["generation"] != BOOTSTRAP_GENERATION:
            fail("bootstrap index", "must be generation 1")
        alpha_wrapper = load_json_strict(alpha_record_path, require_canonical=True)
        beta_wrapper = load_json_strict(beta_record_path, require_canonical=True)
        _require_exact_bootstrap_membership(
            index=index,
            alpha_version=alpha_version,
            alpha_wrapper=alpha_wrapper,
            beta_version=beta_version,
            beta_wrapper=beta_wrapper,
        )
        staged_alpha = validate_bootstrap_evidence(
            load_json_strict(alpha_evidence_path, require_canonical=True)
        )
        if (
            staged_alpha["stage"] != "alpha-assets"
            or staged_alpha["alpha"] != payload["alpha"]
        ):
            fail(
                "alpha bootstrap evidence",
                "must bind the exact alpha release used by generation 1",
            )
        payload.update(
            {
                "alpha_evidence": {
                    "sha256": sha256_file(alpha_evidence_path),
                    "run_id": staged_alpha["run_id"],
                    "run_attempt": staged_alpha["run_attempt"],
                    "initiator": staged_alpha["initiator"],
                    "approvers": staged_alpha["approvers"],
                    "prepare_summary_sha256": staged_alpha[
                        "prepare_summary_sha256"
                    ],
                },
                "beta": _materialize_release_evidence(
                    version=beta_version,
                    channel="beta",
                    source_commit=beta_source_commit,
                    record_path=beta_record_path,
                    assets_dir=beta_assets_dir,
                ),
                "index": {
                    "generation": index["generation"],
                    "sha256": sha256_file(index_path),
                },
                "public_smoke": [
                    {
                        "id": smoke_id,
                        "status": "pass",
                        "sha256": sha256_file(smoke_dir / f"{smoke_id}.txt"),
                    }
                    for smoke_id in sorted(SMOKE_IDS)
                ],
            }
        )
    evidence = validate_bootstrap_evidence(payload)
    write_canonical_json(out, evidence, refuse_existing=True)
    return evidence


def _require_exact_bootstrap_membership(
    *,
    index: dict[str, Any],
    alpha_version: str,
    alpha_wrapper: Any,
    beta_version: str,
    beta_wrapper: Any,
) -> None:
    alpha_record_version, alpha_release = _validate_release_wrapper(
        alpha_wrapper,
        expected_channel="alpha",
        location="alpha release",
    )
    beta_record_version, beta_release = _validate_release_wrapper(
        beta_wrapper,
        expected_channel="beta",
        location="beta release",
    )
    if (
        alpha_record_version != alpha_version
        or beta_record_version != beta_version
    ):
        fail(
            "bootstrap index",
            "evidenced versions must match their exact release records",
        )
    expected_channels = {"alpha": alpha_version, "beta": beta_version}
    if index["ga_status"] != "preview" or index["channels"] != expected_channels:
        fail(
            "bootstrap index",
            "must expose exactly the evidenced alpha and beta preview channels",
        )
    expected_releases = {
        alpha_version: alpha_release,
        beta_version: beta_release,
    }
    if index["releases"] != expected_releases:
        fail(
            "bootstrap index",
            "must contain exactly the evidenced alpha and beta release records",
        )


def expected_asset_names(version: str) -> set[str]:
    names = {f"sifr-installer-{version}"}
    for target in TARGETS:
        archive = f"sifr-{version}-{target}.tar.gz"
        names.update({archive, f"{archive}.sha256"})
    return names


def _materialize_release_evidence(
    *,
    version: str,
    channel: str,
    source_commit: str,
    record_path: Path,
    assets_dir: Path,
) -> dict[str, Any]:
    require_commit(source_commit, f"{channel} source commit")
    record_version, record = _validate_release_wrapper(
        load_json_strict(record_path, require_canonical=True),
        expected_channel=channel,
        location=f"{channel} release",
    )
    if record_version != version:
        fail(f"{channel} release.version", "does not match the evidence version")
    if record["source_commit"] != source_commit:
        fail(f"{channel} release.source_commit", "does not match the evidence source")
    names = expected_asset_names(version)
    actual = {path.name for path in assets_dir.iterdir() if path.is_file()}
    if actual != names:
        fail(f"{channel} assets", "must contain the exact immutable asset set")
    return {
        "version": version,
        "source_commit": source_commit,
        "release_record_sha256": sha256_file(record_path),
        "published_assets": {
            name: sha256_file(assets_dir / name) for name in sorted(names)
        },
    }


def _validate_release_wrapper(
    payload: Any,
    *,
    expected_channel: str,
    location: str,
) -> tuple[str, dict[str, Any]]:
    wrapper = require_object(payload, location)
    require_exact_keys(wrapper, required={"version", "release"}, location=location)
    version = require_nonempty_string(wrapper["version"], f"{location}.version")
    if version_channel(version, f"{location}.version") != expected_channel:
        fail(f"{location}.version", f"must be a {expected_channel} version")
    release = validate_release_record(
        wrapper["release"],
        version=version,
        expected_channel=expected_channel,
    )
    if release["status"] != "active":
        fail(f"{location}.release.status", "must be active")
    return version, release


def _validate_release_evidence(payload: Any, channel: str, location: str) -> None:
    value = require_object(payload, location)
    require_exact_keys(
        value,
        required={
            "version",
            "source_commit",
            "release_record_sha256",
            "published_assets",
        },
        location=location,
    )
    version = require_nonempty_string(value["version"], f"{location}.version")
    if version_channel(version, f"{location}.version") != channel:
        fail(f"{location}.version", f"must be a {channel} version")
    require_commit(value["source_commit"], f"{location}.source_commit")
    require_sha256(value["release_record_sha256"], f"{location}.release_record_sha256")
    assets = require_object(value["published_assets"], f"{location}.published_assets")
    expected = expected_asset_names(version)
    if set(assets) != expected:
        fail(
            f"{location}.published_assets", "must contain the exact immutable asset set"
        )
    for name, digest in assets.items():
        require_sha256(digest, f"{location}.published_assets.{name}")
