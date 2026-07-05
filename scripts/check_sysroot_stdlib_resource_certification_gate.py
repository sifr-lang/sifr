#!/usr/bin/env python3
"""Guard stdlib resource migrations behind Rust interop certification."""

from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
OWNERSHIP_PATH = REPO_ROOT / "internal_docs" / "stdlib_native_surface_ownership.toml"
MATRIX_PATH = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "rust_interop"
    / "data"
    / "rust_interop_compatibility_matrix.json"
)
CERTIFICATION_ISSUE = "plans/issues/active/rust-interop-runtime-ecosystem-certification.md"
FUTURE_OWNED = "future-owned-by-separate-phase"
MIXED_CERTIFICATION_STATES = {
    "mixed-stateless-supported-resource-state-needs-review",
    "mixed-stateless-supported-runtime-sensitive",
    "mixed-stdlib-leaf-plus-runtime-sensitive",
}
RETAINED_COMPILER_GLUE_SURFACES = (
    "_sifr.runtime",
    "_sifr.task",
    "generated-runtime-integer-glue",
)

SURFACE_CERTIFICATION_ROWS: dict[str, tuple[str, ...]] = {
    "_sifr.crypto": ("opaque_resource_matrix",),
    "_sifr.time": ("async_runtime_reqwest",),
    "_sifr.logging": ("callback_subscription_matrix",),
    "_sifr.fs": ("opaque_resource_matrix",),
    "_sifr.process": ("opaque_resource_matrix", "async_runtime_reqwest"),
    "_sifr.sys": ("opaque_resource_matrix",),
    "_sifr.signal": ("callback_subscription_matrix",),
    "_sifr.net": ("opaque_resource_matrix", "async_runtime_reqwest"),
    "_sifr.tls": ("opaque_resource_matrix", "async_runtime_reqwest"),
    "_sifr.http": ("opaque_resource_matrix", "async_runtime_reqwest"),
    "_sifr.python": (
        "opaque_resource_matrix",
        "callbacks_call_scoped",
        "callback_subscription_matrix",
    ),
}


def main() -> int:
    ownership = tomllib.loads(OWNERSHIP_PATH.read_text(encoding="utf-8"))
    matrix = json.loads(MATRIX_PATH.read_text(encoding="utf-8"))
    return _run(ownership, matrix)


def _run(ownership: dict[str, Any], matrix: dict[str, Any]) -> int:
    failures = _validate(ownership, matrix)

    if failures:
        for failure in failures:
            print(f"sysroot stdlib resource certification gate error: {failure}", file=sys.stderr)
        return 1

    future_runtime_rows = _future_runtime_rows(matrix)
    print(
        "sysroot stdlib resource certification gate: PASS "
        f"(surfaces={len(SURFACE_CERTIFICATION_ROWS)}, future_runtime_rows={len(future_runtime_rows)})"
    )
    return 0


