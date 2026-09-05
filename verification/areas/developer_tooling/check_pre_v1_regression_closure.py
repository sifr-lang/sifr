#!/usr/bin/env python3
"""Lock the canonical inputs that closed the pre-v1 regression inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
import tempfile
from collections import Counter
from collections.abc import Callable
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
CONTRACT_PATH = AREA_ROOT / "pre_v1_regression_closure.json"
CREATE_PR_FIXTURES = (
    REPO_ROOT / "verification" / "areas" / "core_language" / "data" / "create_pr_e2e_manifest.json"
)
FUZZ_MANIFEST = REPO_ROOT / "verification" / "areas" / "fuzz_property" / "fuzz_smoke_manifest.json"
PERFORMANCE_MANIFEST = REPO_ROOT / "verification" / "areas" / "performance" / "data" / "benchmark_manifest.json"
PERFORMANCE_TREND = REPO_ROOT / "verification" / "areas" / "performance" / "data" / "trend" / "current.json"
ECOSYSTEM_MANIFEST = (
    REPO_ROOT / "verification" / "areas" / "ecosystem_compatibility" / "data" / "curated_manifest.json"
)

sys.path.insert(0, str(REPO_ROOT / "verification" / "runner"))
from sifr_verify.hardening.oss_and_determinism import (
    project_source_checksum,
)

BASELINE_SHA = "b52fbc7e46257a7fd14d8f551fc9f2c28fb4ac47"
EXPECTED_AREA_COUNTS = {
    "ecosystem_compatibility": 5,
    "fuzz_property": 1,
    "generated_code_quality": 2,
    "performance": 6,
    "rust_interop": 1,
}
EXPECTED_VARIANT_IDS = {
    "matrix/area-check",
    "representative/corpus",
    "representative/panic-scan",
    "representative/benchmark-manifest",
    "representative/benchmark-runner-self-test",
    "representative/trend-policy",
    "representative/trend-policy-self-test",
    "representative/benchmark-subset",
    "representative/budget-subset",
    "fuzz-smoke/metadata",
    "OSS-CURATED-0001/source-checksum",
    "OSS-CURATED-0002/source-checksum",
    "OSS-CURATED-0003/source-checksum",
    "OSS-CURATED-0004/source-checksum",
    "OSS-CURATED-0005/source-checksum",
}
REQUIRED_CREATE_PR_FIXTURES = {
    "nested_optional_safe_operations",
    "nominal_identity_alias_paths",
    "try_nested_finally_error_propagation",
}
PLACEHOLDER_OWNERS = {"later", "nobody", "placeholder", "tbd", "todo", "unknown", "unowned"}


class GuardError(RuntimeError):
    """A canonical regression input is missing or stale."""


def load_json(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise GuardError(f"cannot read {path}: {error}") from error
    if not isinstance(payload, dict):
        raise GuardError(f"{path} must contain a JSON object")
    return payload


def require_string(payload: dict[str, Any], key: str, context: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise GuardError(f"{context}.{key} must be a non-empty string")
    return value


def validate_inventory(contract: dict[str, Any]) -> None:
    if contract.get("schema_version") != 2:
        raise GuardError("regression closure schema_version must be 2")
    baseline = contract.get("baseline")
    if not isinstance(baseline, dict):
        raise GuardError("regression closure baseline must be an object")
    if baseline.get("base_sha") != BASELINE_SHA:
        raise GuardError("regression closure baseline SHA drifted")
    if baseline.get("failed_variant_count") != 15:
        raise GuardError("regression closure must record 15 failed variants")
    variants = baseline.get("failed_variants")
    if not isinstance(variants, list) or len(variants) != 15:
        raise GuardError("regression closure failed_variants must contain exactly 15 rows")

    ids: list[str] = []
    areas: list[str] = []
    for index, raw in enumerate(variants, start=1):
        if not isinstance(raw, dict):
            raise GuardError(f"regression closure row {index} must be an object")
        variant_id = require_string(raw, "id", f"failed_variants[{index}]")
        area = require_string(raw, "area", f"failed_variants[{index}]")
        owner = require_string(raw, "owner", f"failed_variants[{index}]")
        require_string(raw, "mechanism", f"failed_variants[{index}]")
        if any(word in owner.lower().split("-") for word in PLACEHOLDER_OWNERS):
            raise GuardError(f"regression closure row {variant_id} has a placeholder owner")
        ids.append(variant_id)
        areas.append(area)

    if len(ids) != len(set(ids)):
        raise GuardError("regression closure failed-variant IDs must be unique")
    if set(ids) != EXPECTED_VARIANT_IDS:
        raise GuardError("regression closure failed-variant IDs drifted")
    if dict(Counter(areas)) != EXPECTED_AREA_COUNTS:
        raise GuardError("regression closure area counts do not reconcile to 15")


def validate_rust_interop_paths(repo_root: Path) -> None:
    fixture_root = repo_root / "verification" / "areas" / "rust_interop" / "fixtures" / "shared_bridge_crate"
    name = "package_generated_type_import_rejected.sifr"
    canonical = fixture_root / "negative" / name
    duplicate = fixture_root / "negative" / "src" / name
    if not canonical.is_file():
        raise GuardError(f"canonical Rust-interop fixture is missing: {canonical}")
    if duplicate.exists():
        raise GuardError(f"duplicate Rust-interop fixture path restored: {duplicate}")

    fixture_manifest = load_json(fixture_root / "fixture.json")
    manifest_text = json.dumps(fixture_manifest, sort_keys=True)
    expected_relative = f"negative/{name}"
    old_relative = f"negative/src/{name}"
    if expected_relative not in manifest_text or old_relative in manifest_text:
        raise GuardError("Rust-interop fixture manifest does not use the canonical flat negative path")


def validate_recorded_hash(input_path: Path, record_path: Path) -> None:
    expected = hashlib.sha256(input_path.read_bytes()).hexdigest()
    actual = require_string(load_json(record_path), "manifest_sha256", str(record_path))
    if actual != expected:
        raise GuardError(f"{record_path} manifest_sha256 is stale")


def validate_ecosystem_checksums(payload: dict[str, Any], repo_root: Path) -> None:
    entries = payload.get("entries")
    if not isinstance(entries, list) or len(entries) != 5:
        raise GuardError("curated ecosystem manifest must contain five entries")
    for index, raw in enumerate(entries, start=1):
        if not isinstance(raw, dict):
            raise GuardError(f"curated ecosystem entry {index} must be an object")
        entry_id = require_string(raw, "id", f"curated entry {index}")
        project_root = require_string(raw, "project_root", entry_id)
        expected = require_string(raw, "source_checksum_sha256", entry_id)
        actual = project_source_checksum(repo_root, project_root)
        if not actual or actual != expected:
            raise GuardError(f"{entry_id} source_checksum_sha256 is stale")


def validate_fuzz_paths(payload: dict[str, Any], repo_root: Path) -> None:
    targets = payload.get("targets")
    if not isinstance(targets, list):
        raise GuardError("fuzz target manifest must contain targets")
    target = next(
        (
            raw
            for raw in targets
            if isinstance(raw, dict) and raw.get("id") == "package_project_manifest_entrypoint"
        ),
        None,
    )
    if not isinstance(target, dict):
        raise GuardError("package-project fuzz target is missing")
    canonical = (
        "verification/areas/project_workspace/fixtures/project/"
        "workspace_missing_import_canonical/src/main.sifr"
    )
    old = canonical.replace("/src/main.sifr", "/main.sifr")
    seeds = target.get("seed_files")
    reproduction = target.get("reproduction_command")
    if seeds != [canonical]:
        raise GuardError("package-project fuzz seed does not use the canonical src/main.sifr path")
    if not isinstance(reproduction, list) or canonical not in reproduction:
        raise GuardError("package-project fuzz reproduction command does not use the canonical seed path")
    if old in json.dumps(payload, sort_keys=True):
        raise GuardError("package-project fuzz manifest restored the pre-src seed path")
    if not (repo_root / canonical).is_file():
        raise GuardError("canonical package-project fuzz seed is missing")


def validate_create_pr_locks(contract: dict[str, Any], payload: dict[str, Any], repo_root: Path) -> None:
    locks = contract.get("create_pr_fixture_locks")
    if not isinstance(locks, list) or set(locks) != REQUIRED_CREATE_PR_FIXTURES:
        raise GuardError("regression closure create-PR fixture locks drifted")
    fixture_names = payload.get("fixture_names")
    if not isinstance(fixture_names, list) or len(fixture_names) != len(set(fixture_names)):
        raise GuardError("create-PR E2E fixture names must be a unique list")
    missing = sorted(REQUIRED_CREATE_PR_FIXTURES.difference(fixture_names))
    if missing:
        raise GuardError(f"create-PR E2E manifest is missing regression fixtures: {', '.join(missing)}")
    for fixture in REQUIRED_CREATE_PR_FIXTURES:
        source = repo_root / "crates" / "sifr" / "tests" / "e2e" / "pass" / f"{fixture}.sifr"
        if not source.is_file():
            raise GuardError(f"locked create-PR fixture is missing: {source}")

    decision = contract.get("return_capture_coverage_decision")
    if decision != "not-added-existing-return-capture-coverage":
        raise GuardError("Return-capture twin decision drifted")
    return_capture_test = (
        repo_root
        / "crates"
        / "sifr_codegen"
        / "src"
        / "lib_codegen_tests"
        / "async_runtime_codegen_tests.rs"
    ).read_text(encoding="utf-8")
    if "fn test_try_finally_runs_cleanup_before_timeout_propagates()" not in return_capture_test:
        raise GuardError("existing Return-capture regression coverage is missing")


def validate_all(repo_root: Path = REPO_ROOT) -> None:
    contract = load_json(CONTRACT_PATH)
    validate_inventory(contract)
    validate_rust_interop_paths(repo_root)
    validate_recorded_hash(PERFORMANCE_MANIFEST, PERFORMANCE_TREND)
    validate_ecosystem_checksums(load_json(ECOSYSTEM_MANIFEST), repo_root)
    validate_fuzz_paths(load_json(FUZZ_MANIFEST), repo_root)
    validate_create_pr_locks(contract, load_json(CREATE_PR_FIXTURES), repo_root)


def expect_rejection(label: str, operation: Callable[[], None]) -> None:
    try:
        operation()
    except GuardError:
        return
    raise GuardError(f"self-test mutation unexpectedly passed: {label}")


def run_self_test() -> None:
    contract = load_json(CONTRACT_PATH)
    missing_variant = copy.deepcopy(contract)
    missing_variant["baseline"]["failed_variants"].pop()
    expect_rejection("missing baseline variant", lambda: validate_inventory(missing_variant))

    with tempfile.TemporaryDirectory(prefix="sifr-pre-v1-guard-") as raw_tmp:
        temp_root = Path(raw_tmp)
        relative = Path("verification/areas/rust_interop/fixtures/shared_bridge_crate/negative")
        canonical = temp_root / relative / "package_generated_type_import_rejected.sifr"
        duplicate = temp_root / relative / "src" / canonical.name
        canonical.parent.mkdir(parents=True)
        canonical.write_text("pass\n", encoding="utf-8")
        duplicate.parent.mkdir(parents=True)
        duplicate.write_text("pass\n", encoding="utf-8")
        fixture_manifest = canonical.parents[1] / "fixture.json"
        fixture_manifest.write_text(
            json.dumps({"path": f"negative/{canonical.name}"}), encoding="utf-8"
        )
        expect_rejection("restored duplicate fixture path", lambda: validate_rust_interop_paths(temp_root))

        input_path = temp_root / "manifest.json"
        record_path = temp_root / "record.json"
        input_path.write_text("{}\n", encoding="utf-8")
        record_path.write_text(json.dumps({"manifest_sha256": "0" * 64}), encoding="utf-8")
        expect_rejection("stale recorded hash", lambda: validate_recorded_hash(input_path, record_path))

    stale_ecosystem = copy.deepcopy(load_json(ECOSYSTEM_MANIFEST))
    stale_ecosystem["entries"][0]["source_checksum_sha256"] = "0" * 64
    expect_rejection(
        "stale ecosystem checksum",
        lambda: validate_ecosystem_checksums(stale_ecosystem, REPO_ROOT),
    )

    stale_fuzz = copy.deepcopy(load_json(FUZZ_MANIFEST))
    target = next(
        raw for raw in stale_fuzz["targets"] if raw["id"] == "package_project_manifest_entrypoint"
    )
    target["seed_files"][0] = target["seed_files"][0].replace("/src/main.sifr", "/main.sifr")
    expect_rejection("stale fuzz seed path", lambda: validate_fuzz_paths(stale_fuzz, REPO_ROOT))

    missing_fixture = copy.deepcopy(load_json(CREATE_PR_FIXTURES))
    missing_fixture["fixture_names"].remove("nominal_identity_alias_paths")
    expect_rejection(
        "missing create-PR regression fixture",
        lambda: validate_create_pr_locks(contract, missing_fixture, REPO_ROOT),
    )
    print("pre-v1 regression closure guard self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            run_self_test()
        else:
            validate_all()
            print("pre-v1 regression closure guard: PASS (baseline_variants=15)")
    except (GuardError, OSError) as error:
        print(f"pre-v1 regression closure guard: FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
