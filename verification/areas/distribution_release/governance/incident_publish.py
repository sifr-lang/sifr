"""Stage incident index/site evidence and materialize protected sign-off."""

from __future__ import annotations

from pathlib import Path
from typing import Any

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
from .incident import validate_incident_signoff
from .incident_prepare import validate_incident_prepare_summary
from .release_plan import (
    generate_site_release_facts,
    validate_release_plan,
    validate_release_signoff,
    validate_site_release_facts,
)
from .stable_publish import DISPATCHERS

SMOKE_FILES = {
    "governed-index": "governed-index.json",
    "install-dispatcher": "install-dispatcher",
    "stable-dispatcher": "stable-dispatcher",
    "stable-release-docs": "stable-release-docs.html",
    "version-assets": "version-assets.json",
    "installed-self-update": "installed-self-update.json",
    "marketplace-vsix": "marketplace.vsix",
    "incident-recovery": "incident-recovery.json",
}


def stage_incident_publication(
    *,
    prepare_summary_path: Path,
    successor_plan_path: Path,
    site_plan_path: Path,
    dispatcher_root: Path,
    output_root: Path,
) -> dict[str, str]:
    """Stage the exact proposed incident index and derived site facts."""
    if output_root.exists() or output_root.is_symlink():
        fail("output_root", "must not already exist")
    summary = validate_incident_prepare_summary(
        load_json_strict(prepare_summary_path, require_canonical=True)
    )
    if sha256_file(successor_plan_path) != summary["successor"]["plan_sha256"]:
        fail("successor_plan_path", "does not match protected prepare")
    successor_plan = validate_release_plan(
        load_json_strict(successor_plan_path, require_canonical=True)
    )
    site_plan = validate_release_plan(
        load_json_strict(site_plan_path, require_canonical=True)
    )
    if (
        successor_plan["version"] != summary["successor"]["version"]
        or site_plan["site"]["repository"] != summary["site"]["repository"]
        or site_plan["site"]["base_commit"] != summary["site"]["base_commit"]
    ):
        fail("site_plan_path", "identity does not match protected prepare")
    expected_site_plan_sha256 = (
        summary["affected"]["plan_sha256"]
        if summary["operation"] == "rollback"
        else summary["successor"]["plan_sha256"]
    )
    if sha256_file(site_plan_path) != expected_site_plan_sha256:
        fail("site_plan_path", "does not match the operation's approved site plan")
    # The site regenerates dispatchers from the rollback target source commit;
    # only byte-compatible target and affected plans can reconcile safely.
    if (
        summary["operation"] == "rollback"
        and successor_plan["site"]["dispatcher_sha256"]
        != site_plan["site"]["dispatcher_sha256"]
    ):
        fail(
            "successor_plan_path",
            "rollback target and affected site dispatcher digests disagree",
        )
    dispatchers = _validated_dispatchers(site_plan, dispatcher_root)
    output_root.mkdir()
    write_canonical_json(
        output_root / "channels.json",
        summary["mutation"]["proposed_index"],
        refuse_existing=True,
    )
    if (
        sha256_file(output_root / "channels.json")
        != summary["mutation"]["proposed_index_sha256"]
    ):
        fail("$.mutation.proposed_index_sha256", "staged index bytes drifted")
    facts = generate_site_release_facts(
        summary["mutation"]["proposed_index"],
        source_plan_sha256=summary["successor"]["plan_sha256"],
        release_index_sha256=summary["mutation"]["proposed_index_sha256"],
        dispatchers=dispatchers,
    )
    write_canonical_json(
        output_root / "stable-site-release-facts.json",
        facts,
        refuse_existing=True,
    )
    return {
        "release_index_sha256": summary["mutation"]["proposed_index_sha256"],
        "site_facts_sha256": sha256_file(
            output_root / "stable-site-release-facts.json"
        ),
    }