def _validate(ownership: dict[str, Any], matrix: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    rows_by_id = _rows_by_id(failures, matrix)
    surfaces_by_id = _surfaces_by_id(failures, ownership)

    for surface_id, required_rows in SURFACE_CERTIFICATION_ROWS.items():
        surface = surfaces_by_id.get(surface_id)
        if surface is None:
            failures.append(f"{surface_id}: missing native surface ownership entry")
            continue

        future_rows = [
            row_id
            for row_id in required_rows
            if _matrix_category(failures, rows_by_id, row_id) == FUTURE_OWNED
        ]
        if not future_rows:
            continue

        if surface.get("can_move_before_runtime_certification") is not False:
            failures.append(
                f"{surface_id}: must not be movable while matrix rows remain future-owned: "
                + ", ".join(future_rows)
            )

        state = str(surface.get("certification_state", ""))
        if not (
            state.startswith("future-owned-by-runtime-resource-certification")
            or state in MIXED_CERTIFICATION_STATES
            or state == "retained-compiler-language-glue"
        ):
            failures.append(
                f"{surface_id}: certification_state must record runtime/resource retention "
                f"while {', '.join(future_rows)} remain future-owned"
            )

        blocker = str(surface.get("migration_blocker", "")).strip().lower()
        if not blocker or blocker == "none":
            failures.append(
                f"{surface_id}: migration_blocker must name the certification blocker "
                f"while {', '.join(future_rows)} remain future-owned"
            )

    for surface_id in RETAINED_COMPILER_GLUE_SURFACES:
        surface = surfaces_by_id.get(surface_id)
        if surface is None:
            failures.append(f"{surface_id}: missing retained compiler-language surface entry")
            continue
        if surface.get("certification_state") != "retained-compiler-language-glue":
            failures.append(f"{surface_id}: retained compiler glue must keep retained certification_state")
        if surface.get("can_move_before_runtime_certification") is not False:
            failures.append(f"{surface_id}: retained compiler glue must not be movable before certification")

    future_runtime_rows = _future_runtime_rows(matrix)
    if not future_runtime_rows:
        failures.append(
            "expected at least one runtime/resource compatibility row to remain future-owned; "
            "update this guard when resource certification lands"
        )
    return failures


def _future_runtime_rows(matrix: dict[str, Any]) -> list[str]:
    return [
        row_id
        for row_id, row in sorted(_rows_by_id([], matrix).items())
        if row.get("category") == FUTURE_OWNED and row.get("future_owner") == CERTIFICATION_ISSUE
    ]


def _rows_by_id(failures: list[str], matrix: dict[str, Any]) -> dict[str, dict[str, Any]]:
    rows: dict[str, dict[str, Any]] = {}
    for row in matrix.get("rows", []):
        if not isinstance(row, dict):
            failures.append("compatibility matrix rows must be objects")
            continue
        row_id = row.get("id")
        if not isinstance(row_id, str) or not row_id:
            failures.append("compatibility matrix row id is required")
            continue
        rows[row_id] = row
    return rows


def _surfaces_by_id(failures: list[str], ownership: dict[str, Any]) -> dict[str, dict[str, Any]]:
    surfaces: dict[str, dict[str, Any]] = {}
    for surface in ownership.get("surface", []):
        if not isinstance(surface, dict):
            failures.append("native surface ownership entries must be objects")
            continue
        surface_id = surface.get("id")
        if not isinstance(surface_id, str) or not surface_id:
            failures.append("native surface ownership id is required")
            continue
        surfaces[surface_id] = surface
    return surfaces


def _matrix_category(
    failures: list[str],
    rows_by_id: dict[str, dict[str, Any]],
    row_id: str,
) -> str:
    row = rows_by_id.get(row_id)
    if row is None:
        failures.append(f"{row_id}: missing Rust interop compatibility matrix row")
        return ""
    return str(row.get("category", ""))


def _self_test() -> int:
    base_ownership = {
        "surface": [
            {
                "id": surface_id,
                "certification_state": "future-owned-by-runtime-resource-certification",
                "can_move_before_runtime_certification": False,
                "migration_blocker": "certification evidence required",
            }
            for surface_id in SURFACE_CERTIFICATION_ROWS
        ]
        + [
            {
                "id": surface_id,
                "certification_state": "retained-compiler-language-glue",
                "can_move_before_runtime_certification": False,
                "migration_blocker": "final retained allowlist decision",
            }
            for surface_id in RETAINED_COMPILER_GLUE_SURFACES
        ]
    }
    base_matrix = {
        "rows": [
            {"id": row_id, "category": FUTURE_OWNED, "future_owner": CERTIFICATION_ISSUE}
            for row_id in sorted({row for rows in SURFACE_CERTIFICATION_ROWS.values() for row in rows})
        ]
    }

    if _validate(base_ownership, base_matrix):
        print("self-test seed should pass", file=sys.stderr)
        return 1

    movable_ownership = json.loads(json.dumps(base_ownership))
    movable_ownership["surface"][0]["can_move_before_runtime_certification"] = True
    if not any("must not be movable" in failure for failure in _validate(movable_ownership, base_matrix)):
        print("self-test movable future-owned surface was not rejected", file=sys.stderr)
        return 1

    retained_ownership = json.loads(json.dumps(base_ownership))
    retained_ownership["surface"][-1]["can_move_before_runtime_certification"] = True
    if not any(
        "retained compiler glue must not be movable" in failure
        for failure in _validate(retained_ownership, base_matrix)
    ):
        print("self-test movable retained compiler glue was not rejected", file=sys.stderr)
        return 1

    completed_matrix = json.loads(json.dumps(base_matrix))
    for row in completed_matrix["rows"]:
        row["category"] = "supported"
    if not any(
        "expected at least one runtime/resource compatibility row" in failure
        for failure in _validate(base_ownership, completed_matrix)
    ):
        print("self-test missing future-owned backstop was not rejected", file=sys.stderr)
        return 1

    print("sysroot stdlib resource certification gate self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
