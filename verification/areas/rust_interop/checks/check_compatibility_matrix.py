"""Validate Rust interop compatibility statements against fixture evidence."""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path
from typing import Any

from _provenance_checks import load_fixture_manifests
from _provenance_checks import load_profiles
from _provenance_checks import validate_evidence_provenance

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "rust_interop"
FIXTURES_ROOT = AREA_ROOT / "fixtures"
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


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    if args == ["--self-test"]:
        return _run_self_test()
    if args:
        print(f"usage: {Path(__file__).name} [--self-test]", file=sys.stderr)
        return 2

    fixture_matrix = json.loads(FIXTURE_MATRIX_PATH.read_text(encoding="utf-8"))
    compatibility_matrix = json.loads(COMPATIBILITY_MATRIX_PATH.read_text(encoding="utf-8"))
    failures: list[str] = []
    fixture_manifests = load_fixture_manifests(FIXTURES_ROOT, failures)
    profiles = load_profiles(REPO_ROOT)

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
    used_evidence_tests: dict[tuple[str, str], str] = {}
    for row in rows:
        if not isinstance(row, dict):
            failures.append("compatibility matrix rows must be objects")
            continue
        _validate_row(
            failures,
            row,
            fixtures,
            fixture_manifests,
            profiles,
            REPO_ROOT,
            used_evidence_tests,
            seen_rows,
            fixture_rows,
            seen_categories,
        )

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
    fixture_manifests: dict[str, dict[str, Any]],
    profiles: dict[str, dict[str, Any]],
    repo_root: Path,
    used_evidence_tests: dict[tuple[str, str], str],
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
    _expect_equal(failures, row_id, row, fixture, "diagnostic_crate_rationale")
    _expect_equal(failures, row_id, row, fixture, "positive_evidence")
    _expect_equal(failures, row_id, row, fixture, "negative_evidence")

    positive_status = _evidence_status(fixture, "positive_evidence")
    negative_status = _evidence_status(fixture, "negative_evidence")
    if category in CLAIMED_SUPPORT_CATEGORIES and (positive_status, negative_status) != ("passing", "passing"):
        failures.append(
            f"{row_id}: {category} rows require passing positive and negative fixture evidence"
        )
    if category in CLAIMED_SUPPORT_CATEGORIES:
        _validate_claimed_provenance(
            failures,
            row_id,
            fixture_id,
            fixture,
            fixture_manifests.get(fixture_id),
            profiles,
            repo_root,
            used_evidence_tests,
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
    notes = row.get("notes")
    if not isinstance(notes, str) or not notes.strip():
        failures.append(f"{row_id}: notes are required")
    elif row.get("execution_kind") == "contract-only" and "contract-only" not in notes.lower():
        failures.append(f"{row_id}: contract-only row notes must say contract-only")


def _validate_claimed_provenance(
    failures: list[str],
    row_id: str,
    fixture_id: str,
    fixture: dict[str, Any],
    manifest: Any,
    profiles: dict[str, dict[str, Any]],
    repo_root: Path,
    used_evidence_tests: dict[tuple[str, str], str],
) -> None:
    if not isinstance(manifest, dict):
        failures.append(f"{row_id}: claimed fixture has no fixture.json manifest")
        return
    if manifest.get("schema_version") != 2:
        failures.append(f"{row_id}: claimed fixture manifest schema_version must be 2")
    if manifest.get("id") != fixture_id:
        failures.append(f"{row_id}: claimed fixture manifest id must match fixture")
    evidence = manifest.get("evidence")
    if not isinstance(evidence, dict):
        failures.append(f"{row_id}: claimed fixture manifest evidence must be an object")
        return
    for side, matrix_field in (
        ("positive", "positive_evidence"),
        ("negative", "negative_evidence"),
    ):
        manifest_record = evidence.get(side)
        matrix_record = fixture.get(matrix_field)
        if not isinstance(manifest_record, dict):
            failures.append(f"{row_id}: claimed fixture manifest evidence.{side} is required")
            continue
        if not isinstance(matrix_record, dict):
            continue
        for field in ("id", "status"):
            if manifest_record.get(field) != matrix_record.get(field):
                failures.append(
                    f"{row_id}: claimed fixture manifest evidence.{side}.{field} "
                    "must match fixture matrix"
                )
        validate_evidence_provenance(
            failures,
            repo_root=repo_root,
            profiles=profiles,
            fixture_id=fixture_id,
            side=side,
            evidence=manifest_record,
            execution_kind=str(fixture.get("execution_kind")),
            used_tests=used_evidence_tests,
        )


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


def _run_self_test() -> int:
    rationale = {
        "purpose": "crate APIs supply diagnostic shapes only",
        "linked": False,
        "executed": False,
    }
    fixture = {
        "id": "diagnostic_fixture",
        "tier": 0,
        "capability": "diagnostic contract",
        "execution_kind": "compiler-diagnostic",
        "required_crates": ["example"],
        "diagnostic_crate_rationale": rationale,
        "positive_evidence": {"id": "positive", "status": "passing"},
        "negative_evidence": {"id": "negative", "status": "passing"},
    }
    base_row = {
        **fixture,
        "fixture": "diagnostic_fixture",
        "category": "unsupported-by-design",
        "notes": "diagnostic behavior is supported",
    }
    with tempfile.TemporaryDirectory(prefix="sifr-rust-interop-compat-") as raw_root:
        repo_root = Path(raw_root)
        test_path = repo_root / "crates" / "sifr_driver" / "src" / "evidence.rs"
        test_path.parent.mkdir(parents=True)
        test_path.write_text(
            "#[test]\nfn positive_test() {}\n#[test]\nfn negative_test() {}\n",
            encoding="utf-8",
        )
        profiles = _self_test_profiles()
        manifest = _self_test_manifest()
        control_failures: list[str] = []
        _validate_row(
            control_failures,
            base_row,
            {"diagnostic_fixture": fixture},
            {"diagnostic_fixture": manifest},
            profiles,
            repo_root,
            {},
            set(),
            set(),
            set(),
        )
        if control_failures:
            print(
                "rust interop compatibility matrix self-test error: "
                f"valid rationale was rejected: {control_failures}",
                file=sys.stderr,
            )
            return 1

        cases = (
            (
                "missing rationale",
                {key: value for key, value in base_row.items() if key != "diagnostic_crate_rationale"},
                manifest,
                "diagnostic_fixture: diagnostic_crate_rationale must match fixture matrix",
            ),
            (
                "mismatched rationale",
                {
                    **base_row,
                    "diagnostic_crate_rationale": {
                        **rationale,
                        "purpose": "mismatched rationale",
                    },
                },
                manifest,
                "diagnostic_fixture: diagnostic_crate_rationale must match fixture matrix",
            ),
            (
                "claimed row missing provenance",
                base_row,
                {
                    **manifest,
                    "evidence": {
                        **manifest["evidence"],
                        "negative": {
                            key: value
                            for key, value in manifest["evidence"]["negative"].items()
                            if key != "validation"
                        },
                    },
                },
                "evidence.negative.validation is required",
            ),
        )
        for name, row, case_manifest, expected in cases:
            failures: list[str] = []
            _validate_row(
                failures,
                row,
                {"diagnostic_fixture": fixture},
                {"diagnostic_fixture": case_manifest},
                profiles,
                repo_root,
                {},
                set(),
                set(),
                set(),
            )
            if not any(expected in failure for failure in failures):
                print(
                    f"rust interop compatibility matrix self-test error: {name} passed",
                    file=sys.stderr,
                )
                return 1
        compile_fixture = {**fixture, "execution_kind": "contract-only"}
        compile_row = {
            **base_row,
            "execution_kind": "contract-only",
            "notes": "compile-time behavior is supported",
        }
        scope_failures: list[str] = []
        _validate_row(
            scope_failures,
            compile_row,
            {"diagnostic_fixture": compile_fixture},
            {"diagnostic_fixture": manifest},
            profiles,
            repo_root,
            {},
            set(),
            set(),
            set(),
        )
        if not any("contract-only row notes must say contract-only" in failure for failure in scope_failures):
            print(
                "rust interop compatibility matrix self-test error: "
                "contract-only note overclaim passed",
                file=sys.stderr,
            )
            return 1
    print(f"rust interop compatibility matrix self-test ok: cases={len(cases) + 2}")
    return 0


def _self_test_profiles() -> dict[str, dict[str, Any]]:
    profiles: dict[str, dict[str, Any]] = {}
    for name in ("create-pr", "merge", "nightly", "release"):
        profiles[name] = {
            "legacy_facade": {"crate_tests": "smoke" if name == "create-pr" else "full"},
            "crate_test_membership": {
                "suites": [
                    {
                        "id": "driver",
                        "package": "sifr_driver",
                        "command": ["test", "-p", "sifr_driver", "--lib"],
                        "modes": ["smoke", "full"],
                        "status": "blocking",
                    }
                ]
            },
        }
    return profiles


def _self_test_manifest() -> dict[str, Any]:
    def record(side: str) -> dict[str, Any]:
        return {
            "id": side,
            "status": "passing",
            "validation": {
                "profile": "create-pr",
                "step": "crate_tests",
                "suite_id": "driver",
                "test_file": "crates/sifr_driver/src/evidence.rs",
                "test_name": f"{side}_test",
            },
        }

    return {
        "schema_version": 2,
        "id": "diagnostic_fixture",
        "evidence": {
            "positive": record("positive"),
            "negative": record("negative"),
        },
    }


if __name__ == "__main__":
    raise SystemExit(main())
