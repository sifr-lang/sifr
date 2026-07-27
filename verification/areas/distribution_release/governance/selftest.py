"""Mutation tests for all stable release-governance contracts."""

from __future__ import annotations

import copy
import importlib.util
import json
import tempfile
from pathlib import Path
from typing import Any, Callable

from .artifact_index import validate_qualification_artifact_index
from .common import (
    GovernanceError,
    TARGETS,
    canonical_json_bytes,
    load_json_strict,
    sha256_bytes,
)
from .evidence_custody import (
    require_comparison_base,
    validate_candidate_directory,
    validate_changed_path_set,
)
from .incident import validate_incident_request, validate_incident_signoff
from .release_index import (
    propose_preview_release,
    validate_release_index,
    validate_release_index_transition,
)
from .release_plan import (
    generate_site_release_facts,
    validate_release_plan,
    validate_release_signoff,
    validate_site_release_facts,
)
from .release_report import validate_release_profile_report
from .schema_contracts import qualification_index, validate_schema_contracts
from .surface_contracts import (
    validate_install_receipt,
    validate_self_update_plan,
    validate_self_version,
)

AREA_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = AREA_ROOT.parents[2]
SHA_A = "a" * 64
SHA_B = "b" * 64
SHA_C = "c" * 64
SHA_D = "d" * 64
COMMIT = "e" * 40


def run_self_tests() -> int:
    tests = (
        test_schemas_use_epoch_two,
        test_stable_gate_inventory,
        test_release_tooling_expansion,
        test_release_index_mutations,
        test_release_index_transitions,
        test_release_plan_mutations,
        test_incident_mutations,
        test_signoff_mutations,
        test_site_facts_mutations,
        test_artifact_index_mutations,
        test_surface_contract_mutations,
        test_release_report_mutations,
        test_evidence_custody_mutations,
        test_strict_loader_rejects_duplicate_keys,
    )
    for test in tests:
        test()
        print(f"governance-self-test pass: {test.__name__}")
    print(f"governance self-tests ok: tests={len(tests)}")
    return 0


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
        "channels": {
            "alpha": "0.1.0-alpha.2",
            "beta": "0.1.0-beta.2",
        },
        "releases": {
            "0.1.0-alpha.2": release_record("alpha"),
            "0.1.0-beta.2": release_record("beta"),
        },
    }


def active_index() -> dict[str, Any]:
    index = preview_index()
    index["generation"] = 8
    index["ga_status"] = "active"
    index["channels"]["stable"] = "0.1.0"
    index["releases"]["0.1.0"] = release_record("stable")
    return index


def valid_plan(*, transition: str = "ga-activation") -> dict[str, Any]:
    desired = release_record("stable")
    target_rows = []
    builders = {
        "aarch64-apple-darwin": "macos-15",
        "x86_64-apple-darwin": "macos-15-intel",
        "aarch64-unknown-linux-gnu": "ubuntu-24.04-arm",
        "x86_64-unknown-linux-gnu": "ubuntu-24.04",
    }
    for target in TARGETS:
        target_rows.append(
            {
                "triple": target,
                "builder": builders[target],
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
        )
    plan: dict[str, Any] = {
        "schema_version": 2,
        "plan_id": "stable-0.1.0-eeeeeeeeeeee",
        "version": "0.1.0",
        "transition": transition,
        "source_commit": COMMIT,
        "submodules": {"editor_integrations": "f" * 40},
        "cargo_lock_sha256": SHA_A,
        "toolchain": {
            "rustc": "rustc 1.90.0",
            "cargo": "cargo 1.90.0",
            "profile_manifest_sha256": SHA_B,
        },
        "expected_stable_predecessor": "none",
        "desired_release": desired,
        "rollback_target": "none",
        "targets": target_rows,
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
            "version": "0.1.0",
            "vsix_sha256": SHA_C,
            "compiler_compatibility": ">=0.1.0 <0.2.0",
            "validation_report_sha256": SHA_D,
        },
        "release_notes_sha256": SHA_A,
    }
    if transition == "normal":
        predecessor = {"version": "0.0.9", "status": "active", "plan_sha256": SHA_D}
        plan["expected_stable_predecessor"] = predecessor
        plan["rollback_target"] = {"version": "0.0.9", "plan_sha256": SHA_D}
    elif transition == "incident-roll-forward":
        plan["incident_request_sha256"] = SHA_D
    return plan


