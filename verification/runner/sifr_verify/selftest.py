"""Self-tests for the verification runner foundation."""

from __future__ import annotations

import tempfile
from pathlib import Path

from .areas import discover_areas
from .profiles import failure_reproduction_command, load_all_profiles, selected_resource_classes
from .results import build_result
from .schemas import load_schema, validate_all_committed_schemas, validate_data, validate_schema_contract


def run_all() -> list[str]:
    checks = [
        ("schema self-tests", _schema_self_test),
        ("profile schema self-test", _profile_schema_self_test),
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
    }
    missing = required - set(committed)
    if missing:
        raise AssertionError(f"missing schema self-test coverage: {sorted(missing)}")
    try:
        validate_schema_contract({"type": "object", "oneOf": []}, Path("bad.schema.json"))
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
    expected = {"create-pr", "merge", "nightly", "release"}
    if set(profiles) != expected:
        raise AssertionError(f"unexpected profiles: {sorted(profiles)}")


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
        "resource_policy": {"classes": ["default-local", "network"]},
        "selected_areas": [
            {"area": "core_language", "resource_classes": ["default-local"]},
            {"area": "ecosystem_compatibility", "resource_classes": ["external-corpus"]},
        ],
    }
    expected = {"default-local", "network", "external-corpus"}
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
