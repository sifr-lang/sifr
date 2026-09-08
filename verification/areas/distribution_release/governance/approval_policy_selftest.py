"""Positive and negative contracts for the one live approval policy."""

from __future__ import annotations

import argparse
import copy
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from unittest.mock import patch

from . import approval_live_cli
from .approval_policy import OPERATIONS, resolve_live_approval
from .common import GovernanceError, sha256_file


def test_live_approval() -> None:
    identity = {"login": "yaseralnajjar", "id": 10493809}
    environment = {
        "name": "stable-release", "can_admins_bypass": False,
        "protection_rules": [{
            "type": "required_reviewers", "prevent_self_review": False,
            "reviewers": [{"type": "User", "reviewer": identity}],
        }],
    }
    run = {
        "id": 123, "run_attempt": 2, "status": "in_progress",
        "repository": {"full_name": "sifr-lang/sifr"},
        "triggering_actor": {"login": "yaseralnajjar"},
    }
    approval = {
        "state": "approved", "user": identity,
        "environments": [{"name": "stable-release"}],
    }
    arguments = dict(
        configuration=environment, run=run, repository="sifr-lang/sifr",
        operation="normal", initiator="yaseralnajjar", run_id=123,
        run_attempt=2, evidence_sha256="a" * 64,
    )
    expected = {
        "approvers": ["yaseralnajjar"],
        "approval_policy": {"mode": "solo-maintainer", "waiver_sha256": "none"},
    }
    for operation in sorted(OPERATIONS):
        assert resolve_live_approval([approval], **{**arguments, "operation": operation}) == expected
    other_initiator = copy.deepcopy(run)
    other_initiator["triggering_actor"]["login"] = "release-operator"
    assert resolve_live_approval([approval], **{
        **arguments, "run": other_initiator, "initiator": "release-operator",
    }) == expected

    def rejected(callback, label):
        try:
            callback()
        except GovernanceError:
            return
        raise AssertionError(f"accepted invalid live approval: {label}")

    for label, history in (
        ("absent approval", []),
        ("non-designated reviewer", [{**approval, "user": {"login": "other", "id": 7}}]),
        ("wrong user ID", [{**approval, "user": {**identity, "id": 7}}]),
        ("rejected review", [{**approval, "state": "rejected"}]),
        ("other environment", [{**approval, "environments": [{"name": "preview-release"}]}]),
    ):
        rejected(lambda: resolve_live_approval(history, **arguments), label)
    for label, field, value in (
        ("different run", "run_id", 124),
        ("different attempt", "run_attempt", 1),
        ("different repository", "repository", "other/repo"),
        ("different initiator", "initiator", "other"),
        ("retired waiver operation", "operation", "single-maintainer-waiver"),
        ("invalid evidence", "evidence_sha256", "none"),
        ("completed run", "run", {**run, "status": "completed"}),
    ):
        rejected(lambda: resolve_live_approval([approval], **{**arguments, field: value}), label)
    for label, mutate in (
        ("admin bypass", lambda v: v.update(can_admins_bypass=True)),
        ("missing bypass setting", lambda v: v.pop("can_admins_bypass")),
        ("self-review forbidden", lambda v: v["protection_rules"][0].update(prevent_self_review=True)),
        ("missing reviewer", lambda v: v["protection_rules"][0].update(reviewers=[])),
        ("team reviewer", lambda v: v["protection_rules"][0]["reviewers"][0].update(type="Team")),
        ("different reviewer", lambda v: v["protection_rules"][0]["reviewers"][0].update(
            reviewer={"login": "other", "id": 7})),
    ):
        changed = copy.deepcopy(environment)
        mutate(changed)
        rejected(lambda: resolve_live_approval([approval], **{
            **arguments, "configuration": changed,
        }), label)

    with tempfile.TemporaryDirectory() as raw:
        evidence = Path(raw) / "summary.json"
        evidence.write_text('{"release_evidence":"exact"}\n')
        args = argparse.Namespace(
            environment="stable-release", repository="sifr-lang/sifr",
            run_id=123, run_attempt=2, operation="normal", initiator="yaseralnajjar",
            evidence=str(evidence), expected_evidence_sha256=sha256_file(evidence),
            include_policy=True,
        )
        runtime = {
            "GITHUB_ACTIONS": "true", "GITHUB_REPOSITORY": "sifr-lang/sifr",
            "GITHUB_RUN_ID": "123", "GITHUB_RUN_ATTEMPT": "2",
            "GITHUB_TRIGGERING_ACTOR": "yaseralnajjar",
        }
        responses = {
            "repos/sifr-lang/sifr/actions/runs/123/approvals": [approval],
            "repos/sifr-lang/sifr/actions/runs/123": run,
            "repos/sifr-lang/sifr/environments/stable-release": environment,
        }
        with patch.dict("os.environ", runtime, clear=True), patch.object(
            approval_live_cli, "_github", side_effect=responses.__getitem__,
        ) as fetch, patch("builtins.print") as output:
            approval_live_cli.resolve_publication_approvers(args)
            assert json.loads(output.call_args.args[0]) == expected
            assert {call.args[0] for call in fetch.call_args_list} == set(responses)
            evidence.write_text('{"release_evidence":"different"}\n')
            fetch.reset_mock()
            rejected(lambda: approval_live_cli.resolve_publication_approvers(args), "changed evidence")
            fetch.assert_not_called()
            args.expected_evidence_sha256 = sha256_file(evidence)
            args.run_attempt = 1
            rejected(lambda: approval_live_cli.resolve_publication_approvers(args), "stale current attempt")
            fetch.assert_not_called()

        root = Path(__file__).resolve().parents[4]
        cli = root / "scripts/distribution/release_governance.py"
        command = [
            sys.executable, str(cli), "resolve-publication-approvers",
            "--initiator", "yaseralnajjar", "--operation", "normal",
            "--run-id", "123", "--run-attempt", "2",
            "--evidence", str(evidence),
            "--expected-evidence-sha256", sha256_file(evidence),
        ]
        for retired in ("--single-maintainer-waiver", "--expected-waiver-sha256", "--approvals"):
            result = subprocess.run([*command, retired, "retired"], capture_output=True, text=True)
            assert result.returncode == 2 and "unrecognized arguments" in result.stderr
