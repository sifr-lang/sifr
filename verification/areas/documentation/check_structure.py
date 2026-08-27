#!/usr/bin/env python3
"""Validate documentation inventory and registered mutation harnesses."""

from __future__ import annotations

import copy
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from check_architecture import ARCHITECTURE_MUTATION_CASES
from check_ga_release_docs import MUTATION_CASES as GA_RELEASE_MUTATION_CASES

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
INVENTORY_PATH = AREA_ROOT / "docs_inventory.json"
MANIFEST_PATH = AREA_ROOT / "manifest.json"
EXPECTED_CHECKS = {
    "architecture": ("architecture", "active", "check_architecture.py"),
    "structure": ("structure", "active", "check_structure.py"),
    "ga-release": ("ga-release", "active", "check_ga_release_docs.py"),
}
STRUCTURE_MUTATION_CASES = (
    "missing-inventory-field",
    "duplicate-check",
    "missing-mutation-harness",
    "missing-active-suite",
)
EXPECTED_MUTATION_CASES = {
    "architecture": ARCHITECTURE_MUTATION_CASES,
    "structure": STRUCTURE_MUTATION_CASES,
    "ga-release": GA_RELEASE_MUTATION_CASES,
}


class StructureError(ValueError):
    """Documentation structure contract violation."""


def validate_inventory(
    payload: Any,
    *,
    require_files: bool = True,
    manifest_suites: set[str] | None = None,
) -> None:
    if not isinstance(payload, dict) or set(payload) != {
        "schema_version",
        "owner",
        "roots",
        "checks",
    }:
        raise StructureError("inventory must contain exactly the governed fields")
    if type(payload["schema_version"]) is not int or payload["schema_version"] != 2:
        raise StructureError("schema_version must be integer 2")
    if payload["owner"] != "documentation":
        raise StructureError("owner must be documentation")
    if payload["roots"] != ["docs", "internal_docs"]:
        raise StructureError("documentation roots must be docs and internal_docs")
    checks = payload["checks"]
    if not isinstance(checks, list):
        raise StructureError("checks must be an array")
    observed: dict[str, dict[str, Any]] = {}
    for check in checks:
        if not isinstance(check, dict) or set(check) != {
            "id",
            "suite",
            "command",
            "mutation_cases",
            "status",
        }:
            raise StructureError("each check must contain exactly the governed fields")
        check_id = check["id"]
        if not isinstance(check_id, str) or check_id in observed:
            raise StructureError("check ids must be unique strings")
        observed[check_id] = check
        mutations = check["mutation_cases"]
        if not isinstance(mutations, list) or len(mutations) < 3:
            raise StructureError(f"{check_id} must register a mutation harness")
        if len(set(mutations)) != len(mutations):
            raise StructureError(f"{check_id} mutation cases must be unique")
        if tuple(mutations) != EXPECTED_MUTATION_CASES.get(check_id):
            raise StructureError(f"{check_id} mutation case registration drifted")
    if set(observed) != set(EXPECTED_CHECKS):
        raise StructureError("inventory check registration drifted")
    for check_id, (suite, status, script_name) in EXPECTED_CHECKS.items():
        check = observed[check_id]
        if check["suite"] != suite or check["status"] != status:
            raise StructureError(f"{check_id} suite/status drifted")
        expected_command = ["python3", f"verification/areas/documentation/{script_name}"]
        if check["command"] != expected_command:
            raise StructureError(f"{check_id} command drifted")
        if require_files and not (AREA_ROOT / script_name).is_file():
            raise StructureError(f"{check_id} script is missing")
        if (
            check["status"] == "active"
            and manifest_suites is not None
            and check["suite"] not in manifest_suites
        ):
            raise StructureError(f"{check_id} active suite is missing from the manifest")


def load_manifest_suites() -> set[str]:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    suites = manifest.get("suites")
    if not isinstance(suites, list):
        raise StructureError("documentation manifest suites must be an array")
    names = {suite.get("name") for suite in suites if isinstance(suite, dict)}
    if None in names or len(names) != len(suites):
        raise StructureError("documentation manifest suites must have unique names")
    return names


def run_self_tests() -> None:
    payload = json.loads(INVENTORY_PATH.read_text(encoding="utf-8"))
    manifest_suites = load_manifest_suites()
    mutations = {
        "missing-inventory-field": lambda item: item.pop("owner"),
        "duplicate-check": lambda item: item["checks"].append(
            copy.deepcopy(item["checks"][0])
        ),
        "missing-mutation-harness": lambda item: item["checks"][0].update(
            {"mutation_cases": []}
        ),
        "missing-active-suite": lambda item: item["checks"][0].update(
            {"suite": "missing-active-suite"}
        ),
    }
    if tuple(mutations) != STRUCTURE_MUTATION_CASES:
        raise StructureError("structure mutation registration drifted")
    for case_id, callback in mutations.items():
        changed = copy.deepcopy(payload)
        callback(changed)
        try:
            validate_inventory(
                changed,
                require_files=False,
                manifest_suites=manifest_suites,
            )
        except StructureError:
            continue
        raise StructureError(f"structure mutation unexpectedly passed: {case_id}")


def main() -> int:
    try:
        payload = json.loads(INVENTORY_PATH.read_text(encoding="utf-8"))
        validate_inventory(payload, manifest_suites=load_manifest_suites())
        run_self_tests()
        subprocess.run(
            [sys.executable, str(AREA_ROOT / "check_ga_release_docs.py"), "--self-test"],
            cwd=REPO_ROOT,
            check=True,
        )
    except (OSError, json.JSONDecodeError, StructureError, subprocess.CalledProcessError) as exc:
        print(f"documentation-structure: {exc}", file=sys.stderr)
        return 2
    print("documentation structure ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