def materialize_incident_signoff(
    *,
    prepare_summary_path: Path,
    request_path: Path,
    withdrawal_evidence_path: Path,
    site_facts_path: Path,
    site_run_path: Path,
    smoke_root: Path,
    run_id: int,
    approver: str,
    release_signoff_path: Path | None = None,
) -> dict[str, Any]:
    """Build immutable incident sign-off from realized public evidence."""
    summary = validate_incident_prepare_summary(
        load_json_strict(prepare_summary_path, require_canonical=True)
    )
    run_id = require_positive_int(run_id, "run_id")
    approver = require_nonempty_string(approver, "approver")
    if sha256_file(request_path) != summary["incident"]["request_sha256"]:
        fail("request_path", "does not match protected incident evidence")
    if (
        sha256_file(withdrawal_evidence_path)
        != summary["incident"]["withdrawal_evidence_sha256"]
    ):
        fail("withdrawal_evidence_path", "does not match protected incident evidence")

    facts = validate_site_release_facts(
        load_json_strict(site_facts_path, require_canonical=True),
        governed_index=summary["mutation"]["proposed_index"],
    )
    if (
        facts["source_plan_sha256"] != summary["successor"]["plan_sha256"]
        or facts["release_index_sha256"]
        != summary["mutation"]["proposed_index_sha256"]
    ):
        fail("site_facts_path", "does not bind the realized incident mutation")
    site = _site_run(site_run_path, summary)
    smoke = _smoke_evidence(smoke_root, summary)
    smoke_manifest_sha256 = sha256_bytes(canonical_json_bytes(smoke))

    if summary["operation"] == "incident-roll-forward":
        if release_signoff_path is None:
            fail("release_signoff_path", "is required for incident roll-forward")
        release_signoff = validate_release_signoff(
            load_json_strict(release_signoff_path, require_canonical=True)
        )
        if (
            release_signoff["version"] != summary["successor"]["version"]
            or release_signoff["plan_sha256"] != summary["successor"]["plan_sha256"]
            or release_signoff["channel_generation"]
            != summary["mutation"]["proposed_index"]["generation"]
        ):
            fail("release_signoff_path", "does not cross-bind the incident mutation")
        release_signoff_sha256: str = sha256_file(release_signoff_path)
    else:
        if release_signoff_path is not None:
            fail("release_signoff_path", "rollback must not supply release sign-off")
        release_signoff_sha256 = "none"

    mutations = {
        "incident-request": summary["incident"]["request_sha256"],
        "release-index": summary["mutation"]["proposed_index_sha256"],
        "site-release-facts": sha256_file(site_facts_path),
    }
    if release_signoff_sha256 != "none":
        mutations["stable-release-signoff"] = release_signoff_sha256
    closure_sha256 = sha256_bytes(
        canonical_json_bytes(
            {
                "incident_id": summary["incident"]["incident_id"],
                "operation": summary["operation"],
                "generation": summary["mutation"]["proposed_index"]["generation"],
                "site_run_id": site["run_id"],
                "smoke_sha256": smoke_manifest_sha256,
            }
        )
    )
    signoff = {
        "schema_version": 2,
        "incident_id": summary["incident"]["incident_id"],
        "operation": summary["operation"],
        "request_sha256": summary["incident"]["request_sha256"],
        "release_signoff_sha256": release_signoff_sha256,
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
                    for kind, digest in mutations.items()
                ],
            }
        ],
        "index_mutation": {
            "previous_generation": summary["mutation"]["previous_index"][
                "generation"
            ],
            "previous_sha256": summary["mutation"]["previous_index"]["sha256"],
            "realized_generation": summary["mutation"]["proposed_index"][
                "generation"
            ],
            "realized_sha256": summary["mutation"]["proposed_index_sha256"],
            "affected_version": summary["affected"]["version"],
            "successor_version": summary["successor"]["version"],
        },
        "site_reconciliation": {
            "status": "pass",
            "evidence_sha256": sha256_file(site_facts_path),
            **site,
        },
        "validation": {
            "status": "pass",
            "evidence_sha256": smoke_manifest_sha256,
        },
        "communications": {
            "status": "pass",
            "evidence_sha256": sha256_file(withdrawal_evidence_path),
        },
        "closure": {
            "status": "pass",
            "evidence_sha256": closure_sha256,
        },
    }
    return validate_incident_signoff(
        signoff,
        incident_request=load_json_strict(request_path, require_canonical=True),
    )


def _validated_dispatchers(
    plan: dict[str, Any],
    root: Path,
) -> dict[str, str]:
    entries = list(root.iterdir()) if root.is_dir() else []
    if {path.name for path in entries} != set(DISPATCHERS):
        fail("dispatcher_root", "must contain exactly index, stable, alpha, and beta")
    if any(path.is_symlink() or not path.is_file() for path in entries):
        fail("dispatcher_root", "must contain regular non-symlink files")
    actual = {name: sha256_file(root / name) for name in DISPATCHERS}
    if actual != plan["site"]["dispatcher_sha256"]:
        fail("dispatcher_root", "digests do not match the approved successor plan")
    return actual


def _site_run(path: Path, summary: dict[str, Any]) -> dict[str, Any]:
    value = load_json_strict(path, require_canonical=True)
    if not isinstance(value, dict) or set(value) != {
        "repository",
        "workflow",
        "run_id",
        "deployed_commit",
    }:
        fail("site_run_path", "must contain exact correlated site run fields")
    if (
        value["repository"] != "sifr-lang/sifr-website"
        or value["workflow"] != "release-site.yml"
        or value["deployed_commit"] != summary["site"]["base_commit"]
        or not isinstance(value["run_id"], int)
        or isinstance(value["run_id"], bool)
        or value["run_id"] < 1
    ):
        fail("site_run_path", "does not match the approved deployment")
    return value


def _smoke_evidence(root: Path, summary: dict[str, Any]) -> list[dict[str, str]]:
    entries = list(root.iterdir()) if root.is_dir() else []
    if {path.name for path in entries} != set(SMOKE_FILES.values()):
        fail("smoke_root", "does not contain the exact incident smoke evidence set")
    if any(path.is_symlink() or not path.is_file() for path in entries):
        fail("smoke_root", "must contain regular non-symlink files")
    if (root / "governed-index.json").read_bytes() != canonical_json_bytes(
        summary["mutation"]["proposed_index"]
    ):
        fail("smoke_root", "public governed index bytes drifted")
    recovery = load_json_strict(
        root / "incident-recovery.json",
        require_canonical=True,
    )
    if recovery != {
        "schema_version": 2,
        "operation": summary["operation"],
        "affected_version": summary["affected"]["version"],
        "successor_version": summary["successor"]["version"],
        "working_client": "pass",
        "out_of_band": "pass",
    }:
        fail("smoke_root", "incident recovery evidence does not match prepare")
    return [
        {
            "id": identifier,
            "status": "pass",
            "sha256": sha256_file(root / filename),
        }
        for identifier, filename in SMOKE_FILES.items()
    ]


def _mutation_identity(kind: str, summary: dict[str, Any]) -> str:
    if kind == "incident-request":
        return summary["incident"]["incident_id"]
    if kind == "stable-release-signoff":
        return summary["successor"]["version"]
    return f"generation-{summary['mutation']['proposed_index']['generation']}"
