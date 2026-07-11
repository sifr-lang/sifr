#!/usr/bin/env python3
"""Validate the final retained compiler-native stdlib glue manifest schema."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "internal_docs" / "stdlib_retained_compiler_intrinsics.toml"

EXPECTED_SCHEMA_VERSION = 2
FINAL_STATE = "retained-by-design"
ALLOWED_TOP_LEVEL_FIELDS = {"schema_version", "surface"}
ALLOWED_SURFACE_FIELDS = {
    "id",
    "state",
    "owner",
    "issue",
    "evidence_links",
    "declaration_files",
    "certification_rows",
    "reason",
    "registry_files",
    "preamble_files",
    "exact_intrinsics",
    "retained_direct_dependency_packages",
    "direct_runtime_roots",
}
REQUIRED_SURFACE_STATES = {
    "_sifr.task::language_runtime_glue": FINAL_STATE,
    "generated-test-glue": FINAL_STATE,
}
PLANNED_REARCHITECTURE_DELETIONS = {
    "_sifr.runtime::observability_glue",
    "_sifr.bytes::first_class_constructors",
    "_sifr.collections::counter_defaultdict",
    "retained-fallback-signature-glue",
}
DEFAULT_BASE_REF = "origin/main"


def main() -> int:
    manifest = tomllib.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    failures = _validate(manifest)
    failures.extend(_validate_required_surface_states(manifest, REQUIRED_SURFACE_STATES))
    base_manifest, base_error = _base_manifest()
    if base_manifest is None:
        base_ref = os.environ.get("SIFR_STDLIB_MANIFEST_BASE_REF", DEFAULT_BASE_REF)
        failures.append(
            f"could not load retained manifest from {base_ref} for transition "
            f"validation: {base_error}"
        )
    else:
        failures.extend(_validate_final_transitions(manifest, base_manifest))
    if failures:
        print("stdlib retained manifest schema: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print(
        "stdlib retained manifest schema: PASS "
        f"(surfaces={len(manifest.get('surface', []))}, "
        f"schema_version={EXPECTED_SCHEMA_VERSION}, final_state={FINAL_STATE})"
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
        if state and state != FINAL_STATE:
            failures.append(f"{context}: state must be {FINAL_STATE}")

        for key in ("owner", "issue", "reason"):
            _required_text(failures, surface, key, context)

        _required_string_list(failures, surface, "evidence_links", context)

        for key in (
            "declaration_files",
            "certification_rows",
            "registry_files",
            "preamble_files",
            "exact_intrinsics",
            "retained_direct_dependency_packages",
            "direct_runtime_roots",
        ):
            _optional_string_list(failures, surface, key, context)

        has_owned_surface = any(
            surface.get(key)
            for key in (
                "registry_files",
                "preamble_files",
                "exact_intrinsics",
                "retained_direct_dependency_packages",
                "direct_runtime_roots",
            )
        )
        if not has_owned_surface:
            failures.append(f"{context}: must own retained-by-design compiler glue")

    return failures


def _validate_final_transitions(
    current_manifest: dict[str, Any],
    base_manifest: dict[str, Any],
) -> list[str]:
    failures: list[str] = []
    current_surfaces = _surface_states(current_manifest)
    base_surfaces = _surface_states(base_manifest)

    for surface_id, current_state in sorted(current_surfaces.items()):
        base_state = base_surfaces.get(surface_id)
        if base_state is None:
            if current_state != FINAL_STATE:
                failures.append(
                    f"{surface_id}: new manifest rows must be {FINAL_STATE}, "
                    f"not {current_state}"
                )
            continue
        if base_state == "closing":
            failures.append(f"{surface_id}: final closure must delete closing rows")
            continue
        if current_state != base_state:
            failures.append(
                f"{surface_id}: invalid final manifest state change "
                f"{base_state} -> {current_state}"
            )

    for surface_id, base_state in sorted(base_surfaces.items()):
        if surface_id in current_surfaces:
            continue
        if surface_id in PLANNED_REARCHITECTURE_DELETIONS:
            continue
        if base_state != "closing":
            failures.append(
                f"{surface_id}: only closing rows may be deleted, got {base_state}"
            )

    return failures


def _validate_required_surface_states(
    manifest: dict[str, Any],
    required_states: dict[str, str],
) -> list[str]:
    failures: list[str] = []
    current_states = _surface_states(manifest)
    for surface_id, required_state in sorted(required_states.items()):
        current_state = current_states.get(surface_id)
        if current_state is None:
            failures.append(
                f"{surface_id}: required retained manifest surface is missing"
            )
            continue
        if current_state != required_state:
            failures.append(
                f"{surface_id}: state must remain {required_state}, got {current_state}"
            )
    return failures


def _surface_states(manifest: dict[str, Any]) -> dict[str, str]:
    states: dict[str, str] = {}
    for surface in manifest.get("surface", []):
        if not isinstance(surface, dict):
            continue
        surface_id = surface.get("id")
        state = surface.get("state")
        if isinstance(surface_id, str) and isinstance(state, str):
            states[surface_id] = state
    return states


def _base_manifest() -> tuple[dict[str, Any] | None, str]:
    base_ref = os.environ.get("SIFR_STDLIB_MANIFEST_BASE_REF", DEFAULT_BASE_REF)
    relative_path = MANIFEST_PATH.relative_to(REPO_ROOT).as_posix()
    result = subprocess.run(
        ["git", "show", f"{base_ref}:{relative_path}"],
        cwd=REPO_ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or f"git show exited {result.returncode}"
        return None, detail
    return tomllib.loads(result.stdout), ""


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
                "state": FINAL_STATE,
                "owner": "stdlib-native-boundary-completion",
                "issue": "stdlib-native-boundary-completion",
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
    unknown["closed_surface"] = []
    if not any("unknown top-level fields: closed_surface" in failure for failure in _validate(unknown)):
        print("self-test closed_surface was not rejected", file=sys.stderr)
        return 1

    unknown_field = json.loads(json.dumps(manifest))
    unknown_field["surface"][0]["removal_criteria"] = ["temporary"]
    if not any("unknown fields: removal_criteria" in failure for failure in _validate(unknown_field)):
        print("self-test removal_criteria was not rejected", file=sys.stderr)
        return 1

    missing_evidence = json.loads(json.dumps(manifest))
    missing_evidence["surface"][0].pop("evidence_links")
    if not any(
        "evidence_links must be a non-empty list" in failure
        for failure in _validate(missing_evidence)
    ):
        print("self-test missing evidence links were not rejected", file=sys.stderr)
        return 1

    bad_state = json.loads(json.dumps(manifest))
    bad_state["surface"][0]["state"] = "closing"
    if not any(
        "state must be retained-by-design" in failure for failure in _validate(bad_state)
    ):
        print("self-test bad state was not rejected", file=sys.stderr)
        return 1

    metadata_only = json.loads(json.dumps(manifest))
    metadata_only["surface"][0].pop("registry_files")
    metadata_only["surface"][0]["declaration_files"] = ["stdlib/_sifr/example.sifr"]
    if not any(
        "must own retained-by-design compiler glue" in failure
        for failure in _validate(metadata_only)
    ):
        print("self-test metadata-only row was not rejected", file=sys.stderr)
        return 1

    base_manifest = json.loads(json.dumps(manifest))
    closing_base = json.loads(json.dumps(manifest))
    closing_base["surface"][0]["state"] = "closing"
    deleted_closing = json.loads(json.dumps(manifest))
    deleted_closing["surface"] = []
    if _validate_final_transitions(deleted_closing, closing_base):
        print("self-test deleted closing row failed", file=sys.stderr)
        return 1

    retained_deleted = json.loads(json.dumps(manifest))
    retained_deleted["surface"] = []
    if not any(
        "only closing rows may be deleted" in failure
        for failure in _validate_final_transitions(retained_deleted, base_manifest)
    ):
        print("self-test deleted retained-by-design row was not rejected", file=sys.stderr)
        return 1

    closing_still_active = json.loads(json.dumps(manifest))
    if not any(
        "final closure must delete closing rows" in failure
        for failure in _validate_final_transitions(closing_still_active, closing_base)
    ):
        print("self-test active closing row was not rejected", file=sys.stderr)
        return 1

    new_design = json.loads(json.dumps(manifest))
    new_design["surface"].append(
        {
            "id": "_sifr.new_design",
            "state": FINAL_STATE,
            "owner": "stdlib-native-boundary-completion",
            "issue": "stdlib-native-boundary-completion",
            "evidence_links": ["stdlib-native-boundary-completion"],
            "reason": "test fixture",
            "registry_files": ["new_design.rs"],
        }
    )
    if _validate_final_transitions(new_design, base_manifest):
        print("self-test new retained-by-design row failed", file=sys.stderr)
        return 1

    new_bad_state = json.loads(json.dumps(manifest))
    new_bad_state["surface"].append(
        {
            "id": "_sifr.new_bad_state",
            "state": "pilot",
            "owner": "stdlib-native-boundary-completion",
            "issue": "stdlib-native-boundary-completion",
            "evidence_links": ["stdlib-native-boundary-completion"],
            "reason": "test fixture",
            "registry_files": ["new_bad_state.rs"],
        }
    )
    if not any(
        "_sifr.new_bad_state: new manifest rows must be retained-by-design"
        in failure
        for failure in _validate_final_transitions(new_bad_state, base_manifest)
    ):
        print("self-test new non-design row was not rejected", file=sys.stderr)
        return 1

    required_state_manifest = {
        "schema_version": EXPECTED_SCHEMA_VERSION,
        "surface": [
            {
                "id": "_sifr.task::language_runtime_glue",
                "state": FINAL_STATE,
                "owner": "stdlib-native-boundary-completion",
                "issue": "stdlib-native-boundary-completion",
                "evidence_links": ["stdlib-native-boundary-completion"],
                "reason": "test fixture",
                "registry_files": ["task.rs"],
            },
            {
                "id": "generated-test-glue",
                "state": FINAL_STATE,
                "owner": "stdlib-native-boundary-completion",
                "issue": "stdlib-native-boundary-completion",
                "evidence_links": ["stdlib-native-boundary-completion"],
                "reason": "test fixture",
                "registry_files": ["test.rs"],
            },
        ],
    }
    if _validate_required_surface_states(
        required_state_manifest, REQUIRED_SURFACE_STATES
    ):
        print("self-test required retained-by-design states failed", file=sys.stderr)
        return 1

    required_state_missing = json.loads(json.dumps(required_state_manifest))
    required_state_missing["surface"] = required_state_missing["surface"][:-1]
    if not any(
        "generated-test-glue: required retained manifest surface is missing"
        in failure
        for failure in _validate_required_surface_states(
            required_state_missing, REQUIRED_SURFACE_STATES
        )
    ):
        print("self-test missing required retained-by-design row was not rejected", file=sys.stderr)
        return 1

    print("stdlib retained manifest schema self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
