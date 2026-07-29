"""Materialize protected stable publication files and final sign-off evidence."""

from __future__ import annotations

import shutil
from pathlib import Path
from typing import Any

from .artifact_index import validate_qualification_artifact_index
from .common import (
    canonical_json_bytes,
    fail,
    load_json_strict,
    require_nonempty_string,
    require_positive_int,
    sha256_bytes,
    sha256_file,
    write_canonical_json,
)
from .planner import verify_transported_artifacts
from .release_plan import (
    generate_site_release_facts,
    validate_release_plan,
    validate_release_signoff,
    validate_site_release_facts,
)
from .stable_prepare import validate_stable_prepare_summary

DISPATCHERS = ("index", "stable", "alpha", "beta")
PLAN_ASSET_NAME = "stable-release-plan.json"
SMOKE_FILES = {
    "governed-index": "governed-index.json",
    "install-dispatcher": "install-dispatcher",
    "stable-dispatcher": "stable-dispatcher",
    "stable-release-docs": "stable-release-docs.html",
    "version-assets": "version-assets.json",
    "installed-self-update": "installed-self-update.json",
    "marketplace-vsix": "marketplace.vsix",
}


def stage_stable_publication(
    *,
    prepare_summary_path: Path,
    qualification_index_path: Path,
    artifact_root: Path,
    plan_path: Path,
    dispatcher_root: Path,
    output_root: Path,
) -> dict[str, Any]:
    """Stage the exact qualified bytes and derived active-index evidence."""
    if output_root.exists() or output_root.is_symlink():
        fail("output_root", "must not already exist")
    if output_root.parent.is_symlink() or not output_root.parent.is_dir():
        fail("output_root", "parent must be an existing non-symlink directory")

    summary = validate_stable_prepare_summary(
        load_json_strict(prepare_summary_path, require_canonical=True)
    )
    if sha256_file(plan_path) != summary["evidence"]["plan_sha256"]:
        fail("plan_path", "does not match the protected prepare summary")
    plan = validate_release_plan(
        load_json_strict(plan_path, require_canonical=True)
    )
    if (
        plan["version"] != summary["version"]
        or plan["transition"] != summary["operation"]
        or plan["source_commit"] != summary["source"]["commit"]
    ):
        fail("plan_path", "identity does not match the protected prepare summary")

    if sha256_file(qualification_index_path) != summary["qualification"]["sha256"]:
        fail(
            "qualification_index_path",
            "does not match the protected prepare summary",
        )
    qualification = validate_qualification_artifact_index(
        load_json_strict(qualification_index_path, require_canonical=True),
        require_unexpired=True,
    )
    transported = verify_transported_artifacts(qualification, artifact_root)
    _require_summary_artifacts(summary, qualification)
    dispatchers = _validated_dispatchers(plan, dispatcher_root)

    output_root.mkdir()
    release_assets = output_root / "release-assets"
    release_assets.mkdir()
    names: set[str] = set()
    for artifact in qualification["artifacts"]:
        name = artifact["name"]
        if name == PLAN_ASSET_NAME or name in names:
            fail("$.artifacts", f"release asset name collision: {name}")
        names.add(name)
        shutil.copyfile(transported[artifact["id"]], release_assets / name)
    shutil.copyfile(plan_path, release_assets / PLAN_ASSET_NAME)

    proposed_index = summary["mutation"]["proposed_index"]
    write_canonical_json(
        output_root / "channels.json",
        proposed_index,
        refuse_existing=True,
    )
    proposed_sha256 = sha256_file(output_root / "channels.json")
    if proposed_sha256 != summary["mutation"]["proposed_index_sha256"]:
        fail("$.mutation.proposed_index_sha256", "staged index bytes drifted")

    site_facts = generate_site_release_facts(
        proposed_index,
        source_plan_sha256=summary["evidence"]["plan_sha256"],
        release_index_sha256=proposed_sha256,
        dispatchers=dispatchers,
    )
    write_canonical_json(
        output_root / "stable-site-release-facts.json",
        site_facts,
        refuse_existing=True,
    )
    return {
        "version": summary["version"],
        "publication_state": summary["publication_state"],
        "release_assets": {
            path.name: sha256_file(path)
            for path in sorted(release_assets.iterdir())
        },
        "release_index_sha256": proposed_sha256,
        "site_facts_sha256": sha256_file(
            output_root / "stable-site-release-facts.json"
        ),
    }


def materialize_stable_signoff(
    *,
    prepare_summary_path: Path,
    release_assets_root: Path,
    site_facts_path: Path,
    site_run_path: Path,
    smoke_root: Path,
    run_id: int,
    approver: str,
    approval_policy: dict[str, str],
) -> dict[str, Any]:
    """Build completed write-once sign-off evidence from verified public bytes."""
    summary = validate_stable_prepare_summary(
        load_json_strict(prepare_summary_path, require_canonical=True)
    )
    run_id = require_positive_int(run_id, "run_id")
    approver = require_nonempty_string(approver, "approver")
    published_assets = _published_asset_digests(summary, release_assets_root)

    site_facts = validate_site_release_facts(
        load_json_strict(site_facts_path, require_canonical=True),
        governed_index=summary["mutation"]["proposed_index"],
    )
    if (
        site_facts["source_plan_sha256"] != summary["evidence"]["plan_sha256"]
        or site_facts["release_index_sha256"]
        != summary["mutation"]["proposed_index_sha256"]
    ):
        fail("site_facts_path", "does not bind the protected proposal")
    site_publication = _site_publication(site_run_path, summary)

    smoke = _smoke_evidence(summary, smoke_root, published_assets)
    mutation_manifest = {
        "version-release": sha256_bytes(canonical_json_bytes(published_assets)),
        "marketplace": summary["marketplace"]["vsix_sha256"],
        "release-index": summary["mutation"]["proposed_index_sha256"],
        "site-release-facts": sha256_file(site_facts_path),
    }
    signoff = {
        "schema_version": 2,
        "version": summary["version"],
        "plan_sha256": summary["evidence"]["plan_sha256"],
        "approval_policy": approval_policy,
        "attempts": [
            {
                "run_id": run_id,
                "mode": summary["mode"],
                "approver": approver,
                "status": "completed",
                "mutations": [
                    {
                        "kind": kind,
                        "identity": _mutation_identity(kind, summary),
                        "sha256": digest,
                    }
                    for kind, digest in mutation_manifest.items()
                ],
            }
        ],
        "published_assets": published_assets,
        "marketplace": summary["marketplace"],
        "channel_generation": summary["mutation"]["proposed_index"]["generation"],
        "site_publication": site_publication,
        "site_facts_sha256": sha256_file(site_facts_path),
        "post_publication_smoke": smoke,
    }
    return validate_release_signoff(signoff)


