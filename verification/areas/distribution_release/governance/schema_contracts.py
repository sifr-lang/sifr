"""Cross-check governed fixtures against the checked-in JSON Schemas."""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

from verification.json_schema_202012 import JsonSchemaError, validate_instance

from .common import TARGETS

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


def schema_fixtures() -> dict[str, Any]:
    index = preview_index()
    return {
        "qualification_artifact_index.schema.json": qualification_index(),
        "release_index.schema.json": index,
        "release_profile_report.schema.json": release_report(),
        "self_update_install_receipt.schema.json": install_receipt(),
        "self_update_plan.schema.json": self_update_plan(),
        "self_version.schema.json": self_version(),
        "stable_incident_request.schema.json": incident_request(),
        "stable_incident_signoff.schema.json": incident_signoff(),
        "stable_release_plan.schema.json": release_plan(),
        "stable_release_signoff.schema.json": release_signoff(),
        "stable_site_release_facts.schema.json": site_facts(),
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
                "name": "sifr-vscode-0.1.0.vsix",
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
            "repository": "sifr-lang/sifr-blog-website",
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
            "version": "0.1.0",
            "vsix_sha256": SHA_C,
            "compiler_compatibility": ">=0.1.0 <0.2.0",
            "validation_report_sha256": SHA_D,
        },
        "release_notes_sha256": SHA_A,
    }


def release_report() -> dict[str, Any]:
    suites = {
        "rust_interop_checks": [
            ("rust_interop", name)
            for name in (
                "matrix",
                "tiers",
                "compatibility-matrix",
                "stale-drafts",
                "stable-candidate",
            )
        ],
        "developer_tooling_checks": [
            ("developer_tooling", "full"),
            ("developer_tooling", "editor-release"),
        ],
        "documentation_checks": [("documentation", "structure")],
        "distribution_validation": [
            ("distribution_release", "full"),
            ("distribution_release", "qualification"),
            ("distribution_release", "evidence-custody"),
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
                    ("documentation", ["structure"]),
                    (
                        "distribution_release",
                        ["full", "qualification", "evidence-custody"],
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
        "attempts": [attempt()],
        "published_assets": {"sifr-installer-0.1.0": SHA_A},
        "marketplace": {
            "publisher": "sifr",
            "extension": "sifr",
            "version": "0.1.0",
            "vsix_sha256": SHA_B,
        },
        "channel_generation": 8,
        "site_facts_sha256": SHA_C,
        "post_publication_smoke": [
            {"id": f"smoke-{index}", "status": "pass", "sha256": SHA_D}
            for index in range(4)
        ],
    }


def incident_signoff() -> dict[str, Any]:
    evidence = lambda digest: {"status": "pass", "evidence_sha256": digest}
    return {
        "schema_version": 2,
        "incident_id": "inc-2026-001",
        "request_sha256": SHA_A,
        "attempts": [attempt()],
        "index_mutation": {
            "previous_generation": 8,
            "previous_sha256": SHA_A,
            "realized_generation": 9,
            "realized_sha256": SHA_B,
            "affected_version": "0.1.0",
            "successor_version": "0.0.9",
        },
        "site_reconciliation": evidence(SHA_A),
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
