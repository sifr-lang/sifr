"""Self-tests for the verification runner foundation."""

from __future__ import annotations

import tempfile
from pathlib import Path

from .areas import discover_areas
from .profiles import (
    ProfileError,
    crate_test_suites_for_mode,
    failure_reproduction_command,
    legacy_facade,
    load_all_profiles,
    selected_resource_classes,
    validate_crate_test_membership,
    validate_selected_area_suites,
)
from .results import build_result
from .schemas import load_schema, validate_all_committed_schemas, validate_data, validate_schema_requirement


def run_all() -> list[str]:
    checks = [
        ("schema self-tests", _schema_self_test),
        ("profile schema self-test", _profile_schema_self_test),
        ("crate membership self-test", _crate_membership_self_test),
        ("e2e profile self-test", _e2e_profile_self_test),
        ("runner discovery self-test", _discovery_self_test),
        ("resource class selection self-test", _resource_class_self_test),
        ("resume/failure-reproduction self-test", _failure_reproduction_self_test),
    ]
    passed: list[str] = []
    for name, check in checks:
        check()
        passed.append(name)
    return passed


def _schema_self_test() -> None:
    committed = validate_all_committed_schemas()
    required = {
        "verification/schemas/profile.schema.json",
        "verification/schemas/area.schema.json",
        "verification/schemas/suite.schema.json",
        "verification/schemas/case.schema.json",
        "verification/schemas/result.schema.json",
        "verification/schemas/owners.schema.json",
    }
    missing = required - set(committed)
    if missing:
        raise AssertionError(f"missing schema self-test coverage: {sorted(missing)}")
    try:
        validate_schema_requirement({"type": "object", "oneOf": []}, Path("bad.schema.json"))
    except Exception as exc:
        if "unsupported schema keyword 'oneOf'" not in str(exc):
            raise
    else:
        raise AssertionError("unsupported schema keyword was accepted")

    area_schema = load_schema("area.schema.json")
    invalid_area = {
        "schema_version": 1,
        "name": "core_language",
        "description": "Missing owner and wrong parallel_safe type.",
        "parallel_safe": "yes",
        "resource_classes": ["default-local"],
        "timeout_seconds": 60,
        "suites": [],
    }
    try:
        validate_data(invalid_area, area_schema, source="invalid area self-test")
    except Exception as exc:
        if "missing required key 'owner'" not in str(exc):
            raise
    else:
        raise AssertionError("invalid area manifest data was accepted")


def _profile_schema_self_test() -> None:
    profiles = load_all_profiles()
    expected = {"create-pr", "merge", "nightly", "python-interop-live", "release"}
    if set(profiles) != expected:
        raise AssertionError(f"unexpected profiles: {sorted(profiles)}")
    if profiles["python-interop-live"].get("execution_mode") != "selected-areas-only":
        raise AssertionError("python-interop-live must use selected-areas-only execution")


