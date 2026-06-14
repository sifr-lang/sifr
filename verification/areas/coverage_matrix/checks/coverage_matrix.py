"""Validate the advisory coverage matrix introduced for gate closure."""

from __future__ import annotations

import json
import os
import sys
from datetime import date
from pathlib import Path
from typing import Any

from sifr_verify.schemas import load_schema, validate_data

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "coverage_matrix"
OWNERS_PATH = REPO_ROOT / "verification" / "owners.json"
GUARANTEES_PATH = AREA_ROOT / "shipped_guarantees.json"
SURFACES_PATH = AREA_ROOT / "compiler_surface_matrix.json"
CLI_CONTRACTS_PATH = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "data" / "cli_exit_code_contracts.json"
WORKSPACE_CONTRACTS_PATH = REPO_ROOT / "verification" / "areas" / "project_workspace" / "data" / "workspace_contracts.json"
PROFILES_DIR = REPO_ROOT / "verification" / "profiles"
AREAS_DIR = REPO_ROOT / "verification" / "areas"

VALID_SUPPORT_STATUSES = {"stable", "experimental", "internal", "unsupported"}
VALID_MATRIX_STATUSES = {
    "blocking",
    "broad-only",
    "expected-missing",
    "tests:none",
    "not-applicable",
    "red-blocker",
    "quarantined",
}
TEMPORARY_STATUSES = {"expected-missing", "tests:none", "red-blocker", "quarantined"}
NON_CLOSEOUT_STATUSES = {"expected-missing", "tests:none", "red-blocker"}
ALLOWED_WAVES = {str(value) for value in range(1, 10)}
ALLOWED_SUBWAVES_BY_WAVE = {
    "2": {"final"},
    "5": {"1", "2", "3", "4", "5", "6", "7", "8"},
    "6": {"0", "1"},
    "9": {"1", "2", "3", "4", "5", "6"},
}
REQUIRED_PROFILES = {"create-pr", "merge", "nightly", "release"}


def main() -> int:
    errors: list[str] = []
    strict = os.environ.get("SIFR_COVERAGE_MATRIX_STRICT") == "1"

    owners = load_owner_ids(errors)
    guarantees = load_json_object(GUARANTEES_PATH, errors).get("guarantees", [])
    surfaces = load_json_object(SURFACES_PATH, errors).get("rows", [])
    validate_guarantees(guarantees, surfaces, owners, errors)
    validate_surfaces(surfaces, guarantees, owners, strict, errors)
    validate_owner_registry_covers_area_manifests(owners, errors)
    validate_contract_inventory(CLI_CONTRACTS_PATH, "contracts", "profile_surface", errors)
    validate_contract_inventory(WORKSPACE_CONTRACTS_PATH, "contracts", "profile_surface", errors)
    validate_profile_policy(errors)

    if errors:
        for error in errors:
            print(f"coverage-matrix error: {error}", file=sys.stderr)
        return 1

    temporary_count = sum(1 for row in surfaces if row.get("status") in TEMPORARY_STATUSES)
    print(
        "coverage matrix ok: "
        f"guarantees={len(guarantees)} surfaces={len(surfaces)} "
        f"temporary_rows={temporary_count} strict={'yes' if strict else 'no'}"
    )
    return 0


def load_json_object(path: Path, errors: list[str]) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        errors.append(f"missing JSON file: {repo_path(path)}")
        return {}
    except json.JSONDecodeError as exc:
        errors.append(f"invalid JSON {repo_path(path)}: {exc}")
        return {}
    if not isinstance(payload, dict):
        errors.append(f"{repo_path(path)} must contain a JSON object")
        return {}
    return payload


