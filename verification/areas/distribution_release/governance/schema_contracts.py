"""Cross-check governed fixtures against the checked-in JSON Schemas."""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

from verification.json_schema_202012 import JsonSchemaError, validate_instance

from .approval_waiver_selftest import approval_waiver_fixture
from .common import TARGETS, canonical_json_bytes, sha256_bytes
from .schema_bootstrap import (
    BOOTSTRAP_GENERATION,
    LEGACY_INDEX_SHA256,
    LEGACY_INDEX_SIZE_BYTES,
    expected_asset_names,
)
from .schema_negative_contracts import validate_incident_schema_negatives

SCHEMA_ROOT = Path(__file__).resolve().parents[1] / "schemas"
SHA_A = "a" * 64
SHA_B = "b" * 64
SHA_C = "c" * 64
SHA_D = "d" * 64
COMMIT = "e" * 40


def validate_schema_contracts() -> None:
    fixtures = schema_fixtures()
    schema_paths = sorted(SCHEMA_ROOT.glob("*.schema.json"))
    if {path.name for path in schema_paths} != set(fixtures):
        raise ValueError("governance schema fixture registration drifted")
    for path in schema_paths:
        validate_instance(fixtures[path.name], path)
    for expires_at in ("not-a-timestamp", "2026-08-01T00:00:00"):
        invalid_expiry = copy.deepcopy(
            fixtures["qualification_artifact_index.schema.json"]
        )
        invalid_expiry["workflow"]["expires_at"] = expires_at
        try:
            validate_instance(
                invalid_expiry,
                SCHEMA_ROOT / "qualification_artifact_index.schema.json",
            )
        except JsonSchemaError:
            pass
        else:
            raise ValueError(
                f"qualification schema accepted invalid expiry: {expires_at}"
            )
    incident_schema = SCHEMA_ROOT / "stable_incident_signoff.schema.json"
    no_completed = copy.deepcopy(fixtures["stable_incident_signoff.schema.json"])
    no_completed["attempts"][0]["status"] = "failed"
    duplicate_completed = copy.deepcopy(fixtures["stable_incident_signoff.schema.json"])
    duplicate_completed["attempts"].append(
        {**copy.deepcopy(duplicate_completed["attempts"][0]), "run_id": 2}
    )
    for invalid_signoff in (no_completed, duplicate_completed):
        try:
            validate_instance(invalid_signoff, incident_schema)
        except JsonSchemaError:
            pass
        else:
            raise ValueError(
                "incident sign-off schema accepted an invalid completed-attempt count"
            )
    bootstrap_schema = SCHEMA_ROOT / "schema_epoch_bootstrap_evidence.schema.json"
    alpha_stage = copy.deepcopy(
        fixtures["schema_epoch_bootstrap_evidence.schema.json"]
    )
    alpha_stage["stage"] = "alpha-assets"
    for key in ("alpha_evidence", "beta", "index", "public_smoke"):
        del alpha_stage[key]
    validate_instance(alpha_stage, bootstrap_schema)
    duplicate_smoke = copy.deepcopy(
        fixtures["schema_epoch_bootstrap_evidence.schema.json"]
    )
    duplicate_smoke["public_smoke"][1]["id"] = duplicate_smoke["public_smoke"][0]["id"]
    duplicate_smoke["public_smoke"][1]["sha256"] = SHA_A
    extra_asset = copy.deepcopy(fixtures["schema_epoch_bootstrap_evidence.schema.json"])
    extra_asset["alpha"]["published_assets"][
        "sifr-0.9.9-alpha.1-x86_64-apple-darwin.tar.gz"
    ] = SHA_A
    alpha_with_beta = copy.deepcopy(
        fixtures["schema_epoch_bootstrap_evidence.schema.json"]
    )
    alpha_with_beta["stage"] = "alpha-assets"
    for key in ("alpha_evidence", "index", "public_smoke"):
        del alpha_with_beta[key]
    for invalid_bootstrap in (duplicate_smoke, extra_asset, alpha_with_beta):
        try:
            validate_instance(invalid_bootstrap, bootstrap_schema)
        except JsonSchemaError:
            pass
        else:
            raise ValueError(
                "bootstrap evidence schema accepted an invalid governed collection"
            )
    drill_schema = SCHEMA_ROOT / "protected_release_drill_evidence.schema.json"
    unknown_scenario = copy.deepcopy(
        fixtures["protected_release_drill_evidence.schema.json"]
    )
    unknown_scenario["scenarios"][0]["name"] = "production"
    duplicate_test = copy.deepcopy(
        fixtures["protected_release_drill_evidence.schema.json"]
    )
    duplicate_test["scenarios"][0]["tests"].append(
        duplicate_test["scenarios"][0]["tests"][0]
    )
    for invalid_drill in (unknown_scenario, duplicate_test):
        try:
            validate_instance(invalid_drill, drill_schema)
        except JsonSchemaError:
            pass
        else:
            raise ValueError(
                "protected drill schema accepted an invalid governed collection"
            )
    mutation_schema = SCHEMA_ROOT / "stable_index_mutation_evidence.schema.json"
    invalid_transition = copy.deepcopy(
        fixtures["stable_index_mutation_evidence.schema.json"]
    )
    invalid_transition["transition"] = "rollback"
    invalid_previous_generation = copy.deepcopy(
        fixtures["stable_index_mutation_evidence.schema.json"]
    )
    invalid_previous_generation["previous_index"]["generation"] = 0
    for invalid_mutation in (invalid_transition, invalid_previous_generation):
        try:
            validate_instance(invalid_mutation, mutation_schema)
        except JsonSchemaError:
            pass
        else:
            raise ValueError(
                "stable mutation evidence schema accepted an invalid binding"
            )
    prepare_schema = SCHEMA_ROOT / "stable_publication_prepare.schema.json"
    unknown_artifact = copy.deepcopy(
        fixtures["stable_publication_prepare.schema.json"]
    )
    unknown_artifact["artifacts"]["unknown"] = unknown_artifact["artifacts"].pop(
        "installer"
    )
    try:
        validate_instance(unknown_artifact, prepare_schema)
    except JsonSchemaError:
        pass
    else:
        raise ValueError("stable prepare schema accepted an unknown artifact identity")
    activated_initial = copy.deepcopy(
        fixtures["stable_publication_prepare.schema.json"]
    )
    activated_initial["publication_state"] = "activated"
    try:
        validate_instance(activated_initial, prepare_schema)
    except JsonSchemaError:
        pass
    else:
        raise ValueError("stable prepare schema accepted activated initial mode")
    validate_incident_schema_negatives(fixtures, SCHEMA_ROOT)
    signoff_schema = SCHEMA_ROOT / "stable_release_signoff.schema.json"
    wrong_site = copy.deepcopy(fixtures["stable_release_signoff.schema.json"])
    wrong_site["site_publication"]["repository"] = "example.invalid/site"
    try:
        validate_instance(wrong_site, signoff_schema)
    except JsonSchemaError:
        pass
    else:
        raise ValueError("stable sign-off schema accepted the wrong site repository")


