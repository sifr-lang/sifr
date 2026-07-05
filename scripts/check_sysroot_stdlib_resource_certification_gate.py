#!/usr/bin/env python3
"""Guard stdlib resource migrations behind Rust interop certification."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
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
    matrix = json.loads(MATRIX_PATH.read_text(encoding="utf-8"))
    return _run(matrix)


def _run(matrix: dict[str, Any]) -> int:
    failures = _validate(matrix)

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


def _validate(matrix: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    rows_by_id = _rows_by_id(failures, matrix)

    for surface_id, required_rows in SURFACE_CERTIFICATION_ROWS.items():
        for row_id in required_rows:
            row = rows_by_id.get(row_id)
            if row is None:
                failures.append(f"{surface_id}: missing Rust interop compatibility matrix row {row_id}")
                continue

            category = str(row.get("category", ""))
            if category != FUTURE_OWNED:
                failures.append(
                    f"{surface_id}: {row_id} is {category or '<missing category>'}; "
                    "update this gate when resource certification lands"
                )
            future_owner = row.get("future_owner")
            if future_owner != CERTIFICATION_ISSUE:
                failures.append(
                    f"{surface_id}: {row_id} future_owner must remain {CERTIFICATION_ISSUE}"
                )

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


def _self_test() -> int:
    base_matrix = {
        "rows": [
            {"id": row_id, "category": FUTURE_OWNED, "future_owner": CERTIFICATION_ISSUE}
            for row_id in sorted({row for rows in SURFACE_CERTIFICATION_ROWS.values() for row in rows})
        ]
    }

    if _validate(base_matrix):
        print("self-test seed should pass", file=sys.stderr)
        return 1

    certified_matrix = json.loads(json.dumps(base_matrix))
    certified_matrix["rows"][0]["category"] = "supported"
    if not any("update this gate when resource certification lands" in failure for failure in _validate(certified_matrix)):
        print("self-test supported resource row was not rejected", file=sys.stderr)
        return 1

    wrong_owner_matrix = json.loads(json.dumps(base_matrix))
    wrong_owner_matrix["rows"][0]["future_owner"] = "plans/issues/active/other.md"
    if not any("future_owner must remain" in failure for failure in _validate(wrong_owner_matrix)):
        print("self-test wrong future owner was not rejected", file=sys.stderr)
        return 1

    completed_matrix = json.loads(json.dumps(base_matrix))
    for row in completed_matrix["rows"]:
        row["category"] = "supported"
    if not any(
        "expected at least one runtime/resource compatibility row" in failure
        for failure in _validate(completed_matrix)
    ):
        print("self-test missing future-owned backstop was not rejected", file=sys.stderr)
        return 1

    print("sysroot stdlib resource certification gate self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
