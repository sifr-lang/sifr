"""Validate Rust interop tier metadata."""

from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "rust_interop"
MATRIX_PATH = AREA_ROOT / "data" / "rust_interop_fixture_matrix.json"
TIERS_PATH = AREA_ROOT / "data" / "rust_interop_tiers.toml"


def main() -> int:
    matrix = json.loads(MATRIX_PATH.read_text(encoding="utf-8"))
    tiers = tomllib.loads(TIERS_PATH.read_text(encoding="utf-8"))
    failures: list[str] = []

    expected_tiers = {f"tier{index}" for index in range(5)}
    actual_tiers = set(tiers)
    failures.extend(f"missing tier definition: {tier}" for tier in sorted(expected_tiers - actual_tiers))
    failures.extend(f"unexpected tier definition: {tier}" for tier in sorted(actual_tiers - expected_tiers))

    matrix_tiers = {
        str(fixture["id"]): int(fixture["tier"])
        for fixture in matrix.get("fixtures", [])
        if isinstance(fixture, dict) and "id" in fixture and "tier" in fixture
    }
    tier_assignments: dict[str, int] = {}
    for tier_name, tier_data in tiers.items():
        if not isinstance(tier_data, dict):
            failures.append(f"{tier_name}: tier data must be a table")
            continue
        if not tier_data.get("name"):
            failures.append(f"{tier_name}: name is required")
        if not tier_data.get("description"):
            failures.append(f"{tier_name}: description is required")
        try:
            tier_number = int(tier_name.removeprefix("tier"))
        except ValueError:
            continue
        fixtures = tier_data.get("fixtures", [])
        if not isinstance(fixtures, list) or not fixtures:
            failures.append(f"{tier_name}: fixtures must be a non-empty list")
            continue
        for fixture_id in fixtures:
            key = str(fixture_id)
            if key in tier_assignments:
                failures.append(f"{key}: assigned to multiple tiers")
            tier_assignments[key] = tier_number

    for fixture_id, matrix_tier in sorted(matrix_tiers.items()):
        assigned_tier = tier_assignments.get(fixture_id)
        if assigned_tier is None:
            failures.append(f"{fixture_id}: missing tier assignment")
        elif assigned_tier != matrix_tier:
            failures.append(
                f"{fixture_id}: tier matrix mismatch, matrix={matrix_tier} tiers={assigned_tier}"
            )
    for fixture_id in sorted(set(tier_assignments) - set(matrix_tiers)):
        failures.append(f"{fixture_id}: tier assignment has no matrix fixture")

    if failures:
        for failure in failures:
            print(f"rust interop tier error: {failure}", file=sys.stderr)
        return 1
    print(f"rust interop tiers ok: tiers={len(actual_tiers)} fixtures={len(tier_assignments)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
