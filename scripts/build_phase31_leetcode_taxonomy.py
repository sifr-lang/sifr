#!/usr/bin/env python3

from __future__ import annotations

import argparse
from pathlib import Path

from phase31_leetcode_lib import REPO_ROOT, write_json
from phase31_leetcode_taxonomy import (
    SPOT_AUDIT_CASES,
    build_markdown_report,
    build_repro_inventory,
    build_spot_audit,
    build_taxonomy,
    load_manifest_cases,
    load_results,
    write_markdown,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build Phase 31 LeetCode failure taxonomy artifacts from seed results."
    )
    parser.add_argument(
        "--results",
        default="verification/leetcode/phase31_seed_results.json",
        help="Path to the seed results JSON, relative to the repo root.",
    )
    parser.add_argument(
        "--manifest",
        default="verification/leetcode/phase31_seed_corpus.json",
        help="Path to the seed corpus manifest JSON, relative to the repo root.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    results = load_results(REPO_ROOT / args.results)
    manifest_cases = load_manifest_cases(REPO_ROOT / args.manifest)
    classified_payload = build_taxonomy(results, manifest_cases)
    repro_inventory = build_repro_inventory(classified_payload)
    spot_audit = build_spot_audit(classified_payload)
    report_markdown = build_markdown_report(
        classified_payload, repro_inventory, spot_audit
    )

    verification_dir = REPO_ROOT / "verification" / "leetcode"
    taxonomy_json_path = verification_dir / "phase31_failure_taxonomy.json"
    repros_json_path = verification_dir / "phase31_failure_repros.json"
    spot_audit_json_path = verification_dir / "phase31_spot_audit.json"
    spot_audit_cases_path = verification_dir / "phase31_spot_audit_cases.json"
    report_md_path = verification_dir / "phase31_failure_report.md"

    write_json(
        taxonomy_json_path,
        {
            "phase": 31,
            "name": "phase31_failure_taxonomy",
            **classified_payload,
        },
    )
    write_json(
        repros_json_path,
        {
            "phase": 31,
            "name": "phase31_failure_repros",
            "repros": repro_inventory,
        },
    )
    write_json(
        spot_audit_json_path,
        {
            "phase": 31,
            "name": "phase31_spot_audit",
            **spot_audit,
        },
    )
    write_json(
        spot_audit_cases_path,
        {
            "phase": 31,
            "name": "phase31_spot_audit_cases",
            "cases": SPOT_AUDIT_CASES,
        },
    )
    write_markdown(report_md_path, report_markdown)

    print(f"wrote {taxonomy_json_path.relative_to(REPO_ROOT)}")
    print(f"wrote {repros_json_path.relative_to(REPO_ROOT)}")
    print(f"wrote {spot_audit_json_path.relative_to(REPO_ROOT)}")
    print(f"wrote {spot_audit_cases_path.relative_to(REPO_ROOT)}")
    print(f"wrote {report_md_path.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
