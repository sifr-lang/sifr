"""Result document construction for verification reports."""

from __future__ import annotations

from datetime import UTC, datetime
from typing import Any

from .profiles import failure_reproduction_command


def build_result(
    *,
    profile: str,
    status: str,
    cases: list[dict[str, Any]],
    elapsed_ms: int,
) -> dict[str, Any]:
    failures = []
    for case in cases:
        if case.get("status") != "pass":
            case_id = str(case.get("id", "unknown"))
            failures.append(
                {
                    "case_id": case_id,
                    "reproduce": failure_reproduction_command(profile, case_id),
                }
            )
    return {
        "schema_version": 1,
        "profile": profile,
        "status": status,
        "generated_at": datetime.now(UTC).replace(microsecond=0).isoformat(),
        "elapsed_ms": elapsed_ms,
        "cases": cases,
        "failures": failures,
    }
