#!/usr/bin/env python3

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ScorecardPaths:
    baseline_results: Path
    wave_results: Path
    taxonomy: Path
    backlog: Path


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def index_results_by_id(results: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    return {result["id"]: result for result in results}


def normalize_status_counts(summary: dict[str, Any]) -> dict[str, int]:
    return {key: int(value) for key, value in summary["status_counts"].items()}


def collect_case_labels(results: dict[str, dict[str, Any]]) -> dict[str, str]:
    return {case_id: payload["fixture_slug"] for case_id, payload in results.items()}


def carry_forward_target_phase(proposed_target_phase: str) -> str:
    if proposed_target_phase == "phase31":
        return "phase32"
    return proposed_target_phase


def build_scorecard(paths: ScorecardPaths) -> dict[str, Any]:
    baseline = load_json(paths.baseline_results)
    wave = load_json(paths.wave_results)
    taxonomy = load_json(paths.taxonomy)
    backlog = load_json(paths.backlog)

    baseline_results = index_results_by_id(baseline["results"])
    wave_results = index_results_by_id(wave["results"])
    case_labels = collect_case_labels(wave_results)

    fixed_case_ids = sorted(
        case_id
        for case_id, baseline_case in baseline_results.items()
        if baseline_case["status"] != "PASS" and wave_results[case_id]["status"] == "PASS"
    )
    remaining_failure_ids = sorted(
        case_id for case_id, result in wave_results.items() if result["status"] != "PASS"
    )

    bucket_lookup = {bucket["bucket_id"]: bucket for bucket in taxonomy["buckets"]}
    backlog_lookup = {entry["bucket_id"]: entry for entry in backlog["entries"]}

    baseline_bucket_breakdown = []
    wave_bucket_breakdown = []
    resolved_buckets = []
    unresolved_handoff = []
    for bucket in taxonomy["buckets"]:
        remaining_ids = sorted(
            case_id for case_id in bucket["case_ids"] if case_id in set(remaining_failure_ids)
        )
        resolved_ids = sorted(
            case_id for case_id in bucket["case_ids"] if case_id not in set(remaining_failure_ids)
        )

        baseline_bucket_breakdown.append(
            {
                "bucket_id": bucket["bucket_id"],
                "title": bucket["title"],
                "layer": bucket["layer"],
                "case_count": int(bucket["case_count"]),
            }
        )

        if remaining_ids:
            entry = backlog_lookup[bucket["bucket_id"]]
            unresolved_handoff.append(
                {
                    "backlog_item_id": entry["item_id"],
                    "bucket_id": bucket["bucket_id"],
                    "title": bucket["title"],
                    "remaining_case_count": len(remaining_ids),
                    "remaining_case_ids": remaining_ids,
                    "remaining_cases": [case_labels[case_id] for case_id in remaining_ids],
                    "owner": entry["owner"],
                    "priority": entry["priority"],
                    "item_type": entry["item_type"],
                    "dependencies": entry["dependencies"],
                    "original_target_phase": entry["proposed_target_phase"],
                    "handoff_target_phase": carry_forward_target_phase(
                        entry["proposed_target_phase"]
                    ),
                }
            )
            wave_bucket_breakdown.append(
                {
                    "bucket_id": bucket["bucket_id"],
                    "title": bucket["title"],
                    "layer": bucket["layer"],
                    "remaining_case_count": len(remaining_ids),
                    "remaining_case_ids": remaining_ids,
                }
            )
        elif resolved_ids:
            resolved_buckets.append(
                {
                    "bucket_id": bucket["bucket_id"],
                    "title": bucket["title"],
                    "resolved_case_ids": resolved_ids,
                    "resolved_cases": [case_labels[case_id] for case_id in resolved_ids],
                }
            )

    unresolved_handoff.sort(
        key=lambda item: (-item["remaining_case_count"], item["priority"], item["bucket_id"])
    )
    wave_bucket_breakdown.sort(
        key=lambda item: (-item["remaining_case_count"], item["bucket_id"])
    )
    resolved_buckets.sort(key=lambda item: item["bucket_id"])

    baseline_status_counts = normalize_status_counts(baseline["summary"])
    wave_status_counts = normalize_status_counts(wave["summary"])
    total_cases = int(wave["summary"]["case_count"])
    baseline_passes = baseline_status_counts.get("PASS", 0)
    wave_passes = wave_status_counts.get("PASS", 0)

    return {
        "name": "phase31_scorecard",
        "phase": 31,
        "review_status": "external_review_approved",
        "input_artifacts": {
            "baseline_results": str(paths.baseline_results),
            "wave_results": str(paths.wave_results),
            "taxonomy": str(paths.taxonomy),
            "backlog": str(paths.backlog),
        },
        "totals": {
            "case_count": total_cases,
            "baseline": {
                "status_counts": baseline_status_counts,
                "pass_rate": round(baseline_passes / total_cases, 4),
                "scope_counts": baseline["summary"]["scope_counts"],
            },
            "wave_1": {
                "status_counts": wave_status_counts,
                "pass_rate": round(wave_passes / total_cases, 4),
                "scope_counts": wave["summary"]["scope_counts"],
            },
            "delta": {
                "PASS": wave_passes - baseline_passes,
                "CHECK_ERROR": wave_status_counts.get("CHECK_ERROR", 0)
                - baseline_status_counts.get("CHECK_ERROR", 0),
                "RUN_ERROR": wave_status_counts.get("RUN_ERROR", 0)
                - baseline_status_counts.get("RUN_ERROR", 0),
            },
        },
        "fixed_cases": {
            "count": len(fixed_case_ids),
            "case_ids": fixed_case_ids,
            "cases": [case_labels[case_id] for case_id in fixed_case_ids],
        },
        "category_breakdown": {
            "baseline": baseline_bucket_breakdown,
            "wave_1_remaining": wave_bucket_breakdown,
        },
        "resolved_buckets": resolved_buckets,
        "unresolved_handoff": unresolved_handoff,
        "approval_process": backlog["approval_process"],
        "stale_blocker_policy": backlog["stale_blocker_policy"],
    }


def render_scorecard_markdown(scorecard: dict[str, Any]) -> str:
    totals = scorecard["totals"]
    baseline = totals["baseline"]["status_counts"]
    wave = totals["wave_1"]["status_counts"]
    delta = totals["delta"]

    lines = [
        "# Phase 31 Compatibility Scorecard",
        "",
        f"- Review status: `{scorecard['review_status']}`",
        f"- Seed corpus size: `{totals['case_count']}`",
        f"- Baseline status counts: `PASS={baseline.get('PASS', 0)}`, `CHECK_ERROR={baseline.get('CHECK_ERROR', 0)}`, `RUN_ERROR={baseline.get('RUN_ERROR', 0)}`",
        f"- Wave 1 status counts: `PASS={wave.get('PASS', 0)}`, `CHECK_ERROR={wave.get('CHECK_ERROR', 0)}`, `RUN_ERROR={wave.get('RUN_ERROR', 0)}`",
        f"- Delta: `PASS {delta['PASS']:+d}`, `CHECK_ERROR {delta['CHECK_ERROR']:+d}`, `RUN_ERROR {delta['RUN_ERROR']:+d}`",
        "",
        "## Fixed Cases",
        "",
    ]

    for case_label in scorecard["fixed_cases"]["cases"]:
        lines.append(f"- `{case_label}`")

    lines.extend(["", "## Remaining Buckets", ""])
    for bucket in scorecard["category_breakdown"]["wave_1_remaining"]:
        lines.append(
            f"- `{bucket['bucket_id']}`: `{bucket['remaining_case_count']}` remaining"
        )

    lines.extend(["", "## Unresolved Handoff", ""])
    for item in scorecard["unresolved_handoff"]:
        lines.append(
            f"- `{item['backlog_item_id']}` / `{item['bucket_id']}`: owner=`{item['owner']}`, priority=`{item['priority']}`, handoff_target=`{item['handoff_target_phase']}`, remaining=`{item['remaining_case_count']}`"
        )

    lines.extend(
        [
            "",
            "## Closure Note",
            "",
            "- Phase 31 now has a reproducible baseline, taxonomy, remediation backlog, first remediation wave, and stable scorecard artifacts.",
            "- External review/sign-off is complete, and the phase is approved for closure.",
        ]
    )
    return "\n".join(lines) + "\n"