def schema_fixtures() -> dict[str, Any]:
    index = preview_index()
    return {
        "qualification_artifact_index.schema.json": qualification_index(),
        "protected_release_drill_evidence.schema.json": protected_drill_evidence(),
        "release_index.schema.json": index,
        "release_profile_report.schema.json": release_report(),
        "schema_epoch_bootstrap_evidence.schema.json": schema_bootstrap_evidence(),
        "single_maintainer_approval_waiver.schema.json": (
            approval_waiver_fixture()
        ),
        "self_update_install_receipt.schema.json": install_receipt(),
        "self_update_plan.schema.json": self_update_plan(),
        "self_version.schema.json": self_version(),
        "site_publication_facts.schema.json": site_publication_facts(),
        "incident_index_mutation_evidence.schema.json": (
            incident_publication_prepare()["mutation"]
        ),
        "incident_publication_prepare.schema.json": incident_publication_prepare(),
        "stable_incident_request.schema.json": incident_request(),
        "stable_incident_signoff.schema.json": incident_signoff(),
        "stable_index_mutation_evidence.schema.json": stable_index_mutation_evidence(),
        "stable_publication_prepare.schema.json": stable_publication_prepare(),
        "stable_release_plan.schema.json": release_plan(),
        "stable_release_signoff.schema.json": release_signoff(),
        "stable_site_release_facts.schema.json": site_facts(),
    }


