#!/usr/bin/env python3
"""Stage protected incident publication or materialize final sign-off."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError,
    write_canonical_json,
)
from verification.areas.distribution_release.governance.incident_publish import (  # noqa: E402
    materialize_incident_signoff,
    stage_incident_publication,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    stage = commands.add_parser("stage")
    stage.add_argument("--prepare-summary", type=Path, required=True)
    stage.add_argument("--successor-plan", type=Path, required=True)
    stage.add_argument("--site-plan", type=Path, required=True)
    stage.add_argument("--dispatchers", type=Path, required=True)
    stage.add_argument("--out", type=Path, required=True)
    signoff = commands.add_parser("signoff")
    signoff.add_argument("--prepare-summary", type=Path, required=True)
    signoff.add_argument("--request", type=Path, required=True)
    signoff.add_argument("--withdrawal-evidence", type=Path, required=True)
    signoff.add_argument("--site-facts", type=Path, required=True)
    signoff.add_argument("--site-run", type=Path, required=True)
    signoff.add_argument("--smoke", type=Path, required=True)
    signoff.add_argument("--run-id", type=int, required=True)
    signoff.add_argument("--approver", required=True)
    signoff.add_argument("--release-signoff", type=Path)
    signoff.add_argument("--out", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "stage":
            stage_incident_publication(
                prepare_summary_path=args.prepare_summary,
                successor_plan_path=args.successor_plan,
                site_plan_path=args.site_plan,
                dispatcher_root=args.dispatchers,
                output_root=args.out,
            )
        else:
            signoff = materialize_incident_signoff(
                prepare_summary_path=args.prepare_summary,
                request_path=args.request,
                withdrawal_evidence_path=args.withdrawal_evidence,
                site_facts_path=args.site_facts,
                site_run_path=args.site_run,
                smoke_root=args.smoke,
                run_id=args.run_id,
                approver=args.approver,
                release_signoff_path=args.release_signoff,
            )
            write_canonical_json(args.out, signoff, refuse_existing=True)
    except GovernanceError as exc:
        print(f"incident publication materialization failed: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
