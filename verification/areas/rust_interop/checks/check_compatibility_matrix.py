"""Validate Rust interop compatibility statements against fixture evidence."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "rust_interop"
FIXTURE_MATRIX_PATH = AREA_ROOT / "data" / "rust_interop_fixture_matrix.json"
COMPATIBILITY_MATRIX_PATH = AREA_ROOT / "data" / "rust_interop_compatibility_matrix.json"

VALID_CATEGORIES = {
    "supported",
    "supported-through-bridge",
    "unsupported-by-design",
    "future-owned-by-separate-phase",
}
CLAIMED_SUPPORT_CATEGORIES = {"supported", "supported-through-bridge", "unsupported-by-design"}
FUTURE_OWNER_PREFIXES = ("plans/issues/active/", "plans/phases/")


def main() -> int:
    fixture_matrix = json.loads(FIXTURE_MATRIX_PATH.read_text(encoding="utf-8"))
    compatibility_matrix = json.loads(COMPATIBILITY_MATRIX_PATH.read_text(encoding="utf-8"))
    failures: list[str] = []

    if compatibility_matrix.get("schema_version") != 1:
        failures.append("compatibility matrix schema_version must be 1")
    if compatibility_matrix.get("phase") != fixture_matrix.get("phase"):
        failures.append("compatibility matrix phase must match fixture matrix phase")
    if compatibility_matrix.get("source_fixture_matrix") != str(
        FIXTURE_MATRIX_PATH.relative_to(REPO_ROOT)
    ):
        failures.append("compatibility matrix source_fixture_matrix points at the wrong fixture matrix")

    categories = compatibility_matrix.get("categories")
    if not isinstance(categories, dict):
        failures.append("compatibility matrix categories must be an object")
        categories = {}
    actual_categories = set(categories)
    failures.extend(f"missing compatibility category: {item}" for item in sorted(VALID_CATEGORIES - actual_categories))
    failures.extend(f"unexpected compatibility category: {item}" for item in sorted(actual_categories - VALID_CATEGORIES))

    fixtures = {
        str(fixture.get("id")): fixture
        for fixture in fixture_matrix.get("fixtures", [])
        if isinstance(fixture, dict)
    }
    rows = compatibility_matrix.get("rows", [])
    if not isinstance(rows, list):
        failures.append("compatibility matrix rows must be a list")
        rows = []

    seen_rows: set[str] = set()
    fixture_rows: set[str] = set()
    seen_categories: set[str] = set()
    for row in rows:
        if not isinstance(row, dict):
            failures.append("compatibility matrix rows must be objects")
            continue
        _validate_row(failures, row, fixtures, seen_rows, fixture_rows, seen_categories)

    failures.extend(f"missing compatibility row for fixture: {item}" for item in sorted(set(fixtures) - fixture_rows))
    failures.extend(f"compatibility category is unused: {item}" for item in sorted(VALID_CATEGORIES - seen_categories))

    if failures:
        for failure in failures:
            print(f"rust interop compatibility matrix error: {failure}", file=sys.stderr)
        return 1
    print(
        "rust interop compatibility matrix ok: "
        f"rows={len(seen_rows)} fixture_rows={len(fixture_rows)} categories={len(seen_categories)}"
    )
    return 0


def _validate_row(
    failures: list[str],
    row: dict[str, Any],
    fixtures: dict[str, dict[str, Any]],
    seen_rows: set[str],
    fixture_rows: set[str],
    seen_categories: set[str],
) -> None:
    row_id = str(row.get("id", ""))
    if not row_id:
        failures.append("compatibility row id is required")
    elif row_id in seen_rows:
        failures.append(f"{row_id}: compatibility row ids must be unique")
    else:
        seen_rows.add(row_id)

    category = row.get("category")
    if category not in VALID_CATEGORIES:
        failures.append(f"{row_id}: invalid compatibility category")
    else:
        seen_categories.add(str(category))

    fixture_id = row.get("fixture")
    if not isinstance(fixture_id, str) or not fixture_id:
        failures.append(f"{row_id}: fixture is required")
        return
    fixture = fixtures.get(fixture_id)
    if fixture is None:
        failures.append(f"{row_id}: fixture does not exist in fixture matrix")
        return
    if fixture_id in fixture_rows:
        failures.append(f"{fixture_id}: compatibility fixture rows must be unique")
    fixture_rows.add(fixture_id)

    _expect_equal(failures, row_id, row, fixture, "tier")
    _expect_equal(failures, row_id, row, fixture, "capability")
    _expect_equal(failures, row_id, row, fixture, "execution_kind")
    _expect_equal(failures, row_id, row, fixture, "required_crates")
    _expect_equal(failures, row_id, row, fixture, "positive_evidence")
    _expect_equal(failures, row_id, row, fixture, "negative_evidence")

    positive_status = _evidence_status(fixture, "positive_evidence")
    negative_status = _evidence_status(fixture, "negative_evidence")
    if category in CLAIMED_SUPPORT_CATEGORIES and (positive_status, negative_status) != ("passing", "passing"):
        failures.append(
            f"{row_id}: {category} rows require passing positive and negative fixture evidence"
        )
    if category == "future-owned-by-separate-phase":
        if (positive_status, negative_status) == ("passing", "passing"):
            failures.append(f"{row_id}: future-owned row already has passing positive and negative evidence")
        future_owner = row.get("future_owner")
        if not isinstance(future_owner, str) or not future_owner:
            failures.append(f"{row_id}: future-owned row must name future_owner")
        elif not future_owner.startswith(FUTURE_OWNER_PREFIXES):
            failures.append(
                f"{row_id}: future_owner must reference plans/issues/active/ or plans/phases/"
            )
        elif not (REPO_ROOT / future_owner).is_file():
            failures.append(f"{row_id}: future_owner does not exist: {future_owner}")
    if not row.get("notes"):
        failures.append(f"{row_id}: notes are required")


def _expect_equal(
    failures: list[str],
    row_id: str,
    row: dict[str, Any],
    fixture: dict[str, Any],
    field: str,
) -> None:
    if row.get(field) != fixture.get(field):
        failures.append(f"{row_id}: {field} must match fixture matrix")


def _evidence_status(fixture: dict[str, Any], field: str) -> str:
    evidence = fixture.get(field)
    if not isinstance(evidence, dict):
        return ""
    return str(evidence.get("status", ""))


if __name__ == "__main__":
    raise SystemExit(main())