def protected_drill_evidence() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "environment": "stable-release-drill",
        "external_network": "blocked",
        "production_credentials": "absent",
        "scenarios": [
            {
                "name": "publication",
                "tests": [
                    "test_ga_activation",
                    "test_normal_successor",
                    "test_fail_closed_identity_and_transition",
                    "test_direct_transition_defenses",
                    "test_cli_producer",
                    "test_evidence_contract",
                ],
            }
        ],
        "status": "pass",
    }


def stable_index_mutation_evidence() -> dict[str, Any]:
    proposed = preview_index()
    proposed["generation"] = 8
    proposed["ga_status"] = "active"
    proposed["channels"]["stable"] = "0.1.0"
    proposed["channels"] = dict(sorted(proposed["channels"].items()))
    proposed["releases"]["0.1.0"] = release_record("stable")
    proposed["releases"] = dict(sorted(proposed["releases"].items()))
    return {
        "schema_version": 2,
        "transition": "ga-activation",
        "version": "0.1.0",
        "plan_sha256": SHA_A,
        "previous_index": {"generation": 7, "sha256": SHA_B},
        "proposed_index": proposed,
        "proposed_index_sha256": sha256_bytes(canonical_json_bytes(proposed)),
    }


def stable_publication_prepare() -> dict[str, Any]:
    mutation = stable_index_mutation_evidence()
    qualification = qualification_index()
    transported = {
        artifact["id"]: artifact for artifact in qualification["artifacts"]
    }
    return {
        "schema_version": 2,
        "operation": "ga-activation",
        "mode": "initial",
        "publication_state": "pending",
        "next_generation": 8,
        "version": "0.1.0",
        "evidence": {
            "commit": COMMIT,
            "candidate_path": "plans/releases/candidates/0.1.0",
            "plan_sha256": SHA_A,
        },
        "source": {
            "commit": COMMIT,
            "submodules": {"editor_integrations": "f" * 40},
        },
        "release_report": {"id": "release-report-a", "sha256": SHA_B},
        "qualification": {
            "id": "qualification-42-1",
            "sha256": SHA_C,
            "run_id": 42,
            "run_attempt": 1,
            "expires_at": "2026-08-20T00:00:00Z",
        },
        "live_index": {"generation": 7, "sha256": SHA_B},
        "mutation": mutation,
        "artifacts": {
            artifact_id: {
                "name": artifact["name"],
                "sha256": artifact["sha256"],
                "size_bytes": artifact["size_bytes"],
                "workflow_artifact_id": artifact["workflow_artifact_id"],
                "workflow_artifact_name": artifact["workflow_artifact_name"],
            }
            for artifact_id, artifact in sorted(transported.items())
        },
        "marketplace": {
            "publisher": "sifr",
            "extension": "sifr-vscode",
            "version": "0.2.0",
            "vsix_sha256": transported["vsix"]["sha256"],
        },
        "site": {
            "repository": "sifr-lang/sifr-website",
            "base_commit": "1" * 40,
        },
    }


def incident_publication_prepare() -> dict[str, Any]:
    release_prepare = stable_publication_prepare()
    release_prepare["operation"] = "incident-roll-forward"
    release_prepare["mutation"]["transition"] = "incident-roll-forward"
    release_prepare["incident"] = {
        "incident_id": "inc-2026-001",
        "request_sha256": SHA_D,
        "affected_version": "0.0.9",
        "affected_plan_sha256": SHA_C,
    }
    mutation = {
        "schema_version": 2,
        "operation": "incident-roll-forward",
        "request_sha256": SHA_D,
        "affected_plan_sha256": SHA_C,
        "successor_plan_sha256": SHA_A,
        "affected_version": "0.0.9",
        "successor_version": "0.1.0",
        "previous_index": release_prepare["mutation"]["previous_index"],
        "proposed_index": release_prepare["mutation"]["proposed_index"],
        "proposed_index_sha256": release_prepare["mutation"][
            "proposed_index_sha256"
        ],
        "plan_sha256": SHA_A,
    }
    return {
        "schema_version": 2,
        "operation": "incident-roll-forward",
        "mode": "initial",
        "publication_state": "pending",
        "next_generation": 8,
        "incident": {
            "commit": COMMIT,
            "path": (
                "plans/releases/incidents/inc-2026-001/"
                "stable-incident-request.json"
            ),
            "incident_id": "inc-2026-001",
            "request_sha256": SHA_D,
            "withdrawal_evidence_sha256": SHA_B,
        },
        "affected": {"version": "0.0.9", "plan_sha256": SHA_C},
        "successor": {"version": "0.1.0", "plan_sha256": SHA_A},
        "live_index": {"generation": 7, "sha256": SHA_B},
        "mutation": mutation,
        "site": release_prepare["site"],
        "release_prepare": release_prepare,
    }