def load_owner_ids(errors: list[str]) -> set[str]:
    payload = load_json_object(OWNERS_PATH, errors)
    if payload:
        try:
            validate_data(payload, load_schema("owners.schema.json"), source=repo_path(OWNERS_PATH))
        except Exception as exc:
            errors.append(str(exc))
    owners = payload.get("owners", [])
    if not isinstance(owners, list):
        errors.append("verification/owners.json owners must be an array")
        return set()
    ids: set[str] = set()
    for index, owner in enumerate(owners):
        if not isinstance(owner, dict):
            errors.append(f"verification/owners.json owners[{index}] must be an object")
            continue
        owner_id = owner.get("id")
        if not isinstance(owner_id, str) or not owner_id:
            errors.append(f"verification/owners.json owners[{index}] has invalid id")
            continue
        if owner_id == "unassigned":
            errors.append("owner id 'unassigned' is forbidden")
        if owner_id in ids:
            errors.append(f"duplicate owner id: {owner_id}")
        ids.add(owner_id)
    return ids


def validate_guarantees(
    guarantees: Any,
    surfaces: Any,
    owners: set[str],
    errors: list[str],
) -> None:
    if not isinstance(guarantees, list) or not guarantees:
        errors.append("shipped_guarantees.json must define a non-empty guarantees array")
        return
    surface_ids = collect_ids(surfaces, "surface_id")
    guarantee_ids: set[str] = set()
    rows_by_guarantee: dict[str, int] = {}
    if isinstance(surfaces, list):
        for row in surfaces:
            if isinstance(row, dict) and isinstance(row.get("guarantee_id"), str):
                rows_by_guarantee[row["guarantee_id"]] = rows_by_guarantee.get(row["guarantee_id"], 0) + 1

    for index, guarantee in enumerate(guarantees):
        if not isinstance(guarantee, dict):
            errors.append(f"guarantees[{index}] must be an object")
            continue
        guarantee_id = require_string(guarantee, "guarantee_id", f"guarantees[{index}]", errors)
        if guarantee_id:
            if guarantee_id in guarantee_ids:
                errors.append(f"duplicate guarantee_id: {guarantee_id}")
            guarantee_ids.add(guarantee_id)
        status = require_string(guarantee, "support_status", f"guarantees[{index}]", errors)
        if status and status not in VALID_SUPPORT_STATUSES:
            errors.append(f"{guarantee_id}: invalid support_status {status}")
        validate_owner(guarantee.get("owner"), owners, f"{guarantee_id}.owner", errors)
        public_doc = require_string(guarantee, "public_doc_path", f"{guarantee_id}", errors)
        if public_doc and public_doc != "internal" and not (REPO_ROOT / public_doc).exists():
            errors.append(f"{guarantee_id}: public_doc_path does not exist: {public_doc}")
        for key in ("merge_surface", "nightly_release_surface", "regression_surface"):
            surface_id = require_string(guarantee, key, f"{guarantee_id}", errors)
            if surface_id and key != "regression_surface" and surface_id not in surface_ids:
                errors.append(f"{guarantee_id}: {key} references unknown surface {surface_id}")
        if status == "stable" and rows_by_guarantee.get(str(guarantee_id), 0) == 0:
            errors.append(f"{guarantee_id}: stable guarantee has no matrix row")


