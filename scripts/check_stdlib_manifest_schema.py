#!/usr/bin/env python3
"""Validate the retained compiler-native stdlib glue manifest schema."""

from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "internal_docs" / "stdlib_retained_compiler_intrinsics.toml"

EXPECTED_SCHEMA_VERSION = 2
ALLOWED_TOP_LEVEL_FIELDS = {"schema_version", "surface"}
ALLOWED_SURFACE_FIELDS = {
    "id",
    "state",
    "owner",
    "issue",
    "removal_criteria",
    "evidence_links",
    "declaration_files",
    "certification_rows",
    "reason",
    "registry_files",
    "preamble_files",
    "exact_intrinsics",
}
VALID_STATES = {"retained", "pilot", "closing", "retained-by-design"}


def main() -> int:
    manifest = tomllib.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    failures = _validate(manifest)
    if failures:
        print("stdlib retained manifest schema: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print(
        "stdlib retained manifest schema: PASS "
        f"(surfaces={len(manifest.get('surface', []))}, schema_version={EXPECTED_SCHEMA_VERSION})"
    )
    return 0


def _validate(manifest: dict[str, Any]) -> list[str]:
    failures: list[str] = []

    unknown_top_level = sorted(set(manifest) - ALLOWED_TOP_LEVEL_FIELDS)
    if unknown_top_level:
        failures.append(f"unknown top-level fields: {', '.join(unknown_top_level)}")

    if manifest.get("schema_version") != EXPECTED_SCHEMA_VERSION:
        failures.append(f"schema_version must be {EXPECTED_SCHEMA_VERSION}")

    surfaces = manifest.get("surface")
    if not isinstance(surfaces, list) or not surfaces:
        failures.append("surface must contain at least one [[surface]] table")
        return failures

    ids: set[str] = set()
    for index, surface in enumerate(surfaces):
        context = f"surface entry {index}"
        if not isinstance(surface, dict):
            failures.append(f"{context}: must be a table")
            continue

        unknown_fields = sorted(set(surface) - ALLOWED_SURFACE_FIELDS)
        if unknown_fields:
            failures.append(f"{context}: unknown fields: {', '.join(unknown_fields)}")

        surface_id = _required_text(failures, surface, "id", context)
        if surface_id:
            context = surface_id
            if surface_id in ids:
                failures.append(f"{context}: duplicate surface id")
            ids.add(surface_id)

        state = _required_text(failures, surface, "state", context)
        if state and state not in VALID_STATES:
            failures.append(f"{context}: state must be one of {', '.join(sorted(VALID_STATES))}")

        for key in ("owner", "issue", "reason"):
            _required_text(failures, surface, key, context)

        _required_string_list(failures, surface, "evidence_links", context)

        for key in (
            "declaration_files",
            "certification_rows",
            "registry_files",
            "preamble_files",
            "exact_intrinsics",
        ):
            _optional_string_list(failures, surface, key, context)

        removal_criteria = surface.get("removal_criteria")
        if state != "retained-by-design":
            if not isinstance(removal_criteria, list) or not removal_criteria:
                failures.append(f"{context}: removal_criteria is required for state {state}")
            else:
                _string_list_entries(failures, removal_criteria, "removal_criteria", context)
        elif removal_criteria is not None:
            _optional_string_list(failures, surface, "removal_criteria", context)

        has_owned_surface = any(
            surface.get(key)
            for key in ("registry_files", "preamble_files", "exact_intrinsics")
        )
        if not has_owned_surface:
            failures.append(f"{context}: must own registry_files, preamble_files, or exact_intrinsics")

    return failures


def _required_text(
    failures: list[str],
    table: dict[str, Any],
    key: str,
    context: str,
) -> str:
    value = table.get(key)
    if not isinstance(value, str) or not value.strip():
        failures.append(f"{context}: {key} must be a non-empty string")
        return ""
    return value.strip()


def _optional_string_list(
    failures: list[str],
    table: dict[str, Any],
    key: str,
    context: str,
) -> None:
    if key not in table:
        return
    value = table[key]
    if not isinstance(value, list):
        failures.append(f"{context}: {key} must be a list")
        return
    _string_list_entries(failures, value, key, context)


def _required_string_list(
    failures: list[str],
    table: dict[str, Any],
    key: str,
    context: str,
) -> None:
    value = table.get(key)
    if not isinstance(value, list) or not value:
        failures.append(f"{context}: {key} must be a non-empty list")
        return
    _string_list_entries(failures, value, key, context)


def _string_list_entries(
    failures: list[str],
    value: list[Any],
    key: str,
    context: str,
) -> None:
    for item in value:
        if not isinstance(item, str) or not item.strip():
            failures.append(f"{context}: {key} entries must be non-empty strings")


def _self_test() -> int:
    manifest = {
        "schema_version": EXPECTED_SCHEMA_VERSION,
        "surface": [
            {
                "id": "_sifr.example",
                "state": "retained",
                "owner": "stdlib-native-boundary-completion",
                "issue": "stdlib-native-boundary-completion",
                "removal_criteria": ["migration lands"],
                "evidence_links": ["stdlib-native-boundary-completion"],
                "reason": "test fixture",
                "registry_files": ["example.rs"],
            }
        ],
    }
    if _validate(manifest):
        print("self-test seed should pass", file=sys.stderr)
        return 1

    unknown = json.loads(json.dumps(manifest))
    unknown["surface"][0]["prefix_intrinsics"] = ["example_"]
    if not any("unknown fields: prefix_intrinsics" in failure for failure in _validate(unknown)):
        print("self-test unknown surface field was not rejected", file=sys.stderr)
        return 1

    missing_evidence = json.loads(json.dumps(manifest))
    missing_evidence["surface"][0].pop("evidence_links")
    if not any(
        "evidence_links must be a non-empty list" in failure
        for failure in _validate(missing_evidence)
    ):
        print("self-test missing evidence links were not rejected", file=sys.stderr)
        return 1

    missing_removal = json.loads(json.dumps(manifest))
    missing_removal["surface"][0].pop("removal_criteria")
    if not any("removal_criteria is required" in failure for failure in _validate(missing_removal)):
        print("self-test missing removal criteria was not rejected", file=sys.stderr)
        return 1

    bad_state = json.loads(json.dumps(manifest))
    bad_state["surface"][0]["state"] = "done"
    if not any("state must be one of" in failure for failure in _validate(bad_state)):
        print("self-test bad state was not rejected", file=sys.stderr)
        return 1

    print("stdlib retained manifest schema self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