def schema_bootstrap_evidence() -> dict[str, Any]:
    def release(version: str) -> dict[str, Any]:
        return {
            "version": version,
            "source_commit": COMMIT,
            "release_record_sha256": SHA_A,
            "published_assets": {
                name: SHA_B for name in sorted(expected_asset_names(version))
            },
        }

    return {
        "schema_version": 2,
        "operation": "schema-epoch-bootstrap",
        "stage": "preview-index",
        "run_id": 42,
        "run_attempt": 1,
        "initiator": "release-initiator",
        "approval_policy": {
            "mode": "distinct-reviewer",
            "waiver_sha256": "none",
        },
        "approvers": ["release-reviewer"],
        "prepare_summary_sha256": SHA_A,
        "legacy_index": {
            "sha256": LEGACY_INDEX_SHA256,
            "size_bytes": LEGACY_INDEX_SIZE_BYTES,
        },
        "alpha": release("0.1.0-alpha.2"),
        "alpha_evidence": {
            "sha256": SHA_B,
            "run_id": 41,
            "run_attempt": 1,
            "initiator": "alpha-initiator",
            "approval_policy": {
                "mode": "distinct-reviewer",
                "waiver_sha256": "none",
            },
            "approvers": ["alpha-reviewer"],
            "prepare_summary_sha256": SHA_C,
        },
        "beta": release("0.1.0-beta.15"),
        "index": {"generation": BOOTSTRAP_GENERATION, "sha256": SHA_C},
        "public_smoke": [
            {"id": smoke_id, "status": "pass", "sha256": SHA_D}
            for smoke_id in (
                "dispatcher-default",
                "dispatcher-stable-rejection",
                "governance-index",
                "installed-self-update",
            )
        ],
    }


def site_publication_facts() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "contract": "sifr-site-publication-binding-v2",
        "publication_attempt": "run-42-1",
        "source_commit": COMMIT,
        "site_base_commit": "1" * 40,
        "release_plan_sha256": SHA_A,
        "release_index": {"generation": 9, "sha256": SHA_B},
        "dispatcher_default_channel": "beta",
        "dispatchers": {
            "index": SHA_A,
            "stable": SHA_B,
            "alpha": SHA_C,
            "beta": SHA_D,
        },
    }


def release_record(channel: str) -> dict[str, Any]:
    return {
        "channel": channel,
        "status": "active",
        "source_commit": COMMIT,
        "installer_sha256": SHA_A,
        "targets": {
            target: {
                "artifact_sha256": SHA_B,
                "sysroot_content_sha256": SHA_C,
            }
            for target in TARGETS
        },
    }


def preview_index() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "generation": 7,
        "ga_status": "preview",
        "channels": {"alpha": "0.1.0-alpha.2", "beta": "0.1.0-beta.2"},
        "releases": {
            "0.1.0-alpha.2": release_record("alpha"),
            "0.1.0-beta.2": release_record("beta"),
        },
    }


