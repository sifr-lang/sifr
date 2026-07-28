#!/usr/bin/env python3
"""Recompute protected incident prepare evidence before production mutation."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError,
    canonical_json_bytes,
    load_json_bytes_strict,
    require_sha256,
    sha256_bytes,
)
from verification.areas.distribution_release.governance.incident_prepare import (  # noqa: E402
    materialize_incident_prepare,
    validate_incident_prepare_summary,
)


def revalidate_incident_publication(args: argparse.Namespace) -> dict[str, object]:
    """Require protected revalidation to reproduce reviewer-visible bytes."""
    require_sha256(args.expected_summary_sha256, "expected_summary_sha256")
    summary_bytes = args.prepare_summary.read_bytes()
    if sha256_bytes(summary_bytes) != args.expected_summary_sha256:
        raise GovernanceError("prepare summary digest changed before publication")
    summary = validate_incident_prepare_summary(
        load_json_bytes_strict(
            summary_bytes,
            source=str(args.prepare_summary),
            require_canonical=True,
        )
    )
    expected_incident = {
        "commit": args.incident_commit,
        "path": args.incident_path,
        "incident_id": summary["incident"]["incident_id"],
        "request_sha256": args.expected_request_sha256,
        "withdrawal_evidence_sha256": summary["incident"][
            "withdrawal_evidence_sha256"
        ],
    }
    if (
        summary["operation"] != args.operation
        or summary["mode"] != args.mode
        or summary["incident"] != expected_incident
    ):
        raise GovernanceError("protected inputs do not match the prepare summary")
    recomputed = materialize_incident_prepare(
        operation=args.operation,
        mode=args.mode,
        governance_root=args.governance_root,
        incident_root=args.incident_root,
        incident_commit=args.incident_commit,
        incident_path=args.incident_path,
        expected_request_sha256=args.expected_request_sha256,
        live_index_path=args.live_index,
        snapshot_root=args.snapshot_root,
        proposed_generation=summary["next_generation"],
        candidate_root=args.candidate_root,
        candidate_commit=args.candidate_commit,
        candidate_path=args.candidate_path,
        expected_plan_sha256=args.expected_plan_sha256,
        source_root=args.source_root,
        artifact_root=args.artifact_root,
    )
    if canonical_json_bytes(recomputed) != summary_bytes:
        raise GovernanceError(
            "protected revalidation did not reproduce the prepare summary"
        )
    return recomputed


def parse_args(parser: argparse.ArgumentParser) -> argparse.Namespace:
    parser.add_argument("--prepare-summary", type=Path, required=True)
    parser.add_argument("--expected-summary-sha256", required=True)
    parser.add_argument(
        "--operation",
        required=True,
        choices=("rollback", "incident-roll-forward"),
    )
    parser.add_argument("--mode", required=True, choices=("initial", "resume"))
    parser.add_argument("--governance-root", type=Path, required=True)
    parser.add_argument("--incident-root", type=Path, required=True)
    parser.add_argument("--incident-commit", required=True)
    parser.add_argument("--incident-path", required=True)
    parser.add_argument("--expected-request-sha256", required=True)
    parser.add_argument("--live-index", type=Path, required=True)
    parser.add_argument("--snapshot-root", type=Path, required=True)
    parser.add_argument("--candidate-root", type=Path)
    parser.add_argument("--candidate-commit", default="")
    parser.add_argument("--candidate-path", default="")
    parser.add_argument("--expected-plan-sha256", default="")
    parser.add_argument("--source-root", type=Path)
    parser.add_argument("--artifact-root", type=Path)
    return parser.parse_args()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    args = parse_args(parser)
    try:
        summary = revalidate_incident_publication(args)
    except (GovernanceError, OSError) as exc:
        parser.error(str(exc))
    print(
        "incident publication revalidation ok: "
        f"operation={summary['operation']} incident={summary['incident']['incident_id']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
