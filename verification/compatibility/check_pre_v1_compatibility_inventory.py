#!/usr/bin/env python3
"""Validate the pre-v1 compatibility inventory and its ownership contract."""

from __future__ import annotations

import argparse
import copy
import json
import re
import sys
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[2]
INVENTORY_PATH = Path(__file__).with_name("pre_v1_compatibility_inventory.json")

REQUIRED_TOP_LEVEL_FIELDS = {
    "schema_version",
    "phase_id",
    "snapshot_base_sha",
    "classifications",
    "visibility_classes",
    "receiver_contract",
    "surfaces",
    "baseline_counts",
}
REQUIRED_SURFACE_FIELDS = {
    "id",
    "area",
    "old_surface",
    "canonical_contract",
    "behavior_difference",
    "consumers",
    "visibility",
    "classification",
    "disposition",
    "owner",
}
REQUIRED_COUNT_FIELDS = {"id", "count", "command", "owner"}
EXPECTED_CLASSIFICATIONS = {
    "canonical",
    "merge-then-remove",
    "remove",
    "distinct",
    "retained",
}
EXPECTED_VISIBILITY = {
    "public-export",
    "private-implementation-import",
    "compiler-internal-api",
    "sifr-owned-schema",
    "rejected-only-form",
    "external-or-current-contract",
}
EXPECTED_RECEIVERS = {
    "self": ("shared-borrow", "rejected"),
    "mut self": ("mutable-borrow", "allowed"),
    "own self": ("owned", "rejected"),
    "own mut self": ("owned-mutable", "allowed"),
}
REQUIRED_RETAINED_IDS = {
    "retained-dlpack-protocol",
    "retained-lsp-utf16-default",
    "retained-cargo-metadata",
    "retained-cargo-semver",
    "retained-ipc-negotiation",
    "retained-host-portability",
    "retained-cancellation-cleanup",
    "retained-configuration-defaults",
    "retained-translation-fallbacks",
    "retained-vendored-compatibility",
    "retained-external-file-formats",
    "retained-phase40-legacy-index",
    "retained-lint-deprecated-status",
}
FORBIDDEN_PLACEHOLDERS = {"unknown", "later", "todo", "tbd", "unowned", "follow-up"}


class InventoryError(ValueError):
    """The checked-in inventory violates its locked schema or ownership rules."""


def require_nonempty_string(row: dict[str, Any], field: str, row_id: str) -> str:
    value = row.get(field)
    if not isinstance(value, str) or not value.strip():
        raise InventoryError(f"{row_id}: {field} must be a non-empty string")
    return value


def validate_inventory(payload: Any, *, require_consumer_paths: bool = True) -> None:
    if not isinstance(payload, dict) or set(payload) != REQUIRED_TOP_LEVEL_FIELDS:
        raise InventoryError("inventory top-level fields drifted")
    if payload["schema_version"] != 1:
        raise InventoryError("schema_version must be 1")
    if payload["phase_id"] != "ad-hoc-pre-v1-compatibility-removal":
        raise InventoryError("phase_id drifted")
    sha = payload["snapshot_base_sha"]
    if not isinstance(sha, str) or len(sha) != 40 or any(ch not in "0123456789abcdef" for ch in sha):
        raise InventoryError("snapshot_base_sha must be a lowercase full Git SHA")
    if set(payload["classifications"]) != EXPECTED_CLASSIFICATIONS:
        raise InventoryError("classification registry drifted")
    if set(payload["visibility_classes"]) != EXPECTED_VISIBILITY:
        raise InventoryError("visibility registry drifted")
    validate_receivers(payload["receiver_contract"])
    validate_surfaces(payload["surfaces"], require_consumer_paths=require_consumer_paths)
    validate_counts(payload["baseline_counts"])


def validate_receivers(rows: Any) -> None:
    if not isinstance(rows, list) or len(rows) != len(EXPECTED_RECEIVERS):
        raise InventoryError("receiver_contract must contain exactly four receiver spellings")
    observed: dict[str, tuple[str, str]] = {}
    for row in rows:
        if not isinstance(row, dict) or set(row) != {"syntax", "convention", "mutation", "meaning"}:
            raise InventoryError("receiver row fields drifted")
        syntax = require_nonempty_string(row, "syntax", "receiver")
        require_nonempty_string(row, "meaning", syntax)
        if syntax in observed:
            raise InventoryError(f"duplicate receiver syntax: {syntax}")
        observed[syntax] = (
            require_nonempty_string(row, "convention", syntax),
            require_nonempty_string(row, "mutation", syntax),
        )
    if observed != EXPECTED_RECEIVERS:
        raise InventoryError("receiver syntax has a compatibility interpretation")


