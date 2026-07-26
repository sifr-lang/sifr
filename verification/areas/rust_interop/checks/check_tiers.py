"""Validate Rust interop tier metadata."""

from __future__ import annotations

import json
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "rust_interop"
MATRIX_PATH = AREA_ROOT / "data" / "rust_interop_fixture_matrix.json"
TIERS_PATH = AREA_ROOT / "data" / "rust_interop_tiers.toml"


def _validate_tiers(matrix: dict[str, Any], tiers: dict[str, Any]) -> list[str]:
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

    return failures


def _load_and_validate(matrix_path: Path, tiers_path: Path) -> list[str]:
    matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    tiers = tomllib.loads(tiers_path.read_text(encoding="utf-8"))
    return _validate_tiers(matrix, tiers)


def _run_self_test() -> int:
    matrix = json.loads(MATRIX_PATH.read_text(encoding="utf-8"))
    tiers = tomllib.loads(TIERS_PATH.read_text(encoding="utf-8"))
    cases: tuple[tuple[str, dict[str, Any], dict[str, Any], str], ...] = (
        (
            "missing assignment",
            matrix,
            _without_fixture(tiers, "dotted_path_resolution"),
            "dotted_path_resolution: missing tier assignment",
        ),
        (
            "duplicate assignment",
            matrix,
            _with_fixture(tiers, "tier1", "dotted_path_resolution"),
            "dotted_path_resolution: assigned to multiple tiers",
        ),
        (
            "matrix mismatch",
            _with_matrix_tier(matrix, "dotted_path_resolution", 1),
            tiers,
            "dotted_path_resolution: tier matrix mismatch",
        ),
        (
            "invalid tier name",
            matrix,
            _with_invalid_tier_name(tiers),
            "unexpected tier definition: tierx",
        ),
        (
            "empty fixture list",
            matrix,
            _with_empty_fixtures(tiers, "tier0"),
            "tier0: fixtures must be a non-empty list",
        ),
    )
    with tempfile.TemporaryDirectory(prefix="sifr-rust-interop-tiers-") as temp:
        root = Path(temp)
        control_matrix_path = root / "matrix-control.json"
        control_tiers_path = root / "tiers-control.toml"
        control_matrix_path.write_text(json.dumps(matrix), encoding="utf-8")
        control_tiers_path.write_text(_render_tiers(tiers), encoding="utf-8")
        control_failures = _load_and_validate(control_matrix_path, control_tiers_path)
        if control_failures:
            print(
                "rust interop tier self-test error: "
                f"valid temporary tier data was rejected: {control_failures}",
                file=sys.stderr,
            )
            return 1
        for index, (name, case_matrix, case_tiers, expected) in enumerate(cases):
            matrix_path = root / f"matrix-{index}.json"
            tiers_path = root / f"tiers-{index}.toml"
            matrix_path.write_text(json.dumps(case_matrix), encoding="utf-8")
            tiers_path.write_text(_render_tiers(case_tiers), encoding="utf-8")
            failures = _load_and_validate(matrix_path, tiers_path)
            if not any(expected in failure for failure in failures):
                print(
                    f"rust interop tier self-test error: {name} did not report {expected!r}",
                    file=sys.stderr,
                )
                return 1
    print(f"rust interop tier self-test ok: cases={len(cases) + 1}")
    return 0


def _without_fixture(tiers: dict[str, Any], fixture_id: str) -> dict[str, Any]:
    result = _clone(tiers)
    for tier in result.values():
        if isinstance(tier, dict) and isinstance(tier.get("fixtures"), list):
            tier["fixtures"] = [item for item in tier["fixtures"] if item != fixture_id]
    return result


def _with_fixture(tiers: dict[str, Any], tier_name: str, fixture_id: str) -> dict[str, Any]:
    result = _clone(tiers)
    result[tier_name]["fixtures"].append(fixture_id)
    return result


def _with_matrix_tier(
    matrix: dict[str, Any],
    fixture_id: str,
    tier_number: int,
) -> dict[str, Any]:
    result = _clone(matrix)
    for fixture in result["fixtures"]:
        if fixture.get("id") == fixture_id:
            fixture["tier"] = tier_number
            break
    return result


def _with_invalid_tier_name(tiers: dict[str, Any]) -> dict[str, Any]:
    result = _clone(tiers)
    result["tierx"] = result.pop("tier4")
    return result


def _with_empty_fixtures(tiers: dict[str, Any], tier_name: str) -> dict[str, Any]:
    result = _clone(tiers)
    result[tier_name]["fixtures"] = []
    return result


def _clone(value: dict[str, Any]) -> dict[str, Any]:
    return json.loads(json.dumps(value))


def _render_tiers(tiers: dict[str, Any]) -> str:
    sections: list[str] = []
    for tier_name, tier_data in tiers.items():
        lines = [
            f"[{tier_name}]",
            f"name = {json.dumps(tier_data.get('name', ''))}",
            f"description = {json.dumps(tier_data.get('description', ''))}",
            "fixtures = [",
        ]
        lines.extend(f"  {json.dumps(fixture)}," for fixture in tier_data.get("fixtures", []))
        lines.append("]")
        sections.append("\n".join(lines))
    return "\n\n".join(sections) + "\n"


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    if args == ["--self-test"]:
        return _run_self_test()
    if args:
        print(f"usage: {Path(__file__).name} [--self-test]", file=sys.stderr)
        return 2

    tiers = tomllib.loads(TIERS_PATH.read_text(encoding="utf-8"))
    failures = _load_and_validate(MATRIX_PATH, TIERS_PATH)
    if failures:
        for failure in failures:
            print(f"rust interop tier error: {failure}", file=sys.stderr)
        return 1
    fixture_count = sum(
        len(data.get("fixtures", []))
        for data in tiers.values()
        if isinstance(data, dict)
    )
    print(f"rust interop tiers ok: tiers={len(tiers)} fixtures={fixture_count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
