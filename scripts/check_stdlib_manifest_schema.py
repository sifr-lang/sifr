#!/usr/bin/env python3
"""Validate the retained compiler-native stdlib glue manifest schema."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "internal_docs" / "stdlib_retained_compiler_intrinsics.toml"

EXPECTED_SCHEMA_VERSION = 2
ALLOWED_TOP_LEVEL_FIELDS = {"schema_version", "surface", "closed_surface"}
ALLOWED_CLOSURE_FIELDS = {
    "id",
    "previous_state",
    "removed_in_pr",
    "evidence_links",
    "reason",
}
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
    "retained_direct_dependency_packages",
    "direct_runtime_roots",
}
VALID_STATES = {"retained", "pilot", "closing", "retained-by-design"}
ALLOWED_STATE_TRANSITIONS = {
    ("retained", "pilot"),
    ("retained", "closing"),
    ("retained", "retained-by-design"),
    ("pilot", "closing"),
    ("pilot", "retained-by-design"),
}
DEFAULT_BASE_REF = "origin/main"


def main() -> int:
    manifest = tomllib.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    failures = _validate(manifest)
    base_manifest, base_error = _base_manifest()
    if base_manifest is None:
        base_ref = os.environ.get("SIFR_STDLIB_MANIFEST_BASE_REF", DEFAULT_BASE_REF)
        failures.append(
            f"could not load retained manifest from {base_ref} for transition "
            f"validation: {base_error}"
        )
    else:
        failures.extend(_validate_transitions(manifest, base_manifest))
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

    closure_records = manifest.get("closed_surface", [])
    if closure_records and not isinstance(closure_records, list):
        failures.append("closed_surface must be a list of [[closed_surface]] tables")
    elif isinstance(closure_records, list):
        _validate_closure_records(failures, closure_records)

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
            "retained_direct_dependency_packages",
            "direct_runtime_roots",
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
            for key in (
                "registry_files",
                "preamble_files",
                "exact_intrinsics",
                "retained_direct_dependency_packages",
                "direct_runtime_roots",
            )
        )
        if not has_owned_surface:
            failures.append(f"{context}: must own registry_files, preamble_files, or exact_intrinsics")

    return failures


def _validate_closure_records(
    failures: list[str],
    closure_records: list[Any],
) -> None:
    ids: set[str] = set()
    for index, record in enumerate(closure_records):
        context = f"closed_surface entry {index}"
        if not isinstance(record, dict):
            failures.append(f"{context}: must be a table")
            continue

        unknown_fields = sorted(set(record) - ALLOWED_CLOSURE_FIELDS)
        if unknown_fields:
            failures.append(f"{context}: unknown fields: {', '.join(unknown_fields)}")

        record_id = _required_text(failures, record, "id", context)
        if record_id:
            context = record_id
            if record_id in ids:
                failures.append(f"{context}: duplicate closed_surface id")
            ids.add(record_id)

        previous_state = _required_text(failures, record, "previous_state", context)
        if previous_state and previous_state != "closing":
            failures.append(f"{context}: previous_state must be closing")
        removed_in_pr = _required_text(failures, record, "removed_in_pr", context)
        if removed_in_pr and not _is_pr_reference(removed_in_pr):
            failures.append(f"{context}: removed_in_pr must reference a PR")
        _required_text(failures, record, "reason", context)
        _required_string_list(failures, record, "evidence_links", context)


def _validate_transitions(
    current_manifest: dict[str, Any],
    base_manifest: dict[str, Any],
) -> list[str]:
    failures: list[str] = []
    current_surfaces = _surface_states(current_manifest)
    base_surfaces = _surface_states(base_manifest)
    closure_records = _closure_ids(current_manifest)

    for surface_id, current_state in sorted(current_surfaces.items()):
        base_state = base_surfaces.get(surface_id)
        if base_state is None:
            if current_state != "retained-by-design":
                failures.append(
                    f"{surface_id}: new manifest rows must be retained-by-design, "
                    f"not {current_state}"
                )
            continue
        if current_state == base_state:
            continue
        if (base_state, current_state) not in ALLOWED_STATE_TRANSITIONS:
            failures.append(
                f"{surface_id}: invalid state transition {base_state} -> {current_state}"
            )

    for surface_id, base_state in sorted(base_surfaces.items()):
        if surface_id in current_surfaces:
            continue
        if base_state != "closing":
            failures.append(
                f"{surface_id}: only closing rows may be deleted, got {base_state}"
            )
            continue
        if surface_id not in closure_records:
            failures.append(
                f"{surface_id}: deleted closing row requires a closed_surface "
                "record with PR-linked evidence"
            )

    active_closed = sorted(set(current_surfaces) & closure_records)
    if active_closed:
        failures.append(
            "closed_surface records must not reference active surface rows: "
            + ", ".join(active_closed)
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


def _closure_ids(manifest: dict[str, Any]) -> set[str]:
    ids: set[str] = set()
    for record in manifest.get("closed_surface", []):
        if not isinstance(record, dict):
            continue
        record_id = record.get("id")
        if isinstance(record_id, str):
            ids.add(record_id)
    return ids


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


def _is_pr_reference(value: str) -> bool:
    return (
        re.search(r"\bPR\s+#\d+\b", value, flags=re.IGNORECASE) is not None
        or re.search(r"/pull/\d+\b", value) is not None
    )


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

    base_manifest = json.loads(json.dumps(manifest))
    for base_state, current_state in sorted(ALLOWED_STATE_TRANSITIONS):
        transition_base = json.loads(json.dumps(manifest))
        transition_base["surface"][0]["state"] = base_state
        transitioned = json.loads(json.dumps(manifest))
        transitioned["surface"][0]["state"] = current_state
        if _validate_transitions(transitioned, transition_base):
            print(
                f"self-test allowed {base_state} -> {current_state} transition failed",
                file=sys.stderr,
            )
            return 1

    invalid_transition = json.loads(json.dumps(manifest))
    invalid_transition["surface"][0]["state"] = "closing"
    invalid_base = json.loads(json.dumps(manifest))
    invalid_base["surface"][0]["state"] = "retained-by-design"
    if not any(
        "invalid state transition retained-by-design -> closing" in failure
        for failure in _validate_transitions(invalid_transition, invalid_base)
    ):
        print("self-test invalid transition was not rejected", file=sys.stderr)
        return 1

    new_retained = json.loads(json.dumps(manifest))
    new_retained["surface"].append(
        {
            "id": "_sifr.new",
            "state": "retained",
            "owner": "stdlib-native-boundary-completion",
            "issue": "stdlib-native-boundary-completion",
            "removal_criteria": ["test fixture"],
            "evidence_links": ["stdlib-native-boundary-completion"],
            "reason": "test fixture",
            "registry_files": ["new.rs"],
        }
    )
    if not any(
        "new manifest rows must be retained-by-design" in failure
        for failure in _validate_transitions(new_retained, base_manifest)
    ):
        print("self-test new retained row was not rejected", file=sys.stderr)
        return 1

    new_design = json.loads(json.dumps(manifest))
    new_design["surface"].append(
        {
            "id": "_sifr.new_design",
            "state": "retained-by-design",
            "owner": "stdlib-native-boundary-completion",
            "issue": "stdlib-native-boundary-completion",
            "evidence_links": ["stdlib-native-boundary-completion"],
            "reason": "test fixture",
            "registry_files": ["new_design.rs"],
        }
    )
    if _validate_transitions(new_design, base_manifest):
        print("self-test new retained-by-design row failed", file=sys.stderr)
        return 1

    closing_base = json.loads(json.dumps(manifest))
    closing_base["surface"][0]["state"] = "closing"
    deleted_without_closure = json.loads(json.dumps(manifest))
    deleted_without_closure["surface"] = []
    if not any(
        "deleted closing row requires a closed_surface record" in failure
        for failure in _validate_transitions(deleted_without_closure, closing_base)
    ):
        print("self-test deleted closing row without closure was not rejected", file=sys.stderr)
        return 1

    deleted_with_closure = json.loads(json.dumps(deleted_without_closure))
    deleted_with_closure["closed_surface"] = [
        {
            "id": "_sifr.example",
            "previous_state": "closing",
            "removed_in_pr": "PR #9999",
            "evidence_links": ["PR #9999"],
            "reason": "test closure",
        }
    ]
    if _validate_transitions(deleted_with_closure, closing_base):
        print("self-test deleted closing row with closure failed", file=sys.stderr)
        return 1

    deleted_retained = json.loads(json.dumps(deleted_without_closure))
    if not any(
        "only closing rows may be deleted" in failure
        for failure in _validate_transitions(deleted_retained, base_manifest)
    ):
        print("self-test deleted retained row was not rejected", file=sys.stderr)
        return 1

    active_closed_surface = json.loads(json.dumps(manifest))
    active_closed_surface["closed_surface"] = [
        {
            "id": "_sifr.example",
            "previous_state": "closing",
            "removed_in_pr": "PR #9999",
            "evidence_links": ["PR #9999"],
            "reason": "test closure",
        }
    ]
    if not any(
        "closed_surface records must not reference active surface rows" in failure
        for failure in _validate_transitions(active_closed_surface, base_manifest)
    ):
        print("self-test active closed_surface record was not rejected", file=sys.stderr)
        return 1

    bad_closure = json.loads(json.dumps(manifest))
    bad_closure["closed_surface"] = deleted_with_closure["closed_surface"]
    bad_closure["closed_surface"][0]["previous_state"] = "retained"
    if not any(
        "previous_state must be closing" in failure
        for failure in _validate(bad_closure)
    ):
        print("self-test bad closure state was not rejected", file=sys.stderr)
        return 1

    bad_closure_pr = json.loads(json.dumps(manifest))
    bad_closure_pr["closed_surface"] = [
        {
            "id": "_sifr.closed",
            "previous_state": "closing",
            "removed_in_pr": "merge evidence pending",
            "evidence_links": ["PR #9999"],
            "reason": "test closure",
        }
    ]
    if not any(
        "removed_in_pr must reference a PR" in failure
        for failure in _validate(bad_closure_pr)
    ):
        print("self-test bad closure PR reference was not rejected", file=sys.stderr)
        return 1

    duplicate_closure = json.loads(json.dumps(manifest))
    duplicate_closure["closed_surface"] = [
        {
            "id": "_sifr.closed",
            "previous_state": "closing",
            "removed_in_pr": "PR #9999",
            "evidence_links": ["PR #9999"],
            "reason": "test closure",
        },
        {
            "id": "_sifr.closed",
            "previous_state": "closing",
            "removed_in_pr": "PR #10000",
            "evidence_links": ["PR #10000"],
            "reason": "test closure",
        },
    ]
    if not any(
        "duplicate closed_surface id" in failure
        for failure in _validate(duplicate_closure)
    ):
        print("self-test duplicate closure record was not rejected", file=sys.stderr)
        return 1

    empty_surface_bad_closure = json.loads(json.dumps(bad_closure))
    empty_surface_bad_closure["surface"] = []
    empty_surface_failures = _validate(empty_surface_bad_closure)
    if not any("previous_state must be closing" in failure for failure in empty_surface_failures):
        print("self-test empty-surface bad closure was not validated", file=sys.stderr)
        return 1
    if not any("surface must contain" in failure for failure in empty_surface_failures):
        print("self-test empty-surface error was not preserved", file=sys.stderr)
        return 1

    pr_url_closure = json.loads(json.dumps(manifest))
    pr_url_closure["closed_surface"] = [
        {
            "id": "_sifr.closed",
            "previous_state": "closing",
            "removed_in_pr": "https://github.com/sifr-lang/sifr/pull/9999",
            "evidence_links": ["https://github.com/sifr-lang/sifr/pull/9999"],
            "reason": "test closure",
        }
    ]
    if _validate(pr_url_closure):
        print("self-test PR URL closure record failed", file=sys.stderr)
        return 1

    print("stdlib retained manifest schema self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
