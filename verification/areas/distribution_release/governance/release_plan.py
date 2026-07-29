"""Stable plan, sign-off, and derived site-facts validation."""

from __future__ import annotations

from typing import Any

from .approval_waiver import validate_approval_policy
from .common import (
    BUILDERS,
    TARGETS,
    fail,
    require_array,
    require_commit,
    require_enum,
    require_exact_keys,
    require_nonempty_string,
    require_object,
    require_plan_id,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    version_channel,
)
from .incident import validate_attempts
from .release_index import validate_release_index, validate_release_record

PLAN_REQUIRED = {
    "schema_version",
    "plan_id",
    "version",
    "transition",
    "source_commit",
    "submodules",
    "cargo_lock_sha256",
    "toolchain",
    "expected_stable_predecessor",
    "desired_release",
    "rollback_target",
    "targets",
    "installer_sha256",
    "release_profile_report",
    "qualification_artifact_index",
    "rust_interop",
    "documentation_report",
    "site",
    "vscode",
    "release_notes_sha256",
}


def validate_release_plan(
    payload: Any,
    *,
    active_index: Any | None = None,
    incident_request_sha256: str | None = None,
) -> dict[str, Any]:
    plan = require_object(payload, "$")
    transition = plan.get("transition")
    required = set(PLAN_REQUIRED)
    if transition == "incident-roll-forward":
        required.add("incident_request_sha256")
    require_exact_keys(plan, required=required, location="$")
    require_schema_v2(plan)
    version = plan["version"]
    if version_channel(version, "$.version") != "stable":
        fail("$.version", "must be an exact stable version")
    require_plan_id(plan["plan_id"], version, "$.plan_id")
    transition = require_enum(
        transition,
        {"ga-activation", "normal", "incident-roll-forward"},
        "$.transition",
    )
    require_commit(plan["source_commit"], "$.source_commit")
    if plan["plan_id"] != f"stable-{version}-{plan['source_commit'][:12]}":
        fail("$.plan_id", "must bind the exact source commit prefix")
    validate_submodules(plan["submodules"])
    require_sha256(plan["cargo_lock_sha256"], "$.cargo_lock_sha256")
    validate_toolchain(plan["toolchain"])

    desired = validate_release_record(
        plan["desired_release"],
        version=version,
        expected_channel="stable",
    )
    if desired["status"] != "active" or "incident_id" in desired:
        fail("$.desired_release", "candidate release must be active and have no incident id")
    if desired["source_commit"] != plan["source_commit"]:
        fail("$.desired_release.source_commit", "must equal source_commit")
    if desired["installer_sha256"] != plan["installer_sha256"]:
        fail("$.installer_sha256", "must match desired_release")
    require_sha256(plan["installer_sha256"], "$.installer_sha256")

    predecessor = validate_predecessor(plan["expected_stable_predecessor"])
    rollback = validate_rollback_target(plan["rollback_target"])
    if transition == "ga-activation":
        if predecessor != "none" or rollback != "none":
            fail("$.transition", "ga-activation requires predecessor and rollback_target to be none")
    elif transition == "normal":
        if predecessor == "none" or rollback == "none":
            fail("$.transition", "normal requires an active predecessor and rollback target")
        if predecessor["version"] != rollback["version"] or predecessor["plan_sha256"] != rollback["plan_sha256"]:
            fail("$.rollback_target", "must match the active predecessor")
    else:
        if rollback != "none":
            fail("$.rollback_target", "incident-roll-forward requires rollback_target none")
        digest = require_sha256(plan["incident_request_sha256"], "$.incident_request_sha256")
        if incident_request_sha256 is not None and digest != incident_request_sha256:
            fail("$.incident_request_sha256", "does not match the approved incident request")

    validate_targets(plan["targets"], version, desired["targets"])
    validate_report_ref(plan["release_profile_report"], "$.release_profile_report")
    validate_report_ref(plan["qualification_artifact_index"], "$.qualification_artifact_index")
    validate_rust_interop(plan["rust_interop"])
    validate_report_ref(plan["documentation_report"], "$.documentation_report")
    validate_site(plan["site"])
    validate_vscode(plan["vscode"])
    require_sha256(plan["release_notes_sha256"], "$.release_notes_sha256")

    if active_index is not None:
        index = validate_release_index(active_index)
        live_stable = index["channels"].get("stable")
        if transition == "ga-activation":
            if index["ga_status"] != "preview" or live_stable is not None:
                fail("$.transition", "ga-activation requires preview metadata without stable")
        else:
            if index["ga_status"] != "active" or predecessor == "none":
                fail("$.expected_stable_predecessor", "requires an active live stable predecessor")
            if live_stable != predecessor["version"]:
                fail("$.expected_stable_predecessor.version", "does not match live stable")
            live_release = index["releases"].get(live_stable)
            if not isinstance(live_release, dict) or live_release.get("status") != "active":
                fail("$.expected_stable_predecessor", "does not name an active release")
    return plan