def qualification_index() -> dict[str, Any]:
    version = "0.1.0"
    prefix = f"sifr-stable-candidate-{version}-{COMMIT}-"
    artifacts: list[dict[str, Any]] = []
    for workflow_artifact_id, target in enumerate(TARGETS, start=1):
        workflow_artifact_name = f"{prefix}{target}"
        for kind, name in (
            ("binary-archive", f"sifr-{version}-{target}.tar.gz"),
            ("checksum", f"sifr-{version}-{target}.tar.gz.sha256"),
            ("sysroot", f"sifr-{version}-{target}-sysroot.tar.gz"),
        ):
            artifacts.append(
                {
                    "id": f"{kind}-{target}",
                    "kind": kind,
                    "name": name,
                    "sha256": SHA_A,
                    "size_bytes": 1,
                    "workflow_artifact_id": workflow_artifact_id,
                    "workflow_artifact_name": workflow_artifact_name,
                    "expires_at": "2026-08-20T00:00:00Z",
                    "target": target,
                }
            )
        artifacts.append(
            {
                "id": f"qualification-report-{target}",
                "kind": "report",
                "name": f"qualification-{target}.json",
                "sha256": SHA_B,
                "size_bytes": 1,
                "workflow_artifact_id": workflow_artifact_id,
                "workflow_artifact_name": workflow_artifact_name,
                "expires_at": "2026-08-20T00:00:00Z",
            }
        )
    artifacts.extend(
        [
            {
                "id": "installer",
                "kind": "installer",
                "name": f"sifr-installer-{version}",
                "sha256": SHA_A,
                "size_bytes": 1,
                "workflow_artifact_id": 10,
                "workflow_artifact_name": f"{prefix}assemble",
                "expires_at": "2026-08-20T00:00:00Z",
            },
            {
                "id": "checksums",
                "kind": "checksums",
                "name": "checksums.txt",
                "sha256": SHA_B,
                "size_bytes": 1,
                "workflow_artifact_id": 10,
                "workflow_artifact_name": f"{prefix}assemble",
                "expires_at": "2026-08-20T00:00:00Z",
            },
            {
                "id": "vsix",
                "kind": "vsix",
                "name": "sifr-vscode-0.2.0.vsix",
                "sha256": SHA_C,
                "size_bytes": 1,
                "workflow_artifact_id": 11,
                "workflow_artifact_name": f"{prefix}editor",
                "expires_at": "2026-08-20T00:00:00Z",
            },
            {
                "id": "editor-qualification-report",
                "kind": "report",
                "name": "qualification-editor.json",
                "sha256": SHA_D,
                "size_bytes": 1,
                "workflow_artifact_id": 11,
                "workflow_artifact_name": f"{prefix}editor",
                "expires_at": "2026-08-20T00:00:00Z",
            },
        ]
    )
    return {
        "schema_version": 2,
        "candidate_version": version,
        "source_commit": COMMIT,
        "submodules": {"editor_integrations": "f" * 40},
        "workflow": {
            "repository": "sifr-lang/sifr",
            "run_id": 1,
            "run_attempt": 1,
            "retention_days": 30,
            "overwrite": False,
            "expires_at": "2026-08-20T00:00:00Z",
        },
        "artifacts": artifacts,
    }


def release_plan() -> dict[str, Any]:
    targets = [
        {
            "triple": target,
            "builder": {
                "aarch64-apple-darwin": "macos-15",
                "x86_64-apple-darwin": "macos-15-intel",
                "aarch64-unknown-linux-gnu": "ubuntu-24.04-arm",
                "x86_64-unknown-linux-gnu": "ubuntu-24.04",
            }[target],
            "binary_sha256": SHA_A,
            "sysroot_sha256": SHA_C,
            "archive_sha256": SHA_B,
            "checksum_sha256": SHA_D,
            "sifr_version": "0.1.0",
            "installer_version": "0.1.0",
            "receipt_channel": "stable",
            "sysroot_version": "0.1.0",
            "sysroot_target": target,
        }
        for target in TARGETS
    ]
    return {
        "schema_version": 2,
        "plan_id": "stable-0.1.0-eeeeeeeeeeee",
        "version": "0.1.0",
        "transition": "ga-activation",
        "source_commit": COMMIT,
        "submodules": {"editor_integrations": "f" * 40},
        "cargo_lock_sha256": SHA_A,
        "toolchain": {
            "rustc": "rustc fixture",
            "cargo": "cargo fixture",
            "profile_manifest_sha256": SHA_B,
        },
        "expected_stable_predecessor": "none",
        "desired_release": release_record("stable"),
        "rollback_target": "none",
        "targets": targets,
        "installer_sha256": SHA_A,
        "release_profile_report": {"id": "release-report-a", "sha256": SHA_A},
        "qualification_artifact_index": {"id": "qualification-a", "sha256": SHA_B},
        "rust_interop": {
            "compatibility_matrix_sha256": SHA_A,
            "stable_support_claims_sha256": SHA_B,
            "advertised_claim_ids": ["rust-crate-bridge"],
            "validation_report_sha256": SHA_C,
        },
        "documentation_report": {"id": "docs-a", "sha256": SHA_D},
        "site": {
            "repository": "sifr-lang/sifr-website",
            "base_commit": "1" * 40,
            "dispatcher_sha256": {
                "index": SHA_A,
                "stable": SHA_B,
                "alpha": SHA_C,
                "beta": SHA_D,
            },
            "facts_schema_sha256": SHA_A,
            "facts_generator_sha256": SHA_B,
        },
        "vscode": {
            "submodule_path": "editor_integrations",
            "package_path": "editor_integrations/vscode",
            "version": "0.2.0",
            "vsix_sha256": SHA_C,
            "compiler_compatibility": ">=0.1.0,<0.2.0",
            "validation_report_sha256": SHA_D,
        },
        "release_notes_sha256": SHA_A,
    }


