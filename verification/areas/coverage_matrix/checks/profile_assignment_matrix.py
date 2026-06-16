#!/usr/bin/env python3
"""Validate profile assignments against the closeout assignment matrix."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "coverage_matrix"
MATRIX_PATH = AREA_ROOT / "profile_assignment_matrix.json"
PROFILES_DIR = REPO_ROOT / "verification" / "profiles"
AREA_ROOTS = REPO_ROOT / "verification" / "areas"
PROFILE_NAMES = ("create-pr", "merge", "nightly", "release")


def main() -> int:
    errors: list[str] = []
    matrix = load_json(MATRIX_PATH, errors)
    if matrix.get("schema_version") != 1:
        errors.append("profile_assignment_matrix.json must use schema_version 1")
    rows = matrix.get("rows", [])
    if not isinstance(rows, list) or not rows:
        errors.append("profile_assignment_matrix.json must define non-empty rows")

    area_suites = load_area_suites(errors)
    profiles = {profile: load_profile(profile, errors) for profile in PROFILE_NAMES}
    profile_suites = {
        profile: selected_area_suite_tokens(payload)
        for profile, payload in profiles.items()
        if payload
    }
    seen: set[str] = set()
    for index, row in enumerate(rows if isinstance(rows, list) else []):
        if not isinstance(row, dict):
            errors.append(f"rows[{index}] must be an object")
            continue
        surface_id = require_string(row, "surface_id", f"rows[{index}]", errors)
        if not surface_id:
            continue
        if surface_id in seen:
            errors.append(f"{surface_id}: duplicate profile assignment row")
        seen.add(surface_id)
        assignments = row.get("profiles")
        if not isinstance(assignments, dict):
            errors.append(f"{surface_id}: profiles must be an object")
            continue
        for profile in PROFILE_NAMES:
            expected = assignments.get(profile)
            if not isinstance(expected, list):
                errors.append(f"{surface_id}: missing {profile} assignment list")
                continue
            validate_expected_tokens(surface_id, profile, expected, area_suites, errors)
            validate_row_membership(
                surface_id,
                profile,
                expected,
                profile_suites.get(profile, set()),
                errors,
            )

    if errors:
        for error in errors:
            print(f"profile-assignment-matrix error: {error}", file=sys.stderr)
        return 1
    print(f"profile assignment matrix ok: rows={len(rows)}")
    return 0


def load_json(path: Path, errors: list[str]) -> dict[str, Any]:
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


def load_profile(profile: str, errors: list[str]) -> dict[str, Any]:
    payload = load_json(PROFILES_DIR / f"{profile}.json", errors)
    if payload and payload.get("name") != profile:
        errors.append(f"{profile}.json has mismatched profile name")
    return payload


def load_area_suites(errors: list[str]) -> dict[str, set[str]]:
    suites: dict[str, set[str]] = {}
    for manifest in sorted(AREA_ROOTS.glob("*/manifest.json")):
        payload = load_json(manifest, errors)
        area = payload.get("name")
        raw_suites = payload.get("suites", [])
        if not isinstance(area, str) or not isinstance(raw_suites, list):
            continue
        suites[area] = {
            str(suite.get("name"))
            for suite in raw_suites
            if isinstance(suite, dict) and isinstance(suite.get("name"), str)
        }
    return suites


def selected_area_suite_tokens(profile: dict[str, Any]) -> set[str]:
    tokens: set[str] = set()
    for selection in profile.get("selected_areas", []):
        if not isinstance(selection, dict):
            continue
        area = selection.get("area")
        suites = selection.get("suites", [])
        if not isinstance(area, str) or not isinstance(suites, list):
            continue
        for suite in suites:
            if isinstance(suite, str):
                tokens.add(f"{area}:{suite}")
    return tokens


def validate_expected_tokens(
    surface_id: str,
    profile: str,
    expected: list[Any],
    area_suites: dict[str, set[str]],
    errors: list[str],
) -> None:
    for token in expected:
        if not isinstance(token, str) or not token:
            errors.append(f"{surface_id}: {profile} assignment tokens must be non-empty strings")
            continue
        if not is_area_suite_token(token):
            continue
        area, suite = token.split(":", maxsplit=1)
        if area not in area_suites:
            errors.append(f"{surface_id}: {profile} references unknown area {area}")
        elif suite not in area_suites[area]:
            errors.append(f"{surface_id}: {profile} references unknown suite {token}")


def validate_row_membership(
    surface_id: str,
    profile: str,
    expected: list[Any],
    actual: set[str],
    errors: list[str],
) -> None:
    for token in expected:
        if is_area_suite_token(token) and token not in actual:
            errors.append(f"{surface_id}: {profile} omits required suite {token}")


def is_area_suite_token(token: Any) -> bool:
    if not isinstance(token, str) or ":" not in token:
        return False
    area, suite = token.split(":", maxsplit=1)
    return bool(area and suite) and area not in {"cargo", "e2e", "sifr", "sifr_codegen"}


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
