"""Explicit ownership and executed evidence for strict demo freshness."""
from __future__ import annotations

from typing import Any

from .profile_results import AreaResultError

FRESHNESS_SUITES = frozenset({"freshness", "representative", "full"})
STRICT_FLAG = "--require-fresh-demo-companions"


def requires_strict_freshness(profile: dict[str, Any], area: str) -> bool:
    return any(selection["area"] == area
               and selection.get("require_fresh_demo_companions") is True
               for selection in profile["selected_areas"])


def validate_freshness_ownership(profile: dict[str, Any]) -> None:
    owners = []
    for selection in profile["selected_areas"]:
        if "require_fresh_demo_companions" not in selection:
            continue
        if (selection["area"] != "generated_code_quality"
                or selection["require_fresh_demo_companions"] is not True
                or not FRESHNESS_SUITES.intersection(selection["suites"])):
            raise ValueError("strict demo freshness requires a selected GCQ freshness gate")
        owners.append(selection)
    standalone = "demo-emitted-freshness" in profile["guardrail_steps"]
    if len(owners) + int(standalone) > 1:
        raise ValueError("strict demo freshness must have exactly one owner")
    if profile["name"] in {"create-pr", "merge", "nightly", "release"}:
        if not owners and not standalone:
            raise ValueError("delivery profile omits strict demo freshness")


def validate_strict_freshness_evidence(payload: dict[str, Any]) -> None:
    variants = [variant for suite in payload["suites"]
                for case in suite["cases"] for variant in case["variants"]
                if variant.get("label") == "freshness"]
    if not variants:
        raise AreaResultError("strict demo freshness has no executed gate")
    for variant in variants:
        detail = variant.get("case_evidence")
        if (variant.get("status") != "pass"
                or STRICT_FLAG not in variant.get("argv", [])
                or not isinstance(detail, dict)
                or detail.get("checks") != {"strict": "pass", "debt": "pass"}):
            raise AreaResultError("strict demo freshness lacks passing policy evidence")