def valid_attempt() -> dict[str, Any]:
    return {
        "run_id": 10,
        "mode": "initial",
        "approver": "release-reviewer",
        "status": "completed",
        "mutations": [{"kind": "release-index", "identity": "generation-8", "sha256": SHA_A}],
    }


def valid_incident_request() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "incident_id": "inc-2026-001",
        "operation": "rollback",
        "trigger": "stable smoke regression",
        "affected_release": {"version": "0.1.0", "plan_sha256": SHA_A},
        "withdrawal": {"reason": "regression", "evidence_sha256": SHA_B},
        "rollback_target": {"version": "0.0.9", "plan_sha256": SHA_C},
    }


def valid_report() -> dict[str, Any]:
    suite_map = {
        "rust_interop_checks": [
            ("rust_interop", suite)
            for suite in ("matrix", "tiers", "compatibility-matrix", "stale-drafts", "stable-candidate")
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
            ("distribution_release", "incident-governance"),
        ],
    }
    steps = []
    for name, suites in suite_map.items():
        steps.append(
            {
                "name": name,
                "status": "pass",
                "elapsed_ms": 1,
                "suite_results": [
                    {
                        "area": area,
                        "suite": suite,
                        "status": "pass",
                        "case_ids": [
                            f"{suite}:case"
                            if suite != "editor-release"
                            else "editor-release:vscode-extension"
                        ],
                        "result_artifact_sha256": SHA_A,
                    }
                    for area, suite in suites
                ],
            }
        )
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
                    "suites": sorted(suites),
                }
                for area, suites in (
                    (
                        "rust_interop",
                        {"matrix", "tiers", "compatibility-matrix", "stale-drafts", "stable-candidate"},
                    ),
                    ("developer_tooling", {"full", "editor-release"}),
                    ("documentation", {"structure"}),
                    (
                        "distribution_release",
                        {
                            "full",
                            "qualification",
                            "evidence-custody",
                            "incident-governance",
                        },
                    ),
                )
            ],
        },
        "command": ["scripts/run_all_tests.sh", "--profile", "release"],
        "toolchain": {
            "rustc": "rustc 1.90.0",
            "cargo": "cargo 1.90.0",
            "uv": "uv 0.9.28",
            "python": "Python 3.13.0",
        },
        "overall_status": "pass",
        "steps": steps,
        "result_artifacts": [
            {"path": "target/verification/example.json", "sha256": SHA_A}
        ],
    }


def expect_rejected(
    validator: Callable[[Any], Any],
    payload: Any,
    *,
    contains: str | None = None,
) -> None:
    try:
        validator(payload)
    except GovernanceError as exc:
        if contains is not None and contains not in str(exc):
            raise AssertionError(f"expected error containing {contains!r}, got {exc}") from exc
        return
    raise AssertionError("invalid mutation unexpectedly passed")


def mutate(payload: dict[str, Any], callback: Callable[[dict[str, Any]], None]) -> dict[str, Any]:
    changed = copy.deepcopy(payload)
    callback(changed)
    return changed


def test_schemas_use_epoch_two() -> None:
    def contains_default_keyword(value: Any) -> bool:
        if isinstance(value, dict):
            return "default" in value or any(
                contains_default_keyword(item) for item in value.values()
            )
        return isinstance(value, list) and any(
            contains_default_keyword(item) for item in value
        )

    schema_paths = sorted((AREA_ROOT / "schemas").glob("*.schema.json"))
    assert schema_paths
    for path in schema_paths:
        schema = json.loads(path.read_text(encoding="utf-8"))
        assert "schema_version" in schema["required"], path
        assert schema["properties"]["schema_version"] == {"const": 2}, path
        assert not contains_default_keyword(schema), path
        assert '"rc"' not in path.read_text(encoding="utf-8"), path
    validate_schema_contracts()