def validate_submodules(payload: Any) -> None:
    submodules = require_object(payload, "$.submodules")
    if not submodules:
        fail("$.submodules", "must contain recursive submodule identities")
    for path, commit in submodules.items():
        require_nonempty_string(path, "$.submodules key")
        require_commit(commit, f"$.submodules.{path}")


def validate_toolchain(payload: Any) -> None:
    toolchain = require_object(payload, "$.toolchain")
    require_exact_keys(
        toolchain,
        required={"rustc", "cargo", "profile_manifest_sha256"},
        location="$.toolchain",
    )
    require_nonempty_string(toolchain["rustc"], "$.toolchain.rustc")
    require_nonempty_string(toolchain["cargo"], "$.toolchain.cargo")
    require_sha256(toolchain["profile_manifest_sha256"], "$.toolchain.profile_manifest_sha256")


def validate_predecessor(payload: Any) -> str | dict[str, Any]:
    if payload == "none":
        return "none"
    predecessor = require_object(payload, "$.expected_stable_predecessor")
    require_exact_keys(
        predecessor,
        required={"version", "status", "plan_sha256"},
        location="$.expected_stable_predecessor",
    )
    if version_channel(predecessor["version"], "$.expected_stable_predecessor.version") != "stable":
        fail("$.expected_stable_predecessor.version", "must be stable")
    if predecessor["status"] != "active":
        fail("$.expected_stable_predecessor.status", "must be active")
    require_sha256(predecessor["plan_sha256"], "$.expected_stable_predecessor.plan_sha256")
    return predecessor


def validate_rollback_target(payload: Any) -> str | dict[str, Any]:
    if payload == "none":
        return "none"
    target = require_object(payload, "$.rollback_target")
    require_exact_keys(target, required={"version", "plan_sha256"}, location="$.rollback_target")
    if version_channel(target["version"], "$.rollback_target.version") != "stable":
        fail("$.rollback_target.version", "must be stable")
    require_sha256(target["plan_sha256"], "$.rollback_target.plan_sha256")
    return target


def validate_targets(payload: Any, version: str, desired_targets: dict[str, Any]) -> None:
    targets = require_array(payload, "$.targets")
    if len(targets) != len(TARGETS):
        fail("$.targets", "must contain exactly the four supported targets")
    seen: set[str] = set()
    for index, target_value in enumerate(targets):
        location = f"$.targets[{index}]"
        target = require_object(target_value, location)
        require_exact_keys(
            target,
            required={
                "triple",
                "builder",
                "binary_sha256",
                "sysroot_sha256",
                "archive_sha256",
                "checksum_sha256",
                "sifr_version",
                "installer_version",
                "receipt_channel",
                "sysroot_version",
                "sysroot_target",
            },
            location=location,
        )
        triple = target["triple"]
        if triple not in TARGETS or triple in seen:
            fail(f"{location}.triple", "must be a unique supported target")
        seen.add(triple)
        if target["builder"] != BUILDERS[triple]:
            fail(f"{location}.builder", "does not match the governed builder")
        for field in ("binary_sha256", "sysroot_sha256", "archive_sha256", "checksum_sha256"):
            require_sha256(target[field], f"{location}.{field}")
        if target["archive_sha256"] != desired_targets[triple]["artifact_sha256"]:
            fail(f"{location}.archive_sha256", "must match desired release record")
        if target["sysroot_sha256"] != desired_targets[triple]["sysroot_content_sha256"]:
            fail(f"{location}.sysroot_sha256", "must match desired release record")
        for field in ("sifr_version", "installer_version", "sysroot_version"):
            if target[field] != version:
                fail(f"{location}.{field}", "must match plan version")
        if target["receipt_channel"] != "stable":
            fail(f"{location}.receipt_channel", "must be stable")
        if target["sysroot_target"] != triple:
            fail(f"{location}.sysroot_target", "must match target triple")
    if seen != set(TARGETS):
        fail("$.targets", "is missing a supported target")


def validate_report_ref(payload: Any, location: str) -> None:
    reference = require_object(payload, location)
    require_exact_keys(reference, required={"id", "sha256"}, location=location)
    require_nonempty_string(reference["id"], f"{location}.id")
    require_sha256(reference["sha256"], f"{location}.sha256")


