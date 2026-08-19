"""Strict result checks for canonical selected-area profile steps."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .errors import VerificationError


class AreaResultError(VerificationError):
    """A selected area failed to emit complete passing evidence."""


def validate_area_result(
    result_path: Path,
    *,
    area: str,
    expected_suites: list[str],
) -> dict[str, Any]:
    if not result_path.is_file():
        raise AreaResultError(f"{area} area emitted no result JSON: {result_path}")
    try:
        payload = json.loads(result_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise AreaResultError(f"{area} area emitted invalid result JSON: {result_path}") from exc
    if (
        not isinstance(payload, dict)
        or payload.get("schema_version") != 1
        or payload.get("area") != area
        or payload.get("bless") is not False
    ):
        raise AreaResultError(f"{area} area emitted an invalid result document: {result_path}")
    raw_suites = payload.get("suites")
    if not isinstance(raw_suites, list) or not raw_suites:
        raise AreaResultError(f"{area} result JSON has no suite results: {result_path}")
    suite_names: list[str] = []
    for suite in raw_suites:
        name = suite.get("name") if isinstance(suite, dict) else None
        blocking = suite.get("blocking") if isinstance(suite, dict) else None
        failures = suite.get("total_failures") if isinstance(suite, dict) else None
        if (
            not isinstance(suite, dict)
            or not isinstance(name, str)
            or not name
            or not isinstance(blocking, bool)
            or not _is_positive_int(suite.get("total_variants"))
            or not _is_nonnegative_int(failures)
            or (blocking and failures != 0)
        ):
            raise AreaResultError(
                f"{area} result JSON contains invalid suite evidence: {result_path}"
            )
        suite_names.append(name)
    actual_suites = set(suite_names)
    if len(actual_suites) != len(suite_names):
        raise AreaResultError(f"{area} result JSON contains duplicate suite evidence: {result_path}")
    if actual_suites != set(expected_suites):
        raise AreaResultError(
            f"{area} result JSON suite mismatch: "
            f"expected={sorted(expected_suites)} actual={sorted(actual_suites)}"
        )
    summary = payload.get("summary")
    if (
        not isinstance(summary, dict)
        or not _is_zero_int(summary.get("blocking_failures"))
        or not _is_positive_int(summary.get("total_variants"))
    ):
        raise AreaResultError(
            f"{area} result JSON contains invalid blocking summary: {result_path}"
        )
    return payload


def _is_positive_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value > 0


def _is_zero_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value == 0


def _is_nonnegative_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0