def test_stable_gate_inventory() -> None:
    path = REPO_ROOT / "plans" / "releases" / "stable_gate_inventory.json"
    inventory = json.loads(path.read_text(encoding="utf-8"))
    if set(inventory) != {"schema_version", "owner", "gates"}:
        raise AssertionError("stable gate inventory fields drifted")
    if inventory["schema_version"] != 2 or inventory["owner"] != "release/distribution":
        raise AssertionError("stable gate inventory epoch/owner drifted")
    gates = inventory["gates"]
    if not isinstance(gates, list) or not gates:
        raise AssertionError("stable gate inventory is empty")
    ids: set[str] = set()
    required = {
        "id",
        "location",
        "owner",
        "current_behavior",
        "activation_boundary",
        "disposition",
    }
    for gate in gates:
        if not isinstance(gate, dict) or set(gate) != required:
            raise AssertionError(f"stable gate has invalid fields: {gate}")
        if not all(isinstance(gate[field], str) and gate[field] for field in required):
            raise AssertionError(f"stable gate has an empty owned field: {gate}")
        if gate["id"] in ids:
            raise AssertionError(f"duplicate stable gate: {gate['id']}")
        ids.add(gate["id"])
        if not (REPO_ROOT / gate["location"]).exists():
            raise AssertionError(f"stable gate location does not exist: {gate['location']}")


def test_release_tooling_expansion() -> None:
    profile = json.loads(
        (REPO_ROOT / "verification" / "profiles" / "release.json").read_text(encoding="utf-8")
    )
    if profile["legacy_facade"]["tooling_suites"] != ["full"]:
        raise AssertionError("release profile must select developer_tooling:full exactly once")
    runner_path = (
        REPO_ROOT / "verification" / "areas" / "developer_tooling" / "runner.py"
    )
    spec = importlib.util.spec_from_file_location("release_developer_tooling_runner", runner_path)
    if spec is None or spec.loader is None:
        raise AssertionError("could not load developer tooling runner")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    full_suites = getattr(module, "FULL_SUITES", [])
    if full_suites.count("editor-release") != 1:
        raise AssertionError("developer_tooling:full must expand editor-release exactly once")


def test_release_index_mutations() -> None:
    valid = preview_index()
    validate_release_index(valid)
    active = active_index()
    validate_release_index(active)
    mutations = [
        lambda item: item.pop("generation"),
        lambda item: item.update({"unknown": True}),
        lambda item: item.update({"ga_status": "invalid"}),
        lambda item: item.update({"ga_status": []}),
        lambda item: item.update({"schema_version": 1}),
        lambda item: item.update({"schema_version": "2"}),
        lambda item: item["channels"].update({"stable": "0.1.0"}),
        lambda item: item["channels"].update({"alpha": "0.1.0-beta.2"}),
        lambda item: item["releases"]["0.1.0-alpha.2"]["targets"].pop(TARGETS[0]),
        lambda item: item["releases"]["0.1.0-alpha.2"].update({"installer_sha256": "0" * 64}),
        lambda item: item["releases"]["0.1.0-alpha.2"].update({"channel": "beta"}),
        lambda item: item["releases"]["0.1.0-alpha.2"].update({"status": []}),
        lambda item: item["releases"]["0.1.0-alpha.2"].update({"incident_id": "inc-invalid"}),
    ]
    for callback in mutations:
        expect_rejected(validate_release_index, mutate(valid, callback))
    withdrawn_target = mutate(
        active,
        lambda item: item["releases"]["0.1.0"].update(
            {"status": "withdrawn", "incident_id": "inc-2026-001"}
        ),
    )
    expect_rejected(validate_release_index, withdrawn_target)
    active_without_stable = mutate(active, lambda item: item["channels"].pop("stable"))
    expect_rejected(validate_release_index, active_without_stable)


