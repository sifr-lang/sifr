#!/usr/bin/env python3
"""Run the credential-free stable incident recovery fixture harness."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "distribution_release"
sys.path.insert(0, str(AREA_ROOT))

from governance.common import GovernanceError  # noqa: E402
from governance.incident_fixture import (  # noqa: E402
    check_release_submission_allowed,
    plan_fixture_recovery,
    run_incident_fixture,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("run")
    run.add_argument("--fixture-root", required=True)
    run.add_argument("--live-index", required=True)
    run.add_argument("--governance-release", required=True)
    run.add_argument("--release-assets", required=True)
    run.add_argument("--marketplace-stub", required=True)
    run.add_argument("--extension-metadata", required=True)
    run.add_argument("--site-repo", required=True)
    run.add_argument("--request", required=True)
    run.add_argument("--affected-plan", required=True)
    run.add_argument("--successor-plan", required=True)
    run.add_argument("--mode", required=True, choices=("initial", "resume"))
    run.add_argument("--approver", required=True)
    run.add_argument(
        "--fail-at",
        default="none",
        choices=(
            "none",
            "after-reservation",
            "race-before-index",
            "after-index",
            "site-timeout",
        ),
    )
    check = commands.add_parser("check-submission")
    check.add_argument("--fixture-root", required=True)
    check.add_argument("--submission", required=True, choices=("preview", "stable"))
    recover = commands.add_parser("recover")
    recover.add_argument("--fixture-root", required=True)
    recover.add_argument("--current-version")
    recover.add_argument(
        "--entrypoint",
        required=True,
        choices=("fresh-install", "self-update", "out-of-band"),
    )
    recover.add_argument("--force", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "run":
            result = run_incident_fixture(
                fixture_root=Path(args.fixture_root),
                live_index_path=Path(args.live_index),
                governance_root=Path(args.governance_release),
                release_assets_root=Path(args.release_assets),
                marketplace_path=Path(args.marketplace_stub),
                extension_metadata_path=Path(args.extension_metadata),
                site_root=Path(args.site_repo),
                request_path=Path(args.request),
                affected_plan_path=Path(args.affected_plan),
                successor_plan_path=Path(args.successor_plan),
                mode=args.mode,
                approver=args.approver,
                fail_at=args.fail_at,
            )
            print(json.dumps(result, sort_keys=True, separators=(",", ":")))
            return 3 if result["status"] == "failed" else 0
        elif args.command == "recover":
            result = plan_fixture_recovery(
                fixture_root=Path(args.fixture_root),
                current_version=args.current_version,
                entrypoint=args.entrypoint,
                force=args.force,
            )
            print(json.dumps(result, sort_keys=True, separators=(",", ":")))
        else:
            check_release_submission_allowed(Path(args.fixture_root), args.submission)
            print(f"incident fixture submission allowed: {args.submission}")
    except GovernanceError as exc:
        print(f"incident-fixture: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