def validate_rust_interop(payload: Any) -> None:
    rust = require_object(payload, "$.rust_interop")
    require_exact_keys(
        rust,
        required={
            "compatibility_matrix_sha256",
            "stable_support_claims_sha256",
            "advertised_claim_ids",
            "validation_report_sha256",
        },
        location="$.rust_interop",
    )
    for field in (
        "compatibility_matrix_sha256",
        "stable_support_claims_sha256",
        "validation_report_sha256",
    ):
        require_sha256(rust[field], f"$.rust_interop.{field}")
    claims = require_array(rust["advertised_claim_ids"], "$.rust_interop.advertised_claim_ids")
    if not claims:
        fail("$.rust_interop.advertised_claim_ids", "must be non-empty and unique")
    for index, claim in enumerate(claims):
        require_nonempty_string(claim, f"$.rust_interop.advertised_claim_ids[{index}]")
    if len(set(claims)) != len(claims):
        fail("$.rust_interop.advertised_claim_ids", "must be non-empty and unique")


def validate_site(payload: Any) -> None:
    site = require_object(payload, "$.site")
    require_exact_keys(
        site,
        required={
            "repository",
            "base_commit",
            "dispatcher_sha256",
            "facts_schema_sha256",
            "facts_generator_sha256",
        },
        location="$.site",
    )
    if site["repository"] != "sifr-lang/sifr-website":
        fail("$.site.repository", "must be sifr-lang/sifr-website")
    require_commit(site["base_commit"], "$.site.base_commit")
    dispatchers = require_object(site["dispatcher_sha256"], "$.site.dispatcher_sha256")
    require_exact_keys(
        dispatchers,
        required={"index", "stable", "alpha", "beta"},
        location="$.site.dispatcher_sha256",
    )
    for name, digest in dispatchers.items():
        require_sha256(digest, f"$.site.dispatcher_sha256.{name}")
    require_sha256(site["facts_schema_sha256"], "$.site.facts_schema_sha256")
    require_sha256(site["facts_generator_sha256"], "$.site.facts_generator_sha256")


def validate_vscode(payload: Any) -> None:
    vscode = require_object(payload, "$.vscode")
    require_exact_keys(
        vscode,
        required={
            "submodule_path",
            "package_path",
            "version",
            "vsix_sha256",
            "compiler_compatibility",
            "validation_report_sha256",
        },
        location="$.vscode",
    )
    if vscode["submodule_path"] != "editor_integrations":
        fail("$.vscode.submodule_path", "must be editor_integrations")
    if vscode["package_path"] != "editor_integrations/vscode":
        fail("$.vscode.package_path", "must be editor_integrations/vscode")
    require_nonempty_string(vscode["version"], "$.vscode.version")
    require_sha256(vscode["vsix_sha256"], "$.vscode.vsix_sha256")
    require_nonempty_string(vscode["compiler_compatibility"], "$.vscode.compiler_compatibility")
    require_sha256(vscode["validation_report_sha256"], "$.vscode.validation_report_sha256")


def validate_release_signoff(payload: Any) -> dict[str, Any]:
    signoff = require_object(payload, "$")
    require_exact_keys(
        signoff,
        required={
            "schema_version",
            "version",
            "plan_sha256",
            "approval_policy",
            "attempts",
            "published_assets",
            "marketplace",
            "channel_generation",
            "site_publication",
            "site_facts_sha256",
            "post_publication_smoke",
        },
        location="$",
    )
    require_schema_v2(signoff)
    if version_channel(signoff["version"], "$.version") != "stable":
        fail("$.version", "must be an exact stable version")
    require_sha256(signoff["plan_sha256"], "$.plan_sha256")
    validate_approval_policy(signoff["approval_policy"], "$.approval_policy")
    validate_attempts(signoff["attempts"], "$.attempts")
    assets = require_object(signoff["published_assets"], "$.published_assets")
    if not assets:
        fail("$.published_assets", "must contain published asset evidence")
    for name, digest in assets.items():
        require_nonempty_string(name, "$.published_assets key")
        require_sha256(digest, f"$.published_assets.{name}")
    marketplace = require_object(signoff["marketplace"], "$.marketplace")
    require_exact_keys(
        marketplace,
        required={"publisher", "extension", "version", "vsix_sha256"},
        location="$.marketplace",
    )
    for field in ("publisher", "extension", "version"):
        require_nonempty_string(marketplace[field], f"$.marketplace.{field}")
    require_sha256(marketplace["vsix_sha256"], "$.marketplace.vsix_sha256")
    require_positive_int(signoff["channel_generation"], "$.channel_generation")
    site = require_object(signoff["site_publication"], "$.site_publication")
    require_exact_keys(
        site,
        required={"repository", "workflow", "run_id", "deployed_commit"},
        location="$.site_publication",
    )
    if site["repository"] != "sifr-lang/sifr-website":
        fail("$.site_publication.repository", "must be sifr-lang/sifr-website")
    if site["workflow"] != "release-site.yml":
        fail("$.site_publication.workflow", "must be release-site.yml")
    require_positive_int(site["run_id"], "$.site_publication.run_id")
    require_commit(site["deployed_commit"], "$.site_publication.deployed_commit")
    require_sha256(signoff["site_facts_sha256"], "$.site_facts_sha256")
    smoke = require_array(signoff["post_publication_smoke"], "$.post_publication_smoke")
    if len(smoke) < 4:
        fail("$.post_publication_smoke", "must contain at least four smoke results")
    for index, value in enumerate(smoke):
        evidence = require_object(value, f"$.post_publication_smoke[{index}]")
        require_exact_keys(
            evidence,
            required={"id", "status", "sha256"},
            location=f"$.post_publication_smoke[{index}]",
        )
        require_nonempty_string(evidence["id"], f"$.post_publication_smoke[{index}].id")
        if evidence["status"] != "pass":
            fail(f"$.post_publication_smoke[{index}].status", "must be pass")
        require_sha256(evidence["sha256"], f"$.post_publication_smoke[{index}].sha256")
    return signoff