def validate_surfaces(
    surfaces: Any,
    guarantees: Any,
    owners: set[str],
    strict: bool,
    errors: list[str],
) -> None:
    if not isinstance(surfaces, list) or not surfaces:
        errors.append("compiler_surface_matrix.json must define a non-empty rows array")
        return
    guarantee_status = {
        guarantee.get("guarantee_id"): guarantee.get("support_status")
        for guarantee in guarantees
        if isinstance(guarantee, dict)
    }
    surface_ids: set[str] = set()
    today = date.today()
    for index, row in enumerate(surfaces):
        if not isinstance(row, dict):
            errors.append(f"rows[{index}] must be an object")
            continue
        location = f"rows[{index}]"
        surface_id = require_string(row, "surface_id", location, errors)
        if surface_id:
            if surface_id in surface_ids:
                errors.append(f"duplicate surface_id: {surface_id}")
            surface_ids.add(surface_id)
            location = surface_id
        guarantee_id = require_string(row, "guarantee_id", location, errors)
        if guarantee_id and guarantee_id not in guarantee_status:
            errors.append(f"{location}: unknown guarantee_id {guarantee_id}")
        validate_owner(row.get("owner"), owners, f"{location}.owner", errors)
        status = require_string(row, "status", location, errors)
        if status and status not in VALID_MATRIX_STATUSES:
            errors.append(f"{location}: invalid status {status}")
        if strict and status in NON_CLOSEOUT_STATUSES:
            errors.append(f"{location}: status {status} is illegal in strict closeout mode")
        if guarantee_status.get(guarantee_id) == "stable" and status == "broad-only":
            errors.append(f"{location}: stable guarantees may not be broad-only")
        for key in ("merge_suite", "nightly_release_suite", "regression_suite", "reproduction_command"):
            require_string(row, key, location, errors)
        if status in TEMPORARY_STATUSES:
            validate_temporary_row(row, location, today, errors)
        if status == "red-blocker":
            for key in ("command", "triage_file"):
                require_string(row, key, location, errors)
            if not isinstance(row.get("current_failure_count"), int):
                errors.append(f"{location}: red-blocker requires integer current_failure_count")
        if status == "not-applicable" and guarantee_status.get(guarantee_id) == "stable":
            errors.append(f"{location}: not-applicable is illegal for stable shipped guarantees")


def validate_temporary_row(row: dict[str, Any], location: str, today: date, errors: list[str]) -> None:
    for key in ("issue", "expiry"):
        require_string(row, key, location, errors)
    if row.get("status") in {"expected-missing", "red-blocker"}:
        wave = require_string(row, "closes_in_wave", location, errors)
        if wave and wave not in ALLOWED_WAVES:
            errors.append(f"{location}: closes_in_wave must be one of 1-9")
        subwave = row.get("closes_in_subwave")
        if subwave is not None:
            allowed = ALLOWED_SUBWAVES_BY_WAVE.get(str(wave), set())
            if subwave not in allowed:
                errors.append(f"{location}: unknown closes_in_subwave {wave}.{subwave}")
    expiry = row.get("expiry")
    if isinstance(expiry, str):
        try:
            expiry_date = date.fromisoformat(expiry)
        except ValueError:
            errors.append(f"{location}: expiry must be YYYY-MM-DD")
        else:
            if expiry_date < today:
                errors.append(f"{location}: expiry has passed: {expiry}")


def validate_owner(value: Any, owners: set[str], location: str, errors: list[str]) -> None:
    if not isinstance(value, str) or not value:
        errors.append(f"{location}: missing owner")
        return
    if value == "unassigned":
        errors.append(f"{location}: owner must not be unassigned")
    if value not in owners:
        errors.append(f"{location}: unknown owner {value}")


def validate_owner_registry_covers_area_manifests(owners: set[str], errors: list[str]) -> None:
    for manifest in sorted(AREAS_DIR.glob("*/manifest.json")):
        payload = load_json_object(manifest, errors)
        validate_owner(payload.get("owner"), owners, f"{repo_path(manifest)}.owner", errors)


def validate_contract_inventory(path: Path, array_key: str, surface_key: str, errors: list[str]) -> None:
    payload = load_json_object(path, errors)
    contracts = payload.get(array_key)
    if not isinstance(contracts, list) or not contracts:
        errors.append(f"{repo_path(path)} must define a non-empty {array_key} array")
        return
    seen: set[str] = set()
    for index, contract in enumerate(contracts):
        if not isinstance(contract, dict):
            errors.append(f"{repo_path(path)} {array_key}[{index}] must be an object")
            continue
        contract_id = require_string(contract, "id", f"{repo_path(path)}[{index}]", errors)
        if contract_id:
            if contract_id in seen:
                errors.append(f"{repo_path(path)} duplicate contract id: {contract_id}")
            seen.add(contract_id)
        require_string(contract, surface_key, str(contract_id), errors)


