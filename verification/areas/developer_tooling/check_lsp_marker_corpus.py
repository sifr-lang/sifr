#!/usr/bin/env python3
"""Validate LSP capability inventory and marker corpus coverage."""

from __future__ import annotations

import argparse
import copy
import json
import re
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
INVENTORY_PATH = AREA_ROOT / "data" / "lsp_capability_inventory.json"
MARKER_MANIFEST_PATH = AREA_ROOT / "lsp_marker_corpus" / "manifest.json"
MARKER_ROOT = AREA_ROOT / "lsp_marker_corpus"
MARKER_PATTERN = re.compile(r"#\s*@lsp-marker\s+([A-Za-z0-9_.:-]+)")


class LspMarkerError(Exception):
    pass


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            run_self_test()
            print("LSP marker corpus self-test: PASS")
            return 0
        validate(load_json(INVENTORY_PATH), load_json(MARKER_MANIFEST_PATH))
    except LspMarkerError as error:
        print(f"LSP marker corpus: FAIL: {error}", file=sys.stderr)
        return 1
    print("LSP marker corpus: PASS")
    return 0


def validate(inventory: dict[str, Any], marker_manifest: dict[str, Any]) -> None:
    capabilities = validate_inventory(inventory)
    cases = validate_marker_manifest(marker_manifest)
    validate_capabilities_match_source(inventory, capabilities)
    validate_marker_coverage(capabilities, cases, marker_manifest)


def validate_inventory(inventory: dict[str, Any]) -> list[dict[str, Any]]:
    if inventory.get("schema_version") != 1:
        raise LspMarkerError("capability inventory schema_version must be 1")
    source = require_string(inventory, "source", "capability inventory")
    source_path = REPO_ROOT / source
    if not source_path.exists():
        raise LspMarkerError(f"capability inventory source is missing: {source}")
    raw_capabilities = inventory.get("capabilities")
    if not isinstance(raw_capabilities, list) or not raw_capabilities:
        raise LspMarkerError("capability inventory must include capabilities")
    ids = []
    for raw in raw_capabilities:
        if not isinstance(raw, dict):
            raise LspMarkerError("capability inventory entries must be objects")
        capability_id = require_string(raw, "id", "capability")
        ids.append(capability_id)
        require_string(raw, "server_capability_path", capability_id)
        require_string(raw, "source_token", capability_id)
        require_string(raw, "category", capability_id)
        require_string_list(raw, "methods", capability_id)
        if "commands" in raw:
            require_string_list(raw, "commands", capability_id)
        if not isinstance(raw.get("marker_required"), bool):
            raise LspMarkerError(f"capability {capability_id} marker_required must be boolean")
    if ids != sorted(ids):
        raise LspMarkerError("capability inventory ids must be sorted")
    if len(ids) != len(set(ids)):
        raise LspMarkerError("capability inventory ids must be unique")
    return raw_capabilities


def validate_capabilities_match_source(inventory: dict[str, Any], capabilities: list[dict[str, Any]]) -> None:
    source_text = (REPO_ROOT / str(inventory["source"])).read_text(encoding="utf-8")
    missing = []
    for capability in capabilities:
        token = str(capability["source_token"])
        if token not in source_text:
            missing.append(f"{capability['id']}:{token}")
        for command in capability.get("commands", []):
            if str(command) not in source_text:
                missing.append(f"{capability['id']}:{command}")
    if missing:
        raise LspMarkerError(f"capability inventory references tokens absent from capabilities.rs: {missing}")