def test_release_index_transitions() -> None:
    previous = preview_index()
    proposed = active_index()
    validate_release_index_transition(previous, proposed)
    expect_rejected(
        lambda value: validate_release_index_transition(previous, value),
        mutate(proposed, lambda item: item.update({"generation": 7})),
    )
    expect_rejected(
        lambda value: validate_release_index_transition(proposed, value),
        preview_index(),
    )
    forward = propose_preview_release(
        preview_index(),
        channel="beta",
        version="0.1.0-beta.3",
        release_value=release_record("beta"),
    )
    assert forward["channels"]["beta"] == "0.1.0-beta.3"
    active_forward = propose_preview_release(
        active_index(),
        channel="alpha",
        version="0.1.0-alpha.3",
        release_value=release_record("alpha"),
    )
    assert active_forward["ga_status"] == "active"
    assert active_forward["channels"]["stable"] == "0.1.0"
    assert active_forward["channels"]["alpha"] == "0.1.0-alpha.3"
    reserved_generation = propose_preview_release(
        preview_index(),
        channel="beta",
        version="0.1.0-beta.3",
        release_value=release_record("beta"),
        proposed_generation=12,
    )
    assert reserved_generation["generation"] == 12
    expect_rejected(
        lambda value: propose_preview_release(
            value,
            channel="beta",
            version="0.1.0-beta.3",
            release_value=release_record("beta"),
            proposed_generation=value["generation"],
        ),
        preview_index(),
    )
    expect_rejected(
        lambda value: propose_preview_release(
            value,
            channel="beta",
            version="0.1.0-beta.1",
            release_value=release_record("beta"),
        ),
        preview_index(),
    )


def test_release_plan_mutations() -> None:
    ga = valid_plan()
    validate_release_plan(ga, active_index=preview_index())
    expect_rejected(
        validate_release_plan,
        mutate(ga, lambda item: item.update({"transition": []})),
    )
    expect_rejected(
        validate_release_plan,
        mutate(ga, lambda item: item.update({"plan_id": "stable-0.1.1-aaaaaaaaaaaa"})),
    )
    expect_rejected(
        validate_release_plan,
        mutate(ga, lambda item: item["rust_interop"].update({"advertised_claim_ids": [[]]})),
    )
    bad_ga = mutate(ga, lambda item: item.update({"rollback_target": {"version": "0.0.9", "plan_sha256": SHA_A}}))
    expect_rejected(validate_release_plan, bad_ga)
    normal = valid_plan(transition="normal")
    live = active_index()
    live["channels"]["stable"] = "0.0.9"
    live["releases"]["0.0.9"] = release_record("stable")
    validate_release_plan(normal, active_index=live)
    expect_rejected(
        validate_release_plan,
        mutate(normal, lambda item: item.update({"expected_stable_predecessor": "none"})),
    )
    expect_rejected(
        validate_release_plan,
        mutate(normal, lambda item: item["rollback_target"].update({"plan_sha256": SHA_A})),
    )
    incident = valid_plan(transition="incident-roll-forward")
    validate_release_plan(incident, incident_request_sha256=SHA_D)
    expect_rejected(
        validate_release_plan,
        mutate(incident, lambda item: item.pop("incident_request_sha256")),
    )
    expect_rejected(
        validate_release_plan,
        mutate(incident, lambda item: item.update({"rollback_target": {"version": "0.0.9", "plan_sha256": SHA_A}})),
    )
    expect_rejected(
        lambda value: validate_release_plan(value, incident_request_sha256=SHA_B),
        incident,
    )


def test_incident_mutations() -> None:
    request = valid_incident_request()
    live = active_index()
    live["releases"]["0.0.9"] = release_record("stable")
    validate_incident_request(
        request,
        live_index=live,
        approved_plan_digests={"0.1.0": SHA_A, "0.0.9": SHA_C},
    )
    expect_rejected(
        validate_incident_request,
        mutate(request, lambda item: item.pop("rollback_target")),
    )
    expect_rejected(
        validate_incident_request,
        mutate(request, lambda item: item.update({"operation": []})),
    )
    expect_rejected(
        lambda value: validate_incident_request(value, live_index=live),
        mutate(request, lambda item: item["affected_release"].update({"version": "0.0.8"})),
    )
    inactive = mutate(live, lambda item: item["releases"]["0.0.9"].update({"status": "withdrawn", "incident_id": "inc-old"}))
    expect_rejected(
        lambda value: validate_incident_request(value, live_index=inactive),
        request,
    )
    expect_rejected(
        lambda value: validate_incident_request(
            value,
            live_index=live,
            approved_plan_digests={"0.1.0": SHA_A, "0.0.9": SHA_B},
        ),
        request,
    )