def validate_profile_policy(errors: list[str]) -> None:
    area_suites = load_area_suites(errors)
    profiles = {}
    for path in sorted(PROFILES_DIR.glob("*.json")):
        payload = load_json_object(path, errors)
        name = payload.get("name")
        if isinstance(name, str):
            profiles[name] = payload
    missing = REQUIRED_PROFILES.difference(profiles)
    if missing:
        errors.append(f"missing required profiles: {', '.join(sorted(missing))}")
    for profile_name, profile in sorted(profiles.items()):
        validate_selected_area_suites(
            profile_name,
            profile.get("selected_areas", []),
            area_suites,
            errors,
        )
    for profile_name in ("create-pr", "merge"):
        profile = profiles.get(profile_name)
        if profile is None:
            continue
        if profile.get("schema_version") != 2:
            errors.append(f"{profile_name}: create-pr/merge profiles must use schema_version 2")
        network_policy = profile.get("network_policy")
        if not isinstance(network_policy, dict) or network_policy.get("mode") != "offline":
            errors.append(f"{profile_name}: network_policy.mode must be offline")
        if isinstance(network_policy, dict) and network_policy.get("live_network_allowed") is not False:
            errors.append(f"{profile_name}: live network is forbidden")
        cargo_policy = profile.get("cargo_policy")
        if not isinstance(cargo_policy, dict):
            errors.append(f"{profile_name}: missing cargo_policy")
        elif cargo_policy.get("locked") is not True or cargo_policy.get("offline") is not True:
            errors.append(f"{profile_name}: cargo_policy must require locked and offline execution")
        selected = profile.get("selected_areas", [])
        if not any(
            isinstance(item, dict)
            and item.get("area") == "coverage_matrix"
            and "advisory" in item.get("suites", [])
            for item in selected
        ):
            errors.append(f"{profile_name}: coverage_matrix advisory suite must be selected")
        profile_plan = profile.get("profile_plan")
        if not isinstance(profile_plan, dict) or not profile_plan.get("emit_command"):
            errors.append(f"{profile_name}: missing profile plan emission command")


def load_area_suites(errors: list[str]) -> dict[str, set[str]]:
    area_suites: dict[str, set[str]] = {}
    for manifest in sorted(AREAS_DIR.glob("*/manifest.json")):
        payload = load_json_object(manifest, errors)
        name = payload.get("name")
        suites = payload.get("suites", [])
        if not isinstance(name, str):
            continue
        if not isinstance(suites, list):
            errors.append(f"{repo_path(manifest)}: suites must be an array")
            continue
        area_suites[name] = {
            suite["name"]
            for suite in suites
            if isinstance(suite, dict) and isinstance(suite.get("name"), str)
        }
    return area_suites


def validate_selected_area_suites(
    profile_name: str,
    selected: Any,
    area_suites: dict[str, set[str]],
    errors: list[str],
) -> None:
    if not isinstance(selected, list):
        errors.append(f"{profile_name}: selected_areas must be an array")
        return
    for index, selection in enumerate(selected):
        if not isinstance(selection, dict):
            errors.append(f"{profile_name}: selected_areas[{index}] must be an object")
            continue
        area = selection.get("area")
        if not isinstance(area, str) or area not in area_suites:
            errors.append(f"{profile_name}: selected_areas[{index}] unknown area {area}")
            continue
        suites = selection.get("suites", [])
        if not isinstance(suites, list):
            errors.append(f"{profile_name}: selected_areas[{index}].suites must be an array")
            continue
        for suite in suites:
            if suite not in area_suites[area]:
                errors.append(f"{profile_name}: selected_areas[{index}] unknown suite {area}:{suite}")


def collect_ids(rows: Any, key: str) -> set[str]:
    if not isinstance(rows, list):
        return set()
    return {row[key] for row in rows if isinstance(row, dict) and isinstance(row.get(key), str)}


def require_string(payload: dict[str, Any], key: str, location: str, errors: list[str]) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        errors.append(f"{location}: missing or invalid {key}")
        return ""
    return value


def repo_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


if __name__ == "__main__":
    raise SystemExit(main())
