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
    observed_variants = 0
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
        observed_variants += _validate_cases(suite, area=area, result_path=result_path)
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
        or summary["total_variants"] != observed_variants
    ):
        raise AreaResultError(
            f"{area} result JSON contains invalid blocking summary: {result_path}"
        )
    return payload


def _validate_cases(suite: dict[str, Any], *, area: str, result_path: Path) -> int:
    def reject(reason: str) -> None:
        raise AreaResultError(
            f"{area} suite {suite['name']} has {reason}: {result_path}"
        )

    cases = suite.get("cases")
    if not isinstance(cases, list) or not cases:
        reject("missing case evidence")
    count = 0
    failures = 0
    for case in cases:
        variants = case.get("variants") if isinstance(case, dict) else None
        if not isinstance(variants, list) or not variants:
            reject("missing variant evidence")
        for variant in variants:
            if not isinstance(variant, dict):
                reject("invalid variant evidence")
            status = variant.get("status")
            if status not in ("pass", "fail", "skip"):
                reject("invalid variant status")
            if suite["blocking"] and status != "pass":
                reject(f"unexecuted or failed blocking variant ({status})")
            if status == "pass":
                if variant.get("mismatches") or variant.get("truncated"):
                    reject("contradictory passing variant")
                if "process_cause" in variant and variant["process_cause"] != "exit":
                    reject("abnormal passing process")
                if "actual_exit_code" in variant and type(variant["actual_exit_code"]) is not int:
                    reject("invalid passing exit code")
                if "expected_exit_code" in variant and (
                    type(variant["expected_exit_code"]) is not int
                    or variant.get("actual_exit_code") != variant["expected_exit_code"]
                ):
                    reject("mismatched passing exit code")
            count += 1
            failures += status == "fail"
    if suite["total_variants"] != count or suite["total_failures"] != failures:
        reject("case totals inconsistent with variant evidence")
    return count


def _is_positive_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value > 0


def _is_zero_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value == 0


def _is_nonnegative_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0
