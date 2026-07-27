"""Synthetic Rust-interop result consumed by stable qualification fixtures."""

from __future__ import annotations

from typing import Any


def rust_candidate_result() -> dict[str, Any]:
    suites = []
    for name in (
        "matrix",
        "tiers",
        "compatibility-matrix",
        "stale-drafts",
        "stable-candidate",
    ):
        case_ids = (
            [
                "rust-interop-stable-candidate",
                "rust-interop-stable-candidate-self-test",
            ]
            if name == "stable-candidate"
            else [f"rust-interop-{name}"]
        )
        cases = [
            {
                "id": case_id,
                "variants": [
                    {
                        "actual_exit_code": 0,
                        "expected_exit_code": 0,
                        "mismatches": [],
                        "status": "pass",
                    }
                ],
            }
            for case_id in case_ids
        ]
        suites.append(
            {
                "blocking": True,
                "cases": cases,
                "failed_cases": 0,
                "name": name,
                "total_failures": 0,
                "total_variants": len(cases),
            }
        )
    return {
        "area": "rust_interop",
        "bless": False,
        "manifest": "verification/areas/rust_interop/manifest.json",
        "suites": suites,
        "summary": {
            "blocking_failures": 0,
            "non_blocking_failures": 0,
            "total_failures": 0,
            "total_variants": sum(suite["total_variants"] for suite in suites),
        },
    }
