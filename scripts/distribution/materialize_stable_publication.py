#!/usr/bin/env python3
"""Stage protected stable publication files or materialize final sign-off."""

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
from verification.areas.distribution_release.governance.stable_publish import (  # noqa: E402
    materialize_stable_signoff,
    stage_stable_publication,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    stage = commands.add_parser("stage")
    stage.add_argument("--prepare-summary", type=Path, required=True)
    stage.add_argument("--qualification-index", type=Path, required=True)
    stage.add_argument("--artifact-root", type=Path, required=True)
    stage.add_argument("--plan", type=Path, required=True)
    stage.add_argument("--dispatchers", type=Path, required=True)
    stage.add_argument("--out", type=Path, required=True)

    signoff = commands.add_parser("signoff")
    signoff.add_argument("--prepare-summary", type=Path, required=True)
    signoff.add_argument("--release-assets", type=Path, required=True)
    signoff.add_argument("--site-facts", type=Path, required=True)
    signoff.add_argument("--site-run", type=Path, required=True)
    signoff.add_argument("--smoke", type=Path, required=True)
    signoff.add_argument("--run-id", type=int, required=True)
    signoff.add_argument("--initiator", required=True)
    signoff.add_argument("--approver", required=True)
    signoff.add_argument(
        "--approval-mode",
        required=True,
        choices=("distinct-reviewer", "single-maintainer-waiver"),
    )
    signoff.add_argument("--approval-waiver-sha256", required=True)
    signoff.add_argument("--out", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "stage":
            stage_stable_publication(
                prepare_summary_path=args.prepare_summary,
                qualification_index_path=args.qualification_index,
                artifact_root=args.artifact_root,
                plan_path=args.plan,
                dispatcher_root=args.dispatchers,
                output_root=args.out,
            )
        else:
            signoff = materialize_stable_signoff(
                prepare_summary_path=args.prepare_summary,
                release_assets_root=args.release_assets,
                site_facts_path=args.site_facts,
                site_run_path=args.site_run,
                smoke_root=args.smoke,
                run_id=args.run_id,
                initiator=args.initiator,
                approver=args.approver,
                approval_policy={
                    "mode": args.approval_mode,
                    "waiver_sha256": args.approval_waiver_sha256,
                },
            )
            write_canonical_json(args.out, signoff, refuse_existing=True)
    except GovernanceError as exc:
        parser = argparse.ArgumentParser(description=__doc__)
        parser.error(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