def generate_site_release_facts(
    index_value: Any,
    *,
    source_plan_sha256: str,
    release_index_sha256: str,
    dispatchers: dict[str, str],
) -> dict[str, Any]:
    index = validate_release_index(index_value)
    if index["ga_status"] != "active":
        fail("$.ga_status", "site facts require an active GA index")
    require_sha256(source_plan_sha256, "source_plan_sha256")
    require_sha256(release_index_sha256, "release_index_sha256")
    require_exact_keys(
        dispatchers,
        required={"index", "stable", "alpha", "beta"},
        location="dispatchers",
    )
    for name, digest in dispatchers.items():
        require_sha256(digest, f"dispatchers.{name}")
    withdrawals = [
        {"version": version, "incident_id": release["incident_id"]}
        for version, release in sorted(index["releases"].items())
        if release["status"] == "withdrawn"
    ]
    facts = {
        "schema_version": 2,
        "generation": index["generation"],
        "stable_version": index["channels"]["stable"],
        "stable_status": "active",
        "source_plan_sha256": source_plan_sha256,
        "release_index_sha256": release_index_sha256,
        "dispatchers": dict(sorted(dispatchers.items())),
        "withdrawals": withdrawals,
    }
    validate_site_release_facts(facts, governed_index=index)
    return facts


def validate_site_release_facts(payload: Any, *, governed_index: Any | None = None) -> dict[str, Any]:
    facts = require_object(payload, "$")
    require_exact_keys(
        facts,
        required={
            "schema_version",
            "generation",
            "stable_version",
            "stable_status",
            "source_plan_sha256",
            "release_index_sha256",
            "dispatchers",
            "withdrawals",
        },
        location="$",
    )
    require_schema_v2(facts)
    require_positive_int(facts["generation"], "$.generation")
    if version_channel(facts["stable_version"], "$.stable_version") != "stable":
        fail("$.stable_version", "must be stable")
    if facts["stable_status"] != "active":
        fail("$.stable_status", "must be active")
    require_sha256(facts["source_plan_sha256"], "$.source_plan_sha256")
    require_sha256(facts["release_index_sha256"], "$.release_index_sha256")
    dispatchers = require_object(facts["dispatchers"], "$.dispatchers")
    require_exact_keys(
        dispatchers,
        required={"index", "stable", "alpha", "beta"},
        location="$.dispatchers",
    )
    for name, digest in dispatchers.items():
        require_sha256(digest, f"$.dispatchers.{name}")
    withdrawals = require_array(facts["withdrawals"], "$.withdrawals")
    observed: list[tuple[str, str]] = []
    for index, value in enumerate(withdrawals):
        item = require_object(value, f"$.withdrawals[{index}]")
        require_exact_keys(
            item,
            required={"version", "incident_id"},
            location=f"$.withdrawals[{index}]",
        )
        version_channel(item["version"], f"$.withdrawals[{index}].version")
        require_nonempty_string(item["incident_id"], f"$.withdrawals[{index}].incident_id")
        observed.append((item["version"], item["incident_id"]))
    if observed != sorted(observed):
        fail("$.withdrawals", "must use deterministic version ordering")
    if governed_index is not None:
        index = validate_release_index(governed_index)
        expected = [
            (version, release["incident_id"])
            for version, release in sorted(index["releases"].items())
            if release["status"] == "withdrawn"
        ]
        if facts["generation"] != index["generation"]:
            fail("$.generation", "disagrees with governed index")
        if facts["stable_version"] != index["channels"].get("stable"):
            fail("$.stable_version", "disagrees with governed index")
        if observed != expected:
            fail("$.withdrawals", "disagrees with governed index")
    return facts
