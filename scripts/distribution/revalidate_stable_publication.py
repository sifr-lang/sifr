#!/usr/bin/env python3
"""Recompute protected stable prepare evidence before any production mutation."""

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
from verification.areas.distribution_release.governance.generation import (  # noqa: E402
    allocate_next_generation,
)
from verification.areas.distribution_release.governance.stable_prepare import (  # noqa: E402
    materialize_stable_prepare,
    validate_stable_prepare_summary,
)


def revalidate_stable_publication(
    *,
    prepare_summary_path: Path,
    expected_summary_sha256: str,
    operation: str,
    mode: str,
    evidence_root: Path,
    evidence_commit: str,
    candidate_path: str,
    expected_plan_sha256: str,
    source_root: Path,
    live_index_path: Path,
    snapshot_root: Path,
    artifact_root: Path,
) -> dict[str, object]:
    """Require protected revalidation to reproduce the reviewer-visible bytes."""
    require_sha256(expected_summary_sha256, "expected_summary_sha256")
    try:
        summary_bytes = prepare_summary_path.read_bytes()
    except OSError as exc:
        raise GovernanceError(f"could not read prepare summary: {exc}") from exc
    if sha256_bytes(summary_bytes) != expected_summary_sha256:
        raise GovernanceError("prepare summary digest changed before publication")
    summary = validate_stable_prepare_summary(
        load_json_bytes_strict(
            summary_bytes,
            source=str(prepare_summary_path),
            require_canonical=True,
        )
    )
    if (
        summary["operation"] != operation
        or summary["mode"] != mode
        or summary["evidence"]
        != {
            "commit": evidence_commit,
            "candidate_path": candidate_path,
            "plan_sha256": expected_plan_sha256,
        }
    ):
        raise GovernanceError("protected inputs do not match the prepare summary")
    proposed_generation = summary["mutation"]["proposed_index"]["generation"]
    if (
        allocate_next_generation(
            live_index_path=live_index_path,
            snapshot_root=snapshot_root,
        )
        != proposed_generation
    ):
        raise GovernanceError("retained generations changed after prepare")
    recomputed = materialize_stable_prepare(
        operation=operation,
        mode=mode,
        evidence_root=evidence_root,
        evidence_commit=evidence_commit,
        candidate_path=candidate_path,
        expected_plan_sha256=expected_plan_sha256,
        source_root=source_root,
        live_index_path=live_index_path,
        artifact_root=artifact_root,
        proposed_generation=proposed_generation,
    )
    if canonical_json_bytes(recomputed) != summary_bytes:
        raise GovernanceError(
            "protected revalidation did not reproduce the prepare summary"
        )
    return recomputed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare-summary", type=Path, required=True)
    parser.add_argument("--expected-summary-sha256", required=True)
    parser.add_argument(
        "--operation",
        required=True,
        choices=("ga-activation", "normal"),
    )
    parser.add_argument("--mode", required=True, choices=("initial", "resume"))
    parser.add_argument("--evidence-root", type=Path, required=True)
    parser.add_argument("--evidence-commit", required=True)
    parser.add_argument("--candidate-path", required=True)
    parser.add_argument("--expected-plan-sha256", required=True)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--live-index", type=Path, required=True)
    parser.add_argument("--snapshot-root", type=Path, required=True)
    parser.add_argument("--artifact-root", type=Path, required=True)
    args = parser.parse_args()
    try:
        summary = revalidate_stable_publication(
            prepare_summary_path=args.prepare_summary,
            expected_summary_sha256=args.expected_summary_sha256,
            operation=args.operation,
            mode=args.mode,
            evidence_root=args.evidence_root,
            evidence_commit=args.evidence_commit,
            candidate_path=args.candidate_path,
            expected_plan_sha256=args.expected_plan_sha256,
            source_root=args.source_root,
            live_index_path=args.live_index,
            snapshot_root=args.snapshot_root,
            artifact_root=args.artifact_root,
        )
    except GovernanceError as exc:
        parser.error(str(exc))
    print(
        "stable publication revalidation ok: "
        f"operation={summary['operation']} version={summary['version']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
