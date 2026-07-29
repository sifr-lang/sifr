#!/usr/bin/env python3
"""Materialize canonical evidence for one protected schema bootstrap stage."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "distribution_release"
sys.path.insert(0, str(AREA_ROOT))

from governance.common import GovernanceError  # noqa: E402
from governance.schema_bootstrap import materialize_bootstrap_evidence  # noqa: E402


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--stage", required=True, choices=("alpha-assets", "preview-index")
    )
    parser.add_argument("--run-id", required=True, type=int)
    parser.add_argument("--run-attempt", required=True, type=int)
    parser.add_argument("--initiator", required=True)
    parser.add_argument(
        "--approval-mode",
        required=True,
        choices=("distinct-reviewer", "single-maintainer-waiver"),
    )
    parser.add_argument("--approval-waiver-sha256", required=True)
    parser.add_argument("--approvers-json", required=True)
    parser.add_argument("--prepare-summary", required=True)
    parser.add_argument("--legacy-index", required=True)
    parser.add_argument("--alpha-version", required=True)
    parser.add_argument("--alpha-source-commit", required=True)
    parser.add_argument("--alpha-record", required=True)
    parser.add_argument("--alpha-assets", required=True)
    parser.add_argument("--beta-version")
    parser.add_argument("--beta-source-commit")
    parser.add_argument("--beta-record")
    parser.add_argument("--beta-assets")
    parser.add_argument("--index")
    parser.add_argument("--smoke-dir")
    parser.add_argument("--alpha-evidence")
    parser.add_argument("--out", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        approvers = json.loads(args.approvers_json)
        if not isinstance(approvers, list):
            raise GovernanceError("--approvers-json must be a JSON array")
        materialize_bootstrap_evidence(
            stage=args.stage,
            run_id=args.run_id,
            run_attempt=args.run_attempt,
            initiator=args.initiator,
            approval_policy={
                "mode": args.approval_mode,
                "waiver_sha256": args.approval_waiver_sha256,
            },
            approvers=approvers,
            prepare_summary_path=Path(args.prepare_summary),
            legacy_index_path=Path(args.legacy_index),
            alpha_version=args.alpha_version,
            alpha_source_commit=args.alpha_source_commit,
            alpha_record_path=Path(args.alpha_record),
            alpha_assets_dir=Path(args.alpha_assets),
            out=Path(args.out),
            beta_version=args.beta_version,
            beta_source_commit=args.beta_source_commit,
            beta_record_path=Path(args.beta_record) if args.beta_record else None,
            beta_assets_dir=Path(args.beta_assets) if args.beta_assets else None,
            index_path=Path(args.index) if args.index else None,
            smoke_dir=Path(args.smoke_dir) if args.smoke_dir else None,
            alpha_evidence_path=(
                Path(args.alpha_evidence) if args.alpha_evidence else None
            ),
        )
    except (GovernanceError, OSError, json.JSONDecodeError) as exc:
        print(f"schema-bootstrap-evidence: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