def validate_marker_manifest(marker_manifest: dict[str, Any]) -> list[dict[str, Any]]:
    if marker_manifest.get("schema_version") != 1:
        raise LspMarkerError("marker corpus schema_version must be 1")
    require_string_list(marker_manifest, "required_categories", "marker corpus")
    raw_cases = marker_manifest.get("cases")
    if not isinstance(raw_cases, list) or not raw_cases:
        raise LspMarkerError("marker corpus must include cases")
    ids = []
    for raw in raw_cases:
        if not isinstance(raw, dict):
            raise LspMarkerError("marker corpus cases must be objects")
        case_id = require_string(raw, "id", "marker case")
        ids.append(case_id)
        fixture = require_string(raw, "fixture", f"marker case {case_id}")
        require_string(raw, "category", f"marker case {case_id}")
        covers = require_string_list(raw, "covers", f"marker case {case_id}")
        markers = require_string_list(raw, "markers", f"marker case {case_id}")
        fixture_markers = markers_in_fixture(MARKER_ROOT / fixture)
        supporting = raw.get("supporting_files", [])
        if supporting:
            if not isinstance(supporting, list) or not all(isinstance(item, str) and item for item in supporting):
                raise LspMarkerError(f"marker case {case_id} supporting_files must be string list")
            for path in supporting:
                fixture_markers.update(markers_in_fixture(MARKER_ROOT / path))
        missing_markers = sorted(set(markers) - fixture_markers)
        if missing_markers:
            raise LspMarkerError(f"marker case {case_id} markers missing from fixture text: {missing_markers}")
        if len(covers) != len(set(covers)):
            raise LspMarkerError(f"marker case {case_id} has duplicate capability coverage")
    if ids != sorted(ids):
        raise LspMarkerError("marker corpus case ids must be sorted")
    if len(ids) != len(set(ids)):
        raise LspMarkerError("marker corpus case ids must be unique")
    return raw_cases


def markers_in_fixture(path: Path) -> set[str]:
    if not path.exists():
        raise LspMarkerError(f"marker fixture missing: {path.relative_to(REPO_ROOT)}")
    return set(MARKER_PATTERN.findall(path.read_text(encoding="utf-8")))


def validate_marker_coverage(
    capabilities: list[dict[str, Any]],
    cases: list[dict[str, Any]],
    marker_manifest: dict[str, Any],
) -> None:
    capability_ids = {str(capability["id"]) for capability in capabilities}
    covered = {capability for case in cases for capability in case["covers"]}
    unknown = sorted(covered - capability_ids)
    if unknown:
        raise LspMarkerError(f"marker corpus covers unknown capabilities: {unknown}")
    required = {str(capability["id"]) for capability in capabilities if capability["marker_required"]}
    missing = sorted(required - covered)
    if missing:
        raise LspMarkerError(f"marker corpus missing required capability coverage: {missing}")
    category_by_capability = {str(capability["id"]): str(capability["category"]) for capability in capabilities}
    categories = {str(case["category"]) for case in cases}
    categories.update(category_by_capability[capability] for capability in covered if capability in category_by_capability)
    missing_categories = sorted(set(marker_manifest["required_categories"]) - categories)
    if missing_categories:
        raise LspMarkerError(f"marker corpus missing required categories: {missing_categories}")


def run_self_test() -> None:
    inventory = load_json(INVENTORY_PATH)
    manifest = load_json(MARKER_MANIFEST_PATH)
    missing_case = copy.deepcopy(manifest)
    missing_case["cases"] = missing_case["cases"][1:]
    assert_fails(lambda: validate(inventory, missing_case), "missing required capability coverage")
    bad_marker = copy.deepcopy(manifest)
    bad_marker["cases"][0]["markers"].append("missing-marker")
    assert_fails(lambda: validate(inventory, bad_marker), "markers missing from fixture text")
    bad_inventory = copy.deepcopy(inventory)
    bad_inventory["capabilities"][0]["source_token"] = "__missing_lsp_capability_token__"
    assert_fails(lambda: validate(bad_inventory, manifest), "tokens absent")


def assert_fails(action: Any, expected: str) -> None:
    try:
        action()
    except LspMarkerError as error:
        if expected not in str(error):
            raise LspMarkerError(f"negative self-test failed with wrong diagnostic: {error}") from error
        return
    raise LspMarkerError(f"negative self-test did not fail; expected {expected!r}")


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise LspMarkerError(f"failed to read {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise LspMarkerError(f"malformed JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise LspMarkerError(f"{path} root must be an object")
    return value


def require_string(raw: dict[str, Any], field: str, owner: str) -> str:
    value = raw.get(field)
    if not isinstance(value, str) or not value:
        raise LspMarkerError(f"{owner} field {field} must be a non-empty string")
    return value


def require_string_list(raw: dict[str, Any], field: str, owner: str) -> list[str]:
    value = raw.get(field)
    if not isinstance(value, list) or not value or not all(isinstance(item, str) and item for item in value):
        raise LspMarkerError(f"{owner} field {field} must be a non-empty string list")
    return value


if __name__ == "__main__":
    raise SystemExit(main())
