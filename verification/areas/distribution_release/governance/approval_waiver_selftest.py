"""Fixture and repository checks for the temporary approval waiver."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from verification.json_schema_202012 import validate_instance

from .approval_waiver import (
    CANONICAL_WAIVER_EXPIRY,
    WAIVED_OPERATIONS,
    validate_repository_approval_waiver,
    validate_single_maintainer_waiver,
)
from .common import load_json_strict, sha256_file, write_canonical_json

REPO_ROOT = Path(__file__).resolve().parents[4]
WAIVER_PATH = REPO_ROOT / "plans/releases/single-maintainer-approval-waiver.json"
WAIVER_SCHEMA_PATH = (
    REPO_ROOT
    / "verification/areas/distribution_release/schemas/"
    "single_maintainer_approval_waiver.schema.json"
)
GOVERNANCE_CLI = REPO_ROOT / "scripts/distribution/release_governance.py"


def approval_waiver_fixture() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "repository": "sifr-lang/sifr",
        "environment": "stable-release",
        "owner_login": "yaseralnajjar",
        "allowed_operations": [
            "bootstrap-alpha",
            "bootstrap-index",
            "ga-activation",
        ],
        "expires_at": "2026-08-27T00:00:00Z",
        "reason": "Temporary single-maintainer initial stable approval exception.",
    }


def validate_repository_waiver() -> None:
    waiver = load_json_strict(WAIVER_PATH, require_canonical=True)
    validate_instance(waiver, WAIVER_SCHEMA_PATH)
    assert waiver["repository"] == "sifr-lang/sifr"
    assert waiver["environment"] == "stable-release"
    assert waiver["owner_login"] == "yaseralnajjar"
    assert set(waiver["allowed_operations"]) == WAIVED_OPERATIONS
    assert waiver["expires_at"] == CANONICAL_WAIVER_EXPIRY
    validate_repository_approval_waiver(waiver, require_unexpired=True)
    for operation in sorted(WAIVED_OPERATIONS):
        validate_single_maintainer_waiver(
            waiver,
            repository="sifr-lang/sifr",
            environment="stable-release",
            operation=operation,
            initiator="yaseralnajjar",
            require_unexpired=True,
            now=datetime.now(timezone.utc),
        )
    validate_approval_cli(waiver)


def validate_approval_cli(waiver: dict[str, Any]) -> None:
    owner_approval = {
        "state": "approved",
        "environments": [{"name": "stable-release"}],
        "user": {"login": waiver["owner_login"]},
    }
    reviewer_approval = {
        "state": "approved",
        "environments": [{"name": "stable-release"}],
        "user": {"login": "release-reviewer"},
    }
    with tempfile.TemporaryDirectory() as raw:
        root = Path(raw)
        approvals = root / "approvals.json"
        write_canonical_json(approvals, [owner_approval], refuse_existing=True)
        base = [
            sys.executable,
            str(GOVERNANCE_CLI),
            "resolve-publication-approvers",
            "--approvals",
            str(approvals),
            "--initiator",
            waiver["owner_login"],
            "--repository",
            waiver["repository"],
            "--environment",
            waiver["environment"],
            "--single-maintainer-waiver",
            str(WAIVER_PATH),
            "--expected-waiver-sha256",
            sha256_file(WAIVER_PATH),
            "--include-policy",
        ]
        for operation in sorted(WAIVED_OPERATIONS):
            result = subprocess.run(
                [*base, "--operation", operation],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=True,
            )
            assert json.loads(result.stdout) == {
                "approvers": [waiver["owner_login"]],
                "approval_policy": {
                    "mode": "single-maintainer-waiver",
                    "waiver_sha256": sha256_file(WAIVER_PATH),
                },
            }
        approvals.unlink()
        write_canonical_json(
            approvals,
            [owner_approval, reviewer_approval],
            refuse_existing=True,
        )
        distinct = subprocess.run(
            [*base, "--operation", "ga-activation"],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=True,
        )
        assert json.loads(distinct.stdout) == {
            "approvers": ["release-reviewer"],
            "approval_policy": {
                "mode": "distinct-reviewer",
                "waiver_sha256": "none",
            },
        }
        approvals.unlink()
        write_canonical_json(approvals, [owner_approval], refuse_existing=True)
        for operation in ("normal", "rollback", "incident-roll-forward"):
            rejected = subprocess.run(
                [*base, "--operation", operation],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )
            assert rejected.returncode == 2, (operation, rejected.stderr)
        no_waiver = subprocess.run(
            base[: base.index("--single-maintainer-waiver")],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        assert no_waiver.returncode == 2
        drifted = [*base, "--operation", "ga-activation"]
        drifted[drifted.index(sha256_file(WAIVER_PATH))] = "a" * 64
        assert subprocess.run(
            drifted,
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        ).returncode == 2
        subprocess.run(
            [
                sys.executable,
                str(GOVERNANCE_CLI),
                "validate",
                "--kind",
                "single-maintainer-approval-waiver",
                "--input",
                str(WAIVER_PATH),
                "--require-canonical",
            ],
            cwd=REPO_ROOT,
            check=True,
        )
