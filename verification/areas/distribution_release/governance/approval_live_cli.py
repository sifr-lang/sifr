"""Fetch current GitHub approval authority; no caller-supplied history or waiver."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path
from typing import Any

from .approval_policy import ENVIRONMENT, REPOSITORY, resolve_live_approval
from .common import GovernanceError, require_sha256, sha256_file


def _github(endpoint: str) -> Any:
    result = subprocess.run(["gh", "api", endpoint], text=True, capture_output=True)
    if result.returncode:
        raise GovernanceError("could not read GitHub approval authority")
    try:
        return json.loads(result.stdout)
    except ValueError as exc:
        raise GovernanceError("invalid GitHub approval response") from exc


def resolve_publication_approvers(args: argparse.Namespace) -> None:
    if args.environment != ENVIRONMENT or args.repository != REPOSITORY:
        raise GovernanceError("live approval requires sifr-lang/sifr stable-release")
    # Prevent a direct script invocation from selecting a different run/attempt.
    for name, expected in (
        ("GITHUB_ACTIONS", "true"), ("GITHUB_REPOSITORY", args.repository),
        ("GITHUB_RUN_ID", str(args.run_id)),
        ("GITHUB_RUN_ATTEMPT", str(args.run_attempt)),
        ("GITHUB_TRIGGERING_ACTOR", args.initiator),
    ):
        if os.environ.get(name) != expected:
            raise GovernanceError(f"{name} does not match current publication")
    require_sha256(args.expected_evidence_sha256, "release evidence SHA-256")
    if sha256_file(Path(args.evidence)) != args.expected_evidence_sha256:
        raise GovernanceError("release evidence digest does not match approval context")
    run_endpoint = f"repos/{args.repository}/actions/runs/{args.run_id}"
    decision = resolve_live_approval(
        _github(f"{run_endpoint}/approvals"),
        configuration=_github(f"repos/{args.repository}/environments/{ENVIRONMENT}"),
        run=_github(run_endpoint), repository=args.repository,
        operation=args.operation, initiator=args.initiator,
        run_id=args.run_id, run_attempt=args.run_attempt,
        evidence_sha256=args.expected_evidence_sha256,
    )
    print(json.dumps(decision if args.include_policy else decision["approvers"],
                     separators=(",", ":")))