def _require_summary_artifacts(
    summary: dict[str, Any],
    qualification: dict[str, Any],
) -> None:
    expected = {
        artifact["id"]: {
            key: artifact[key]
            for key in (
                "name",
                "sha256",
                "size_bytes",
                "workflow_artifact_id",
                "workflow_artifact_name",
            )
        }
        for artifact in qualification["artifacts"]
    }
    if summary["artifacts"] != expected:
        fail("$.artifacts", "does not equal the qualification artifact index")


def _validated_dispatchers(
    plan: dict[str, Any],
    dispatcher_root: Path,
) -> dict[str, str]:
    entries = list(dispatcher_root.iterdir()) if dispatcher_root.is_dir() else []
    if {path.name for path in entries} != set(DISPATCHERS):
        fail("dispatcher_root", "must contain exactly index, stable, alpha, and beta")
    if any(path.is_symlink() or not path.is_file() for path in entries):
        fail("dispatcher_root", "must contain regular non-symlink files")
    actual = {name: sha256_file(dispatcher_root / name) for name in DISPATCHERS}
    if actual != plan["site"]["dispatcher_sha256"]:
        fail("dispatcher_root", "digests do not match the approved plan")
    return actual


def _published_asset_digests(
    summary: dict[str, Any],
    release_assets_root: Path,
) -> dict[str, str]:
    expected = {
        artifact["name"]: artifact["sha256"]
        for artifact in summary["artifacts"].values()
    }
    expected[PLAN_ASSET_NAME] = summary["evidence"]["plan_sha256"]
    entries = list(release_assets_root.iterdir()) if release_assets_root.is_dir() else []
    if {path.name for path in entries} != set(expected):
        fail("release_assets_root", "does not contain the exact published asset set")
    if any(path.is_symlink() or not path.is_file() for path in entries):
        fail("release_assets_root", "must contain regular non-symlink files")
    actual = {path.name: sha256_file(path) for path in entries}
    if actual != expected:
        fail("release_assets_root", "published asset bytes drifted")
    return dict(sorted(actual.items()))


def _smoke_evidence(
    summary: dict[str, Any],
    smoke_root: Path,
    published_assets: dict[str, str],
) -> list[dict[str, str]]:
    entries = list(smoke_root.iterdir()) if smoke_root.is_dir() else []
    if {path.name for path in entries} != set(SMOKE_FILES.values()):
        fail("smoke_root", "does not contain the exact stable smoke evidence set")
    if any(path.is_symlink() or not path.is_file() for path in entries):
        fail("smoke_root", "must contain regular non-symlink files")
    if (smoke_root / "governed-index.json").read_bytes() != canonical_json_bytes(
        summary["mutation"]["proposed_index"]
    ):
        fail("smoke_root", "public governed index bytes drifted")
    observed_assets = load_json_strict(
        smoke_root / "version-assets.json",
        require_canonical=True,
    )
    if observed_assets != published_assets:
        fail("smoke_root", "public version asset bytes drifted")
    if (
        sha256_file(smoke_root / "marketplace.vsix")
        != summary["marketplace"]["vsix_sha256"]
    ):
        fail("smoke_root", "Marketplace VSIX bytes drifted")
    return [
        {
            "id": identifier,
            "status": "pass",
            "sha256": sha256_file(smoke_root / filename),
        }
        for identifier, filename in SMOKE_FILES.items()
    ]


def _site_publication(
    site_run_path: Path,
    summary: dict[str, Any],
) -> dict[str, Any]:
    value = load_json_strict(site_run_path, require_canonical=True)
    if not isinstance(value, dict) or set(value) != {
        "repository",
        "workflow",
        "run_id",
        "deployed_commit",
    }:
        fail("site_run_path", "must contain the exact correlated site run fields")
    if (
        value["repository"] != "sifr-lang/sifr-website"
        or value["workflow"] != "release-site.yml"
        or value["deployed_commit"] != summary["site"]["base_commit"]
        or not isinstance(value["run_id"], int)
        or isinstance(value["run_id"], bool)
        or value["run_id"] < 1
    ):
        fail("site_run_path", "does not match the approved correlated deployment")
    return value


def _mutation_identity(kind: str, summary: dict[str, Any]) -> str:
    if kind == "version-release":
        return summary["version"]
    if kind == "marketplace":
        marketplace = summary["marketplace"]
        return (
            f"{marketplace['publisher']}.{marketplace['extension']}"
            f"@{marketplace['version']}"
        )
    if kind == "release-index":
        return f"generation-{summary['mutation']['proposed_index']['generation']}"
    return f"generation-{summary['mutation']['proposed_index']['generation']}"