def test_signoff_mutations() -> None:
    release_signoff = {
        "schema_version": 2,
        "version": "0.1.0",
        "plan_sha256": SHA_A,
        "attempts": [valid_attempt()],
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
    validate_release_signoff(release_signoff)
    expect_rejected(
        validate_release_signoff,
        mutate(
            release_signoff,
            lambda item: item.update({"version": "0.1.0-alpha.1"}),
        ),
    )
    expect_rejected(
        validate_release_signoff,
        mutate(release_signoff, lambda item: item["attempts"][0].update({"mutations": []})),
    )
    expect_rejected(
        validate_release_signoff,
        mutate(release_signoff, lambda item: item["attempts"][0].update({"mode": []})),
    )
    expect_rejected(
        validate_release_signoff,
        mutate(release_signoff, lambda item: item["attempts"][0].update({"status": []})),
    )
    incident_signoff = {
        "schema_version": 2,
        "incident_id": "inc-2026-001",
        "operation": "rollback",
        "request_sha256": SHA_A,
        "attempts": [valid_attempt()],
        "index_mutation": {
            "previous_generation": 8,
            "previous_sha256": SHA_A,
            "realized_generation": 9,
            "realized_sha256": SHA_B,
            "affected_version": "0.1.0",
            "successor_version": "0.0.9",
        },
        "site_reconciliation": {"status": "pass", "evidence_sha256": SHA_A},
        "validation": {"status": "pass", "evidence_sha256": SHA_B},
        "communications": {"status": "pass", "evidence_sha256": SHA_C},
        "closure": {"status": "pass", "evidence_sha256": SHA_D},
    }
    validate_incident_signoff(incident_signoff)
    expect_rejected(
        validate_incident_signoff,
        mutate(incident_signoff, lambda item: item["attempts"][0].pop("approver")),
    )


def test_site_facts_mutations() -> None:
    index = active_index()
    facts = generate_site_release_facts(
        index,
        source_plan_sha256=SHA_A,
        release_index_sha256=SHA_B,
        dispatchers={"index": SHA_A, "stable": SHA_B, "alpha": SHA_C, "beta": SHA_D},
    )
    validate_site_release_facts(facts, governed_index=index)
    expect_rejected(
        lambda value: validate_site_release_facts(value, governed_index=index),
        mutate(facts, lambda item: item.update({"stable_version": "0.1.1"})),
    )


def test_artifact_index_mutations() -> None:
    payload = qualification_index()
    validate_qualification_artifact_index(payload)
    expect_rejected(
        validate_qualification_artifact_index,
        mutate(payload, lambda item: item["artifacts"][0].pop("target")),
    )
    expect_rejected(
        validate_qualification_artifact_index,
        mutate(payload, lambda item: item["workflow"].update({"expires_at": "not-a-timestamp"})),
    )
    expect_rejected(
        validate_qualification_artifact_index,
        mutate(
            payload,
            lambda item: item["workflow"].update(
                {"expires_at": "2026-08-01T00:00:00"}
            ),
        ),
    )
    expect_rejected(
        validate_qualification_artifact_index,
        mutate(payload, lambda item: item["artifacts"][0].update({"kind": []})),
    )
    expect_rejected(
        validate_qualification_artifact_index,
        mutate(payload, lambda item: item["artifacts"][0].update({"id": " " })),
    )
    expect_rejected(
        validate_qualification_artifact_index,
        mutate(
            payload,
            lambda item: item["artifacts"].__setitem__(
                slice(None),
                [
                    artifact
                    for artifact in item["artifacts"]
                    if not (
                        artifact["kind"] == "binary-archive"
                        and artifact.get("target") == TARGETS[0]
                    )
                ],
            ),
        ),
    )
    expect_rejected(
        lambda value: validate_qualification_artifact_index(
            value,
            require_unexpired=True,
        ),
        mutate(
            payload,
            lambda item: item["workflow"].update(
                {"expires_at": "2000-01-01T00:00:00Z"}
            ),
        ),
    )


def test_surface_contract_mutations() -> None:
    receipt = {
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
    validate_install_receipt(receipt)
    expect_rejected(
        validate_install_receipt,
        mutate(receipt, lambda item: item.update({"channel": "rc"})),
    )
    expect_rejected(
        validate_install_receipt,
        mutate(receipt, lambda item: item.update({"sysroot_schema_version": 7})),
    )
    response = {
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
    validate_self_version(response)
    expect_rejected(
        validate_self_version,
        mutate(response, lambda item: item.update({"schema_version": 1})),
    )
    expect_rejected(
        validate_self_version,
        mutate(response, lambda item: item.update({"sysroot_schema_version": 7})),
    )
    plan = {
        "schema_version": 2,
        "current_version": "0.1.0",
        "target_version": "0.1.1",
        "receipt_channel": "stable",
        "requested_channel": "stable",
        "resolved_channel": "stable",
        "install_dir": "/tmp/sifr/bin",
        "binary_path": "/tmp/sifr/bin/sifr",
        "sysroot_path": "/tmp/sifr",
        "installer_url": "https://github.com/sifr-lang/sifr/releases/download/0.1.1/sifr-installer-0.1.1",
        "action": "update",
        "force": False,
        "would_run_installer": True,
        "warnings": [],
    }
    validate_self_update_plan(plan)
    expect_rejected(
        validate_self_update_plan,
        mutate(plan, lambda item: item.update({"would_run_installer": False})),
    )
    expect_rejected(
        validate_self_update_plan,
        mutate(plan, lambda item: item.update({"action": []})),
    )


def test_release_report_mutations() -> None:
    report = valid_report()
    validate_release_profile_report(report, expected_profile_sha256=SHA_A)
    mutations = [
        lambda item: item["source"].update({"clean": False}),
        lambda item: item["source"].update({"unresolved": True}),
        lambda item: item["profile"].update({"manifest_sha256": SHA_B}),
        lambda item: item.update({"overall_status": "fail"}),
        lambda item: item["steps"].pop(0),
        lambda item: item["steps"][0]["suite_results"].pop(),
        lambda item: item["result_artifacts"][0].update({"sha256": "0" * 64}),
    ]
    for index, callback in enumerate(mutations):
        validator = (
            (lambda value: validate_release_profile_report(value, expected_profile_sha256=SHA_A))
            if index == 2
            else validate_release_profile_report
        )
        expect_rejected(validator, mutate(report, callback))
    noncanonical = json.dumps(report, indent=2).encode()
    expect_rejected(
        lambda value: validate_release_profile_report(value, canonical_bytes=noncanonical),
        report,
    )


def test_evidence_custody_mutations() -> None:
    expect_rejected(
        lambda value: require_comparison_base(value, base_ref="missing"),
        "",
    )
    candidate = "plans/releases/candidates/0.1.0/stable-release-plan.json"
    validate_changed_path_set({candidate})
    validate_changed_path_set({candidate, "plans/releases/README.md"})
    path_mutations = [
        {candidate, "crates/sifr/src/main.rs"},
        {candidate, "plans/releases/candidates/0.1.1/stable-release-plan.json"},
        {"plans/releases/candidates/0.1.0/unexpected.json"},
    ]
    for paths in path_mutations:
        try:
            validate_changed_path_set(paths)
        except GovernanceError:
            continue
        raise AssertionError(f"invalid evidence custody paths passed: {paths}")

    with tempfile.TemporaryDirectory() as directory:
        candidate_dir = Path(directory) / "0.1.0"
        candidate_dir.mkdir()
        report = valid_report()
        report_bytes = canonical_json_bytes(report)
        qualification = qualification_index()
        qualification_bytes = canonical_json_bytes(qualification)
        plan = valid_plan()
        plan["release_profile_report"]["sha256"] = sha256_bytes(report_bytes)
        plan["qualification_artifact_index"]["sha256"] = sha256_bytes(qualification_bytes)
        (candidate_dir / "stable-release-plan.json").write_bytes(canonical_json_bytes(plan))
        (candidate_dir / "release-profile-report.json").write_bytes(report_bytes)
        (candidate_dir / "qualification-artifact-index.json").write_bytes(qualification_bytes)
        validate_candidate_directory(candidate_dir)
        (candidate_dir / "release-profile-report.json").write_bytes(
            canonical_json_bytes({**report, "report_id": "tampered"})
        )
        try:
            validate_candidate_directory(candidate_dir)
        except GovernanceError:
            pass
        else:
            raise AssertionError("candidate report digest mismatch passed custody")


def test_strict_loader_rejects_duplicate_keys() -> None:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "duplicate.json"
        path.write_text('{"schema_version":2,"schema_version":2}\n', encoding="utf-8")
        try:
            load_json_strict(path)
        except GovernanceError:
            return
    raise AssertionError("strict loader accepted duplicate keys")


if __name__ == "__main__":
    raise SystemExit(run_self_tests())
