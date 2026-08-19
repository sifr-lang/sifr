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
    for suite in raw_suites:
        if (
            not isinstance(suite, dict)
            or suite.get("blocking") is not True
            or not _is_positive_int(suite.get("total_variants"))
            or not _is_zero_int(suite.get("total_failures"))
        ):
            raise AreaResultError(
                f"{area} result JSON contains invalid suite evidence: {result_path}"
            )
    actual_suites = {
        str(suite.get("name"))
        for suite in raw_suites
        if isinstance(suite, dict) and isinstance(suite.get("name"), str)
    }
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
