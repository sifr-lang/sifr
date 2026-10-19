"""Explicit correctness selection without a second executable profile catalogue."""

from __future__ import annotations

import copy
from typing import Any

from .errors import VerificationError

BENCHMARK_SUITES = frozenset({"smoke", "representative", "full"})


def scope_name(correctness_only: bool) -> str:
    return "correctness" if correctness_only else "full"


def evidence_name(profile_name: str, correctness_only: bool) -> str:
    return f"{profile_name}-correctness" if correctness_only else profile_name


def select_execution_scope(profile: dict[str, Any], correctness_only: bool) -> dict[str, Any]:
    if not correctness_only:
        return profile
    if profile["name"] not in {"create-pr", "merge"}:
        raise VerificationError("--correctness-only supports create-pr and merge only")
    selected = copy.deepcopy(profile)
    # Keep performance-area syntax/cache/policy correctness assertions. Only the
    # named benchmark captures are outside the local correctness contract.
    for area in selected["selected_areas"]:
        if area["area"] == "performance":
            area["suites"] = [s for s in area["suites"] if s not in BENCHMARK_SUITES]
    selected["selected_areas"] = [a for a in selected["selected_areas"] if a["suites"]]
    if not any(a["area"] == "performance" for a in selected["selected_areas"]):
        selected.get("step_budgets", {}).pop("area_performance", None)
    return selected


def scope_metadata(profile: dict[str, Any], correctness_only: bool) -> dict[str, Any]:
    selected = select_execution_scope(profile, correctness_only)
    retained = {(a["area"], s) for a in selected["selected_areas"] for s in a["suites"]}
    return {
        "execution_scope": scope_name(correctness_only),
        "evidence_name": evidence_name(str(profile["name"]), correctness_only),
        "excluded_area_suites": [
            {"area": a["area"], "suite": s, "reason": "benchmark capture"}
            for a in profile["selected_areas"] for s in a["suites"]
            if (a["area"], s) not in retained
        ],
    }
