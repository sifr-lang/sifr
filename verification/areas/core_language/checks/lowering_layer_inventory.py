"""Validate Wave 5 lowering-layer snapshot inventory rows."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
INVENTORY = (
    REPO_ROOT / "verification" / "areas" / "core_language" / "data" / "lowering_layer_inventory.json"
)
ALLOWED_LAYERS = {
    "parsed_source",
    "hir_lowering",
    "name_resolution",
    "type_ownership",
    "cfg_flow",
}
ALLOWED_PROFILES = {"create-pr", "merge", "nightly", "release"}
REQUIRED_FIELDS = {
    "id",
    "compiler_layer",
    "owner",
    "contract_id",
    "source_fixture",
    "snapshot_id",
    "snapshot_kind",
    "normalizers",
    "profile_assignment",
    "status",
    "replacement",
}


def main() -> int:
    failures = validate_inventory()
    if failures:
        for failure in failures:
            print(f"lowering layer inventory error: {failure}", file=sys.stderr)
        return 1
    print("lowering layer inventory ok")
    return 0


def validate_inventory() -> list[str]:
    try:
        inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"failed to read {INVENTORY.relative_to(REPO_ROOT)}: {error}"]

    failures: list[str] = []
    if inventory.get("schema_version") != 1:
        failures.append("schema_version must be 1")
    layers = inventory.get("layers")
    if not isinstance(layers, list) or not layers:
        failures.append("layers must be a non-empty list")
        return failures

    seen_ids: set[str] = set()
    seen_contracts: set[str] = set()
    seen_snapshots: set[str] = set()
    for index, row in enumerate(layers):
        if not isinstance(row, dict):
            failures.append(f"layers[{index}] must be an object")
            continue
        row_id = str(row.get("id", f"layers[{index}]"))
        missing = sorted(REQUIRED_FIELDS.difference(row))
        if missing:
            failures.append(f"{row_id} missing required fields: {', '.join(missing)}")
        validate_unique_string(row, "id", seen_ids, failures, row_id)
        validate_unique_string(row, "contract_id", seen_contracts, failures, row_id)
        validate_unique_string(row, "snapshot_id", seen_snapshots, failures, row_id)
        validate_required_string(row, "owner", failures, row_id)
        validate_required_string(row, "snapshot_kind", failures, row_id)
        validate_enum(row, "compiler_layer", ALLOWED_LAYERS, failures, row_id)
        validate_enum(row, "status", {"active", "mapped", "deferred"}, failures, row_id)
        validate_string_list(row, "normalizers", failures, row_id)
        profiles = validate_string_list(row, "profile_assignment", failures, row_id)
        for profile in profiles:
            if profile not in ALLOWED_PROFILES:
                failures.append(f"{row_id} profile_assignment contains unknown profile {profile!r}")
        if row.get("status") == "active" and not {"create-pr", "merge"}.intersection(profiles):
            failures.append(f"{row_id} active snapshot must be assigned to create-pr or merge")
        replacement = row.get("replacement")
        if row.get("status") == "mapped" and not (
            isinstance(replacement, str) and replacement
        ):
            failures.append(f"{row_id} mapped snapshot must declare replacement")
        if row.get("status") == "active" and replacement is not None:
            failures.append(f"{row_id} active snapshot replacement must be null")
        failures.extend(validate_source_fixture(row, row_id))

    return failures


def validate_unique_string(
    row: dict[str, Any],
    field: str,
    seen: set[str],
    failures: list[str],
    row_id: str,
) -> None:
    value = validate_required_string(row, field, failures, row_id)
    if value is None:
        return
    if value in seen:
        failures.append(f"{row_id} duplicates {field} {value!r}")
    seen.add(value)


def validate_required_string(
    row: dict[str, Any],
    field: str,
    failures: list[str],
    row_id: str,
) -> str | None:
    value = row.get(field)
    if not isinstance(value, str) or not value:
        failures.append(f"{row_id} {field} must be a non-empty string")
        return None
    return value


def validate_enum(
    row: dict[str, Any],
    field: str,
    allowed: set[str],
    failures: list[str],
    row_id: str,
) -> None:
    value = validate_required_string(row, field, failures, row_id)
    if value is not None and value not in allowed:
        failures.append(f"{row_id} {field} must be one of {sorted(allowed)}, got {value!r}")


def validate_string_list(
    row: dict[str, Any],
    field: str,
    failures: list[str],
    row_id: str,
) -> list[str]:
    value = row.get(field)
    if not isinstance(value, list) or not value:
        failures.append(f"{row_id} {field} must be a non-empty list")
        return []
    strings = [item for item in value if isinstance(item, str) and item]
    if len(strings) != len(value):
        failures.append(f"{row_id} {field} must contain only non-empty strings")
    return strings


def validate_source_fixture(row: dict[str, Any], row_id: str) -> list[str]:
    source_fixture = row.get("source_fixture")
    if not isinstance(source_fixture, str) or "#" not in source_fixture:
        return [f"{row_id} source_fixture must be '<path>#<collection>/<id>'"]
    path_text, fragment = source_fixture.split("#", 1)
    if "/" not in fragment:
        return [f"{row_id} source_fixture fragment must be '<collection>/<id>'"]
    collection, fixture_id = fragment.split("/", 1)
    fixture_path = REPO_ROOT / path_text
    try:
        payload = json.loads(fixture_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"{row_id} failed to read source_fixture {source_fixture!r}: {error}"]
    rows = payload.get(collection)
    if not isinstance(rows, list):
        return [f"{row_id} source_fixture collection {collection!r} is not a list"]
    matched = [item for item in rows if isinstance(item, dict) and item.get("id") == fixture_id]
    if not matched:
        return [f"{row_id} source_fixture target {fragment!r} does not exist"]
    target = matched[0]
    if "expected_statement_tree" not in target:
        return [f"{row_id} source_fixture target {fragment!r} lacks expected_statement_tree"]
    expected_snapshot_id = f"{fixture_path.stem}.{collection}.{fixture_id}"
    if row.get("snapshot_id") != expected_snapshot_id:
        return [f"{row_id} snapshot_id does not match source_fixture fragment {fragment!r}"]
    return []


if __name__ == "__main__":
    raise SystemExit(main())