def _crate_membership_self_test() -> None:
    profiles = load_all_profiles()
    merge = profiles["merge"]
    full_suites = crate_test_suites_for_mode(merge, "full")
    by_id = {str(suite.get("id")): suite for suite in full_suites}
    expected_executed = {
        "sifr_type_system",
        "sifr_format",
        "sifr_lint",
        "sifr_source",
        "sifr_ir",
    }
    missing = sorted(expected_executed.difference(by_id))
    if missing:
        raise AssertionError(f"merge crate membership omitted expected suites: {missing}")
    for suite_id in sorted(expected_executed):
        suite = by_id[suite_id]
        if suite.get("status") != "blocking" or suite.get("executed_in_merge") is not True:
            raise AssertionError(f"merge crate suite is not blocking/executed: {suite_id}")

    for profile_name in ("create-pr", "merge", "nightly", "release"):
        profile = profiles[profile_name]
        profile_full_suites = crate_test_suites_for_mode(profile, "full")
        profile_smoke_suites = crate_test_suites_for_mode(profile, "smoke")
        profile_by_id = {str(suite.get("id")): suite for suite in profile_full_suites}
        codegen = profile_by_id.get("sifr_codegen")
        if not isinstance(codegen, dict):
            raise AssertionError(f"sifr_codegen missing from {profile_name} crate membership")
        if codegen.get("status") != "blocking" or codegen.get("executed_in_merge") is not True:
            raise AssertionError(
                f"sifr_codegen is not blocking/executed in {profile_name}: {codegen}",
            )
        generated_build_suites = {
            "sifr_cli_generated_builds",
            "sifr_driver_generated_builds",
        }
        missing_generated = sorted(generated_build_suites.difference(profile_by_id))
        if missing_generated:
            raise AssertionError(
                f"generated-build crate suites missing from {profile_name}: {missing_generated}",
            )
        for suite_id in sorted(generated_build_suites):
            suite = profile_by_id[suite_id]
            if suite.get("status") != "blocking" or suite.get("executed_in_merge") is not True:
                raise AssertionError(
                    f"generated-build crate suite is not blocking/executed in {profile_name}: "
                    f"{suite}",
                )
        smoke_ids = {str(suite.get("id")) for suite in profile_smoke_suites}
        misplaced_generated = sorted(generated_build_suites.intersection(smoke_ids))
        if misplaced_generated:
            raise AssertionError(
                f"generated-build crate suites must not run in smoke for {profile_name}: "
                f"{misplaced_generated}",
            )

    duplicate_profile = {
        "name": "self-test",
        "crate_test_membership": {
            "suites": [
                {
                    "id": "duplicate",
                    "package": "sifr_ir",
                    "command": ["test", "-p", "sifr_ir"],
                    "modes": ["full"],
                    "status": "blocking",
                    "executed_in_merge": True,
                },
                {
                    "id": "duplicate",
                    "package": "sifr_ir",
                    "command": ["test", "-p", "sifr_ir"],
                    "modes": ["full"],
                    "status": "blocking",
                    "executed_in_merge": True,
                },
            ],
        },
    }
    try:
        validate_crate_test_membership(duplicate_profile)
    except ProfileError as exc:
        if "duplicate crate test suite duplicate" not in str(exc):
            raise
    else:
        raise AssertionError("duplicate crate membership suite was accepted")

    unknown_crate_profile = {
        "name": "self-test",
        "crate_test_membership": {
            "suites": [
                {
                    "id": "unknown",
                    "package": "sifr_does_not_exist",
                    "command": ["test", "-p", "sifr_does_not_exist"],
                    "modes": ["full"],
                    "status": "blocking",
                    "executed_in_merge": True,
                },
            ],
        },
    }
    try:
        validate_crate_test_membership(unknown_crate_profile)
    except ProfileError as exc:
        if "references unknown package sifr_does_not_exist" not in str(exc):
            raise
    else:
        raise AssertionError("unknown crate membership package was accepted")

    non_executed_full_profile = {
        "name": "self-test",
        "crate_test_membership": {
            "suites": [
                {
                    "id": "not_executed",
                    "package": "sifr_ir",
                    "command": ["test", "-p", "sifr_ir"],
                    "modes": ["full"],
                    "status": "blocking",
                    "executed_in_merge": False,
                },
            ],
        },
    }
    try:
        validate_crate_test_membership(non_executed_full_profile)
    except ProfileError as exc:
        if "must execute in merge unless it is a red-blocker" not in str(exc):
            raise
    else:
        raise AssertionError("non-executed full-mode blocking suite was accepted")

    unknown_suite_profile = {
        "name": "self-test",
        "selected_areas": [{"area": "core_language", "suites": ["not_a_suite"]}],
    }
    try:
        validate_selected_area_suites(unknown_suite_profile)
    except ProfileError as exc:
        if "selects unknown suite core_language:not_a_suite" not in str(exc):
            raise
    else:
        raise AssertionError("unknown selected area suite was accepted")