def validate_surfaces(rows: Any, *, require_consumer_paths: bool) -> None:
    if not isinstance(rows, list) or not rows:
        raise InventoryError("surfaces must be a non-empty list")
    seen_ids: set[str] = set()
    retained_ids: set[str] = set()
    observed_classifications: set[str] = set()
    for index, row in enumerate(rows):
        if not isinstance(row, dict) or set(row) != REQUIRED_SURFACE_FIELDS:
            raise InventoryError(f"surface[{index}] fields drifted")
        row_id = require_nonempty_string(row, "id", f"surface[{index}]")
        if row_id in seen_ids:
            raise InventoryError(f"duplicate surface id: {row_id}")
        seen_ids.add(row_id)
        for field in ("area", "old_surface", "canonical_contract", "behavior_difference", "owner"):
            value = require_nonempty_string(row, field, row_id)
            words = set(re.split(r"[^a-z0-9]+", value.lower()))
            if value.strip().lower() in FORBIDDEN_PLACEHOLDERS or (
                field == "owner" and words.intersection(FORBIDDEN_PLACEHOLDERS)
            ):
                raise InventoryError(f"{row_id}: {field} contains an unowned placeholder")
        consumers = row["consumers"]
        if not isinstance(consumers, list) or not consumers or not all(
            isinstance(value, str) and value for value in consumers
        ):
            raise InventoryError(f"{row_id}: consumers must be a non-empty string list")
        if require_consumer_paths:
            for consumer in consumers:
                path = REPO_ROOT / consumer
                if not path.exists():
                    raise InventoryError(f"{row_id}: consumer path does not exist: {consumer}")
        visibility = row["visibility"]
        classification = row["classification"]
        disposition = row["disposition"]
        owner = row["owner"]
        if visibility not in EXPECTED_VISIBILITY:
            raise InventoryError(f"{row_id}: unsupported visibility {visibility!r}")
        if classification not in EXPECTED_CLASSIFICATIONS:
            raise InventoryError(f"{row_id}: unsupported classification {classification!r}")
        observed_classifications.add(classification)
        if disposition not in {"remove", "retain"}:
            raise InventoryError(f"{row_id}: unsupported disposition {disposition!r}")
        if disposition == "remove":
            if not owner.startswith("pre_v1_compat_"):
                raise InventoryError(f"{row_id}: removal row has no implementation-item owner")
            if classification not in {"remove", "merge-then-remove"}:
                raise InventoryError(f"{row_id}: removal row has non-removal classification")
        else:
            if not owner.startswith("contract:"):
                raise InventoryError(f"{row_id}: retained row has no external/current contract owner")
            if classification not in {"canonical", "distinct", "retained"}:
                raise InventoryError(f"{row_id}: retained row has removal classification")
        if classification == "retained":
            retained_ids.add(row_id)
            if visibility != "external-or-current-contract":
                raise InventoryError(f"{row_id}: retained contract has incorrect visibility")
    if observed_classifications != EXPECTED_CLASSIFICATIONS:
        raise InventoryError("not every stdlib classification is represented")
    missing_retained = REQUIRED_RETAINED_IDS - retained_ids
    if missing_retained:
        raise InventoryError(f"required retained contracts missing: {sorted(missing_retained)}")


def validate_counts(rows: Any) -> None:
    if not isinstance(rows, list) or not rows:
        raise InventoryError("baseline_counts must be a non-empty list")
    seen_ids: set[str] = set()
    for index, row in enumerate(rows):
        if not isinstance(row, dict) or set(row) != REQUIRED_COUNT_FIELDS:
            raise InventoryError(f"baseline_count[{index}] fields drifted")
        row_id = require_nonempty_string(row, "id", f"baseline_count[{index}]")
        if row_id in seen_ids:
            raise InventoryError(f"duplicate baseline count id: {row_id}")
        seen_ids.add(row_id)
        if type(row["count"]) is not int or row["count"] < 0:
            raise InventoryError(f"{row_id}: count must be a non-negative integer")
        require_nonempty_string(row, "command", row_id)
        owner = require_nonempty_string(row, "owner", row_id)
        if not owner.startswith("pre_v1_compat_"):
            raise InventoryError(f"{row_id}: count has no implementation-item owner")


def run_self_tests(payload: dict[str, Any]) -> None:
    mutations: dict[str, Callable[[dict[str, Any]], None]] = {
        "missing-top-level-field": lambda value: value.pop("phase_id"),
        "invalid-classification": lambda value: value["surfaces"][0].update(
            {"classification": "unknown"}
        ),
        "unowned-removal": lambda value: value["surfaces"][0].update(
            {"owner": "contract:nobody"}
        ),
        "placeholder-owner": lambda value: value["surfaces"][0].update(
            {"owner": "pre_v1_compat_later"}
        ),
        "duplicate-surface": lambda value: value["surfaces"].append(
            copy.deepcopy(value["surfaces"][0])
        ),
        "private-public-drift": lambda value: value["surfaces"][-1].update(
            {"visibility": "public-export"}
        ),
        "missing-retained-contract": lambda value: value.update(
            {
                "surfaces": [
                    row
                    for row in value["surfaces"]
                    if row["id"] != "retained-dlpack-protocol"
                ]
            }
        ),
        "receiver-reinterpretation": lambda value: value["receiver_contract"][2].update(
            {"convention": "shared-borrow"}
        ),
        "invalid-count": lambda value: value["baseline_counts"][0].update({"count": -1}),
    }
    for case_id, mutate in mutations.items():
        changed = copy.deepcopy(payload)
        mutate(changed)
        try:
            validate_inventory(changed, require_consumer_paths=False)
        except InventoryError:
            continue
        raise InventoryError(f"self-test mutation unexpectedly passed: {case_id}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        payload = json.loads(INVENTORY_PATH.read_text(encoding="utf-8"))
        validate_inventory(payload)
        if args.self_test:
            run_self_tests(payload)
    except (OSError, json.JSONDecodeError, InventoryError) as error:
        print(f"pre-v1 compatibility inventory: {error}", file=sys.stderr)
        return 1
    suffix = " and self-test" if args.self_test else ""
    print(f"pre-v1 compatibility inventory{suffix} ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