def release_report() -> dict[str, Any]:
    suites = {
        "area_rust_interop": [
            ("rust_interop", name)
            for name in (
                "matrix",
                "tiers",
                "compatibility-matrix",
                "stale-drafts",
                "stable-candidate",
            )
        ],
        "area_developer_tooling": [
            ("developer_tooling", "full"),
            ("developer_tooling", "editor-release"),
        ],
        "area_documentation": [
            ("documentation", "structure"),
            ("documentation", "ga-release"),
        ],
        "area_distribution_release": [
            ("distribution_release", "full"),
            ("distribution_release", "qualification"),
            ("distribution_release", "evidence-custody"),
            ("distribution_release", "incident-governance"),
            ("distribution_release", "epoch-bootstrap"),
            ("distribution_release", "protected-drill"),
            ("distribution_release", "stable-prepare"),
            ("distribution_release", "stable-publish-primitives"),
            ("distribution_release", "stable-publication"),
        ],
    }
    return {
        "schema_version": 2,
        "report_id": "release-eeeeeeeeeeee-aaaaaaaaaaaa",
        "source": {
            "commit": COMMIT,
            "clean": True,
            "unresolved": False,
            "submodules": {"editor_integrations": "f" * 40},
        },
        "profile": {
            "name": "release",
            "manifest_sha256": SHA_A,
            "expanded_selected_areas": [
                {
                    "area": area,
                    "suites": names,
                }
                for area, names in (
                    (
                        "rust_interop",
                        [
                            "matrix",
                            "tiers",
                            "compatibility-matrix",
                            "stale-drafts",
                            "stable-candidate",
                        ],
                    ),
                    ("developer_tooling", ["full", "editor-release"]),
                    ("documentation", ["structure", "ga-release"]),
                    (
                        "distribution_release",
                        [
                            "full",
                            "qualification",
                            "evidence-custody",
                            "incident-governance",
                            "epoch-bootstrap",
                            "protected-drill",
                            "stable-prepare",
                            "stable-publish-primitives",
                            "stable-publication",
                        ],
                    ),
                )
            ],
        },
        "command": ["scripts/run_all_tests.sh", "--profile", "release"],
        "toolchain": {
            "rustc": "rustc fixture",
            "cargo": "cargo fixture",
            "uv": "uv fixture",
            "python": "python fixture",
        },
        "overall_status": "pass",
        "steps": [
            {
                "name": step,
                "status": "pass",
                "elapsed_ms": 1,
                "suite_results": [
                    {
                        "area": area,
                        "suite": suite,
                        "status": "pass",
                        "case_ids": [f"{suite}:case"],
                        "result_artifact_sha256": SHA_A,
                    }
                    for area, suite in entries
                ],
            }
            for step, entries in suites.items()
        ],
        "result_artifacts": [
            {"path": "target/verification/example.json", "sha256": SHA_A}
        ],
    }


def incident_request() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "incident_id": "inc-2026-001",
        "operation": "rollback",
        "trigger": "stable smoke regression",
        "affected_release": {"version": "0.1.0", "plan_sha256": SHA_A},
        "withdrawal": {"reason": "regression", "evidence_sha256": SHA_B},
        "rollback_target": {"version": "0.0.9", "plan_sha256": SHA_C},
    }