def _e2e_profile_self_test() -> None:
    profiles = load_all_profiles()
    create_pr_manifest = legacy_facade(profiles["create-pr"])["e2e"].get("fixture_manifest")
    if create_pr_manifest != "verification/areas/core_language/data/create_pr_e2e_manifest.json":
        raise AssertionError(f"create-pr e2e must remain representative, got: {create_pr_manifest}")

    for profile_name in ("merge", "nightly", "release"):
        fixture_manifest = legacy_facade(profiles[profile_name])["e2e"].get("fixture_manifest")
        if fixture_manifest:
            raise AssertionError(
                f"{profile_name} e2e must use the full pass corpus, got fixture manifest: "
                f"{fixture_manifest}",
            )

    merge_full_suites = crate_test_suites_for_mode(profiles["merge"], "full")
    by_id = {str(suite.get("id")): suite for suite in merge_full_suites}
    cli_suite = by_id.get("sifr_cli_full")
    if not isinstance(cli_suite, dict):
        raise AssertionError("merge full crate tests must include sifr_cli_full")
    command = cli_suite.get("command")
    if command != ["test", "-p", "sifr", "--", "--skip", "test_e2e_pass"]:
        raise AssertionError(
            "sifr_cli_full must skip only test_e2e_pass so the full fail corpus remains "
            f"merge-blocking, got: {command}",
        )


def _discovery_self_test() -> None:
    committed_areas = {area.name for area in discover_areas()}
    if "core_language" not in committed_areas:
        raise AssertionError(f"core_language area was not discovered: {sorted(committed_areas)}")
    if "diagnostics" not in committed_areas:
        raise AssertionError(f"diagnostics area was not discovered: {sorted(committed_areas)}")
    if "project_workspace" not in committed_areas:
        raise AssertionError(f"project_workspace area was not discovered: {sorted(committed_areas)}")
    if "regression" not in committed_areas:
        raise AssertionError(f"regression area was not discovered: {sorted(committed_areas)}")
    if "fuzz_property" not in committed_areas:
        raise AssertionError(f"fuzz_property area was not discovered: {sorted(committed_areas)}")

    with tempfile.TemporaryDirectory() as tmp:
        areas_dir = Path(tmp) / "areas"
        demo_dir = areas_dir / "core_language"
        demo_dir.mkdir(parents=True)
        (demo_dir / "manifest.json").write_text(
            """
{
  "schema_version": 1,
  "name": "core_language",
  "owner": "compiler/core",
  "description": "Temporary discovery fixture.",
  "parallel_safe": true,
  "resource_classes": ["default-local"],
  "timeout_seconds": 60,
  "suites": []
}
""".strip()
            + "\n",
            encoding="utf-8",
        )
        areas = discover_areas(areas_dir)
    if [area.name for area in areas] != ["core_language"]:
        raise AssertionError(f"unexpected discovery result: {areas}")


def _resource_class_self_test() -> None:
    profile = {
        "resource_policy": {"classes": ["default-local", "network", "container-runtime"]},
        "selected_areas": [
            {"area": "core_language", "resource_classes": ["default-local"]},
            {"area": "ecosystem_compatibility", "resource_classes": ["external-corpus"]},
        ],
    }
    expected = {"default-local", "network", "external-corpus", "container-runtime"}
    actual = selected_resource_classes(profile)
    if actual != expected:
        raise AssertionError(f"resource class selection mismatch: {actual}")


def _failure_reproduction_self_test() -> None:
    command = failure_reproduction_command("create-pr", "case-001")
    expected = "uv run --project verification python -m sifr_verify --profile create-pr --case case-001"
    if command != expected:
        raise AssertionError(f"unexpected reproduction command: {command}")
    result = build_result(
        profile="create-pr",
        status="fail",
        elapsed_ms=7,
        cases=[{"id": "case-001", "status": "fail", "elapsed_ms": 7}],
    )
    schema = load_schema("result.schema.json")
    validate_data(result, schema, source="self-test result")
    failures = result.get("failures")
    if failures != [{"case_id": "case-001", "reproduce": expected}]:
        raise AssertionError(f"unexpected failure reproduction data: {failures}")