def attempt() -> dict[str, Any]:
    return {
        "run_id": 1,
        "mode": "initial",
        "approver": "release-reviewer",
        "status": "completed",
        "mutations": [
            {"kind": "release-index", "identity": "generation-8", "sha256": SHA_A}
        ],
    }


def release_signoff() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "version": "0.1.0",
        "plan_sha256": SHA_A,
        "initiator": "release-initiator",
        "approval_policy": {
            "mode": "distinct-reviewer",
            "waiver_sha256": "none",
        },
        "attempts": [attempt()],
        "published_assets": {"sifr-installer-0.1.0": SHA_A},
        "marketplace": {
            "publisher": "sifr",
            "extension": "sifr",
            "version": "0.1.0",
            "vsix_sha256": SHA_B,
        },
        "channel_generation": 8,
        "site_publication": {
            "repository": "sifr-lang/sifr-website",
            "workflow": "release-site.yml",
            "run_id": 11,
            "deployed_commit": COMMIT,
        },
        "site_facts_sha256": SHA_C,
        "post_publication_smoke": [
            {"id": f"smoke-{index}", "status": "pass", "sha256": SHA_D}
            for index in range(4)
        ],
    }


def incident_signoff() -> dict[str, Any]:
    def evidence(digest: str) -> dict[str, str]:
        return {"status": "pass", "evidence_sha256": digest}

    return {
        "schema_version": 2,
        "incident_id": "inc-2026-001",
        "operation": "rollback",
        "request_sha256": SHA_A,
        "release_signoff_sha256": "none",
        "attempts": [attempt()],
        "index_mutation": {
            "previous_generation": 8,
            "previous_sha256": SHA_A,
            "realized_generation": 9,
            "realized_sha256": SHA_B,
            "affected_version": "0.1.0",
            "successor_version": "0.0.9",
        },
        "site_reconciliation": {
            **evidence(SHA_A),
            "repository": "sifr-lang/sifr-website",
            "workflow": "release-site.yml",
            "run_id": 11,
            "deployed_commit": COMMIT,
        },
        "validation": evidence(SHA_B),
        "communications": evidence(SHA_C),
        "closure": evidence(SHA_D),
    }


def site_facts() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "generation": 8,
        "stable_version": "0.1.0",
        "stable_status": "active",
        "source_plan_sha256": SHA_A,
        "release_index_sha256": SHA_B,
        "dispatchers": {
            "index": SHA_A,
            "stable": SHA_B,
            "alpha": SHA_C,
            "beta": SHA_D,
        },
        "withdrawals": [],
    }


def install_receipt() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "name": "sifr",
        "version": "0.1.0",
        "channel": "stable",
        "target": "aarch64-apple-darwin",
        "install_dir": "/tmp/sifr/bin",
        "binary_path": "/tmp/sifr/bin/sifr",
        "sysroot_path": "/tmp/sifr",
        "sysroot_schema_version": 1,
        "sysroot_sifr_version": "0.1.0",
        "sysroot_target_triple": "aarch64-apple-darwin",
        "sysroot_content_sha256": SHA_A,
        "artifact": "sifr-0.1.0-aarch64-apple-darwin.tar.gz",
        "modify_path": False,
    }


def self_version() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "current_executable": "/tmp/sifr/bin/sifr",
        "current_version": "0.1.0",
        "receipt_version": "0.1.0",
        "install_dir": "/tmp/sifr/bin",
        "binary_path": "/tmp/sifr/bin/sifr",
        "sysroot_path": "/tmp/sifr",
        "sysroot_schema_version": 1,
        "sysroot_sifr_version": "0.1.0",
        "sysroot_target_triple": "aarch64-apple-darwin",
        "channel": "stable",
        "target": "aarch64-apple-darwin",
        "matches_receipt": True,
        "warnings": [],
    }


def self_update_plan() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "current_version": "0.1.0",
        "target_version": "0.1.1",
        "receipt_channel": "stable",
        "requested_channel": "stable",
        "resolved_channel": "stable",
        "install_dir": "/tmp/sifr/bin",
        "binary_path": "/tmp/sifr/bin/sifr",
        "sysroot_path": "/tmp/sifr",
        "installer_url": "https://example.invalid/sifr-installer-0.1.1",
        "action": "update",
        "force": False,
        "would_run_installer": True,
        "warnings": [],
    }
